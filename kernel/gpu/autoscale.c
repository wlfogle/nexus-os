#include "../../include/kernel/autoscale.h"
#include "../../include/kernel/model_serving.h"
#include "../../include/kernel/serial.h"
#include <string.h>

/* Per-endpoint scaling state (indexed same as policies) */
static scale_policy_t policies[AUTOSCALE_MAX_POLICIES];
static scale_state_t states[AUTOSCALE_MAX_POLICIES];
static scale_event_t events[AUTOSCALE_MAX_EVENTS];
static autoscale_stats_t stats = {0};
static autoscale_config_t config = {0};
static uint32_t policy_id_counter = 1;
static uint32_t event_id_counter = 1;
static uint32_t event_write_idx = 0;
static uint32_t event_count = 0;

/* Simulated tick counter (would come from timer in real system) */
static uint32_t sim_tick = 0;

static uint32_t get_tick(void)
{
    return sim_tick++;
}

static scale_policy_t *find_policy(uint32_t policy_id)
{
    for (int i = 0; i < AUTOSCALE_MAX_POLICIES; i++) {
        if (policies[i].in_use && policies[i].policy_id == policy_id)
            return &policies[i];
    }
    return (void *)0;
}

static scale_policy_t *find_policy_for_endpoint(uint32_t endpoint_id)
{
    for (int i = 0; i < AUTOSCALE_MAX_POLICIES; i++) {
        if (policies[i].in_use && policies[i].endpoint_id == endpoint_id)
            return &policies[i];
    }
    return (void *)0;
}

static scale_state_t *find_state(uint32_t endpoint_id)
{
    for (int i = 0; i < AUTOSCALE_MAX_POLICIES; i++) {
        if (policies[i].in_use && policies[i].endpoint_id == endpoint_id)
            return &states[i];
    }
    return (void *)0;
}

static int get_policy_index(uint32_t policy_id)
{
    for (int i = 0; i < AUTOSCALE_MAX_POLICIES; i++) {
        if (policies[i].in_use && policies[i].policy_id == policy_id)
            return i;
    }
    return -1;
}

static void record_event(uint32_t endpoint_id, uint32_t policy_id,
                          scale_direction_t dir, uint16_t old_rep,
                          uint16_t new_rep, uint32_t metric_val,
                          scale_metric_t metric_type)
{
    scale_event_t *ev = &events[event_write_idx];
    ev->event_id = event_id_counter++;
    ev->endpoint_id = endpoint_id;
    ev->policy_id = policy_id;
    ev->direction = dir;
    ev->old_replicas = old_rep;
    ev->new_replicas = new_rep;
    ev->metric_value = metric_val;
    ev->metric_type = metric_type;
    ev->timestamp = get_tick();

    event_write_idx = (event_write_idx + 1) % AUTOSCALE_MAX_EVENTS;
    if (event_count < AUTOSCALE_MAX_EVENTS)
        event_count++;

    stats.total_events++;
}

/* Fetch current metric value for an endpoint */
static uint32_t get_metric_value(uint32_t endpoint_id, scale_metric_t metric)
{
    serving_endpoint_stats_t ep_stats;
    if (serving_get_endpoint_stats(endpoint_id, &ep_stats) < 0)
        return 0;

    switch (metric) {
        case SCALE_METRIC_GPU_UTIL:
            /* Approximate GPU util from request rate */
            return ep_stats.requests_per_sec > 0 ?
                   (ep_stats.requests_per_sec * 100) / 1000 : 0;
        case SCALE_METRIC_QUEUE_DEPTH:
            return ep_stats.queue_depth;
        case SCALE_METRIC_LATENCY_P99:
            return ep_stats.latency.p99_us;
        case SCALE_METRIC_CONNECTIONS:
            return ep_stats.active_connections;
        default:
            return 0;
    }
}

void autoscale_init(void)
{
    memset(policies, 0, sizeof(policies));
    memset(states, 0, sizeof(states));
    memset(events, 0, sizeof(events));
    memset(&stats, 0, sizeof(autoscale_stats_t));

    config.evaluation_interval_ms = AUTOSCALE_DEFAULT_EVAL_INTERVAL_MS;
    config.default_cooldown_ms = AUTOSCALE_DEFAULT_COOLDOWN_MS;
    config.enabled = 1;
    config.stabilization_window = 3;  /* 3 consecutive breaches */

    policy_id_counter = 1;
    event_id_counter = 1;
    event_write_idx = 0;
    event_count = 0;
    sim_tick = 0;

    serial_puts("[autoscale] Auto-scaling engine initialized\n");
}

uint32_t autoscale_create_policy(uint32_t endpoint_id,
                                 scale_metric_t metric,
                                 uint32_t scale_up_threshold,
                                 uint32_t scale_down_threshold,
                                 uint16_t min_replicas,
                                 uint16_t max_replicas)
{
    if (endpoint_id == 0) return 0;
    if (min_replicas > max_replicas) return 0;
    if (min_replicas == 0) min_replicas = 1;

    /* Check for existing policy on this endpoint */
    if (find_policy_for_endpoint(endpoint_id)) {
        serial_printf("[autoscale] Policy already exists for endpoint %u\n",
                      endpoint_id);
        return 0;
    }

    /* Find free slot */
    int free_idx = -1;
    for (int i = 0; i < AUTOSCALE_MAX_POLICIES; i++) {
        if (!policies[i].in_use) {
            free_idx = i;
            break;
        }
    }

    if (free_idx < 0) return 0;

    scale_policy_t *p = &policies[free_idx];
    p->policy_id = policy_id_counter++;
    p->endpoint_id = endpoint_id;
    p->metric = metric;
    p->scale_up_threshold = scale_up_threshold;
    p->scale_down_threshold = scale_down_threshold;
    p->min_replicas = min_replicas;
    p->max_replicas = max_replicas;
    p->cooldown_ms = config.default_cooldown_ms;
    p->enabled = 1;
    p->in_use = 1;

    /* Initialize state */
    scale_state_t *s = &states[free_idx];
    memset(s, 0, sizeof(scale_state_t));
    s->endpoint_id = endpoint_id;
    s->current_replicas = min_replicas;
    s->desired_replicas = min_replicas;
    s->last_direction = SCALE_NONE;

    stats.active_policies++;

    serial_printf("[autoscale] Policy %u created: endpoint=%u metric=%d "
                  "up=%u down=%u replicas=[%u,%u]\n",
                  p->policy_id, endpoint_id, (int)metric,
                  scale_up_threshold, scale_down_threshold,
                  (unsigned)min_replicas, (unsigned)max_replicas);

    return p->policy_id;
}

int autoscale_remove_policy(uint32_t policy_id)
{
    int idx = get_policy_index(policy_id);
    if (idx < 0) return -1;

    policies[idx].in_use = 0;

    if (stats.active_policies > 0)
        stats.active_policies--;

    serial_printf("[autoscale] Policy %u removed\n", policy_id);
    return 0;
}

int autoscale_get_policy(uint32_t policy_id, scale_policy_t *out)
{
    if (!out) return -1;

    scale_policy_t *p = find_policy(policy_id);
    if (!p) return -1;

    memcpy(out, p, sizeof(scale_policy_t));
    return 0;
}

int autoscale_set_policy_enabled(uint32_t policy_id, uint8_t enabled)
{
    scale_policy_t *p = find_policy(policy_id);
    if (!p) return -1;

    p->enabled = enabled;
    return 0;
}

int autoscale_set_replica_range(uint32_t policy_id,
                                uint16_t min_replicas,
                                uint16_t max_replicas)
{
    if (min_replicas > max_replicas) return -1;

    scale_policy_t *p = find_policy(policy_id);
    if (!p) return -1;

    p->min_replicas = min_replicas;
    p->max_replicas = max_replicas;

    /* Clamp current replicas */
    int idx = get_policy_index(policy_id);
    if (idx >= 0) {
        if (states[idx].current_replicas < min_replicas)
            states[idx].current_replicas = min_replicas;
        if (states[idx].current_replicas > max_replicas)
            states[idx].current_replicas = max_replicas;
        states[idx].desired_replicas = states[idx].current_replicas;
    }

    return 0;
}

int autoscale_set_cooldown(uint32_t policy_id, uint32_t cooldown_ms)
{
    scale_policy_t *p = find_policy(policy_id);
    if (!p) return -1;

    p->cooldown_ms = cooldown_ms;
    return 0;
}

int autoscale_evaluate(uint32_t endpoint_id)
{
    if (!config.enabled) return 0;

    scale_policy_t *p = find_policy_for_endpoint(endpoint_id);
    if (!p || !p->enabled) return 0;

    int idx = -1;
    for (int i = 0; i < AUTOSCALE_MAX_POLICIES; i++) {
        if (policies[i].in_use && policies[i].endpoint_id == endpoint_id) {
            idx = i;
            break;
        }
    }
    if (idx < 0) return -1;

    scale_state_t *s = &states[idx];
    uint32_t now = get_tick();

    stats.total_evaluations++;
    s->last_eval_time = now;

    /* Get current metric value */
    uint32_t metric_val = get_metric_value(endpoint_id, p->metric);
    s->last_metric_value = metric_val;

    /* Determine desired direction */
    scale_direction_t desired_dir = SCALE_NONE;
    uint16_t desired_replicas = s->current_replicas;

    if (metric_val > p->scale_up_threshold) {
        desired_dir = SCALE_UP;
        desired_replicas = s->current_replicas + 1;
        if (desired_replicas > p->max_replicas)
            desired_replicas = p->max_replicas;
    } else if (metric_val < p->scale_down_threshold) {
        desired_dir = SCALE_DOWN;
        if (s->current_replicas > p->min_replicas)
            desired_replicas = s->current_replicas - 1;
        else
            desired_replicas = s->current_replicas;
    }

    s->desired_replicas = desired_replicas;

    /* Check stabilization window */
    if (desired_dir != SCALE_NONE) {
        s->consecutive_breaches++;
    } else {
        s->consecutive_breaches = 0;
        return 0;  /* No action needed */
    }

    if (s->consecutive_breaches < config.stabilization_window) {
        return 0;  /* Not enough consecutive breaches yet */
    }

    /* Check cooldown */
    if (s->last_scale_time > 0 &&
        (now - s->last_scale_time) < (p->cooldown_ms / 1000)) {
        stats.total_cooldown_skips++;
        return 0;
    }

    /* Apply scaling action */
    if (desired_replicas != s->current_replicas) {
        uint16_t old = s->current_replicas;
        s->current_replicas = desired_replicas;
        s->last_direction = desired_dir;
        s->last_scale_time = now;
        s->consecutive_breaches = 0;

        if (desired_dir == SCALE_UP)
            stats.total_scale_ups++;
        else
            stats.total_scale_downs++;

        record_event(endpoint_id, p->policy_id, desired_dir,
                     old, desired_replicas, metric_val, p->metric);

        serial_printf("[autoscale] Endpoint %u: %s %u -> %u replicas "
                      "(metric=%u threshold=%u)\n",
                      endpoint_id,
                      desired_dir == SCALE_UP ? "SCALE_UP" : "SCALE_DOWN",
                      (unsigned)old, (unsigned)desired_replicas,
                      metric_val,
                      desired_dir == SCALE_UP ?
                          p->scale_up_threshold : p->scale_down_threshold);
    }

    return (int)desired_dir;
}

int autoscale_evaluate_all(void)
{
    if (!config.enabled) return 0;

    int actions = 0;
    for (int i = 0; i < AUTOSCALE_MAX_POLICIES; i++) {
        if (policies[i].in_use && policies[i].enabled) {
            int result = autoscale_evaluate(policies[i].endpoint_id);
            if (result > 0) actions++;
        }
    }
    return actions;
}

int autoscale_get_state(uint32_t endpoint_id, scale_state_t *out)
{
    if (!out) return -1;

    scale_state_t *s = find_state(endpoint_id);
    if (!s) return -1;

    memcpy(out, s, sizeof(scale_state_t));
    return 0;
}

int autoscale_set_replicas(uint32_t endpoint_id, uint16_t replicas)
{
    scale_state_t *s = find_state(endpoint_id);
    if (!s) return -1;

    scale_policy_t *p = find_policy_for_endpoint(endpoint_id);
    if (p) {
        if (replicas < p->min_replicas) replicas = p->min_replicas;
        if (replicas > p->max_replicas) replicas = p->max_replicas;
    }

    uint16_t old = s->current_replicas;
    s->current_replicas = replicas;
    s->desired_replicas = replicas;
    s->consecutive_breaches = 0;

    serial_printf("[autoscale] Manual scale endpoint %u: %u -> %u replicas\n",
                  endpoint_id, (unsigned)old, (unsigned)replicas);
    return 0;
}

uint32_t autoscale_get_events(scale_event_t *out_events, uint32_t max_events)
{
    if (!out_events || max_events == 0) return 0;

    uint32_t count = event_count < max_events ? event_count : max_events;

    /* Return most recent events first */
    uint32_t read_idx;
    if (event_count >= AUTOSCALE_MAX_EVENTS) {
        read_idx = event_write_idx;  /* Ring buffer wrapped */
    } else {
        read_idx = 0;
    }

    for (uint32_t i = 0; i < count; i++) {
        memcpy(&out_events[i], &events[(read_idx + i) % AUTOSCALE_MAX_EVENTS],
               sizeof(scale_event_t));
    }

    return count;
}

autoscale_stats_t *autoscale_get_stats(void)
{
    return &stats;
}

int autoscale_set_config(const autoscale_config_t *cfg)
{
    if (!cfg) return -1;

    memcpy(&config, cfg, sizeof(autoscale_config_t));
    return 0;
}

int autoscale_get_config(autoscale_config_t *out)
{
    if (!out) return -1;

    memcpy(out, &config, sizeof(autoscale_config_t));
    return 0;
}

int autoscale_set_enabled(uint8_t enabled)
{
    config.enabled = enabled;

    serial_printf("[autoscale] Global autoscaling %s\n",
                  enabled ? "enabled" : "disabled");
    return 0;
}

uint32_t autoscale_get_policy_count(void)
{
    return stats.active_policies;
}
