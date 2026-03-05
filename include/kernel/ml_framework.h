#ifndef KERNEL_ML_FRAMEWORK_H
#define KERNEL_ML_FRAMEWORK_H

#include <stdint.h>
#include <stddef.h>

/* Maximum number of concurrent models in memory */
#define ML_MAX_MODELS 16

/* Maximum number of layers per model */
#define ML_MAX_LAYERS 128

/* Maximum batch size */
#define ML_MAX_BATCH_SIZE 256

/* Maximum tensor dimensions */
#define ML_MAX_TENSOR_DIMS 8

/* Maximum model signature entries */
#define ML_MAX_SIGNATURES 32

/* Model format identifiers */
typedef enum {
    ML_FORMAT_UNKNOWN = 0,
    ML_FORMAT_ONNX = 1,
    ML_FORMAT_SAVEDMODEL = 2,
    ML_FORMAT_PYTORCH_JIT = 3,
} ml_model_format_t;

/* Operator types */
typedef enum {
    ML_OP_DENSE = 0,
    ML_OP_CONV2D = 1,
    ML_OP_RELU = 2,
    ML_OP_SOFTMAX = 3,
    ML_OP_BATCHNORM = 4,
    ML_OP_POOLING = 5,
    ML_OP_DROPOUT = 6,
    ML_OP_EMBEDDING = 7,
    ML_OP_LSTM = 8,
    ML_OP_ATTENTION = 9,
} ml_operator_t;

/* Quantization methods */
typedef enum {
    ML_QUANT_NONE = 0,
    ML_QUANT_INT8 = 1,
    ML_QUANT_INT4 = 2,
} ml_quantization_method_t;

/* Tensor data type */
typedef enum {
    ML_DTYPE_FLOAT32 = 0,
    ML_DTYPE_FLOAT16 = 1,
    ML_DTYPE_INT32 = 2,
    ML_DTYPE_INT8 = 3,
} ml_data_type_t;

/* Tensor shape and metadata */
typedef struct {
    uint32_t dims[ML_MAX_TENSOR_DIMS];
    uint32_t ndims;
    ml_data_type_t dtype;
    uint32_t size_bytes;
} ml_tensor_shape_t;

/* Layer descriptor */
typedef struct {
    uint16_t layer_id;
    ml_operator_t op_type;
    uint16_t input_count;
    uint16_t output_count;
    uint32_t param_count;
    uint32_t param_bytes;
    uint32_t fused_flag;  /* Operator fusion flag (0=disabled, 1=enabled) */
    uint16_t padding;
} ml_layer_t;

/* Model signature entry (input/output mapping) */
typedef struct {
    char name[32];
    uint16_t tensor_index;
    ml_tensor_shape_t shape;
} ml_signature_entry_t;

/* Model signature (input/output spec) */
typedef struct {
    uint32_t sig_id;
    uint16_t input_count;
    uint16_t output_count;
    ml_signature_entry_t inputs[8];
    ml_signature_entry_t outputs[8];
} ml_model_signature_t;

/* Quantization configuration */
typedef struct {
    ml_quantization_method_t method;
    uint16_t scale_factor_count;
    uint16_t per_channel;  /* 0=per-tensor, 1=per-channel */
    uint32_t* scale_factors;  /* Quantization scale factors */
    uint32_t* zero_points;    /* Quantization zero points */
} ml_quantization_config_t;

/* Model descriptor */
typedef struct {
    uint16_t model_id;
    uint16_t padding;
    ml_model_format_t format;
    uint32_t layer_count;
    uint32_t param_count;
    ml_layer_t* layers;
    ml_tensor_shape_t input_shape;
    ml_tensor_shape_t output_shape;
    uint32_t version;
    uint32_t size_bytes;
    uint32_t memory_mb;
    ml_model_signature_t* signatures;
    ml_quantization_config_t quant_config;
    uint32_t loaded;
} ml_model_t;

/* Batch request metadata */
typedef struct {
    uint32_t request_id;
    uint16_t batch_index;
    uint16_t padding;
    ml_tensor_shape_t shape;
    uint8_t* input_data;
    uint32_t input_size_bytes;
} ml_batch_item_t;

/* Batch descriptor */
typedef struct {
    uint16_t batch_id;
    uint16_t item_count;
    uint32_t model_id;
    ml_batch_item_t items[ML_MAX_BATCH_SIZE];
    uint32_t execution_plan_id;
    uint32_t fused_layers_mask;  /* Bitmask of fused layer indices */
    uint32_t priority;
    uint32_t deadline_ms;
} ml_batch_t;

/* Execution plan (optimized operation sequence) */
typedef struct {
    uint32_t plan_id;
    uint32_t layer_sequence[ML_MAX_LAYERS];
    uint16_t layer_count;
    uint16_t fusion_count;
    uint32_t estimated_latency_ms;
} ml_execution_plan_t;

/* Framework API */

/* Initialize ML framework */
int32_t ml_framework_init(void);

/* Load model from memory (format auto-detected) */
int32_t ml_model_load(uint16_t model_id, const void* model_data,
                      uint32_t model_size, ml_model_format_t format);

/* Unload model from memory */
int32_t ml_model_unload(uint16_t model_id);

/* Get loaded model */
const ml_model_t* ml_model_get(uint16_t model_id);

/* Parse model signature (input/output shapes) */
int32_t ml_model_parse_signature(uint16_t model_id);

/* Model format and metadata */

/* Get model layer count */
uint32_t ml_model_get_layer_count(uint16_t model_id);

/* Get model parameter count */
uint32_t ml_model_get_param_count(uint16_t model_id);

/* Get model input shape */
int32_t ml_model_get_input_shape(uint16_t model_id,
                                  ml_tensor_shape_t* shape);

/* Get model output shape */
int32_t ml_model_get_output_shape(uint16_t model_id,
                                   ml_tensor_shape_t* shape);

/* Get model memory requirements (MB) */
uint32_t ml_model_get_memory_mb(uint16_t model_id);

/* Quantization */

/* Apply quantization to model (INT8 or INT4) */
int32_t ml_model_quantize(uint16_t model_id,
                          ml_quantization_method_t method);

/* Get quantization configuration */
const ml_quantization_config_t* ml_model_get_quant_config(uint16_t model_id);

/* Execution and inference */

/* Create batch for inference */
int32_t ml_batch_create(uint16_t batch_id, uint32_t model_id,
                        uint32_t priority, uint32_t deadline_ms);

/* Add item to batch */
int32_t ml_batch_add_item(uint16_t batch_id, uint32_t request_id,
                          const void* input_data, uint32_t input_size);

/* Execute batch (synchronous) */
int32_t ml_batch_execute(uint16_t batch_id, void* output_buffer,
                         uint32_t output_size);

/* Execute batch (asynchronous) */
int32_t ml_batch_execute_async(uint16_t batch_id,
                               void (*callback)(uint16_t batch_id, int32_t result));

/* Wait for async execution to complete */
int32_t ml_batch_wait(uint16_t batch_id, uint32_t timeout_ms);

/* Get batch status */
int32_t ml_batch_get_status(uint16_t batch_id);

/* Clear batch */
int32_t ml_batch_clear(uint16_t batch_id);

/* Operator fusion and optimization */

/* Generate optimized execution plan for model */
int32_t ml_execution_plan_generate(uint16_t model_id,
                                   ml_execution_plan_t* plan);

/* Apply execution plan to batch */
int32_t ml_batch_set_execution_plan(uint16_t batch_id,
                                    const ml_execution_plan_t* plan);

/* Get estimated inference latency for batch */
uint32_t ml_batch_estimate_latency(uint16_t batch_id);

/* Dynamic shape support */

/* Update input shape for next inference (for variable-size inputs) */
int32_t ml_model_set_input_shape(uint16_t model_id,
                                  const ml_tensor_shape_t* shape);

/* Validate input shape against model requirements */
int32_t ml_model_validate_input_shape(uint16_t model_id,
                                       const ml_tensor_shape_t* shape);

/* Get minimum/maximum supported batch sizes */
int32_t ml_model_get_batch_size_range(uint16_t model_id,
                                       uint32_t* min_batch, uint32_t* max_batch);

/* Checkpoint and serialization */

/* Save model checkpoint */
int32_t ml_model_save_checkpoint(uint16_t model_id, const char* path);

/* Load model from checkpoint */
int32_t ml_model_load_checkpoint(uint16_t model_id, const char* path);

/* Statistics and diagnostics */

/* Get inference throughput (requests/second) */
uint32_t ml_framework_get_throughput(void);

/* Get average inference latency (milliseconds) */
uint32_t ml_framework_get_avg_latency(void);

/* Get peak GPU memory usage (MB) */
uint32_t ml_framework_get_peak_memory(void);

/* Get total executed batches */
uint32_t ml_framework_get_total_batches(void);

/* Get total executed requests */
uint32_t ml_framework_get_total_requests(void);

/* Print framework status */
void ml_framework_print_status(void);

#endif /* KERNEL_ML_FRAMEWORK_H */
