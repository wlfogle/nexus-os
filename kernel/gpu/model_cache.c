#include "../../include/kernel/model_cache.h"
#include "../../include/kernel/serial.h"
#include <string.h>

/* Global cache state */
static cache_descriptor_t g_cache = {0};

/* Prefetch request tracking (up to 32 pending) */
typedef struct {
    uint16_t prefetch_id;
    uint16_t model_id;
    uint32_t status;  /* 0=pending, 1=in_progress, 2=complete */
} prefetch_entry_t;

#define CACHE_MAX_PREFETCH 32
static prefetch_entry_t g_prefetch_requests[CACHE_MAX_PREFETCH] = {0};
static uint16_t g_prefetch_count = 0;

/* Initialize cache for local node */
int32_t model_cache_init(uint16_t node_id, uint32_t capacity_mb,
                         cache_coherency_policy_t coherency,
                         cache_eviction_policy_t eviction) {
    memset(&g_cache, 0, sizeof(cache_descriptor_t));
    memset(g_prefetch_requests, 0, sizeof(g_prefetch_requests));

    g_cache.node_id = node_id;
    g_cache.total_capacity_kb = capacity_mb * 1024;
    g_cache.used_capacity_kb = 0;
    g_cache.line_count = 0;
    g_cache.config.coherency = coherency;
    g_cache.config.eviction = eviction;
    g_cache.config.max_lines_per_model = CACHE_LINES_PER_MODEL;
    g_cache.config.prefetch_enabled = 1;
    g_cache.config.broadcast_invalidation = 0;

    const char* coherency_str = "WRITE_THROUGH";
    if (coherency == CACHE_COHERENCY_WRITE_BACK) {
        coherency_str = "WRITE_BACK";
    } else if (coherency == CACHE_COHERENCY_WRITE_INVALIDATE) {
        coherency_str = "WRITE_INVALIDATE";
    }

    const char* eviction_str = "LRU";
    if (eviction == CACHE_EVICTION_LFU) {
        eviction_str = "LFU";
    }

    serial_printf("[model_cache] Initialized cache (node %u, %uMB, coherency: %s, eviction: %s)\n",
                  node_id, capacity_mb, coherency_str, eviction_str);

    return 0;
}

/* Load model into cache */
int32_t model_cache_load_model(uint16_t model_id, uint32_t size_bytes,
                               const void* model_data) {
    if (model_data == NULL || size_bytes == 0) {
        return -1;
    }

    /* Check if already cached */
    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id &&
            g_cache.lines[i].state != CACHE_LINE_INVALID) {
            return -2;  /* Already cached */
        }
    }

    /* Estimate lines needed */
    uint32_t lines_needed = (size_bytes + CACHE_LINE_SIZE_KB * 1024 - 1) /
                            (CACHE_LINE_SIZE_KB * 1024);

    if (lines_needed > CACHE_LINES_PER_MODEL) {
        lines_needed = CACHE_LINES_PER_MODEL;
    }

    uint32_t capacity_needed_kb = lines_needed * CACHE_LINE_SIZE_KB;

    if (g_cache.used_capacity_kb + capacity_needed_kb > g_cache.total_capacity_kb) {
        return -3;  /* Not enough space */
    }

    /* Add cache lines for model */
    for (uint16_t i = 0; i < lines_needed && g_cache.line_count < CACHE_MAX_MODELS * CACHE_LINES_PER_MODEL; i++) {
        cache_line_t* line = &g_cache.lines[g_cache.line_count];
        line->model_id = model_id;
        line->line_index = i;
        line->size_bytes = (i == lines_needed - 1) ?
                          (size_bytes - (i * CACHE_LINE_SIZE_KB * 1024)) :
                          (CACHE_LINE_SIZE_KB * 1024);
        line->state = CACHE_LINE_VALID;
        line->node_id = g_cache.node_id;
        line->replica_count = 1;
        line->version = 1;
        line->access_count = 0;
        line->last_access_ts = 0;

        g_cache.line_count++;
    }

    g_cache.used_capacity_kb += capacity_needed_kb;
    g_cache.stats.hits++;

    serial_printf("[model_cache] Loaded model %u (%u bytes, %u lines, %uKB used)\n",
                  model_id, size_bytes, lines_needed, g_cache.used_capacity_kb);

    return 0;
}

/* Unload model from cache */
int32_t model_cache_unload_model(uint16_t model_id) {
    uint32_t lines_removed = 0;
    uint32_t capacity_freed = 0;

    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id) {
            capacity_freed += g_cache.lines[i].size_bytes / 1024;

            /* Remove by shifting */
            for (uint16_t j = i; j < g_cache.line_count - 1; j++) {
                g_cache.lines[j] = g_cache.lines[j + 1];
            }
            g_cache.line_count--;
            lines_removed++;
            i--;  /* Re-check this position */
        }
    }

    g_cache.used_capacity_kb -= capacity_freed;
    g_cache.stats.evictions += lines_removed;

    if (lines_removed > 0) {
        serial_printf("[model_cache] Unloaded model %u (%u lines, freed %uKB)\n",
                      model_id, lines_removed, capacity_freed);
    }

    return (lines_removed > 0) ? 0 : -1;
}

/* Get model from cache */
const void* model_cache_get(uint16_t model_id) {
    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id &&
            g_cache.lines[i].state == CACHE_LINE_VALID) {
            g_cache.lines[i].access_count++;
            g_cache.stats.hits++;
            return (const void*)(uintptr_t)(model_id | 0x80000000);  /* Dummy pointer */
        }
    }

    g_cache.stats.misses++;
    return NULL;
}

/* Check if model is cached */
int32_t model_cache_is_cached(uint16_t model_id) {
    return (model_cache_get(model_id) != NULL) ? 1 : 0;
}

/* Get cache line */
int32_t model_cache_get_line(uint16_t model_id, uint16_t line_index,
                             void* buffer, uint32_t buffer_size) {
    if (buffer == NULL || buffer_size == 0) {
        return -1;
    }

    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id &&
            g_cache.lines[i].line_index == line_index) {

            if (g_cache.lines[i].size_bytes > buffer_size) {
                return -2;  /* Buffer too small */
            }

            if (g_cache.lines[i].state == CACHE_LINE_INVALID) {
                return -3;  /* Line not available */
            }

            g_cache.lines[i].access_count++;
            g_cache.lines[i].last_access_ts = 0;
            g_cache.stats.hits++;
            g_cache.stats.total_bytes_served += g_cache.lines[i].size_bytes;

            return g_cache.lines[i].size_bytes;
        }
    }

    g_cache.stats.misses++;
    return -4;  /* Line not found */
}

/* Write cache line */
int32_t model_cache_write_line(uint16_t model_id, uint16_t line_index,
                               const void* data, uint32_t size) {
    if (data == NULL || size == 0) {
        return -1;
    }

    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id &&
            g_cache.lines[i].line_index == line_index) {

            if (size > CACHE_LINE_SIZE_KB * 1024) {
                return -2;  /* Data too large */
            }

            g_cache.lines[i].state = CACHE_LINE_MODIFIED;
            g_cache.lines[i].version++;
            g_cache.lines[i].access_count++;
            g_cache.stats.coherency_messages++;

            return 0;
        }
    }

    return -3;  /* Line not found */
}

/* Invalidate cache line */
int32_t model_cache_invalidate_line(uint16_t model_id, uint16_t line_index) {
    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id &&
            g_cache.lines[i].line_index == line_index) {

            g_cache.lines[i].state = CACHE_LINE_INVALID;
            g_cache.stats.invalidations++;

            return 0;
        }
    }

    return -1;
}

/* Process invalidation message */
int32_t model_cache_process_invalidation(const cache_invalidation_msg_t* msg) {
    if (msg == NULL) {
        return -1;
    }

    uint32_t invalidated = 0;

    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == msg->model_id &&
            g_cache.lines[i].version < msg->version) {

            g_cache.lines[i].state = CACHE_LINE_INVALID;
            g_cache.lines[i].version = msg->version;
            invalidated++;
            g_cache.stats.invalidations++;
        }
    }

    g_cache.stats.coherency_messages++;

    return invalidated;
}

/* Replicate cache line */
int32_t model_cache_replicate_line(uint16_t model_id, uint16_t line_index,
                                   uint16_t target_node) {
    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id &&
            g_cache.lines[i].line_index == line_index) {

            g_cache.lines[i].replica_count++;
            g_cache.stats.coherency_messages++;

            serial_printf("[model_cache] Replicated line (model %u, line %u) to node %u\n",
                          model_id, line_index, target_node);

            return 0;
        }
    }

    return -1;
}

/* Sync cache line */
int32_t model_cache_sync_line(uint16_t model_id, uint16_t line_index) {
    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id &&
            g_cache.lines[i].line_index == line_index) {

            if (g_cache.lines[i].state == CACHE_LINE_MODIFIED) {
                g_cache.lines[i].state = CACHE_LINE_VALID;
            }

            g_cache.stats.coherency_messages++;

            return 0;
        }
    }

    return -1;
}

/* Update version */
int32_t model_cache_update_version(uint16_t model_id, uint32_t new_version) {
    uint32_t updated = 0;

    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id) {
            g_cache.lines[i].version = new_version;
            updated++;
        }
    }

    return (updated > 0) ? 0 : -1;
}

/* Evict cache line */
int32_t model_cache_evict_line(uint16_t model_id, uint16_t line_index) {
    uint32_t capacity_freed = 0;

    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id == model_id &&
            g_cache.lines[i].line_index == line_index) {

            capacity_freed = (g_cache.lines[i].size_bytes + 1023) / 1024;

            for (uint16_t j = i; j < g_cache.line_count - 1; j++) {
                g_cache.lines[j] = g_cache.lines[j + 1];
            }

            g_cache.line_count--;
            g_cache.used_capacity_kb -= capacity_freed;
            g_cache.stats.evictions++;

            return 0;
        }
    }

    return -1;
}

/* Prefetch */
int32_t model_cache_prefetch(uint16_t prefetch_id, uint16_t model_id,
                             uint32_t priority, uint32_t deadline_ms) {
    (void)deadline_ms;
    if (g_prefetch_count >= CACHE_MAX_PREFETCH) {
        return -1;
    }

    prefetch_entry_t* req = &g_prefetch_requests[g_prefetch_count];
    req->prefetch_id = prefetch_id;
    req->model_id = model_id;
    req->status = 1;  /* In progress */

    g_prefetch_count++;

    serial_printf("[model_cache] Prefetch requested (id: %u, model: %u, priority: %u)\n",
                  prefetch_id, model_id, priority);

    return 0;
}

/* Prefetch status */
int32_t model_cache_prefetch_status(uint16_t prefetch_id) {
    for (uint16_t i = 0; i < g_prefetch_count; i++) {
        if (g_prefetch_requests[i].prefetch_id == prefetch_id) {
            return g_prefetch_requests[i].status;
        }
    }

    return -1;
}

/* Get pending prefetch count */
uint32_t model_cache_get_pending_prefetch_count(void) {
    uint32_t pending = 0;

    for (uint16_t i = 0; i < g_prefetch_count; i++) {
        if (g_prefetch_requests[i].status < 2) {
            pending++;
        }
    }

    return pending;
}

/* Get stats */
const cache_stats_t* model_cache_get_stats(void) {
    return &g_cache.stats;
}

/* Get utilization */
uint32_t model_cache_get_utilization(void) {
    if (g_cache.total_capacity_kb == 0) {
        return 0;
    }

    return (g_cache.used_capacity_kb * 100) / g_cache.total_capacity_kb;
}

/* Get cached model count */
uint16_t model_cache_get_cached_model_count(void) {
    uint16_t count = 0;
    uint16_t last_model = 0xFFFF;

    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].model_id != last_model &&
            g_cache.lines[i].state != CACHE_LINE_INVALID) {
            count++;
            last_model = g_cache.lines[i].model_id;
        }
    }

    return count;
}

/* Get hit rate */
uint32_t model_cache_get_hit_rate(void) {
    uint32_t total = g_cache.stats.hits + g_cache.stats.misses;

    if (total == 0) {
        return 0;
    }

    return (g_cache.stats.hits * 100) / total;
}

/* Reset stats */
int32_t model_cache_reset_stats(void) {
    memset(&g_cache.stats, 0, sizeof(cache_stats_t));

    return 0;
}

/* Print status */
void model_cache_print_status(void) {
    serial_printf("\n=== Model Cache Status ===\n");
    serial_printf("Node ID: %u\n", g_cache.node_id);
    serial_printf("Capacity: %uKB (%uMB)\n", g_cache.total_capacity_kb,
                  g_cache.total_capacity_kb / 1024);
    serial_printf("Used: %uKB (%.1f%%)\n", g_cache.used_capacity_kb,
                  (float)model_cache_get_utilization());
    serial_printf("Cache Lines: %u\n", g_cache.line_count);
    serial_printf("Cached Models: %u\n", model_cache_get_cached_model_count());
    serial_printf("Hit Rate: %u%%\n", model_cache_get_hit_rate());
    serial_printf("Hits: %u, Misses: %u\n", g_cache.stats.hits, g_cache.stats.misses);
    serial_printf("Evictions: %u, Invalidations: %u\n", g_cache.stats.evictions,
                  g_cache.stats.invalidations);
    serial_printf("Bytes Served: %u\n", g_cache.stats.total_bytes_served);
    serial_printf("Coherency Messages: %u\n", g_cache.stats.coherency_messages);
    serial_printf("Pending Prefetch: %u\n", model_cache_get_pending_prefetch_count());

    const char* coherency_str = "WRITE_THROUGH";
    if (g_cache.config.coherency == CACHE_COHERENCY_WRITE_BACK) {
        coherency_str = "WRITE_BACK";
    } else if (g_cache.config.coherency == CACHE_COHERENCY_WRITE_INVALIDATE) {
        coherency_str = "WRITE_INVALIDATE";
    }

    serial_printf("Coherency: %s, Eviction: %s\n", coherency_str,
                  g_cache.config.eviction == CACHE_EVICTION_LFU ? "LFU" : "LRU");
    serial_printf("===========================\n\n");
}

/* Get config */
const cache_config_t* model_cache_get_config(void) {
    return &g_cache.config;
}

/* Set coherency */
int32_t model_cache_set_coherency(cache_coherency_policy_t policy) {
    g_cache.config.coherency = policy;

    return 0;
}

/* Set eviction */
int32_t model_cache_set_eviction(cache_eviction_policy_t policy) {
    g_cache.config.eviction = policy;

    return 0;
}

/* Set prefetch enabled */
int32_t model_cache_set_prefetch_enabled(uint32_t enabled) {
    g_cache.config.prefetch_enabled = enabled ? 1 : 0;

    return 0;
}

/* Set broadcast invalidation */
int32_t model_cache_set_broadcast_invalidation(uint32_t enabled) {
    g_cache.config.broadcast_invalidation = enabled ? 1 : 0;

    return 0;
}

/* Validate consistency */
int32_t model_cache_validate_consistency(void) {
    uint32_t valid_count = 0;
    uint32_t invalid_count = 0;

    for (uint16_t i = 0; i < g_cache.line_count; i++) {
        if (g_cache.lines[i].state == CACHE_LINE_VALID) {
            valid_count++;
        } else if (g_cache.lines[i].state == CACHE_LINE_INVALID) {
            invalid_count++;
        }
    }

    serial_printf("[model_cache] Consistency check: %u valid, %u invalid\n",
                  valid_count, invalid_count);

    return 0;
}

/* Dump contents */
void model_cache_dump_contents(void) {
    serial_printf("\n=== Cache Contents ===\n");

    for (uint16_t i = 0; i < g_cache.line_count && i < 16; i++) {
        const char* state_str = "INVALID";
        if (g_cache.lines[i].state == CACHE_LINE_VALID) {
            state_str = "VALID";
        } else if (g_cache.lines[i].state == CACHE_LINE_MODIFIED) {
            state_str = "MODIFIED";
        } else if (g_cache.lines[i].state == CACHE_LINE_PENDING) {
            state_str = "PENDING";
        }

        serial_printf("  Line %u: model %u, line %u, state %s, size %uKB, v%u, refs %u\n",
                      i, g_cache.lines[i].model_id, g_cache.lines[i].line_index,
                      state_str, (g_cache.lines[i].size_bytes + 1023) / 1024,
                      g_cache.lines[i].version, g_cache.lines[i].replica_count);
    }

    serial_printf("======================\n\n");
}
