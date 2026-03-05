#ifndef KERNEL_PAGING_H
#define KERNEL_PAGING_H

#include "../libc/stdint.h"
#include "pmem.h"

#define PAGE_TABLE_ENTRIES 1024
#define PAGE_DIR_ENTRIES 1024
#define PAGE_SIZE 4096

/* Page protection flags */
#define PAGING_READ 0x1
#define PAGING_WRITE 0x2
#define PAGING_EXEC 0x4
#define PAGING_USER 0x8

void paging_init(void);
void enable_paging(void);
uint32_t *get_page_directory(void);
void map_page(uint32_t virt, uint32_t phys, int flags);
void unmap_page(uint32_t virt);

/* Per-task memory protection */
int paging_init_task(uint32_t task_id);
int paging_map_page(uint32_t task_id, uint32_t virtual_addr, uint32_t physical_addr, uint32_t flags);
int paging_unmap_page(uint32_t task_id, uint32_t virtual_addr);
uint32_t paging_get_page_directory(uint32_t task_id);
int paging_cleanup_task(uint32_t task_id);

/* Shared memory regions */
int paging_create_shared_region(uint32_t task_id, uint32_t address, uint32_t size, uint32_t flags);
int paging_attach_shared_region(uint32_t task_id, int region_id);
int paging_detach_shared_region(uint32_t task_id, int region_id);

#endif
