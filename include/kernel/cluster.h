#ifndef KERNEL_CLUSTER_H
#define KERNEL_CLUSTER_H

#include <stdint.h>
#include <stddef.h>

/* Maximum number of nodes in a cluster */
#define CLUSTER_MAX_NODES 16

/* Maximum number of federation routing entries */
#define CLUSTER_FEDERATION_TABLE_SIZE 256

/* Heartbeat interval in milliseconds */
#define CLUSTER_HEARTBEAT_INTERVAL_MS 5000

/* Heartbeat timeout threshold (failed beats before node marked down) */
#define CLUSTER_HEARTBEAT_THRESHOLD 5

/* Cross-cluster ping timeout in milliseconds */
#define CLUSTER_PING_TIMEOUT_MS 500

/* Node replication factor (copies of model across cluster) */
#define CLUSTER_DEFAULT_REPLICATION_FACTOR 3

/* Health monitoring constants */
#define CLUSTER_MAX_LATENCY_SAMPLES 32

/* Request routing table entry */
typedef struct {
    uint32_t request_hash;      /* Hash of request (model+input signature) */
    uint16_t preferred_node;    /* Preferred node ID (0-15) */
    uint16_t padding;
} cluster_federation_entry_t;

/* Node health metrics */
typedef struct {
    uint32_t latency_ms;        /* Average latency to node (milliseconds) */
    uint32_t last_heartbeat_ts; /* Last heartbeat timestamp (kernel ticks) */
    uint16_t consecutive_failures; /* Consecutive heartbeat failures */
    uint16_t gpu_count;         /* Number of GPUs on node */
    uint32_t load_factor;       /* Load percentage (0-100) */
} cluster_node_health_t;

/* Cluster node descriptor */
typedef struct {
    uint16_t node_id;           /* Node ID (0-15) */
    uint16_t padding;
    uint32_t ip_address;        /* IPv4 address (network byte order) */
    uint16_t port;              /* Communication port */
    uint16_t gpu_count;         /* Number of GPUs */
    uint32_t total_memory_mb;   /* Total GPU memory in MB */
    uint32_t load_factor;       /* Current load (0-100) */
    cluster_node_health_t health; /* Health metrics */
} cluster_node_t;

/* Cluster group descriptor */
typedef struct {
    uint16_t master_node;       /* Master node ID */
    uint16_t node_count;        /* Number of active nodes */
    cluster_node_t nodes[CLUSTER_MAX_NODES]; /* Node list */
    uint32_t replication_factor; /* Default replication factor */
    uint32_t last_update_ts;    /* Last cluster update timestamp */
} cluster_group_t;

/* Replication policy */
typedef enum {
    CLUSTER_REPLICATION_NONE = 0,
    CLUSTER_REPLICATION_2X = 2,
    CLUSTER_REPLICATION_3X = 3,
} cluster_replication_policy_t;

/* Node state */
typedef enum {
    CLUSTER_NODE_UP = 0,
    CLUSTER_NODE_DEGRADED = 1,
    CLUSTER_NODE_DOWN = 2,
} cluster_node_state_t;

/* Model replication state */
typedef struct {
    uint16_t model_id;          /* Model ID */
    uint16_t replica_count;     /* Number of replicas */
    uint16_t replica_nodes[CLUSTER_MAX_NODES]; /* Node IDs holding replicas */
    uint32_t version;           /* Model version (for coherency) */
    uint32_t last_sync_ts;      /* Last synchronization timestamp */
} cluster_model_replica_t;

/* Cluster management API */

/* Initialize cluster with local node info */
int32_t cluster_init(uint16_t local_node_id, uint32_t local_ip, 
                     uint16_t local_port, uint16_t gpu_count,
                     uint32_t total_memory_mb);

/* Register a new node in the cluster */
int32_t cluster_register_node(uint16_t node_id, uint32_t ip_address,
                              uint16_t port, uint16_t gpu_count,
                              uint32_t total_memory_mb);

/* Unregister a node from the cluster */
int32_t cluster_unregister_node(uint16_t node_id);

/* Get cluster descriptor */
const cluster_group_t* cluster_get_group(void);

/* Get node by ID */
const cluster_node_t* cluster_get_node(uint16_t node_id);

/* Update node load factor */
int32_t cluster_update_load(uint16_t node_id, uint32_t load_factor);

/* Update node health metrics (called by monitoring thread) */
int32_t cluster_update_health(uint16_t node_id, uint32_t latency_ms,
                              uint16_t failed_heartbeat);

/* Check if node is healthy and available */
int32_t cluster_is_node_healthy(uint16_t node_id);

/* Get node state */
cluster_node_state_t cluster_get_node_state(uint16_t node_id);

/* Federation and routing */

/* Add federation routing entry (request hash -> preferred node) */
int32_t cluster_federation_add_route(uint32_t request_hash,
                                     uint16_t preferred_node);

/* Get preferred node for request (or load-balanced node if preferred is down) */
uint16_t cluster_federation_get_node(uint32_t request_hash);

/* Update federation table based on node health */
int32_t cluster_federation_rebalance(void);

/* Model replication */

/* Register model replication across nodes */
int32_t cluster_replicate_model(uint16_t model_id, 
                                cluster_replication_policy_t policy);

/* Update model replicas (after training/update) */
int32_t cluster_update_replicas(uint16_t model_id, uint32_t new_version);

/* Get replica nodes for model (preferred order by health) */
int32_t cluster_get_replicas(uint16_t model_id, uint16_t* node_ids,
                             uint16_t max_replicas);

/* Remove model from all replicas */
int32_t cluster_unreplicate_model(uint16_t model_id);

/* Failover and health management */

/* Mark node as down and trigger failover */
int32_t cluster_failover_node(uint16_t failed_node_id);

/* Check all nodes' health (called by monitoring thread) */
int32_t cluster_health_check_all(void);

/* Get node with lowest load in cluster */
uint16_t cluster_get_least_loaded_node(void);

/* Get node with lowest latency in cluster */
uint16_t cluster_get_lowest_latency_node(void);

/* Synchronize cluster state across all nodes */
int32_t cluster_sync_state(void);

/* Statistics and diagnostics */

/* Get total model replica count */
uint32_t cluster_get_total_replicas(void);

/* Get cluster utilization (average load) */
uint32_t cluster_get_utilization(void);

/* Get total GPU memory available in cluster */
uint32_t cluster_get_total_gpu_memory(void);

/* Print cluster status (for diagnostics) */
void cluster_print_status(void);

#endif /* KERNEL_CLUSTER_H */
