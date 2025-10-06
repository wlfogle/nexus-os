# NexusOS Build Configuration

# Architecture settings
ARCH ?= i386
TARGET_ARCH = $(ARCH)

# Cross-compiler toolchain
CROSS_COMPILE = $(ARCH)-elf-

# Toolchain paths (adjust as needed for your system)
TOOLCHAIN_PREFIX ?= /usr/local/cross
TOOLCHAIN_BIN = $(TOOLCHAIN_PREFIX)/bin

# Compiler settings
CC = $(CROSS_COMPILE)gcc
AS = $(CROSS_COMPILE)as
LD = $(CROSS_COMPILE)ld
OBJCOPY = $(CROSS_COMPILE)objcopy
OBJDUMP = $(CROSS_COMPILE)objdump

# Build flags
CFLAGS = -std=gnu99 -ffreestanding -O2 -Wall -Wextra
CPPFLAGS = -I include/kernel -I include/libc
ASFLAGS = --32
LDFLAGS = -nostdlib

# Debug settings
DEBUG ?= 0
ifeq ($(DEBUG), 1)
    CFLAGS += -g -DDEBUG
    ASFLAGS += -g
endif

# Target settings
KERNEL_LOAD_ADDR = 0x100000
KERNEL_VIRTUAL_BASE = 0xC0000000