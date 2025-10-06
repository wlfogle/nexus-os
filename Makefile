# NexusOS Makefile

# Include build configuration
include config.mk

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
	$(LD) $(LDFLAGS) -T linker.ld -o $@ $(BOOT_OBJECTS) $(KERNEL_OBJECTS)

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
