#ifndef KERNEL_DIST_INFERENCE_H
#define KERNEL_DIST_INFERENCE_H

#include <stdint.h>

/* Load balancing strategy */
typedef enum {
    LB_ROUND_ROBIN = 0,
    LB_LEAST_LOADED,
    LB_PERFORMANCE_AWARE,
    LB_POWER_AWARE
} load_balance_strategy_t;

/* GPU device status for distributed scheduling */
typedef struct {
    uint32_t gpu_id;
    uint8_t available;
    uint32_t active_inferences;
    uint32_t queue_depth;
    uint32_t memory_utilization_pct;
    uint32_t power_draw_mw;
    uint32_t temperature_c;
    uint32_t inference_throughput;  /* inferences per second */
} gpu_device_status_t;

/* Model replica assignment to GPU */
typedef struct {
    uint32_t model_id;
    uint32_t gpu_id;
    uint32_t replica_index;
    uint32_t inference_count;
    uint32_t avg_latency_us;
    uint8_t ready;
} model_replica_t;

/* Distributed inference request */
typedef struct {
    uint32_t request_id;
    uint32_t model_id;
    uint32_t preferred_gpu;  /* -1 for auto-select */
    uint32_t assigned_gpu;
    uint32_t batch_size;
    uint32_t input_size;
    uint32_t priority;       /* 0-255 */
    uint64_t created_time;
    uint64_t completed_time;
} dist_infer_req_t;

/* Distributed scheduler state */
typedef struct {
    uint32_t total_requests;
    uint32_t completed_requests;
    uint32_t failed_requests;
    uint32_t pending_requests;
    uint32_t total_model_replicas;
    uint32_t active_replicas;
    load_balance_strategy_t strategy;
    uint32_t avg_latency_us;
} dist_sched_stats_t;

/* Load balancing configuration */
typedef struct {
    load_balance_strategy_t strategy;
    uint8_t enable_replication;
    uint8_t enable_prefetch;
    uint8_t enable_batching;
    uint16_t max_batch_size;
    uint32_t load_check_interval_ms;
    uint16_t rebalance_threshold_pct;  /* Trigger rebalance at this utilization diff */
} dist_sched_config_t;

/* Core Distributed Inference APIs */

/**
 * Initialize distributed inference scheduler
 */
void dist_inference_init(void);

/**
 * Set load balancing strategy
 */
int dist_set_load_balance_strategy(load_balance_strategy_t strategy);

/**
 * Get current GPU device status
 */
int dist_get_gpu_status(uint32_t gpu_id, gpu_device_status_t *out);

/**
 * Get status of all GPU devices
 */
uint32_t dist_get_all_gpu_status(gpu_device_status_t *devices, uint32_t max_devices);

/**
 * Submit a distributed inference request
 */
uint32_t dist_submit_inference(uint32_t model_id, uint32_t batch_size, uint32_t priority);

/**
 * Get inference request status
 */
int dist_get_request_status(uint32_t request_id, dist_infer_req_t *out);

/**
 * Assign GPU for an inference request (based on current strategy)
 */
uint32_t dist_assign_gpu(uint32_t request_id, uint32_t model_id);

/**
 * Create model replica on specific GPU
 */
uint32_t dist_replicate_model(uint32_t model_id, uint32_t gpu_id);

/**
 * Get model replica info
 */
int dist_get_model_replica(uint32_t model_id, uint32_t gpu_id, model_replica_t *out);

/**
 * Get all replicas of a model
 */
uint32_t dist_list_model_replicas(uint32_t model_id, model_replica_t *replicas, uint32_t max);

/**
 * Rebalance load across GPUs (redistribute requests)
 */
int dist_rebalance_load(void);

/**
 * Enable model prefetching on specific GPU
 */
int dist_enable_prefetch(uint32_t model_id, uint32_t gpu_id);

/**
 * Enable batching for model inference
 */
int dist_enable_batching(uint32_t model_id, uint16_t batch_size);

/**
 * Get distributed scheduler statistics
 */
dist_sched_stats_t *dist_get_stats(void);

/**
 * Get current load balance configuration
 */
int dist_get_config(dist_sched_config_t *out);

/**
 * Update load balance configuration
 */
int dist_set_config(dist_sched_config_t *config);

/**
 * Estimate GPU selection for upcoming inference
 */
uint32_t dist_estimate_best_gpu(uint32_t model_id, uint32_t batch_size);

/**
 * Get average inference latency per GPU
 */
uint32_t dist_get_avg_latency(uint32_t gpu_id);

/**
 * Clear old completed requests from tracker
 */
int dist_cleanup_completed_requests(uint32_t older_than_ms);

#endif /* KERNEL_DIST_INFERENCE_H */
