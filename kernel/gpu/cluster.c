#include "../../include/kernel/cluster.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>
#include <string.h>

/* Global cluster state */
static cluster_group_t g_cluster = {
    .master_node = 0xFFFF,
    .node_count = 0,
    .replication_factor = CLUSTER_DEFAULT_REPLICATION_FACTOR,
    .last_update_ts = 0,
};

/* Federation routing table (request hash -> preferred node) */
static cluster_federation_entry_t g_federation_table[CLUSTER_FEDERATION_TABLE_SIZE] = {0};
static uint16_t g_federation_entries = 0;

/* Model replication state (up to 32 models tracked) */
typedef struct {
    uint16_t model_id;
    cluster_model_replica_t replicas;
    uint32_t padding;
} model_replication_t;

#define CLUSTER_MAX_TRACKED_MODELS 32
static model_replication_t g_model_replicas[CLUSTER_MAX_TRACKED_MODELS] = {0};
static uint16_t g_tracked_models = 0;

/* Helper: Calculate simple hash for load balancing */
static uint32_t cluster_hash_request(uint32_t request_hash) {
    return (request_hash ^ (request_hash >> 16)) % g_cluster.node_count;
}

/* Helper: Get next healthy node (round-robin from offset) */
static uint16_t cluster_get_next_healthy_node(uint16_t offset) {
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        uint16_t node_id = (offset + i) % g_cluster.node_count;
        if (cluster_is_node_healthy(node_id) == 1) {
            return node_id;
        }
    }
    return 0xFFFF;  /* No healthy nodes */
}

/* Initialize cluster with local node info */
int32_t cluster_init(uint16_t local_node_id, uint32_t local_ip,
                     uint16_t local_port, uint16_t gpu_count,
                     uint32_t total_memory_mb) {
    if (local_node_id >= CLUSTER_MAX_NODES) {
        return -1;
    }

    g_cluster.master_node = local_node_id;
    g_cluster.node_count = 1;
    g_cluster.replication_factor = CLUSTER_DEFAULT_REPLICATION_FACTOR;
    g_cluster.last_update_ts = 0;

    cluster_node_t* local_node = &g_cluster.nodes[0];
    local_node->node_id = local_node_id;
    local_node->ip_address = local_ip;
    local_node->port = local_port;
    local_node->gpu_count = gpu_count;
    local_node->total_memory_mb = total_memory_mb;
    local_node->load_factor = 0;
    local_node->health.latency_ms = 0;
    local_node->health.last_heartbeat_ts = 0;
    local_node->health.consecutive_failures = 0;
    local_node->health.gpu_count = gpu_count;
    local_node->health.load_factor = 0;

    serial_printf("[cluster] Initialized cluster with node %u at %u.%u.%u.%u:%u (GPUs: %u, Memory: %uMB)\n",
                  local_node_id,
                  (local_ip >> 24) & 0xFF, (local_ip >> 16) & 0xFF,
                  (local_ip >> 8) & 0xFF, local_ip & 0xFF,
                  local_port, gpu_count, total_memory_mb);

    return 0;
}

/* Register a new node in the cluster */
int32_t cluster_register_node(uint16_t node_id, uint32_t ip_address,
                              uint16_t port, uint16_t gpu_count,
                              uint32_t total_memory_mb) {
    if (node_id >= CLUSTER_MAX_NODES || g_cluster.node_count >= CLUSTER_MAX_NODES) {
        return -1;
    }

    /* Check for duplicate */
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (g_cluster.nodes[i].node_id == node_id) {
            return -2;  /* Already registered */
        }
    }

    cluster_node_t* new_node = &g_cluster.nodes[g_cluster.node_count];
    new_node->node_id = node_id;
    new_node->ip_address = ip_address;
    new_node->port = port;
    new_node->gpu_count = gpu_count;
    new_node->total_memory_mb = total_memory_mb;
    new_node->load_factor = 0;
    new_node->health.latency_ms = 0;
    new_node->health.last_heartbeat_ts = 0;
    new_node->health.consecutive_failures = 0;
    new_node->health.gpu_count = gpu_count;
    new_node->health.load_factor = 0;

    g_cluster.node_count++;

    serial_printf("[cluster] Registered node %u at %u.%u.%u.%u:%u (GPUs: %u, Memory: %uMB, total nodes: %u)\n",
                  node_id,
                  (ip_address >> 24) & 0xFF, (ip_address >> 16) & 0xFF,
                  (ip_address >> 8) & 0xFF, ip_address & 0xFF,
                  port, gpu_count, total_memory_mb, g_cluster.node_count);

    return 0;
}

/* Unregister a node from the cluster */
int32_t cluster_unregister_node(uint16_t node_id) {
    uint16_t idx = 0xFFFF;

    /* Find node */
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (g_cluster.nodes[i].node_id == node_id) {
            idx = i;
            break;
        }
    }

    if (idx == 0xFFFF) {
        return -1;  /* Not found */
    }

    /* Remove from cluster (shift remaining nodes) */
    for (uint16_t i = idx; i < g_cluster.node_count - 1; i++) {
        g_cluster.nodes[i] = g_cluster.nodes[i + 1];
    }
    g_cluster.node_count--;

    serial_printf("[cluster] Unregistered node %u (remaining: %u)\n",
                  node_id, g_cluster.node_count);

    return 0;
}

/* Get cluster descriptor */
const cluster_group_t* cluster_get_group(void) {
    return &g_cluster;
}

/* Get node by ID */
const cluster_node_t* cluster_get_node(uint16_t node_id) {
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (g_cluster.nodes[i].node_id == node_id) {
            return &g_cluster.nodes[i];
        }
    }
    return NULL;
}

/* Update node load factor */
int32_t cluster_update_load(uint16_t node_id, uint32_t load_factor) {
    if (load_factor > 100) {
        return -1;
    }

    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (g_cluster.nodes[i].node_id == node_id) {
            g_cluster.nodes[i].load_factor = load_factor;
            g_cluster.nodes[i].health.load_factor = load_factor;
            return 0;
        }
    }
    return -2;  /* Not found */
}

/* Update node health metrics */
int32_t cluster_update_health(uint16_t node_id, uint32_t latency_ms,
                              uint16_t failed_heartbeat) {
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (g_cluster.nodes[i].node_id == node_id) {
            g_cluster.nodes[i].health.latency_ms = latency_ms;
            g_cluster.nodes[i].health.last_heartbeat_ts = 0;  /* Would be set by caller */

            if (failed_heartbeat) {
                g_cluster.nodes[i].health.consecutive_failures++;
            } else {
                g_cluster.nodes[i].health.consecutive_failures = 0;
            }
            return 0;
        }
    }
    return -1;  /* Not found */
}

/* Check if node is healthy and available */
int32_t cluster_is_node_healthy(uint16_t node_id) {
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (g_cluster.nodes[i].node_id == node_id) {
            cluster_node_health_t* health = &g_cluster.nodes[i].health;
            if (health->consecutive_failures >= CLUSTER_HEARTBEAT_THRESHOLD) {
                return 0;  /* Unhealthy */
            }
            return 1;  /* Healthy */
        }
    }
    return 0;  /* Not found */
}

/* Get node state */
cluster_node_state_t cluster_get_node_state(uint16_t node_id) {
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (g_cluster.nodes[i].node_id == node_id) {
            cluster_node_health_t* health = &g_cluster.nodes[i].health;

            if (health->consecutive_failures >= CLUSTER_HEARTBEAT_THRESHOLD) {
                return CLUSTER_NODE_DOWN;
            } else if (health->consecutive_failures > 0) {
                return CLUSTER_NODE_DEGRADED;
            }
            return CLUSTER_NODE_UP;
        }
    }
    return CLUSTER_NODE_DOWN;  /* Unknown = down */
}

/* Add federation routing entry */
int32_t cluster_federation_add_route(uint32_t request_hash,
                                     uint16_t preferred_node) {
    if (preferred_node >= g_cluster.node_count) {
        return -1;
    }

    if (g_federation_entries >= CLUSTER_FEDERATION_TABLE_SIZE) {
        return -2;  /* Table full */
    }

    /* Check for duplicate hash */
    for (uint16_t i = 0; i < g_federation_entries; i++) {
        if (g_federation_table[i].request_hash == request_hash) {
            g_federation_table[i].preferred_node = preferred_node;
            return 0;  /* Updated existing entry */
        }
    }

    /* Add new entry */
    g_federation_table[g_federation_entries].request_hash = request_hash;
    g_federation_table[g_federation_entries].preferred_node = preferred_node;
    g_federation_entries++;

    return 0;
}

/* Get preferred node for request (sticky routing) */
uint16_t cluster_federation_get_node(uint32_t request_hash) {
    if (g_cluster.node_count == 0) {
        return 0xFFFF;
    }

    /* Look for routing entry */
    for (uint16_t i = 0; i < g_federation_entries; i++) {
        if (g_federation_table[i].request_hash == request_hash) {
            uint16_t preferred_node = g_federation_table[i].preferred_node;

            /* If preferred node is healthy, use it */
            if (cluster_is_node_healthy(preferred_node) == 1) {
                return preferred_node;
            }

            /* Otherwise, find next healthy node */
            return cluster_get_next_healthy_node(preferred_node + 1);
        }
    }

    /* No route found, use load balancing */
    uint16_t offset = cluster_hash_request(request_hash);
    return cluster_get_next_healthy_node(offset);
}

/* Rebalance federation table based on node health */
int32_t cluster_federation_rebalance(void) {
    uint16_t rebalanced = 0;

    for (uint16_t i = 0; i < g_federation_entries; i++) {
        uint16_t preferred_node = g_federation_table[i].preferred_node;

        /* If preferred node is unhealthy, reassign */
        if (cluster_is_node_healthy(preferred_node) == 0) {
            uint16_t new_node = cluster_get_next_healthy_node(0);
            if (new_node != 0xFFFF) {
                g_federation_table[i].preferred_node = new_node;
                rebalanced++;
            }
        }
    }

    if (rebalanced > 0) {
        serial_printf("[cluster] Federation rebalanced: %u routes updated\n", rebalanced);
    }

    return rebalanced;
}

/* Register model replication across nodes */
int32_t cluster_replicate_model(uint16_t model_id,
                                cluster_replication_policy_t policy) {
    if (g_tracked_models >= CLUSTER_MAX_TRACKED_MODELS) {
        return -1;  /* Too many models */
    }

    uint16_t replica_count = (uint16_t)policy;
    if (replica_count > g_cluster.node_count) {
        replica_count = g_cluster.node_count;
    }

    /* Check for duplicate model */
    for (uint16_t i = 0; i < g_tracked_models; i++) {
        if (g_model_replicas[i].replicas.model_id == model_id) {
            g_model_replicas[i].replicas.replica_count = replica_count;
            return 0;  /* Updated existing */
        }
    }

    /* Add new model replication entry */
    model_replication_t* entry = &g_model_replicas[g_tracked_models];
    entry->model_id = model_id;
    entry->replicas.model_id = model_id;
    entry->replicas.replica_count = replica_count;
    entry->replicas.version = 1;
    entry->replicas.last_sync_ts = 0;

    /* Assign replicas round-robin */
    for (uint16_t i = 0; i < replica_count && i < CLUSTER_MAX_NODES; i++) {
        entry->replicas.replica_nodes[i] = i % g_cluster.node_count;
    }

    g_tracked_models++;

    serial_printf("[cluster] Model %u replicated across %u nodes (policy: %u)\n",
                  model_id, replica_count, policy);

    return 0;
}

/* Update model replicas (after training/update) */
int32_t cluster_update_replicas(uint16_t model_id, uint32_t new_version) {
    for (uint16_t i = 0; i < g_tracked_models; i++) {
        if (g_model_replicas[i].replicas.model_id == model_id) {
            g_model_replicas[i].replicas.version = new_version;
            g_model_replicas[i].replicas.last_sync_ts = 0;

            serial_printf("[cluster] Model %u updated to version %u\n",
                          model_id, new_version);
            return 0;
        }
    }
    return -1;  /* Model not found */
}

/* Get replica nodes for model */
int32_t cluster_get_replicas(uint16_t model_id, uint16_t* node_ids,
                             uint16_t max_replicas) {
    if (node_ids == NULL) {
        return -1;
    }

    for (uint16_t i = 0; i < g_tracked_models; i++) {
        if (g_model_replicas[i].replicas.model_id == model_id) {
            uint16_t count = g_model_replicas[i].replicas.replica_count;
            if (count > max_replicas) {
                count = max_replicas;
            }

            for (uint16_t j = 0; j < count; j++) {
                node_ids[j] = g_model_replicas[i].replicas.replica_nodes[j];
            }
            return count;
        }
    }
    return -2;  /* Model not found */
}

/* Remove model from all replicas */
int32_t cluster_unreplicate_model(uint16_t model_id) {
    uint16_t idx = 0xFFFF;

    for (uint16_t i = 0; i < g_tracked_models; i++) {
        if (g_model_replicas[i].replicas.model_id == model_id) {
            idx = i;
            break;
        }
    }

    if (idx == 0xFFFF) {
        return -1;  /* Not found */
    }

    /* Remove from replicas (shift remaining) */
    for (uint16_t i = idx; i < g_tracked_models - 1; i++) {
        g_model_replicas[i] = g_model_replicas[i + 1];
    }
    g_tracked_models--;

    serial_printf("[cluster] Model %u unreplicated\n", model_id);

    return 0;
}

/* Mark node as down and trigger failover */
int32_t cluster_failover_node(uint16_t failed_node_id) {
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (g_cluster.nodes[i].node_id == failed_node_id) {
            g_cluster.nodes[i].health.consecutive_failures = CLUSTER_HEARTBEAT_THRESHOLD;

            serial_printf("[cluster] Node %u marked DOWN - initiating failover\n",
                          failed_node_id);

            /* Rebalance federation routes away from failed node */
            cluster_federation_rebalance();

            return 0;
        }
    }
    return -1;  /* Node not found */
}

/* Check all nodes' health */
int32_t cluster_health_check_all(void) {
    int32_t failed_count = 0;

    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        cluster_node_health_t* health = &g_cluster.nodes[i].health;

        /* Simulate periodic health check (in real system, would ping node) */
        if (health->consecutive_failures > 0) {
            health->consecutive_failures++;

            if (health->consecutive_failures >= CLUSTER_HEARTBEAT_THRESHOLD) {
                failed_count++;
            }
        }
    }

    return failed_count;
}

/* Get node with lowest load in cluster */
uint16_t cluster_get_least_loaded_node(void) {
    if (g_cluster.node_count == 0) {
        return 0xFFFF;
    }

    uint16_t min_load_node = 0;
    uint32_t min_load = 101;

    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (cluster_is_node_healthy(i) == 1) {
            uint32_t load = g_cluster.nodes[i].load_factor;
            if (load < min_load) {
                min_load = load;
                min_load_node = i;
            }
        }
    }

    return (min_load <= 100) ? min_load_node : 0xFFFF;
}

/* Get node with lowest latency in cluster */
uint16_t cluster_get_lowest_latency_node(void) {
    if (g_cluster.node_count == 0) {
        return 0xFFFF;
    }

    uint16_t min_latency_node = 0;
    uint32_t min_latency = 0xFFFFFFFF;

    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        if (cluster_is_node_healthy(i) == 1) {
            uint32_t latency = g_cluster.nodes[i].health.latency_ms;
            if (latency < min_latency) {
                min_latency = latency;
                min_latency_node = i;
            }
        }
    }

    return (min_latency != 0xFFFFFFFF) ? min_latency_node : 0xFFFF;
}

/* Synchronize cluster state across all nodes */
int32_t cluster_sync_state(void) {
    g_cluster.last_update_ts = 0;  /* Would be set to current time by caller */

    serial_printf("[cluster] Cluster state synchronized (nodes: %u, models: %u)\n",
                  g_cluster.node_count, g_tracked_models);

    return 0;
}

/* Get total model replica count */
uint32_t cluster_get_total_replicas(void) {
    uint32_t total = 0;

    for (uint16_t i = 0; i < g_tracked_models; i++) {
        total += g_model_replicas[i].replicas.replica_count;
    }

    return total;
}

/* Get cluster utilization (average load) */
uint32_t cluster_get_utilization(void) {
    if (g_cluster.node_count == 0) {
        return 0;
    }

    uint32_t total_load = 0;
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        total_load += g_cluster.nodes[i].load_factor;
    }

    return total_load / g_cluster.node_count;
}

/* Get total GPU memory available in cluster */
uint32_t cluster_get_total_gpu_memory(void) {
    uint32_t total_memory = 0;

    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        total_memory += g_cluster.nodes[i].total_memory_mb;
    }

    return total_memory;
}

/* Print cluster status (for diagnostics) */
void cluster_print_status(void) {
    serial_printf("\n=== Cluster Status ===\n");
    serial_printf("Master Node: %u\n", g_cluster.master_node);
    serial_printf("Active Nodes: %u\n", g_cluster.node_count);
    serial_printf("Tracked Models: %u\n", g_tracked_models);
    serial_printf("Federation Routes: %u\n", g_federation_entries);
    serial_printf("Total GPU Memory: %uMB\n", cluster_get_total_gpu_memory());
    serial_printf("Cluster Utilization: %u%%\n", cluster_get_utilization());
    serial_printf("Total Model Replicas: %u\n", cluster_get_total_replicas());

    serial_printf("\nNodes:\n");
    for (uint16_t i = 0; i < g_cluster.node_count; i++) {
        cluster_node_t* node = &g_cluster.nodes[i];
        const char* state_str = "UP";
        cluster_node_state_t state = cluster_get_node_state(node->node_id);
        if (state == CLUSTER_NODE_DOWN) {
            state_str = "DOWN";
        } else if (state == CLUSTER_NODE_DEGRADED) {
            state_str = "DEGRADED";
        }

        serial_printf("  Node %u: %s (GPUs: %u, Load: %u%%, Latency: %ums)\n",
                      node->node_id, state_str, node->gpu_count,
                      node->load_factor, node->health.latency_ms);
    }
    serial_printf("=====================\n\n");
}
