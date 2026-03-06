#include "../../include/kernel/model_serving.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define LATENCY_SAMPLE_COUNT 128

/* Internal latency tracking per endpoint */
typedef struct {
    uint32_t samples[LATENCY_SAMPLE_COUNT];
    uint32_t sample_idx;
    uint32_t sample_count;
} latency_tracker_t;

/* Per-endpoint internal state */
typedef struct {
    serving_endpoint_t endpoint;
    serving_endpoint_stats_t stats;
    latency_tracker_t latency;
    uint32_t request_counter;
    uint32_t last_qps_time;
    uint32_t qps_window_count;
} endpoint_entry_t;

static endpoint_entry_t endpoints[SERVING_MAX_ENDPOINTS];
static serving_global_stats_t global_stats = {0};
static serving_config_t global_config = {0};
static uint32_t endpoint_id_counter = 1;

/* Simple LCG for A/B split randomization */
static uint32_t ab_rng_state = 12345;

static uint32_t ab_random(void)
{
    ab_rng_state = ab_rng_state * 1103515245 + 12345;
    return (ab_rng_state >> 16) & 0x7FFF;
}

/* Compute latency percentiles from sorted samples */
static void compute_latency_histogram(latency_tracker_t *tracker,
                                      latency_histogram_t *out)
{
    if (tracker->sample_count == 0) {
        memset(out, 0, sizeof(latency_histogram_t));
        return;
    }

    /* Simple insertion sort on a copy for percentile computation */
    uint32_t sorted[LATENCY_SAMPLE_COUNT];
    uint32_t n = tracker->sample_count;
    if (n > LATENCY_SAMPLE_COUNT) n = LATENCY_SAMPLE_COUNT;

    memcpy(sorted, tracker->samples, n * sizeof(uint32_t));

    for (uint32_t i = 1; i < n; i++) {
        uint32_t key = sorted[i];
        int j = (int)i - 1;
        while (j >= 0 && sorted[j] > key) {
            sorted[j + 1] = sorted[j];
            j--;
        }
        sorted[j + 1] = key;
    }

    out->p50_us = sorted[n * 50 / 100];
    out->p95_us = sorted[n * 95 / 100];
    out->p99_us = sorted[n * 99 / 100];
    out->max_us = sorted[n - 1];

    /* Compute average (use 32-bit division — safe for <=128 samples) */
    uint32_t total = 0;
    for (uint32_t i = 0; i < n; i++) {
        total += sorted[i];
    }
    out->avg_us = total / n;
}

static void record_latency(endpoint_entry_t *ep, uint32_t latency_us)
{
    latency_tracker_t *t = &ep->latency;
    t->samples[t->sample_idx] = latency_us;
    t->sample_idx = (t->sample_idx + 1) % LATENCY_SAMPLE_COUNT;
    if (t->sample_count < LATENCY_SAMPLE_COUNT)
        t->sample_count++;
}

static endpoint_entry_t *find_endpoint(uint32_t endpoint_id)
{
    for (int i = 0; i < SERVING_MAX_ENDPOINTS; i++) {
        if (endpoints[i].endpoint.in_use &&
            endpoints[i].endpoint.endpoint_id == endpoint_id) {
            return &endpoints[i];
        }
    }
    return (void *)0;
}

void model_serving_init(void)
{
    memset(endpoints, 0, sizeof(endpoints));
    memset(&global_stats, 0, sizeof(serving_global_stats_t));

    global_config.max_connections = SERVING_MAX_CONNECTIONS;
    global_config.request_timeout_ms = SERVING_REQUEST_TIMEOUT_MS;
    global_config.max_queue_depth = SERVING_MAX_QUEUE_DEPTH;
    global_config.health_check_enabled = 1;

    endpoint_id_counter = 1;

    serial_puts("[model_serving] Model serving gateway initialized\n");
}

uint32_t serving_register_endpoint(uint32_t model_id, uint16_t port,
                                   const char *name)
{
    if (model_id == 0 || port == 0 || !name) return 0;

    /* Check for port conflict */
    for (int i = 0; i < SERVING_MAX_ENDPOINTS; i++) {
        if (endpoints[i].endpoint.in_use &&
            endpoints[i].endpoint.port == port) {
            serial_printf("[model_serving] Port %u already in use\n",
                          (unsigned)port);
            return 0;
        }
    }

    /* Find free slot */
    int free_idx = -1;
    for (int i = 0; i < SERVING_MAX_ENDPOINTS; i++) {
        if (!endpoints[i].endpoint.in_use) {
            free_idx = i;
            break;
        }
    }

    if (free_idx < 0) return 0;  /* No slots available */

    endpoint_entry_t *ep = &endpoints[free_idx];
    memset(ep, 0, sizeof(endpoint_entry_t));

    ep->endpoint.endpoint_id = endpoint_id_counter++;
    ep->endpoint.model_id = model_id;
    ep->endpoint.port = port;
    ep->endpoint.state = ENDPOINT_ACTIVE;
    ep->endpoint.active_connections = 0;
    ep->endpoint.queue_depth = 0;
    ep->endpoint.in_use = 1;
    ep->request_counter = 1;

    /* Copy name safely */
    uint32_t name_len = 0;
    while (name[name_len] && name_len < SERVING_MAX_NAME_LEN - 1) name_len++;
    memcpy(ep->endpoint.name, name, name_len);
    ep->endpoint.name[name_len] = '\0';

    ep->stats.endpoint_id = ep->endpoint.endpoint_id;

    global_stats.active_endpoints++;

    serial_printf("[model_serving] Endpoint '%s' registered: model=%u port=%u id=%u\n",
                  ep->endpoint.name, model_id, (unsigned)port,
                  ep->endpoint.endpoint_id);
    return ep->endpoint.endpoint_id;
}

int serving_unregister_endpoint(uint32_t endpoint_id)
{
    endpoint_entry_t *ep = find_endpoint(endpoint_id);
    if (!ep) return -1;

    serial_printf("[model_serving] Unregistering endpoint '%s' (id=%u)\n",
                  ep->endpoint.name, endpoint_id);

    ep->endpoint.in_use = 0;
    ep->endpoint.state = ENDPOINT_INACTIVE;

    if (global_stats.active_endpoints > 0)
        global_stats.active_endpoints--;

    return 0;
}

int serving_get_endpoint(uint32_t endpoint_id, serving_endpoint_t *out)
{
    if (!out) return -1;

    endpoint_entry_t *ep = find_endpoint(endpoint_id);
    if (!ep) return -1;

    memcpy(out, &ep->endpoint, sizeof(serving_endpoint_t));
    return 0;
}

uint32_t serving_list_endpoints(serving_endpoint_t *out_endpoints,
                                uint32_t max_endpoints)
{
    if (!out_endpoints || max_endpoints == 0) return 0;

    uint32_t count = 0;
    for (int i = 0; i < SERVING_MAX_ENDPOINTS && count < max_endpoints; i++) {
        if (endpoints[i].endpoint.in_use) {
            memcpy(&out_endpoints[count], &endpoints[i].endpoint,
                   sizeof(serving_endpoint_t));
            count++;
        }
    }
    return count;
}

int serving_handle_request(uint32_t endpoint_id, const uint8_t *raw_data,
                           uint32_t len, serving_response_t *response)
{
    if (!raw_data || len == 0 || !response) return -1;

    endpoint_entry_t *ep = find_endpoint(endpoint_id);
    if (!ep) {
        response->status = SERVING_ERROR_NOT_FOUND;
        return -1;
    }

    if (ep->endpoint.state != ENDPOINT_ACTIVE) {
        response->status = SERVING_ERROR_OVERLOADED;
        return -1;
    }

    if (ep->endpoint.queue_depth >= global_config.max_queue_depth) {
        response->status = SERVING_ERROR_OVERLOADED;
        ep->stats.failed_requests++;
        global_stats.total_errors_all++;
        return -1;
    }

    ep->endpoint.queue_depth++;
    ep->endpoint.active_connections++;
    global_stats.total_connections++;

    if (global_stats.total_connections > global_stats.peak_connections)
        global_stats.peak_connections = global_stats.total_connections;

    /* Resolve model via A/B traffic split */
    uint32_t resolved_model = ep->endpoint.model_id;
    if (ep->endpoint.traffic_split.enabled) {
        uint32_t rand_pct = ab_random() % 100;
        if (rand_pct < ep->endpoint.traffic_split.split_pct_b) {
            resolved_model = ep->endpoint.traffic_split.model_id_b;
            ep->stats.model_b_requests++;
        } else {
            resolved_model = ep->endpoint.traffic_split.model_id_a;
            ep->stats.model_a_requests++;
        }
    }

    /*
     * In a real system this would call inference_execute() from model_runtime.
     * For now, simulate a successful inference with the resolved model.
     */
    uint32_t simulated_latency = 500 + (ab_random() % 2000);  /* 0.5-2.5ms */

    response->request_id = ep->request_counter++;
    response->status = SERVING_OK;
    response->output_data = (void *)0;  /* Would point to inference output */
    response->output_size = 0;
    response->latency_us = simulated_latency;
    response->model_id = resolved_model;

    /* Update stats */
    record_latency(ep, simulated_latency);
    ep->stats.total_requests++;
    ep->stats.successful_requests++;
    global_stats.total_requests_all++;

    ep->endpoint.queue_depth--;
    ep->endpoint.active_connections--;
    if (global_stats.total_connections > 0)
        global_stats.total_connections--;

    ep->stats.active_connections = ep->endpoint.active_connections;
    ep->stats.queue_depth = ep->endpoint.queue_depth;

    return 0;
}

int serving_set_traffic_split(uint32_t endpoint_id, uint32_t model_id_a,
                              uint32_t model_id_b, uint8_t split_pct_b)
{
    endpoint_entry_t *ep = find_endpoint(endpoint_id);
    if (!ep) return -1;
    if (split_pct_b > 100) return -1;

    ep->endpoint.traffic_split.model_id_a = model_id_a;
    ep->endpoint.traffic_split.model_id_b = model_id_b;
    ep->endpoint.traffic_split.split_pct_b = split_pct_b;
    ep->endpoint.traffic_split.enabled = 1;

    serial_printf("[model_serving] Traffic split on endpoint %u: "
                  "model_a=%u model_b=%u split=%u%%\n",
                  endpoint_id, model_id_a, model_id_b,
                  (unsigned)split_pct_b);
    return 0;
}

int serving_disable_traffic_split(uint32_t endpoint_id)
{
    endpoint_entry_t *ep = find_endpoint(endpoint_id);
    if (!ep) return -1;

    ep->endpoint.traffic_split.enabled = 0;
    return 0;
}

int serving_health_check(uint32_t endpoint_id)
{
    endpoint_entry_t *ep = find_endpoint(endpoint_id);
    if (!ep) return -1;

    if (ep->endpoint.state != ENDPOINT_ACTIVE)
        return -2;  /* Not active */

    if (ep->endpoint.queue_depth >= global_config.max_queue_depth)
        return -3;  /* Overloaded */

    return 0;  /* Healthy */
}

int serving_set_endpoint_state(uint32_t endpoint_id, endpoint_state_t state)
{
    endpoint_entry_t *ep = find_endpoint(endpoint_id);
    if (!ep) return -1;

    ep->endpoint.state = state;

    serial_printf("[model_serving] Endpoint %u state -> %d\n",
                  endpoint_id, state);
    return 0;
}

int serving_get_endpoint_stats(uint32_t endpoint_id,
                               serving_endpoint_stats_t *out)
{
    if (!out) return -1;

    endpoint_entry_t *ep = find_endpoint(endpoint_id);
    if (!ep) return -1;

    /* Recompute latency histogram before returning */
    compute_latency_histogram(&ep->latency, &ep->stats.latency);

    memcpy(out, &ep->stats, sizeof(serving_endpoint_stats_t));
    return 0;
}

serving_global_stats_t *serving_get_global_stats(void)
{
    /* Recompute average latency across all endpoints (32-bit safe) */
    uint32_t total_lat = 0;
    uint32_t total_samples = 0;

    for (int i = 0; i < SERVING_MAX_ENDPOINTS; i++) {
        if (endpoints[i].endpoint.in_use) {
            latency_tracker_t *t = &endpoints[i].latency;
            uint32_t n = t->sample_count;
            if (n > LATENCY_SAMPLE_COUNT) n = LATENCY_SAMPLE_COUNT;
            for (uint32_t j = 0; j < n; j++) {
                total_lat += t->samples[j];
            }
            total_samples += n;
        }
    }

    if (total_samples > 0)
        global_stats.avg_latency_us = total_lat / total_samples;

    return &global_stats;
}

int serving_set_config(const serving_config_t *config)
{
    if (!config) return -1;

    memcpy(&global_config, config, sizeof(serving_config_t));
    return 0;
}

int serving_get_config(serving_config_t *out)
{
    if (!out) return -1;

    memcpy(out, &global_config, sizeof(serving_config_t));
    return 0;
}

uint32_t serving_get_endpoint_count(void)
{
    return global_stats.active_endpoints;
}
