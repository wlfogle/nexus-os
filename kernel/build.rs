//! NexusOS kernel build script.
//!
//! Assembles `src/userspace/shell_init.asm` (NASM flat binary) into
//! `$OUT_DIR/shell_init.bin`, which is embedded into the kernel via
//! `include_bytes!` in `src/userspace/mod.rs`.
//!
//! Prerequisites: `nasm` must be on PATH.
//!   Pop!_OS / Ubuntu:  sudo nala install nasm
//!   Arch:              sudo pacman -S nasm

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let asm_src = PathBuf::from("src/userspace/shell_init.asm");
    let bin_out = out_dir.join("shell_init.bin");

    // Re-run the build script whenever the source changes.
    println!("cargo:rerun-if-changed=src/userspace/shell_init.asm");
    println!("cargo:rerun-if-changed=build.rs");

    let status = Command::new("nasm")
        .args([
            "-f",
            "bin",
            "-o",
            bin_out.to_str().expect("OUT_DIR path is not valid UTF-8"),
            asm_src.to_str().expect("asm_src path is not valid UTF-8"),
        ])
        .status()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to run nasm: {e}\n\
                 Install it with:  sudo nala install nasm"
            )
        });

    assert!(
        status.success(),
        "nasm failed to assemble src/userspace/shell_init.asm — \
         see error output above"
    );
}
