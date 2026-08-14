//! Standalone lid-monitor binary.
//!
//! Today this is a genuinely working `--clear-once` tool (real
//! functionality, not a stub): it opens the RGB device via `clevo-hw` and
//! clears every key, exactly the recovery action the Python
//! `lid-monitor-daemon.py` took on lid close. osd-lidmonitor-agent (per the
//! migration plan) extends this into the actual lid-state polling loop
//! (mirroring `_check_lid_state()`'s multi-method detection: ACPI
//! `/proc/acpi/button/lid/*/state` first, since it's confirmed present and
//! reliable on this hardware) and installs it as a systemd --user service.

use clevo_hw::RgbController;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("--clear-once") {
        eprintln!("Usage: occ-lid-monitor --clear-once");
        eprintln!("(the persistent lid-watching loop is not yet implemented - see CONTRACT.md)");
        std::process::exit(2);
    }

    let controller = RgbController::new();
    match controller.clear_all_keys() {
        Ok(()) => println!("RGB keyboard cleared."),
        Err(e) => {
            eprintln!("Failed to clear RGB keyboard: {e}");
            std::process::exit(1);
        }
    }
}
