# NexusOS Makefile

# Target architecture (can be i386, x86_64, etc.)
ARCH ?= i386

# Cross-compiler settings
TARGET = $(ARCH)-elf
CC = $(TARGET)-gcc
AS = $(TARGET)-as
LD = $(TARGET)-ld

# Compiler and assembler flags
CFLAGS = -std=gnu99 -ffreestanding -O2 -Wall -Wextra -nostdlib -lgcc
ASFLAGS = --32
LDFLAGS = -T linker.ld -ffreestanding -O2 -nostdlib -lgcc

# Directories
KERNEL_DIR = kernel
BOOT_DIR = boot
BUILD_DIR = build
ISO_DIR = $(BUILD_DIR)/isofiles

# Source files
BOOT_SOURCES = $(BOOT_DIR)/boot.s
KERNEL_SOURCES = $(wildcard $(KERNEL_DIR)/*.c)

# Object files
BOOT_OBJECTS = $(BUILD_DIR)/boot.o
KERNEL_OBJECTS = $(KERNEL_SOURCES:$(KERNEL_DIR)/%.c=$(BUILD_DIR)/%.o)

# Target files
KERNEL_BIN = $(BUILD_DIR)/kernel.bin
ISO_FILE = $(BUILD_DIR)/nexus-os.iso

.PHONY: all clean run iso kernel

# Default target
all: kernel

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Compile assembly files
$(BUILD_DIR)/boot.o: $(BOOT_DIR)/boot.s | $(BUILD_DIR)
	$(AS) $(ASFLAGS) $< -o $@

# Compile C files
$(BUILD_DIR)/%.o: $(KERNEL_DIR)/%.c | $(BUILD_DIR)
	$(CC) -c $< -o $@ $(CFLAGS)

# Link kernel
$(KERNEL_BIN): $(BOOT_OBJECTS) $(KERNEL_OBJECTS) linker.ld | $(BUILD_DIR)
	$(CC) $(LDFLAGS) -o $@ $(BOOT_OBJECTS) $(KERNEL_OBJECTS)

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

# Run in QEMU
run: $(ISO_FILE)
	qemu-system-i386 -cdrom $(ISO_FILE) -m 32M

# Clean build files
clean:
	rm -rf $(BUILD_DIR)

# Install dependencies (example for Ubuntu/Debian)
install-deps:
	@echo "Install cross-compiler toolchain for your system:"
	@echo "Ubuntu/Debian: sudo apt install gcc-multilib nasm qemu-system-x86 grub-pc-bin xorriso"
	@echo "Arch Linux: sudo pacman -S multilib-devel nasm qemu grub xorriso"
	@echo "macOS: brew install i686-elf-gcc nasm qemu grub xorriso"