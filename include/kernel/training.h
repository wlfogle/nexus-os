#ifndef KERNEL_TRAINING_H
#define KERNEL_TRAINING_H

#include <stdint.h>

/* Training job states */
typedef enum {
    JOB_CREATED = 0,
    JOB_QUEUED = 1,
    JOB_RUNNING = 2,
    JOB_PAUSED = 3,
    JOB_CHECKPOINTING = 4,
    JOB_COMPLETED = 5,
    JOB_FAILED = 6
} job_state_t;

/* Checkpoint metadata */
typedef struct {
    uint32_t checkpoint_id;
    uint32_t job_id;
    uint32_t epoch;
    uint32_t step;
    uint64_t timestamp;
    uint32_t size_bytes;
    uint8_t valid;
} checkpoint_t;

/* Training metrics */
typedef struct {
    uint32_t epoch;
    uint32_t step;
    float loss;
    float accuracy;
    float learning_rate;
    uint32_t batch_size;
    uint32_t samples_processed;
    uint64_t elapsed_time_ms;
} training_metrics_t;

/* Training job */
typedef struct {
    uint32_t job_id;
    uint32_t model_id;
    uint32_t num_gpus;
    uint32_t batch_size;
    uint32_t num_epochs;
    job_state_t state;
    uint32_t current_epoch;
    uint32_t current_step;
    uint64_t created_time;
    uint64_t start_time;
    uint64_t end_time;
    uint32_t progress_pct;
} training_job_t;

/* Job statistics */
typedef struct {
    uint32_t total_jobs;
    uint32_t running_jobs;
    uint32_t completed_jobs;
    uint32_t failed_jobs;
    uint32_t total_checkpoints;
    uint64_t total_samples_processed;
    uint32_t avg_job_duration_ms;
} job_stats_t;

/* Core Training Job APIs */

/**
 * Initialize training job orchestration
 */
void training_init(void);

/**
 * Create a training job
 */
uint32_t training_create_job(uint32_t model_id, uint32_t num_gpus,
                            uint32_t batch_size, uint32_t num_epochs);

/**
 * Start/resume training job
 */
int training_start_job(uint32_t job_id);

/**
 * Pause training job
 */
int training_pause_job(uint32_t job_id);

/**
 * Resume paused training job
 */
int training_resume_job(uint32_t job_id);

/**
 * Stop training job
 */
int training_stop_job(uint32_t job_id);

/**
 * Get job status
 */
int training_get_job(uint32_t job_id, training_job_t *out);

/**
 * Update training progress
 */
int training_update_progress(uint32_t job_id, uint32_t epoch, uint32_t step,
                            float loss, float accuracy, uint32_t samples);

/**
 * Get training metrics
 */
int training_get_metrics(uint32_t job_id, training_metrics_t *out);

/**
 * Checkpoint model state
 */
uint32_t training_checkpoint(uint32_t job_id, const uint8_t *state_data,
                            uint32_t state_size);

/**
 * Restore from checkpoint
 */
int training_restore(uint32_t job_id, uint32_t checkpoint_id,
                    uint8_t *out_state, uint32_t max_size);

/**
 * List checkpoints for job
 */
uint32_t training_list_checkpoints(uint32_t job_id, checkpoint_t *out,
                                  uint32_t max_checkpoints);

/**
 * Delete checkpoint
 */
int training_delete_checkpoint(uint32_t checkpoint_id);

/**
 * Get checkpoint info
 */
int training_get_checkpoint(uint32_t checkpoint_id, checkpoint_t *out);

/**
 * Handle training failure (fault tolerance)
 */
int training_handle_failure(uint32_t job_id, uint32_t failed_gpu);

/**
 * Get job statistics
 */
job_stats_t *training_get_stats(void);

/**
 * List all jobs
 */
uint32_t training_list_jobs(training_job_t *jobs, uint32_t max_jobs);

/**
 * Get job count
 */
uint32_t training_get_job_count(void);

/**
 * Set learning rate schedule
 */
int training_set_lr_schedule(uint32_t job_id, float initial_lr, float decay_rate);

/**
 * Get current learning rate
 */
float training_get_learning_rate(uint32_t job_id);

#endif /* KERNEL_TRAINING_H */
