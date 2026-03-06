#ifndef KERNEL_CONTAINER_H
#define KERNEL_CONTAINER_H

#include <stdint.h>

/* Container state enumeration */
typedef enum {
    CONTAINER_CREATED = 0,
    CONTAINER_RUNNING,
    CONTAINER_PAUSED,
    CONTAINER_STOPPED,
    CONTAINER_DESTROYED
} container_state_t;

/* Container resource limits */
typedef struct {
    uint32_t max_memory;       /* bytes */
    uint32_t max_gpu_memory;   /* bytes */
    uint32_t max_cpu_shares;
    uint16_t max_threads;
    uint16_t max_gpus;
} container_limits_t;

/* Container resource constraints */
typedef struct {
    uint32_t memory_used;      /* Current memory usage */
    uint32_t gpu_memory_used;  /* Current GPU memory usage */
    uint16_t active_threads;
    uint16_t gpus_assigned;
    uint32_t inference_quota;  /* Inferences allowed per period */
} container_resources_t;

/* Container metadata and configuration */
typedef struct {
    uint32_t container_id;
    char name[64];
    container_state_t state;
    uint32_t process_id;
    uint32_t created_time;
    uint32_t last_modified;
    uint32_t memory_limit;     /* bytes */
    uint16_t priority;         /* 0-255 */
    uint8_t allow_gpu;
    uint8_t allow_networking;
} container_t;

/* Container statistics for observability */
typedef struct {
    uint32_t container_id;
    uint64_t memory_used;
    uint64_t gpu_memory_used;
    uint32_t cpu_time_ms;
    uint32_t active_threads;
    uint32_t inferences_executed;
    uint32_t task_switches;
    uint32_t page_faults;
} container_stats_t;

/* Container I/O namespace (filesystem isolation) */
typedef struct {
    uint32_t container_id;
    uint32_t root_inode;
    uint32_t cwd_inode;
    uint8_t read_only;
} container_namespace_t;

/* Container network namespace */
typedef struct {
    uint32_t container_id;
    uint32_t ipv4_address;
    uint32_t gateway;
    uint16_t port_mappings[16];
    uint8_t network_isolated;
} container_netns_t;

/* Core Container Runtime APIs */

/**
 * Initialize container runtime subsystem
 */
void container_runtime_init(void);

/**
 * Create a new container with specified limits
 */
uint32_t container_create(const char *name, container_limits_t *limits);

/**
 * Start a stopped container (or resume paused)
 */
int container_start(uint32_t container_id);

/**
 * Stop a running container gracefully
 */
int container_stop(uint32_t container_id);

/**
 * Pause a running container (freeze execution)
 */
int container_pause(uint32_t container_id);

/**
 * Resume a paused container
 */
int container_resume(uint32_t container_id);

/**
 * Destroy a container (cleanup resources)
 */
int container_destroy(uint32_t container_id);

/**
 * Get container metadata
 */
int container_get(uint32_t container_id, container_t *out);

/**
 * Get real-time container statistics
 */
int container_get_stats(uint32_t container_id, container_stats_t *out);

/**
 * Assign GPU to container
 */
int container_assign_gpu(uint32_t container_id, uint32_t gpu_device_id);

/**
 * Release GPU from container
 */
int container_release_gpu(uint32_t container_id, uint32_t gpu_device_id);

/**
 * Check if container can allocate memory
 */
int container_can_allocate_memory(uint32_t container_id, uint32_t size);

/**
 * Set container priority (higher = more scheduling priority)
 */
int container_set_priority(uint32_t container_id, uint16_t priority);

/**
 * List all containers (returns count)
 */
uint32_t container_list(container_t *containers, uint32_t max_containers);

/**
 * Get container count
 */
uint32_t container_get_count(void);

/**
 * Get total container memory usage
 */
uint64_t container_get_total_memory(void);

/**
 * Get total GPU memory usage across all containers
 */
uint64_t container_get_total_gpu_memory(void);

/**
 * Container resource management APIs */

/**
 * Update memory limit for a container
 */
int container_set_memory_limit(uint32_t container_id, uint32_t limit);

/**
 * Update GPU memory limit
 */
int container_set_gpu_memory_limit(uint32_t container_id, uint32_t limit);

/**
 * Get current resource usage
 */
int container_get_resources(uint32_t container_id, container_resources_t *out);

/**
 * Enable/disable GPU access for container
 */
int container_set_gpu_access(uint32_t container_id, int enabled);

/**
 * Enable/disable networking for container
 */
int container_set_network_access(uint32_t container_id, int enabled);

/**
 * Get container state
 */
container_state_t container_get_state(uint32_t container_id);

#endif /* KERNEL_CONTAINER_H */
