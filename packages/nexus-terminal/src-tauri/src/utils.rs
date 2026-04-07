use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::process::Command;
use walkdir::WalkDir;
use ignore::WalkBuilder;

pub fn get_system_info() -> Result<HashMap<String, String>> {
    let mut info = HashMap::new();
    
    // OS Information
    info.insert("os".to_string(), std::env::consts::OS.to_string());
    info.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    
    // Hostname
    if let Ok(hostname) = std::process::Command::new("hostname").output() {
        if let Ok(hostname_str) = String::from_utf8(hostname.stdout) {
            info.insert("hostname".to_string(), hostname_str.trim().to_string());
        }
    }
    
    // User
    if let Ok(user) = std::env::var("USER") {
        info.insert("user".to_string(), user);
    } else if let Ok(user) = std::env::var("USERNAME") {
        info.insert("user".to_string(), user);
    }
    
    // Shell
    if let Ok(shell) = std::env::var("SHELL") {
        info.insert("shell".to_string(), shell);
    }
    
    // Current directory
    if let Ok(cwd) = std::env::current_dir() {
        info.insert("cwd".to_string(), cwd.display().to_string());
    }
    
    Ok(info)
}

pub async fn search_files(query: &str, path: &str, include_content: bool) -> Result<Vec<String>> {
    let mut results = Vec::new();
    
    let walker = WalkBuilder::new(path)
        .hidden(false)
        .git_ignore(true)
        .build();
    
    for entry in walker {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.is_file() {
            let path_str = path.display().to_string();
            
            // Search in filename
            if path_str.to_lowercase().contains(&query.to_lowercase()) {
                results.push(format!("file:{}", path_str));
                continue;
            }
            
            // Search in file content if requested
            if include_content {
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    if content.to_lowercase().contains(&query.to_lowercase()) {
                        results.push(format!("content:{}", path_str));
                    }
                }
            }
        }
    }
    
    Ok(results)
}

pub fn get_file_tree(path: &str, max_depth: Option<usize>) -> Result<String> {
    let mut tree = String::new();
    let walker = WalkDir::new(path);
    
    let walker = if let Some(depth) = max_depth {
        walker.max_depth(depth)
    } else {
        walker
    };
    
    for entry in walker {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let depth = entry.depth();
        
        // Create indentation based on depth
        let indent = "  ".repeat(depth);
        let name = path.file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| "?".into());
        
        if path.is_dir() {
            tree.push_str(&format!("{}📁 {}/\n", indent, name));
        } else {
            let icon = get_file_icon(&name);
            tree.push_str(&format!("{}{} {}\n", indent, icon, name));
        }
    }
    
    Ok(tree)
}

fn get_file_icon(filename: &str) -> &'static str {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    match ext.to_lowercase().as_str() {
        "rs" => "🦀",
        "js" | "ts" => "📜",
        "vue" => "💚",
        "py" => "🐍",
        "go" => "🐹",
        "java" => "☕",
        "cpp" | "c" => "⚡",
        "html" => "🌐",
        "css" => "🎨",
        "md" => "📝",
        "json" => "📋",
        "toml" | "yaml" | "yml" => "⚙️",
        "sh" | "bash" => "📟",
        "git" => "🌿",
        "lock" => "🔒",
        _ => "📄",
    }
}

pub async fn execute_safe_command(command: &str) -> Result<String> {
    let safe_commands = [
        "ls", "pwd", "whoami", "date", "uname", "df", "free", "ps", "top",
        "git status", "git log", "git branch", "git diff",
        "npm list", "cargo --version", "rustc --version",
        "node --version", "python --version"
    ];
    
    // Check if command is in safe list
    let is_safe = safe_commands.iter().any(|&safe_cmd| {
        command.trim().starts_with(safe_cmd)
    });
    
    if !is_safe {
        return Err(anyhow::anyhow!("Command not in safe list: {}", command));
    }
    
    // Parse command and arguments
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }
    
    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    
    let output = cmd.output().await
        .context("Failed to execute command")?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Command failed: {}", error))
    }
}

pub async fn get_detailed_system_info() -> Result<String> {
    let mut info = String::new();
    
    // OS and Architecture
    info.push_str(&format!("Operating System: {} {}\n", std::env::consts::OS, std::env::consts::ARCH));
    
    // Kernel version (Linux)
    if let Ok(output) = Command::new("uname").arg("-r").output().await {
        if output.status.success() {
            let kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
            info.push_str(&format!("Kernel Version: {}\n", kernel));
        }
    }
    
    // Distribution info (Linux)
    let os_release_path = std::env::var("OS_RELEASE_PATH")
        .unwrap_or_else(|_| "/etc/os-release".to_string());
    if std::path::Path::new(&os_release_path).exists() {
        if let Ok(content) = tokio::fs::read_to_string(&os_release_path).await {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    let distro = line.split('=').nth(1).unwrap_or("Unknown")
                        .trim_matches('"').to_string();
                    info.push_str(&format!("Distribution: {}\n", distro));
                    break;
                }
            }
        }
    }
    
    // Memory info
    if let Ok(output) = Command::new("free").arg("-h").output().await {
        if output.status.success() {
            let memory_info = String::from_utf8_lossy(&output.stdout);
            info.push_str(&format!("Memory Info:\n{}\n", memory_info));
        }
    }
    
    // CPU info
    if let Ok(output) = Command::new("lscpu").output().await {
        if output.status.success() {
            let cpu_info = String::from_utf8_lossy(&output.stdout);
            for line in cpu_info.lines() {
                if line.contains("Model name:") {
                    info.push_str(&format!("CPU: {}\n", line.split(':').nth(1).unwrap_or("").trim()));
                    break;
                }
            }
        }
    }
    
    // Disk space
    if let Ok(output) = Command::new("df").arg("-h").arg("/").output().await {
        if output.status.success() {
            let disk_info = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = disk_info.lines().nth(1) {
                info.push_str(&format!("Root Disk Usage: {}\n", line));
            }
        }
    }
    
    Ok(info)
}

pub async fn analyze_project_structure(project_path: &str) -> Result<String> {
    let mut analysis = String::new();
    let path = std::path::Path::new(project_path);
    
    if !path.exists() {
        return Err(anyhow::anyhow!("Project path does not exist: {}", project_path));
    }
    
    analysis.push_str(&format!("Project Analysis for: {}\n\n", project_path));
    
    // Check for common project files
    let project_files = [
        ("package.json", "Node.js/JavaScript Project"),
        ("Cargo.toml", "Rust Project"),
        ("pyproject.toml", "Python Project (Modern)"),
        ("requirements.txt", "Python Project (Classic)"),
        ("go.mod", "Go Project"),
        ("pom.xml", "Java Maven Project"),
        ("build.gradle", "Java Gradle Project"),
        ("composer.json", "PHP Project"),
        ("Gemfile", "Ruby Project"),
        ("Makefile", "C/C++/Make Project"),
        ("CMakeLists.txt", "CMake Project"),
    ];
    
    analysis.push_str("Project Type Detection:\n");
    for (file, description) in &project_files {
        if path.join(file).exists() {
            analysis.push_str(&format!("✓ {} - {}\n", file, description));
        }
    }
    analysis.push_str("\n");
    
    // Directory structure overview
    analysis.push_str("Directory Structure:\n");
    if let Ok(file_tree) = get_file_tree(project_path, Some(2)) {
        analysis.push_str(&file_tree);
    }
    
    // Git status if it's a repo
    if git2::Repository::open(project_path).is_ok() {
        analysis.push_str("\nGit Repository Status:\n");
        
        // Get current branch
        if let Ok(repo) = git2::Repository::open(project_path) {
            if let Ok(head) = repo.head() {
                if let Some(name) = head.shorthand() {
                    analysis.push_str(&format!("Branch: {}\n", name));
                } else {
                    analysis.push_str("Branch: HEAD\n");
                }
            }
            
            // Get repository status
            if let Ok(statuses) = repo.statuses(None) {
                if statuses.is_empty() {
                    analysis.push_str("Status: Clean\n");
                } else {
                    analysis.push_str(&format!("Status: {} changes\n", statuses.len()));
                }
            }
        }
    }
    
    Ok(analysis)
}

pub async fn get_service_status(service_name: &str) -> Result<String> {
    let output = Command::new("systemctl")
        .arg("status")
        .arg(service_name)
        .arg("--no-pager")
        .output()
        .await
        .context("Failed to execute systemctl status")?;
    
    let status_output = String::from_utf8_lossy(&output.stdout);
    let error_output = String::from_utf8_lossy(&output.stderr);
    
    if !error_output.is_empty() {
        Ok(format!("Status Output:\n{}\nError Output:\n{}", status_output, error_output))
    } else {
        Ok(status_output.to_string())
    }
}

pub async fn get_service_logs(service_name: &str) -> Result<String> {
    let output = Command::new("journalctl")
        .arg("-u")
        .arg(service_name)
        .arg("--no-pager")
        .arg("-n")
        .arg("50")
        .output()
        .await
        .context("Failed to execute journalctl")?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Failed to get service logs: {}", error))
    }
}

pub async fn get_environment_info() -> Result<String> {
    let mut info = String::new();
    
    // Shell information
    if let Ok(shell) = std::env::var("SHELL") {
        info.push_str(&format!("Shell: {}\n", shell));
    }
    
    // PATH
    if let Ok(path) = std::env::var("PATH") {
        info.push_str("PATH directories:\n");
        for dir in path.split(':') {
            info.push_str(&format!("  {}\n", dir));
        }
    }
    
    // Common development tools
    let dev_tools = [
        "rustc", "cargo", "node", "npm", "python3", "python", "pip", "git",
        "gcc", "g++", "make", "cmake", "docker", "docker-compose", "kubectl"
    ];
    
    info.push_str("\nAvailable Development Tools:\n");
    for tool in &dev_tools {
        match Command::new("which").arg(tool).output().await {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                info.push_str(&format!("✓ {}: {}\n", tool, path));
            }
            _ => {
                info.push_str(&format!("✗ {}: not found\n", tool));
            }
        }
    }
    
    Ok(info)
}

pub async fn get_desktop_environment() -> Result<String> {
    // Check common desktop environment variables
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        return Ok(de);
    }
    
    if let Ok(de) = std::env::var("DESKTOP_SESSION") {
        return Ok(de);
    }
    
    if let Ok(de) = std::env::var("GDMSESSION") {
        return Ok(de);
    }
    
    // Check for specific desktop processes
    let desktop_processes = [
        ("gnome-shell", "GNOME"),
        ("kwin", "KDE Plasma"),
        ("xfwm4", "XFCE"),
        ("i3", "i3"),
        ("awesome", "Awesome"),
        ("openbox", "Openbox"),
    ];
    
    for (process, de_name) in &desktop_processes {
        match Command::new("pgrep").arg(process).output().await {
            Ok(output) if output.status.success() => {
                return Ok(de_name.to_string());
            }
            _ => continue,
        }
    }
    
    Ok("Unknown".to_string())
}

pub async fn get_network_config() -> Result<String> {
    let mut config = String::new();
    
    // Network interfaces
    match Command::new("ip").arg("addr").arg("show").output().await {
        Ok(output) if output.status.success() => {
            let interfaces = String::from_utf8_lossy(&output.stdout);
            config.push_str("Network Interfaces:\n");
            config.push_str(&interfaces);
            config.push_str("\n");
        }
        _ => {
            config.push_str("Could not get network interface information\n");
        }
    }
    
    // DNS configuration
    if std::path::Path::new("/etc/resolv.conf").exists() {
        match tokio::fs::read_to_string("/etc/resolv.conf").await {
            Ok(resolv_conf) => {
                config.push_str("DNS Configuration (/etc/resolv.conf):\n");
                config.push_str(&resolv_conf);
                config.push_str("\n");
            }
            Err(_) => {
                config.push_str("Could not read DNS configuration\n");
            }
        }
    }
    
    // Default route
    match Command::new("ip").arg("route").arg("show").arg("default").output().await {
        Ok(output) if output.status.success() => {
            let route = String::from_utf8_lossy(&output.stdout);
            config.push_str("Default Route:\n");
            config.push_str(&route);
        }
        _ => {
            config.push_str("Could not get default route information\n");
        }
    }
    
    Ok(config)
}

pub async fn analyze_file_permissions(file_path: &str) -> Result<String> {
    let mut analysis = String::new();
    let path = std::path::Path::new(file_path);
    
    analysis.push_str(&format!("File Permission Analysis for: {}\n\n", file_path));
    
    if !path.exists() {
        return Ok(format!("File does not exist: {}", file_path));
    }
    
    // Get detailed file information
    match Command::new("ls").arg("-la").arg(file_path).output().await {
        Ok(output) if output.status.success() => {
            let ls_output = String::from_utf8_lossy(&output.stdout);
            analysis.push_str("Detailed file information:\n");
            analysis.push_str(&ls_output);
            analysis.push_str("\n");
        }
        _ => {
            analysis.push_str("Could not get detailed file information\n");
        }
    }
    
    // Get file ownership and group
    match Command::new("stat").arg("-c").arg("%U:%G %a %n").arg(file_path).output().await {
        Ok(output) if output.status.success() => {
            let stat_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            analysis.push_str(&format!("Owner:Group Permissions: {}\n", stat_output));
        }
        _ => {
            analysis.push_str("Could not get file ownership information\n");
        }
    }
    
    // Check if current user can access
    let current_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    analysis.push_str(&format!("\nCurrent user: {}\n", current_user));
    
    // Check access permissions
    let access_checks = [
        ("-r", "readable"),
        ("-w", "writable"),
        ("-x", "executable"),
    ];
    
    for (flag, description) in &access_checks {
        match Command::new("test").arg(flag).arg(file_path).output().await {
            Ok(output) if output.status.success() => {
                analysis.push_str(&format!("✓ File is {} by current user\n", description));
            }
            _ => {
                analysis.push_str(&format!("✗ File is not {} by current user\n", description));
            }
        }
    }
    
    // Check if it's in a directory with proper permissions
    if let Some(parent) = path.parent() {
        analysis.push_str(&format!("\nParent directory: {}\n", parent.display()));
        match Command::new("ls").arg("-ld").arg(parent).output().await {
            Ok(output) if output.status.success() => {
                let parent_info = String::from_utf8_lossy(&output.stdout);
                analysis.push_str(&format!("Parent directory permissions: {}", parent_info));
            }
            _ => {
                analysis.push_str("Could not get parent directory permissions\n");
            }
        }
    }
    
    Ok(analysis)
}
