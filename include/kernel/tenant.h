#ifndef KERNEL_TENANT_H
#define KERNEL_TENANT_H

#include <stdint.h>

/* Maximum limits */
#define TENANT_MAX_TENANTS 16
#define TENANT_MAX_NAME_LEN 64

/* Resource types for quota checking and accounting */
typedef enum {
    TENANT_RES_GPU = 0,             /* GPU device count */
    TENANT_RES_MEMORY_MB = 1,       /* Memory in megabytes */
    TENANT_RES_MODELS = 2,          /* Loaded model count */
    TENANT_RES_INFER_PER_SEC = 3,   /* Inference requests per second */
    TENANT_RES_TRAINING_JOBS = 4,   /* Concurrent training jobs */
    TENANT_RES_ENDPOINTS = 5,       /* Serving endpoints */
    TENANT_RES_PIPELINES = 6,       /* Active pipelines */
    TENANT_RES_COUNT = 7            /* Sentinel — number of resource types */
} tenant_resource_t;

/* Tenant state */
typedef enum {
    TENANT_INACTIVE = 0,
    TENANT_ACTIVE = 1,
    TENANT_SUSPENDED = 2
} tenant_state_t;

/* Resource quota — per-resource limits */
typedef struct {
    uint32_t limits[TENANT_RES_COUNT];  /* Max allowed per resource type */
} tenant_quota_t;

/* Real-time resource usage */
typedef struct {
    uint32_t current[TENANT_RES_COUNT]; /* Current usage per resource type */
    uint32_t peak[TENANT_RES_COUNT];    /* Peak usage per resource type */
    uint32_t cumulative[TENANT_RES_COUNT]; /* Cumulative usage (for billing) */
} tenant_usage_t;

/* Tenant descriptor */
typedef struct {
    uint32_t tenant_id;
    char name[TENANT_MAX_NAME_LEN];
    tenant_state_t state;
    tenant_quota_t quota;
    tenant_usage_t usage;
    uint16_t priority_weight;       /* Fair-share weight (1-100) */
    uint32_t container_id;          /* Associated container (0 = none) */
    uint32_t created_time;
    uint8_t in_use;
} tenant_t;

/* Per-tenant statistics */
typedef struct {
    uint32_t tenant_id;
    uint32_t total_inferences;
    uint32_t total_training_jobs;
    uint32_t quota_violations;      /* Times quota was exceeded/denied */
    uint32_t active_models;
    uint32_t active_endpoints;
    uint32_t gpu_seconds_used;      /* Cumulative GPU time */
} tenant_stats_t;

/* Global multi-tenant statistics */
typedef struct {
    uint32_t active_tenants;
    uint32_t total_quota_checks;
    uint32_t total_quota_denials;
    uint32_t total_fair_share_selections;
} tenant_global_stats_t;

/* Core Multi-Tenant APIs */

/**
 * Initialize multi-tenant resource manager
 */
void tenant_init(void);

/**
 * Create a new tenant with resource quotas
 */
uint32_t tenant_create(const char *name, const tenant_quota_t *quota,
                       uint16_t priority_weight);

/**
 * Destroy a tenant
 */
int tenant_destroy(uint32_t tenant_id);

/**
 * Get tenant by ID
 */
int tenant_get(uint32_t tenant_id, tenant_t *out);

/**
 * List all tenants
 */
uint32_t tenant_list(tenant_t *tenants, uint32_t max_tenants);

/**
 * Set/update tenant resource quota
 */
int tenant_set_quota(uint32_t tenant_id, const tenant_quota_t *quota);

/**
 * Get tenant quota
 */
int tenant_get_quota(uint32_t tenant_id, tenant_quota_t *out);

/**
 * Check if tenant can allocate resource (returns 1=yes, 0=denied)
 */
int tenant_check_quota(uint32_t tenant_id, tenant_resource_t resource,
                       uint32_t amount);

/**
 * Account resource usage (increment current usage)
 */
int tenant_account_usage(uint32_t tenant_id, tenant_resource_t resource,
                         uint32_t amount);

/**
 * Release resource usage (decrement current usage)
 */
int tenant_release_usage(uint32_t tenant_id, tenant_resource_t resource,
                         uint32_t amount);

/**
 * Get current usage for tenant
 */
int tenant_get_usage(uint32_t tenant_id, tenant_usage_t *out);

/**
 * Set tenant state (active, suspended)
 */
int tenant_set_state(uint32_t tenant_id, tenant_state_t state);

/**
 * Set tenant priority weight for fair scheduling
 */
int tenant_set_priority(uint32_t tenant_id, uint16_t weight);

/**
 * Weighted fair-share selection: returns tenant_id of tenant that should
 * be scheduled next for the given resource type
 */
uint32_t tenant_select_fair(tenant_resource_t resource);

/**
 * Associate a container with a tenant
 */
int tenant_set_container(uint32_t tenant_id, uint32_t container_id);

/**
 * Get per-tenant statistics
 */
int tenant_get_stats(uint32_t tenant_id, tenant_stats_t *out);

/**
 * Get global multi-tenant statistics
 */
tenant_global_stats_t *tenant_get_global_stats(void);

/**
 * Get active tenant count
 */
uint32_t tenant_get_count(void);

#endif /* KERNEL_TENANT_H */
