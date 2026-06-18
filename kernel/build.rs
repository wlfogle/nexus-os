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
    println!("cargo:rerun-if-changed=src/userspace/hello.asm");
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

    // ── Reference user program: hello.elf (real static ELF64) ────────────────
    // Assembled as an ELF object then linked at a fixed high base in the user
    // half (PML4[1]) so the ELF loader can map it without colliding with the
    // resident shell.  Embedded into the kernel and written to the ESP by the
    // installer so `run HELLO.ELF` works on an installed system.
    let hello_src = PathBuf::from("src/userspace/hello.asm");
    let hello_obj = out_dir.join("hello.o");
    let hello_elf = out_dir.join("hello.elf");

    let nasm_elf = Command::new("nasm")
        .args([
            "-f", "elf64",
            "-o", hello_obj.to_str().expect("OUT_DIR path is not valid UTF-8"),
            hello_src.to_str().expect("hello_src path is not valid UTF-8"),
        ])
        .status()
        .unwrap_or_else(|e| panic!("Failed to run nasm for hello.asm: {e}"));
    assert!(nasm_elf.success(), "nasm failed to assemble src/userspace/hello.asm");

    let ld = Command::new("ld")
        .args([
            "-N",                       // single writable load segment, no page-align gaps
            "-Ttext=0x8040000000",      // link base = exec::EXEC_BASE (PML4[1], PDPT[1])
            "-e", "_start",
            "-melf_x86_64",
            "-o", hello_elf.to_str().expect("OUT_DIR path is not valid UTF-8"),
            hello_obj.to_str().expect("OUT_DIR path is not valid UTF-8"),
        ])
        .status()
        .unwrap_or_else(|e| panic!("Failed to run ld for hello.elf: {e}"));
    assert!(ld.success(), "ld failed to link hello.elf — see error output above");
}
