mod adb;

use adb_client::ADBDeviceExt;
use tokio::process::Command as TokioCommand;

// ── Play Store Integration ──────────────────────────────────────────────────

#[tauri::command]
async fn search_play_store(query: String) -> Result<Vec<String>, String> {
    let output = TokioCommand::new("/home/loufogle/.local/bin/apksearch")
        .arg(&query)
        .output()
        .await
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
    let output = TokioCommand::new("apkeep")
        .args(["-a", &package_id, "-d", "apk-pure", &folder])
        .output()
        .await
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
async fn execute_stream_pipeline(package_id: String, device_id: Option<String>) -> Result<String, String> {
    let tmp_dir = "/tmp/gplay_stream_cache";
    let _ = std::fs::create_dir_all(tmp_dir);

    // Download APK to staging cache via apkeep
    let dl = TokioCommand::new("apkeep")
        .args(["-a", &package_id, "-d", "apk-pure", tmp_dir])
        .output()
        .await
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

    // Stream-install to device via the native ADB protocol
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let filename = apk_path
            .file_name()
            .ok_or("Invalid APK path")?
            .to_string_lossy()
            .to_string();
        let apk_path_str = apk_path
            .to_str()
            .ok_or("Invalid APK path encoding")?
            .to_string();

        let install_result = adb::install_apk_on_device(&mut device, &apk_path_str, &filename);
        let _ = std::fs::remove_file(&apk_path);
        install_result
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(format!("Installed {} on device", package_id))
}

// ── File Transfer ───────────────────────────────────────────────────────────

#[tauri::command]
async fn list_android_files(remote_path: String, device_id: Option<String>) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let items = device
            .list(&remote_path)
            .map_err(|e| format!("Failed to list files: {}", e))?;
        Ok(items.iter().map(adb::list_item_name).collect())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn push_file(local_path: String, remote_path: String, device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let mut file = std::fs::File::open(&local_path)
            .map_err(|e| format!("Failed to open local file: {}", e))?;
        device
            .push(&mut file, &remote_path)
            .map_err(|e| format!("Push failed: {}", e))?;
        Ok(format!("Pushed to {}", remote_path))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn pull_file(remote_path: String, local_path: String, device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let mut file = std::fs::File::create(&local_path)
            .map_err(|e| format!("Failed to create local file: {}", e))?;
        device
            .pull(&remote_path, &mut file)
            .map_err(|e| format!("Pull failed: {}", e))?;
        Ok(format!("Pulled {} to {}", remote_path, local_path))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ── APK Management ──────────────────────────────────────────────────────────

#[tauri::command]
async fn install_apk(path: String, device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let filename = std::path::Path::new(&path)
            .file_name()
            .ok_or("Invalid APK path")?
            .to_string_lossy()
            .to_string();
        adb::install_apk_on_device(&mut device, &path, &filename)?;
        Ok(format!("Installed {}", filename))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn batch_install_apks(folder: String, device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let entries =
            std::fs::read_dir(&folder).map_err(|e| format!("Cannot read folder: {}", e))?;
        let mut device = adb::open_device(device_id)?;

        let mut installed = 0u32;
        let mut failed = 0u32;
        let mut errors: Vec<String> = Vec::new();

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "apk").unwrap_or(false) {
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let local_path = path.to_string_lossy().to_string();
                match adb::install_apk_on_device(&mut device, &local_path, &filename) {
                    Ok(()) => installed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", filename, e));
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
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ── Package Management ──────────────────────────────────────────────────────

#[tauri::command]
async fn list_packages(device_id: Option<String>) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = device
            .shell_command(&"pm list packages", Some(&mut stdout), Some(&mut stderr))
            .map_err(|e| format!("ADB error: {}", e))?;

        if code.unwrap_or(0) != 0 {
            return Err(format!("Failed: {}", String::from_utf8_lossy(&stderr)));
        }

        let text = String::from_utf8_lossy(&stdout);
        let mut packages: Vec<String> = text
            .lines()
            .filter_map(|line| line.strip_prefix("package:"))
            .map(|s| s.trim().to_string())
            .collect();
        packages.sort();
        Ok(packages)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn purge_app_cache(package_id: String, device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let cmd = format!("pm clear {}", package_id);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = device
            .shell_command(&cmd, Some(&mut stdout), Some(&mut stderr))
            .map_err(|e| format!("ADB error: {}", e))?;

        if code.unwrap_or(0) == 0 {
            Ok(format!("Cleared data for {}", package_id))
        } else {
            let stderr_text = String::from_utf8_lossy(&stderr);
            let message = if stderr_text.trim().is_empty() {
                String::from_utf8_lossy(&stdout).into_owned()
            } else {
                stderr_text.into_owned()
            };
            Err(format!("Clear failed: {}", message.trim()))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ── Diagnostics & Interaction ───────────────────────────────────────────────

#[tauri::command]
async fn inject_text(text: String, device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        // adb shell input text requires %s for spaces
        let escaped = text.replace(' ', "%s");
        let cmd = format!("input text {}", escaped);
        let mut stderr = Vec::new();
        let code = device
            .shell_command(&cmd, None, Some(&mut stderr))
            .map_err(|e| format!("ADB error: {}", e))?;

        if code.unwrap_or(0) == 0 {
            Ok(format!("Injected: {}", text))
        } else {
            Err(format!(
                "Injection failed: {}",
                String::from_utf8_lossy(&stderr).trim()
            ))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn capture_logcat(device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = device
            .shell_command(&"logcat -d -t 200", Some(&mut stdout), Some(&mut stderr))
            .map_err(|e| format!("ADB error: {}", e))?;

        if code.unwrap_or(0) == 0 {
            Ok(String::from_utf8_lossy(&stdout).into_owned())
        } else {
            Err(format!("Logcat failed: {}", String::from_utf8_lossy(&stderr)))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn capture_screenshot(save_path: String, device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = device
            .shell_command(&"screencap -p", Some(&mut stdout), Some(&mut stderr))
            .map_err(|e| format!("ADB error: {}", e))?;

        if code.unwrap_or(0) != 0 {
            return Err(format!(
                "Screenshot failed: {}",
                String::from_utf8_lossy(&stderr)
            ));
        }

        std::fs::write(&save_path, &stdout).map_err(|e| format!("Failed to save: {}", e))?;

        Ok(format!("Screenshot saved to {}", save_path))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn record_screen(save_path: String, device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let device_path = "/sdcard/adb_toolbox_record.mp4";

        // Record for 10 seconds on the device
        let record_cmd = format!("screenrecord --time-limit 10 {}", device_path);
        let mut stderr = Vec::new();
        let code = device
            .shell_command(&record_cmd, None, Some(&mut stderr))
            .map_err(|e| format!("ADB error: {}", e))?;

        if code.unwrap_or(0) != 0 {
            return Err(format!(
                "Recording failed: {}",
                String::from_utf8_lossy(&stderr)
            ));
        }

        // Pull the recording to local filesystem
        let mut file = std::fs::File::create(&save_path)
            .map_err(|e| format!("Failed to create local file: {}", e))?;
        let pull_result = device.pull(&device_path, &mut file);

        // Clean up device file regardless of pull outcome
        let cleanup_cmd = format!("rm {}", device_path);
        let _ = device.shell_command(&cleanup_cmd, None, None);

        pull_result.map_err(|e| format!("Pull failed: {}", e))?;

        Ok(format!("Recording saved to {}", save_path))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ── Host Storage ────────────────────────────────────────────────────────────

#[tauri::command]
async fn copy_to_mount(source: String, mount_point: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let src = std::path::Path::new(&source);
        let filename = src
            .file_name()
            .ok_or("Invalid source path")?
            .to_str()
            .ok_or("Invalid filename encoding")?;
        let dest = std::path::Path::new(&mount_point).join(filename);

        std::fs::copy(&source, &dest).map_err(|e| format!("Copy failed: {}", e))?;

        Ok(format!("Copied {} to {}", filename, dest.display()))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ── Device Power Control ────────────────────────────────────────────────────

#[tauri::command]
async fn restart_framework(device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        let mut stderr = Vec::new();
        let code = device
            .shell_command(&"su -c 'stop; sleep 1; start'", None, Some(&mut stderr))
            .map_err(|e| format!("ADB error: {}", e))?;

        if code.unwrap_or(0) == 0 {
            Ok("UI framework restarted.".to_string())
        } else {
            Err(format!(
                "Restart failed: {}",
                String::from_utf8_lossy(&stderr)
            ))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn reboot_bootloader(device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        device
            .reboot(adb_client::RebootType::Bootloader)
            .map_err(|e| format!("Reboot failed: {}", e))?;
        Ok("Rebooting to bootloader...".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn reboot_recovery(device_id: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut device = adb::open_device(device_id)?;
        device
            .reboot(adb_client::RebootType::Recovery)
            .map_err(|e| format!("Reboot failed: {}", e))?;
        Ok("Rebooting to recovery...".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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
            adb::list_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
