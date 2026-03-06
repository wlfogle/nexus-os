#ifndef KERNEL_PIPELINE_H
#define KERNEL_PIPELINE_H

#include <stdint.h>

/* Maximum limits */
#define PIPELINE_MAX_PIPELINES 16
#define PIPELINE_MAX_STAGES 16
#define PIPELINE_MAX_EXECUTIONS 64
#define PIPELINE_MAX_NAME_LEN 64
#define PIPELINE_MAX_STAGE_DEPS 4
#define PIPELINE_INTERMEDIATE_BUF_SIZE 4096

/* Pipeline stage status */
typedef enum {
    STAGE_IDLE = 0,
    STAGE_PENDING = 1,
    STAGE_RUNNING = 2,
    STAGE_COMPLETED = 3,
    STAGE_FAILED = 4,
    STAGE_SKIPPED = 5
} stage_status_t;

/* Pipeline execution status */
typedef enum {
    PIPELINE_EXEC_CREATED = 0,
    PIPELINE_EXEC_RUNNING = 1,
    PIPELINE_EXEC_COMPLETED = 2,
    PIPELINE_EXEC_FAILED = 3,
    PIPELINE_EXEC_CANCELLED = 4
} pipeline_exec_status_t;

/* Pipeline stage — single node in the DAG */
typedef struct {
    uint32_t stage_id;
    uint32_t model_id;               /* Model to execute at this stage */
    uint32_t depends_on[PIPELINE_MAX_STAGE_DEPS]; /* Stage IDs this depends on */
    uint8_t dependency_count;
    char name[32];
    uint8_t in_use;
} pipeline_stage_t;

/* Pipeline definition */
typedef struct {
    uint32_t pipeline_id;
    char name[PIPELINE_MAX_NAME_LEN];
    pipeline_stage_t stages[PIPELINE_MAX_STAGES];
    uint32_t stage_count;
    uint32_t version;
    uint32_t created_time;
    uint8_t in_use;
} pipeline_t;

/* Per-stage execution state (during pipeline run) */
typedef struct {
    uint32_t stage_id;
    stage_status_t status;
    uint32_t latency_us;
    uint32_t output_size;
    uint8_t output_buffer[PIPELINE_INTERMEDIATE_BUF_SIZE];
} stage_exec_state_t;

/* Pipeline execution context */
typedef struct {
    uint32_t exec_id;
    uint32_t pipeline_id;
    pipeline_exec_status_t status;
    uint32_t current_stage_idx;
    uint32_t stages_completed;
    uint32_t stages_failed;
    uint32_t total_latency_us;
    stage_exec_state_t stage_states[PIPELINE_MAX_STAGES];
    uint8_t final_output[PIPELINE_INTERMEDIATE_BUF_SIZE];
    uint32_t final_output_size;
    uint32_t start_time;
    uint32_t end_time;
} pipeline_exec_t;

/* Per-pipeline statistics */
typedef struct {
    uint32_t pipeline_id;
    uint64_t total_executions;
    uint64_t successful_executions;
    uint64_t failed_executions;
    uint32_t avg_latency_us;
    uint32_t max_latency_us;
    uint32_t min_latency_us;
    uint32_t avg_stages_per_exec;
} pipeline_stats_t;

/* Global pipeline statistics */
typedef struct {
    uint32_t active_pipelines;
    uint32_t active_executions;
    uint64_t total_executions;
    uint64_t total_stage_runs;
    uint32_t avg_pipeline_latency_us;
} pipeline_global_stats_t;

/* Core Pipeline Orchestration APIs */

/**
 * Initialize pipeline subsystem
 */
void pipeline_init(void);

/**
 * Create an empty pipeline
 */
uint32_t pipeline_create(const char *name);

/**
 * Add a stage to a pipeline
 * depends_on_stage: stage ID this depends on (0 = entry stage, no dependency)
 */
uint32_t pipeline_add_stage(uint32_t pipeline_id, uint32_t model_id,
                            const char *stage_name,
                            uint32_t depends_on_stage);

/**
 * Add a stage with multiple dependencies
 */
uint32_t pipeline_add_stage_multi_dep(uint32_t pipeline_id, uint32_t model_id,
                                      const char *stage_name,
                                      const uint32_t *depends_on,
                                      uint8_t dep_count);

/**
 * Remove a stage from pipeline
 */
int pipeline_remove_stage(uint32_t pipeline_id, uint32_t stage_id);

/**
 * Validate pipeline DAG (no cycles, all deps exist, has entry point)
 */
int pipeline_validate(uint32_t pipeline_id);

/**
 * Execute a pipeline end-to-end
 * Returns execution ID (0 on failure)
 */
uint32_t pipeline_execute(uint32_t pipeline_id, const uint8_t *input_data,
                          uint32_t input_size);

/**
 * Get execution status
 */
int pipeline_get_status(uint32_t exec_id, pipeline_exec_status_t *out);

/**
 * Get execution details
 */
int pipeline_get_execution(uint32_t exec_id, pipeline_exec_t *out);

/**
 * Get final output from completed execution
 */
int pipeline_get_output(uint32_t exec_id, uint8_t *output_data,
                        uint32_t max_size, uint32_t *actual_size);

/**
 * Cancel a running execution
 */
int pipeline_cancel_execution(uint32_t exec_id);

/**
 * Get pipeline definition
 */
int pipeline_get(uint32_t pipeline_id, pipeline_t *out);

/**
 * List all pipelines
 */
uint32_t pipeline_list(pipeline_t *pipelines, uint32_t max_pipelines);

/**
 * Get per-pipeline statistics
 */
int pipeline_get_stats(uint32_t pipeline_id, pipeline_stats_t *out);

/**
 * Get global pipeline statistics
 */
pipeline_global_stats_t *pipeline_get_global_stats(void);

/**
 * Destroy a pipeline
 */
int pipeline_destroy(uint32_t pipeline_id);

/**
 * Get number of active pipelines
 */
uint32_t pipeline_get_count(void);

/**
 * Get stage count for pipeline
 */
uint32_t pipeline_get_stage_count(uint32_t pipeline_id);

#endif /* KERNEL_PIPELINE_H */
