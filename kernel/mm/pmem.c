#include "../../include/kernel/pmem.h"
#include "../../include/kernel/serial.h"

#define BITMAP_SIZE 1024
static uint8_t page_bitmap[BITMAP_SIZE];
static uint32_t total_pages = 0;
static uint32_t free_pages = 0;

void pmem_init(uint32_t total_memory_bytes)
{
    total_pages = total_memory_bytes / PAGE_SIZE;
    if (total_pages > BITMAP_SIZE * 8) {
        total_pages = BITMAP_SIZE * 8;
    }
    free_pages = total_pages;
    
    for (int i = 0; i < BITMAP_SIZE; i++) {
        page_bitmap[i] = 0;
    }
    
    serial_puts("Physical memory manager initialized\n");
    serial_puts("Total pages: ");
    serial_puts(__PRETTY_FUNCTION__);
    serial_puts("\n");
}

static void set_bit(uint32_t page_num)
{
    uint32_t byte_idx = page_num / 8;
    uint32_t bit_idx = page_num % 8;
    if (byte_idx < BITMAP_SIZE) {
        page_bitmap[byte_idx] |= (1 << bit_idx);
    }
}

static void clear_bit(uint32_t page_num)
{
    uint32_t byte_idx = page_num / 8;
    uint32_t bit_idx = page_num % 8;
    if (byte_idx < BITMAP_SIZE) {
        page_bitmap[byte_idx] &= ~(1 << bit_idx);
    }
}

static int get_bit(uint32_t page_num)
{
    uint32_t byte_idx = page_num / 8;
    uint32_t bit_idx = page_num % 8;
    if (byte_idx < BITMAP_SIZE) {
        return (page_bitmap[byte_idx] >> bit_idx) & 1;
    }
    return 1;
}

uint32_t pmem_alloc_page(void)
{
    for (uint32_t i = 0; i < total_pages; i++) {
        if (!get_bit(i)) {
            set_bit(i);
            free_pages--;
            return i * PAGE_SIZE;
        }
    }
    return 0;
}

void pmem_free_page(uint32_t page_addr)
{
    uint32_t page_num = page_addr / PAGE_SIZE;
    if (page_num < total_pages && get_bit(page_num)) {
        clear_bit(page_num);
        free_pages++;
    }
}

uint32_t pmem_alloc_pages(int num_pages)
{
    if (num_pages <= 0 || free_pages < num_pages) {
        return 0;
    }
    
    for (uint32_t i = 0; i <= total_pages - num_pages; i++) {
        int found = 1;
        for (int j = 0; j < num_pages; j++) {
            if (get_bit(i + j)) {
                found = 0;
                break;
            }
        }
        if (found) {
            for (int j = 0; j < num_pages; j++) {
                set_bit(i + j);
            }
            free_pages -= num_pages;
            return i * PAGE_SIZE;
        }
    }
    return 0;
}

void pmem_free_pages(uint32_t page_addr, int num_pages)
{
    uint32_t page_num = page_addr / PAGE_SIZE;
    for (int i = 0; i < num_pages; i++) {
        if (page_num + i < total_pages && get_bit(page_num + i)) {
            clear_bit(page_num + i);
            free_pages++;
        }
    }
}

uint32_t pmem_get_free_pages(void)
{
    return free_pages;
}

uint32_t pmem_get_total_pages(void)
{
    return total_pages;
}
