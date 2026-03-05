#include "../../include/kernel/dma.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/heap.h"
#include <string.h>

#define MAX_DMA_BUFFERS 64
#define MAX_SG_LISTS 32
#define MAX_SG_ENTRIES_PER_LIST 16

typedef struct {
    dma_buffer_t buffer;
    int in_use;
} dma_buffer_entry_t;

typedef struct {
    dma_sg_list_t sg_list;
    int in_use;
} dma_sg_list_entry_t;

static dma_buffer_entry_t dma_buffers[MAX_DMA_BUFFERS];
static dma_sg_list_entry_t dma_sg_lists[MAX_SG_LISTS];
static dma_stats_t dma_stats = {0};
static uint32_t next_buffer_id = 1;
static uint32_t next_sg_list_id = 1;

void dma_init(void)
{
    memset(dma_buffers, 0, sizeof(dma_buffers));
    memset(dma_sg_lists, 0, sizeof(dma_sg_lists));
    memset(&dma_stats, 0, sizeof(dma_stats));
    next_buffer_id = 1;
    next_sg_list_id = 1;
    
    serial_puts("[dma] DMA manager initialized\n");
}

dma_buffer_t *dma_buffer_alloc(uint32_t size, dma_coherency_t coherency, uint32_t device_id)
{
    if (size == 0 || coherency < 1 || coherency > 2) {
        return NULL;
    }
    
    /* Find free buffer slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_DMA_BUFFERS; i++) {
        if (!dma_buffers[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) {
        serial_puts("[dma] DMA buffer table full\n");
        return NULL;
    }
    
    /* Allocate memory (simplified: assume physical = virtual) */
    void *virt_addr = kmalloc(size);
    if (!virt_addr) {
        serial_puts("[dma] Failed to allocate DMA buffer memory\n");
        return NULL;
    }
    
    /* Initialize buffer */
    dma_buffer_entry_t *entry = &dma_buffers[free_idx];
    entry->buffer.buffer_id = next_buffer_id++;
    entry->buffer.virtual_addr = (uint32_t)virt_addr;
    entry->buffer.physical_addr = (uint32_t)virt_addr;  /* Simplified */
    entry->buffer.size = size;
    entry->buffer.coherency = coherency;
    entry->buffer.device_id = device_id;
    entry->buffer.flags = DMA_BUFFER_ALLOCATED;
    
    if (coherency == DMA_COHERENT) {
        entry->buffer.flags |= DMA_BUFFER_COHERENT;
        dma_stats.coherent_allocations++;
    } else {
        dma_stats.non_coherent_allocations++;
    }
    
    entry->in_use = 1;
    dma_stats.total_buffers_allocated++;
    dma_stats.bytes_dma_allocated += size;
    
    serial_printf("[dma] Allocated DMA buffer %d (size=%d, device=%d)\n",
                  entry->buffer.buffer_id, size, device_id);
    
    return &entry->buffer;
}

int dma_buffer_free(uint32_t buffer_id)
{
    if (buffer_id == 0) return -1;
    
    for (int i = 0; i < MAX_DMA_BUFFERS; i++) {
        if (dma_buffers[i].in_use && dma_buffers[i].buffer.buffer_id == buffer_id) {
            dma_buffer_t *buf = &dma_buffers[i].buffer;
            
            /* Free the allocated memory */
            kfree((void *)buf->virtual_addr);
            
            dma_buffers[i].in_use = 0;
            dma_stats.total_buffers_freed++;
            
            serial_printf("[dma] Freed DMA buffer %d\n", buffer_id);
            return 0;
        }
    }
    
    return -1;
}

dma_buffer_t *dma_buffer_get(uint32_t buffer_id)
{
    if (buffer_id == 0) return NULL;
    
    for (int i = 0; i < MAX_DMA_BUFFERS; i++) {
        if (dma_buffers[i].in_use && dma_buffers[i].buffer.buffer_id == buffer_id) {
            return &dma_buffers[i].buffer;
        }
    }
    
    return NULL;
}

dma_buffer_t *dma_buffer_get_by_device(uint32_t device_id, int index)
{
    if (index < 0) return NULL;
    
    int count = 0;
    for (int i = 0; i < MAX_DMA_BUFFERS; i++) {
        if (dma_buffers[i].in_use && dma_buffers[i].buffer.device_id == device_id) {
            if (count == index) {
                return &dma_buffers[i].buffer;
            }
            count++;
        }
    }
    
    return NULL;
}

int dma_buffer_flush(uint32_t buffer_id)
{
    dma_buffer_t *buf = dma_buffer_get(buffer_id);
    if (!buf) return -1;
    
    /* Simplified: actual flush would involve cache operations */
    if (buf->coherency == DMA_NON_COHERENT) {
        /* Flush L1/L2 cache for this buffer range */
        /* This is a placeholder for actual cache flush operations */
    }
    
    return 0;
}

int dma_buffer_invalidate(uint32_t buffer_id)
{
    dma_buffer_t *buf = dma_buffer_get(buffer_id);
    if (!buf) return -1;
    
    /* Simplified: actual invalidate would involve cache operations */
    if (buf->coherency == DMA_NON_COHERENT) {
        /* Invalidate L1/L2 cache for this buffer range */
        /* This is a placeholder for actual cache invalidate operations */
    }
    
    return 0;
}

dma_sg_list_t *dma_sg_alloc(int entry_count, uint32_t device_id)
{
    if (entry_count <= 0 || entry_count > MAX_SG_ENTRIES_PER_LIST) {
        return NULL;
    }
    
    /* Find free SG list slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_SG_LISTS; i++) {
        if (!dma_sg_lists[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) {
        serial_puts("[dma] DMA SG list table full\n");
        return NULL;
    }
    
    /* Allocate SG entries array */
    dma_sg_entry_t *entries = (dma_sg_entry_t *)kmalloc(
        sizeof(dma_sg_entry_t) * entry_count);
    if (!entries) {
        serial_puts("[dma] Failed to allocate SG entries\n");
        return NULL;
    }
    
    memset(entries, 0, sizeof(dma_sg_entry_t) * entry_count);
    
    /* Initialize SG list */
    dma_sg_list_entry_t *sg_entry = &dma_sg_lists[free_idx];
    sg_entry->sg_list.sg_list_id = next_sg_list_id++;
    sg_entry->sg_list.entries = entries;
    sg_entry->sg_list.entry_count = 0;
    sg_entry->sg_list.total_length = 0;
    sg_entry->sg_list.device_id = device_id;
    sg_entry->in_use = 1;
    
    dma_stats.sg_lists_created++;
    
    serial_printf("[dma] Allocated SG list %d (capacity=%d, device=%d)\n",
                  sg_entry->sg_list.sg_list_id, entry_count, device_id);
    
    return &sg_entry->sg_list;
}

int dma_sg_free(uint32_t sg_list_id)
{
    if (sg_list_id == 0) return -1;
    
    for (int i = 0; i < MAX_SG_LISTS; i++) {
        if (dma_sg_lists[i].in_use && dma_sg_lists[i].sg_list.sg_list_id == sg_list_id) {
            kfree(dma_sg_lists[i].sg_list.entries);
            dma_sg_lists[i].in_use = 0;
            
            serial_printf("[dma] Freed SG list %d\n", sg_list_id);
            return 0;
        }
    }
    
    return -1;
}

int dma_sg_add_entry(uint32_t sg_list_id, uint32_t phys_addr, uint32_t length)
{
    if (sg_list_id == 0 || length == 0) return -1;
    
    dma_sg_list_t *sg = dma_sg_get(sg_list_id);
    if (!sg) return -1;
    
    /* Check if we have space for another entry */
    if (sg->entry_count >= MAX_SG_ENTRIES_PER_LIST) {
        return -1;
    }
    
    /* Add entry */
    dma_sg_entry_t *entry = &sg->entries[sg->entry_count];
    entry->physical_addr = phys_addr;
    entry->virtual_addr = phys_addr;  /* Simplified */
    entry->length = length;
    entry->flags = 0;
    
    sg->entry_count++;
    sg->total_length += length;
    
    return sg->entry_count - 1;
}

dma_sg_list_t *dma_sg_get(uint32_t sg_list_id)
{
    if (sg_list_id == 0) return NULL;
    
    for (int i = 0; i < MAX_SG_LISTS; i++) {
        if (dma_sg_lists[i].in_use && dma_sg_lists[i].sg_list.sg_list_id == sg_list_id) {
            return &dma_sg_lists[i].sg_list;
        }
    }
    
    return NULL;
}

dma_stats_t *dma_get_stats(void)
{
    return &dma_stats;
}
