#ifndef KERNEL_GPU_CACHE_H
#define KERNEL_GPU_CACHE_H

#include <stdint.h>

/* Cache eviction policy */
typedef enum {
    CACHE_POLICY_LRU = 0,       /* Least Recently Used */
    CACHE_POLICY_LFU = 1,       /* Least Frequently Used */
    CACHE_POLICY_FIFO = 2       /* First In First Out */
} cache_policy_t;

/* Cache entry types */
typedef enum {
    CACHE_TYPE_TENSOR = 0,
    CACHE_TYPE_INFERENCE_RESULT = 1,
    CACHE_TYPE_MODEL_WEIGHT = 2,
    CACHE_TYPE_ACTIVATION = 3
} cache_entry_type_t;

/* GPU cache entry metadata */
typedef struct {
    uint32_t cache_id;
    uint32_t gpu_id;
    uint32_t source_id;         /* Model ID, tensor ID, etc */
    cache_entry_type_t type;
    uint32_t size_bytes;
    uint32_t access_count;
    uint64_t last_access_time;
    uint64_t created_time;
    uint32_t hit_count;
    uint8_t valid;
} cache_entry_t;

/* Cache statistics */
typedef struct {
    uint32_t total_entries;
    uint32_t total_size;        /* bytes */
    uint32_t available_size;    /* bytes */
    uint32_t hit_count;
    uint32_t miss_count;
    uint32_t eviction_count;
    uint32_t avg_access_time_us;
} cache_stats_t;

/* GPU pipeline request */
typedef struct {
    uint32_t request_id;
    uint32_t model_id;
    uint32_t batch_size;
    uint32_t priority;
    uint8_t use_cache;
    uint8_t prefetch_enabled;
    uint32_t estimated_latency_us;
    uint32_t actual_latency_us;
    uint32_t submitted_time;
    uint32_t completed_time;
} pipeline_request_t;

/* Pipeline statistics */
typedef struct {
    uint32_t total_requests;
    uint32_t completed_requests;
    uint32_t pending_requests;
    uint32_t avg_latency_us;
    uint32_t max_throughput_reqs_sec;
    uint32_t batches_processed;
    uint32_t avg_batch_size;
} pipeline_stats_t;

/* Core GPU Cache APIs */

/**
 * Initialize GPU cache subsystem
 */
void gpu_cache_init(void);

/**
 * Set cache eviction policy for GPU
 */
int gpu_cache_set_policy(uint32_t gpu_id, cache_policy_t policy);

/**
 * Allocate cache entry on GPU
 */
uint32_t gpu_cache_allocate(uint32_t gpu_id, uint32_t source_id, 
                            cache_entry_type_t type, uint32_t size);

/**
 * Free cache entry
 */
int gpu_cache_free(uint32_t cache_id);

/**
 * Get cache entry
 */
int gpu_cache_get(uint32_t cache_id, cache_entry_t *out);

/**
 * Check cache hit
 */
int gpu_cache_hit(uint32_t cache_id);

/**
 * Invalidate cache entry
 */
int gpu_cache_invalidate(uint32_t cache_id);

/**
 * Clear all cache entries for a source
 */
int gpu_cache_clear_source(uint32_t gpu_id, uint32_t source_id);

/**
 * Get cache statistics
 */
int gpu_cache_get_stats(uint32_t gpu_id, cache_stats_t *out);

/**
 * Prefetch data into cache
 */
int gpu_cache_prefetch(uint32_t gpu_id, uint32_t source_id, 
                       cache_entry_type_t type, uint32_t size);

/**
 * Evict least valuable entry (based on policy)
 */
uint32_t gpu_cache_evict_entry(uint32_t gpu_id);

/**
 * Set cache memory limit for GPU
 */
int gpu_cache_set_limit(uint32_t gpu_id, uint32_t limit_bytes);

/**
 * Get current cache memory usage
 */
uint32_t gpu_cache_get_usage(uint32_t gpu_id);

/* Core GPU Pipeline APIs */

/**
 * Initialize inference pipeline
 */
void gpu_pipeline_init(void);

/**
 * Submit request to pipeline
 */
uint32_t gpu_pipeline_submit(uint32_t model_id, uint32_t batch_size, uint32_t priority);

/**
 * Get request status
 */
int gpu_pipeline_get_request(uint32_t request_id, pipeline_request_t *out);

/**
 * Enable batching for model
 */
int gpu_pipeline_enable_batching(uint32_t model_id, uint16_t batch_size);

/**
 * Enable prefetching for model
 */
int gpu_pipeline_enable_prefetch(uint32_t model_id);

/**
 * Enable pipelining mode (concurrent requests)
 */
int gpu_pipeline_enable_pipelining(uint32_t model_id);

/**
 * Process next batch in pipeline
 */
uint32_t gpu_pipeline_process_batch(uint32_t gpu_id);

/**
 * Get pipeline statistics
 */
pipeline_stats_t *gpu_pipeline_get_stats(uint32_t gpu_id);

/**
 * Estimate latency for request
 */
uint32_t gpu_pipeline_estimate_latency(uint32_t model_id, uint32_t batch_size);

/**
 * Complete inference request
 */
int gpu_pipeline_complete_request(uint32_t request_id);

/**
 * Flush pending requests in pipeline
 */
int gpu_pipeline_flush(uint32_t gpu_id);

#endif /* KERNEL_GPU_CACHE_H */
