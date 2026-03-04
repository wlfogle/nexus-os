#ifndef KERNEL_H
#define KERNEL_H

#include "../libc/stddef.h"
#include "../libc/stdint.h"

/* Multiboot information structure */
struct multiboot_info {
    uint32_t flags;
    uint32_t mem_lower;
    uint32_t mem_upper;
    uint32_t boot_device;
    uint32_t cmdline;
    uint32_t mods_count;
    uint32_t mods_addr;
    uint32_t syms[4];
    uint32_t mmap_length;
    uint32_t mmap_addr;
};

/* Early kernel initialization */
void kernel_main(struct multiboot_info *mbi, uint32_t magic);

#endif /* KERNEL_H */
