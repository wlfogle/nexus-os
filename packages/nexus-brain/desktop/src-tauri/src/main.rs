// Suppress the extra console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|a| a == "--compare") {
        std::process::exit(librarian_lib::compare::cli(
            std::env::args().skip(1).collect(),
        ));
    }
    // `--headless` runs the same pipeline the window drives, printing progress
    // instead of emitting events. Useful from cron, and it makes the engine
    // verifiable without a display.
    let headless = std::env::args()
        .skip(1)
        .any(|a| matches!(a.as_str(), "--headless" | "--run" | "-H"));

    if headless {
        std::process::exit(librarian_lib::run_headless());
    }

    librarian_lib::run()
}
