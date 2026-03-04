# NexusOS Makefile

# Include build configuration
include config.mk

# Directories
KERNEL_DIR = kernel
BOOT_DIR = boot
BUILD_DIR = build
ISO_DIR = $(BUILD_DIR)/isofiles

# Source files - all C and S files in kernel directory and subdirectories
BOOT_SOURCES = $(BOOT_DIR)/boot.s
BOOT_ASM_SOURCES = $(wildcard $(BOOT_DIR)/*.s)
KERNEL_SOURCES = $(wildcard $(KERNEL_DIR)/*.c) $(wildcard $(KERNEL_DIR)/*/*.c)
KERNEL_ASM_SOURCES = $(wildcard $(KERNEL_DIR)/*.s) $(wildcard $(KERNEL_DIR)/*/*.s)

# Object files
BOOT_OBJECTS = $(BUILD_DIR)/boot.o
KERNEL_OBJECTS = $(KERNEL_SOURCES:$(KERNEL_DIR)/%.c=$(BUILD_DIR)/%.o)

# Target files
KERNEL_BIN = $(BUILD_DIR)/kernel.bin
ISO_FILE = $(BUILD_DIR)/nexus-os.iso

.PHONY: all clean run iso kernel

# Default target
all: kernel

# Create build directory and subdirectories
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR) $(BUILD_DIR)/arch $(BUILD_DIR)/irq $(BUILD_DIR)/mm $(BUILD_DIR)/proc $(BUILD_DIR)/syscall

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

# Link kernel (use 32-bit elf format for multiboot compatibility)
$(KERNEL_BIN): $(BOOT_OBJECTS) $(KERNEL_OBJECTS) $(KERNEL_ASM_OBJECTS) linker.ld | $(BUILD_DIR)
	$(LD) -m elf_i386 -T linker.ld -o $@ $(BOOT_OBJECTS) $(KERNEL_OBJECTS) $(KERNEL_ASM_OBJECTS)

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
