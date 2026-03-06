#include "../../include/kernel/federated.h"
#include "../../include/kernel/serial.h"
#include <string.h>

static fed_session_t sessions[FED_MAX_SESSIONS];
static fed_global_stats_t global_stats = {0};
static uint32_t session_id_counter = 1;
static uint32_t client_id_counter = 1;

/* Per-session round tracking */
static fed_round_t current_rounds[FED_MAX_SESSIONS];

/* Per-session stats */
static fed_session_stats_t session_stats[FED_MAX_SESSIONS];

/* Simple LCG for DP noise */
static uint32_t dp_rng = 98765;

static uint32_t dp_random(void)
{
    dp_rng = dp_rng * 1103515245 + 12345;
    return (dp_rng >> 16) & 0x7FFF;
}

static int find_session_idx(uint32_t session_id)
{
    for (int i = 0; i < FED_MAX_SESSIONS; i++) {
        if (sessions[i].in_use && sessions[i].session_id == session_id)
            return i;
    }
    return -1;
}

static fed_client_t *find_client(fed_session_t *s, uint32_t client_id)
{
    for (int i = 0; i < FED_MAX_CLIENTS; i++) {
        if (s->clients[i].in_use && s->clients[i].client_id == client_id)
            return &s->clients[i];
    }
    return (void *)0;
}

void federated_init(void)
{
    memset(sessions, 0, sizeof(sessions));
    memset(current_rounds, 0, sizeof(current_rounds));
    memset(session_stats, 0, sizeof(session_stats));
    memset(&global_stats, 0, sizeof(fed_global_stats_t));

    session_id_counter = 1;
    client_id_counter = 1;
    dp_rng = 98765;

    serial_puts("[federated] Federated learning coordinator initialized\n");
}

uint32_t federated_create_session(uint32_t model_id, const char *name,
                                  const fed_config_t *config)
{
    if (model_id == 0 || !name || !config) return 0;

    int free_idx = -1;
    for (int i = 0; i < FED_MAX_SESSIONS; i++) {
        if (!sessions[i].in_use) {
            free_idx = i;
            break;
        }
    }
    if (free_idx < 0) return 0;

    fed_session_t *s = &sessions[free_idx];
    memset(s, 0, sizeof(fed_session_t));

    s->session_id = session_id_counter++;
    s->model_id = model_id;
    s->state = FED_SESSION_RECRUITING;
    s->global_loss = 999.0f;
    s->best_loss = 999.0f;
    s->in_use = 1;

    memcpy(&s->config, config, sizeof(fed_config_t));

    uint32_t nlen = 0;
    while (name[nlen] && nlen < FED_MAX_NAME_LEN - 1) nlen++;
    memcpy(s->name, name, nlen);
    s->name[nlen] = '\0';

    /* Init round */
    current_rounds[free_idx].round_id = 0;
    current_rounds[free_idx].state = FED_ROUND_PENDING;

    /* Init stats */
    session_stats[free_idx].session_id = s->session_id;
    session_stats[free_idx].initial_loss = 999.0f;

    global_stats.active_sessions++;
    global_stats.total_sessions++;

    serial_printf("[federated] Session '%s' created (id=%u model=%u agg=%d)\n",
                  s->name, s->session_id, model_id,
                  (int)config->aggregation);
    return s->session_id;
}

int federated_destroy_session(uint32_t session_id)
{
    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    sessions[idx].in_use = 0;
    sessions[idx].state = FED_SESSION_STOPPED;

    if (global_stats.active_sessions > 0)
        global_stats.active_sessions--;

    serial_printf("[federated] Session %u destroyed\n", session_id);
    return 0;
}

uint32_t federated_register_client(uint32_t session_id, uint16_t node_id)
{
    int idx = find_session_idx(session_id);
    if (idx < 0) return 0;

    fed_session_t *s = &sessions[idx];

    if (s->client_count >= FED_MAX_CLIENTS) return 0;

    /* Find free client slot */
    int free_c = -1;
    for (int i = 0; i < FED_MAX_CLIENTS; i++) {
        if (!s->clients[i].in_use) {
            free_c = i;
            break;
        }
    }
    if (free_c < 0) return 0;

    fed_client_t *c = &s->clients[free_c];
    memset(c, 0, sizeof(fed_client_t));
    c->client_id = client_id_counter++;
    c->node_id = node_id;
    c->state = FED_CLIENT_IDLE;
    c->in_use = 1;

    s->client_count++;

    serial_printf("[federated] Client %u (node %u) registered for session %u\n",
                  c->client_id, (unsigned)node_id, session_id);
    return c->client_id;
}

int federated_unregister_client(uint32_t session_id, uint32_t client_id)
{
    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    fed_client_t *c = find_client(&sessions[idx], client_id);
    if (!c) return -1;

    c->in_use = 0;
    if (sessions[idx].client_count > 0)
        sessions[idx].client_count--;

    return 0;
}

int federated_start_round(uint32_t session_id)
{
    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    fed_session_t *s = &sessions[idx];

    /* Check minimum clients */
    if (s->client_count < s->config.min_clients_per_round) {
        serial_printf("[federated] Session %u: not enough clients (%u < %u)\n",
                      session_id, (unsigned)s->client_count,
                      (unsigned)s->config.min_clients_per_round);
        return -2;
    }

    /* Check max rounds */
    if (s->config.max_rounds > 0 &&
        s->total_rounds_completed >= s->config.max_rounds) {
        serial_printf("[federated] Session %u: max rounds reached\n",
                      session_id);
        return -3;
    }

    s->state = FED_SESSION_TRAINING;
    s->current_round++;

    fed_round_t *r = &current_rounds[idx];
    r->round_id = s->current_round;
    r->state = FED_ROUND_ACTIVE;
    r->participating_clients = s->client_count;
    r->updates_received = 0;
    r->avg_loss = 0.0f;

    /* Set all clients to training state */
    for (int i = 0; i < FED_MAX_CLIENTS; i++) {
        if (s->clients[i].in_use) {
            s->clients[i].state = FED_CLIENT_TRAINING;
            s->clients[i].weights_size = 0;
        }
    }

    global_stats.total_rounds++;

    serial_printf("[federated] Session %u: round %u started (%u clients)\n",
                  session_id, s->current_round,
                  (unsigned)s->client_count);
    return 0;
}

int federated_submit_update(uint32_t session_id, uint32_t client_id,
                            const uint8_t *weights, uint32_t size,
                            uint32_t samples_trained, float local_loss)
{
    if (!weights || size == 0) return -1;
    if (size > FED_MAX_WEIGHTS_SIZE) size = FED_MAX_WEIGHTS_SIZE;

    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    fed_session_t *s = &sessions[idx];
    fed_client_t *c = find_client(s, client_id);
    if (!c) return -1;

    if (c->state != FED_CLIENT_TRAINING) return -2;

    memcpy(c->weight_buffer, weights, size);
    c->weights_size = size;
    c->samples_trained = samples_trained;
    c->local_loss = local_loss;
    c->state = FED_CLIENT_SUBMITTED;

    current_rounds[idx].updates_received++;
    session_stats[idx].total_client_updates++;
    session_stats[idx].total_samples_trained += samples_trained;

    serial_printf("[federated] Session %u: client %u submitted "
                  "(samples=%u loss=%.4f)\n",
                  session_id, client_id, samples_trained,
                  (double)local_loss);
    return 0;
}

int federated_aggregate(uint32_t session_id)
{
    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    fed_session_t *s = &sessions[idx];
    fed_round_t *r = &current_rounds[idx];

    if (r->state != FED_ROUND_ACTIVE) return -2;
    if (r->updates_received == 0) return -3;

    r->state = FED_ROUND_AGGREGATING;

    /* Determine weight size (use first submitted client's size) */
    uint32_t weight_size = 0;
    for (int i = 0; i < FED_MAX_CLIENTS; i++) {
        if (s->clients[i].in_use &&
            s->clients[i].state == FED_CLIENT_SUBMITTED) {
            weight_size = s->clients[i].weights_size;
            break;
        }
    }
    if (weight_size == 0) return -4;

    /* Zero the global weights buffer */
    memset(s->global_weights, 0, weight_size);
    s->global_weights_size = weight_size;

    float total_loss = 0.0f;
    uint32_t total_samples = 0;
    uint32_t num_updates = 0;

    if (s->config.aggregation == FED_AGG_FEDAVG) {
        /*
         * FedAvg: weighted average by number of samples.
         * w_global = sum(n_k * w_k) / sum(n_k)
         */
        for (int i = 0; i < FED_MAX_CLIENTS; i++) {
            fed_client_t *c = &s->clients[i];
            if (!c->in_use || c->state != FED_CLIENT_SUBMITTED) continue;

            uint32_t n = c->samples_trained;
            if (n == 0) n = 1;  /* Avoid division by zero */
            total_samples += n;

            /* Weighted accumulation (byte-level for simulation) */
            for (uint32_t b = 0; b < weight_size; b++) {
                uint32_t val = (uint32_t)s->global_weights[b] +
                               ((uint32_t)c->weight_buffer[b] * n) / 256;
                if (val > 255) val = 255;
                s->global_weights[b] = (uint8_t)val;
            }

            total_loss += c->local_loss * (float)n;
            num_updates++;
        }

        /* Normalize by total samples */
        if (total_samples > 0) {
            r->avg_loss = total_loss / (float)total_samples;
        }
    } else {
        /* FedSGD or Median: simple average for now */
        for (int i = 0; i < FED_MAX_CLIENTS; i++) {
            fed_client_t *c = &s->clients[i];
            if (!c->in_use || c->state != FED_CLIENT_SUBMITTED) continue;

            for (uint32_t b = 0; b < weight_size; b++) {
                uint32_t val = (uint32_t)s->global_weights[b] +
                               (uint32_t)c->weight_buffer[b];
                if (val > 255) val = 255;
                s->global_weights[b] = (uint8_t)val;
            }

            total_loss += c->local_loss;
            total_samples += c->samples_trained;
            num_updates++;
        }

        if (num_updates > 0) {
            /* Average the weights */
            for (uint32_t b = 0; b < weight_size; b++) {
                s->global_weights[b] =
                    (uint8_t)((uint32_t)s->global_weights[b] / num_updates);
            }
            r->avg_loss = total_loss / (float)num_updates;
        }
    }

    /* Apply differential privacy noise if configured */
    if (s->config.privacy_epsilon > 0.0f) {
        federated_add_noise(s->global_weights, weight_size,
                            s->config.privacy_epsilon);
        session_stats[idx].noise_injections++;
    }

    /* Update global loss */
    float prev_loss = s->global_loss;
    s->global_loss = r->avg_loss;
    r->loss_delta = prev_loss - r->avg_loss;

    if (r->avg_loss < s->best_loss)
        s->best_loss = r->avg_loss;

    /* Complete the round */
    r->state = FED_ROUND_COMPLETED;
    s->total_rounds_completed++;
    session_stats[idx].rounds_completed = s->total_rounds_completed;
    session_stats[idx].current_loss = s->global_loss;
    session_stats[idx].best_loss = s->best_loss;

    global_stats.total_aggregations++;

    /* Reset client states */
    for (int i = 0; i < FED_MAX_CLIENTS; i++) {
        if (s->clients[i].in_use)
            s->clients[i].state = FED_CLIENT_IDLE;
    }

    serial_printf("[federated] Session %u round %u aggregated: "
                  "loss=%.4f delta=%.4f updates=%u\n",
                  session_id, r->round_id, (double)r->avg_loss,
                  (double)r->loss_delta, num_updates);

    /* Check convergence */
    federated_check_convergence(session_id);

    return 0;
}

int federated_get_global_model(uint32_t session_id, uint8_t *weights,
                               uint32_t max_size, uint32_t *actual_size)
{
    if (!weights || !actual_size) return -1;

    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    fed_session_t *s = &sessions[idx];
    uint32_t copy = s->global_weights_size;
    if (copy > max_size) copy = max_size;

    memcpy(weights, s->global_weights, copy);
    *actual_size = copy;
    return 0;
}

int federated_add_noise(uint8_t *weights, uint32_t size, float epsilon)
{
    if (!weights || size == 0 || epsilon <= 0.0f) return -1;

    /*
     * Simplified Laplace noise: scale = 1/epsilon.
     * For each weight byte, add noise proportional to 1/epsilon.
     * Larger epsilon = less noise = less privacy.
     */
    uint32_t noise_scale = (uint32_t)(256.0f / epsilon);
    if (noise_scale > 128) noise_scale = 128;
    if (noise_scale == 0) noise_scale = 1;

    for (uint32_t i = 0; i < size; i++) {
        int32_t noise = (int32_t)(dp_random() % (noise_scale * 2 + 1)) -
                        (int32_t)noise_scale;
        int32_t val = (int32_t)weights[i] + noise;
        if (val < 0) val = 0;
        if (val > 255) val = 255;
        weights[i] = (uint8_t)val;
    }

    return 0;
}

int federated_check_convergence(uint32_t session_id)
{
    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    fed_session_t *s = &sessions[idx];
    fed_round_t *r = &current_rounds[idx];

    if (s->config.convergence_threshold <= 0.0f) return 0;

    /* Check if loss delta is below threshold */
    float delta = r->loss_delta;
    if (delta < 0.0f) delta = -delta;  /* abs */

    if (s->total_rounds_completed >= 2 &&
        delta < s->config.convergence_threshold) {
        s->state = FED_SESSION_CONVERGED;
        global_stats.convergences++;
        serial_printf("[federated] Session %u CONVERGED at round %u "
                      "(loss=%.4f delta=%.4f)\n",
                      session_id, s->current_round,
                      (double)s->global_loss, (double)delta);
        return 1;  /* Converged */
    }

    return 0;  /* Not yet */
}

int federated_get_session(uint32_t session_id, fed_session_t *out)
{
    if (!out) return -1;
    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    memcpy(out, &sessions[idx], sizeof(fed_session_t));
    return 0;
}

uint32_t federated_list_sessions(fed_session_t *out, uint32_t max)
{
    if (!out || max == 0) return 0;

    uint32_t count = 0;
    for (int i = 0; i < FED_MAX_SESSIONS && count < max; i++) {
        if (sessions[i].in_use) {
            memcpy(&out[count], &sessions[i], sizeof(fed_session_t));
            count++;
        }
    }
    return count;
}

int federated_get_session_stats(uint32_t session_id,
                                fed_session_stats_t *out)
{
    if (!out) return -1;
    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    memcpy(out, &session_stats[idx], sizeof(fed_session_stats_t));
    return 0;
}

fed_global_stats_t *federated_get_global_stats(void)
{
    return &global_stats;
}

uint32_t federated_get_session_count(void)
{
    return global_stats.active_sessions;
}

int federated_get_current_round(uint32_t session_id, fed_round_t *out)
{
    if (!out) return -1;
    int idx = find_session_idx(session_id);
    if (idx < 0) return -1;

    memcpy(out, &current_rounds[idx], sizeof(fed_round_t));
    return 0;
}
