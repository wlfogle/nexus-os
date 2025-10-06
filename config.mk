# NexusOS Build Configuration - Optimized for Arch-based Systems

# Architecture settings - Default to x86_64 for modern systems
ARCH ?= x86_64
TARGET_ARCH = $(ARCH)

# Cross-compiler toolchain - Use system toolchain on Arch
CROSS_COMPILE = $(ARCH)-elf-

# Toolchain paths - Arch Linux standard paths
TOOLCHAIN_PREFIX ?= /usr
TOOLCHAIN_BIN = $(TOOLCHAIN_PREFIX)/bin

# Use toolchains from main system
MAIN_SYSTEM_BIN = $(MAIN_SYSTEM_ROOT)/usr/bin

# Check for cross-compiler on main system first, then fallback to system GCC
ifeq ($(shell test -f $(MAIN_SYSTEM_BIN)/$(ARCH)-elf-gcc && echo yes),yes)
    CC = $(MAIN_SYSTEM_BIN)/$(ARCH)-elf-gcc
    AS = $(MAIN_SYSTEM_BIN)/$(ARCH)-elf-as
    LD = $(MAIN_SYSTEM_BIN)/$(ARCH)-elf-ld
    OBJCOPY = $(MAIN_SYSTEM_BIN)/$(ARCH)-elf-objcopy
    OBJDUMP = $(MAIN_SYSTEM_BIN)/$(ARCH)-elf-objdump
else ifeq ($(shell which $(ARCH)-elf-gcc 2>/dev/null),)
    # Use main system GCC with enhanced paths
    CC = $(MAIN_SYSTEM_BIN)/gcc
    AS = $(MAIN_SYSTEM_BIN)/as
    LD = $(MAIN_SYSTEM_BIN)/ld
    OBJCOPY = $(MAIN_SYSTEM_BIN)/objcopy
    OBJDUMP = $(MAIN_SYSTEM_BIN)/objdump
else
    CC = $(CROSS_COMPILE)gcc
    AS = $(CROSS_COMPILE)as
    LD = $(CROSS_COMPILE)ld
    OBJCOPY = $(CROSS_COMPILE)objcopy
    OBJDUMP = $(CROSS_COMPILE)objdump
endif

# Build flags optimized for Alderlake architecture
CFLAGS = -std=gnu99 -ffreestanding -Wall -Wextra -fno-stack-protector
CPPFLAGS = -I include/kernel -I include/libc

# Architecture-specific flags
ifeq ($(ARCH), x86_64)
    ASFLAGS = --64
    ARCH_CFLAGS = -m64 -mcmodel=large -mno-red-zone -mno-mmx -mno-sse -mno-sse2
    LDFLAGS = -nostdlib -m elf_x86_64
else
    ASFLAGS = --32
    ARCH_CFLAGS = -m32
    LDFLAGS = -nostdlib -m elf_i386
endif

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

# AI/ML Integration Settings
CUDA_SUPPORT ?= 1
AI_NATIVE ?= 1

# CUDA paths (from main system)
MAIN_SYSTEM_ROOT = /run/media/garuda/34c008f3-1990-471c-bd80-c72985c7dc5c/@
CUDA_PATH = $(MAIN_SYSTEM_ROOT)/opt/cuda
CUDA_INCLUDE = $(CUDA_PATH)/include
CUDA_LIB = $(CUDA_PATH)/lib64

# Intel OneAPI paths (from main system)
INTEL_PATH = $(MAIN_SYSTEM_ROOT)/opt/intel
INTEL_ONEAPI = $(INTEL_PATH)/oneapi

# AI/ML specific flags
ifeq ($(AI_NATIVE), 1)
    CFLAGS += -DAI_NATIVE_KERNEL
    CFLAGS += -DCUDA_ENABLED
    CPPFLAGS += -I $(CUDA_INCLUDE)
endif

# RTX 4080 specific optimizations
ifeq ($(CUDA_SUPPORT), 1)
    CFLAGS += -DRTX_4080_OPTIMIZED
    CFLAGS += -DTENSOR_CORES_ENABLED
    CFLAGS += -DRT_CORES_ENABLED
endif

# Target settings
# Use 2MB for x86_64 (better for large pages), 1MB for i386
ifeq ($(ARCH), x86_64)
    KERNEL_LOAD_ADDR = 0x200000
    KERNEL_VIRTUAL_BASE = 0xFFFFFFFF80000000
    # AI-optimized memory layout
    AI_MODEL_CACHE_SIZE = 0xF00000000  # 60GB for AI models
    GPU_MEMORY_SIZE = 0x400000000      # 16GB RTX 4080
else
    KERNEL_LOAD_ADDR = 0x100000
    KERNEL_VIRTUAL_BASE = 0xC0000000
    # Limited AI support on i386
    AI_MODEL_CACHE_SIZE = 0x40000000   # 1GB
    GPU_MEMORY_SIZE = 0x100000000      # 4GB max
endif
