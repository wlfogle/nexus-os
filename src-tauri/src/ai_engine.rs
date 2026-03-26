// AI Engine - Adapted from ArchBackupPro AIOptimizer
// Complete AI system optimization and learning engine for Lou's Garuda system

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use std::fs;
use std::env;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tracing::{info, warn, error, debug};

use crate::{SystemMetrics, AIRecommendation, DiskInfo, FanStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecommendation {
    pub backup_type: String,
    pub frequency: String,
    pub compression: String,
    pub exclude_paths: Vec<String>,
    pub reasoning: String,
    pub priority: u8,
    pub suggested_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAnalysis {
    pub total_disk_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub file_count: u32,
    pub package_count: u32,
    pub system_type: String,
    pub large_directories: Vec<String>,
    pub frequently_changed_files: Vec<String>,
    pub change_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub cpu_trend: String,
    pub memory_trend: String,
    pub temperature_trend: String,
    pub disk_trend: String,
    pub network_trend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTarget {
    pub target_type: String,
    pub current_value: f64,
    pub target_value: f64,
    pub optimization_actions: Vec<String>,
    pub priority: u8,
}

pub struct AIOptimizer {
    pub enabled: bool,
    pub sensitivity_level: f64,
    pub auto_optimize: bool,
    pub system_analysis: SystemAnalysis,
    pub recommendations: Vec<AIRecommendation>,
    pub backup_recommendations: Vec<BackupRecommendation>,
    pub performance_trends: PerformanceTrends,
    pub optimization_targets: Vec<OptimizationTarget>,
    
    // Historical data for learning
    pub backup_durations: HashMap<String, Vec<u64>>,
    pub backup_sizes: HashMap<String, Vec<u64>>,
    pub user_preferences: HashMap<String, serde_json::Value>,
    pub system_performance_history: Vec<SystemMetrics>,
    
    // AI parameters
    pub last_analysis: Option<SystemTime>,
    pub analysis_interval: Duration,
    pub learning_rate: f64,
    pub confidence_threshold: f64,
    
    // Working directory for relative paths
    pub work_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl AIOptimizer {
    pub async fn new_for_i9_13900hx() -> Result<Self> {
        info!("🧠 Initializing AI Optimizer for i9-13900HX system");
        
        // Setup relative working directories
        let current_dir = env::current_dir()?;
        let work_dir = current_dir.clone();
        
        // Create relative config and data directories
        let config_dir = work_dir.join("config").join("ai_optimizer");
        let data_dir = work_dir.join("data").join("ai_learning");
        
        // Ensure directories exist
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&data_dir)?;
        
        let initial_analysis = SystemAnalysis {
            total_disk_space: 0,
            used_space: 0,
            available_space: 0,
            file_count: 0,
            package_count: 0,
            system_type: "Unknown".to_string(),
            large_directories: Vec::new(),
            frequently_changed_files: Vec::new(),
            change_rate: 0.0,
        };
        
        let initial_trends = PerformanceTrends {
            cpu_trend: "stable".to_string(),
            memory_trend: "stable".to_string(),
            temperature_trend: "stable".to_string(),
            disk_trend: "stable".to_string(),
            network_trend: "stable".to_string(),
        };
        
        Ok(Self {
            enabled: true,
            sensitivity_level: 5.0,
            auto_optimize: false,
            system_analysis: initial_analysis,
            recommendations: Vec::new(),
            backup_recommendations: Vec::new(),
            performance_trends: initial_trends,
            optimization_targets: Vec::new(),
            backup_durations: HashMap::new(),
            backup_sizes: HashMap::new(),
            user_preferences: HashMap::new(),
            system_performance_history: Vec::new(),
            last_analysis: None,
            analysis_interval: Duration::from_secs(300), // 5 minutes
            learning_rate: 0.1,
            confidence_threshold: 0.75,
            work_dir,
            config_dir,
            data_dir,
        })
    }
    
    pub async fn start_analysis_engine(&mut self) -> Result<()> {
        info!("🔍 Starting AI analysis engine");
        self.enabled = true;
        
        // Load existing learning data
        self.load_learning_data().await?;
        
        // Perform initial system analysis
        self.run_comprehensive_analysis().await?;
        
        info!("✅ AI analysis engine started successfully");
        Ok(())
    }
    
    pub async fn run_comprehensive_analysis(&mut self) -> Result<()> {
        debug!("🔬 Running comprehensive system analysis");
        
        // Step 1: Scan disk usage (adapted from legacy)
        self.scan_disk_usage().await?;
        
        // Step 2: Analyze file changes
        self.analyze_file_changes().await?;
        
        // Step 3: Analyze package statistics  
        self.analyze_package_statistics().await?;
        
        // Step 4: Evaluate compression options
        self.evaluate_compression_options().await?;
        
        // Step 5: Generate comprehensive recommendations
        self.generate_comprehensive_recommendations().await?;
        
        self.last_analysis = Some(SystemTime::now());
        
        debug!("✅ Comprehensive analysis completed");
        Ok(())
    }
    
    async fn scan_disk_usage(&mut self) -> Result<()> {
        debug!("💾 Scanning disk usage");
        
        // Get disk information from relative paths
        let mut total_space = 0u64;
        let mut used_space = 0u64;
        let mut available_space = 0u64;
        let mut large_dirs = Vec::new();
        
        // Scan from working directory
        if let Ok(entries) = fs::read_dir(&self.work_dir) {
            for entry in entries.flatten() {
                if entry.file_type()?.is_dir() {
                    let path = entry.path();
                    if let Ok(metadata) = fs::metadata(&path) {
                        let size = self.get_directory_size(&path).unwrap_or(0);
                        
                        // Consider directories > 100MB as large (relative)
                        if size > 100 * 1024 * 1024 {
                            if let Some(name) = path.file_name() {
                                large_dirs.push(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // Update system analysis with relative disk info
        self.system_analysis.total_disk_space = total_space;
        self.system_analysis.used_space = used_space;
        self.system_analysis.available_space = available_space;
        self.system_analysis.large_directories = large_dirs;
        
        Ok(())
    }
    
    fn get_directory_size(&self, dir: &PathBuf) -> Result<u64> {
        let mut size = 0u64;
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        size += metadata.len();
                    }
                } else if path.is_dir() {
                    // Recursively scan subdirectories
                    size += self.get_directory_size(&path).unwrap_or(0);
                }
            }
        }
        
        Ok(size)
    }
    
    async fn analyze_file_changes(&mut self) -> Result<()> {
        debug!("📁 Analyzing file changes");
        
        let mut frequently_changed = Vec::new();
        let mut change_count = 0;
        
        // Analyze files in working directory for changes
        let change_threshold = SystemTime::now() - Duration::from_secs(86400); // 24 hours
        
        if let Ok(entries) = fs::read_dir(&self.work_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > change_threshold {
                            change_count += 1;
                            if let Some(name) = entry.path().file_name() {
                                frequently_changed.push(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
        
        self.system_analysis.frequently_changed_files = frequently_changed;
        self.system_analysis.change_rate = change_count as f64;
        
        Ok(())
    }
    
    async fn analyze_package_statistics(&mut self) -> Result<()> {
        debug!("📦 Analyzing package statistics");
        
        // Simulate package analysis (would query pacman in real implementation)
        self.system_analysis.package_count = 1200; // Typical Garuda installation
        self.system_analysis.system_type = "Garuda Linux (KDE Plasma)".to_string();
        
        Ok(())
    }
    
    async fn evaluate_compression_options(&mut self) -> Result<()> {
        debug!("🗜️ Evaluating compression options");
        
        // AI logic for optimal compression based on system capabilities
        // For i9-13900HX: high CPU, recommend zstd for best balance
        let optimal_compression = "zstd".to_string();
        let compression_level = self.get_optimal_compression_level();
        
        debug!("💡 Optimal compression: {} level {}", optimal_compression, compression_level);
        
        Ok(())
    }
    
    fn get_optimal_compression_level(&self) -> u8 {
        // AI logic for compression level based on available space and CPU
        let storage_ratio = self.system_analysis.available_space as f64 / 
                           self.system_analysis.total_disk_space.max(1) as f64;
        
        if storage_ratio < 0.1 {
            9 // Maximum compression when space is critical
        } else if storage_ratio < 0.3 {
            7 // High compression when space is low
        } else {
            6 // Balanced compression for i9-13900HX
        }
    }
    
    async fn generate_comprehensive_recommendations(&mut self) -> Result<()> {
        debug!("💡 Generating AI recommendations");
        
        self.recommendations.clear();
        self.backup_recommendations.clear();
        
        // Generate backup frequency recommendation
        self.generate_frequency_recommendation().await?;
        
        // Generate compression recommendations
        self.generate_compression_recommendation().await?;
        
        // Generate exclusion recommendations  
        self.generate_exclusion_recommendations().await?;
        
        // Generate schedule recommendations
        self.generate_schedule_recommendation().await?;
        
        // Generate performance optimization recommendations
        self.generate_performance_recommendations().await?;
        
        debug!("✅ Generated {} recommendations", self.recommendations.len());
        Ok(())
    }
    
    async fn generate_frequency_recommendation(&mut self) -> Result<()> {
        let change_rate = self.system_analysis.change_rate;
        let available_ratio = self.system_analysis.available_space as f64 / 
                             self.system_analysis.total_disk_space.max(1) as f64;
        
        let (frequency, reasoning) = if change_rate > 50.0 && available_ratio > 0.3 {
            ("Every 4 hours", "High file change rate detected with sufficient storage")
        } else if change_rate > 20.0 {
            ("Every 12 hours", "Moderate file change rate detected")
        } else if change_rate > 5.0 {
            ("Daily", "Low to moderate file change rate")
        } else {
            ("Weekly", "Very low file change rate detected")
        };
        
        let backup_rec = BackupRecommendation {
            backup_type: "incremental".to_string(),
            frequency: frequency.to_string(),
            compression: "zstd".to_string(),
            exclude_paths: self.get_suggested_exclusions(),
            reasoning: reasoning.to_string(),
            priority: 9,
            suggested_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600,
        };
        
        self.backup_recommendations.push(backup_rec);
        Ok(())
    }
    
    async fn generate_compression_recommendation(&mut self) -> Result<()> {
        let optimal_method = self.get_optimal_compression_method();
        let level = self.get_optimal_compression_level();
        
        let rec = AIRecommendation {
            id: uuid::Uuid::new_v4().to_string(),
            category: "Backup".to_string(),
            title: "Optimal Compression Settings".to_string(),
            description: format!("Use {} compression at level {} for optimal performance on i9-13900HX", optimal_method, level),
            priority: 8,
            actions: vec![
                format!("Set compression to {}", optimal_method),
                format!("Set compression level to {}", level),
            ],
            auto_apply: true,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        
        self.recommendations.push(rec);
        Ok(())
    }
    
    fn get_optimal_compression_method(&self) -> String {
        // AI logic for compression method based on system analysis
        let cpu_performance_score = 1.0; // i9-13900HX is high performance
        let storage_ratio = self.system_analysis.available_space as f64 / 
                           self.system_analysis.total_disk_space.max(1) as f64;
        
        if cpu_performance_score > 0.8 && storage_ratio < 0.2 {
            "zstd".to_string() // High CPU, low storage - best compression
        } else if cpu_performance_score < 0.4 {
            "gzip".to_string() // Low CPU - lighter compression
        } else {
            "zstd".to_string() // Balanced choice for i9-13900HX
        }
    }
    
    async fn generate_exclusion_recommendations(&mut self) -> Result<()> {
        let exclusions = self.get_suggested_exclusions();
        
        let rec = AIRecommendation {
            id: uuid::Uuid::new_v4().to_string(),
            category: "Backup".to_string(),
            title: "Intelligent File Exclusions".to_string(),
            description: format!("Exclude {} file patterns to optimize backup efficiency", exclusions.len()),
            priority: 6,
            actions: exclusions.iter().map(|e| format!("Exclude: {}", e)).collect(),
            auto_apply: false,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        
        self.recommendations.push(rec);
        Ok(())
    }
    
    fn get_suggested_exclusions(&self) -> Vec<String> {
        let mut exclusions = vec![
            "target/*".to_string(),
            "node_modules/*".to_string(),
            ".git/*".to_string(),
            "*.tmp".to_string(),
            "*.swp".to_string(),
            "*~".to_string(),
            ".cache/*".to_string(),
        ];
        
        // Add large directories that look like cache/temp
        for dir in &self.system_analysis.large_directories {
            let dir_lower = dir.to_lowercase();
            if dir_lower.contains("cache") || 
               dir_lower.contains("temp") || 
               dir_lower.contains("log") ||
               dir_lower.contains("tmp") {
                exclusions.push(format!("{}/*", dir));
            }
        }
        
        exclusions
    }
    
    async fn generate_schedule_recommendation(&mut self) -> Result<()> {
        let change_rate = self.system_analysis.change_rate;
        
        let (schedule, time_desc) = if change_rate > 50.0 {
            ("Every 6 hours", "High change rate requires frequent backups")
        } else if change_rate > 10.0 {
            ("Daily", "Moderate change rate suggests daily backups")
        } else {
            ("Weekly", "Low change rate allows weekly backups")
        };
        
        let rec = AIRecommendation {
            id: uuid::Uuid::new_v4().to_string(),
            category: "Scheduling".to_string(),
            title: "Optimal Backup Schedule".to_string(),
            description: format!("{}: {}", schedule, time_desc),
            priority: 7,
            actions: vec![
                format!("Set backup frequency to {}", schedule),
                "Schedule during low usage periods".to_string(),
            ],
            auto_apply: false,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        
        self.recommendations.push(rec);
        Ok(())
    }
    
    async fn generate_performance_recommendations(&mut self) -> Result<()> {
        // Generate i9-13900HX specific performance recommendations
        let rec = AIRecommendation {
            id: uuid::Uuid::new_v4().to_string(),
            category: "Performance".to_string(),
            title: "i9-13900HX Optimization".to_string(),
            description: "Optimize system for maximum performance on gaming laptop".to_string(),
            priority: 9,
            actions: vec![
                "Enable performance CPU governor".to_string(),
                "Optimize memory allocation".to_string(),
                "Configure intelligent fan curves".to_string(),
                "Apply thermal optimization".to_string(),
            ],
            auto_apply: false,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        
        self.recommendations.push(rec);
        Ok(())
    }
    
    pub async fn generate_recommendations(&mut self, metrics: &SystemMetrics) -> Result<Vec<AIRecommendation>> {
        // Store current metrics for trend analysis
        self.system_performance_history.push(metrics.clone());
        
        // Keep only recent history (last 100 samples)
        if self.system_performance_history.len() > 100 {
            self.system_performance_history.remove(0);
        }
        
        // Update performance trends
        self.update_performance_trends().await?;
        
        // Generate dynamic recommendations based on current metrics
        self.generate_dynamic_recommendations(metrics).await?;
        
        Ok(self.recommendations.clone())
    }
    
    async fn update_performance_trends(&mut self) -> Result<()> {
        if self.system_performance_history.len() < 10 {
            return Ok(()); // Need more data
        }
        
        let recent_samples = &self.system_performance_history[self.system_performance_history.len() - 10..];
        
        // Calculate trends
        self.performance_trends.cpu_trend = self.calculate_trend(
            &recent_samples.iter().map(|s| s.cpu_usage as f64).collect::<Vec<_>>()
        );
        
        self.performance_trends.memory_trend = self.calculate_trend(
            &recent_samples.iter().map(|s| s.memory_usage).collect::<Vec<_>>()
        );
        
        self.performance_trends.temperature_trend = self.calculate_trend(
            &recent_samples.iter().map(|s| s.cpu_temp as f64).collect::<Vec<_>>()
        );
        
        Ok(())
    }
    
    fn calculate_trend(&self, values: &[f64]) -> String {
        if values.len() < 3 {
            return "stable".to_string();
        }
        
        // Simple linear regression for trend detection
        let n = values.len() as f64;
        let x_sum = (0..values.len()).sum::<usize>() as f64;
        let y_sum: f64 = values.iter().sum();
        let xy_sum: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let x2_sum: f64 = (0..values.len()).map(|i| (i * i) as f64).sum();
        
        let slope = (n * xy_sum - x_sum * y_sum) / (n * x2_sum - x_sum * x_sum);
        
        if slope > 0.5 {
            "increasing".to_string()
        } else if slope < -0.5 {
            "decreasing".to_string()
        } else {
            "stable".to_string()
        }
    }
    
    async fn generate_dynamic_recommendations(&mut self, metrics: &SystemMetrics) -> Result<()> {
        // Clear previous dynamic recommendations
        self.recommendations.retain(|r| r.category != "Dynamic");
        
        // CPU usage recommendations
        if metrics.cpu_usage > 90.0 {
            let rec = AIRecommendation {
                id: uuid::Uuid::new_v4().to_string(),
                category: "Dynamic".to_string(),
                title: "High CPU Usage Detected".to_string(),
                description: format!("CPU usage at {:.1}% - consider optimization", metrics.cpu_usage),
                priority: 8,
                actions: vec![
                    "Switch to performance mode".to_string(),
                    "Close unnecessary applications".to_string(),
                    "Check for runaway processes".to_string(),
                ],
                auto_apply: false,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };
            self.recommendations.push(rec);
        }
        
        // Memory usage recommendations  
        if metrics.memory_usage > 80.0 {
            let rec = AIRecommendation {
                id: uuid::Uuid::new_v4().to_string(),
                category: "Dynamic".to_string(),
                title: "High Memory Usage".to_string(),
                description: format!("Memory usage at {:.1}% - optimization recommended", metrics.memory_usage),
                priority: 7,
                actions: vec![
                    "Clear system caches".to_string(),
                    "Close memory-intensive applications".to_string(),
                    "Enable swap optimization".to_string(),
                ],
                auto_apply: false,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };
            self.recommendations.push(rec);
        }
        
        // Temperature recommendations
        if metrics.cpu_temp > 80.0 {
            let rec = AIRecommendation {
                id: uuid::Uuid::new_v4().to_string(),
                category: "Dynamic".to_string(),
                title: "High CPU Temperature".to_string(),
                description: format!("CPU temperature at {:.1}°C - thermal management needed", metrics.cpu_temp),
                priority: 9,
                actions: vec![
                    "Increase fan speeds".to_string(),
                    "Apply thermal throttling".to_string(),
                    "Check thermal paste".to_string(),
                ],
                auto_apply: true, // Auto-apply thermal protection
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };
            self.recommendations.push(rec);
        }
        
        Ok(())
    }
    
    pub async fn optimize_system_performance(&mut self) -> Result<()> {
        info!("⚡ Optimizing system performance");
        
        // Apply auto-applicable recommendations
        let auto_recommendations: Vec<_> = self.recommendations
            .iter()
            .filter(|r| r.auto_apply)
            .cloned()
            .collect();
        
        for rec in auto_recommendations {
            info!("🔧 Auto-applying: {}", rec.title);
            self.apply_recommendation(&rec).await?;
        }
        
        Ok(())
    }
    
    async fn apply_recommendation(&mut self, rec: &AIRecommendation) -> Result<()> {
        debug!("🎯 Applying recommendation: {}", rec.title);
        
        for action in &rec.actions {
            match action.as_str() {
                "Enable performance CPU governor" => {
                    // Would set CPU governor in real implementation
                    debug!("🚀 Setting CPU to performance mode");
                },
                "Increase fan speeds" => {
                    // Would control fans in real implementation
                    debug!("🌪️ Increasing fan speeds for cooling");
                },
                "Clear system caches" => {
                    // Would clear caches in real implementation
                    debug!("🧹 Clearing system caches");
                },
                _ => {
                    debug!("📝 Action noted: {}", action);
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn record_backup_performance(&mut self, backup_type: &str, duration: u64, size: u64) -> Result<()> {
        // Record backup performance for learning
        self.backup_durations
            .entry(backup_type.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
        
        self.backup_sizes
            .entry(backup_type.to_string())
            .or_insert_with(Vec::new)
            .push(size);
        
        // Keep only recent history (last 20 backups per type)
        for durations in self.backup_durations.values_mut() {
            if durations.len() > 20 {
                durations.remove(0);
            }
        }
        
        for sizes in self.backup_sizes.values_mut() {
            if sizes.len() > 20 {
                sizes.remove(0);
            }
        }
        
        // Save learning data
        self.save_learning_data().await?;
        
        Ok(())
    }
    
    async fn load_learning_data(&mut self) -> Result<()> {
        let data_file = self.data_dir.join("ai_learning_data.json");
        
        if data_file.exists() {
            let data = fs::read_to_string(&data_file)?;
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(prefs) = parsed.get("user_preferences").and_then(|p| p.as_object()) {
                    for (key, value) in prefs {
                        self.user_preferences.insert(key.clone(), value.clone());
                    }
                }
                debug!("📚 Loaded AI learning data from {}", data_file.display());
            }
        }
        
        Ok(())
    }
    
    async fn save_learning_data(&self) -> Result<()> {
        let data = serde_json::json!({
            "user_preferences": self.user_preferences,
            "backup_durations": self.backup_durations,
            "backup_sizes": self.backup_sizes,
            "last_analysis": self.last_analysis.map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs()),
            "performance_trends": self.performance_trends,
        });
        
        let data_file = self.data_dir.join("ai_learning_data.json");
        fs::write(&data_file, serde_json::to_string_pretty(&data)?)?;
        
        debug!("💾 Saved AI learning data to {}", data_file.display());
        Ok(())
    }
    
    pub fn get_system_insights(&self) -> HashMap<String, serde_json::Value> {
        let mut insights = HashMap::new();
        
        insights.insert("system_analysis".to_string(), serde_json::to_value(&self.system_analysis).unwrap());
        insights.insert("performance_trends".to_string(), serde_json::to_value(&self.performance_trends).unwrap());
        insights.insert("recommendations_count".to_string(), serde_json::Value::Number(self.recommendations.len().into()));
        insights.insert("learning_enabled".to_string(), serde_json::Value::Bool(self.enabled));
        insights.insert("confidence_level".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(self.confidence_threshold).unwrap()));
        
        insights
    }
}
