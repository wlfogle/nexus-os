#include "../../include/kernel/resource_scheduler.h"
#include "../../include/kernel/gpu.h"
#include "../../include/kernel/pmem.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_GPU_TASKS 64
#define MAX_TASK_HINTS 256

typedef struct {
    gpu_sched_context_t context;
    int in_use;
} gpu_task_entry_t;

typedef struct {
    uint32_t task_id;
    sched_hint_t hint;
    task_resource_req_t resources;
    int in_use;
} task_hint_entry_t;

static gpu_task_entry_t gpu_tasks[MAX_GPU_TASKS];
static task_hint_entry_t task_hints[MAX_TASK_HINTS];
static system_resources_t system_resources = {0};
static sched_stats_t sched_stats = {0};
static int power_saving_enabled = 0;
static uint32_t thermal_level = 0;

void resource_scheduler_init(void)
{
    memset(gpu_tasks, 0, sizeof(gpu_tasks));
    memset(task_hints, 0, sizeof(task_hints));
    memset(&system_resources, 0, sizeof(system_resources));
    memset(&sched_stats, 0, sizeof(sched_stats));
    
    system_resources.total_memory = 128 * 1024 * 1024;  /* 128 MB */
    system_resources.available_memory = system_resources.total_memory;
    system_resources.gpu_memory_available = 256 * 1024 * 1024;  /* 256 MB GPU */
    system_resources.memory_pressure = MEM_PRESSURE_NORMAL;
    system_resources.thermal_level = 0;
    
    power_saving_enabled = 0;
    thermal_level = 0;
    
    serial_puts("[resource_scheduler] Resource-aware scheduler initialized\n");
}

int sched_register_gpu_task(uint32_t task_id, gpu_sched_context_t *context)
{
    if (!context || task_id == 0) return -1;
    
    /* Find free GPU task slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_GPU_TASKS; i++) {
        if (!gpu_tasks[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return -1;
    
    gpu_task_entry_t *entry = &gpu_tasks[free_idx];
    memcpy(&entry->context, context, sizeof(gpu_sched_context_t));
    entry->in_use = 1;
    
    system_resources.active_gpu_tasks++;
    sched_stats.gpu_task_count++;
    
    serial_printf("[resource_scheduler] Registered GPU task %d (GPU=%d, memory=%d)\n",
                  task_id, context->gpu_device_id, context->estimated_gpu_memory);
    
    return 0;
}

int sched_unregister_gpu_task(uint32_t task_id)
{
    if (task_id == 0) return -1;
    
    for (int i = 0; i < MAX_GPU_TASKS; i++) {
        if (gpu_tasks[i].in_use && gpu_tasks[i].context.task_id == task_id) {
            system_resources.available_memory += gpu_tasks[i].context.estimated_gpu_memory;
            system_resources.active_gpu_tasks--;
            gpu_tasks[i].in_use = 0;
            
            return 0;
        }
    }
    
    return -1;
}

int sched_get_gpu_context(uint32_t task_id, gpu_sched_context_t *out_context)
{
    if (!out_context || task_id == 0) return -1;
    
    for (int i = 0; i < MAX_GPU_TASKS; i++) {
        if (gpu_tasks[i].in_use && gpu_tasks[i].context.task_id == task_id) {
            memcpy(out_context, &gpu_tasks[i].context, sizeof(gpu_sched_context_t));
            return 0;
        }
    }
    
    return -1;
}

system_resources_t *sched_get_system_resources(void)
{
    /* Update memory pressure based on availability */
    if (system_resources.available_memory < system_resources.total_memory / 10) {
        system_resources.memory_pressure = MEM_PRESSURE_CRITICAL;
        sched_stats.memory_pressure_events++;
    } else if (system_resources.available_memory < system_resources.total_memory / 4) {
        system_resources.memory_pressure = MEM_PRESSURE_HIGH;
    } else if (system_resources.available_memory < system_resources.total_memory / 2) {
        system_resources.memory_pressure = MEM_PRESSURE_ELEVATED;
    } else {
        system_resources.memory_pressure = MEM_PRESSURE_NORMAL;
    }
    
    return &system_resources;
}

memory_pressure_t sched_get_memory_pressure(void)
{
    system_resources_t *res = sched_get_system_resources();
    return res->memory_pressure;
}

uint32_t sched_get_available_gpu_memory(uint32_t gpu_device_id)
{
    (void)gpu_device_id;  /* Simplified: return global available GPU memory */
    
    return system_resources.gpu_memory_available;
}

int sched_set_task_hint(uint32_t task_id, sched_hint_t hint)
{
    if (task_id == 0 || hint < 0 || hint > 5) return -1;
    
    /* Find or create hint entry */
    int free_idx = -1;
    for (int i = 0; i < MAX_TASK_HINTS; i++) {
        if (task_hints[i].in_use && task_hints[i].task_id == task_id) {
            task_hints[i].hint = hint;
            return 0;
        }
        
        if (!task_hints[i].in_use && free_idx < 0) {
            free_idx = i;
        }
    }
    
    if (free_idx >= 0) {
        task_hints[free_idx].task_id = task_id;
        task_hints[free_idx].hint = hint;
        task_hints[free_idx].in_use = 1;
        return 0;
    }
    
    return -1;
}

sched_hint_t sched_get_task_hint(uint32_t task_id)
{
    if (task_id == 0) return SCHED_HINT_NONE;
    
    for (int i = 0; i < MAX_TASK_HINTS; i++) {
        if (task_hints[i].in_use && task_hints[i].task_id == task_id) {
            return task_hints[i].hint;
        }
    }
    
    return SCHED_HINT_NONE;
}

int sched_set_task_resources(uint32_t task_id, task_resource_req_t *req)
{
    if (!req || task_id == 0) return -1;
    
    for (int i = 0; i < MAX_TASK_HINTS; i++) {
        if (task_hints[i].in_use && task_hints[i].task_id == task_id) {
            memcpy(&task_hints[i].resources, req, sizeof(task_resource_req_t));
            return 0;
        }
    }
    
    return -1;
}

int sched_should_defer_gpu_task(uint32_t task_id)
{
    if (task_id == 0) return -1;
    
    sched_hint_t hint = sched_get_task_hint(task_id);
    
    /* Defer if memory pressure is critical */
    if (sched_get_memory_pressure() == MEM_PRESSURE_CRITICAL) {
        if (hint == SCHED_HINT_GPU_INTENSIVE) {
            sched_stats.deferred_gpu_tasks++;
            return 1;
        }
    }
    
    /* Defer if thermal level is high */
    if (thermal_level > 80) {
        sched_stats.thermal_throttle_events++;
        return 1;
    }
    
    return 0;
}

int sched_can_allocate_gpu_memory(uint32_t gpu_device_id, uint32_t size)
{
    (void)gpu_device_id;  /* Simplified */
    
    return (system_resources.gpu_memory_available >= size) ? 1 : 0;
}

uint32_t sched_get_best_gpu_device(void)
{
    /* Simplified: always return device 0 */
    return 0;
}

int sched_get_thermal_level(void)
{
    return thermal_level;
}

int sched_thermal_throttle(uint32_t level)
{
    if (level > 100) return -1;
    
    thermal_level = level;
    
    if (level >= 90) {
        serial_printf("[resource_scheduler] THERMAL CRITICAL: %d%%\n", level);
        sched_stats.thermal_throttle_events++;
    }
    
    return 0;
}

int sched_is_thermal_critical(void)
{
    return (thermal_level >= 90) ? 1 : 0;
}

int sched_enable_power_saving(void)
{
    power_saving_enabled = 1;
    
    serial_puts("[resource_scheduler] Power saving mode enabled\n");
    return 0;
}

int sched_disable_power_saving(void)
{
    power_saving_enabled = 0;
    
    serial_puts("[resource_scheduler] Power saving mode disabled\n");
    return 0;
}

int sched_is_power_saving_enabled(void)
{
    return power_saving_enabled;
}

sched_stats_t *sched_get_stats(void)
{
    /* Calculate averages */
    if (system_resources.active_gpu_tasks > 0) {
        uint32_t total_utilization = 0;
        for (int i = 0; i < MAX_GPU_TASKS; i++) {
            if (gpu_tasks[i].in_use) {
                total_utilization += gpu_tasks[i].context.gpu_utilization_pct;
            }
        }
        sched_stats.avg_gpu_utilization = total_utilization / system_resources.active_gpu_tasks;
    }
    
    sched_stats.avg_memory_pressure = (uint32_t)sched_get_memory_pressure();
    
    return &sched_stats;
}
