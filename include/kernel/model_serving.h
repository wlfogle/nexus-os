#ifndef KERNEL_MODEL_SERVING_H
#define KERNEL_MODEL_SERVING_H

#include <stdint.h>

/* Maximum limits */
#define SERVING_MAX_ENDPOINTS 32
#define SERVING_MAX_CONNECTIONS 128
#define SERVING_MAX_NAME_LEN 64
#define SERVING_REQUEST_TIMEOUT_MS 30000
#define SERVING_MAX_QUEUE_DEPTH 256

/* Serving status codes */
typedef enum {
    SERVING_OK = 0,
    SERVING_ERROR_NOT_FOUND = 1,
    SERVING_ERROR_TIMEOUT = 2,
    SERVING_ERROR_OVERLOADED = 3,
    SERVING_ERROR_MODEL_FAILED = 4,
    SERVING_ERROR_INVALID_INPUT = 5,
    SERVING_ERROR_INTERNAL = 6
} serving_status_t;

/* Endpoint state */
typedef enum {
    ENDPOINT_INACTIVE = 0,
    ENDPOINT_ACTIVE = 1,
    ENDPOINT_DRAINING = 2
} endpoint_state_t;

/* Traffic split configuration for A/B testing */
typedef struct {
    uint32_t model_id_a;        /* Primary model version */
    uint32_t model_id_b;        /* Canary model version */
    uint8_t split_pct_b;        /* Percentage of traffic to model B (0-100) */
    uint8_t enabled;
} traffic_split_t;

/* Per-endpoint latency histogram (p50/p95/p99) */
typedef struct {
    uint32_t p50_us;            /* 50th percentile latency (microseconds) */
    uint32_t p95_us;            /* 95th percentile latency */
    uint32_t p99_us;            /* 99th percentile latency */
    uint32_t avg_us;            /* Mean latency */
    uint32_t max_us;            /* Max observed latency */
} latency_histogram_t;

/* Serving endpoint — maps a model to a TCP port */
typedef struct {
    uint32_t endpoint_id;
    uint32_t model_id;          /* Primary model */
    uint16_t port;              /* TCP listen port */
    char name[SERVING_MAX_NAME_LEN];
    endpoint_state_t state;
    traffic_split_t traffic_split;
    uint32_t created_time;
    uint32_t active_connections;
    uint32_t queue_depth;
    uint8_t in_use;
} serving_endpoint_t;

/* Incoming inference request (parsed from TCP payload) */
typedef struct {
    uint32_t request_id;
    uint32_t endpoint_id;
    uint32_t model_id;          /* Resolved model (after A/B split) */
    const uint8_t *input_data;
    uint32_t input_size;
    uint32_t timeout_ms;
    uint32_t received_time;
    uint8_t priority;
} serving_request_t;

/* Inference response */
typedef struct {
    uint32_t request_id;
    serving_status_t status;
    uint8_t *output_data;
    uint32_t output_size;
    uint32_t latency_us;
    uint32_t model_id;          /* Which model version served this */
} serving_response_t;

/* Global serving configuration */
typedef struct {
    uint32_t max_connections;
    uint32_t request_timeout_ms;
    uint32_t max_queue_depth;
    uint8_t health_check_enabled;
} serving_config_t;

/* Per-endpoint statistics */
typedef struct {
    uint32_t endpoint_id;
    uint64_t total_requests;
    uint64_t successful_requests;
    uint64_t failed_requests;
    uint64_t timed_out_requests;
    uint32_t requests_per_sec;  /* Estimated QPS */
    latency_histogram_t latency;
    uint32_t model_a_requests;
    uint32_t model_b_requests;
    uint32_t active_connections;
    uint32_t queue_depth;
} serving_endpoint_stats_t;

/* Global serving statistics */
typedef struct {
    uint32_t active_endpoints;
    uint64_t total_requests_all;
    uint64_t total_errors_all;
    uint32_t total_connections;
    uint32_t peak_connections;
    uint32_t avg_latency_us;
} serving_global_stats_t;

/* Core Model Serving APIs */

/**
 * Initialize model serving subsystem
 */
void model_serving_init(void);

/**
 * Register a serving endpoint (bind model to TCP port)
 */
uint32_t serving_register_endpoint(uint32_t model_id, uint16_t port,
                                   const char *name);

/**
 * Unregister serving endpoint
 */
int serving_unregister_endpoint(uint32_t endpoint_id);

/**
 * Get endpoint by ID
 */
int serving_get_endpoint(uint32_t endpoint_id, serving_endpoint_t *out);

/**
 * List all active endpoints
 */
uint32_t serving_list_endpoints(serving_endpoint_t *endpoints,
                                uint32_t max_endpoints);

/**
 * Handle incoming request on endpoint
 */
int serving_handle_request(uint32_t endpoint_id, const uint8_t *raw_data,
                           uint32_t len, serving_response_t *response);

/**
 * Set A/B traffic split on endpoint
 */
int serving_set_traffic_split(uint32_t endpoint_id, uint32_t model_id_a,
                              uint32_t model_id_b, uint8_t split_pct_b);

/**
 * Disable traffic split (route all traffic to primary model)
 */
int serving_disable_traffic_split(uint32_t endpoint_id);

/**
 * Health check — returns 0 if endpoint healthy, <0 otherwise
 */
int serving_health_check(uint32_t endpoint_id);

/**
 * Set endpoint state (active, draining)
 */
int serving_set_endpoint_state(uint32_t endpoint_id, endpoint_state_t state);

/**
 * Get per-endpoint statistics
 */
int serving_get_endpoint_stats(uint32_t endpoint_id,
                               serving_endpoint_stats_t *out);

/**
 * Get global serving statistics
 */
serving_global_stats_t *serving_get_global_stats(void);

/**
 * Set global serving configuration
 */
int serving_set_config(const serving_config_t *config);

/**
 * Get global serving configuration
 */
int serving_get_config(serving_config_t *out);

/**
 * Get number of active endpoints
 */
uint32_t serving_get_endpoint_count(void);

#endif /* KERNEL_MODEL_SERVING_H */
