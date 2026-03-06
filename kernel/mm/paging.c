#include "../../include/kernel/paging.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/heap.h"
#include <string.h>

#define PAGE_PRESENT 0x001
#define PAGE_WRITABLE 0x002
#define PAGE_USER 0x004

static uint32_t page_directory[PAGE_DIR_ENTRIES] __attribute__((aligned(4096)));
static uint32_t page_tables[4][PAGE_TABLE_ENTRIES] __attribute__((aligned(4096)));

void paging_init(void)
{
    for (int i = 0; i < PAGE_DIR_ENTRIES; i++) {
        page_directory[i] = 0;
    }
    
    for (int t = 0; t < 4; t++) {
        for (int i = 0; i < PAGE_TABLE_ENTRIES; i++) {
            page_tables[t][i] = 0;
        }
    }
    
    for (int i = 0; i < 4; i++) {
        page_directory[i] = (uint32_t)&page_tables[i] | PAGE_PRESENT | PAGE_WRITABLE;
    }
    
    for (int t = 0; t < 4; t++) {
        for (int i = 0; i < 1024; i++) {
            uint32_t addr = (uint32_t)(t * 1024 + i) * PAGE_SIZE;
            page_tables[t][i] = addr | PAGE_PRESENT | PAGE_WRITABLE;
        }
    }
    
    serial_puts("Paging structures initialized\n");
}

void enable_paging(void)
{
    __asm__ volatile("mov %0, %%cr3" : : "r"((uint32_t)page_directory));
    uint32_t cr0;
    __asm__ volatile("mov %%cr0, %0" : "=r"(cr0));
    cr0 |= 0x80000000;
    __asm__ volatile("mov %0, %%cr0" : : "r"(cr0));
    serial_puts("Paging enabled\n");
}

uint32_t *get_page_directory(void)
{
    return page_directory;
}

void map_page(uint32_t virt, uint32_t phys, int flags)
{
    uint32_t dir_idx = virt / (PAGE_TABLE_ENTRIES * PAGE_SIZE);
    uint32_t table_idx = (virt / PAGE_SIZE) % PAGE_TABLE_ENTRIES;
    if (dir_idx < 4 && table_idx < PAGE_TABLE_ENTRIES) {
        page_tables[dir_idx][table_idx] = phys | flags | PAGE_PRESENT;
    }
}

void unmap_page(uint32_t virt)
{
    uint32_t dir_idx = virt / (PAGE_TABLE_ENTRIES * PAGE_SIZE);
    uint32_t table_idx = (virt / PAGE_SIZE) % PAGE_TABLE_ENTRIES;
    if (dir_idx < 4 && table_idx < PAGE_TABLE_ENTRIES) {
        page_tables[dir_idx][table_idx] = 0;
    }
}

/* Per-task memory protection structures */
#define MAX_PAGING_TASKS 64
#define MAX_SHARED_REGIONS 32
#define PAGE_TABLES_PER_DIR 1024
#define PTE_PRESENT 0x1
#define PTE_WRITE 0x2
#define PTE_USER 0x4
#define PTE_ACCESSED 0x20
#define PTE_DIRTY 0x40

struct shared_region {
    uint32_t address;
    uint32_t size;
    uint32_t owner_task_id;
    uint32_t share_count;
    uint32_t permissions;
};

struct task_paging {
    uint32_t task_id;
    uint32_t *page_directory;
    uint32_t page_dir_physical;
    struct shared_region *shared_pages[32];
    int shared_page_count;
};

static struct shared_region shared_regions[MAX_SHARED_REGIONS];
static int shared_region_count = 0;
static struct task_paging task_paging[MAX_PAGING_TASKS];
static int paging_task_count = 0;

int paging_init_task(uint32_t task_id)
{
    if (paging_task_count >= MAX_PAGING_TASKS) {
        return -1;
    }
    
    struct task_paging *tp = &task_paging[paging_task_count++];
    tp->task_id = task_id;
    tp->shared_page_count = 0;
    
    /* Allocate page directory (4KB for 1024 entries) */
    tp->page_directory = (uint32_t *)kmalloc(4096);
    if (!tp->page_directory) {
        paging_task_count--;
        return -1;
    }
    
    /* Initialize as empty */
    memset(tp->page_directory, 0, 4096);
    
    /* Physical address would be set during actual paging setup */
    tp->page_dir_physical = (uint32_t)tp->page_directory;
    
    serial_printf("[paging] Initialized page tables for task %d\n", task_id);
    return 0;
}

static struct task_paging *paging_find_task(uint32_t task_id)
{
    for (int i = 0; i < paging_task_count; i++) {
        if (task_paging[i].task_id == task_id) {
            return &task_paging[i];
        }
    }
    return NULL;
}

int paging_map_page(uint32_t task_id, uint32_t virtual_addr, uint32_t physical_addr, uint32_t flags)
{
    if (!virtual_addr || !physical_addr) return -1;
    
    struct task_paging *tp = paging_find_task(task_id);
    if (!tp) return -1;
    
    /* Align addresses to page boundaries */
    virtual_addr &= ~(PAGE_SIZE - 1);
    physical_addr &= ~(PAGE_SIZE - 1);
    
    uint32_t pdi = virtual_addr / (PAGE_SIZE * PAGE_TABLES_PER_DIR);
    uint32_t pti = (virtual_addr / PAGE_SIZE) % PAGE_TABLES_PER_DIR;
    
    /* Ensure page directory entry exists */
    if (tp->page_directory[pdi] == 0) {
        uint32_t *page_table = (uint32_t *)kmalloc(4096);
        if (!page_table) return -1;
        
        memset(page_table, 0, 4096);
        tp->page_directory[pdi] = ((uint32_t)page_table | PTE_PRESENT | PTE_WRITE | PTE_USER);
    }
    
    /* Get page table */
    uint32_t *page_table = (uint32_t *)(tp->page_directory[pdi] & ~0xFFF);
    
    /* Set page table entry */
    uint32_t pte = physical_addr | PTE_PRESENT;
    if (flags & PAGING_WRITE) pte |= PTE_WRITE;
    if (flags & PAGING_USER) pte |= PTE_USER;
    
    page_table[pti] = pte;
    
    return 0;
}

int paging_unmap_page(uint32_t task_id, uint32_t virtual_addr)
{
    if (!virtual_addr) return -1;
    
    struct task_paging *tp = paging_find_task(task_id);
    if (!tp) return -1;
    
    virtual_addr &= ~(PAGE_SIZE - 1);
    
    uint32_t pdi = virtual_addr / (PAGE_SIZE * PAGE_TABLES_PER_DIR);
    uint32_t pti = (virtual_addr / PAGE_SIZE) % PAGE_TABLES_PER_DIR;
    
    if (tp->page_directory[pdi] == 0) return -1;
    
    uint32_t *page_table = (uint32_t *)(tp->page_directory[pdi] & ~0xFFF);
    page_table[pti] = 0;
    
    return 0;
}

int paging_create_shared_region(uint32_t task_id, uint32_t address, uint32_t size, uint32_t flags)
{
    if (!address || !size || shared_region_count >= MAX_SHARED_REGIONS) {
        return -1;
    }
    
    struct shared_region *region = &shared_regions[shared_region_count++];
    region->address = address & ~(PAGE_SIZE - 1);
    region->size = (size + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1);
    region->owner_task_id = task_id;
    region->share_count = 1;
    region->permissions = flags;
    
    serial_printf("[paging] Created shared region at 0x%x size %d for task %d\n", 
                  region->address, region->size, task_id);
    
    return shared_region_count - 1;
}

int paging_attach_shared_region(uint32_t task_id, int region_id)
{
    if (region_id < 0 || region_id >= shared_region_count) return -1;
    
    struct task_paging *tp = paging_find_task(task_id);
    if (!tp) return -1;
    
    struct shared_region *region = &shared_regions[region_id];
    
    /* Add region to task's shared pages */
    if (tp->shared_page_count >= 32) return -1;
    
    tp->shared_pages[tp->shared_page_count++] = region;
    region->share_count++;
    
    /* Map pages into task's address space */
    for (uint32_t addr = region->address; addr < region->address + region->size; addr += PAGE_SIZE) {
        paging_map_page(task_id, addr, addr, region->permissions);
    }
    
    return 0;
}

int paging_detach_shared_region(uint32_t task_id, int region_id)
{
    if (region_id < 0 || region_id >= shared_region_count) return -1;
    
    struct task_paging *tp = paging_find_task(task_id);
    if (!tp) return -1;
    
    struct shared_region *region = &shared_regions[region_id];
    
    /* Unmap pages */
    for (uint32_t addr = region->address; addr < region->address + region->size; addr += PAGE_SIZE) {
        paging_unmap_page(task_id, addr);
    }
    
    /* Remove from shared pages list */
    for (int i = 0; i < tp->shared_page_count; i++) {
        if (tp->shared_pages[i] == region) {
            for (int j = i; j < tp->shared_page_count - 1; j++) {
                tp->shared_pages[j] = tp->shared_pages[j + 1];
            }
            tp->shared_page_count--;
            break;
        }
    }
    
    region->share_count--;
    
    return 0;
}

int paging_cleanup_task(uint32_t task_id)
{
    struct task_paging *tp = paging_find_task(task_id);
    if (!tp) return -1;
    
    /* Detach all shared regions */
    while (tp->shared_page_count > 0) {
        paging_detach_shared_region(task_id, 
            shared_regions[shared_region_count - 1].owner_task_id);
    }
    
    /* Free page directory and tables */
    if (tp->page_directory) {
        /* Free all page tables first */
        for (int i = 0; i < PAGE_TABLES_PER_DIR; i++) {
            if (tp->page_directory[i] & PTE_PRESENT) {
                uint32_t *pt = (uint32_t *)(tp->page_directory[i] & ~0xFFF);
                kfree(pt);
            }
        }
        kfree(tp->page_directory);
    }
    
    /* Remove from task list */
    for (int i = 0; i < paging_task_count; i++) {
        if (task_paging[i].task_id == task_id) {
            for (int j = i; j < paging_task_count - 1; j++) {
                task_paging[j] = task_paging[j + 1];
            }
            paging_task_count--;
            break;
        }
    }
    
    return 0;
}

uint32_t paging_get_page_directory(uint32_t task_id)
{
    struct task_paging *tp = paging_find_task(task_id);
    return tp ? tp->page_dir_physical : 0;
}
