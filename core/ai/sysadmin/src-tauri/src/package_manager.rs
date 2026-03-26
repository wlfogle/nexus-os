// Package Management System - Comprehensive package management for Arch Linux
// Adapted from ArchForgePro and PackageManager modules 
// Uses only relative paths and direct system calls

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fs;
use std::env;
use std::process::{Command, Stdio};
use std::str;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command as TokioCommand;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub architecture: String,
    pub repository: String,
    pub installed: bool,
    pub installed_size: u64,
    pub download_size: u64,
    pub dependencies: Vec<String>,
    pub optional_dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub groups: Vec<String>,
    pub licenses: Vec<String>,
    pub maintainer: Option<String>,
    pub last_modified: Option<String>,
    pub first_submitted: Option<String>,
    pub url: Option<String>,
    pub aur_package: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageOperation {
    pub operation_id: String,
    pub operation_type: OperationType,
    pub packages: Vec<String>,
    pub status: OperationStatus,
    pub progress: f32,
    pub log: Vec<String>,
    pub started_at: u64,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Install,
    Remove,
    Update,
    Upgrade,
    Search,
    Info,
    Clean,
    AurInstall,
    AurUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStats {
    pub total_packages: u32,
    pub installed_packages: u32,
    pub available_updates: u32,
    pub orphaned_packages: u32,
    pub cache_size: u64,
    pub last_update: Option<u64>,
}

pub struct PackageManager {
    pub work_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    
    // Package databases
    pub installed_packages: HashMap<String, PackageInfo>,
    pub available_packages: HashMap<String, PackageInfo>,
    pub aur_packages: HashMap<String, PackageInfo>,
    
    // Operations tracking
    pub active_operations: HashMap<String, PackageOperation>,
    pub operation_history: Vec<PackageOperation>,
    
    // Configuration
    pub pacman_conf: PathBuf,
    pub makepkg_conf: PathBuf,
    pub aur_helper: String,
    pub auto_clean: bool,
    pub parallel_downloads: u32,
}

impl PackageManager {
    pub async fn new_comprehensive() -> Result<Self> {
        info!("📦 Initializing comprehensive package management");
        
        let current_dir = env::current_dir()?;
        let work_dir = current_dir.clone();
        let data_dir = work_dir.join("data").join("packages");
        let cache_dir = work_dir.join("cache").join("packages");
        let config_dir = work_dir.join("config").join("packages");
        
        // Ensure directories exist
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&config_dir)?;
        
        // Detect AUR helper
        let aur_helper = Self::detect_aur_helper().await;
        
        let mut manager = Self {
            work_dir,
            data_dir,
            cache_dir,
            config_dir,
            installed_packages: HashMap::new(),
            available_packages: HashMap::new(),
            aur_packages: HashMap::new(),
            active_operations: HashMap::new(),
            operation_history: Vec::new(),
            pacman_conf: PathBuf::from("/etc/pacman.conf"),
            makepkg_conf: PathBuf::from("/etc/makepkg.conf"),
            aur_helper,
            auto_clean: true,
            parallel_downloads: 5,
        };
        
        // Load package databases
        manager.refresh_package_database().await?;
        manager.load_installed_packages().await?;
        
        info!("✅ Package management initialized with {} installed packages", 
              manager.installed_packages.len());
        
        Ok(manager)
    }
    
    async fn detect_aur_helper() -> String {
        let helpers = ["yay", "paru", "pikaur", "trizen", "yaourt"];
        
        for helper in &helpers {
            if let Ok(output) = TokioCommand::new("which")
                .arg(helper)
                .output()
                .await {
                if output.status.success() {
                    debug!("🔍 Found AUR helper: {}", helper);
                    return helper.to_string();
                }
            }
        }
        
        warn!("⚠️ No AUR helper found, AUR functionality disabled");
        "none".to_string()
    }
    
    pub async fn refresh_package_database(&mut self) -> Result<()> {
        info!("🔄 Refreshing package databases");
        
        let operation_id = Uuid::new_v4().to_string();
        let mut operation = PackageOperation {
            operation_id: operation_id.clone(),
            operation_type: OperationType::Update,
            packages: vec!["database".to_string()],
            status: OperationStatus::Running,
            progress: 0.0,
            log: Vec::new(),
            started_at: chrono::Utc::now().timestamp() as u64,
            completed_at: None,
        };
        
        self.active_operations.insert(operation_id.clone(), operation.clone());
        
        // Update pacman database
        let output = TokioCommand::new("pacman")
            .args(&["-Sy", "--noconfirm"])
            .output()
            .await?;
        
        operation.progress = 50.0;
        if output.status.success() {
            operation.log.push("Pacman database updated successfully".to_string());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            operation.log.push(format!("Pacman update warning: {}", stderr));
        }
        
        // Update AUR database if helper available
        if self.aur_helper != "none" {
            let aur_output = TokioCommand::new(&self.aur_helper)
                .args(&["-Sy", "--noconfirm"])
                .output()
                .await;
            
            operation.progress = 100.0;
            match aur_output {
                Ok(out) if out.status.success() => {
                    operation.log.push(format!("{} database updated successfully", self.aur_helper));
                },
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    operation.log.push(format!("{} update warning: {}", self.aur_helper, stderr));
                },
                Err(e) => {
                    operation.log.push(format!("AUR helper error: {}", e));
                }
            }
        }
        
        operation.status = OperationStatus::Completed;
        operation.completed_at = Some(chrono::Utc::now().timestamp() as u64);
        
        self.active_operations.remove(&operation_id);
        self.operation_history.push(operation);
        
        info!("✅ Package databases refreshed");
        Ok(())
    }
    
    pub async fn load_installed_packages(&mut self) -> Result<()> {
        debug!("📋 Loading installed packages");
        
        let output = TokioCommand::new("pacman")
            .args(&["-Q", "-i"])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to query installed packages"));
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let packages = self.parse_pacman_info(&output_str);
        
        self.installed_packages.clear();
        for package in packages {
            self.installed_packages.insert(package.name.clone(), package);
        }
        
        debug!("📋 Loaded {} installed packages", self.installed_packages.len());
        Ok(())
    }
    
    pub async fn search_packages(&self, query: &str, include_aur: bool) -> Result<Vec<PackageInfo>> {
        info!("🔍 Searching packages: {} (AUR: {})", query, include_aur);
        
        let mut results = Vec::new();
        
        // Search official repositories
        let output = TokioCommand::new("pacman")
            .args(&["-Ss", query])
            .output()
            .await?;
        
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut packages = self.parse_pacman_search(&output_str);
            results.append(&mut packages);
        }
        
        // Search AUR if requested and helper available
        if include_aur && self.aur_helper != "none" {
            let aur_output = TokioCommand::new(&self.aur_helper)
                .args(&["-Ss", query, "--aur"])
                .output()
                .await;
                
            if let Ok(output) = aur_output {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let mut aur_packages = self.parse_aur_search(&output_str);
                    results.append(&mut aur_packages);
                }
            }
        }
        
        debug!("🔍 Found {} packages matching '{}'", results.len(), query);
        Ok(results)
    }
    
    pub async fn get_package_info(&self, package_name: &str, check_aur: bool) -> Result<PackageInfo> {
        debug!("📝 Getting package info: {}", package_name);
        
        // Try installed packages first
        if let Some(package) = self.installed_packages.get(package_name) {
            return Ok(package.clone());
        }
        
        // Try official repositories
        let output = TokioCommand::new("pacman")
            .args(&["-Si", package_name])
            .output()
            .await?;
        
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(package) = self.parse_pacman_info(&output_str).into_iter().next() {
                return Ok(package);
            }
        }
        
        // Try AUR if requested and helper available
        if check_aur && self.aur_helper != "none" {
            let aur_output = TokioCommand::new(&self.aur_helper)
                .args(&["-Si", package_name, "--aur"])
                .output()
                .await;
                
            if let Ok(output) = aur_output {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(package) = self.parse_aur_info(&output_str) {
                        return Ok(package);
                    }
                }
            }
        }
        
        Err(anyhow!("Package '{}' not found", package_name))
    }
    
    pub async fn install_packages(&mut self, packages: Vec<String>, from_aur: bool) -> Result<String> {
        let operation_id = Uuid::new_v4().to_string();
        info!("📦 Installing packages: {:?} (AUR: {})", packages, from_aur);
        
        let mut operation = PackageOperation {
            operation_id: operation_id.clone(),
            operation_type: if from_aur { OperationType::AurInstall } else { OperationType::Install },
            packages: packages.clone(),
            status: OperationStatus::Running,
            progress: 0.0,
            log: Vec::new(),
            started_at: chrono::Utc::now().timestamp() as u64,
            completed_at: None,
        };
        
        self.active_operations.insert(operation_id.clone(), operation.clone());
        
        let command = if from_aur && self.aur_helper != "none" {
            &self.aur_helper
        } else {
            "pacman"
        };
        
        let mut args = if from_aur {
            vec!["-S", "--noconfirm"]
        } else {
            vec!["-S", "--noconfirm"]
        };
        
        for package in &packages {
            args.push(package);
        }
        
        operation.log.push(format!("Starting installation: {} {:?}", command, args));
        
        let output = TokioCommand::new(command)
            .args(&args)
            .output()
            .await?;
        
        operation.progress = 100.0;
        
        if output.status.success() {
            operation.status = OperationStatus::Completed;
            operation.log.push("Installation completed successfully".to_string());
            
            // Refresh installed packages
            self.load_installed_packages().await?;
        } else {
            operation.status = OperationStatus::Failed;
            let stderr = String::from_utf8_lossy(&output.stderr);
            operation.log.push(format!("Installation failed: {}", stderr));
        }
        
        operation.completed_at = Some(chrono::Utc::now().timestamp() as u64);
        
        self.active_operations.remove(&operation_id);
        self.operation_history.push(operation);
        
        Ok(operation_id)
    }
    
    pub async fn remove_packages(&mut self, packages: Vec<String>, remove_deps: bool) -> Result<String> {
        let operation_id = Uuid::new_v4().to_string();
        info!("🗑️ Removing packages: {:?} (deps: {})", packages, remove_deps);
        
        let mut operation = PackageOperation {
            operation_id: operation_id.clone(),
            operation_type: OperationType::Remove,
            packages: packages.clone(),
            status: OperationStatus::Running,
            progress: 0.0,
            log: Vec::new(),
            started_at: chrono::Utc::now().timestamp() as u64,
            completed_at: None,
        };
        
        self.active_operations.insert(operation_id.clone(), operation.clone());
        
        let mut args = vec!["-R", "--noconfirm"];
        if remove_deps {
            args.push("-s"); // Remove dependencies
        }
        
        for package in &packages {
            args.push(package);
        }
        
        operation.log.push(format!("Starting removal: pacman {:?}", args));
        
        let output = TokioCommand::new("pacman")
            .args(&args)
            .output()
            .await?;
        
        operation.progress = 100.0;
        
        if output.status.success() {
            operation.status = OperationStatus::Completed;
            operation.log.push("Removal completed successfully".to_string());
            
            // Refresh installed packages
            self.load_installed_packages().await?;
        } else {
            operation.status = OperationStatus::Failed;
            let stderr = String::from_utf8_lossy(&output.stderr);
            operation.log.push(format!("Removal failed: {}", stderr));
        }
        
        operation.completed_at = Some(chrono::Utc::now().timestamp() as u64);
        
        self.active_operations.remove(&operation_id);
        self.operation_history.push(operation);
        
        Ok(operation_id)
    }
    
    pub async fn upgrade_system(&mut self, include_aur: bool) -> Result<String> {
        let operation_id = Uuid::new_v4().to_string();
        info!("⬆️ Upgrading system (AUR: {})", include_aur);
        
        let mut operation = PackageOperation {
            operation_id: operation_id.clone(),
            operation_type: OperationType::Upgrade,
            packages: vec!["system".to_string()],
            status: OperationStatus::Running,
            progress: 0.0,
            log: Vec::new(),
            started_at: chrono::Utc::now().timestamp() as u64,
            completed_at: None,
        };
        
        self.active_operations.insert(operation_id.clone(), operation.clone());
        
        // First, update databases
        self.refresh_package_database().await?;
        operation.progress = 25.0;
        
        // Upgrade official packages
        operation.log.push("Upgrading official packages...".to_string());
        let output = TokioCommand::new("pacman")
            .args(&["-Su", "--noconfirm"])
            .output()
            .await?;
        
        operation.progress = 50.0;
        
        if output.status.success() {
            operation.log.push("Official packages upgraded successfully".to_string());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            operation.log.push(format!("Official upgrade warning: {}", stderr));
        }
        
        // Upgrade AUR packages if requested and helper available
        if include_aur && self.aur_helper != "none" {
            operation.log.push("Upgrading AUR packages...".to_string());
            let aur_output = TokioCommand::new(&self.aur_helper)
                .args(&["-Su", "--noconfirm", "--aur"])
                .output()
                .await;
            
            operation.progress = 75.0;
            
            match aur_output {
                Ok(out) if out.status.success() => {
                    operation.log.push("AUR packages upgraded successfully".to_string());
                },
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    operation.log.push(format!("AUR upgrade warning: {}", stderr));
                },
                Err(e) => {
                    operation.log.push(format!("AUR upgrade error: {}", e));
                }
            }
        }
        
        operation.progress = 100.0;
        operation.status = OperationStatus::Completed;
        operation.completed_at = Some(chrono::Utc::now().timestamp() as u64);
        
        // Refresh installed packages
        self.load_installed_packages().await?;
        
        self.active_operations.remove(&operation_id);
        self.operation_history.push(operation);
        
        Ok(operation_id)
    }
    
    pub async fn clean_cache(&mut self, clean_all: bool) -> Result<String> {
        let operation_id = Uuid::new_v4().to_string();
        info!("🧹 Cleaning package cache (all: {})", clean_all);
        
        let mut operation = PackageOperation {
            operation_id: operation_id.clone(),
            operation_type: OperationType::Clean,
            packages: vec!["cache".to_string()],
            status: OperationStatus::Running,
            progress: 0.0,
            log: Vec::new(),
            started_at: chrono::Utc::now().timestamp() as u64,
            completed_at: None,
        };
        
        self.active_operations.insert(operation_id.clone(), operation.clone());
        
        let args = if clean_all {
            vec!["-Scc", "--noconfirm"]
        } else {
            vec!["-Sc", "--noconfirm"]
        };
        
        operation.log.push(format!("Cleaning cache: pacman {:?}", args));
        
        let output = TokioCommand::new("pacman")
            .args(&args)
            .output()
            .await?;
        
        operation.progress = 100.0;
        
        if output.status.success() {
            operation.status = OperationStatus::Completed;
            operation.log.push("Cache cleaned successfully".to_string());
        } else {
            operation.status = OperationStatus::Failed;
            let stderr = String::from_utf8_lossy(&output.stderr);
            operation.log.push(format!("Cache cleaning failed: {}", stderr));
        }
        
        operation.completed_at = Some(chrono::Utc::now().timestamp() as u64);
        
        self.active_operations.remove(&operation_id);
        self.operation_history.push(operation);
        
        Ok(operation_id)
    }
    
    pub async fn get_orphaned_packages(&self) -> Result<Vec<PackageInfo>> {
        debug!("🔍 Finding orphaned packages");
        
        let output = TokioCommand::new("pacman")
            .args(&["-Qdt"])
            .output()
            .await?;
        
        if !output.status.success() {
            return Ok(Vec::new()); // No orphaned packages
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut orphaned = Vec::new();
        
        for line in output_str.lines() {
            if let Some(package_name) = line.split_whitespace().next() {
                if let Some(package) = self.installed_packages.get(package_name) {
                    orphaned.push(package.clone());
                }
            }
        }
        
        debug!("🔍 Found {} orphaned packages", orphaned.len());
        Ok(orphaned)
    }
    
    pub async fn get_available_updates(&self) -> Result<Vec<PackageInfo>> {
        debug!("📋 Checking for available updates");
        
        let output = TokioCommand::new("pacman")
            .args(&["-Qu"])
            .output()
            .await?;
        
        if !output.status.success() {
            return Ok(Vec::new()); // No updates available
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut updates = Vec::new();
        
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let current_version = parts[1].to_string();
                let new_version = parts[3].to_string();
                
                if let Some(mut package) = self.installed_packages.get(&name).cloned() {
                    package.version = format!("{} -> {}", current_version, new_version);
                    updates.push(package);
                }
            }
        }
        
        debug!("📋 Found {} available updates", updates.len());
        Ok(updates)
    }
    
    pub async fn get_repository_stats(&self) -> Result<RepositoryStats> {
        debug!("📊 Gathering repository statistics");
        
        let total_output = TokioCommand::new("pacman")
            .args(&["-Sl"])
            .output()
            .await?;
        
        let total_packages = if total_output.status.success() {
            String::from_utf8_lossy(&total_output.stdout).lines().count() as u32
        } else {
            0
        };
        
        let installed_packages = self.installed_packages.len() as u32;
        
        let available_updates = self.get_available_updates().await?.len() as u32;
        
        let orphaned_packages = self.get_orphaned_packages().await?.len() as u32;
        
        // Get cache size
        let cache_size = self.get_cache_size().await.unwrap_or(0);
        
        Ok(RepositoryStats {
            total_packages,
            installed_packages,
            available_updates,
            orphaned_packages,
            cache_size,
            last_update: None, // Would need to read from pacman logs
        })
    }
    
    async fn get_cache_size(&self) -> Result<u64> {
        let cache_dirs = ["/var/cache/pacman/pkg", "/tmp/makepkg"];
        let mut total_size = 0u64;
        
        for cache_dir in &cache_dirs {
            if let Ok(entries) = fs::read_dir(cache_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }
        
        Ok(total_size)
    }
    
    pub fn get_operation_status(&self, operation_id: &str) -> Option<PackageOperation> {
        self.active_operations.get(operation_id).cloned()
            .or_else(|| {
                self.operation_history.iter()
                    .find(|op| op.operation_id == operation_id)
                    .cloned()
            })
    }
    
    pub fn get_active_operations(&self) -> Vec<PackageOperation> {
        self.active_operations.values().cloned().collect()
    }
    
    pub fn get_operation_history(&self, limit: Option<usize>) -> Vec<PackageOperation> {
        let mut history = self.operation_history.clone();
        history.sort_by(|a, b| b.started_at.cmp(&a.started_at)); // Most recent first
        
        if let Some(limit) = limit {
            history.truncate(limit);
        }
        
        history
    }
    
    fn parse_pacman_info(&self, output: &str) -> Vec<PackageInfo> {
        let mut packages = Vec::new();
        let mut current_package = None;
        
        for line in output.lines() {
            if line.starts_with("Name") {
                if let Some(package) = current_package.take() {
                    packages.push(package);
                }
                current_package = Some(PackageInfo {
                    name: line.split(':').nth(1).unwrap_or("").trim().to_string(),
                    version: String::new(),
                    description: String::new(),
                    architecture: String::new(),
                    repository: String::new(),
                    installed: true,
                    installed_size: 0,
                    download_size: 0,
                    dependencies: Vec::new(),
                    optional_dependencies: Vec::new(),
                    conflicts: Vec::new(),
                    provides: Vec::new(),
                    groups: Vec::new(),
                    licenses: Vec::new(),
                    maintainer: None,
                    last_modified: None,
                    first_submitted: None,
                    url: None,
                    aur_package: false,
                });
            } else if let Some(ref mut pkg) = current_package {
                if line.starts_with("Version") {
                    pkg.version = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Description") {
                    pkg.description = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Architecture") {
                    pkg.architecture = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Repository") {
                    pkg.repository = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Installed Size") {
                    if let Some(size_str) = line.split(':').nth(1) {
                        pkg.installed_size = self.parse_size(size_str.trim());
                    }
                } else if line.starts_with("Download Size") {
                    if let Some(size_str) = line.split(':').nth(1) {
                        pkg.download_size = self.parse_size(size_str.trim());
                    }
                } else if line.starts_with("Depends On") {
                    let deps = line.split(':').nth(1).unwrap_or("").trim();
                    if deps != "None" {
                        pkg.dependencies = deps.split_whitespace()
                            .map(|s| s.to_string())
                            .collect();
                    }
                }
            }
        }
        
        if let Some(package) = current_package {
            packages.push(package);
        }
        
        packages
    }
    
    fn parse_pacman_search(&self, output: &str) -> Vec<PackageInfo> {
        let mut packages = Vec::new();
        
        let lines: Vec<&str> = output.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i];
            if line.starts_with("extra/") || line.starts_with("core/") || 
               line.starts_with("community/") || line.starts_with("multilib/") {
                
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name_repo = parts[0];
                    let version = parts[1];
                    
                    let repo_name: Vec<&str> = name_repo.split('/').collect();
                    if repo_name.len() == 2 {
                        let repository = repo_name[0].to_string();
                        let name = repo_name[1].to_string();
                        
                        let description = if i + 1 < lines.len() {
                            lines[i + 1].trim().to_string()
                        } else {
                            String::new()
                        };
                        
                        packages.push(PackageInfo {
                            name,
                            version: version.to_string(),
                            description,
                            architecture: "x86_64".to_string(),
                            repository,
                            installed: self.installed_packages.contains_key(&repo_name[1]),
                            installed_size: 0,
                            download_size: 0,
                            dependencies: Vec::new(),
                            optional_dependencies: Vec::new(),
                            conflicts: Vec::new(),
                            provides: Vec::new(),
                            groups: Vec::new(),
                            licenses: Vec::new(),
                            maintainer: None,
                            last_modified: None,
                            first_submitted: None,
                            url: None,
                            aur_package: false,
                        });
                        
                        i += 1; // Skip description line
                    }
                }
            }
            i += 1;
        }
        
        packages
    }
    
    fn parse_aur_search(&self, output: &str) -> Vec<PackageInfo> {
        let mut packages = Vec::new();
        
        for line in output.lines() {
            if line.starts_with("aur/") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[0].strip_prefix("aur/").unwrap_or(parts[0]).to_string();
                    let version = parts[1].to_string();
                    let votes_downloads = parts[2]; // Format: (+votes, downloads%)
                    
                    packages.push(PackageInfo {
                        name,
                        version,
                        description: parts[3..].join(" "),
                        architecture: "any".to_string(),
                        repository: "aur".to_string(),
                        installed: false,
                        installed_size: 0,
                        download_size: 0,
                        dependencies: Vec::new(),
                        optional_dependencies: Vec::new(),
                        conflicts: Vec::new(),
                        provides: Vec::new(),
                        groups: Vec::new(),
                        licenses: Vec::new(),
                        maintainer: None,
                        last_modified: None,
                        first_submitted: None,
                        url: None,
                        aur_package: true,
                    });
                }
            }
        }
        
        packages
    }
    
    fn parse_aur_info(&self, output: &str) -> Option<PackageInfo> {
        let mut package = PackageInfo {
            name: String::new(),
            version: String::new(),
            description: String::new(),
            architecture: "any".to_string(),
            repository: "aur".to_string(),
            installed: false,
            installed_size: 0,
            download_size: 0,
            dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            groups: Vec::new(),
            licenses: Vec::new(),
            maintainer: None,
            last_modified: None,
            first_submitted: None,
            url: None,
            aur_package: true,
        };
        
        for line in output.lines() {
            if line.starts_with("Name") {
                package.name = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Version") {
                package.version = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Description") {
                package.description = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Maintainer") {
                package.maintainer = Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
            } else if line.starts_with("URL") {
                package.url = Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
            } else if line.starts_with("Depends On") {
                let deps = line.split(':').nth(1).unwrap_or("").trim();
                if deps != "None" {
                    package.dependencies = deps.split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
            }
        }
        
        if !package.name.is_empty() {
            Some(package)
        } else {
            None
        }
    }
    
    fn parse_size(&self, size_str: &str) -> u64 {
        let parts: Vec<&str> = size_str.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(value) = parts[0].replace(",", "").parse::<f64>() {
                let multiplier = match parts[1] {
                    "KiB" => 1024,
                    "MiB" => 1024 * 1024,
                    "GiB" => 1024 * 1024 * 1024,
                    "TiB" => 1024u64 * 1024 * 1024 * 1024,
                    _ => 1,
                };
                return (value * multiplier as f64) as u64;
            }
        }
        0
    }
}
