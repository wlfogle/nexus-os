#include "../../include/kernel/pipeline.h"
#include "../../include/kernel/serial.h"
#include <string.h>

/* Internal pipeline storage */
typedef struct {
    pipeline_t pipeline;
    pipeline_stats_t stats;
} pipeline_entry_t;

static pipeline_entry_t pipelines[PIPELINE_MAX_PIPELINES];
static pipeline_exec_t executions[PIPELINE_MAX_EXECUTIONS];
static pipeline_global_stats_t global_stats = {0};
static uint32_t pipeline_id_counter = 1;
static uint32_t stage_id_counter = 1;
static uint32_t exec_id_counter = 1;

/* Simple LCG for simulated latency */
static uint32_t pipe_rng = 54321;

static uint32_t pipe_random(void)
{
    pipe_rng = pipe_rng * 1103515245 + 12345;
    return (pipe_rng >> 16) & 0x7FFF;
}

static pipeline_entry_t *find_pipeline(uint32_t pipeline_id)
{
    for (int i = 0; i < PIPELINE_MAX_PIPELINES; i++) {
        if (pipelines[i].pipeline.in_use &&
            pipelines[i].pipeline.pipeline_id == pipeline_id)
            return &pipelines[i];
    }
    return (void *)0;
}

static pipeline_exec_t *find_execution(uint32_t exec_id)
{
    for (int i = 0; i < PIPELINE_MAX_EXECUTIONS; i++) {
        if (executions[i].exec_id == exec_id &&
            executions[i].status != PIPELINE_EXEC_CREATED)
            return &executions[i];
    }
    /* Also check CREATED status with valid exec_id */
    for (int i = 0; i < PIPELINE_MAX_EXECUTIONS; i++) {
        if (executions[i].exec_id == exec_id)
            return &executions[i];
    }
    return (void *)0;
}

static pipeline_stage_t *find_stage_in_pipeline(pipeline_t *p, uint32_t stage_id)
{
    for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (p->stages[i].in_use && p->stages[i].stage_id == stage_id)
            return &p->stages[i];
    }
    return (void *)0;
}

/* Cycle detection using DFS with coloring (0=white, 1=gray, 2=black) */
static int dfs_has_cycle(pipeline_t *p, uint32_t stage_id, uint8_t *color)
{
    /* Find index of this stage */
    int idx = -1;
    for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (p->stages[i].in_use && p->stages[i].stage_id == stage_id) {
            idx = (int)i;
            break;
        }
    }
    if (idx < 0) return 0;

    color[idx] = 1;  /* Gray — currently visiting */

    /* Visit dependencies (which are "predecessors" in our DAG) */
    pipeline_stage_t *stage = &p->stages[idx];
    for (uint8_t d = 0; d < stage->dependency_count; d++) {
        uint32_t dep_id = stage->depends_on[d];
        if (dep_id == 0) continue;

        /* Find dep index */
        int dep_idx = -1;
        for (uint32_t j = 0; j < PIPELINE_MAX_STAGES; j++) {
            if (p->stages[j].in_use && p->stages[j].stage_id == dep_id) {
                dep_idx = (int)j;
                break;
            }
        }
        if (dep_idx < 0) continue;

        if (color[dep_idx] == 1) return 1;  /* Back edge = cycle */
        if (color[dep_idx] == 0) {
            if (dfs_has_cycle(p, dep_id, color)) return 1;
        }
    }

    color[idx] = 2;  /* Black — done */
    return 0;
}

/* Topological sort into order array; returns count or -1 on cycle */
static int topological_sort(pipeline_t *p, uint32_t *order, uint32_t max_order)
{
    /* Simple Kahn's algorithm */
    uint32_t in_degree[PIPELINE_MAX_STAGES];
    uint8_t processed[PIPELINE_MAX_STAGES];
    memset(in_degree, 0, sizeof(in_degree));
    memset(processed, 0, sizeof(processed));

    /* Compute in-degree for each stage */
    for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (!p->stages[i].in_use) continue;
        in_degree[i] = p->stages[i].dependency_count;
    }

    uint32_t count = 0;
    int progress = 1;

    while (progress && count < max_order) {
        progress = 0;
        for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
            if (!p->stages[i].in_use) continue;
            if (processed[i]) continue;
            if (in_degree[i] == 0) {
                order[count++] = i;
                processed[i] = 1;
                progress = 1;

                /* Decrease in-degree for dependents */
                uint32_t sid = p->stages[i].stage_id;
                for (uint32_t j = 0; j < PIPELINE_MAX_STAGES; j++) {
                    if (!p->stages[j].in_use || processed[j]) continue;
                    for (uint8_t d = 0; d < p->stages[j].dependency_count; d++) {
                        if (p->stages[j].depends_on[d] == sid) {
                            if (in_degree[j] > 0) in_degree[j]--;
                        }
                    }
                }
            }
        }
    }

    /* If not all stages processed, there's a cycle */
    if (count != p->stage_count) return -1;

    return (int)count;
}

/* Check if all dependencies of a stage are completed */
static int all_deps_completed(pipeline_t *p, pipeline_stage_t *stage,
                               stage_exec_state_t *stage_states)
{
    for (uint8_t d = 0; d < stage->dependency_count; d++) {
        uint32_t dep_id = stage->depends_on[d];
        if (dep_id == 0) continue;

        /* Find the dep's exec state */
        for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
            if (p->stages[i].in_use && p->stages[i].stage_id == dep_id) {
                if (stage_states[i].status != STAGE_COMPLETED)
                    return 0;
                break;
            }
        }
    }
    return 1;
}

void pipeline_init(void)
{
    memset(pipelines, 0, sizeof(pipelines));
    memset(executions, 0, sizeof(executions));
    memset(&global_stats, 0, sizeof(pipeline_global_stats_t));

    pipeline_id_counter = 1;
    stage_id_counter = 1;
    exec_id_counter = 1;

    serial_puts("[pipeline] Pipeline orchestration initialized\n");
}

uint32_t pipeline_create(const char *name)
{
    if (!name) return 0;

    int free_idx = -1;
    for (int i = 0; i < PIPELINE_MAX_PIPELINES; i++) {
        if (!pipelines[i].pipeline.in_use) {
            free_idx = i;
            break;
        }
    }

    if (free_idx < 0) return 0;

    pipeline_entry_t *pe = &pipelines[free_idx];
    memset(pe, 0, sizeof(pipeline_entry_t));

    pe->pipeline.pipeline_id = pipeline_id_counter++;
    pe->pipeline.stage_count = 0;
    pe->pipeline.version = 1;
    pe->pipeline.in_use = 1;

    uint32_t name_len = 0;
    while (name[name_len] && name_len < PIPELINE_MAX_NAME_LEN - 1) name_len++;
    memcpy(pe->pipeline.name, name, name_len);
    pe->pipeline.name[name_len] = '\0';

    pe->stats.pipeline_id = pe->pipeline.pipeline_id;
    pe->stats.min_latency_us = 0xFFFFFFFF;

    global_stats.active_pipelines++;

    serial_printf("[pipeline] Pipeline '%s' created (id=%u)\n",
                  pe->pipeline.name, pe->pipeline.pipeline_id);
    return pe->pipeline.pipeline_id;
}

uint32_t pipeline_add_stage(uint32_t pipeline_id, uint32_t model_id,
                            const char *stage_name,
                            uint32_t depends_on_stage)
{
    return pipeline_add_stage_multi_dep(pipeline_id, model_id, stage_name,
                                        depends_on_stage > 0 ? &depends_on_stage : (void *)0,
                                        depends_on_stage > 0 ? 1 : 0);
}

uint32_t pipeline_add_stage_multi_dep(uint32_t pipeline_id, uint32_t model_id,
                                      const char *stage_name,
                                      const uint32_t *depends_on,
                                      uint8_t dep_count)
{
    if (model_id == 0) return 0;
    if (dep_count > PIPELINE_MAX_STAGE_DEPS) return 0;

    pipeline_entry_t *pe = find_pipeline(pipeline_id);
    if (!pe) return 0;

    if (pe->pipeline.stage_count >= PIPELINE_MAX_STAGES) return 0;

    /* Validate dependencies exist */
    for (uint8_t d = 0; d < dep_count; d++) {
        if (depends_on[d] == 0) continue;
        if (!find_stage_in_pipeline(&pe->pipeline, depends_on[d])) {
            serial_printf("[pipeline] Dependency stage %u not found\n",
                          depends_on[d]);
            return 0;
        }
    }

    /* Find free stage slot */
    int free_idx = -1;
    for (int i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (!pe->pipeline.stages[i].in_use) {
            free_idx = i;
            break;
        }
    }

    if (free_idx < 0) return 0;

    pipeline_stage_t *stage = &pe->pipeline.stages[free_idx];
    memset(stage, 0, sizeof(pipeline_stage_t));

    stage->stage_id = stage_id_counter++;
    stage->model_id = model_id;
    stage->in_use = 1;

    for (uint8_t d = 0; d < dep_count; d++) {
        stage->depends_on[d] = depends_on[d];
    }
    stage->dependency_count = dep_count;

    if (stage_name) {
        uint32_t sn_len = 0;
        while (stage_name[sn_len] && sn_len < 31) sn_len++;
        memcpy(stage->name, stage_name, sn_len);
        stage->name[sn_len] = '\0';
    }

    pe->pipeline.stage_count++;
    pe->pipeline.version++;

    serial_printf("[pipeline] Stage '%s' added to pipeline %u "
                  "(stage_id=%u model=%u deps=%u)\n",
                  stage->name, pipeline_id, stage->stage_id,
                  model_id, (unsigned)dep_count);

    return stage->stage_id;
}

int pipeline_remove_stage(uint32_t pipeline_id, uint32_t stage_id)
{
    pipeline_entry_t *pe = find_pipeline(pipeline_id);
    if (!pe) return -1;

    pipeline_stage_t *stage = find_stage_in_pipeline(&pe->pipeline, stage_id);
    if (!stage) return -1;

    /* Check no other stage depends on this one */
    for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (!pe->pipeline.stages[i].in_use) continue;
        for (uint8_t d = 0; d < pe->pipeline.stages[i].dependency_count; d++) {
            if (pe->pipeline.stages[i].depends_on[d] == stage_id) {
                serial_printf("[pipeline] Cannot remove stage %u: "
                              "stage %u depends on it\n",
                              stage_id, pe->pipeline.stages[i].stage_id);
                return -2;  /* Has dependents */
            }
        }
    }

    stage->in_use = 0;
    if (pe->pipeline.stage_count > 0)
        pe->pipeline.stage_count--;
    pe->pipeline.version++;

    return 0;
}

int pipeline_validate(uint32_t pipeline_id)
{
    pipeline_entry_t *pe = find_pipeline(pipeline_id);
    if (!pe) return -1;

    pipeline_t *p = &pe->pipeline;

    if (p->stage_count == 0) {
        serial_puts("[pipeline] Validation failed: no stages\n");
        return -2;
    }

    /* Check for at least one entry stage (no dependencies) */
    int has_entry = 0;
    for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (p->stages[i].in_use && p->stages[i].dependency_count == 0) {
            has_entry = 1;
            break;
        }
    }
    if (!has_entry) {
        serial_puts("[pipeline] Validation failed: no entry stage\n");
        return -3;
    }

    /* Check all dependencies exist */
    for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (!p->stages[i].in_use) continue;
        for (uint8_t d = 0; d < p->stages[i].dependency_count; d++) {
            uint32_t dep = p->stages[i].depends_on[d];
            if (dep == 0) continue;
            if (!find_stage_in_pipeline(p, dep)) {
                serial_printf("[pipeline] Validation failed: "
                              "stage %u has missing dep %u\n",
                              p->stages[i].stage_id, dep);
                return -4;
            }
        }
    }

    /* Cycle detection */
    uint8_t color[PIPELINE_MAX_STAGES];
    memset(color, 0, sizeof(color));

    for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (!p->stages[i].in_use) continue;
        if (color[i] == 0) {
            if (dfs_has_cycle(p, p->stages[i].stage_id, color)) {
                serial_puts("[pipeline] Validation failed: cycle detected\n");
                return -5;
            }
        }
    }

    serial_printf("[pipeline] Pipeline %u validated (%u stages)\n",
                  pipeline_id, p->stage_count);
    return 0;
}

uint32_t pipeline_execute(uint32_t pipeline_id,
                          const uint8_t *input_data __attribute__((unused)),
                          uint32_t input_size)
{
    pipeline_entry_t *pe = find_pipeline(pipeline_id);
    if (!pe) return 0;

    /* Validate first */
    if (pipeline_validate(pipeline_id) < 0) return 0;

    /* Find free execution slot */
    int free_idx = -1;
    for (int i = 0; i < PIPELINE_MAX_EXECUTIONS; i++) {
        if (executions[i].exec_id == 0) {
            free_idx = i;
            break;
        }
    }
    if (free_idx < 0) return 0;

    pipeline_exec_t *exec = &executions[free_idx];
    memset(exec, 0, sizeof(pipeline_exec_t));

    exec->exec_id = exec_id_counter++;
    exec->pipeline_id = pipeline_id;
    exec->status = PIPELINE_EXEC_RUNNING;

    /* Initialize stage exec states */
    pipeline_t *p = &pe->pipeline;
    for (uint32_t i = 0; i < PIPELINE_MAX_STAGES; i++) {
        if (p->stages[i].in_use) {
            exec->stage_states[i].stage_id = p->stages[i].stage_id;
            exec->stage_states[i].status = STAGE_PENDING;
        }
    }

    global_stats.active_executions++;
    global_stats.total_executions++;

    /* Get topological order */
    uint32_t topo_order[PIPELINE_MAX_STAGES];
    int topo_count = topological_sort(p, topo_order, PIPELINE_MAX_STAGES);
    if (topo_count < 0) {
        exec->status = PIPELINE_EXEC_FAILED;
        global_stats.active_executions--;
        return 0;
    }

    /* Execute stages in topological order */
    uint32_t total_latency = 0;

    for (int t = 0; t < topo_count; t++) {
        uint32_t idx = topo_order[t];
        pipeline_stage_t *stage = &p->stages[idx];
        stage_exec_state_t *ss = &exec->stage_states[idx];

        /* Verify all deps completed */
        if (!all_deps_completed(p, stage, exec->stage_states)) {
            ss->status = STAGE_FAILED;
            exec->stages_failed++;
            exec->status = PIPELINE_EXEC_FAILED;
            break;
        }

        ss->status = STAGE_RUNNING;
        exec->current_stage_idx = idx;

        /*
         * Simulate stage execution. In a real system this would call
         * inference_execute() with model_id and input from previous stage.
         */
        uint32_t stage_latency = 200 + (pipe_random() % 1000);
        ss->latency_us = stage_latency;
        total_latency += stage_latency;

        /* Simulate output */
        uint32_t out_size = input_size > 0 ? input_size : 64;
        if (out_size > PIPELINE_INTERMEDIATE_BUF_SIZE)
            out_size = PIPELINE_INTERMEDIATE_BUF_SIZE;
        ss->output_size = out_size;
        memset(ss->output_buffer, (uint8_t)(stage->model_id & 0xFF), out_size);

        ss->status = STAGE_COMPLETED;
        exec->stages_completed++;
        global_stats.total_stage_runs++;
    }

    /* Finalize execution */
    if (exec->status == PIPELINE_EXEC_RUNNING) {
        exec->status = PIPELINE_EXEC_COMPLETED;

        /* Copy last stage's output as final output */
        if (topo_count > 0) {
            uint32_t last_idx = topo_order[topo_count - 1];
            stage_exec_state_t *last_ss = &exec->stage_states[last_idx];
            uint32_t copy_size = last_ss->output_size;
            if (copy_size > PIPELINE_INTERMEDIATE_BUF_SIZE)
                copy_size = PIPELINE_INTERMEDIATE_BUF_SIZE;
            memcpy(exec->final_output, last_ss->output_buffer, copy_size);
            exec->final_output_size = copy_size;
        }

        pe->stats.successful_executions++;
    } else {
        pe->stats.failed_executions++;
    }

    exec->total_latency_us = total_latency;
    pe->stats.total_executions++;

    /* Update latency stats */
    if (total_latency < pe->stats.min_latency_us)
        pe->stats.min_latency_us = total_latency;
    if (total_latency > pe->stats.max_latency_us)
        pe->stats.max_latency_us = total_latency;

    /* Rolling average (32-bit safe) */
    if (pe->stats.total_executions > 0) {
        uint32_t prev = pe->stats.avg_latency_us;
        uint32_t n = (uint32_t)pe->stats.total_executions;
        /* Incremental mean: avg = prev + (new - prev) / n */
        if (total_latency >= prev)
            pe->stats.avg_latency_us = prev + (total_latency - prev) / n;
        else
            pe->stats.avg_latency_us = prev - (prev - total_latency) / n;
    }

    pe->stats.avg_stages_per_exec = pe->pipeline.stage_count;

    if (global_stats.active_executions > 0)
        global_stats.active_executions--;

    serial_printf("[pipeline] Pipeline %u execution %u: %s "
                  "(%u stages, %u us)\n",
                  pipeline_id, exec->exec_id,
                  exec->status == PIPELINE_EXEC_COMPLETED ?
                      "COMPLETED" : "FAILED",
                  exec->stages_completed, total_latency);

    return exec->exec_id;
}

int pipeline_get_status(uint32_t exec_id, pipeline_exec_status_t *out)
{
    if (!out || exec_id == 0) return -1;

    pipeline_exec_t *exec = find_execution(exec_id);
    if (!exec) return -1;

    *out = exec->status;
    return 0;
}

int pipeline_get_execution(uint32_t exec_id, pipeline_exec_t *out)
{
    if (!out || exec_id == 0) return -1;

    pipeline_exec_t *exec = find_execution(exec_id);
    if (!exec) return -1;

    memcpy(out, exec, sizeof(pipeline_exec_t));
    return 0;
}

int pipeline_get_output(uint32_t exec_id, uint8_t *output_data,
                        uint32_t max_size, uint32_t *actual_size)
{
    if (!output_data || !actual_size || exec_id == 0) return -1;

    pipeline_exec_t *exec = find_execution(exec_id);
    if (!exec) return -1;

    if (exec->status != PIPELINE_EXEC_COMPLETED) return -2;

    uint32_t copy_size = exec->final_output_size;
    if (copy_size > max_size) copy_size = max_size;

    memcpy(output_data, exec->final_output, copy_size);
    *actual_size = copy_size;

    return 0;
}

int pipeline_cancel_execution(uint32_t exec_id)
{
    pipeline_exec_t *exec = find_execution(exec_id);
    if (!exec) return -1;

    if (exec->status != PIPELINE_EXEC_RUNNING) return -2;

    exec->status = PIPELINE_EXEC_CANCELLED;

    if (global_stats.active_executions > 0)
        global_stats.active_executions--;

    return 0;
}

int pipeline_get(uint32_t pipeline_id, pipeline_t *out)
{
    if (!out) return -1;

    pipeline_entry_t *pe = find_pipeline(pipeline_id);
    if (!pe) return -1;

    memcpy(out, &pe->pipeline, sizeof(pipeline_t));
    return 0;
}

uint32_t pipeline_list(pipeline_t *out_pipelines, uint32_t max_pipelines)
{
    if (!out_pipelines || max_pipelines == 0) return 0;

    uint32_t count = 0;
    for (int i = 0; i < PIPELINE_MAX_PIPELINES && count < max_pipelines; i++) {
        if (pipelines[i].pipeline.in_use) {
            memcpy(&out_pipelines[count], &pipelines[i].pipeline,
                   sizeof(pipeline_t));
            count++;
        }
    }
    return count;
}

int pipeline_get_stats(uint32_t pipeline_id, pipeline_stats_t *out)
{
    if (!out) return -1;

    pipeline_entry_t *pe = find_pipeline(pipeline_id);
    if (!pe) return -1;

    memcpy(out, &pe->stats, sizeof(pipeline_stats_t));
    return 0;
}

pipeline_global_stats_t *pipeline_get_global_stats(void)
{
    return &global_stats;
}

int pipeline_destroy(uint32_t pipeline_id)
{
    pipeline_entry_t *pe = find_pipeline(pipeline_id);
    if (!pe) return -1;

    serial_printf("[pipeline] Destroying pipeline '%s' (id=%u)\n",
                  pe->pipeline.name, pipeline_id);

    /* Clean up any execution references */
    for (int i = 0; i < PIPELINE_MAX_EXECUTIONS; i++) {
        if (executions[i].pipeline_id == pipeline_id) {
            executions[i].exec_id = 0;  /* Invalidate */
        }
    }

    pe->pipeline.in_use = 0;

    if (global_stats.active_pipelines > 0)
        global_stats.active_pipelines--;

    return 0;
}

uint32_t pipeline_get_count(void)
{
    return global_stats.active_pipelines;
}

uint32_t pipeline_get_stage_count(uint32_t pipeline_id)
{
    pipeline_entry_t *pe = find_pipeline(pipeline_id);
    if (!pe) return 0;

    return pe->pipeline.stage_count;
}
