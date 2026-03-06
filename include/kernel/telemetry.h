#ifndef KERNEL_TELEMETRY_H
#define KERNEL_TELEMETRY_H

#include <stdint.h>

/* Telemetry event types */
typedef enum {
    TELEM_EVENT_NONE = 0,
    TELEM_EVENT_GPU_LAUNCH,
    TELEM_EVENT_GPU_COMPLETE,
    TELEM_EVENT_MODEL_LOAD,
    TELEM_EVENT_MODEL_INFER,
    TELEM_EVENT_MEMORY_ALLOC,
    TELEM_EVENT_THERMAL_SPIKE,
    TELEM_EVENT_TASK_MIGRATE,
    TELEM_EVENT_RESOURCE_CONTENTION,
    TELEM_EVENT_THERMAL_THROTTLE
} telem_event_type_t;

/* Telemetry metric types */
typedef enum {
    METRIC_GPU_UTILIZATION,
    METRIC_GPU_MEMORY,
    METRIC_SYSTEM_TEMP,
    METRIC_INFERENCE_LATENCY,
    METRIC_THROUGHPUT_INFER,
    METRIC_POWER_DRAW,
    METRIC_TASK_QUEUE_DEPTH,
    METRIC_CACHE_HIT_RATE,
    METRIC_PAGE_FAULT_RATE,
    METRIC_THERMAL_HEADROOM
} telem_metric_type_t;

/* Telemetry event structure */
typedef struct {
    uint32_t timestamp;        /* Kernel tick count */
    telem_event_type_t type;
    uint32_t task_id;
    uint32_t gpu_device_id;
    uint32_t data_u32;         /* Generic 32-bit event data */
    uint16_t data_u16;         /* Generic 16-bit event data */
} telem_event_t;

/* Metric sample structure */
typedef struct {
    uint32_t timestamp;
    telem_metric_type_t metric_type;
    uint32_t value;            /* Metric value (0-100000) */
    uint16_t gpu_device_id;
    uint16_t task_id;
} telem_metric_t;

/* Telemetry session for batch collection */
typedef struct {
    uint32_t session_id;
    uint32_t start_time;
    uint32_t end_time;
    uint32_t total_events;
    uint32_t total_metrics;
    int active;
} telem_session_t;

/* Aggregated metrics for observability dashboard */
typedef struct {
    uint32_t avg_gpu_utilization;      /* 0-100 */
    uint32_t peak_gpu_memory;          /* bytes */
    uint32_t avg_inference_latency_us; /* microseconds */
    uint32_t total_inferences;
    uint32_t thermal_events_count;
    uint32_t memory_pressure_events;
    uint32_t task_migrations;
    uint32_t power_draw_mw;            /* milliwatts */
    uint64_t uptime_seconds;
} telem_aggregates_t;

/* Core telemetry APIs */

/**
 * Initialize telemetry subsystem
 */
void telemetry_init(void);

/**
 * Log a telemetry event
 */
int telemetry_log_event(telem_event_type_t type, uint32_t task_id, 
                        uint32_t gpu_device_id, uint32_t data_u32, uint16_t data_u16);

/**
 * Log a metric sample
 */
int telemetry_log_metric(telem_metric_type_t metric, uint32_t value, 
                         uint32_t task_id, uint16_t gpu_device_id);

/**
 * Start a telemetry session for batch collection
 */
uint32_t telemetry_session_start(void);

/**
 * End a telemetry session and return aggregates
 */
int telemetry_session_end(uint32_t session_id, telem_aggregates_t *out_agg);

/**
 * Get current aggregated metrics
 */
telem_aggregates_t *telemetry_get_aggregates(void);

/**
 * Query events by type within a session
 */
int telemetry_query_events(telem_event_type_t type, telem_event_t *events, 
                           uint32_t max_events, uint32_t *out_count);

/**
 * Query metrics by type
 */
int telemetry_query_metrics(telem_metric_type_t metric, telem_metric_t *metrics, 
                            uint32_t max_metrics, uint32_t *out_count);

/**
 * Export telemetry data (for observability dashboards)
 */
int telemetry_export_snapshot(char *buffer, uint32_t buffer_size);

/**
 * Reset telemetry state
 */
void telemetry_reset(void);

/**
 * Get telemetry buffer statistics
 */
uint32_t telemetry_get_event_count(void);
uint32_t telemetry_get_metric_count(void);
uint32_t telemetry_get_buffer_usage_pct(void);

#endif /* KERNEL_TELEMETRY_H */
