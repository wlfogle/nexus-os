use std::process::Command;

// ── Play Store Integration ──────────────────────────────────────────────────

#[tauri::command]
async fn search_play_store(query: String) -> Result<Vec<String>, String> {
    let output = Command::new("/home/loufogle/.local/bin/apksearch")
        .arg(&query)
        .output()
        .map_err(|e| format!("Failed to run apksearch: {}", e))?;

    if !output.status.success() {
        return Err(format!("apksearch error: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Robust parsing: collect lines, remove empty ones, and format
    let results: Vec<String> = stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| format!("{} [{}]", line, line)) // Wrap in brackets for regex
        .collect();

    Ok(results)
}

#[tauri::command]
async fn download_apk(package_id: String, folder: String) -> Result<String, String> {
    let _ = std::fs::create_dir_all(&folder);

    // apkeep -a <PACKAGE_ID> -d apk-pure <FOLDER_PATH>
    let output = Command::new("apkeep")
        .args(["-a", &package_id, "-d", "apk-pure", &folder])
        .output()
        .map_err(|e| format!("Failed to run apkeep: {}", e))?;

    if output.status.success() {
        Ok(format!("Downloaded {} to {}", package_id, folder))
    } else {
        Err(format!(
            "Download failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command]
async fn execute_stream_pipeline(package_id: String) -> Result<String, String> {
    let tmp_dir = "/tmp/gplay_stream_cache";
    let _ = std::fs::create_dir_all(tmp_dir);

    // Download APK to staging cache via apkeep
    let dl = Command::new("apkeep")
        .args(["-a", &package_id, "-d", "apk-pure", tmp_dir])
        .output()
        .map_err(|e| format!("Download error: {}", e))?;

    if !dl.status.success() {
        return Err(format!(
            "Download failed: {}",
            String::from_utf8_lossy(&dl.stderr)
        ));
    }

    // Locate the downloaded APK
    let apk_path = std::fs::read_dir(tmp_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .find(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".apk") && name.contains(&package_id)
        })
        .map(|e| e.path())
        .ok_or("Downloaded APK not found in cache")?;

    // Stream-install to device via ADB
    let install = Command::new("adb")
        .args(["install", "-r", "-g", apk_path.to_str().unwrap_or("")])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    let _ = std::fs::remove_file(&apk_path);

    if install.status.success() {
        Ok(format!("Installed {} on device", package_id))
    } else {
        Err(format!(
            "Install failed: {}",
            String::from_utf8_lossy(&install.stderr)
        ))
    }
}

// ── File Transfer ───────────────────────────────────────────────────────────

#[tauri::command]
async fn list_android_files(remote_path: String) -> Result<Vec<String>, String> {
    let output = Command::new("adb")
        .args(["shell", "ls", "-1", &remote_path])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to list files: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    Ok(files)
}

#[tauri::command]
async fn push_file(local_path: String, remote_path: String) -> Result<String, String> {
    let output = Command::new("adb")
        .args(["push", &local_path, &remote_path])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        Ok(format!("Pushed to {}", remote_path))
    } else {
        Err(format!(
            "Push failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command]
async fn pull_file(remote_path: String, local_path: String) -> Result<String, String> {
    let output = Command::new("adb")
        .args(["pull", &remote_path, &local_path])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        Ok(format!("Pulled {} to {}", remote_path, local_path))
    } else {
        Err(format!(
            "Pull failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

// ── APK Management ──────────────────────────────────────────────────────────

#[tauri::command]
async fn install_apk(path: String) -> Result<String, String> {
    let output = Command::new("adb")
        .args(["install", "-r", "-g", &path])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        let filename = std::path::Path::new(&path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        Ok(format!("Installed {}", filename))
    } else {
        Err(format!(
            "Install failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command]
async fn batch_install_apks(folder: String) -> Result<String, String> {
    let entries =
        std::fs::read_dir(&folder).map_err(|e| format!("Cannot read folder: {}", e))?;

    let mut installed = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|e| e == "apk").unwrap_or(false) {
            match Command::new("adb")
                .args(["install", "-r", "-g", path.to_str().unwrap_or("")])
                .output()
            {
                Ok(o) if o.status.success() => installed += 1,
                Ok(o) => {
                    failed += 1;
                    errors.push(format!(
                        "{}: {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        String::from_utf8_lossy(&o.stderr).trim()
                    ));
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!(
                        "{}: {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        e
                    ));
                }
            }
        }
    }

    if installed == 0 && failed == 0 {
        Err("No APK files found in the selected folder.".into())
    } else if failed > 0 {
        Err(format!(
            "Installed {}, failed {}:\n{}",
            installed,
            failed,
            errors.join("\n")
        ))
    } else {
        Ok(format!("Successfully installed {} APK(s).", installed))
    }
}

// ── Package Management ──────────────────────────────────────────────────────

#[tauri::command]
async fn list_packages() -> Result<Vec<String>, String> {
    let output = Command::new("adb")
        .args(["shell", "pm", "list", "packages"])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages: Vec<String> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("package:"))
        .map(|s| s.trim().to_string())
        .collect();
    packages.sort();
    Ok(packages)
}

#[tauri::command]
async fn purge_app_cache(package_id: String) -> Result<String, String> {
    let output = Command::new("adb")
        .args(["shell", "pm", "clear", &package_id])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        Ok(format!("Cleared data for {}", package_id))
    } else {
        Err(format!(
            "Clear failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

// ── Diagnostics & Interaction ───────────────────────────────────────────────

#[tauri::command]
async fn inject_text(text: String) -> Result<String, String> {
    // adb shell input text requires %s for spaces
    let escaped = text.replace(' ', "%s");
    let output = Command::new("adb")
        .args(["shell", "input", "text", &escaped])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        Ok(format!("Injected: {}", text))
    } else {
        Err(format!(
            "Injection failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command]
async fn capture_logcat() -> Result<String, String> {
    let output = Command::new("adb")
        .args(["logcat", "-d", "-t", "200"])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "Logcat failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command]
async fn capture_screenshot(save_path: String) -> Result<String, String> {
    let output = Command::new("adb")
        .args(["exec-out", "screencap", "-p"])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Screenshot failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    std::fs::write(&save_path, &output.stdout)
        .map_err(|e| format!("Failed to save: {}", e))?;

    Ok(format!("Screenshot saved to {}", save_path))
}
#[tauri::command]
async fn record_screen(save_path: String) -> Result<String, String> {
    let device_path = "/sdcard/adb_toolbox_record.mp4";

    // Record for 10 seconds on the device
    let record = Command::new("adb")
        .args(["shell", "screenrecord", "--time-limit", "10", device_path])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if !record.status.success() {
        return Err(format!(
            "Recording failed: {}",
            String::from_utf8_lossy(&record.stderr)
        ));
    }

    // Pull the recording to local filesystem
    let pull = Command::new("adb")
        .args(["pull", device_path, &save_path])
        .output()
        .map_err(|e| format!("Pull error: {}", e))?;

    // Clean up device file
    let _ = Command::new("adb")
        .args(["shell", "rm", device_path])
        .output();

    if pull.status.success() {
        Ok(format!("Recording saved to {}", save_path))
    } else {
        Err(format!(
            "Pull failed: {}",
            String::from_utf8_lossy(&pull.stderr)
        ))
    }
}

// ── Host Storage ────────────────────────────────────────────────────────────

#[tauri::command]
async fn copy_to_mount(source: String, mount_point: String) -> Result<String, String> {
    let src = std::path::Path::new(&source);
    let filename = src
        .file_name()
        .ok_or("Invalid source path")?
        .to_str()
        .ok_or("Invalid filename encoding")?;
    let dest = std::path::Path::new(&mount_point).join(filename);

    std::fs::copy(&source, &dest).map_err(|e| format!("Copy failed: {}", e))?;

    Ok(format!("Copied {} to {}", filename, dest.display()))
}

// ── Device Power Control ────────────────────────────────────────────────────

#[tauri::command]
async fn restart_framework() -> Result<String, String> {
    let output = Command::new("adb")
        .args(["shell", "su", "-c", "stop; sleep 1; start"])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        Ok("UI framework restarted.".into())
    } else {
        Err(format!(
            "Restart failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command]
async fn reboot_bootloader() -> Result<String, String> {
    let output = Command::new("adb")
        .args(["reboot", "bootloader"])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        Ok("Rebooting to bootloader...".into())
    } else {
        Err(format!(
            "Reboot failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command]
async fn reboot_recovery() -> Result<String, String> {
    let output = Command::new("adb")
        .args(["reboot", "recovery"])
        .output()
        .map_err(|e| format!("ADB error: {}", e))?;

    if output.status.success() {
        Ok("Rebooting to recovery...".into())
    } else {
        Err(format!(
            "Reboot failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

// ── App Entry Point ─────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            search_play_store,
            download_apk,
            execute_stream_pipeline,
            list_android_files,
            push_file,
            pull_file,
            install_apk,
            batch_install_apks,
            list_packages,
            purge_app_cache,
            inject_text,
            capture_logcat,
            capture_screenshot,
            record_screen,
            copy_to_mount,
            restart_framework,
            reboot_bootloader,
            reboot_recovery,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}