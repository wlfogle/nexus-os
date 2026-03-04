#ifndef KERNEL_PMEM_H
#define KERNEL_PMEM_H

#include "../libc/stdint.h"

/* Page size in bytes */
#define PAGE_SIZE 4096

/* Bitmap-based physical memory allocator */

/* Initialize physical memory manager */
void pmem_init(uint32_t total_memory_bytes);

/* Allocate a single physical page */
uint32_t pmem_alloc_page(void);

/* Free a single physical page */
void pmem_free_page(uint32_t page_addr);

/* Allocate consecutive pages */
uint32_t pmem_alloc_pages(int num_pages);

/* Free consecutive pages */
void pmem_free_pages(uint32_t page_addr, int num_pages);

/* Get the number of free pages */
uint32_t pmem_get_free_pages(void);

/* Get total number of pages */
uint32_t pmem_get_total_pages(void);

#endif /* KERNEL_PMEM_H */
