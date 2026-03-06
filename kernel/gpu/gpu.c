#include "../../include/kernel/gpu.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/device.h"
#include "../../include/kernel/heap.h"
#include <string.h>

#define MAX_GPU_DEVICES 8
#define MAX_GPU_ALLOCATIONS 256
#define MAX_GPU_STREAMS 32

typedef struct {
    gpu_device_t device;
    int in_use;
    int enabled;
} gpu_device_entry_t;

typedef struct {
    gpu_memory_t memory;
    int in_use;
} gpu_memory_entry_t;

static gpu_device_entry_t gpu_devices[MAX_GPU_DEVICES];
static gpu_memory_entry_t gpu_allocations[MAX_GPU_ALLOCATIONS];
static gpu_stats_t gpu_stats = {0};
static int gpu_fallback_mode = 1;  /* Start with QEMU fallback */
static uint32_t next_device_id = 1;
static uint32_t next_alloc_id = 1;

void gpu_init(void)
{
    memset(gpu_devices, 0, sizeof(gpu_devices));
    memset(gpu_allocations, 0, sizeof(gpu_allocations));
    memset(&gpu_stats, 0, sizeof(gpu_stats));
    
    gpu_stats.fallback_mode = 1;  /* Default to QEMU fallback */
    
    serial_puts("[gpu] GPU abstraction layer initialized\n");
}

int gpu_detect_devices(void)
{
    int found = 0;
    
    /* Attempt to detect real GPU devices via PCI */
    /* For now, we create a virtual QEMU fallback GPU */
    
    if (gpu_stats.total_devices >= MAX_GPU_DEVICES) {
        return 0;
    }
    
    /* Create QEMU fallback GPU device */
    gpu_device_entry_t *entry = &gpu_devices[gpu_stats.total_devices];
    entry->device.device_id = next_device_id++;
    entry->device.name = "QEMU Virtual GPU";
    entry->device.type = GPU_TYPE_QEMU_FALLBACK;
    entry->device.compute_capability = GPU_COMPUTE_COMPUTE;
    entry->device.total_memory = 256 * 1024 * 1024;  /* 256 MB virtual memory */
    entry->device.free_memory = entry->device.total_memory;
    entry->device.max_threads_per_block = 1024;
    entry->device.max_blocks = 65535;
    entry->device.warp_size = 32;
    entry->device.pci_bus = 0;
    entry->device.pci_slot = 0;
    entry->device.pci_function = 0;
    entry->in_use = 1;
    entry->enabled = 0;
    
    gpu_stats.total_devices++;
    found++;
    
    serial_printf("[gpu] Detected %d GPU device(s)\n", found);
    return found;
}

gpu_device_t *gpu_get_device(int index)
{
    if (index < 0 || (uint32_t)index >= gpu_stats.total_devices) {
        return NULL;
    }
    
    return &gpu_devices[index].device;
}

int gpu_device_count(void)
{
    return gpu_stats.total_devices;
}

int gpu_enable_device(uint32_t device_id)
{
    for (uint32_t i = 0; i < gpu_stats.total_devices; i++) {
        if (gpu_devices[i].in_use && gpu_devices[i].device.device_id == device_id) {
            gpu_devices[i].enabled = 1;
            gpu_stats.active_devices++;
            
            serial_printf("[gpu] Enabled GPU device %d: %s\n", 
                          device_id, gpu_devices[i].device.name);
            return 0;
        }
    }
    
    return -1;
}

int gpu_disable_device(uint32_t device_id)
{
    for (uint32_t i = 0; i < gpu_stats.total_devices; i++) {
        if (gpu_devices[i].in_use && gpu_devices[i].device.device_id == device_id) {
            gpu_devices[i].enabled = 0;
            gpu_stats.active_devices--;
            
            serial_printf("[gpu] Disabled GPU device %d\n", device_id);
            return 0;
        }
    }
    
    return -1;
}

gpu_memory_t *gpu_malloc(uint32_t device_id, uint32_t size, gpu_memory_type_t mem_type)
{
    if (size == 0 || mem_type < 1 || mem_type > 4) {
        return NULL;
    }
    
    /* Find free allocation slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_GPU_ALLOCATIONS; i++) {
        if (!gpu_allocations[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) {
        serial_puts("[gpu] GPU allocation table full\n");
        return NULL;
    }
    
    /* Verify device exists */
    gpu_device_t *dev = gpu_get_device(0);  /* Simplified: use first device */
    if (!dev || dev->device_id != device_id) {
        return NULL;
    }
    
    /* Allocate host memory (simplified) */
    void *host_ptr = kmalloc(size);
    if (!host_ptr) {
        return NULL;
    }
    
    /* Initialize allocation */
    gpu_memory_entry_t *alloc = &gpu_allocations[free_idx];
    alloc->memory.alloc_id = next_alloc_id++;
    alloc->memory.mem_type = mem_type;
    alloc->memory.host_ptr = host_ptr;
    alloc->memory.device_ptr = (uint32_t)host_ptr;  /* Simplified */
    alloc->memory.size = size;
    alloc->memory.device_id = device_id;
    alloc->memory.flags = 0;
    alloc->in_use = 1;
    
    gpu_stats.total_allocations++;
    gpu_stats.total_memory_allocated += size;
    gpu_stats.current_memory_used += size;
    
    serial_printf("[gpu] Allocated %d bytes on GPU %d (alloc_id=%d)\n",
                  size, device_id, alloc->memory.alloc_id);
    
    return &alloc->memory;
}

int gpu_free(uint32_t alloc_id)
{
    if (alloc_id == 0) return -1;
    
    for (int i = 0; i < MAX_GPU_ALLOCATIONS; i++) {
        if (gpu_allocations[i].in_use && gpu_allocations[i].memory.alloc_id == alloc_id) {
            gpu_memory_t *mem = &gpu_allocations[i].memory;
            
            kfree(mem->host_ptr);
            gpu_stats.current_memory_used -= mem->size;
            gpu_allocations[i].in_use = 0;
            
            serial_printf("[gpu] Freed GPU allocation %d\n", alloc_id);
            return 0;
        }
    }
    
    return -1;
}

gpu_memory_t *gpu_mem_get(uint32_t alloc_id)
{
    if (alloc_id == 0) return NULL;
    
    for (int i = 0; i < MAX_GPU_ALLOCATIONS; i++) {
        if (gpu_allocations[i].in_use && gpu_allocations[i].memory.alloc_id == alloc_id) {
            return &gpu_allocations[i].memory;
        }
    }
    
    return NULL;
}

int gpu_mem_copy_to_device(uint32_t alloc_id, const void *host_ptr, uint32_t size)
{
    gpu_memory_t *mem = gpu_mem_get(alloc_id);
    if (!mem || !host_ptr || size == 0) {
        return -1;
    }
    
    if (size > mem->size) {
        return -1;
    }
    
    /* Simplified: copy to host buffer (QEMU fallback) */
    memcpy(mem->host_ptr, host_ptr, size);
    
    serial_printf("[gpu] Copied %d bytes to GPU allocation %d\n", size, alloc_id);
    return 0;
}

int gpu_mem_copy_from_device(uint32_t alloc_id, void *host_ptr, uint32_t size)
{
    gpu_memory_t *mem = gpu_mem_get(alloc_id);
    if (!mem || !host_ptr || size == 0) {
        return -1;
    }
    
    if (size > mem->size) {
        return -1;
    }
    
    /* Simplified: copy from host buffer (QEMU fallback) */
    memcpy(host_ptr, mem->host_ptr, size);
    
    serial_printf("[gpu] Copied %d bytes from GPU allocation %d\n", size, alloc_id);
    return 0;
}

int gpu_kernel_launch(gpu_kernel_launch_t *launch)
{
    if (!launch || launch->threads_per_block == 0 || launch->blocks == 0) {
        return -1;
    }
    
    gpu_stats.total_kernels_launched++;
    
    serial_printf("[gpu] Launched kernel '%s' on GPU %d (blocks=%d, threads=%d)\n",
                  launch->kernel_name, launch->device_id, 
                  launch->blocks, launch->threads_per_block);
    
    return 0;
}

int gpu_kernel_synchronize(uint32_t device_id)
{
    if (device_id == 0) return -1;
    
    serial_printf("[gpu] Synchronized GPU device %d\n", device_id);
    return 0;
}

int gpu_stream_synchronize(uint32_t stream_id)
{
    if (stream_id == 0) return -1;
    
    return 0;
}

gpu_stats_t *gpu_get_stats(void)
{
    return &gpu_stats;
}

int gpu_get_memory_info(uint32_t device_id __attribute__((unused)),
                        uint32_t *total, uint32_t *free)
{
    if (!total || !free) return -1;
    
    gpu_device_t *dev = gpu_get_device(0);  /* Simplified */
    if (!dev) return -1;
    
    *total = dev->total_memory;
    *free = dev->free_memory;
    
    return 0;
}

int gpu_enable_fallback_mode(void)
{
    gpu_fallback_mode = 1;
    gpu_stats.fallback_mode = 1;
    
    serial_puts("[gpu] Enabled QEMU fallback mode\n");
    return 0;
}

int gpu_is_fallback_mode(void)
{
    return gpu_fallback_mode;
}
