# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

NexusOS is the world's first **AI-Native Operating System** - built from scratch with AI/ML workloads as first-class citizens. Unlike traditional OS designs that treat AI as user-space applications, NexusOS integrates AI capabilities directly into the kernel.

**AI-Powerhouse Architecture**: Based on the comprehensive AI development environment, NexusOS brings AI/ML capabilities, GPU acceleration, container orchestration, and self-hosting services to the kernel level.

**Arch-Linux Optimized**: This build is specifically tuned for Arch-based systems (Garuda Linux) running on Intel i9-13900HX + RTX 4080 with native CUDA acceleration and 64GB DDR5 optimization.

## Common Commands

### Dependencies (Arch/Garuda Linux)
```bash
make install-deps      # Show manual installation commands
make install-deps-auto # Auto-install via pacman (requires sudo)
```

### Building (x86_64 default, Alderlake-optimized)
```bash
make all           # Build the complete kernel (x86_64, O2, Alderlake-tuned)
make kernel        # Build only the kernel binary
make iso           # Create bootable ISO image
make clean         # Clean build artifacts
```

### Performance Optimization Levels
```bash
make OPTIMIZE=0 all    # No optimization (-O0)
make OPTIMIZE=1 all    # Basic optimization (-O1)
make OPTIMIZE=2 all    # Standard optimization (-O2, Alderlake-tuned) [DEFAULT]
make OPTIMIZE=3 all    # Aggressive optimization (-O3, LTO, Alderlake-tuned)
make OPTIMIZE=s all    # Size optimization (-Os, Alderlake-tuned)
```

### Running and Testing
```bash
make run           # Run in QEMU (KVM-accelerated, 128MB RAM, host CPU)
make run-debug     # Run with GDB debugging support (-s -S flags)
```

### Debugging
```bash
make DEBUG=1 all   # Debug build (disables optimizations, adds -g)
```

### Architecture Override
```bash
make ARCH=i386 all     # Build for i386 (legacy mode)
make ARCH=x86_64 all   # Build for x86_64 [DEFAULT]
```

## Code Architecture

### Core Structure
- **Kernel**: Lives in `kernel/` - currently minimal with just `main.c` containing the kernel entry point
- **Bootloader**: `boot/boot.s` contains multiboot-compliant assembly bootloader with 32KB stack
- **Memory Layout**: Defined in `linker.ld` - architecture-aware load addresses
- **Configuration**: Build settings in `config.mk` (Arch-optimized) and main `Makefile`

### Build System (Arch-Linux Optimized)
- **Native Toolchain**: Uses system GCC/binutils, falls back to cross-compiler if available
- **Intel Optimization**: Alderlake-specific tuning (-march=alderlake -mtune=alderlake)
- **Smart Architecture Detection**: Automatically detects and configures for x86_64 vs i386
- **KVM Integration**: QEMU runs with KVM acceleration and host CPU passthrough
- **Pacman Integration**: Native Arch package management for dependencies

### Memory Management (Architecture-Aware)
**x86_64 Mode (Default)**:
- Kernel virtual base: `0xFFFFFFFF80000000` (canonical higher-half)
- Kernel load address: `0x200000` (2MB, optimized for large pages)
- Large page alignment (2MB boundaries)
- 32KB stack reserved in BSS section

**i386 Mode (Legacy)**:
- Kernel virtual base: `0xC0000000` (3GB)
- Kernel load address: `0x100000` (1MB)
- 4K page alignment
- 32KB stack reserved in BSS section

### Development Pattern
The codebase follows a traditional monolithic kernel architecture:
1. **Boot Phase**: Assembly bootstrap (`boot.s`) sets up stack and calls kernel
2. **Kernel Phase**: C kernel entry point (`kernel_main`) in `main.c`
3. **Modular Design**: Directory structure prepared for:
   - Memory management (`kernel/mm/`)
   - File systems (`kernel/fs/`)
   - Process management (`kernel/proc/`)
   - Device drivers (`drivers/`)
   - User space programs (`userland/`)

### Cross-Compilation Requirements (Arch/Garuda)
**Required Packages** (install via `make install-deps-auto`):
- `gcc` and `binutils` (native toolchain)
- `nasm` (assembler)
- `qemu-desktop` (emulation with KVM support)
- `grub` and `xorriso` (bootloader tools)
- `mtools` (disk utilities)
- `gdb` and `make` (development tools)

**Optional AUR Packages** (for pure cross-compilation):
- `x86_64-elf-gcc` and `x86_64-elf-binutils` (via AUR helper like `paru`)

## File Organization

### Critical Files
- `linker.ld`: Defines memory layout and section placement
- `config.mk`: Build configuration and toolchain settings
- `boot/boot.s`: Multiboot-compliant bootloader
- `kernel/main.c`: Kernel entry point

### Build Artifacts
All build outputs go to `build/` directory:
- `build/kernel.bin`: Final kernel binary
- `build/nexus-os.iso`: Bootable ISO image
- `build/*.o`: Object files

### Empty Structure
Many directories (`docs/`, `tests/`, `scripts/`, `include/kernel/`, `include/libc/`) exist but are currently empty, indicating this is an early-stage project with planned expansion.