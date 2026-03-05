#ifndef KERNEL_RESOURCE_SCHEDULER_H
#define KERNEL_RESOURCE_SCHEDULER_H

#include "../libc/stdint.h"

/* Resource Scheduling Hints */
typedef enum {
    SCHED_HINT_NONE = 0,
    SCHED_HINT_GPU_INTENSIVE = 1,    /* Task uses GPU heavily */
    SCHED_HINT_MEMORY_INTENSIVE = 2, /* Task uses lots of memory */
    SCHED_HINT_IO_INTENSIVE = 3,     /* Task does I/O operations */
    SCHED_HINT_CPU_INTENSIVE = 4,    /* Task is CPU-bound */
    SCHED_HINT_LATENCY_CRITICAL = 5  /* Task requires low latency */
} sched_hint_t;

/* GPU Scheduling Context */
typedef struct {
    uint32_t task_id;
    uint32_t gpu_device_id;
    uint32_t estimated_gpu_memory;   /* Estimated GPU memory needed */
    uint32_t gpu_utilization_pct;    /* Estimated GPU utilization */
    uint32_t inference_model_id;
    sched_hint_t scheduling_hint;
} gpu_sched_context_t;

/* Memory Pressure Levels */
typedef enum {
    MEM_PRESSURE_NORMAL = 1,
    MEM_PRESSURE_ELEVATED = 2,
    MEM_PRESSURE_HIGH = 3,
    MEM_PRESSURE_CRITICAL = 4
} memory_pressure_t;

/* System Resource State */
typedef struct {
    uint32_t total_memory;
    uint32_t available_memory;
    uint32_t gpu_memory_available;
    memory_pressure_t memory_pressure;
    uint32_t active_gpu_tasks;
    uint32_t active_cpu_tasks;
    uint32_t io_load;               /* 0-100% */
    uint32_t thermal_level;         /* 0-100% */
} system_resources_t;

/* Task Resource Requirements */
typedef struct {
    uint32_t task_id;
    uint32_t cpu_cores_needed;
    uint32_t memory_needed;         /* In bytes */
    uint32_t gpu_memory_needed;     /* In bytes */
    uint32_t bandwidth_needed;      /* MB/s */
    sched_hint_t hint;
    uint32_t priority;
} task_resource_req_t;

/* Resource Scheduler API */
void resource_scheduler_init(void);

/* GPU Scheduling */
int sched_register_gpu_task(uint32_t task_id, gpu_sched_context_t *context);
int sched_unregister_gpu_task(uint32_t task_id);
int sched_get_gpu_context(uint32_t task_id, gpu_sched_context_t *out_context);

/* Resource Queries */
system_resources_t *sched_get_system_resources(void);
memory_pressure_t sched_get_memory_pressure(void);
uint32_t sched_get_available_gpu_memory(uint32_t gpu_device_id);

/* Task Scheduling Hints */
int sched_set_task_hint(uint32_t task_id, sched_hint_t hint);
sched_hint_t sched_get_task_hint(uint32_t task_id);
int sched_set_task_resources(uint32_t task_id, task_resource_req_t *req);

/* GPU-aware Task Selection */
int sched_should_defer_gpu_task(uint32_t task_id);
int sched_can_allocate_gpu_memory(uint32_t gpu_device_id, uint32_t size);
uint32_t sched_get_best_gpu_device(void);

/* Thermal Management */
int sched_get_thermal_level(void);
int sched_thermal_throttle(uint32_t level);
int sched_is_thermal_critical(void);

/* Performance Tuning */
int sched_enable_power_saving(void);
int sched_disable_power_saving(void);
int sched_is_power_saving_enabled(void);

/* Statistics */
typedef struct {
    uint64_t gpu_task_count;
    uint64_t deferred_gpu_tasks;
    uint64_t memory_pressure_events;
    uint64_t thermal_throttle_events;
    uint32_t avg_gpu_utilization;
    uint32_t avg_memory_pressure;
} sched_stats_t;

sched_stats_t *sched_get_stats(void);

#endif /* KERNEL_RESOURCE_SCHEDULER_H */
