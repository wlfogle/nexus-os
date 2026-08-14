// Prevents additional console window on Windows in release, has no effect on Linux.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    occ_lib::run();
}
