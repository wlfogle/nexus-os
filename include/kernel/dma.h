#ifndef KERNEL_DMA_H
#define KERNEL_DMA_H

#include "../libc/stdint.h"

/* DMA buffer coherency modes */
typedef enum {
    DMA_COHERENT = 1,              /* Cache-coherent memory */
    DMA_NON_COHERENT = 2            /* Non-coherent (must flush/invalidate) */
} dma_coherency_t;

/* DMA buffer flags */
#define DMA_BUFFER_ALLOCATED 0x1
#define DMA_BUFFER_MAPPED    0x2
#define DMA_BUFFER_IN_USE    0x4
#define DMA_BUFFER_COHERENT  0x8

/* Scatter-gather entry */
typedef struct {
    uint32_t physical_addr;         /* Physical address */
    uint32_t virtual_addr;          /* Virtual address */
    uint32_t length;                /* Length in bytes */
    uint32_t flags;                 /* Entry flags */
} dma_sg_entry_t;

/* DMA buffer handle */
typedef struct {
    uint32_t buffer_id;             /* Unique buffer ID */
    uint32_t virtual_addr;          /* Virtual address (kernel) */
    uint32_t physical_addr;         /* Physical address (DMA-capable) */
    uint32_t size;                  /* Size in bytes */
    dma_coherency_t coherency;      /* Coherency mode */
    uint32_t flags;                 /* Buffer flags */
    uint32_t device_id;             /* Associated device ID */
} dma_buffer_t;

/* Scatter-gather list */
typedef struct {
    uint32_t sg_list_id;            /* Unique SG list ID */
    dma_sg_entry_t *entries;        /* Array of SG entries */
    int entry_count;                /* Number of entries */
    uint32_t total_length;          /* Total DMA length */
    uint32_t device_id;             /* Associated device ID */
} dma_sg_list_t;

/* DMA statistics */
typedef struct {
    uint32_t total_buffers_allocated;
    uint32_t total_buffers_freed;
    uint32_t sg_lists_created;
    uint32_t bytes_dma_allocated;
    uint32_t coherent_allocations;
    uint32_t non_coherent_allocations;
} dma_stats_t;

/* DMA API */
void dma_init(void);

/* Buffer allocation */
dma_buffer_t *dma_buffer_alloc(uint32_t size, dma_coherency_t coherency, uint32_t device_id);
int dma_buffer_free(uint32_t buffer_id);

/* Buffer lookup */
dma_buffer_t *dma_buffer_get(uint32_t buffer_id);
dma_buffer_t *dma_buffer_get_by_device(uint32_t device_id, int index);

/* Buffer operations */
int dma_buffer_flush(uint32_t buffer_id);
int dma_buffer_invalidate(uint32_t buffer_id);

/* Scatter-gather operations */
dma_sg_list_t *dma_sg_alloc(int entry_count, uint32_t device_id);
int dma_sg_free(uint32_t sg_list_id);
int dma_sg_add_entry(uint32_t sg_list_id, uint32_t phys_addr, uint32_t length);
dma_sg_list_t *dma_sg_get(uint32_t sg_list_id);

/* Statistics and diagnostics */
dma_stats_t *dma_get_stats(void);

#endif /* KERNEL_DMA_H */
