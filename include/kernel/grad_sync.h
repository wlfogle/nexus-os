#ifndef KERNEL_GRAD_SYNC_H
#define KERNEL_GRAD_SYNC_H

#include <stdint.h>

/* AllReduce operation types */
typedef enum {
    ALLREDUCE_SUM = 0,
    ALLREDUCE_MEAN = 1,
    ALLREDUCE_MAX = 2,
    ALLREDUCE_MIN = 3
} allreduce_op_t;

/* Gradient compression algorithms */
typedef enum {
    COMPRESS_NONE = 0,
    COMPRESS_QUANTIZE = 1,      /* 8-bit quantization */
    COMPRESS_SPARSE = 2,         /* Sparsification (top-k) */
    COMPRESS_HYBRID = 3          /* Quantization + sparsification */
} compress_algo_t;

/* Gradient buffer descriptor */
typedef struct {
    uint32_t buffer_id;
    uint32_t size_bytes;
    uint32_t tensor_id;
    uint32_t layer_id;
    float *data;                 /* Gradient data pointer */
    uint8_t ready;
    uint8_t compressed;
} gradient_buffer_t;

/* Parameter server for gradient aggregation */
typedef struct {
    uint32_t server_id;
    uint32_t assigned_gpu;
    uint32_t aggregated_gradients;
    uint32_t total_replicas;
    uint64_t total_bytes_processed;
    uint32_t avg_aggregation_time_us;
} param_server_t;

/* Gradient compression state */
typedef struct {
    compress_algo_t algorithm;
    float compression_ratio;    /* Original / compressed */
    uint32_t sparse_threshold;  /* For top-k sparsification */
    uint8_t quantize_bits;      /* 8 or 16 bit quantization */
    uint32_t gradients_compressed;
    uint32_t total_bandwidth_saved;
} compression_state_t;

/* Synchronization statistics */
typedef struct {
    uint32_t total_syncs;
    uint32_t successful_syncs;
    uint32_t failed_syncs;
    uint32_t avg_sync_time_us;
    uint32_t max_sync_time_us;
    uint32_t total_gradients_synced;
    uint32_t total_bytes_transferred;
} sync_stats_t;

/* Core Gradient Synchronization APIs */

/**
 * Initialize gradient synchronization subsystem
 */
void grad_sync_init(void);

/**
 * Register gradient buffer for synchronization
 */
uint32_t grad_sync_register_buffer(uint32_t tensor_id, uint32_t layer_id, 
                                   uint32_t size_bytes);

/**
 * Perform AllReduce operation on gradients
 */
int grad_sync_allreduce(uint32_t *buffer_ids, uint32_t num_buffers,
                        allreduce_op_t operation, uint32_t num_gpus);

/**
 * Perform AllReduce with gradient compression
 */
int grad_sync_allreduce_compressed(uint32_t *buffer_ids, uint32_t num_buffers,
                                   allreduce_op_t operation, uint32_t num_gpus,
                                   compress_algo_t compression);

/**
 * Get synchronized gradients
 */
int grad_sync_get_gradients(uint32_t buffer_id, float *out_data, uint32_t size);

/**
 * Set compression algorithm
 */
int grad_sync_set_compression(compress_algo_t algo, uint8_t quantize_bits,
                              uint32_t sparse_threshold);

/**
 * Get compression statistics
 */
compression_state_t *grad_sync_get_compression_stats(void);

/**
 * Create parameter server on GPU
 */
uint32_t grad_sync_create_param_server(uint32_t gpu_id);

/**
 * Register replica with parameter server
 */
int grad_sync_register_replica(uint32_t server_id, uint32_t replica_gpu_id);

/**
 * Push gradients to parameter server
 */
int grad_sync_push_gradient(uint32_t server_id, uint32_t buffer_id,
                           const float *gradient_data, uint32_t size);

/**
 * Pull updated parameters from server
 */
int grad_sync_pull_parameters(uint32_t server_id, uint32_t buffer_id,
                             float *out_params, uint32_t size);

/**
 * Aggregate gradients on parameter server
 */
int grad_sync_aggregate_gradients(uint32_t server_id, allreduce_op_t op);

/**
 * Get parameter server statistics
 */
param_server_t *grad_sync_get_server_stats(uint32_t server_id);

/**
 * Compress gradient buffer
 */
uint32_t grad_sync_compress_buffer(uint32_t buffer_id, uint8_t *compressed_data,
                                   uint32_t max_size);

/**
 * Decompress gradient buffer
 */
int grad_sync_decompress_buffer(const uint8_t *compressed_data, uint32_t compressed_size,
                               float *out_gradients, uint32_t max_size);

/**
 * Get synchronization statistics
 */
sync_stats_t *grad_sync_get_stats(void);

/**
 * Reset synchronization state
 */
int grad_sync_reset(void);

/**
 * Enable asynchronous AllReduce
 */
int grad_sync_enable_async(uint32_t num_async_slots);

/**
 * Check async AllReduce completion
 */
int grad_sync_check_async_complete(uint32_t async_id);

/**
 * Wait for async AllReduce to complete
 */
int grad_sync_wait_async(uint32_t async_id);

#endif /* KERNEL_GRAD_SYNC_H */
