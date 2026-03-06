#ifndef KERNEL_MODEL_CACHE_H
#define KERNEL_MODEL_CACHE_H

#include <stdint.h>
#include <stddef.h>

/* Maximum number of cached models */
#define CACHE_MAX_MODELS 256

/* Maximum number of cache lines per model */
#define CACHE_LINES_PER_MODEL 16

/* Cache line size (64KB) */
#define CACHE_LINE_SIZE_KB 64

/* Coherency protocols */
typedef enum {
    CACHE_COHERENCY_WRITE_THROUGH = 0,   /* Write immediately to all replicas */
    CACHE_COHERENCY_WRITE_BACK = 1,      /* Write to local, invalidate others */
    CACHE_COHERENCY_WRITE_INVALIDATE = 2, /* Write invalidates all other copies */
} cache_coherency_policy_t;

/* Eviction policies */
typedef enum {
    CACHE_EVICTION_LRU = 0,  /* Least Recently Used */
    CACHE_EVICTION_LFU = 1,  /* Least Frequently Used */
} cache_eviction_policy_t;

/* Cache line state */
typedef enum {
    CACHE_LINE_INVALID = 0,   /* Not loaded */
    CACHE_LINE_VALID = 1,     /* Up-to-date copy */
    CACHE_LINE_MODIFIED = 2,  /* Modified locally */
    CACHE_LINE_PENDING = 3,   /* Invalidation in progress */
} cache_line_state_t;

/* Cache line descriptor */
typedef struct {
    uint16_t model_id;                  /* Model ID */
    uint16_t line_index;                /* Line within model (0-15) */
    uint32_t start_layer;               /* Starting layer index in this line */
    uint32_t end_layer;                 /* Ending layer index in this line */
    uint32_t size_bytes;                /* Actual data size (up to 64KB) */
    cache_line_state_t state;           /* Current cache line state */
    uint16_t node_id;                   /* Node holding this line */
    uint16_t replica_count;             /* Number of replicas */
    uint32_t version;                   /* Version for coherency tracking */
    uint32_t access_count;              /* For LFU eviction */
    uint32_t last_access_ts;            /* For LRU eviction */
} cache_line_t;

/* Cache invalidation message */
typedef struct {
    uint16_t model_id;
    uint32_t version;                   /* New version */
    uint16_t target_node_count;
    uint16_t target_nodes[16];          /* Nodes to invalidate (broadcast if 0) */
    uint32_t timestamp;
} cache_invalidation_msg_t;

/* Prefetch request */
typedef struct {
    uint16_t prefetch_id;
    uint16_t model_id;
    uint32_t priority;                  /* Higher = prefetch sooner */
    uint32_t deadline_ms;
    uint16_t source_node;               /* Preferred source node */
    uint16_t target_nodes[16];          /* Nodes to prefetch to */
    uint32_t size_bytes;
} cache_prefetch_request_t;

/* Cache statistics per node */
typedef struct {
    uint32_t hits;                      /* Cache hit count */
    uint32_t misses;                    /* Cache miss count */
    uint32_t evictions;                 /* Lines evicted */
    uint32_t invalidations;             /* Lines invalidated */
    uint32_t total_bytes_served;        /* Total bytes from cache */
    uint32_t coherency_messages;        /* Coherency protocol messages sent */
} cache_stats_t;

/* Cache configuration */
typedef struct {
    cache_coherency_policy_t coherency;
    cache_eviction_policy_t eviction;
    uint32_t max_lines_per_model;
    uint32_t prefetch_enabled;
    uint32_t broadcast_invalidation;    /* 1=broadcast, 0=directed */
} cache_config_t;

/* Cache descriptor */
typedef struct {
    uint16_t node_id;
    uint32_t total_capacity_kb;         /* Max 16MB per node: 256 lines * 64KB */
    uint32_t used_capacity_kb;
    cache_config_t config;
    cache_line_t lines[CACHE_MAX_MODELS * CACHE_LINES_PER_MODEL];
    uint16_t line_count;
    cache_stats_t stats;
} cache_descriptor_t;

/* Cache management API */

/* Initialize cache for local node */
int32_t model_cache_init(uint16_t node_id, uint32_t capacity_mb,
                         cache_coherency_policy_t coherency,
                         cache_eviction_policy_t eviction);

/* Load model into cache */
int32_t model_cache_load_model(uint16_t model_id, uint32_t size_bytes,
                               const void* model_data);

/* Unload model from cache */
int32_t model_cache_unload_model(uint16_t model_id);

/* Get model from cache (or NULL if not cached) */
const void* model_cache_get(uint16_t model_id);

/* Check if model is cached locally */
int32_t model_cache_is_cached(uint16_t model_id);

/* Cache line management */

/* Get cache line (load from replica if needed) */
int32_t model_cache_get_line(uint16_t model_id, uint16_t line_index,
                             void* buffer, uint32_t buffer_size);

/* Write cache line (triggers coherency protocol) */
int32_t model_cache_write_line(uint16_t model_id, uint16_t line_index,
                               const void* data, uint32_t size);

/* Invalidate cache line locally */
int32_t model_cache_invalidate_line(uint16_t model_id, uint16_t line_index);

/* Coherency and replication */

/* Process invalidation message from remote node */
int32_t model_cache_process_invalidation(const cache_invalidation_msg_t* msg);

/* Replicate cache line to remote node */
int32_t model_cache_replicate_line(uint16_t model_id, uint16_t line_index,
                                   uint16_t target_node);

/* Synchronize cache line with replicas (for write-back policy) */
int32_t model_cache_sync_line(uint16_t model_id, uint16_t line_index);

/* Update cache line version for coherency */
int32_t model_cache_update_version(uint16_t model_id, uint32_t new_version);

/* Eviction and prefetching */

/* Manual eviction of cache line */
int32_t model_cache_evict_line(uint16_t model_id, uint16_t line_index);

/* Prefetch model to cache */
int32_t model_cache_prefetch(uint16_t prefetch_id, uint16_t model_id,
                             uint32_t priority, uint32_t deadline_ms);

/* Check prefetch completion */
int32_t model_cache_prefetch_status(uint16_t prefetch_id);

/* List prefetch requests pending */
uint32_t model_cache_get_pending_prefetch_count(void);

/* Statistics and diagnostics */

/* Get cache statistics */
const cache_stats_t* model_cache_get_stats(void);

/* Get cache utilization percentage */
uint32_t model_cache_get_utilization(void);

/* Get number of cached models */
uint16_t model_cache_get_cached_model_count(void);

/* Get total cache hit rate percentage */
uint32_t model_cache_get_hit_rate(void);

/* Reset cache statistics */
int32_t model_cache_reset_stats(void);

/* Print cache status */
void model_cache_print_status(void);

/* Configuration */

/* Get current cache configuration */
const cache_config_t* model_cache_get_config(void);

/* Update coherency policy */
int32_t model_cache_set_coherency(cache_coherency_policy_t policy);

/* Update eviction policy */
int32_t model_cache_set_eviction(cache_eviction_policy_t policy);

/* Enable/disable prefetching */
int32_t model_cache_set_prefetch_enabled(uint32_t enabled);

/* Set broadcast invalidation mode */
int32_t model_cache_set_broadcast_invalidation(uint32_t enabled);

/* Debugging */

/* Validate cache consistency (check version numbers) */
int32_t model_cache_validate_consistency(void);

/* Dump cache content for inspection */
void model_cache_dump_contents(void);

#endif /* KERNEL_MODEL_CACHE_H */
