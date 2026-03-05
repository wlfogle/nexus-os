#ifndef KERNEL_GPU_H
#define KERNEL_GPU_H

#include "../libc/stdint.h"

/* GPU Device Types */
typedef enum {
    GPU_TYPE_DISCRETE_NVIDIA = 1,
    GPU_TYPE_DISCRETE_AMD = 2,
    GPU_TYPE_DISCRETE_INTEL = 3,
    GPU_TYPE_INTEGRATED = 4,
    GPU_TYPE_QEMU_FALLBACK = 5
} gpu_type_t;

/* GPU Memory Types */
typedef enum {
    GPU_MEM_VRAM = 1,           /* Discrete video memory */
    GPU_MEM_UNIFIED = 2,        /* Unified memory (APU/iGPU) */
    GPU_MEM_HOST_PINNED = 3,    /* Host memory pinned for DMA */
    GPU_MEM_MANAGED = 4         /* Automatically managed */
} gpu_memory_type_t;

/* GPU Compute Capability */
typedef enum {
    GPU_COMPUTE_SHADER = 1,
    GPU_COMPUTE_COMPUTE = 2,    /* General compute (CUDA/HIP/OpenCL) */
    GPU_COMPUTE_RT = 3,         /* Ray tracing */
    GPU_COMPUTE_TENSOR = 4      /* Tensor operations (ML accelerators) */
} gpu_compute_t;

/* GPU Device Information */
typedef struct {
    uint32_t device_id;
    const char *name;
    gpu_type_t type;
    uint32_t compute_capability;
    uint32_t total_memory;       /* Total VRAM/memory in bytes */
    uint32_t free_memory;        /* Free memory in bytes */
    uint32_t max_threads_per_block;
    uint32_t max_blocks;
    uint32_t warp_size;          /* Threads per warp (32 for NVIDIA, 64 for AMD) */
    int pci_bus;
    int pci_slot;
    int pci_function;
} gpu_device_t;

/* GPU Memory Allocation */
typedef struct {
    uint32_t alloc_id;
    gpu_memory_type_t mem_type;
    void *host_ptr;              /* Host-side pointer */
    uint32_t device_ptr;         /* Device-side address */
    uint32_t size;
    uint32_t device_id;
    uint32_t flags;
} gpu_memory_t;

/* GPU Command/Kernel Launch */
typedef struct {
    uint32_t kernel_id;
    const char *kernel_name;
    uint32_t blocks;
    uint32_t threads_per_block;
    uint32_t shared_memory;
    uint32_t device_id;
} gpu_kernel_launch_t;

/* GPU Statistics */
typedef struct {
    uint32_t total_devices;
    uint32_t active_devices;
    uint64_t total_allocations;
    uint64_t total_kernels_launched;
    uint64_t total_memory_allocated;
    uint64_t current_memory_used;
    uint32_t fallback_mode;      /* 1 if using QEMU fallback, 0 if hardware */
} gpu_stats_t;

/* GPU Management API */
void gpu_init(void);
int gpu_detect_devices(void);
gpu_device_t *gpu_get_device(int index);
int gpu_device_count(void);
int gpu_enable_device(uint32_t device_id);
int gpu_disable_device(uint32_t device_id);

/* GPU Memory Management */
gpu_memory_t *gpu_malloc(uint32_t device_id, uint32_t size, gpu_memory_type_t mem_type);
int gpu_free(uint32_t alloc_id);
gpu_memory_t *gpu_mem_get(uint32_t alloc_id);
int gpu_mem_copy_to_device(uint32_t alloc_id, const void *host_ptr, uint32_t size);
int gpu_mem_copy_from_device(uint32_t alloc_id, void *host_ptr, uint32_t size);

/* GPU Kernel/Compute Operations */
int gpu_kernel_launch(gpu_kernel_launch_t *launch);
int gpu_kernel_synchronize(uint32_t device_id);
int gpu_stream_synchronize(uint32_t stream_id);

/* GPU Statistics and Diagnostics */
gpu_stats_t *gpu_get_stats(void);
int gpu_get_memory_info(uint32_t device_id, uint32_t *total, uint32_t *free);

/* GPU Fallback Mode (QEMU) */
int gpu_enable_fallback_mode(void);
int gpu_is_fallback_mode(void);

#endif /* KERNEL_GPU_H */
