#include "../../include/kernel/telemetry.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_EVENTS 512
#define MAX_METRICS 512
#define MAX_SESSIONS 16

typedef struct {
    telem_event_t events[MAX_EVENTS];
    uint32_t event_count;
} event_buffer_t;

typedef struct {
    telem_metric_t metrics[MAX_METRICS];
    uint32_t metric_count;
} metric_buffer_t;

typedef struct {
    uint32_t session_id;
    uint32_t start_time;
    uint32_t end_time;
    uint32_t event_start_idx;
    uint32_t event_count;
    uint32_t metric_start_idx;
    uint32_t metric_count;
    int active;
} session_t;

static event_buffer_t event_buffer = {0};
static metric_buffer_t metric_buffer = {0};
static session_t sessions[MAX_SESSIONS] = {0};
static telem_aggregates_t aggregates = {0};
static uint32_t session_counter = 0;
static uint32_t kernel_ticks = 0;

void telemetry_init(void)
{
    memset(&event_buffer, 0, sizeof(event_buffer_t));
    memset(&metric_buffer, 0, sizeof(metric_buffer_t));
    memset(sessions, 0, sizeof(sessions));
    memset(&aggregates, 0, sizeof(telem_aggregates_t));
    
    session_counter = 1;
    kernel_ticks = 0;
    
    serial_puts("[telemetry] Telemetry and observability subsystem initialized\n");
}

int telemetry_log_event(telem_event_type_t type, uint32_t task_id,
                        uint32_t gpu_device_id, uint32_t data_u32, uint16_t data_u16)
{
    if (type == TELEM_EVENT_NONE) return -1;
    
    /* Check buffer overflow */
    if (event_buffer.event_count >= MAX_EVENTS) {
        return -1;  /* Buffer full, event dropped */
    }
    
    telem_event_t *evt = &event_buffer.events[event_buffer.event_count];
    evt->timestamp = kernel_ticks;
    evt->type = type;
    evt->task_id = task_id;
    evt->gpu_device_id = gpu_device_id;
    evt->data_u32 = data_u32;
    evt->data_u16 = data_u16;
    
    event_buffer.event_count++;
    
    /* Update aggregates based on event type */
    if (type == TELEM_EVENT_THERMAL_SPIKE) {
        aggregates.thermal_events_count++;
    } else if (type == TELEM_EVENT_TASK_MIGRATE) {
        aggregates.task_migrations++;
    }
    
    return 0;
}

int telemetry_log_metric(telem_metric_type_t metric, uint32_t value,
                         uint32_t task_id, uint16_t gpu_device_id)
{
    if (metric < 0 || metric > 9) return -1;
    
    /* Check buffer overflow */
    if (metric_buffer.metric_count >= MAX_METRICS) {
        return -1;  /* Buffer full, metric dropped */
    }
    
    telem_metric_t *m = &metric_buffer.metrics[metric_buffer.metric_count];
    m->timestamp = kernel_ticks;
    m->metric_type = metric;
    m->value = value;
    m->task_id = task_id;
    m->gpu_device_id = gpu_device_id;
    
    metric_buffer.metric_count++;
    
    /* Update running aggregates */
    if (metric == METRIC_GPU_UTILIZATION) {
        /* Rolling average of GPU utilization */
        uint32_t weight = metric_buffer.metric_count;
        if (weight > 1) {
            aggregates.avg_gpu_utilization = 
                (aggregates.avg_gpu_utilization * (weight - 1) + value) / weight;
        } else {
            aggregates.avg_gpu_utilization = value;
        }
    } else if (metric == METRIC_GPU_MEMORY) {
        if (value > aggregates.peak_gpu_memory) {
            aggregates.peak_gpu_memory = value;
        }
    } else if (metric == METRIC_THROUGHPUT_INFER) {
        aggregates.total_inferences += value;
    } else if (metric == METRIC_POWER_DRAW) {
        aggregates.power_draw_mw = value;
    }
    
    return 0;
}

uint32_t telemetry_session_start(void)
{
    /* Find free session slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_SESSIONS; i++) {
        if (!sessions[i].active) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;  /* No free sessions */
    
    session_t *sess = &sessions[free_idx];
    sess->session_id = session_counter++;
    sess->start_time = kernel_ticks;
    sess->event_start_idx = event_buffer.event_count;
    sess->metric_start_idx = metric_buffer.metric_count;
    sess->active = 1;
    
    return sess->session_id;
}

int telemetry_session_end(uint32_t session_id, telem_aggregates_t *out_agg)
{
    if (!out_agg || session_id == 0) return -1;
    
    /* Find session */
    int sess_idx = -1;
    for (int i = 0; i < MAX_SESSIONS; i++) {
        if (sessions[i].active && sessions[i].session_id == session_id) {
            sess_idx = i;
            break;
        }
    }
    
    if (sess_idx < 0) return -1;
    
    session_t *sess = &sessions[sess_idx];
    sess->end_time = kernel_ticks;
    sess->event_count = event_buffer.event_count - sess->event_start_idx;
    sess->metric_count = metric_buffer.metric_count - sess->metric_start_idx;
    sess->active = 0;
    
    /* Return current aggregates */
    memcpy(out_agg, &aggregates, sizeof(telem_aggregates_t));
    out_agg->uptime_seconds = kernel_ticks / 1000;  /* Assuming 1000 ticks/second */
    
    return 0;
}

telem_aggregates_t *telemetry_get_aggregates(void)
{
    aggregates.uptime_seconds = kernel_ticks / 1000;
    return &aggregates;
}

int telemetry_query_events(telem_event_type_t type, telem_event_t *events,
                           uint32_t max_events, uint32_t *out_count)
{
    if (!events || !out_count || max_events == 0) return -1;
    
    uint32_t count = 0;
    for (uint32_t i = 0; i < event_buffer.event_count && count < max_events; i++) {
        if (event_buffer.events[i].type == type) {
            memcpy(&events[count], &event_buffer.events[i], sizeof(telem_event_t));
            count++;
        }
    }
    
    *out_count = count;
    return 0;
}

int telemetry_query_metrics(telem_metric_type_t metric, telem_metric_t *metrics,
                            uint32_t max_metrics, uint32_t *out_count)
{
    if (!metrics || !out_count || max_metrics == 0) return -1;
    
    uint32_t count = 0;
    for (uint32_t i = 0; i < metric_buffer.metric_count && count < max_metrics; i++) {
        if (metric_buffer.metrics[i].metric_type == metric) {
            memcpy(&metrics[count], &metric_buffer.metrics[i], sizeof(telem_metric_t));
            count++;
        }
    }
    
    *out_count = count;
    return 0;
}

int telemetry_export_snapshot(char *buffer, uint32_t buffer_size)
{
    if (!buffer || buffer_size < 128) return -1;
    
    /* Create text snapshot of current telemetry state */
    /* Simplified text format to fit in buffer */
    int written = 0;
    (void)buffer_size;  /* Used in size check above */
    
    /* Manual string building to avoid serial_snprintf */
    memcpy(buffer, "=== Telemetry ===\n", 18);
    written = 18;
    
    /* Events line */
    memcpy(buffer + written, "E:", 2);
    written += 2;
    /* Convert event count to string (simplified) */
    uint32_t ec = event_buffer.event_count;
    int digits = 0;
    if (ec == 0) {
        buffer[written++] = '0';
    } else {
        uint32_t temp_ec = ec;
        while (temp_ec) {
            temp_ec /= 10;
            digits++;
        }
        uint32_t div = 1;
        for (int i = 1; i < digits; i++) div *= 10;
        while (div > 0 && ec > 0) {
            buffer[written++] = '0' + (ec / div);
            ec %= div;
            div /= 10;
        }
    }
    buffer[written++] = ' ';
    buffer[written++] = 'M';
    buffer[written++] = ':';
    
    uint32_t mc = metric_buffer.metric_count;
    if (mc == 0) {
        buffer[written++] = '0';
    } else {
        uint32_t temp_mc = mc;
        int m_digits = 0;
        while (temp_mc) {
            temp_mc /= 10;
            m_digits++;
        }
        uint32_t m_div = 1;
        for (int i = 1; i < m_digits; i++) m_div *= 10;
        while (m_div > 0 && mc > 0) {
            buffer[written++] = '0' + (mc / m_div);
            mc %= m_div;
            m_div /= 10;
        }
    }
    buffer[written++] = '\n';
    
    if (written < (int)buffer_size) {
        buffer[written] = '\0';
    }
    
    return written;
}

void telemetry_reset(void)
{
    memset(&event_buffer, 0, sizeof(event_buffer_t));
    memset(&metric_buffer, 0, sizeof(metric_buffer_t));
    memset(&aggregates, 0, sizeof(telem_aggregates_t));
    
    serial_puts("[telemetry] Telemetry buffers reset\n");
}

uint32_t telemetry_get_event_count(void)
{
    return event_buffer.event_count;
}

uint32_t telemetry_get_metric_count(void)
{
    return metric_buffer.metric_count;
}

uint32_t telemetry_get_buffer_usage_pct(void)
{
    uint32_t event_usage = (event_buffer.event_count * 100) / MAX_EVENTS;
    uint32_t metric_usage = (metric_buffer.metric_count * 100) / MAX_METRICS;
    
    /* Return max of the two */
    return (event_usage > metric_usage) ? event_usage : metric_usage;
}

/* Kernel tick update function (called from timer IRQ) */
void telemetry_update_tick(void)
{
    kernel_ticks++;
}
