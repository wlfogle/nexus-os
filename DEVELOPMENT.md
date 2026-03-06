# NexusOS Development Guide

## Quick Start

### Prerequisites (Pop!_OS 22.04)
```bash
sudo apt update
sudo apt install -y build-essential gcc binutils nasm qemu-system-x86 gdb make git
```

### Build
```bash
cd nexus-os
make clean
make all       # Build kernel binary
make iso       # Create ISO (requires grub-mkrescue)
```

### Run
```bash
make run                # Boot in QEMU
make run-debug          # Boot with GDB stub (-s -S)
bash tests/boot_test.sh # Automated boot verification
```

## Project Structure

```
nexus-os/
├── boot/               # Bootloader (32-bit multiboot)
├── kernel/             # Kernel source
│   ├── arch/          # Architecture-specific code
│   ├── irq/           # Interrupt/exception handling
│   ├── mm/            # Memory management
│   ├── proc/          # Process/task management
│   ├── fs/            # Filesystem
│   ├── drivers/       # Kernel-level drivers
│   ├── main.c         # Kernel entry point
│   └── serial.c       # Serial port driver
├── drivers/           # Device drivers
│   ├── display/
│   ├── storage/
│   ├── network/
│   └── input/
├── userland/          # User space
│   ├── bin/           # User programs
│   ├── lib/           # User libraries
│   └── init/          # Init process
├── lib/               # Kernel helper libraries
├── include/           # Header files
│   ├── kernel/
│   └── libc/
├── tests/             # Test suite
├── Makefile           # Main build script
├── config.mk          # Build configuration
├── linker.ld          # Linker script
└── README.md          # Project overview
```

## Build Configuration

### Optimization Levels
```bash
make OPTIMIZE=0 all    # No optimization (-O0)
make OPTIMIZE=2 all    # Default (-O2, Alderlake tuned)
make OPTIMIZE=3 all    # Aggressive (-O3, LTO)
```

### Debug Build
```bash
make DEBUG=1 all       # Disable optimizations, add debug symbols
```

### Architecture (future use)
```bash
make ARCH=i386 all     # 32-bit mode
make ARCH=x86_64 all   # 64-bit mode (not yet implemented)
```

## Coding Standards

### Include Guards
```c
#ifndef COMPONENT_H
#define COMPONENT_H

/* declarations */

#endif /* COMPONENT_H */
```

### Naming Conventions
- Functions: `snake_case`
- Macros: `UPPER_CASE`
- Types: `snake_case_t` (for typedefs)
- Structs: `struct snake_case`

### Error Handling
Return meaningful error codes or use assertion macros:
```c
#define ASSERT(cond) if (!(cond)) { \
    serial_printf("ASSERTION FAILED: %s:%d\n", __FILE__, __LINE__); \
    while(1); }
```

### Logging
Use serial_printf for all kernel logging:
```c
serial_printf("Message: %d\n", value);
```

## Development Workflow

### 1. Make a change
```bash
vim kernel/component.c
```

### 2. Build
```bash
make clean && make all
```

### 3. Test
```bash
bash tests/boot_test.sh
```

### 4. Debug (if needed)
```bash
# Terminal 1: Start QEMU with GDB stub
make run-debug

# Terminal 2: Connect GDB
gdb build/kernel.bin
(gdb) target remote :1234
(gdb) break kernel_main
(gdb) c
```

### 5. Commit
```bash
git add -A
git commit -m "Brief description

Detailed explanation of changes.

Co-Authored-By: Oz <oz-agent@warp.dev>"
```

## Current Phase: Phase 0 ✓

**Status**: Complete
- Build system fixed and working
- Serial logging infrastructure in place
- Subsystem skeletons created
- Boot test infrastructure ready

## Next Phase: Phase 1 (In Progress)

**Target**: Core kernel runtime
- GDT/IDT setup
- Exception handlers
- IRQ and PIT timer
- Physical memory manager
- Paging and virtual memory
- VGA/serial console (serial done, VGA next)

## Debugging Tips

### Serial Output
The kernel logs to COM1 (0x3F8). QEMU redirects this:
```bash
qemu-system-i386 -kernel kernel.bin -serial file:/tmp/serial.log
cat /tmp/serial.log
```

### Symbols and Backtraces
Build with debug symbols:
```bash
make DEBUG=1 all
objdump -d build/kernel.bin | less
```

### Common Issues

**"Kernel doesn't boot"**
- Check serial output in boot test
- Verify multiboot header in boot/boot.s
- Ensure linker.ld memory layout is correct

**"Build fails on missing include"**
- Check CPPFLAGS in config.mk
- Ensure header paths use relative paths from kernel source
- Use `#include "../../include/kernel/..."` pattern

**"QEMU hangs"**
- Add more serial logging to narrow down where it hangs
- Use GDB to set breakpoints and inspect state
- Check for infinite loops or deadlocks

## Resources

- Multiboot Specification: https://www.gnu.org/software/grub/manual/multiboot/multiboot.html
- x86 Architecture: https://wiki.osdev.org/Expanded_Main_Page
- QEMU Documentation: https://qemu.weilnetz.de/doc/qemu-doc.html
- GDB Reference: https://sourceware.org/gdb/documentation/

## Roadmap Phases

- **Phase 0**: ✓ Foundation (DONE)
- **Phase 1**: Core kernel runtime (4-6 weeks)
- **Phase 2**: Process model and syscalls (4-6 weeks)
- **Phase 3**: Storage and filesystem (4-6 weeks)
- **Phase 4**: Userland and toolchain (4-6 weeks)
- **Phase 5**: Device drivers and platform (6-10 weeks)
- **Phase 6**: Networking stack (6-10 weeks)
- **Phase 7**: Reliability and security (ongoing)
- **Phase 8**: AI capabilities (8-16+ weeks)

See ROADMAP.md for detailed phase breakdown.
