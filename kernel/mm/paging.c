#include "../../include/kernel/paging.h"
#include "../../include/kernel/serial.h"

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
    
    for (int i = 0; i < 1024; i++) {
        page_tables[0][i] = (i * PAGE_SIZE) | PAGE_PRESENT | PAGE_WRITABLE;
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
