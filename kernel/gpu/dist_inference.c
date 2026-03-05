#include "../../include/kernel/dist_inference.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_INFERENCE_REQUESTS 256
#define MAX_MODEL_REPLICAS 64
#define MAX_GPUS 8

typedef struct {
    dist_infer_req_t request;
    int in_use;
} inference_request_t;

typedef struct {
    model_replica_t replica;
    int in_use;
} replica_entry_t;

typedef struct {
    gpu_device_status_t status;
    uint32_t total_latency;
    uint32_t latency_samples;
} gpu_state_t;

static inference_request_t requests[MAX_INFERENCE_REQUESTS];
static replica_entry_t replicas[MAX_MODEL_REPLICAS];
static gpu_state_t gpu_states[MAX_GPUS];
static dist_sched_stats_t stats = {0};
static dist_sched_config_t config = {0};
static uint32_t request_id_counter = 1;
static uint32_t replica_id_counter = 1;

void dist_inference_init(void)
{
    memset(requests, 0, sizeof(requests));
    memset(replicas, 0, sizeof(replicas));
    memset(gpu_states, 0, sizeof(gpu_states));
    memset(&stats, 0, sizeof(dist_sched_stats_t));
    
    /* Initialize GPU states */
    for (int i = 0; i < MAX_GPUS; i++) {
        gpu_states[i].status.gpu_id = i;
        gpu_states[i].status.available = 1;
        gpu_states[i].status.active_inferences = 0;
        gpu_states[i].status.queue_depth = 0;
        gpu_states[i].status.memory_utilization_pct = 0;
        gpu_states[i].status.power_draw_mw = 0;
        gpu_states[i].status.temperature_c = 25;
        gpu_states[i].status.inference_throughput = 0;
        gpu_states[i].total_latency = 0;
        gpu_states[i].latency_samples = 0;
    }
    
    /* Default configuration */
    config.strategy = LB_LEAST_LOADED;
    config.enable_replication = 1;
    config.enable_prefetch = 1;
    config.enable_batching = 1;
    config.max_batch_size = 64;
    config.load_check_interval_ms = 100;
    config.rebalance_threshold_pct = 30;
    
    stats.strategy = LB_LEAST_LOADED;
    
    request_id_counter = 1;
    replica_id_counter = 1;
    
    serial_puts("[dist_inference] Distributed inference scheduler initialized\n");
}

int dist_set_load_balance_strategy(load_balance_strategy_t strategy)
{
    if (strategy < 0 || strategy > 3) return -1;
    
    config.strategy = strategy;
    stats.strategy = strategy;
    
    serial_printf("[dist_inference] Load balance strategy set to %d\n", strategy);
    return 0;
}

int dist_get_gpu_status(uint32_t gpu_id, gpu_device_status_t *out)
{
    if (!out || gpu_id >= MAX_GPUS) return -1;
    
    memcpy(out, &gpu_states[gpu_id].status, sizeof(gpu_device_status_t));
    return 0;
}

uint32_t dist_get_all_gpu_status(gpu_device_status_t *devices, uint32_t max_devices)
{
    if (!devices || max_devices == 0) return 0;
    
    uint32_t count = (max_devices > MAX_GPUS) ? MAX_GPUS : max_devices;
    for (uint32_t i = 0; i < count; i++) {
        memcpy(&devices[i], &gpu_states[i].status, sizeof(gpu_device_status_t));
    }
    
    return count;
}

uint32_t dist_submit_inference(uint32_t model_id, uint32_t batch_size, uint32_t priority)
{
    if (batch_size == 0 || batch_size > config.max_batch_size) return 0;
    if (priority > 255) return 0;
    
    /* Find free request slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_INFERENCE_REQUESTS; i++) {
        if (!requests[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;  /* Queue full */
    
    inference_request_t *entry = &requests[free_idx];
    entry->request.request_id = request_id_counter++;
    entry->request.model_id = model_id;
    entry->request.batch_size = batch_size;
    entry->request.priority = priority;
    entry->request.assigned_gpu = 0xFFFFFFFF;  /* Not yet assigned */
    entry->request.created_time = 0;  /* Would be set from timer */
    entry->in_use = 1;
    
    stats.total_requests++;
    stats.pending_requests++;
    
    return entry->request.request_id;
}

int dist_get_request_status(uint32_t request_id, dist_infer_req_t *out)
{
    if (!out || request_id == 0) return -1;
    
    for (int i = 0; i < MAX_INFERENCE_REQUESTS; i++) {
        if (requests[i].in_use && requests[i].request.request_id == request_id) {
            memcpy(out, &requests[i].request, sizeof(dist_infer_req_t));
            return 0;
        }
    }
    
    return -1;
}

uint32_t dist_assign_gpu(uint32_t request_id, uint32_t model_id)
{
    if (request_id == 0 || model_id == 0) return 0xFFFFFFFF;
    
    /* Find request */
    int req_idx = -1;
    for (int i = 0; i < MAX_INFERENCE_REQUESTS; i++) {
        if (requests[i].in_use && requests[i].request.request_id == request_id) {
            req_idx = i;
            break;
        }
    }
    
    if (req_idx < 0) return 0xFFFFFFFF;
    
    uint32_t assigned_gpu = 0xFFFFFFFF;
    
    switch (config.strategy) {
        case LB_ROUND_ROBIN: {
            static uint32_t rr_idx = 0;
            assigned_gpu = rr_idx % MAX_GPUS;
            rr_idx++;
            break;
        }
        case LB_LEAST_LOADED: {
            uint32_t min_load = 0xFFFFFFFF;
            uint32_t best_gpu = 0;
            for (uint32_t i = 0; i < MAX_GPUS; i++) {
                if (gpu_states[i].status.available) {
                    uint32_t load = gpu_states[i].status.queue_depth * 100 / 
                                   (gpu_states[i].status.memory_utilization_pct + 1);
                    if (load < min_load) {
                        min_load = load;
                        best_gpu = i;
                    }
                }
            }
            assigned_gpu = best_gpu;
            break;
        }
        case LB_PERFORMANCE_AWARE: {
            uint32_t max_throughput = 0;
            uint32_t best_gpu = 0;
            for (uint32_t i = 0; i < MAX_GPUS; i++) {
                if (gpu_states[i].status.available &&
                    gpu_states[i].status.inference_throughput > max_throughput) {
                    max_throughput = gpu_states[i].status.inference_throughput;
                    best_gpu = i;
                }
            }
            assigned_gpu = best_gpu;
            break;
        }
        case LB_POWER_AWARE: {
            uint32_t min_power = 0xFFFFFFFF;
            uint32_t best_gpu = 0;
            for (uint32_t i = 0; i < MAX_GPUS; i++) {
                if (gpu_states[i].status.available &&
                    gpu_states[i].status.power_draw_mw < min_power) {
                    min_power = gpu_states[i].status.power_draw_mw;
                    best_gpu = i;
                }
            }
            assigned_gpu = best_gpu;
            break;
        }
        default:
            assigned_gpu = 0;
            break;
    }
    
    if (assigned_gpu < MAX_GPUS) {
        requests[req_idx].request.assigned_gpu = assigned_gpu;
        gpu_states[assigned_gpu].status.queue_depth++;
        gpu_states[assigned_gpu].status.active_inferences++;
    }
    
    return assigned_gpu;
}

uint32_t dist_replicate_model(uint32_t model_id, uint32_t gpu_id)
{
    if (model_id == 0 || gpu_id >= MAX_GPUS) return 0;
    
    /* Find free replica slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_MODEL_REPLICAS; i++) {
        if (!replicas[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;  /* Too many replicas */
    
    /* Check if already replicated on this GPU */
    for (int i = 0; i < MAX_MODEL_REPLICAS; i++) {
        if (replicas[i].in_use && 
            replicas[i].replica.model_id == model_id &&
            replicas[i].replica.gpu_id == gpu_id) {
            return replicas[i].replica.replica_index;  /* Already exists */
        }
    }
    
    replica_entry_t *entry = &replicas[free_idx];
    entry->replica.model_id = model_id;
    entry->replica.gpu_id = gpu_id;
    entry->replica.replica_index = replica_id_counter++;
    entry->replica.inference_count = 0;
    entry->replica.avg_latency_us = 0;
    entry->replica.ready = 1;
    entry->in_use = 1;
    
    stats.total_model_replicas++;
    stats.active_replicas++;
    
    serial_printf("[dist_inference] Replicated model %d on GPU %d\n", model_id, gpu_id);
    
    return entry->replica.replica_index;
}

int dist_get_model_replica(uint32_t model_id, uint32_t gpu_id, model_replica_t *out)
{
    if (!out || model_id == 0 || gpu_id >= MAX_GPUS) return -1;
    
    for (int i = 0; i < MAX_MODEL_REPLICAS; i++) {
        if (replicas[i].in_use && 
            replicas[i].replica.model_id == model_id &&
            replicas[i].replica.gpu_id == gpu_id) {
            memcpy(out, &replicas[i].replica, sizeof(model_replica_t));
            return 0;
        }
    }
    
    return -1;
}

uint32_t dist_list_model_replicas(uint32_t model_id, model_replica_t *replicas_out, uint32_t max)
{
    if (!replicas_out || max == 0 || model_id == 0) return 0;
    
    uint32_t count = 0;
    for (int i = 0; i < MAX_MODEL_REPLICAS && count < max; i++) {
        if (replicas[i].in_use && replicas[i].replica.model_id == model_id) {
            memcpy(&replicas_out[count], &replicas[i].replica, sizeof(model_replica_t));
            count++;
        }
    }
    
    return count;
}

int dist_rebalance_load(void)
{
    /* Simplified rebalancing: mark GPUs for rebalance */
    uint32_t avg_queue = 0;
    uint32_t active_gpus = 0;
    
    for (int i = 0; i < MAX_GPUS; i++) {
        if (gpu_states[i].status.available) {
            avg_queue += gpu_states[i].status.queue_depth;
            active_gpus++;
        }
    }
    
    if (active_gpus == 0) return -1;
    avg_queue /= active_gpus;
    
    /* Check if any GPU exceeds threshold */
    for (int i = 0; i < MAX_GPUS; i++) {
        if (gpu_states[i].status.available) {
            uint32_t diff = (gpu_states[i].status.queue_depth > avg_queue) ?
                           (gpu_states[i].status.queue_depth - avg_queue) :
                           (avg_queue - gpu_states[i].status.queue_depth);
            
            if (diff > (avg_queue * config.rebalance_threshold_pct / 100)) {
                serial_printf("[dist_inference] Rebalancing triggered at GPU %d\n", i);
                return 0;  /* Rebalance in progress */
            }
        }
    }
    
    return 1;  /* No rebalance needed */
}

int dist_enable_prefetch(uint32_t model_id, uint32_t gpu_id)
{
    if (model_id == 0 || gpu_id >= MAX_GPUS) return -1;
    
    /* Verify replica exists */
    for (int i = 0; i < MAX_MODEL_REPLICAS; i++) {
        if (replicas[i].in_use &&
            replicas[i].replica.model_id == model_id &&
            replicas[i].replica.gpu_id == gpu_id) {
            serial_printf("[dist_inference] Prefetch enabled for model %d on GPU %d\n",
                         model_id, gpu_id);
            return 0;
        }
    }
    
    return -1;
}

int dist_enable_batching(uint32_t model_id, uint16_t batch_size)
{
    if (model_id == 0 || batch_size == 0 || batch_size > config.max_batch_size) {
        return -1;
    }
    
    serial_printf("[dist_inference] Batching enabled for model %d (batch_size=%d)\n",
                 model_id, batch_size);
    return 0;
}

dist_sched_stats_t *dist_get_stats(void)
{
    /* Update derived statistics */
    stats.pending_requests = 0;
    for (int i = 0; i < MAX_INFERENCE_REQUESTS; i++) {
        if (requests[i].in_use && requests[i].request.assigned_gpu == 0xFFFFFFFF) {
            stats.pending_requests++;
        }
    }
    
    /* Calculate average latency */
    uint32_t total_latency = 0;
    uint32_t total_samples = 0;
    for (int i = 0; i < MAX_GPUS; i++) {
        total_latency += gpu_states[i].total_latency;
        total_samples += gpu_states[i].latency_samples;
    }
    
    if (total_samples > 0) {
        stats.avg_latency_us = total_latency / total_samples;
    }
    
    return &stats;
}

int dist_get_config(dist_sched_config_t *out)
{
    if (!out) return -1;
    
    memcpy(out, &config, sizeof(dist_sched_config_t));
    return 0;
}

int dist_set_config(dist_sched_config_t *new_config)
{
    if (!new_config) return -1;
    
    /* Validate config */
    if (new_config->strategy < 0 || new_config->strategy > 3) return -1;
    if (new_config->max_batch_size == 0 || new_config->max_batch_size > 256) return -1;
    
    memcpy(&config, new_config, sizeof(dist_sched_config_t));
    stats.strategy = config.strategy;
    
    return 0;
}

uint32_t dist_estimate_best_gpu(uint32_t model_id, uint32_t batch_size)
{
    if (model_id == 0 || batch_size == 0) return 0xFFFFFFFF;
    
    /* Estimate based on current config and GPU state */
    uint32_t best_gpu = 0;
    
    if (config.strategy == LB_LEAST_LOADED) {
        uint32_t min_load = 0xFFFFFFFF;
        for (uint32_t i = 0; i < MAX_GPUS; i++) {
            if (gpu_states[i].status.available) {
                uint32_t load = gpu_states[i].status.queue_depth + batch_size;
                if (load < min_load) {
                    min_load = load;
                    best_gpu = i;
                }
            }
        }
    } else if (config.strategy == LB_PERFORMANCE_AWARE) {
        uint32_t max_throughput = 0;
        for (uint32_t i = 0; i < MAX_GPUS; i++) {
            if (gpu_states[i].status.available &&
                gpu_states[i].status.inference_throughput > max_throughput) {
                max_throughput = gpu_states[i].status.inference_throughput;
                best_gpu = i;
            }
        }
    }
    
    return best_gpu;
}

uint32_t dist_get_avg_latency(uint32_t gpu_id)
{
    if (gpu_id >= MAX_GPUS) return 0;
    
    if (gpu_states[gpu_id].latency_samples == 0) return 0;
    return gpu_states[gpu_id].total_latency / gpu_states[gpu_id].latency_samples;
}

int dist_cleanup_completed_requests(uint32_t older_than_ms)
{
    (void)older_than_ms;  /* Simplified: not tracking time-based cleanup */
    
    /* Could implement time-based cleanup here */
    return 0;
}
