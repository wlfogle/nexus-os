#ifndef KERNEL_MODEL_RUNTIME_H
#define KERNEL_MODEL_RUNTIME_H

#include "../libc/stdint.h"

/* Model Format Types */
typedef enum {
    MODEL_FORMAT_ONNX = 1,       /* ONNX Runtime format */
    MODEL_FORMAT_TENSORRT = 2,   /* NVIDIA TensorRT */
    MODEL_FORMAT_TFLITE = 3,     /* TensorFlow Lite */
    MODEL_FORMAT_NCNN = 4,       /* NCNN (mobile inference) */
    MODEL_FORMAT_CUSTOM = 5      /* Custom/proprietary format */
} model_format_t;

/* Model Precision Types */
typedef enum {
    MODEL_PRECISION_FP32 = 1,    /* 32-bit floating point */
    MODEL_PRECISION_FP16 = 2,    /* 16-bit floating point (half) */
    MODEL_PRECISION_INT8 = 3,    /* 8-bit integer (quantized) */
    MODEL_PRECISION_INT4 = 4     /* 4-bit integer (extreme quantization) */
} model_precision_t;

/* Inference Request Status */
typedef enum {
    INFERENCE_PENDING = 1,
    INFERENCE_RUNNING = 2,
    INFERENCE_COMPLETED = 3,
    INFERENCE_ERROR = 4,
    INFERENCE_TIMEOUT = 5
} inference_status_t;

/* Model Metadata */
typedef struct {
    uint32_t model_id;
    const char *name;
    const char *version;
    model_format_t format;
    model_precision_t precision;
    uint32_t input_count;
    uint32_t output_count;
    uint32_t parameter_count;     /* Number of weights/parameters */
    uint64_t model_size;          /* Size in bytes */
    uint32_t gpu_memory_required;
    uint32_t max_batch_size;
    uint32_t device_id;
} model_t;

/* Tensor Descriptor */
typedef struct {
    uint32_t tensor_id;
    const char *name;
    uint32_t rank;                /* Number of dimensions */
    uint32_t shape[8];            /* Dimensions (max 8D) */
    uint32_t data_type;           /* Element data type (fp32, int8, etc.) */
    uint32_t size_bytes;          /* Total size in bytes */
} tensor_desc_t;

/* Inference Request */
typedef struct {
    uint32_t request_id;
    uint32_t model_id;
    uint64_t timestamp;           /* Request submission time */
    tensor_desc_t *input_tensors;
    tensor_desc_t *output_tensors;
    uint32_t batch_size;
    uint32_t timeout_ms;
    inference_status_t status;
    uint64_t latency_us;          /* Execution time in microseconds */
} inference_request_t;

/* Runtime Statistics */
typedef struct {
    uint64_t total_inferences;
    uint64_t successful_inferences;
    uint64_t failed_inferences;
    uint64_t total_latency_us;
    uint32_t max_latency_us;
    uint32_t min_latency_us;
    uint32_t avg_latency_us;
    uint64_t models_loaded;
    uint64_t peak_memory_usage;
    uint32_t current_memory_usage;
} runtime_stats_t;

/* Model Runtime Service API */
void model_runtime_init(void);

/* Model Management */
int model_load(const char *path, model_format_t format, uint32_t gpu_device_id, model_t **out_model);
int model_unload(uint32_t model_id);
model_t *model_get(uint32_t model_id);
int model_list(model_t **models, int max_count);

/* Inference Execution */
int inference_execute(uint32_t model_id, inference_request_t *request);
int inference_wait(uint32_t request_id, inference_request_t *result);
int inference_is_complete(uint32_t request_id);
int inference_cancel(uint32_t request_id);

/* Tensor Operations */
int tensor_allocate(tensor_desc_t *desc, void **host_ptr, uint32_t *device_ptr);
int tensor_free(uint32_t tensor_id);
int tensor_upload(uint32_t tensor_id, const void *host_data, uint32_t size);
int tensor_download(uint32_t tensor_id, void *host_data, uint32_t size);

/* Performance and Diagnostics */
runtime_stats_t *model_runtime_get_stats(void);
int model_benchmark(uint32_t model_id, uint32_t iterations, uint32_t *avg_latency_us);
int model_get_memory_usage(uint32_t model_id, uint32_t *vram_used, uint32_t *host_used);

/* Batch Processing */
int inference_batch_execute(uint32_t model_id, inference_request_t *requests, 
                           int request_count);
int inference_batch_wait(uint32_t *request_ids, int count);

/* Service Control */
int model_runtime_start(void);
int model_runtime_stop(void);
int model_runtime_is_running(void);

#endif /* KERNEL_MODEL_RUNTIME_H */
