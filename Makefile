# NexusOS Makefile

# Include build configuration
include config.mk

# Directories
KERNEL_DIR = kernel
BOOT_DIR = boot
BUILD_DIR = build
ISO_DIR = $(BUILD_DIR)/isofiles
USERLAND_DIR = userland
USERLAND_BUILD = $(BUILD_DIR)/userland

# Source files - all C and S files in kernel directory and subdirectories
BOOT_SOURCES = $(BOOT_DIR)/boot.s
BOOT_ASM_SOURCES = $(wildcard $(BOOT_DIR)/*.s)
KERNEL_SOURCES = $(wildcard $(KERNEL_DIR)/*.c) $(wildcard $(KERNEL_DIR)/*/*.c)
KERNEL_ASM_SOURCES = $(wildcard $(KERNEL_DIR)/*.s) $(wildcard $(KERNEL_DIR)/*/*.s)
LIB_SOURCES = $(wildcard lib/*.c)

# Userland sources
USERLAND_LIB_SOURCES = $(wildcard $(USERLAND_DIR)/lib/*.c)
USERLAND_INIT_SOURCES = $(USERLAND_DIR)/init/init.c
USERLAND_BIN_SOURCES = $(wildcard $(USERLAND_DIR)/bin/*.c)
USERLAND_CRT0 = $(USERLAND_DIR)/crt0.s

# Object files
BOOT_OBJECTS = $(BUILD_DIR)/boot.o
KERNEL_OBJECTS = $(KERNEL_SOURCES:$(KERNEL_DIR)/%.c=$(BUILD_DIR)/%.o)
LIB_OBJECTS = $(LIB_SOURCES:lib/%.c=$(BUILD_DIR)/lib/%.o)

# Userland object files
USERLAND_LIB_OBJECTS = $(USERLAND_LIB_SOURCES:$(USERLAND_DIR)/%.c=$(USERLAND_BUILD)/%.o)
USERLAND_CRT0_OBJ = $(USERLAND_BUILD)/crt0.o
USERLAND_INIT_OBJ = $(USERLAND_BUILD)/init.o
USERLAND_BIN_OBJS = $(USERLAND_BIN_SOURCES:$(USERLAND_DIR)/%.c=$(USERLAND_BUILD)/%.o)

# Target files
KERNEL_BIN = $(BUILD_DIR)/kernel.bin
ISO_FILE = $(BUILD_DIR)/nexus-os.iso

.PHONY: all clean run iso kernel userland

# Default target
all: kernel userland

# Create build directory and subdirectories
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR) $(BUILD_DIR)/arch $(BUILD_DIR)/irq $(BUILD_DIR)/mm $(BUILD_DIR)/proc $(BUILD_DIR)/syscall $(BUILD_DIR)/drivers $(BUILD_DIR)/fs $(BUILD_DIR)/exec $(BUILD_DIR)/lib $(BUILD_DIR)/net $(BUILD_DIR)/sync $(BUILD_DIR)/device $(BUILD_DIR)/gpu $(BUILD_DIR)/security

# Compile assembly files (bootloader uses 32-bit flags)
$(BUILD_DIR)/boot.o: $(BOOT_DIR)/boot.s | $(BUILD_DIR)
	$(AS) $(BOOT_ASFLAGS) $< -o $@

# Compile C files
$(BUILD_DIR)/%.o: $(KERNEL_DIR)/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Compile assembly files in subdirectories
$(BUILD_DIR)/arch/%.o: $(KERNEL_DIR)/arch/%.s | $(BUILD_DIR)
	$(AS) $(ASFLAGS) $< -o $@

$(BUILD_DIR)/irq/%.o: $(KERNEL_DIR)/irq/%.s | $(BUILD_DIR)
	$(AS) $(ASFLAGS) $< -o $@

$(BUILD_DIR)/mm/%.o: $(KERNEL_DIR)/mm/%.s | $(BUILD_DIR)
	$(AS) $(ASFLAGS) $< -o $@

$(BUILD_DIR)/proc/%.o: $(KERNEL_DIR)/proc/%.s | $(BUILD_DIR)
	$(AS) $(ASFLAGS) $< -o $@

# Collect all kernel assembly objects
KERNEL_ASM_OBJECTS = $(KERNEL_ASM_SOURCES:$(KERNEL_DIR)/%.s=$(BUILD_DIR)/%.o)

# Add syscall directory to build
$(BUILD_DIR)/syscall/%.o: $(KERNEL_DIR)/syscall/%.s | $(BUILD_DIR)
	$(AS) $(ASFLAGS) $< -o $@

# Add drivers directory
$(BUILD_DIR)/drivers/%.o: $(KERNEL_DIR)/drivers/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Add filesystem directory
$(BUILD_DIR)/fs/%.o: $(KERNEL_DIR)/fs/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Add exec directory
$(BUILD_DIR)/exec/%.o: $(KERNEL_DIR)/exec/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Add lib directory
$(BUILD_DIR)/lib/%.o: lib/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Add net directory
$(BUILD_DIR)/net/%.o: $(KERNEL_DIR)/net/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Add sync directory
$(BUILD_DIR)/sync/%.o: $(KERNEL_DIR)/sync/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Add device directory
$(BUILD_DIR)/device/%.o: $(KERNEL_DIR)/device/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Add GPU directory
$(BUILD_DIR)/gpu/%.o: $(KERNEL_DIR)/gpu/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Add security directory
$(BUILD_DIR)/security/%.o: $(KERNEL_DIR)/security/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CPPFLAGS) $(CFLAGS)

# Userland build rules
$(USERLAND_BUILD):
	mkdir -p $(USERLAND_BUILD) $(USERLAND_BUILD)/lib $(USERLAND_BUILD)/init $(USERLAND_BUILD)/bin

# Compile userland C files
$(USERLAND_BUILD)/lib/%.o: $(USERLAND_DIR)/lib/%.c | $(USERLAND_BUILD)
	$(CC) -c $< -o $@ -I$(USERLAND_DIR)/lib -I include/libc $(CPPFLAGS) $(CFLAGS) -ffreestanding

$(USERLAND_BUILD)/init/%.o: $(USERLAND_DIR)/init/%.c | $(USERLAND_BUILD)
	$(CC) -c $< -o $@ -I$(USERLAND_DIR)/lib -I include/libc $(CPPFLAGS) $(CFLAGS) -ffreestanding

$(USERLAND_BUILD)/bin/%.o: $(USERLAND_DIR)/bin/%.c | $(USERLAND_BUILD)
	$(CC) -c $< -o $@ -I$(USERLAND_DIR)/lib -I include/libc $(CPPFLAGS) $(CFLAGS) -ffreestanding

# Compile userland assembly
$(USERLAND_BUILD)/crt0.o: $(USERLAND_CRT0) | $(USERLAND_BUILD)
	$(AS) $(ASFLAGS) $< -o $@

# Build userland libc archive
$(USERLAND_BUILD)/libc.a: $(USERLAND_LIB_OBJECTS) | $(USERLAND_BUILD)
	$(AR) rcs $@ $^

# Link userland binaries
$(USERLAND_BUILD)/init.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/init/init.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/echo.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/echo.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/cat.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/cat.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/ps.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/ps.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/shell.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/shell.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/netstat.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/netstat.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/ifconfig.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/ifconfig.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/nettest.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/nettest.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/threadtest.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/threadtest.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/hwinfo.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/hwinfo.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/devmgr.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/devmgr.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

$(USERLAND_BUILD)/bin/memstat.elf: $(USERLAND_BUILD)/crt0.o $(USERLAND_BUILD)/bin/memstat.o $(USERLAND_BUILD)/libc.a
	$(LD) -m elf_i386 -T userland/userland.ld -o $@ $^

# Phony target to build all userland
.PHONY: userland
userland: $(USERLAND_BUILD)/libc.a $(USERLAND_BUILD)/init.elf $(USERLAND_BUILD)/bin/echo.elf $(USERLAND_BUILD)/bin/cat.elf $(USERLAND_BUILD)/bin/ps.elf $(USERLAND_BUILD)/bin/shell.elf $(USERLAND_BUILD)/bin/netstat.elf $(USERLAND_BUILD)/bin/ifconfig.elf $(USERLAND_BUILD)/bin/nettest.elf $(USERLAND_BUILD)/bin/threadtest.elf $(USERLAND_BUILD)/bin/hwinfo.elf $(USERLAND_BUILD)/bin/devmgr.elf $(USERLAND_BUILD)/bin/memstat.elf

# Link kernel (use 32-bit elf format for multiboot compatibility)
$(KERNEL_BIN): $(BOOT_OBJECTS) $(KERNEL_OBJECTS) $(KERNEL_ASM_OBJECTS) $(LIB_OBJECTS) linker.ld | $(BUILD_DIR)
	$(LD) -m elf_i386 -T linker.ld -o $@ $(BOOT_OBJECTS) $(KERNEL_OBJECTS) $(KERNEL_ASM_OBJECTS) $(LIB_OBJECTS)

# Build kernel
kernel: $(KERNEL_BIN)

# Create ISO
iso: $(ISO_FILE)

$(ISO_FILE): $(KERNEL_BIN) | $(BUILD_DIR)
	mkdir -p $(ISO_DIR)/boot/grub
	cp $(KERNEL_BIN) $(ISO_DIR)/boot/kernel.bin
	echo 'set timeout=0' > $(ISO_DIR)/boot/grub/grub.cfg
	echo 'set default=0' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo 'menuentry "NexusOS" {' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '    multiboot /boot/kernel.bin' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '}' >> $(ISO_DIR)/boot/grub/grub.cfg
	grub-mkrescue -o $(ISO_FILE) $(ISO_DIR) 2>/dev/null || echo "Warning: grub-mkrescue not available"

# Run in QEMU - Architecture-aware
run: $(ISO_FILE)
ifeq ($(ARCH), x86_64)
	qemu-system-x86_64 -cdrom $(ISO_FILE) -m 128M -enable-kvm -cpu host
else
	qemu-system-i386 -cdrom $(ISO_FILE) -m 32M
endif

# Run with debugging support
run-debug: $(ISO_FILE)
ifeq ($(ARCH), x86_64)
	qemu-system-x86_64 -cdrom $(ISO_FILE) -m 128M -enable-kvm -cpu host -s -S
else
	qemu-system-i386 -cdrom $(ISO_FILE) -m 32M -s -S
endif

# Clean build files
clean:
	rm -rf $(BUILD_DIR)

# Install dependencies for Arch-based systems (Garuda Linux)
install-deps:
	@echo "Installing dependencies for Arch-based systems..."
	@echo "Main packages:"
	@echo "sudo pacman -S gcc binutils nasm qemu-desktop grub xorriso mtools"
	@echo ""
	@echo "Optional cross-compiler (for pure cross-compilation):"
	@echo "paru -S x86_64-elf-gcc x86_64-elf-binutils  # or use your AUR helper"
	@echo ""
	@echo "Development tools:"
	@echo "sudo pacman -S gdb make git"
	@echo ""
	@echo "Note: This configuration is optimized for your Intel i9-13900HX (Alderlake)"

install-deps-auto:
	@echo "Attempting to install dependencies automatically..."
	sudo pacman -S --needed gcc binutils nasm qemu-desktop grub xorriso mtools gdb make git
	@echo "Installing AI/ML dependencies..."
	sudo pacman -S --needed cuda python-pytorch python-tensorflow python-numpy

# AI/ML specific targets
ai-kernel: CFLAGS += -DAI_NATIVE_KERNEL -DCUDA_ENABLED
ai-kernel: $(KERNEL_BIN)

cuda-test: $(KERNEL_BIN)
	@echo "Testing CUDA integration..."
	@if [ -d "/opt/cuda" ]; then echo "✅ CUDA found at /opt/cuda"; else echo "❌ CUDA not found - install with: sudo pacman -S cuda"; fi

# Container support test
container-test:
	@echo "Testing container support..."
	@which docker >/dev/null && echo "✅ Docker available" || echo "❌ Docker not found"
	@which podman >/dev/null && echo "✅ Podman available" || echo "❌ Podman not found"

# Full AI stack test
ai-stack-test: cuda-test container-test
	@echo "🤖 AI-Native kernel build test complete"
