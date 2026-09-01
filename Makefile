# NexusOS Kernel Build System
# ──────────────────────────────────────────────────────────────────────────────
# Requirements (host-agnostic, no distro-specific packages):
#   rustup    — https://rustup.rs  (installs rust toolchain from source)
#   make      — universal
#   xorriso   — ISO creation (build from source or package manager)
#   qemu-system-x86_64 / qemu-system-aarch64  — testing only
#
# First run:
#   make setup        — install Rust targets + fetch/build Limine bootloader
#
# Build targets:
#   make laptop       — x86_64 full (Intel i9-13900HX, framebuffer, AI hooks)
#   make tiamat       — x86_64 server (headless, service hooks)
#   make bahamut      — AArch64 edge (2 GB, serial-only)
#
# Run in QEMU:
#   make run-laptop   make run-tiamat   make run-bahamut
#
# Create bootable ISOs:
#   make iso-laptop   make iso-tiamat   make iso-bahamut
# ──────────────────────────────────────────────────────────────────────────────

CARGO        := cargo
XORRISO      := xorriso
QEMU_X86     := qemu-system-x86_64
QEMU_ARM     := qemu-system-aarch64

TARGET_X86   := x86_64-unknown-none
TARGET_ARM   := aarch64-unknown-none-softfloat

KERNEL_DIR   := kernel
BUILD_DIR    := build
LIMINE_DIR   := limine
LIMINE_BIN   := $(LIMINE_DIR)/bin

# Locally-built QEMU >= 8.2 (Bahamut Increment B0): the distro package predates
# raspi4b (added in 8.2). Built from source, scoped to aarch64-softmmu only,
# into toolchain/ (gitignored -- not part of the repo, reproduce with
# `make setup-raspi4b-qemu`).
QEMU_RASPI4B := toolchain/qemu-9.2.0/build/qemu-system-aarch64

# ─── Default target ────────────────────────────────────────────────────────────
.PHONY: all
all: laptop tiamat bahamut

# ─── One-time setup ────────────────────────────────────────────────────────────
.PHONY: setup
setup:
	@echo "==> Installing Rust nightly + bare-metal targets..."
	rustup toolchain install nightly
	rustup override set nightly
	rustup target add $(TARGET_X86) $(TARGET_ARM)
	rustup component add rust-src llvm-tools-preview
	@echo "==> Fetching Limine bootloader submodule..."
	git submodule update --init --recursive $(LIMINE_DIR)
	@echo "==> Building Limine from source (requires cc, clang, lld, llvm-*-14)..."
	cd $(LIMINE_DIR) && ./bootstrap && \
	    OBJCOPY_FOR_TARGET=llvm-objcopy-14 OBJDUMP_FOR_TARGET=llvm-objdump-14 \
	    AR_FOR_TARGET=llvm-ar-14 RANLIB_FOR_TARGET=llvm-ranlib-14 \
	    STRIP_FOR_TARGET=llvm-strip-14 NM_FOR_TARGET=llvm-nm-14 \
	    READELF_FOR_TARGET=llvm-readelf-14 \
	    ./configure --enable-bios --enable-bios-cd --enable-uefi-x86-64 --enable-uefi-aarch64 --enable-uefi-cd && \
	    make -j$$(nproc)
	@echo ""
	@echo "Setup complete. Run 'make laptop', 'make tiamat', or 'make bahamut'."

# ─── Kernel builds ─────────────────────────────────────────────────────────────
.PHONY: laptop
laptop:
	@echo "==> Building NexusOS kernel [laptop / x86_64 full]"
	@mkdir -p $(BUILD_DIR)
	cd $(KERNEL_DIR) && \
	    $(CARGO) +nightly build --release \
	        --features laptop \
	        --target $(TARGET_X86) \
	        -Z build-std=core,alloc,compiler_builtins \
	        -Z build-std-features=compiler-builtins-mem \
	        --target-dir ../$(BUILD_DIR)/.cargo-laptop
	@cp $(BUILD_DIR)/.cargo-laptop/$(TARGET_X86)/release/nexus-kernel \
	    $(BUILD_DIR)/nexus-kernel-laptop
	@echo "==> $(BUILD_DIR)/nexus-kernel-laptop ready"

.PHONY: tiamat
tiamat:
	@echo "==> Building NexusOS kernel [tiamat / x86_64 server]"
	@mkdir -p $(BUILD_DIR)
	cd $(KERNEL_DIR) && \
	    $(CARGO) +nightly build --release \
	        --features tiamat \
	        --target $(TARGET_X86) \
	        -Z build-std=core,alloc,compiler_builtins \
	        -Z build-std-features=compiler-builtins-mem \
	        --target-dir ../$(BUILD_DIR)/.cargo-tiamat
	@cp $(BUILD_DIR)/.cargo-tiamat/$(TARGET_X86)/release/nexus-kernel \
	    $(BUILD_DIR)/nexus-kernel-tiamat
	@echo "==> $(BUILD_DIR)/nexus-kernel-tiamat ready"

.PHONY: bahamut
bahamut:
	@echo "==> Building NexusOS kernel [bahamut / AArch64]"
	@mkdir -p $(BUILD_DIR)
	cd $(KERNEL_DIR) && \
	    $(CARGO) +nightly build --release \
	        --features bahamut \
	        --target $(TARGET_ARM) \
	        -Z build-std=core,alloc,compiler_builtins \
	        -Z build-std-features=compiler-builtins-mem \
	        --target-dir ../$(BUILD_DIR)/.cargo-bahamut
	@cp $(BUILD_DIR)/.cargo-bahamut/$(TARGET_ARM)/release/nexus-kernel \
	    $(BUILD_DIR)/nexus-kernel-bahamut
	@echo "==> $(BUILD_DIR)/nexus-kernel-bahamut ready"

# ─── ISO creation ──────────────────────────────────────────────────────────────
.PHONY: iso-laptop
iso-laptop: laptop
	@echo "==> Creating bootable ISO [laptop]"
	@rm -rf $(BUILD_DIR)/iso-laptop
	@mkdir -p $(BUILD_DIR)/iso-laptop/boot/limine
	@mkdir -p $(BUILD_DIR)/iso-laptop/EFI/BOOT
	cp $(BUILD_DIR)/nexus-kernel-laptop    $(BUILD_DIR)/iso-laptop/boot/nexus-kernel
	cp iso_root/limine-laptop.conf         $(BUILD_DIR)/iso-laptop/boot/limine/limine.conf
	cp $(LIMINE_BIN)/limine-bios.sys       $(BUILD_DIR)/iso-laptop/boot/limine/
	cp $(LIMINE_BIN)/limine-bios-cd.bin    $(BUILD_DIR)/iso-laptop/boot/limine/
	cp $(LIMINE_BIN)/limine-uefi-cd.bin    $(BUILD_DIR)/iso-laptop/boot/limine/
	cp $(LIMINE_BIN)/BOOTX64.EFI           $(BUILD_DIR)/iso-laptop/EFI/BOOT/
	cp $(BUILD_DIR)/nexus-kernel-laptop    $(BUILD_DIR)/iso-laptop/EFI/BOOT/nexus-kernel
	cp iso_root/limine-laptop-uefi.conf    $(BUILD_DIR)/iso-laptop/EFI/BOOT/limine.conf
	$(XORRISO) -as mkisofs \
	    -b boot/limine/limine-bios-cd.bin \
	    -no-emul-boot -boot-load-size 4 -boot-info-table \
	    --efi-boot boot/limine/limine-uefi-cd.bin \
	    -efi-boot-part --efi-boot-image --protective-msdos-label \
	    $(BUILD_DIR)/iso-laptop \
	    -o $(BUILD_DIR)/nexusos-laptop.iso
	$(LIMINE_BIN)/limine bios-install $(BUILD_DIR)/nexusos-laptop.iso
	@echo "==> $(BUILD_DIR)/nexusos-laptop.iso ready"

.PHONY: iso-tiamat
iso-tiamat: tiamat
	@echo "==> Creating bootable ISO [tiamat]"
	@rm -rf $(BUILD_DIR)/iso-tiamat
	@mkdir -p $(BUILD_DIR)/iso-tiamat/boot/limine
	@mkdir -p $(BUILD_DIR)/iso-tiamat/EFI/BOOT
	cp $(BUILD_DIR)/nexus-kernel-tiamat    $(BUILD_DIR)/iso-tiamat/boot/nexus-kernel
	cp iso_root/limine-tiamat.conf         $(BUILD_DIR)/iso-tiamat/boot/limine/limine.conf
	cp $(LIMINE_BIN)/limine-bios.sys       $(BUILD_DIR)/iso-tiamat/boot/limine/
	cp $(LIMINE_BIN)/limine-bios-cd.bin    $(BUILD_DIR)/iso-tiamat/boot/limine/
	cp $(LIMINE_BIN)/limine-uefi-cd.bin    $(BUILD_DIR)/iso-tiamat/boot/limine/
	cp $(LIMINE_BIN)/BOOTX64.EFI           $(BUILD_DIR)/iso-tiamat/EFI/BOOT/
	cp $(BUILD_DIR)/nexus-kernel-tiamat    $(BUILD_DIR)/iso-tiamat/EFI/BOOT/nexus-kernel
	cp iso_root/limine-tiamat-uefi.conf    $(BUILD_DIR)/iso-tiamat/EFI/BOOT/limine.conf
	$(XORRISO) -as mkisofs \
	    -b boot/limine/limine-bios-cd.bin \
	    -no-emul-boot -boot-load-size 4 -boot-info-table \
	    --efi-boot boot/limine/limine-uefi-cd.bin \
	    -efi-boot-part --efi-boot-image --protective-msdos-label \
	    $(BUILD_DIR)/iso-tiamat \
	    -o $(BUILD_DIR)/nexusos-tiamat.iso
	$(LIMINE_BIN)/limine bios-install $(BUILD_DIR)/nexusos-tiamat.iso
	@echo "==> $(BUILD_DIR)/nexusos-tiamat.iso ready"

.PHONY: iso-bahamut
iso-bahamut: bahamut
	@echo "==> Creating bootable AArch64 UEFI ISO [bahamut]"
	@rm -rf $(BUILD_DIR)/iso-bahamut
	@mkdir -p $(BUILD_DIR)/iso-bahamut/boot/limine
	@mkdir -p $(BUILD_DIR)/iso-bahamut/EFI/BOOT
	cp $(BUILD_DIR)/nexus-kernel-bahamut   $(BUILD_DIR)/iso-bahamut/boot/nexus-kernel
	cp $(BUILD_DIR)/nexus-kernel-bahamut   $(BUILD_DIR)/iso-bahamut/EFI/BOOT/nexus-kernel
	cp iso_root/limine-bahamut.conf        $(BUILD_DIR)/iso-bahamut/boot/limine/limine.conf
	cp $(LIMINE_BIN)/limine-uefi-cd.bin    $(BUILD_DIR)/iso-bahamut/boot/limine/
	cp $(LIMINE_BIN)/BOOTAA64.EFI          $(BUILD_DIR)/iso-bahamut/EFI/BOOT/BOOTAA64.EFI
	cp iso_root/limine-bahamut-uefi.conf   $(BUILD_DIR)/iso-bahamut/EFI/BOOT/limine.conf
	printf '\\EFI\\BOOT\\BOOTAA64.EFI\r\n' > $(BUILD_DIR)/iso-bahamut/startup.nsh
	$(XORRISO) -as mkisofs \
	    --efi-boot boot/limine/limine-uefi-cd.bin \
	    -efi-boot-part --efi-boot-image --protective-msdos-label \
	    $(BUILD_DIR)/iso-bahamut \
	    -o $(BUILD_DIR)/nexusos-bahamut.iso
	@echo "==> $(BUILD_DIR)/nexusos-bahamut.iso ready"

# ─── QEMU tests ────────────────────────────────────────────────────────────────
.PHONY: run-laptop
run-laptop: iso-laptop
	$(QEMU_X86) \
	    -cdrom $(BUILD_DIR)/nexusos-laptop.iso \
	    -m 4G -cpu host -enable-kvm \
	    -netdev user,id=n0 -device virtio-net-pci,netdev=n0,disable-modern=on \
	    -serial stdio -display none \
	    -no-reboot -no-shutdown

.PHONY: run-tiamat
run-tiamat: iso-tiamat
	$(QEMU_X86) \
	    -cdrom $(BUILD_DIR)/nexusos-tiamat.iso \
	    -m 8G -cpu host -enable-kvm \
	    -netdev user,id=n0 -device virtio-net-pci,netdev=n0,disable-modern=on \
	    -serial stdio -display none \
	    -no-reboot -no-shutdown

.PHONY: run-bahamut
run-bahamut: iso-bahamut
	$(QEMU_ARM) -M virt -cpu cortex-a72 -m 2G \
	    -bios /usr/share/qemu-efi-aarch64/QEMU_EFI.fd \
	    -drive file=$(BUILD_DIR)/nexusos-bahamut.iso,if=none,media=cdrom,id=cd0 \
	    -device virtio-scsi-device,id=scsi0 -device scsi-cd,bus=scsi0.0,drive=cd0 \
	    -serial stdio -display none \
	    -no-reboot -no-shutdown

# ─── Install disk image + QEMU install/boot targets ────────────────────────────
#
# Workflow:
#   make disk-laptop          — create 8 GB QCOW2 target disk
#   make run-install-laptop   — boot ISO + disk (installer writes kernel to disk)
#   make run-installed-laptop — boot from installed disk only (OVMF, no ISO)
#
# The VirtIO-blk device uses legacy I/O-port mode (disable-modern=on) so the
# kernel's legacy BAR0 driver can initialise it.

DISK_LAPTOP  := $(BUILD_DIR)/nexusos-laptop.qcow2
DISK_BAHAMUT := $(BUILD_DIR)/nexusos-bahamut.qcow2
OVMF         := /usr/share/OVMF/OVMF_CODE.fd
AAVMF        := /usr/share/AAVMF/AAVMF_CODE.fd

.PHONY: disk-laptop
disk-laptop:
	@echo "==> Creating 8 GiB install target [laptop]"
	@mkdir -p $(BUILD_DIR)
	qemu-img create -f qcow2 $(DISK_LAPTOP) 8G
	@echo "==> $(DISK_LAPTOP) ready"

.PHONY: run-install-laptop
run-install-laptop: iso-laptop
	@test -f $(DISK_LAPTOP) || $(MAKE) disk-laptop
	@echo "==> Installer boot: ISO + VirtIO disk [laptop]"
	@echo "    Watch for 'Installation complete!' then Ctrl-C."
	$(QEMU_X86) \
	    -boot d \
	    -cdrom $(BUILD_DIR)/nexusos-laptop.iso \
	    -drive file=$(DISK_LAPTOP),if=virtio \
	    -m 4G -cpu host -enable-kvm \
	    -netdev user,id=n0 -device virtio-net-pci,netdev=n0,disable-modern=on \
	    -serial stdio -display none \
	    -no-reboot -no-shutdown

.PHONY: run-installed-laptop
run-installed-laptop:
	@test -f $(DISK_LAPTOP) || { echo "ERROR: run 'make run-install-laptop' first"; exit 1; }
	@echo "==> Booting installed NexusOS from disk [laptop]"
	$(QEMU_X86) \
	    -drive file=$(DISK_LAPTOP),if=virtio \
	    -bios $(OVMF) \
	    -m 4G -cpu host -enable-kvm \
	    -netdev user,id=n0 -device virtio-net-pci,netdev=n0,disable-modern=on \
	    -serial stdio -display none \
	    -no-reboot -no-shutdown

# ─── Bahamut (AArch64) install workflow ────────────────────────────────────────
# VirtIO on AArch64 virt machine uses MMIO transport (-device virtio-blk-device)

.PHONY: disk-bahamut
disk-bahamut:
	@echo "==> Creating 8 GiB install target [bahamut]"
	@mkdir -p $(BUILD_DIR)
	qemu-img create -f qcow2 $(DISK_BAHAMUT) 8G
	@echo "==> $(DISK_BAHAMUT) ready"

.PHONY: run-install-bahamut
run-install-bahamut: iso-bahamut
	@test -f $(DISK_BAHAMUT) || $(MAKE) disk-bahamut
	@echo "==> Installer boot: ISO + VirtIO-MMIO disk [bahamut]"
	@echo "    Watch serial for NexusOS Installer banner and 'Installation complete!'"
	$(QEMU_ARM) -M virt -cpu cortex-a72 -m 2G \
	    -bios $(AAVMF) \
	    -drive file=$(BUILD_DIR)/nexusos-bahamut.iso,if=none,media=cdrom,id=cd0 \
	    -device virtio-scsi-device,id=scsi0 -device scsi-cd,bus=scsi0.0,drive=cd0 \
	    -drive file=$(DISK_BAHAMUT),if=none,id=hd0 \
	    -device virtio-blk-device,drive=hd0 \
	    -serial stdio -display none \
	    -no-reboot -no-shutdown

.PHONY: run-installed-bahamut
run-installed-bahamut:
	@test -f $(DISK_BAHAMUT) || { echo "ERROR: run 'make run-install-bahamut' first"; exit 1; }
	@echo "==> Booting installed NexusOS from disk [bahamut]"
	$(QEMU_ARM) -M virt -cpu cortex-a72 -m 2G \
	    -bios $(AAVMF) \
	    -drive file=$(DISK_BAHAMUT),if=none,id=hd0 \
	    -device virtio-blk-device,drive=hd0 \
	    -serial stdio -display none \
	    -no-reboot -no-shutdown

# ─── Bahamut Increment B0: local raspi4b QEMU test bed ─────────────────────────
# The distro qemu-system-aarch64 (6.2.0) predates raspi4b (added in 8.2, and
# Pop!_OS/Ubuntu jammy's repos offer nothing newer). This builds QEMU 9.2.0
# from source, scoped to --target-list=aarch64-softmmu only (headless, no
# GTK/SDL/VNC -- matches how every other target here is tested with
# `-display none`), into toolchain/ so it never touches the system package.
# Requires a newer meson than the system package (0.61.2); installed to
# ~/.local/bin via pip --user, which does not affect system Python packages.
.PHONY: setup-raspi4b-qemu
setup-raspi4b-qemu:
	@echo "==> Ensuring a modern meson/ninja are available (pip --user)..."
	pip3 install --user --upgrade meson ninja
	@echo "==> Fetching QEMU 9.2.0 source..."
	@mkdir -p toolchain
	@test -f toolchain/qemu-9.2.0.tar.xz || \
	    wget -q -O toolchain/qemu-9.2.0.tar.xz https://download.qemu.org/qemu-9.2.0.tar.xz
	@test -d toolchain/qemu-9.2.0 || \
	    tar xf toolchain/qemu-9.2.0.tar.xz -C toolchain
	@echo "==> Configuring + building QEMU (aarch64-softmmu only, headless)..."
	@mkdir -p toolchain/qemu-9.2.0/build
	cd toolchain/qemu-9.2.0/build && PATH="$$HOME/.local/bin:$$PATH" \
	    ../configure --target-list=aarch64-softmmu \
	        --disable-gtk --disable-sdl --disable-vnc --disable-docs \
	        --enable-slirp && \
	    PATH="$$HOME/.local/bin:$$PATH" ninja -j$$(nproc)
	@echo ""
	@echo "==> $(QEMU_RASPI4B) ready ($$($(QEMU_RASPI4B) --version | head -1))"

# Informational smoke test only until Increments B1 (platform detection) and
# B2 (real BCM2711 UART0) land -- the kernel still only knows QEMU virt's PL011
# address (0x09000000), not raspi4b's real one (0xFE201000), so no serial
# output is expected yet. This just confirms the raspi4b machine itself boots.
.PHONY: run-raspi4b
run-raspi4b: bahamut
	@test -x $(QEMU_RASPI4B) || { echo "ERROR: run 'make setup-raspi4b-qemu' first"; exit 1; }
	$(QEMU_RASPI4B) -M raspi4b -cpu cortex-a72 -m 2G \
	    -kernel $(BUILD_DIR)/nexus-kernel-bahamut \
	    -serial stdio -display none \
	    -no-reboot -no-shutdown

# ─── Utilities ─────────────────────────────────────────────────────────────────
.PHONY: clean clean-all
clean:
	rm -rf $(BUILD_DIR)

clean-all: clean
	cd $(KERNEL_DIR) && $(CARGO) clean
