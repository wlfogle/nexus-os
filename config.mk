# NexusOS Build Configuration - Optimized for Arch-based Systems

# Architecture settings - Default to x86_64 for modern systems
ARCH ?= x86_64
TARGET_ARCH = $(ARCH)

# Cross-compiler toolchain - Use system toolchain on Arch
CROSS_COMPILE = $(ARCH)-elf-

# Toolchain paths - System standard paths
TOOLCHAIN_PREFIX ?= /usr
TOOLCHAIN_BIN = $(TOOLCHAIN_PREFIX)/bin

# Check for cross-compiler first, then fallback to native GCC
ifeq ($(shell which $(ARCH)-elf-gcc 2>/dev/null),)
    # Use native system compiler
    CC = gcc
    AS = as
    LD = ld
    AR = ar
    OBJCOPY = objcopy
    OBJDUMP = objdump
else
    # Use cross-compiler if available
    CC = $(CROSS_COMPILE)gcc
    AS = $(CROSS_COMPILE)as
    LD = $(CROSS_COMPILE)ld
    AR = $(CROSS_COMPILE)ar
    OBJCOPY = $(CROSS_COMPILE)objcopy
    OBJDUMP = $(CROSS_COMPILE)objdump
endif

# Build flags optimized for Alderlake architecture
CFLAGS = -std=gnu99 -ffreestanding -Wall -Wextra -fno-stack-protector
CPPFLAGS = -I include/kernel -I include/libc

# Architecture-specific flags
# Note: We compile in 32-bit mode for Multiboot compatibility
# Full x86_64 long mode support can be added later
BOOT_ASFLAGS = --32
ASFLAGS = --32
ARCH_CFLAGS = -m32
LDFLAGS = -nostdlib -m elf_i386

CFLAGS += $(ARCH_CFLAGS)

# Performance optimization settings
OPTIMIZE ?= 2

# Optimization flags
ifeq ($(OPTIMIZE), 0)
    OPT_FLAGS = -O0
else ifeq ($(OPTIMIZE), 1)
    OPT_FLAGS = -O1
else ifeq ($(OPTIMIZE), 2)
    OPT_FLAGS = -O2 -march=alderlake -mtune=alderlake
else ifeq ($(OPTIMIZE), 3)
    OPT_FLAGS = -O3 -march=alderlake -mtune=alderlake -flto
else ifeq ($(OPTIMIZE), s)
    OPT_FLAGS = -Os -march=alderlake -mtune=alderlake
else
    OPT_FLAGS = -O2
endif

CFLAGS += $(OPT_FLAGS)

# Debug settings
DEBUG ?= 0
ifeq ($(DEBUG), 1)
    CFLAGS += -g -DDEBUG
    ASFLAGS += -g
    # Reduce optimization for debugging
    CFLAGS := $(filter-out -O% -flto -march=% -mtune=%,$(CFLAGS))
    CFLAGS += -O0
endif

# Target settings
# Use 2MB for x86_64 (better for large pages), 1MB for i386
ifeq ($(ARCH), x86_64)
    KERNEL_LOAD_ADDR = 0x200000
    KERNEL_VIRTUAL_BASE = 0xFFFFFFFF80000000
else
    KERNEL_LOAD_ADDR = 0x100000
    KERNEL_VIRTUAL_BASE = 0xC0000000
endif
