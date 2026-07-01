#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::env;

#[tauri::command]
fn run_mobalivecd() -> String {
    // 1. Define the directory and script path
    let work_dir = "/home/loufogle/nexus-os/packages/mobalivecd-linux/mobalivecd-linux";
    let script_path = format!("{}/mobalivecd.py", work_dir);

    // 2. Execute the script within its own directory context
    let output = Command::new("python3")
    .arg(script_path)
    .current_dir(work_dir)
    .output();

    // 3. Handle the result
    match output {
        Ok(out) => {
            if out.status.success() {
                String::from_utf8_lossy(&out.stdout).to_string()
            } else {
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
