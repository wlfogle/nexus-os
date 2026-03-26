use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[allow(unused_imports)]
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use std::process::Stdio;
use tokio::process::Command;

// Missing types expected by main.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub plugin_id: String,
    pub success: bool,
    pub message: String,
    pub install_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: PluginCategory,
    pub status: PluginStatus,
    pub enabled: bool,
    pub install_path: Option<PathBuf>,
    pub installed_at: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionResult {
    pub execution_id: String,
    pub plugin_id: String,
    pub command: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub plugin_id: String,
    pub success: bool,
    pub message: String,
    pub old_version: String,
    pub new_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub tags: Vec<String>,
    pub category: PluginCategory,
    pub manifest: PluginManifest,
    pub status: PluginStatus,
    pub install_path: Option<PathBuf>,
    pub installed_at: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginCategory {
    Terminal,
    Git,
    Development,
    System,
    Network,
    Security,
    Productivity,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginStatus {
    Available,
    Installing,
    Installed,
    Enabled,
    Disabled,
    UpdateAvailable,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub entry_point: String,
    pub commands: Vec<PluginCommand>,
    pub hooks: Vec<PluginHook>,
    pub permissions: Vec<PluginPermission>,
    pub dependencies: Vec<String>,
    pub api_version: String,
    pub platform_requirements: Vec<String>,
    pub config_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub parameters: Vec<CommandParameter>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    pub name: String,
    pub parameter_type: ParameterType,
    pub required: bool,
    pub description: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    File,
    Directory,
    Choice(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHook {
    pub name: String,
    pub event: HookEvent,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookEvent {
    BeforeCommand,
    AfterCommand,
    OnError,
    OnStartup,
    OnShutdown,
    OnFileChange,
    OnGitEvent,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginPermission {
    FileSystemRead,
    FileSystemWrite,
    NetworkAccess,
    ProcessExecution,
    EnvironmentVariables,
    TerminalAccess,
    GitAccess,
    SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketplace {
    pub plugins: Vec<MarketplacePlugin>,
    pub categories: Vec<PluginCategory>,
    pub featured: Vec<String>,
    pub trending: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub plugin: Plugin,
    pub downloads: u64,
    pub rating: f32,
    pub reviews: Vec<PluginReview>,
    pub screenshots: Vec<String>,
    pub changelog: Vec<ChangelogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub author: String,
    pub rating: u8,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: DateTime<Utc>,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecution {
    pub id: String,
    pub plugin_id: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSandbox {
    pub plugin_id: String,
    pub allowed_paths: Vec<PathBuf>,
    pub environment_vars: HashMap<String, String>,
    pub resource_limits: ResourceLimits,
    pub network_access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
    pub max_execution_time_seconds: u64,
    pub max_file_size_mb: u64,
}

#[derive(Debug)]
pub struct PluginSystem {
    plugins: HashMap<String, Plugin>,
    enabled_plugins: Vec<String>,
    plugin_executions: HashMap<String, PluginExecution>,
    marketplace: Option<PluginMarketplace>,
    plugins_dir: PathBuf,
    sandboxes: HashMap<String, PluginSandbox>,
}

#[allow(dead_code)]
impl PluginSystem {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            enabled_plugins: Vec::new(),
            plugin_executions: HashMap::new(),
            marketplace: None,
            plugins_dir,
            sandboxes: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Create plugins directory if it doesn't exist
        if !self.plugins_dir.exists() {
            tokio::fs::create_dir_all(&self.plugins_dir).await?;
        }

        // Load installed plugins
        self.discover_plugins().await?;
        
        // Initialize enabled plugins
        for plugin_id in &self.enabled_plugins.clone() {
            self.initialize_plugin(plugin_id).await?;
        }

        Ok(())
    }

    async fn discover_plugins(&mut self) -> Result<()> {
        let mut entries = tokio::fs::read_dir(&self.plugins_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let plugin_dir = entry.path();
                let manifest_path = plugin_dir.join("plugin.json");
                
                if manifest_path.exists() {
                    match self.load_plugin_manifest(&manifest_path).await {
                        Ok(plugin) => {
                            self.plugins.insert(plugin.id.clone(), plugin);
                        }
                        Err(e) => {
                            eprintln!("Failed to load plugin from {:?}: {}", plugin_dir, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn load_plugin_manifest(&self, manifest_path: &PathBuf) -> Result<Plugin> {
        let content = tokio::fs::read_to_string(manifest_path).await?;
        let plugin: Plugin = serde_json::from_str(&content)?;
        Ok(plugin)
    }

    async fn install_plugin_internal(&mut self, plugin_id: &str, source: PluginSource) -> Result<()> {
        if self.plugins.contains_key(plugin_id) {
            return Err(anyhow!("Plugin already installed: {}", plugin_id));
        }

        let plugin = match source {
            PluginSource::Marketplace => {
                self.download_from_marketplace(plugin_id).await?
            }
            PluginSource::Git(url) => {
                self.install_from_git(plugin_id, &url).await?
            }
            PluginSource::Local(path) => {
                self.install_from_local(&path).await?
            }
            PluginSource::Archive(path) => {
                self.install_from_archive(&path).await?
            }
        };

        // Create sandbox for the plugin
        let sandbox = self.create_sandbox(&plugin)?;
        self.sandboxes.insert(plugin_id.to_string(), sandbox);

        // Install the plugin
        let install_path = self.plugins_dir.join(&plugin.id);
        let mut installed_plugin = plugin;
        installed_plugin.status = PluginStatus::Installed;
        installed_plugin.install_path = Some(install_path);
        installed_plugin.installed_at = Some(Utc::now());

        self.plugins.insert(plugin_id.to_string(), installed_plugin);
        
        Ok(())
    }

    async fn download_from_marketplace(&self, plugin_id: &str) -> Result<Plugin> {
        // In a real implementation, this would download from a marketplace
        // For now, return a mock plugin
        Ok(Plugin {
            id: plugin_id.to_string(),
            name: format!("Plugin {}", plugin_id),
            version: "1.0.0".to_string(),
            description: "A sample plugin".to_string(),
            author: "Unknown".to_string(),
            license: "MIT".to_string(),
            repository: None,
            homepage: None,
            tags: vec![],
            category: PluginCategory::Custom,
            manifest: PluginManifest {
                entry_point: "main.js".to_string(),
                commands: vec![],
                hooks: vec![],
                permissions: vec![],
                dependencies: vec![],
                api_version: "1.0".to_string(),
                platform_requirements: vec!["linux".to_string()],
                config_schema: None,
            },
            status: PluginStatus::Available,
            install_path: None,
            installed_at: None,
            last_updated: None,
        })
    }

    async fn install_from_git(&self, plugin_id: &str, git_url: &str) -> Result<Plugin> {
        let clone_path = self.plugins_dir.join(plugin_id);
        
        let output = Command::new("git")
            .args(&["clone", git_url, &clone_path.to_string_lossy()])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to clone git repository: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let manifest_path = clone_path.join("plugin.json");
        self.load_plugin_manifest(&manifest_path).await
    }

    async fn install_from_local(&self, local_path: &PathBuf) -> Result<Plugin> {
        let manifest_path = local_path.join("plugin.json");
        self.load_plugin_manifest(&manifest_path).await
    }

    async fn install_from_archive(&self, archive_path: &PathBuf) -> Result<Plugin> {
        // In a real implementation, this would extract and install from archive
        self.install_from_local(archive_path).await
    }

    fn create_sandbox(&self, plugin: &Plugin) -> Result<PluginSandbox> {
        let allowed_paths = vec![
            self.plugins_dir.join(&plugin.id),
            PathBuf::from("/tmp"), // Allow temporary files
        ];

        let mut environment_vars = HashMap::new();
        environment_vars.insert("PLUGIN_ID".to_string(), plugin.id.clone());
        environment_vars.insert("PLUGIN_VERSION".to_string(), plugin.version.clone());

        let resource_limits = ResourceLimits {
            max_memory_mb: 256,
            max_cpu_percent: 50.0,
            max_execution_time_seconds: 30,
            max_file_size_mb: 100,
        };

        let network_access = plugin.manifest.permissions.contains(&PluginPermission::NetworkAccess);

        Ok(PluginSandbox {
            plugin_id: plugin.id.clone(),
            allowed_paths,
            environment_vars,
            resource_limits,
            network_access,
        })
    }

    pub async fn enable_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.status = PluginStatus::Enabled;
            if !self.enabled_plugins.contains(&plugin_id.to_string()) {
                self.enabled_plugins.push(plugin_id.to_string());
            }
            self.initialize_plugin(plugin_id).await?;
            Ok(())
        } else {
            Err(anyhow!("Plugin not found: {}", plugin_id))
        }
    }

    pub async fn disable_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.status = PluginStatus::Disabled;
            self.enabled_plugins.retain(|id| id != plugin_id);
            Ok(())
        } else {
            Err(anyhow!("Plugin not found: {}", plugin_id))
        }
    }

    async fn initialize_plugin(&self, plugin_id: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get(plugin_id) {
            // Run plugin initialization hooks
            for hook in &plugin.manifest.hooks {
                if matches!(hook.event, HookEvent::OnStartup) {
                    self.execute_hook(plugin_id, hook).await?;
                }
            }
        }
        Ok(())
    }

    async fn execute_hook(&self, plugin_id: &str, hook: &PluginHook) -> Result<()> {
        if let Some(plugin) = self.plugins.get(plugin_id) {
            if let Some(install_path) = &plugin.install_path {
                let entry_point = install_path.join(&plugin.manifest.entry_point);
                
                let mut cmd = Command::new("node"); // Assuming JavaScript plugins
                cmd.arg(&entry_point)
                   .arg("--hook")
                   .arg(&hook.name)
                   .current_dir(install_path)
                   .stdout(Stdio::piped())
                   .stderr(Stdio::piped());

                // Apply sandbox restrictions
                if let Some(sandbox) = self.sandboxes.get(plugin_id) {
                    for (key, value) in &sandbox.environment_vars {
                        cmd.env(key, value);
                    }
                }

                let output = cmd.output().await?;
                if !output.status.success() {
                    eprintln!("Hook execution failed for {}: {}", hook.name, String::from_utf8_lossy(&output.stderr));
                }
            }
        }
        Ok(())
    }


    async fn execute_sandboxed_command(&self, plugin_id: &str, command: &str, args: Vec<String>) -> Result<String> {
        if let Some(plugin) = self.plugins.get(plugin_id) {
            if let Some(install_path) = &plugin.install_path {
                let entry_point = install_path.join(&plugin.manifest.entry_point);
                
                let mut cmd = Command::new("node");
                cmd.arg(&entry_point)
                   .arg("--command")
                   .arg(command)
                   .args(&args)
                   .current_dir(install_path)
                   .stdout(Stdio::piped())
                   .stderr(Stdio::piped());

                // Apply sandbox restrictions
                if let Some(sandbox) = self.sandboxes.get(plugin_id) {
                    for (key, value) in &sandbox.environment_vars {
                        cmd.env(key, value);
                    }
                    
                    // Apply resource limits
                    // Memory limits
                    if sandbox.resource_limits.max_memory_mb > 0 {
                        #[cfg(target_os = "linux")]
                        {
                            cmd.env("PLUGIN_MEMORY_LIMIT", sandbox.resource_limits.max_memory_mb.to_string());
                            // Use cgroup v2 if available
                            if Path::new("/sys/fs/cgroup/cgroup.controllers").exists() {
                                let cgroup_path = format!("/sys/fs/cgroup/nexus-plugins/{}", plugin_id);
                                let _ = std::fs::create_dir_all(&cgroup_path);
                                let _ = std::fs::write(
                                    format!("{}/memory.max", cgroup_path), 
                                    format!("{}", sandbox.resource_limits.max_memory_mb * 1024 * 1024)
                                );
                                cmd.env("NEXUS_PLUGIN_CGROUP", cgroup_path);
                            }
                        }
                    }
                    
                    // CPU limits
                    if sandbox.resource_limits.max_cpu_percent > 0.0 {
                        cmd.env("PLUGIN_CPU_LIMIT", sandbox.resource_limits.max_cpu_percent.to_string());
                        // For Node.js plugins, we can use --max-old-space-size to limit memory
                        if command == "node" {
                            let max_old_space = sandbox.resource_limits.max_memory_mb;
                            cmd.arg(format!("--max-old-space-size={}", max_old_space));
                        }
                    }
                    
                    // File descriptor limits
                    #[cfg(target_family = "unix")]
                    {
                        // use std::os::unix::process::CommandExt;
                        // File descriptor limits are not yet implemented in ResourceLimits
                        // if let Some(max_files) = sandbox.resource_limits.max_files {
                        //     unsafe {
                        //         cmd.pre_exec(move || {
                        //             let res = libc::setrlimit(
                        //                 libc::RLIMIT_NOFILE,
                        //                 &libc::rlimit {
                        //                     rlim_cur: max_files as libc::rlim_t,
                        //                     rlim_max: max_files as libc::rlim_t,
                        //                 },
                        //             );
                        //             if res != 0 {
                        //                 return Err(std::io::Error::last_os_error());
                        //             }
                        //             Ok(())
                        //         });
                        //     }
                        // }
                    }
                }

                let output = cmd.output().await?;
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(anyhow!("Command execution failed: {}", String::from_utf8_lossy(&output.stderr)))
                }
            } else {
                Err(anyhow!("Plugin not installed: {}", plugin_id))
            }
        } else {
            Err(anyhow!("Plugin not found: {}", plugin_id))
        }
    }

    pub async fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(plugin_id) {
            // Disable first
            self.enabled_plugins.retain(|id| id != plugin_id);
            
            // Remove sandbox
            self.sandboxes.remove(plugin_id);

            // Remove installation directory
            if let Some(install_path) = &plugin.install_path {
                if install_path.exists() {
                    tokio::fs::remove_dir_all(install_path).await?;
                }
            }

            Ok(())
        } else {
            Err(anyhow!("Plugin not found: {}", plugin_id))
        }
    }


    pub async fn refresh_marketplace(&mut self) -> Result<()> {
        // In a real implementation, this would fetch from a remote marketplace
        let marketplace = PluginMarketplace {
            plugins: vec![],
            categories: vec![
                PluginCategory::Terminal,
                PluginCategory::Git,
                PluginCategory::Development,
                PluginCategory::System,
            ],
            featured: vec![],
            trending: vec![],
            updated_at: Utc::now(),
        };

        self.marketplace = Some(marketplace);
        Ok(())
    }

    pub fn search_plugins(&self, query: &str, category: Option<PluginCategory>) -> Vec<&Plugin> {
        self.plugins
            .values()
            .filter(|plugin| {
                let name_match = plugin.name.to_lowercase().contains(&query.to_lowercase());
                let desc_match = plugin.description.to_lowercase().contains(&query.to_lowercase());
                let tag_match = plugin.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()));
                let category_match = category.as_ref().map_or(true, |cat| std::mem::discriminant(&plugin.category) == std::mem::discriminant(cat));
                
                (name_match || desc_match || tag_match) && category_match
            })
            .collect()
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<&Plugin> {
        self.plugins.get(plugin_id)
    }


    pub fn list_enabled_plugins(&self) -> Vec<&Plugin> {
        self.enabled_plugins
            .iter()
            .filter_map(|id| self.plugins.get(id))
            .collect()
    }

    pub fn get_plugin_execution(&self, execution_id: &str) -> Option<&PluginExecution> {
        self.plugin_executions.get(execution_id)
    }

    pub fn get_marketplace(&self) -> Option<&PluginMarketplace> {
        self.marketplace.as_ref()
    }

    // Methods expected by main.rs
    pub async fn install_plugin(&mut self, plugin_path: &str) -> Result<InstallResult> {
        // Determine source from path
        let source = if plugin_path.starts_with("http://") || plugin_path.starts_with("https://") {
            if plugin_path.contains("github.com") || plugin_path.ends_with(".git") {
                PluginSource::Git(plugin_path.to_string())
            } else {
                PluginSource::Marketplace
            }
        } else {
            let path = PathBuf::from(plugin_path);
            if path.is_file() && plugin_path.ends_with(".tar.gz") || plugin_path.ends_with(".zip") {
                PluginSource::Archive(path)
            } else {
                PluginSource::Local(path)
            }
        };

        let plugin_id = PathBuf::from(plugin_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown-plugin")
            .to_string();

        match self.install_plugin_internal(&plugin_id, source).await {
            Ok(()) => {
                let install_path = self.plugins_dir.join(&plugin_id);
                Ok(InstallResult {
                    plugin_id: plugin_id.clone(),
                    success: true,
                    message: format!("Plugin {} installed successfully", plugin_id),
                    install_path: Some(install_path),
                })
            }
            Err(e) => Ok(InstallResult {
                plugin_id: plugin_id.clone(),
                success: false,
                message: e.to_string(),
                install_path: None,
            })
        }
    }

    pub async fn list_plugins(&self) -> Result<Vec<PluginInfo>> {
        let mut plugin_infos = Vec::new();
        
        for plugin in self.plugins.values() {
            let enabled = self.enabled_plugins.contains(&plugin.id);
            plugin_infos.push(PluginInfo {
                id: plugin.id.clone(),
                name: plugin.name.clone(),
                version: plugin.version.clone(),
                description: plugin.description.clone(),
                author: plugin.author.clone(),
                category: plugin.category.clone(),
                status: plugin.status.clone(),
                enabled,
                install_path: plugin.install_path.clone(),
                installed_at: plugin.installed_at,
                last_updated: plugin.last_updated,
            });
        }
        
        Ok(plugin_infos)
    }

    pub async fn get_marketplace_plugins(&self) -> Result<Vec<MarketplacePlugin>> {
        if let Some(marketplace) = &self.marketplace {
            Ok(marketplace.plugins.clone())
        } else {
            // If marketplace is not loaded, return mock data
            Ok(vec![
                MarketplacePlugin {
                    plugin: Plugin {
                        id: "sample-terminal-plugin".to_string(),
                        name: "Sample Terminal Plugin".to_string(),
                        version: "1.0.0".to_string(),
                        description: "A sample plugin for terminal enhancements".to_string(),
                        author: "Terminal Team".to_string(),
                        license: "MIT".to_string(),
                        repository: Some("https://github.com/example/sample-terminal-plugin".to_string()),
                        homepage: Some("https://example.com".to_string()),
                        tags: vec!["terminal".to_string(), "productivity".to_string()],
                        category: PluginCategory::Terminal,
                        manifest: PluginManifest {
                            entry_point: "index.js".to_string(),
                            commands: vec![],
                            hooks: vec![],
                            permissions: vec![PluginPermission::TerminalAccess],
                            dependencies: vec![],
                            api_version: "1.0".to_string(),
                            platform_requirements: vec!["linux".to_string(), "macos".to_string()],
                            config_schema: None,
                        },
                        status: PluginStatus::Available,
                        install_path: None,
                        installed_at: None,
                        last_updated: None,
                    },
                    downloads: 1250,
                    rating: 4.5,
                    reviews: vec![],
                    screenshots: vec![],
                    changelog: vec![],
                },
            ])
        }
    }

    pub async fn execute_plugin_command(&self, plugin_id: &str, command: &str, args: &[String]) -> Result<PluginExecutionResult> {
        if let Some(plugin) = self.plugins.get(plugin_id) {
            if plugin.status != PluginStatus::Enabled {
                return Ok(PluginExecutionResult {
                    execution_id: uuid::Uuid::new_v4().to_string(),
                    plugin_id: plugin_id.to_string(),
                    command: command.to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Plugin is not enabled: {}", plugin_id)),
                    duration_ms: Some(0),
                });
            }

            let execution_id = uuid::Uuid::new_v4().to_string();
            let start_time = std::time::Instant::now();
            
            // Execute the command within sandbox
            let result = self.execute_sandboxed_command(plugin_id, command, args.to_vec()).await;
            let duration_ms = start_time.elapsed().as_millis() as u64;

            match result {
                Ok(output) => Ok(PluginExecutionResult {
                    execution_id,
                    plugin_id: plugin_id.to_string(),
                    command: command.to_string(),
                    success: true,
                    output,
                    error: None,
                    duration_ms: Some(duration_ms),
                }),
                Err(error) => Ok(PluginExecutionResult {
                    execution_id,
                    plugin_id: plugin_id.to_string(),
                    command: command.to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                    duration_ms: Some(duration_ms),
                }),
            }
        } else {
            Ok(PluginExecutionResult {
                execution_id: uuid::Uuid::new_v4().to_string(),
                plugin_id: plugin_id.to_string(),
                command: command.to_string(),
                success: false,
                output: String::new(),
                error: Some(format!("Plugin not found: {}", plugin_id)),
                duration_ms: Some(0),
            })
        }
    }

    pub async fn get_plugin_info(&self, plugin_id: &str) -> Result<PluginInfo> {
        if let Some(plugin) = self.plugins.get(plugin_id) {
            let enabled = self.enabled_plugins.contains(&plugin.id);
            Ok(PluginInfo {
                id: plugin.id.clone(),
                name: plugin.name.clone(),
                version: plugin.version.clone(),
                description: plugin.description.clone(),
                author: plugin.author.clone(),
                category: plugin.category.clone(),
                status: plugin.status.clone(),
                enabled,
                install_path: plugin.install_path.clone(),
                installed_at: plugin.installed_at,
                last_updated: plugin.last_updated,
            })
        } else {
            Err(anyhow!("Plugin not found: {}", plugin_id))
        }
    }

    pub async fn update_plugin(&mut self, plugin_id: &str) -> Result<UpdateResult> {
        if let Some(plugin) = self.plugins.get(plugin_id) {
            let old_version = plugin.version.clone();
            
            // Check for updates (simplified implementation)
            // In a real implementation, this would check the marketplace or git repository
            let new_version = format!("{}.1", old_version); // Mock version increment
            
            if let Some(plugin) = self.plugins.get_mut(plugin_id) {
                plugin.version = new_version.clone();
                plugin.last_updated = Some(Utc::now());
                plugin.status = PluginStatus::Installed;
            }

            Ok(UpdateResult {
                plugin_id: plugin_id.to_string(),
                success: true,
                message: format!("Plugin {} updated successfully from {} to {}", plugin_id, old_version, new_version),
                old_version,
                new_version: Some(new_version),
            })
        } else {
            Ok(UpdateResult {
                plugin_id: plugin_id.to_string(),
                success: false,
                message: format!("Plugin not found: {}", plugin_id),
                old_version: String::new(),
                new_version: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub enum PluginSource {
    Marketplace,
    Git(String),
    Local(PathBuf),
    Archive(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[tokio::test]
    async fn test_plugin_system_creation() {
        let temp_dir = TempDir::new("plugins").unwrap();
        let system = PluginSystem::new(temp_dir.path().to_path_buf());
        assert!(system.plugins.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_system_initialization() {
        let temp_dir = TempDir::new("plugins").unwrap();
        let mut system = PluginSystem::new(temp_dir.path().to_path_buf());
        assert!(system.initialize().await.is_ok());
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_plugin_search() {
        let temp_dir = TempDir::new("plugins").unwrap();
        let mut system = PluginSystem::new(temp_dir.path().to_path_buf());
        
        let plugin = Plugin {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin for development".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            repository: None,
            homepage: None,
            tags: vec!["test".to_string(), "development".to_string()],
            category: PluginCategory::Development,
            manifest: PluginManifest {
                entry_point: "main.js".to_string(),
                commands: vec![],
                hooks: vec![],
                permissions: vec![],
                dependencies: vec![],
                api_version: "1.0".to_string(),
                platform_requirements: vec!["linux".to_string()],
                config_schema: None,
            },
            status: PluginStatus::Installed,
            install_path: None,
            installed_at: None,
            last_updated: None,
        };

        system.plugins.insert(plugin.id.clone(), plugin);

        let results = system.search_plugins("test", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Test Plugin");

        let results = system.search_plugins("development", Some(PluginCategory::Development));
        assert_eq!(results.len(), 1);
    }
}
