#ifndef KERNEL_FEDERATED_H
#define KERNEL_FEDERATED_H

#include <stdint.h>

/* Maximum limits */
#define FED_MAX_SESSIONS 8
#define FED_MAX_CLIENTS 16
#define FED_MAX_ROUNDS 256
#define FED_MAX_WEIGHTS_SIZE 4096     /* Max weight buffer per client */
#define FED_MAX_NAME_LEN 64

/* Aggregation strategy */
typedef enum {
    FED_AGG_FEDAVG = 0,              /* Federated Averaging (weighted by samples) */
    FED_AGG_FEDSGD = 1,              /* Federated SGD */
    FED_AGG_MEDIAN = 2               /* Coordinate-wise median (Byzantine-robust) */
} fed_aggregation_t;

/* Session state */
typedef enum {
    FED_SESSION_CREATED = 0,
    FED_SESSION_RECRUITING = 1,       /* Waiting for clients */
    FED_SESSION_TRAINING = 2,         /* Rounds in progress */
    FED_SESSION_CONVERGED = 3,        /* Training converged */
    FED_SESSION_STOPPED = 4
} fed_session_state_t;

/* Round state */
typedef enum {
    FED_ROUND_PENDING = 0,
    FED_ROUND_ACTIVE = 1,            /* Waiting for client updates */
    FED_ROUND_AGGREGATING = 2,        /* Aggregating updates */
    FED_ROUND_COMPLETED = 3
} fed_round_state_t;

/* Client state */
typedef enum {
    FED_CLIENT_IDLE = 0,
    FED_CLIENT_TRAINING = 1,
    FED_CLIENT_SUBMITTED = 2,
    FED_CLIENT_FAILED = 3
} fed_client_state_t;

/* Federated learning configuration */
typedef struct {
    uint8_t min_clients_per_round;    /* Minimum clients to start a round */
    uint16_t max_rounds;              /* Maximum training rounds (0=unlimited) */
    float convergence_threshold;      /* Stop if loss delta < threshold */
    float privacy_epsilon;            /* Differential privacy budget (0=disabled) */
    float privacy_delta;              /* DP delta parameter */
    fed_aggregation_t aggregation;    /* Aggregation strategy */
    uint32_t round_timeout_ms;        /* Max time per round */
} fed_config_t;

/* Client descriptor */
typedef struct {
    uint32_t client_id;
    uint16_t node_id;                 /* Cluster node ID */
    fed_client_state_t state;
    uint32_t samples_trained;         /* Local dataset size */
    float local_loss;                 /* Loss after local training */
    uint32_t weights_size;            /* Size of submitted weights */
    uint8_t weight_buffer[FED_MAX_WEIGHTS_SIZE];
    uint8_t in_use;
} fed_client_t;

/* Round descriptor */
typedef struct {
    uint32_t round_id;
    fed_round_state_t state;
    uint8_t participating_clients;
    uint8_t updates_received;
    float avg_loss;                   /* Average loss this round */
    float loss_delta;                 /* Change from previous round */
    uint32_t start_time;
    uint32_t end_time;
} fed_round_t;

/* Session descriptor */
typedef struct {
    uint32_t session_id;
    uint32_t model_id;
    char name[FED_MAX_NAME_LEN];
    fed_session_state_t state;
    fed_config_t config;
    fed_client_t clients[FED_MAX_CLIENTS];
    uint8_t client_count;
    uint32_t current_round;
    uint32_t total_rounds_completed;
    float global_loss;                /* Current global model loss */
    float best_loss;                  /* Best observed loss */
    uint32_t global_weights_size;
    uint8_t global_weights[FED_MAX_WEIGHTS_SIZE];
    uint32_t created_time;
    uint8_t in_use;
} fed_session_t;

/* Per-session statistics */
typedef struct {
    uint32_t session_id;
    uint32_t rounds_completed;
    uint32_t total_client_updates;
    uint32_t failed_updates;
    float initial_loss;
    float current_loss;
    float best_loss;
    uint32_t total_samples_trained;
    uint32_t noise_injections;        /* DP noise applications */
} fed_session_stats_t;

/* Global federated learning statistics */
typedef struct {
    uint32_t active_sessions;
    uint32_t total_sessions;
    uint32_t total_rounds;
    uint32_t total_aggregations;
    uint32_t convergences;
} fed_global_stats_t;

/* Core Federated Learning APIs */

/**
 * Initialize federated learning subsystem
 */
void federated_init(void);

/**
 * Create a new federated learning session
 */
uint32_t federated_create_session(uint32_t model_id, const char *name,
                                  const fed_config_t *config);

/**
 * Destroy a session
 */
int federated_destroy_session(uint32_t session_id);

/**
 * Register a client/node for a session
 */
uint32_t federated_register_client(uint32_t session_id, uint16_t node_id);

/**
 * Unregister a client
 */
int federated_unregister_client(uint32_t session_id, uint32_t client_id);

/**
 * Start a new training round
 */
int federated_start_round(uint32_t session_id);

/**
 * Client submits local model update after training
 */
int federated_submit_update(uint32_t session_id, uint32_t client_id,
                            const uint8_t *weights, uint32_t size,
                            uint32_t samples_trained, float local_loss);

/**
 * Aggregate client updates (FedAvg / FedSGD / Median)
 */
int federated_aggregate(uint32_t session_id);

/**
 * Get current global model weights
 */
int federated_get_global_model(uint32_t session_id, uint8_t *weights,
                               uint32_t max_size, uint32_t *actual_size);

/**
 * Apply differential privacy noise to weight buffer
 */
int federated_add_noise(uint8_t *weights, uint32_t size, float epsilon);

/**
 * Check if session has converged
 */
int federated_check_convergence(uint32_t session_id);

/**
 * Get session info
 */
int federated_get_session(uint32_t session_id, fed_session_t *out);

/**
 * List all sessions
 */
uint32_t federated_list_sessions(fed_session_t *sessions, uint32_t max);

/**
 * Get session statistics
 */
int federated_get_session_stats(uint32_t session_id, fed_session_stats_t *out);

/**
 * Get global statistics
 */
fed_global_stats_t *federated_get_global_stats(void);

/**
 * Get active session count
 */
uint32_t federated_get_session_count(void);

/**
 * Get current round info
 */
int federated_get_current_round(uint32_t session_id, fed_round_t *out);

#endif /* KERNEL_FEDERATED_H */
