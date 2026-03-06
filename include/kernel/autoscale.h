#ifndef KERNEL_AUTOSCALE_H
#define KERNEL_AUTOSCALE_H

#include <stdint.h>

/* Maximum limits */
#define AUTOSCALE_MAX_POLICIES 32
#define AUTOSCALE_MAX_EVENTS 256
#define AUTOSCALE_DEFAULT_COOLDOWN_MS 60000
#define AUTOSCALE_DEFAULT_EVAL_INTERVAL_MS 10000

/* Scaling metric types */
typedef enum {
    SCALE_METRIC_GPU_UTIL = 0,      /* GPU utilization percentage */
    SCALE_METRIC_QUEUE_DEPTH = 1,   /* Request queue depth */
    SCALE_METRIC_LATENCY_P99 = 2,   /* P99 latency in microseconds */
    SCALE_METRIC_CONNECTIONS = 3    /* Active connection count */
} scale_metric_t;

/* Scaling direction */
typedef enum {
    SCALE_NONE = 0,
    SCALE_UP = 1,
    SCALE_DOWN = 2
} scale_direction_t;

/* Scaling policy — defines when and how to scale */
typedef struct {
    uint32_t policy_id;
    uint32_t endpoint_id;           /* Linked serving endpoint */
    scale_metric_t metric;          /* Which metric to evaluate */
    uint32_t scale_up_threshold;    /* Scale up when metric exceeds this */
    uint32_t scale_down_threshold;  /* Scale down when metric drops below */
    uint16_t min_replicas;          /* Minimum replica count */
    uint16_t max_replicas;          /* Maximum replica count */
    uint32_t cooldown_ms;           /* Minimum time between scaling actions */
    uint8_t enabled;
    uint8_t in_use;
} scale_policy_t;

/* Current scaling state per endpoint */
typedef struct {
    uint32_t endpoint_id;
    uint16_t current_replicas;
    uint16_t desired_replicas;
    scale_direction_t last_direction;
    uint32_t last_scale_time;       /* Timestamp of last scaling action */
    uint32_t last_eval_time;        /* Timestamp of last evaluation */
    uint32_t last_metric_value;     /* Last observed metric value */
    uint32_t consecutive_breaches;  /* Consecutive threshold breaches */
} scale_state_t;

/* Scaling event — audit record */
typedef struct {
    uint32_t event_id;
    uint32_t endpoint_id;
    uint32_t policy_id;
    scale_direction_t direction;
    uint16_t old_replicas;
    uint16_t new_replicas;
    uint32_t metric_value;          /* Metric value that triggered scaling */
    scale_metric_t metric_type;
    uint32_t timestamp;
} scale_event_t;

/* Global autoscaler configuration */
typedef struct {
    uint32_t evaluation_interval_ms;
    uint32_t default_cooldown_ms;
    uint8_t enabled;                /* Global enable/disable */
    uint8_t stabilization_window;   /* Consecutive breaches before action */
} autoscale_config_t;

/* Global autoscaler statistics */
typedef struct {
    uint32_t total_evaluations;
    uint32_t total_scale_ups;
    uint32_t total_scale_downs;
    uint32_t total_cooldown_skips;
    uint32_t active_policies;
    uint32_t total_events;
} autoscale_stats_t;

/* Core Auto-Scaling APIs */

/**
 * Initialize autoscaler subsystem
 */
void autoscale_init(void);

/**
 * Create a scaling policy for an endpoint
 */
uint32_t autoscale_create_policy(uint32_t endpoint_id,
                                 scale_metric_t metric,
                                 uint32_t scale_up_threshold,
                                 uint32_t scale_down_threshold,
                                 uint16_t min_replicas,
                                 uint16_t max_replicas);

/**
 * Remove a scaling policy
 */
int autoscale_remove_policy(uint32_t policy_id);

/**
 * Get policy by ID
 */
int autoscale_get_policy(uint32_t policy_id, scale_policy_t *out);

/**
 * Enable/disable a policy
 */
int autoscale_set_policy_enabled(uint32_t policy_id, uint8_t enabled);

/**
 * Set replica range on a policy
 */
int autoscale_set_replica_range(uint32_t policy_id,
                                uint16_t min_replicas,
                                uint16_t max_replicas);

/**
 * Set cooldown period on a policy
 */
int autoscale_set_cooldown(uint32_t policy_id, uint32_t cooldown_ms);

/**
 * Evaluate a single endpoint and apply scaling decision
 */
int autoscale_evaluate(uint32_t endpoint_id);

/**
 * Evaluate all policies
 */
int autoscale_evaluate_all(void);

/**
 * Get current scaling state for endpoint
 */
int autoscale_get_state(uint32_t endpoint_id, scale_state_t *out);

/**
 * Manually set replica count (overrides autoscaler)
 */
int autoscale_set_replicas(uint32_t endpoint_id, uint16_t replicas);

/**
 * Get scaling event history
 */
uint32_t autoscale_get_events(scale_event_t *events, uint32_t max_events);

/**
 * Get global autoscaler statistics
 */
autoscale_stats_t *autoscale_get_stats(void);

/**
 * Set global autoscaler configuration
 */
int autoscale_set_config(const autoscale_config_t *config);

/**
 * Get global autoscaler configuration
 */
int autoscale_get_config(autoscale_config_t *out);

/**
 * Enable/disable global autoscaling
 */
int autoscale_set_enabled(uint8_t enabled);

/**
 * Get number of active policies
 */
uint32_t autoscale_get_policy_count(void);

#endif /* KERNEL_AUTOSCALE_H */
