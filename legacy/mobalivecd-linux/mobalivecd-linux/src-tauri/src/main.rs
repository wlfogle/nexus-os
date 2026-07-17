#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::env;

#[tauri::command]
fn run_mobalivecd() -> String {
    // 1. Corrected path: removing the redundant "mobalivecd-linux/" segment
    let work_dir = "/home/loufogle/nexus-os/packages/mobalivecd-linux";
    let script_path = format!("{}/enhanced_mobalivecd.py", work_dir);

    // 2. Execute
    let output = std::process::Command::new("python3")
    .arg(script_path)
    .current_dir(work_dir)
    .output();

    // 3. Handle result
    match output {
        Ok(out) => {
            if out.status.success() {
                String::from_utf8_lossy(&out.stdout).to_string()
            } else {
                // This will tell us if Python found the file but failed to run it
                format!("Script Error ({}): {}", out.status, String::from_utf8_lossy(&out.stderr))
            }
        }
        Err(e) => format!("Failed to invoke python3: {}", e),
    }
}

fn main() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![run_mobalivecd])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
