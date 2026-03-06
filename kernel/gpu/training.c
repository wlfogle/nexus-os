#include "../../include/kernel/training.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_JOBS 32
#define MAX_CHECKPOINTS 128

typedef struct {
    training_job_t job;
    training_metrics_t metrics;
    float learning_rate;
    float lr_decay_rate;
    int in_use;
} job_slot_t;

typedef struct {
    checkpoint_t checkpoint;
    uint8_t *state_data;
    int in_use;
} checkpoint_slot_t;

static job_slot_t jobs[MAX_JOBS];
static checkpoint_slot_t checkpoints[MAX_CHECKPOINTS];
static job_stats_t stats = {0};
static uint32_t job_id_counter = 1;
static uint32_t checkpoint_id_counter = 1;

void training_init(void)
{
    memset(jobs, 0, sizeof(jobs));
    memset(checkpoints, 0, sizeof(checkpoints));
    memset(&stats, 0, sizeof(job_stats_t));
    
    job_id_counter = 1;
    checkpoint_id_counter = 1;
    
    serial_puts("[training] Training job orchestration initialized\n");
}

uint32_t training_create_job(uint32_t model_id, uint32_t num_gpus,
                            uint32_t batch_size, uint32_t num_epochs)
{
    if (model_id == 0 || num_gpus == 0 || batch_size == 0 || num_epochs == 0) return 0;
    
    int free_idx = -1;
    for (int i = 0; i < MAX_JOBS; i++) {
        if (!jobs[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;
    
    job_slot_t *slot = &jobs[free_idx];
    slot->job.job_id = job_id_counter++;
    slot->job.model_id = model_id;
    slot->job.num_gpus = num_gpus;
    slot->job.batch_size = batch_size;
    slot->job.num_epochs = num_epochs;
    slot->job.state = JOB_CREATED;
    slot->job.current_epoch = 0;
    slot->job.current_step = 0;
    slot->job.progress_pct = 0;
    slot->learning_rate = 0.001f;
    slot->lr_decay_rate = 0.1f;
    slot->in_use = 1;
    
    stats.total_jobs++;
    
    serial_printf("[training] Created job %d (model=%d, gpus=%d, epochs=%d)\n",
                 slot->job.job_id, model_id, num_gpus, num_epochs);
    
    return slot->job.job_id;
}

int training_start_job(uint32_t job_id)
{
    if (job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            if (jobs[i].job.state == JOB_CREATED) {
                jobs[i].job.state = JOB_RUNNING;
                jobs[i].job.start_time = 0;  /* Would be set from timer */
                stats.running_jobs++;
                
                serial_printf("[training] Started job %d\n", job_id);
                return 0;
            }
        }
    }
    
    return -1;
}

int training_pause_job(uint32_t job_id)
{
    if (job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            if (jobs[i].job.state == JOB_RUNNING) {
                jobs[i].job.state = JOB_PAUSED;
                stats.running_jobs--;
                return 0;
            }
        }
    }
    
    return -1;
}

int training_resume_job(uint32_t job_id)
{
    if (job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            if (jobs[i].job.state == JOB_PAUSED) {
                jobs[i].job.state = JOB_RUNNING;
                stats.running_jobs++;
                return 0;
            }
        }
    }
    
    return -1;
}

int training_stop_job(uint32_t job_id)
{
    if (job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            jobs[i].job.state = JOB_COMPLETED;
            jobs[i].job.end_time = 0;
            stats.completed_jobs++;
            if (jobs[i].job.state == JOB_RUNNING) stats.running_jobs--;
            return 0;
        }
    }
    
    return -1;
}

int training_get_job(uint32_t job_id, training_job_t *out)
{
    if (!out || job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            memcpy(out, &jobs[i].job, sizeof(training_job_t));
            return 0;
        }
    }
    
    return -1;
}

int training_update_progress(uint32_t job_id, uint32_t epoch, uint32_t step,
                            float loss, float accuracy, uint32_t samples)
{
    if (job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            jobs[i].job.current_epoch = epoch;
            jobs[i].job.current_step = step;
            jobs[i].job.progress_pct = (epoch * 100) / jobs[i].job.num_epochs;
            
            jobs[i].metrics.epoch = epoch;
            jobs[i].metrics.step = step;
            jobs[i].metrics.loss = loss;
            jobs[i].metrics.accuracy = accuracy;
            jobs[i].metrics.learning_rate = jobs[i].learning_rate;
            jobs[i].metrics.batch_size = jobs[i].job.batch_size;
            jobs[i].metrics.samples_processed += samples;
            
            stats.total_samples_processed += samples;
            
            return 0;
        }
    }
    
    return -1;
}

int training_get_metrics(uint32_t job_id, training_metrics_t *out)
{
    if (!out || job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            memcpy(out, &jobs[i].metrics, sizeof(training_metrics_t));
            return 0;
        }
    }
    
    return -1;
}

uint32_t training_checkpoint(uint32_t job_id, const uint8_t *state_data,
                            uint32_t state_size)
{
    if (job_id == 0 || !state_data || state_size == 0) return 0;
    
    int free_idx = -1;
    for (int i = 0; i < MAX_CHECKPOINTS; i++) {
        if (!checkpoints[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;
    
    checkpoint_slot_t *slot = &checkpoints[free_idx];
    slot->checkpoint.checkpoint_id = checkpoint_id_counter++;
    slot->checkpoint.job_id = job_id;
    slot->checkpoint.timestamp = 0;
    slot->checkpoint.size_bytes = state_size;
    slot->checkpoint.valid = 1;
    
    /* Get current job state */
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            slot->checkpoint.epoch = jobs[i].job.current_epoch;
            slot->checkpoint.step = jobs[i].job.current_step;
            break;
        }
    }
    
    slot->state_data = (uint8_t *)state_data;
    slot->in_use = 1;
    
    stats.total_checkpoints++;
    
    serial_printf("[training] Checkpointed job %d (ckpt=%d, size=%d)\n",
                 job_id, slot->checkpoint.checkpoint_id, state_size);
    
    return slot->checkpoint.checkpoint_id;
}

int training_restore(uint32_t job_id, uint32_t checkpoint_id,
                    uint8_t *out_state, uint32_t max_size)
{
    if (job_id == 0 || checkpoint_id == 0 || !out_state) return -1;
    
    for (int i = 0; i < MAX_CHECKPOINTS; i++) {
        if (checkpoints[i].in_use && 
            checkpoints[i].checkpoint.checkpoint_id == checkpoint_id &&
            checkpoints[i].checkpoint.job_id == job_id) {
            
            if (checkpoints[i].checkpoint.size_bytes > max_size) return -1;
            
            memcpy(out_state, checkpoints[i].state_data, 
                   checkpoints[i].checkpoint.size_bytes);
            
            return checkpoints[i].checkpoint.size_bytes;
        }
    }
    
    return -1;
}

uint32_t training_list_checkpoints(uint32_t job_id, checkpoint_t *out,
                                  uint32_t max_checkpoints)
{
    if (job_id == 0 || !out || max_checkpoints == 0) return 0;
    
    uint32_t count = 0;
    for (int i = 0; i < MAX_CHECKPOINTS && count < max_checkpoints; i++) {
        if (checkpoints[i].in_use && 
            checkpoints[i].checkpoint.job_id == job_id) {
            memcpy(&out[count], &checkpoints[i].checkpoint, sizeof(checkpoint_t));
            count++;
        }
    }
    
    return count;
}

int training_delete_checkpoint(uint32_t checkpoint_id)
{
    if (checkpoint_id == 0) return -1;
    
    for (int i = 0; i < MAX_CHECKPOINTS; i++) {
        if (checkpoints[i].in_use && 
            checkpoints[i].checkpoint.checkpoint_id == checkpoint_id) {
            checkpoints[i].in_use = 0;
            return 0;
        }
    }
    
    return -1;
}

int training_get_checkpoint(uint32_t checkpoint_id, checkpoint_t *out)
{
    if (!out || checkpoint_id == 0) return -1;
    
    for (int i = 0; i < MAX_CHECKPOINTS; i++) {
        if (checkpoints[i].in_use && 
            checkpoints[i].checkpoint.checkpoint_id == checkpoint_id) {
            memcpy(out, &checkpoints[i].checkpoint, sizeof(checkpoint_t));
            return 0;
        }
    }
    
    return -1;
}

int training_handle_failure(uint32_t job_id, uint32_t failed_gpu)
{
    if (job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            jobs[i].job.state = JOB_FAILED;
            stats.failed_jobs++;
            stats.running_jobs--;
            
            serial_printf("[training] Job %d failed on GPU %d\n", job_id, failed_gpu);
            return 0;
        }
    }
    
    return -1;
}

job_stats_t *training_get_stats(void)
{
    return &stats;
}

uint32_t training_list_jobs(training_job_t *jobs_out, uint32_t max_jobs)
{
    if (!jobs_out || max_jobs == 0) return 0;
    
    uint32_t count = 0;
    for (int i = 0; i < MAX_JOBS && count < max_jobs; i++) {
        if (jobs[i].in_use) {
            memcpy(&jobs_out[count], &jobs[i].job, sizeof(training_job_t));
            count++;
        }
    }
    
    return count;
}

uint32_t training_get_job_count(void)
{
    uint32_t count = 0;
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use) count++;
    }
    return count;
}

int training_set_lr_schedule(uint32_t job_id, float initial_lr, float decay_rate)
{
    if (job_id == 0) return -1;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            jobs[i].learning_rate = initial_lr;
            jobs[i].lr_decay_rate = decay_rate;
            return 0;
        }
    }
    
    return -1;
}

float training_get_learning_rate(uint32_t job_id)
{
    if (job_id == 0) return 0.0f;
    
    for (int i = 0; i < MAX_JOBS; i++) {
        if (jobs[i].in_use && jobs[i].job.job_id == job_id) {
            /* Apply decay based on current epoch */
            float current_lr = jobs[i].learning_rate;
            for (uint32_t e = 0; e < jobs[i].job.current_epoch; e++) {
                current_lr *= jobs[i].lr_decay_rate;
            }
            return current_lr;
        }
    }
    
    return 0.0f;
}
