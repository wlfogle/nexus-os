#include "../../include/kernel/container.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_CONTAINERS 32
#define MAX_GPUS_PER_CONTAINER 4

typedef struct {
    container_t metadata;
    container_resources_t resources;
    container_stats_t stats;
    uint32_t gpu_devices[MAX_GPUS_PER_CONTAINER];
    uint8_t gpu_count;
    int in_use;
} container_entry_t;

static container_entry_t containers[MAX_CONTAINERS];
static uint32_t container_id_counter = 1;

void container_runtime_init(void)
{
    memset(containers, 0, sizeof(containers));
    container_id_counter = 1;
    
    serial_puts("[container] Container runtime subsystem initialized\n");
}

uint32_t container_create(const char *name, container_limits_t *limits)
{
    if (!name || !limits) return 0;
    
    /* Find free container slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (!containers[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;  /* No free containers */
    
    container_entry_t *entry = &containers[free_idx];
    entry->metadata.container_id = container_id_counter++;
    entry->metadata.state = CONTAINER_CREATED;
    entry->metadata.created_time = 0;  /* Would be set from timer */
    entry->metadata.process_id = 0;
    entry->metadata.memory_limit = limits->max_memory;
    entry->metadata.priority = 128;  /* Default priority */
    entry->metadata.allow_gpu = (limits->max_gpus > 0) ? 1 : 0;
    entry->metadata.allow_networking = 1;
    
    /* Copy name */
    int name_len = 0;
    while (name[name_len] && name_len < 63) {
        entry->metadata.name[name_len] = name[name_len];
        name_len++;
    }
    entry->metadata.name[name_len] = '\0';
    
    /* Initialize resources */
    entry->resources.memory_used = 0;
    entry->resources.gpu_memory_used = 0;
    entry->resources.active_threads = 0;
    entry->resources.gpus_assigned = 0;
    entry->resources.inference_quota = limits->max_cpu_shares * 100;
    
    entry->gpu_count = 0;
    entry->in_use = 1;
    
    serial_printf("[container] Created container %d: %s (mem=%d)\n",
                  entry->metadata.container_id, name, limits->max_memory);
    
    return entry->metadata.container_id;
}

int container_start(uint32_t container_id)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            if (containers[i].metadata.state == CONTAINER_CREATED || 
                containers[i].metadata.state == CONTAINER_STOPPED ||
                containers[i].metadata.state == CONTAINER_PAUSED) {
                containers[i].metadata.state = CONTAINER_RUNNING;
                containers[i].metadata.process_id = container_id * 1000 + i;  /* Simplified PID */
                
                serial_printf("[container] Started container %d\n", container_id);
                return 0;
            }
        }
    }
    
    return -1;
}

int container_stop(uint32_t container_id)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            if (containers[i].metadata.state == CONTAINER_RUNNING ||
                containers[i].metadata.state == CONTAINER_PAUSED) {
                containers[i].metadata.state = CONTAINER_STOPPED;
                containers[i].resources.active_threads = 0;
                
                serial_printf("[container] Stopped container %d\n", container_id);
                return 0;
            }
        }
    }
    
    return -1;
}

int container_pause(uint32_t container_id)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            if (containers[i].metadata.state == CONTAINER_RUNNING) {
                containers[i].metadata.state = CONTAINER_PAUSED;
                
                return 0;
            }
        }
    }
    
    return -1;
}

int container_resume(uint32_t container_id)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            if (containers[i].metadata.state == CONTAINER_PAUSED) {
                containers[i].metadata.state = CONTAINER_RUNNING;
                
                return 0;
            }
        }
    }
    
    return -1;
}

int container_destroy(uint32_t container_id)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            /* Release any assigned GPUs */
            containers[i].gpu_count = 0;
            memset(containers[i].gpu_devices, 0, sizeof(containers[i].gpu_devices));
            
            containers[i].metadata.state = CONTAINER_DESTROYED;
            containers[i].in_use = 0;
            
            serial_printf("[container] Destroyed container %d\n", container_id);
            return 0;
        }
    }
    
    return -1;
}

int container_get(uint32_t container_id, container_t *out)
{
    if (!out) return -1;
    
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            memcpy(out, &containers[i].metadata, sizeof(container_t));
            return 0;
        }
    }
    
    return -1;
}

int container_get_stats(uint32_t container_id, container_stats_t *out)
{
    if (!out) return -1;
    
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            /* Update stats from current resource state */
            containers[i].stats.container_id = container_id;
            containers[i].stats.memory_used = containers[i].resources.memory_used;
            containers[i].stats.gpu_memory_used = containers[i].resources.gpu_memory_used;
            containers[i].stats.active_threads = containers[i].resources.active_threads;
            
            memcpy(out, &containers[i].stats, sizeof(container_stats_t));
            return 0;
        }
    }
    
    return -1;
}

int container_assign_gpu(uint32_t container_id, uint32_t gpu_device_id)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            if (containers[i].gpu_count >= MAX_GPUS_PER_CONTAINER) {
                return -1;  /* Too many GPUs */
            }
            
            /* Check if already assigned */
            for (int j = 0; j < containers[i].gpu_count; j++) {
                if (containers[i].gpu_devices[j] == gpu_device_id) {
                    return -1;  /* Already assigned */
                }
            }
            
            containers[i].gpu_devices[containers[i].gpu_count] = gpu_device_id;
            containers[i].gpu_count++;
            containers[i].resources.gpus_assigned++;
            
            serial_printf("[container] Assigned GPU %d to container %d\n", gpu_device_id, container_id);
            return 0;
        }
    }
    
    return -1;
}

int container_release_gpu(uint32_t container_id, uint32_t gpu_device_id)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            for (int j = 0; j < containers[i].gpu_count; j++) {
                if (containers[i].gpu_devices[j] == gpu_device_id) {
                    /* Remove by shifting remaining devices */
                    for (int k = j; k < containers[i].gpu_count - 1; k++) {
                        containers[i].gpu_devices[k] = containers[i].gpu_devices[k + 1];
                    }
                    containers[i].gpu_count--;
                    containers[i].resources.gpus_assigned--;
                    
                    return 0;
                }
            }
        }
    }
    
    return -1;
}

int container_can_allocate_memory(uint32_t container_id, uint32_t size)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            uint32_t available = containers[i].metadata.memory_limit - 
                               containers[i].resources.memory_used;
            return (available >= size) ? 1 : 0;
        }
    }
    
    return 0;
}

int container_set_priority(uint32_t container_id, uint16_t priority)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            containers[i].metadata.priority = priority;
            return 0;
        }
    }
    
    return -1;
}

uint32_t container_list(container_t *containers_out, uint32_t max_containers)
{
    if (!containers_out || max_containers == 0) return 0;
    
    uint32_t count = 0;
    for (int i = 0; i < MAX_CONTAINERS && count < max_containers; i++) {
        if (containers[i].in_use) {
            memcpy(&containers_out[count], &containers[i].metadata, sizeof(container_t));
            count++;
        }
    }
    
    return count;
}

uint32_t container_get_count(void)
{
    uint32_t count = 0;
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use) {
            count++;
        }
    }
    
    return count;
}

uint64_t container_get_total_memory(void)
{
    uint64_t total = 0;
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use) {
            total += containers[i].resources.memory_used;
        }
    }
    
    return total;
}

uint64_t container_get_total_gpu_memory(void)
{
    uint64_t total = 0;
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use) {
            total += containers[i].resources.gpu_memory_used;
        }
    }
    
    return total;
}

int container_set_memory_limit(uint32_t container_id, uint32_t limit)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            containers[i].metadata.memory_limit = limit;
            return 0;
        }
    }
    
    return -1;
}

int container_set_gpu_memory_limit(uint32_t container_id, uint32_t limit)
{
    (void)container_id;  /* Simplified: no per-container GPU memory limit tracking */
    (void)limit;
    
    return 0;
}

int container_get_resources(uint32_t container_id, container_resources_t *out)
{
    if (!out) return -1;
    
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            memcpy(out, &containers[i].resources, sizeof(container_resources_t));
            return 0;
        }
    }
    
    return -1;
}

int container_set_gpu_access(uint32_t container_id, int enabled)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            containers[i].metadata.allow_gpu = enabled ? 1 : 0;
            return 0;
        }
    }
    
    return -1;
}

int container_set_network_access(uint32_t container_id, int enabled)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            containers[i].metadata.allow_networking = enabled ? 1 : 0;
            return 0;
        }
    }
    
    return -1;
}

container_state_t container_get_state(uint32_t container_id)
{
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (containers[i].in_use && containers[i].metadata.container_id == container_id) {
            return containers[i].metadata.state;
        }
    }
    
    return CONTAINER_DESTROYED;
}
