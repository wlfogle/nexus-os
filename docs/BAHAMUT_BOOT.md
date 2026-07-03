# Bahamut AArch64 Boot Verification
This document records the current verification path for **Bahamut**, the NexusOS Lite AArch64 network-edge target for Raspberry Pi 4 / DietPi-class duties.
## Current status
- Build target: `bahamut`
- Architecture: AArch64
- Role: network-edge / lightweight node
- Current live host: Raspberry Pi 4 running DietPi 10 at `bahamut` / `192.168.12.244`
- NexusOS artifact: `build/nexusos-bahamut.iso`
- Bootloader artifact required: `limine/bin/BOOTAA64.EFI`
- Current validation status:
  - `make bahamut` builds.
  - `make iso-bahamut` builds.
  - QEMU AArch64 smoke boot on the laptop produced no serial output with the currently installed firmware, so hardware or Pi UEFI validation is still required.
## Build Bahamut
From the repo root:
```bash
make bahamut
make iso-bahamut
```
Expected artifacts:
```text
build/nexus-kernel-bahamut
build/nexusos-bahamut.iso
```
If `make iso-bahamut` fails because `BOOTAA64.EFI` is missing, rebuild Limine's AArch64 UEFI target:
```bash
cd limine
OBJCOPY_FOR_TARGET=llvm-objcopy-14 \
OBJDUMP_FOR_TARGET=llvm-objdump-14 \
AR_FOR_TARGET=llvm-ar-14 \
RANLIB_FOR_TARGET=llvm-ranlib-14 \
STRIP_FOR_TARGET=llvm-strip-14 \
NM_FOR_TARGET=llvm-nm-14 \
READELF_FOR_TARGET=llvm-readelf-14 \
./configure --enable-uefi-aarch64
make limine-uefi-aarch64 -j$(nproc)
cd ..
make iso-bahamut
```
Successful output should include:
```text
==> build/nexus-kernel-bahamut ready
==> build/nexusos-bahamut.iso ready
```
## What Bahamut should boot into today
Bahamut is intentionally not a full laptop/media install yet. The current image is a minimal AArch64 edge skeleton. On a successful boot, serial output should show the normal early boot lines plus Bahamut-specific markers:
```text
NexusOS Kernel v0.6.2  [ bahamut ]
[arch] CPU structures loaded
[mem]  Physical frame allocator online
[mem]  Paging initialised
[mem]  Kernel heap (...) ready
[bahamut] AArch64 network-edge skeleton: UART, memory, paging, heap online
[bahamut] Disk installer, personalities, VFS, and networking services land after AArch64 IRQ/MMIO drivers
NexusOS v0.6.2 — Bahamut AArch64 edge skeleton active.
```
Current Bahamut intentionally does not start the x86_64-only pieces:
- PS/2 keyboard shell
- x86_64 syscall entry path
- PCI I/O-port VirtIO-blk
- NVMe/AHCI x86 PCI scan
- FAT32 installer
- smoltcp/VirtIO-net demo
Those require AArch64 MMIO/IRQ/device-driver work before Bahamut can become a complete installer/runtime target.
## QEMU smoke test attempt
The current laptop host has AArch64 UEFI firmware at:
```text
/usr/share/qemu-efi-aarch64/QEMU_EFI.fd
/usr/share/AAVMF/AAVMF_CODE.fd
```
These commands were attempted:
```bash
timeout 35 qemu-system-aarch64 \
  -M virt -cpu cortex-a72 -m 512M \
  -bios /usr/share/qemu-efi-aarch64/QEMU_EFI.fd \
  -cdrom build/nexusos-bahamut.iso \
  -nographic -no-reboot -no-shutdown
```
and:
```bash
timeout 35 qemu-system-aarch64 \
  -M virt -cpu cortex-a72 -m 512M \
  -bios /usr/share/AAVMF/AAVMF_CODE.fd \
  -cdrom build/nexusos-bahamut.iso \
  -nographic -no-reboot -no-shutdown
```
Both produced zero serial output. This means the artifact builds, but the current QEMU firmware/path has not proven runtime boot. Treat QEMU AArch64 boot as unresolved until serial output is observed.
## Hardware validation path on Raspberry Pi 4
Bahamut currently runs DietPi 10 and provides the live network-edge services. Do not overwrite that SD card without a backup.
Recommended process:
1. Use a spare microSD card or USB boot device.
2. Preserve the current DietPi 10 card as rollback media.
3. Flash or copy the Bahamut ISO/image to the spare media.
4. Boot the Pi 4 with serial UART attached if possible.
5. Capture serial output from power-on through kernel handoff.
6. Confirm the expected Bahamut boot markers listed above.
7. If boot fails before NexusOS output, inspect Pi firmware/UEFI handoff and AArch64 Limine config.
8. If NexusOS starts but panics, record the full serial log and update this document with the failing line.
## Serial console notes
For Raspberry Pi 4 validation, prefer UART serial logging over HDMI-only debugging. Serial output is the only reliable early-boot signal for the current Bahamut profile.
Expected serial path depends on Pi firmware/UEFI setup. For a USB UART adapter on the laptop:
```bash
sudo dmesg | grep -i tty
picocom -b 115200 /dev/ttyUSB0
```
Use the actual device shown by `dmesg`.
## Pass/fail checklist
Pass for the current Bahamut milestone means:
- `make bahamut` succeeds.
- `make iso-bahamut` succeeds.
- `BOOTAA64.EFI` exists under `limine/bin/`.
- Hardware or QEMU boot reaches NexusOS AArch64 boot logs.
- Kernel reaches the Bahamut edge skeleton message without panic.
Fail means:
- No firmware handoff to Limine.
- No NexusOS serial output.
- Panic before memory/paging/heap complete.
- Unexpected attempt to run x86-only subsystems on AArch64.
## Next engineering tasks for Bahamut
To move Bahamut from skeleton to usable NexusOS edge installer:
1. AArch64 timer/IRQ controller support for Pi/QEMU.
2. AArch64 syscall entry path.
3. AArch64 UART input or USB HID input.
4. MMIO VirtIO or Pi storage/network drivers.
5. FAT32 installer path for AArch64.
6. Edge services profile: DNS, ingress, VPN, secrets, and lightweight monitoring.
