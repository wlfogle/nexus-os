#ifndef KERNEL_PAGING_H
#define KERNEL_PAGING_H

#include "../libc/stdint.h"
#include "pmem.h"

#define PAGE_TABLE_ENTRIES 1024
#define PAGE_DIR_ENTRIES 1024

void paging_init(void);
void enable_paging(void);
uint32_t *get_page_directory(void);
void map_page(uint32_t virt, uint32_t phys, int flags);
void unmap_page(uint32_t virt);

#endif
