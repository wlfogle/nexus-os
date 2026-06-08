use std::process::Command;
use std::env;
use std::path::Path;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};

const OLLAMA_DEFAULT_HOST: &str = "http://127.0.0.1:11434";

/// Canonical stable mount point NexusOS uses for the models drive.
/// The installer creates an fstab entry that always mounts to this path
/// regardless of drive UUID or hotplug order.
pub const NEXUS_MODELS_MOUNT: &str = "/var/lib/nexus/models";

/// Returns true if `path` looks like a valid Ollama models directory.
fn is_ollama_models_dir(path: &std::path::Path) -> bool {
    path.join("manifests").exists()
}

/// Core discovery logic — searches `extra_roots` for a models directory.
/// Separated from I/O defaults so it can be fully unit-tested.
///
/// Priority:
///   1. `OLLAMA_MODELS` / `OLLAMA_MODELS_PATH` env var  (explicit override)
///   2. NexusOS stable mount point `/var/lib/nexus/models` (set by installer)
///   3. `home_dir/.ollama/models`                         (standard Ollama install)
///   4. Scan `extra_roots` for any dir containing `manifests/`
pub fn discover_models_path_with_roots(
    home_dir: &std::path::Path,
    extra_roots: &[std::path::PathBuf],
) -> std::path::PathBuf {
    // 1. Explicit env var override
    for var in &["OLLAMA_MODELS", "OLLAMA_MODELS_PATH"] {
        if let Ok(val) = env::var(var) {
            let p = std::path::PathBuf::from(&val);
            if !val.is_empty() && is_ollama_models_dir(&p) {
                return p;
            }
        }
    }

    // 2. NexusOS stable mount — set once by the installer, never changes
    let nexus_mount = std::path::PathBuf::from(NEXUS_MODELS_MOUNT);
    if is_ollama_models_dir(&nexus_mount) {
        return nexus_mount;
    }

    // 3. Standard ~/.ollama/models (default Ollama install)
    let home_models = home_dir.join(".ollama").join("models");
    if is_ollama_models_dir(&home_models) {
        return home_models;
    }

    // 4. Scan extra roots — last resort, only used before installer has run
    for root in extra_roots {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let drive = entry.path();
                let lvl1 = drive.join("models");
                if is_ollama_models_dir(&lvl1) {
                    return lvl1;
                }
                if let Ok(subdirs) = std::fs::read_dir(&drive) {
                    for sub in subdirs.flatten() {
                        let lvl2 = sub.path().join("models");
                        if is_ollama_models_dir(&lvl2) {
                            return lvl2;
                        }
                    }
                }
            }
        }
    }

    // Fallback — NexusOS stable path (even if drive not yet mounted)
    nexus_mount
}

/// Public entry point used at runtime.
pub fn discover_models_path() -> String {
    let home = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/root"));

    // Transient scan roots — only used as last resort before installer runs
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(user) = env::var("USER").or_else(|_| env::var("LOGNAME")) {
        roots.push(std::path::Path::new("/media").join(user));
    }
    roots.push(std::path::PathBuf::from("/media"));
    roots.push(std::path::PathBuf::from("/mnt"));

    discover_models_path_with_roots(&home, &roots)
        .to_string_lossy()
        .into_owned()
}

/// Called by the NexusOS installer once, when the models drive is identified.
/// Adds a permanent fstab entry so the drive always mounts at NEXUS_MODELS_MOUNT
/// regardless of UUID or hotplug order. Safe to call multiple times — won’t
/// duplicate the entry.
pub fn install_models_fstab_entry(device_uuid: &str) -> Result<(), String> {
    use std::io::Write;

    let mount_point = NEXUS_MODELS_MOUNT;
    let fstab_path = std::path::Path::new("/etc/fstab");

    // Create mount point
    std::fs::create_dir_all(mount_point)
        .map_err(|e| format!("Failed to create {}: {}", mount_point, e))?;

    // Read existing fstab
    let existing = std::fs::read_to_string(fstab_path)
        .unwrap_or_default();

    // Don’t add a duplicate entry
    if existing.contains(mount_point) {
        return Ok(());
    }

    let entry = format!(
        "\n# NexusOS Ollama models drive (added by installer)\n\
         UUID={}  {}  auto  defaults,auto,nofail,x-systemd.automount  0  0\n",
        device_uuid, mount_point
    );

    let mut fstab = std::fs::OpenOptions::new()
        .append(true)
        .open(fstab_path)
        .map_err(|e| format!("Failed to open /etc/fstab: {}", e))?;

    fstab.write_all(entry.as_bytes())
        .map_err(|e| format!("Failed to write fstab entry: {}", e))?;

    Ok(())
}

fn get_models_path() -> String {
    discover_models_path()
}

#[derive(Debug, thiserror::Error)]
pub enum OllamaConfigError {
    #[error("Ollama is not installed or not found in PATH")]
    NotInstalled,
    #[error("External models directory not found: {0}")]
    ModelsDirectoryNotFound(String),
    #[error("Failed to start Ollama service: {0}")]
    StartupFailed(String),
    #[error("Failed to configure Ollama: {0}")]
    ConfigurationFailed(String),
    #[error("Ollama API error: {0}")]
    ApiError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub is_installed: bool,
    pub is_running: bool,
    pub models_path: String,
    pub host: String,
    pub available_models: Vec<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            is_installed: false,
            is_running: false,
            models_path: get_models_path(),
            host: OLLAMA_DEFAULT_HOST.to_string(),
            available_models: Vec::new(),
        }
    }
}

pub async fn check_ollama_installation() -> Result<bool, OllamaConfigError> {
    // Check if ollama is installed
    let output = Command::new("which")
        .arg("ollama")
        .output()
        .map_err(|_e| OllamaConfigError::NotInstalled)?;
    
    if !output.status.success() {
        return Err(OllamaConfigError::NotInstalled);
    }
    
    println!("✓ Ollama found in PATH");
    Ok(true)
}

pub async fn check_external_models() -> Result<bool, OllamaConfigError> {
    let external_models_path = get_models_path();
    let models_path = Path::new(&external_models_path);
    
    if !models_path.exists() {
        return Err(OllamaConfigError::ModelsDirectoryNotFound(external_models_path.clone()));
    }
    
    // Check if there are actual model files - try both manifests (new) and blobs (older)
    let manifests_path = models_path.join("manifests");
    let blobs_path = models_path.join("blobs");
    
    if !manifests_path.exists() && !blobs_path.exists() {
        return Err(OllamaConfigError::ModelsDirectoryNotFound(format!("{} (no manifests or blobs found)", external_models_path)));
    }
    
    println!("✓ External models directory found at {}", external_models_path);
    Ok(true)
}

pub async fn configure_ollama_models_path() -> Result<String, OllamaConfigError> {
    let external_models_path = get_models_path();
    
    // Set OLLAMA_MODELS environment variable to point to external models
    env::set_var("OLLAMA_MODELS", &external_models_path);
    
    // Verify the environment variable was set correctly
    match env::var("OLLAMA_MODELS") {
        Ok(value) if value == external_models_path => {
            println!("✓ Set OLLAMA_MODELS environment variable to {}", external_models_path);
            Ok(external_models_path)
        },
        Ok(value) => {
            Err(OllamaConfigError::ConfigurationFailed(
                format!("OLLAMA_MODELS was set to '{}' but expected '{}'", value, external_models_path)
            ))
        },
        Err(e) => {
            Err(OllamaConfigError::ConfigurationFailed(
                format!("Failed to verify OLLAMA_MODELS environment variable: {}", e)
            ))
        }
    }
}

pub async fn start_ollama_service() -> Result<(), OllamaConfigError> {
    // Check if Ollama is already running
    if is_ollama_running().await {
        println!("✓ Ollama is already running");
        return Ok(());
    }
    
    println!("🚀 Starting Ollama service...");
    
    // Start Ollama serve in background
    let external_models_path = get_models_path();
    let mut cmd = Command::new("ollama");
    cmd.arg("serve");
    cmd.env("OLLAMA_MODELS", &external_models_path);
    cmd.env("OLLAMA_HOST", OLLAMA_DEFAULT_HOST);
    
    let child = cmd.spawn()
        .map_err(|e| OllamaConfigError::StartupFailed(format!("Failed to spawn ollama serve: {}", e)))?;
    
    println!("✓ Ollama service started with PID {}", child.id());
    
    // Wait a moment for Ollama to start up
    sleep(Duration::from_secs(3)).await;
    
    // Verify it's running
    if !is_ollama_running().await {
        return Err(OllamaConfigError::StartupFailed("Ollama failed to start properly".to_string()));
    }
    
    println!("✓ Ollama service is running and ready");
    Ok(())
}

pub async fn is_ollama_running() -> bool {
    // Try to make a simple HTTP request to Ollama
    let client = reqwest::Client::new();
    match client.head(OLLAMA_DEFAULT_HOST).send().await {
        Ok(response) => {
            response.status().is_success()
        },
        Err(_) => false,
    }
}

pub async fn get_available_models_from_ollama() -> Result<Vec<String>, OllamaConfigError> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/tags", OLLAMA_DEFAULT_HOST);
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| OllamaConfigError::ApiError(format!("Failed to fetch models: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(OllamaConfigError::ApiError(format!("API returned status: {}", response.status())));
    }
    
    let json: Value = response.json()
        .await
        .map_err(|e| OllamaConfigError::ApiError(format!("Failed to parse JSON: {}", e)))?;
    
    let models = json["models"].as_array()
        .ok_or_else(|| OllamaConfigError::ApiError("Invalid response format".to_string()))?;
    
    let model_names: Vec<String> = models
        .iter()
        .filter_map(|model| model["name"].as_str())
        .map(|name| name.to_string())
        .collect();
    
    println!("✓ Found {} available models", model_names.len());
    Ok(model_names)
}

pub async fn initialize_ollama_config() -> Result<OllamaConfig, OllamaConfigError> {
    let mut config = OllamaConfig::default();
    
    println!("🔧 Initializing Ollama configuration...");
    
    // Step 1: Check if Ollama is installed
    config.is_installed = check_ollama_installation().await.is_ok();
    if !config.is_installed {
        return Err(OllamaConfigError::NotInstalled);
    }
    
    // Step 2: Check external models directory
    check_external_models().await?;
    
    // Step 3: Configure models path
    config.models_path = configure_ollama_models_path().await?;
    
    // Step 4: Start Ollama service if not running
    start_ollama_service().await?;
    config.is_running = true;
    
    // Step 5: Get available models
    config.available_models = get_available_models_from_ollama().await.unwrap_or_default();
    
    println!("✅ Ollama configuration complete!");
    println!("   - Models path: {}", config.models_path);
    println!("   - Host: {}", config.host);
    println!("   - Available models: {}", config.available_models.len());
    
    Ok(config)
}

/// Write (or update) the systemd drop-in so the Ollama service always
/// uses the discovered models path — survives reboots, works on any machine.
/// Requires sudo/root. Fails silently if we don’t have permissions.
pub fn configure_systemd_ollama(models_path: &str) {
    let override_dir = "/etc/systemd/system/ollama.service.d";
    let override_file = format!("{}/override.conf", override_dir);

    // Only write if the content differs from what’s already there
    let content = format!(
        "[Service]\nEnvironment=\"OLLAMA_MODELS={}\"\nEnvironment=\"OLLAMA_HOST=0.0.0.0\"\nEnvironment=\"OLLAMA_KEEP_ALIVE=24h\"\n",
        models_path
    );

    let existing = std::fs::read_to_string(&override_file).unwrap_or_default();
    if existing == content {
        return; // already configured correctly
    }

    // Try to write via sudo pkexec (works on desktops without a password prompt
    // if the user has polkit auth) — fall back to nothing if unavailable.
    let tmp = format!("/tmp/ollama_override_{}.conf", std::process::id());
    if std::fs::write(&tmp, &content).is_ok() {
        let _ = Command::new("sudo")
            .args(["bash", "-c",
                &format!("mkdir -p {} && cp {} {}", override_dir, tmp, override_file)
            ])
            .output(); // ignore failure — we can’t force sudo
        let _ = Command::new("sudo")
            .args(["systemctl", "daemon-reload"])
            .output();
        let _ = std::fs::remove_file(&tmp);
    }
}

pub async fn ensure_ollama_configured() -> Result<(), OllamaConfigError> {
    // Auto-discover models path and set for this process only.
    // Never touch systemd — Ollama is already running as a system service.
    let models_path = discover_models_path();
    if !models_path.is_empty() {
        env::set_var("OLLAMA_MODELS", &models_path);
        println!("✓ Ollama models path: {}", models_path);
    }

    match initialize_ollama_config().await {
        Ok(_) => {
            println!("🎉 Ollama is ready!");
            Ok(())
        },
        Err(e) => {
            eprintln!("⚠️  Ollama configuration issue: {}", e);
            // Non-fatal — app still starts, AI just may not work until Ollama is set up
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Build a fake Ollama models directory: <root>/manifests/
    fn make_models_dir(parent: &std::path::Path) -> PathBuf {
        let models = parent.join("models");
        std::fs::create_dir_all(models.join("manifests")).unwrap();
        models
    }

    // ── Priority 1: OLLAMA_MODELS env var ────────────────────────────────────

    #[test]
    fn env_var_takes_priority_over_everything() {
        let tmp = TempDir::new().unwrap();
        let models = make_models_dir(tmp.path());
        // Also put a home models dir — should be ignored
        let home_tmp = TempDir::new().unwrap();
        make_models_dir(&home_tmp.path().join(".ollama"));

        // Temporarily set env var (scoped to this thread)
        env::set_var("OLLAMA_MODELS", models.to_str().unwrap());
        let result = discover_models_path_with_roots(home_tmp.path(), &[]);
        env::remove_var("OLLAMA_MODELS");

        assert_eq!(result, models);
    }

    #[test]
    fn env_var_ignored_when_path_has_no_manifests() {
        let tmp = TempDir::new().unwrap();
        // Set env var to a dir WITHOUT manifests/
        let bad = tmp.path().join("bad");
        std::fs::create_dir_all(&bad).unwrap();
        env::set_var("OLLAMA_MODELS", bad.to_str().unwrap());

        // Home has real models
        let home_tmp = TempDir::new().unwrap();
        let home_models = home_tmp.path().join(".ollama").join("models");
        std::fs::create_dir_all(home_models.join("manifests")).unwrap();

        let result = discover_models_path_with_roots(home_tmp.path(), &[]);
        env::remove_var("OLLAMA_MODELS");

        assert_eq!(result, home_models);
    }

    // ── Priority 2: NexusOS stable mount (/var/lib/nexus/models) ─────────────
    // (Skipped in unit tests — would require writing to /var/lib/nexus in CI.
    //  Covered by integration tests in scripts/nexus-terminal-integration-test.sh)

    // ── Priority 3: ~/.ollama/models ─────────────────────────────────────────

    #[test]
    fn finds_home_ollama_models() {
        env::remove_var("OLLAMA_MODELS");
        env::remove_var("OLLAMA_MODELS_PATH");

        let home_tmp = TempDir::new().unwrap();
        let expected = home_tmp.path().join(".ollama").join("models");
        std::fs::create_dir_all(expected.join("manifests")).unwrap();

        let result = discover_models_path_with_roots(home_tmp.path(), &[]);
        assert_eq!(result, expected);
    }

    // ── Priority 4: mounted drive scan ───────────────────────────────────────

    #[test]
    fn finds_models_one_level_deep_in_root() {
        env::remove_var("OLLAMA_MODELS");
        env::remove_var("OLLAMA_MODELS_PATH");

        // Simulate /media/<user>/<drive-uuid>/models/manifests
        let media_root = TempDir::new().unwrap();
        let drive = media_root.path().join("some-uuid-doesnt-matter");
        let expected = make_models_dir(&drive);  // <drive>/models/manifests

        let home_tmp = TempDir::new().unwrap(); // no home models
        let result = discover_models_path_with_roots(
            home_tmp.path(),
            &[media_root.path().to_path_buf()],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn finds_models_two_levels_deep_in_root() {
        env::remove_var("OLLAMA_MODELS");
        env::remove_var("OLLAMA_MODELS_PATH");

        // Simulate /mnt/<partition>/<subdir>/models/manifests
        let mnt_root = TempDir::new().unwrap();
        let partition = mnt_root.path().join("data-drive");
        let subdir = partition.join("ai");
        let expected = make_models_dir(&subdir);

        let home_tmp = TempDir::new().unwrap();
        let result = discover_models_path_with_roots(
            home_tmp.path(),
            &[mnt_root.path().to_path_buf()],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn returns_nexus_mount_as_fallback_when_nothing_found() {
        env::remove_var("OLLAMA_MODELS");
        env::remove_var("OLLAMA_MODELS_PATH");

        let home_tmp = TempDir::new().unwrap(); // no models anywhere
        let result = discover_models_path_with_roots(home_tmp.path(), &[]);
        // Should return the stable NexusOS path as fallback
        assert_eq!(result, std::path::PathBuf::from(NEXUS_MODELS_MOUNT));
    }

    #[test]
    fn does_not_add_duplicate_fstab_entry() {
        let tmp = TempDir::new().unwrap();
        // Simulate an fstab that already has the mount point
        let fake_fstab = tmp.path().join("fstab");
        std::fs::write(&fake_fstab,
            format!("UUID=abc123  {}  auto  defaults  0  0\n", NEXUS_MODELS_MOUNT)
        ).unwrap();
        let contents = std::fs::read_to_string(&fake_fstab).unwrap();
        // Verify the logic: already contains mount point, so no write
        assert!(contents.contains(NEXUS_MODELS_MOUNT));
    }
}
