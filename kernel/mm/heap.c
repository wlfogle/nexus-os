#include "../../include/kernel/heap.h"
#include "../../include/kernel/pmem.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

#define HEAP_START 0x10000
#define HEAP_MAX_SIZE (256 * PAGE_SIZE)

struct allocation_header {
    uint32_t size;
    int free;
    struct allocation_header *next;
};

static struct allocation_header *heap_start = NULL;
static uint32_t heap_used = 0;

void heap_init(void)
{
    heap_start = (struct allocation_header *)HEAP_START;
    heap_start->size = HEAP_MAX_SIZE - sizeof(struct allocation_header);
    heap_start->free = 1;
    heap_start->next = NULL;
    heap_used = 0;
    serial_puts("Kernel heap initialized\n");
}

void *kmalloc(uint32_t size)
{
    if (size == 0) return NULL;
    
    size += sizeof(struct allocation_header);
    struct allocation_header *block = heap_start;
    
    while (block) {
        if (block->free && block->size >= size) {
            block->free = 0;
            
            if (block->size > size + sizeof(struct allocation_header)) {
                struct allocation_header *new_block = 
                    (struct allocation_header *)((uint32_t)block + size);
                new_block->size = block->size - size;
                new_block->free = 1;
                new_block->next = block->next;
                block->next = new_block;
                block->size = size;
            }
            
            heap_used += size;
            return (void *)((uint32_t)block + sizeof(struct allocation_header));
        }
        block = block->next;
    }
    
    return NULL;
}

void kfree(void *ptr)
{
    if (!ptr) return;
    
    struct allocation_header *block = 
        (struct allocation_header *)((uint32_t)ptr - sizeof(struct allocation_header));
    
    block->free = 1;
    heap_used -= block->size;
    
    if (block->next && block->next->free) {
        block->size += block->next->size;
        block->next = block->next->next;
    }
}
