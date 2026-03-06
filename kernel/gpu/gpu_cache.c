#include "../../include/kernel/gpu_cache.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_CACHE_ENTRIES 256
#define MAX_PIPELINE_REQUESTS 128
#define MAX_GPUS 8
#define DEFAULT_CACHE_SIZE (64 * 1024 * 1024)  /* 64MB per GPU */

typedef struct {
    cache_entry_t entry;
    int in_use;
} cache_slot_t;

typedef struct {
    cache_policy_t policy;
    uint32_t total_memory;
    uint32_t used_memory;
    uint32_t hit_count;
    uint32_t miss_count;
    uint32_t eviction_count;
} gpu_cache_state_t;

typedef struct {
    pipeline_request_t request;
    int in_use;
} pipeline_slot_t;

static cache_slot_t cache_table[MAX_CACHE_ENTRIES];
static gpu_cache_state_t cache_state[MAX_GPUS];
static pipeline_slot_t pipeline_queue[MAX_PIPELINE_REQUESTS];
static pipeline_stats_t pipeline_stats[MAX_GPUS];
static uint32_t cache_id_counter = 1;
static uint32_t request_id_counter = 1;

void gpu_cache_init(void)
{
    memset(cache_table, 0, sizeof(cache_table));
    memset(cache_state, 0, sizeof(cache_state));
    memset(pipeline_queue, 0, sizeof(pipeline_queue));
    memset(pipeline_stats, 0, sizeof(pipeline_stats));
    
    /* Initialize per-GPU cache state */
    for (int i = 0; i < MAX_GPUS; i++) {
        cache_state[i].policy = CACHE_POLICY_LRU;
        cache_state[i].total_memory = DEFAULT_CACHE_SIZE;
        cache_state[i].used_memory = 0;
        cache_state[i].hit_count = 0;
        cache_state[i].miss_count = 0;
        cache_state[i].eviction_count = 0;
        
        pipeline_stats[i].total_requests = 0;
        pipeline_stats[i].completed_requests = 0;
        pipeline_stats[i].pending_requests = 0;
        pipeline_stats[i].avg_latency_us = 0;
        pipeline_stats[i].max_throughput_reqs_sec = 1000;
        pipeline_stats[i].batches_processed = 0;
        pipeline_stats[i].avg_batch_size = 1;
    }
    
    cache_id_counter = 1;
    request_id_counter = 1;
    
    serial_puts("[gpu_cache] GPU cache and pipeline subsystem initialized\n");
}

int gpu_cache_set_policy(uint32_t gpu_id, cache_policy_t policy)
{
    if (gpu_id >= MAX_GPUS) return -1;
    if (policy < 0 || policy > 2) return -1;
    
    cache_state[gpu_id].policy = policy;
    
    serial_printf("[gpu_cache] GPU %d policy set to %d\n", gpu_id, policy);
    return 0;
}

uint32_t gpu_cache_allocate(uint32_t gpu_id, uint32_t source_id,
                            cache_entry_type_t type, uint32_t size)
{
    if (gpu_id >= MAX_GPUS || source_id == 0 || size == 0) return 0;
    
    /* Check available space, evict if needed */
    while (cache_state[gpu_id].used_memory + size > cache_state[gpu_id].total_memory) {
        uint32_t evicted = gpu_cache_evict_entry(gpu_id);
        if (evicted == 0) return 0;  /* Cannot evict more */
    }
    
    /* Find free cache slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_CACHE_ENTRIES; i++) {
        if (!cache_table[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;
    
    cache_slot_t *slot = &cache_table[free_idx];
    slot->entry.cache_id = cache_id_counter++;
    slot->entry.gpu_id = gpu_id;
    slot->entry.source_id = source_id;
    slot->entry.type = type;
    slot->entry.size_bytes = size;
    slot->entry.access_count = 0;
    slot->entry.hit_count = 0;
    slot->entry.created_time = 0;
    slot->entry.valid = 1;
    slot->in_use = 1;
    
    cache_state[gpu_id].used_memory += size;
    
    return slot->entry.cache_id;
}

int gpu_cache_free(uint32_t cache_id)
{
    if (cache_id == 0) return -1;
    
    for (int i = 0; i < MAX_CACHE_ENTRIES; i++) {
        if (cache_table[i].in_use && cache_table[i].entry.cache_id == cache_id) {
            uint32_t gpu_id = cache_table[i].entry.gpu_id;
            cache_state[gpu_id].used_memory -= cache_table[i].entry.size_bytes;
            cache_table[i].in_use = 0;
            
            return 0;
        }
    }
    
    return -1;
}

int gpu_cache_get(uint32_t cache_id, cache_entry_t *out)
{
    if (!out || cache_id == 0) return -1;
    
    for (int i = 0; i < MAX_CACHE_ENTRIES; i++) {
        if (cache_table[i].in_use && cache_table[i].entry.cache_id == cache_id) {
            memcpy(out, &cache_table[i].entry, sizeof(cache_entry_t));
            return 0;
        }
    }
    
    return -1;
}

int gpu_cache_hit(uint32_t cache_id)
{
    if (cache_id == 0) return -1;
    
    for (int i = 0; i < MAX_CACHE_ENTRIES; i++) {
        if (cache_table[i].in_use && cache_table[i].entry.cache_id == cache_id) {
            cache_table[i].entry.access_count++;
            cache_table[i].entry.hit_count++;
            cache_table[i].entry.last_access_time = 0;  /* Would be set from timer */
            
            uint32_t gpu_id = cache_table[i].entry.gpu_id;
            cache_state[gpu_id].hit_count++;
            
            return 0;
        }
    }
    
    return -1;
}

int gpu_cache_invalidate(uint32_t cache_id)
{
    if (cache_id == 0) return -1;
    
    for (int i = 0; i < MAX_CACHE_ENTRIES; i++) {
        if (cache_table[i].in_use && cache_table[i].entry.cache_id == cache_id) {
            cache_table[i].entry.valid = 0;
            return 0;
        }
    }
    
    return -1;
}

int gpu_cache_clear_source(uint32_t gpu_id, uint32_t source_id)
{
    if (gpu_id >= MAX_GPUS || source_id == 0) return -1;
    
    for (int i = 0; i < MAX_CACHE_ENTRIES; i++) {
        if (cache_table[i].in_use &&
            cache_table[i].entry.gpu_id == gpu_id &&
            cache_table[i].entry.source_id == source_id) {
            cache_state[gpu_id].used_memory -= cache_table[i].entry.size_bytes;
            cache_table[i].in_use = 0;
        }
    }
    
    return 0;
}

int gpu_cache_get_stats(uint32_t gpu_id, cache_stats_t *out)
{
    if (gpu_id >= MAX_GPUS || !out) return -1;
    
    uint32_t total_entries = 0;
    for (int i = 0; i < MAX_CACHE_ENTRIES; i++) {
        if (cache_table[i].in_use && cache_table[i].entry.gpu_id == gpu_id) {
            total_entries++;
        }
    }
    
    out->total_entries = total_entries;
    out->total_size = cache_state[gpu_id].used_memory;
    out->available_size = cache_state[gpu_id].total_memory - cache_state[gpu_id].used_memory;
    out->hit_count = cache_state[gpu_id].hit_count;
    out->miss_count = cache_state[gpu_id].miss_count;
    out->eviction_count = cache_state[gpu_id].eviction_count;
    out->avg_access_time_us = 100;  /* Simplified */
    
    return 0;
}

int gpu_cache_prefetch(uint32_t gpu_id, uint32_t source_id,
                       cache_entry_type_t type, uint32_t size)
{
    if (gpu_id >= MAX_GPUS || source_id == 0) return -1;
    
    uint32_t cache_id = gpu_cache_allocate(gpu_id, source_id, type, size);
    if (cache_id == 0) return -1;
    
    serial_printf("[gpu_cache] Prefetched source %d into GPU %d cache\n",
                 source_id, gpu_id);
    
    return 0;
}

uint32_t gpu_cache_evict_entry(uint32_t gpu_id)
{
    if (gpu_id >= MAX_GPUS) return 0;
    
    cache_policy_t policy = cache_state[gpu_id].policy;
    int victim_idx = -1;
    uint32_t victim_metric = 0xFFFFFFFF;
    
    /* Find eviction candidate based on policy */
    for (int i = 0; i < MAX_CACHE_ENTRIES; i++) {
        if (cache_table[i].in_use && cache_table[i].entry.gpu_id == gpu_id) {
            uint32_t metric = 0;
            
            if (policy == CACHE_POLICY_LRU) {
                metric = cache_table[i].entry.access_count;
            } else if (policy == CACHE_POLICY_LFU) {
                metric = cache_table[i].entry.hit_count;
            } else if (policy == CACHE_POLICY_FIFO) {
                metric = cache_table[i].entry.created_time;
            }
            
            if (metric < victim_metric) {
                victim_metric = metric;
                victim_idx = i;
            }
        }
    }
    
    if (victim_idx < 0) return 0;
    
    uint32_t evicted_id = cache_table[victim_idx].entry.cache_id;
    gpu_cache_free(evicted_id);
    cache_state[gpu_id].eviction_count++;
    
    return evicted_id;
}

int gpu_cache_set_limit(uint32_t gpu_id, uint32_t limit_bytes)
{
    if (gpu_id >= MAX_GPUS || limit_bytes == 0) return -1;
    
    cache_state[gpu_id].total_memory = limit_bytes;
    return 0;
}

uint32_t gpu_cache_get_usage(uint32_t gpu_id)
{
    if (gpu_id >= MAX_GPUS) return 0;
    return cache_state[gpu_id].used_memory;
}

void gpu_pipeline_init(void)
{
    memset(pipeline_queue, 0, sizeof(pipeline_queue));
    memset(pipeline_stats, 0, sizeof(pipeline_stats));
    
    request_id_counter = 1;
    
    serial_puts("[gpu_pipeline] GPU inference pipeline initialized\n");
}

uint32_t gpu_pipeline_submit(uint32_t model_id, uint32_t batch_size, uint32_t priority)
{
    if (model_id == 0 || batch_size == 0) return 0;
    
    /* Find free pipeline slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_PIPELINE_REQUESTS; i++) {
        if (!pipeline_queue[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;  /* Pipeline full */
    
    pipeline_slot_t *slot = &pipeline_queue[free_idx];
    slot->request.request_id = request_id_counter++;
    slot->request.model_id = model_id;
    slot->request.batch_size = batch_size;
    slot->request.priority = priority;
    slot->request.use_cache = 1;
    slot->request.prefetch_enabled = 0;
    slot->request.estimated_latency_us = batch_size * 1000;  /* Rough estimate */
    slot->request.submitted_time = 0;
    slot->in_use = 1;
    
    pipeline_stats[0].total_requests++;
    pipeline_stats[0].pending_requests++;
    
    return slot->request.request_id;
}

int gpu_pipeline_get_request(uint32_t request_id, pipeline_request_t *out)
{
    if (!out || request_id == 0) return -1;
    
    for (int i = 0; i < MAX_PIPELINE_REQUESTS; i++) {
        if (pipeline_queue[i].in_use && pipeline_queue[i].request.request_id == request_id) {
            memcpy(out, &pipeline_queue[i].request, sizeof(pipeline_request_t));
            return 0;
        }
    }
    
    return -1;
}

int gpu_pipeline_enable_batching(uint32_t model_id, uint16_t batch_size)
{
    if (model_id == 0 || batch_size == 0) return -1;
    
    serial_printf("[gpu_pipeline] Batching enabled for model %d (size=%d)\n",
                 model_id, batch_size);
    return 0;
}

int gpu_pipeline_enable_prefetch(uint32_t model_id)
{
    if (model_id == 0) return -1;
    
    serial_printf("[gpu_pipeline] Prefetching enabled for model %d\n", model_id);
    return 0;
}

int gpu_pipeline_enable_pipelining(uint32_t model_id)
{
    if (model_id == 0) return -1;
    
    serial_printf("[gpu_pipeline] Pipelining enabled for model %d\n", model_id);
    return 0;
}

uint32_t gpu_pipeline_process_batch(uint32_t gpu_id)
{
    if (gpu_id >= MAX_GPUS) return 0;
    
    /* Find highest priority pending request */
    int best_idx = -1;
    uint32_t best_priority = 0;
    
    for (int i = 0; i < MAX_PIPELINE_REQUESTS; i++) {
        if (pipeline_queue[i].in_use && 
            pipeline_queue[i].request.priority > best_priority) {
            best_priority = pipeline_queue[i].request.priority;
            best_idx = i;
        }
    }
    
    if (best_idx < 0) return 0;  /* No pending requests */
    
    uint32_t request_id = pipeline_queue[best_idx].request.request_id;
    pipeline_stats[gpu_id].batches_processed++;
    
    return request_id;
}

pipeline_stats_t *gpu_pipeline_get_stats(uint32_t gpu_id)
{
    if (gpu_id >= MAX_GPUS) return NULL;
    
    /* Update derived stats */
    uint32_t pending = 0;
    uint32_t total_batch = 0;
    
    for (int i = 0; i < MAX_PIPELINE_REQUESTS; i++) {
        if (pipeline_queue[i].in_use) {
            pending++;
            total_batch += pipeline_queue[i].request.batch_size;
        }
    }
    
    pipeline_stats[gpu_id].pending_requests = pending;
    if (pipeline_stats[gpu_id].batches_processed > 0) {
        pipeline_stats[gpu_id].avg_batch_size = total_batch / pipeline_stats[gpu_id].batches_processed;
    }
    
    return &pipeline_stats[gpu_id];
}

uint32_t gpu_pipeline_estimate_latency(uint32_t model_id, uint32_t batch_size)
{
    if (model_id == 0 || batch_size == 0) return 0;
    
    /* Simplified latency estimation: base + per-batch overhead */
    return 100 + (batch_size * 50);  /* Base 100us + 50us per item */
}

int gpu_pipeline_complete_request(uint32_t request_id)
{
    if (request_id == 0) return -1;
    
    for (int i = 0; i < MAX_PIPELINE_REQUESTS; i++) {
        if (pipeline_queue[i].in_use &&
            pipeline_queue[i].request.request_id == request_id) {
            
            pipeline_queue[i].request.completed_time = 0;  /* Would be set from timer */
            pipeline_queue[i].in_use = 0;
            
            pipeline_stats[0].completed_requests++;
            pipeline_stats[0].pending_requests--;
            
            return 0;
        }
    }
    
    return -1;
}

int gpu_pipeline_flush(uint32_t gpu_id)
{
    if (gpu_id >= MAX_GPUS) return -1;
    
    /* Complete all pending requests */
    uint32_t flushed = 0;
    for (int i = 0; i < MAX_PIPELINE_REQUESTS; i++) {
        if (pipeline_queue[i].in_use) {
            gpu_pipeline_complete_request(pipeline_queue[i].request.request_id);
            flushed++;
        }
    }
    
    serial_printf("[gpu_pipeline] Flushed %d requests from GPU %d\n", flushed, gpu_id);
    
    return 0;
}
