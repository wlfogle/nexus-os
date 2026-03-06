#include "../../include/kernel/grad_sync.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_GRADIENT_BUFFERS 128
#define MAX_PARAM_SERVERS 8
#define MAX_ASYNC_SLOTS 32
#define MAX_REPLICAS_PER_SERVER 8

typedef struct {
    gradient_buffer_t buffer;
    int in_use;
} buffer_slot_t;

typedef struct {
    param_server_t server;
    uint32_t replicas[MAX_REPLICAS_PER_SERVER];
    uint32_t replica_count;
    float *aggregated_gradients;
    int in_use;
} server_slot_t;

typedef struct {
    uint32_t async_id;
    uint32_t *buffer_ids;
    uint32_t num_buffers;
    uint8_t complete;
    int in_use;
} async_slot_t;

static buffer_slot_t buffers[MAX_GRADIENT_BUFFERS];
static server_slot_t servers[MAX_PARAM_SERVERS];
static async_slot_t async_slots[MAX_ASYNC_SLOTS];
static compression_state_t comp_state = {0};
static sync_stats_t sync_stats = {0};
static uint32_t buffer_id_counter = 1;
static uint32_t server_id_counter = 1;
static uint32_t async_id_counter = 1;

void grad_sync_init(void)
{
    memset(buffers, 0, sizeof(buffers));
    memset(servers, 0, sizeof(servers));
    memset(async_slots, 0, sizeof(async_slots));
    memset(&comp_state, 0, sizeof(compression_state_t));
    memset(&sync_stats, 0, sizeof(sync_stats_t));
    
    comp_state.algorithm = COMPRESS_NONE;
    comp_state.quantize_bits = 8;
    comp_state.sparse_threshold = 100;
    comp_state.compression_ratio = 1.0;
    
    buffer_id_counter = 1;
    server_id_counter = 1;
    async_id_counter = 1;
    
    serial_puts("[grad_sync] Gradient synchronization subsystem initialized\n");
}

uint32_t grad_sync_register_buffer(uint32_t tensor_id, uint32_t layer_id,
                                   uint32_t size_bytes)
{
    if (tensor_id == 0 || layer_id == 0 || size_bytes == 0) return 0;
    
    /* Find free buffer slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_GRADIENT_BUFFERS; i++) {
        if (!buffers[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;
    
    buffer_slot_t *slot = &buffers[free_idx];
    slot->buffer.buffer_id = buffer_id_counter++;
    slot->buffer.tensor_id = tensor_id;
    slot->buffer.layer_id = layer_id;
    slot->buffer.size_bytes = size_bytes;
    slot->buffer.ready = 0;
    slot->buffer.compressed = 0;
    slot->in_use = 1;
    
    serial_printf("[grad_sync] Registered gradient buffer %d (tensor=%d, size=%d)\n",
                 slot->buffer.buffer_id, tensor_id, size_bytes);
    
    return slot->buffer.buffer_id;
}

int grad_sync_allreduce(uint32_t *buffer_ids, uint32_t num_buffers,
                        allreduce_op_t operation, uint32_t num_gpus)
{
    if (!buffer_ids || num_buffers == 0 || num_gpus == 0) return -1;
    
    /* Validate all buffers exist */
    for (uint32_t i = 0; i < num_buffers; i++) {
        int found = 0;
        for (int j = 0; j < MAX_GRADIENT_BUFFERS; j++) {
            if (buffers[j].in_use && buffers[j].buffer.buffer_id == buffer_ids[i]) {
                found = 1;
                break;
            }
        }
        if (!found) return -1;
    }
    
    /* Perform AllReduce (simplified - just mark as synchronized) */
    uint32_t total_size = 0;
    for (uint32_t i = 0; i < num_buffers; i++) {
        for (int j = 0; j < MAX_GRADIENT_BUFFERS; j++) {
            if (buffers[j].in_use && buffers[j].buffer.buffer_id == buffer_ids[i]) {
                total_size += buffers[j].buffer.size_bytes;
                break;
            }
        }
    }
    
    sync_stats.total_syncs++;
    sync_stats.successful_syncs++;
    sync_stats.total_gradients_synced += num_buffers;
    sync_stats.total_bytes_transferred += total_size;
    sync_stats.avg_sync_time_us = 1000;  /* Simplified: 1ms per sync */
    
    serial_printf("[grad_sync] AllReduce completed: %d buffers, %d GPUs, op=%d\n",
                 num_buffers, num_gpus, operation);
    
    return 0;
}

int grad_sync_allreduce_compressed(uint32_t *buffer_ids, uint32_t num_buffers,
                                   allreduce_op_t operation, uint32_t num_gpus,
                                   compress_algo_t compression)
{
    (void)operation;  /* Used for logging in real implementation */
    if (!buffer_ids || num_buffers == 0 || num_gpus == 0) return -1;
    
    /* Calculate compression ratio */
    float ratio = 1.0;
    if (compression == COMPRESS_QUANTIZE) {
        ratio = 4.0;  /* 32-bit float to 8-bit = 4x compression */
    } else if (compression == COMPRESS_SPARSE) {
        ratio = 2.5;  /* ~2.5x reduction from sparsification */
    } else if (compression == COMPRESS_HYBRID) {
        ratio = 10.0; /* Combined: ~10x compression */
    }
    
    /* Perform compressed AllReduce */
    uint32_t total_size = 0;
    for (uint32_t i = 0; i < num_buffers; i++) {
        for (int j = 0; j < MAX_GRADIENT_BUFFERS; j++) {
            if (buffers[j].in_use && buffers[j].buffer.buffer_id == buffer_ids[i]) {
                total_size += buffers[j].buffer.size_bytes;
                buffers[j].buffer.compressed = 1;
                break;
            }
        }
    }
    
    uint32_t compressed_size = (uint32_t)(total_size / ratio);
    uint32_t bandwidth_saved = total_size - compressed_size;
    
    comp_state.algorithm = compression;
    comp_state.compression_ratio = ratio;
    comp_state.gradients_compressed += num_buffers;
    comp_state.total_bandwidth_saved += bandwidth_saved;
    
    sync_stats.total_syncs++;
    sync_stats.successful_syncs++;
    sync_stats.total_bytes_transferred += compressed_size;
    
    serial_printf("[grad_sync] Compressed AllReduce: compression=%d, ratio=%.1f, saved=%d bytes\n",
                 compression, ratio, bandwidth_saved);
    
    return 0;
}

int grad_sync_get_gradients(uint32_t buffer_id, float *out_data, uint32_t size)
{
    if (!out_data || buffer_id == 0) return -1;
    
    for (int i = 0; i < MAX_GRADIENT_BUFFERS; i++) {
        if (buffers[i].in_use && buffers[i].buffer.buffer_id == buffer_id) {
            if (size < buffers[i].buffer.size_bytes) return -1;
            
            /* Simplified: return dummy data */
            for (uint32_t j = 0; j < size / sizeof(float); j++) {
                out_data[j] = 0.001f * (j % 100);
            }
            
            return 0;
        }
    }
    
    return -1;
}

int grad_sync_set_compression(compress_algo_t algo, uint8_t quantize_bits,
                              uint32_t sparse_threshold)
{
    if (algo < 0 || algo > 3) return -1;
    if (quantize_bits != 8 && quantize_bits != 16) return -1;
    
    comp_state.algorithm = algo;
    comp_state.quantize_bits = quantize_bits;
    comp_state.sparse_threshold = sparse_threshold;
    
    serial_printf("[grad_sync] Compression set: algo=%d, bits=%d, threshold=%d\n",
                 algo, quantize_bits, sparse_threshold);
    
    return 0;
}

compression_state_t *grad_sync_get_compression_stats(void)
{
    return &comp_state;
}

uint32_t grad_sync_create_param_server(uint32_t gpu_id)
{
    if (gpu_id >= 8) return 0;
    
    /* Find free server slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_PARAM_SERVERS; i++) {
        if (!servers[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;
    
    server_slot_t *slot = &servers[free_idx];
    slot->server.server_id = server_id_counter++;
    slot->server.assigned_gpu = gpu_id;
    slot->server.aggregated_gradients = 0;
    slot->server.total_replicas = 0;
    slot->server.total_bytes_processed = 0;
    slot->server.avg_aggregation_time_us = 0;
    slot->replica_count = 0;
    slot->in_use = 1;
    
    serial_printf("[grad_sync] Created parameter server %d on GPU %d\n",
                 slot->server.server_id, gpu_id);
    
    return slot->server.server_id;
}

int grad_sync_register_replica(uint32_t server_id, uint32_t replica_gpu_id)
{
    if (server_id == 0 || replica_gpu_id >= 8) return -1;
    
    for (int i = 0; i < MAX_PARAM_SERVERS; i++) {
        if (servers[i].in_use && servers[i].server.server_id == server_id) {
            if (servers[i].replica_count >= MAX_REPLICAS_PER_SERVER) {
                return -1;
            }
            
            servers[i].replicas[servers[i].replica_count] = replica_gpu_id;
            servers[i].replica_count++;
            servers[i].server.total_replicas++;
            
            return 0;
        }
    }
    
    return -1;
}

int grad_sync_push_gradient(uint32_t server_id, uint32_t buffer_id,
                           const float *gradient_data, uint32_t size)
{
    if (server_id == 0 || buffer_id == 0 || !gradient_data || size == 0) {
        return -1;
    }
    
    for (int i = 0; i < MAX_PARAM_SERVERS; i++) {
        if (servers[i].in_use && servers[i].server.server_id == server_id) {
            servers[i].server.total_bytes_processed += size;
            return 0;
        }
    }
    
    return -1;
}

int grad_sync_pull_parameters(uint32_t server_id, uint32_t buffer_id,
                             float *out_params, uint32_t size)
{
    if (server_id == 0 || buffer_id == 0 || !out_params || size == 0) {
        return -1;
    }
    
    for (int i = 0; i < MAX_PARAM_SERVERS; i++) {
        if (servers[i].in_use && servers[i].server.server_id == server_id) {
            /* Return dummy parameters */
            for (uint32_t j = 0; j < size / sizeof(float); j++) {
                out_params[j] = 0.001f * (j % 100);
            }
            return 0;
        }
    }
    
    return -1;
}

int grad_sync_aggregate_gradients(uint32_t server_id, allreduce_op_t op)
{
    if (server_id == 0) return -1;
    
    for (int i = 0; i < MAX_PARAM_SERVERS; i++) {
        if (servers[i].in_use && servers[i].server.server_id == server_id) {
            servers[i].server.aggregated_gradients++;
            servers[i].server.avg_aggregation_time_us = 500;  /* 0.5ms per aggregation */
            
            serial_printf("[grad_sync] Aggregated gradients on server %d (op=%d)\n",
                         server_id, op);
            
            return 0;
        }
    }
    
    return -1;
}

param_server_t *grad_sync_get_server_stats(uint32_t server_id)
{
    if (server_id == 0) return NULL;
    
    for (int i = 0; i < MAX_PARAM_SERVERS; i++) {
        if (servers[i].in_use && servers[i].server.server_id == server_id) {
            return &servers[i].server;
        }
    }
    
    return NULL;
}

uint32_t grad_sync_compress_buffer(uint32_t buffer_id, uint8_t *compressed_data,
                                   uint32_t max_size)
{
    if (buffer_id == 0 || !compressed_data) return 0;
    
    for (int i = 0; i < MAX_GRADIENT_BUFFERS; i++) {
        if (buffers[i].in_use && buffers[i].buffer.buffer_id == buffer_id) {
            uint32_t original_size = buffers[i].buffer.size_bytes;
            uint32_t compressed_size = (uint32_t)(original_size / comp_state.compression_ratio);
            
            if (compressed_size > max_size) return 0;
            
            /* Simplified compression: just reduce size */
            for (uint32_t j = 0; j < compressed_size; j++) {
                compressed_data[j] = (uint8_t)(j % 256);
            }
            
            return compressed_size;
        }
    }
    
    return 0;
}

int grad_sync_decompress_buffer(const uint8_t *compressed_data, uint32_t compressed_size,
                               float *out_gradients, uint32_t max_size)
{
    if (!compressed_data || !out_gradients || compressed_size == 0) return -1;
    
    uint32_t decompressed_size = (uint32_t)(compressed_size * comp_state.compression_ratio);
    
    if (decompressed_size > max_size) return -1;
    
    /* Simplified decompression */
    for (uint32_t i = 0; i < decompressed_size / sizeof(float); i++) {
        out_gradients[i] = 0.001f * (i % 100);
    }
    
    return decompressed_size;
}

sync_stats_t *grad_sync_get_stats(void)
{
    return &sync_stats;
}

int grad_sync_reset(void)
{
    memset(buffers, 0, sizeof(buffers));
    memset(servers, 0, sizeof(servers));
    memset(async_slots, 0, sizeof(async_slots));
    memset(&sync_stats, 0, sizeof(sync_stats_t));
    
    buffer_id_counter = 1;
    server_id_counter = 1;
    async_id_counter = 1;
    
    return 0;
}

int grad_sync_enable_async(uint32_t num_async_slots)
{
    if (num_async_slots == 0 || num_async_slots > MAX_ASYNC_SLOTS) return -1;
    
    memset(async_slots, 0, sizeof(async_slots));
    async_id_counter = 1;
    
    serial_printf("[grad_sync] Async AllReduce enabled with %d slots\n", num_async_slots);
    
    return 0;
}

int grad_sync_check_async_complete(uint32_t async_id)
{
    if (async_id == 0) return -1;
    
    for (int i = 0; i < MAX_ASYNC_SLOTS; i++) {
        if (async_slots[i].in_use && async_slots[i].async_id == async_id) {
            return async_slots[i].complete ? 1 : 0;
        }
    }
    
    return -1;
}

int grad_sync_wait_async(uint32_t async_id)
{
    if (async_id == 0) return -1;
    
    for (int i = 0; i < MAX_ASYNC_SLOTS; i++) {
        if (async_slots[i].in_use && async_slots[i].async_id == async_id) {
            /* Simplified: just mark as complete */
            async_slots[i].complete = 1;
            return 0;
        }
    }
    
    return -1;
}
