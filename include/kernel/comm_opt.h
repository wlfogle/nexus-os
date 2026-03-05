#ifndef KERNEL_COMM_OPT_H
#define KERNEL_COMM_OPT_H

#include <stdint.h>

/* Communication topology types */
typedef enum {
    TOPO_TREE = 0,          /* Tree-based AllReduce */
    TOPO_RING = 1,          /* Ring AllReduce (bandwidth optimal) */
    TOPO_BUTTERFLY = 2,     /* Butterfly AllReduce (latency optimal) */
    TOPO_MESH = 3           /* Mesh AllReduce (hybrid) */
} comm_topology_t;

/* Communication optimization modes */
typedef enum {
    OPT_MODE_DEFAULT = 0,
    OPT_MODE_BANDWIDTH = 1,  /* Maximize bandwidth utilization */
    OPT_MODE_LATENCY = 2,    /* Minimize latency */
    OPT_MODE_POWER = 3       /* Minimize power consumption */
} opt_mode_t;

/* GPU link bandwidth information */
typedef struct {
    uint32_t src_gpu;
    uint32_t dst_gpu;
    uint32_t bandwidth_gbps;  /* Gigabits per second */
    uint32_t latency_ns;      /* Nanoseconds */
} gpu_link_t;

/* Ring AllReduce phase */
typedef struct {
    uint32_t phase_id;
    uint32_t src_gpu;
    uint32_t dst_gpu;
    uint32_t send_offset;
    uint32_t recv_offset;
    uint32_t chunk_size;
    uint8_t compute_overlap;  /* Can overlap with compute */
} ring_phase_t;

/* Communication stats */
typedef struct {
    uint32_t total_messages;
    uint32_t total_bytes;
    uint32_t avg_latency_us;
    uint32_t max_latency_us;
    uint32_t bandwidth_utilization_pct;
    uint32_t compute_communication_overlap_pct;
} comm_stats_t;

/* Core Communication Optimization APIs */

/**
 * Initialize communication optimization
 */
void comm_opt_init(void);

/**
 * Set communication topology
 */
int comm_opt_set_topology(comm_topology_t topo);

/**
 * Set optimization mode
 */
int comm_opt_set_mode(opt_mode_t mode);

/**
 * Discover GPU topology and bandwidth
 */
int comm_opt_discover_topology(void);

/**
 * Get bandwidth between two GPUs
 */
uint32_t comm_opt_get_bandwidth(uint32_t src_gpu, uint32_t dst_gpu);

/**
 * Perform Ring AllReduce
 */
int comm_opt_ring_allreduce(uint32_t *gpu_ids, uint32_t num_gpus,
                           const float *input_data, uint32_t data_size,
                           float *output_data);

/**
 * Get Ring AllReduce phase info
 */
int comm_opt_get_ring_phase(uint32_t phase_id, ring_phase_t *out);

/**
 * Enable computation/communication overlap
 */
int comm_opt_enable_overlap(uint32_t gpu_id);

/**
 * Disable computation/communication overlap
 */
int comm_opt_disable_overlap(uint32_t gpu_id);

/**
 * Get current overlap status
 */
int comm_opt_get_overlap_status(uint32_t gpu_id);

/**
 * Optimize communication for given GPU set
 */
int comm_opt_optimize_for_gpus(uint32_t *gpu_ids, uint32_t num_gpus);

/**
 * Schedule non-blocking communication
 */
uint32_t comm_opt_schedule_async_send(uint32_t src_gpu, uint32_t dst_gpu,
                                     const float *data, uint32_t size);

/**
 * Check async communication completion
 */
int comm_opt_check_async_complete(uint32_t comm_id);

/**
 * Wait for async communication
 */
int comm_opt_wait_async(uint32_t comm_id);

/**
 * Get communication statistics
 */
comm_stats_t *comm_opt_get_stats(void);

/**
 * Estimate AllReduce latency
 */
uint32_t comm_opt_estimate_latency(uint32_t num_gpus, uint32_t data_size);

/**
 * Get estimated bandwidth efficiency
 */
uint32_t comm_opt_get_efficiency_pct(void);

/**
 * Rebalance communication load
 */
int comm_opt_rebalance(void);

#endif /* KERNEL_COMM_OPT_H */
