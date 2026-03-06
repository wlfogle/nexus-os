#include "../../include/kernel/tenant.h"
#include "../../include/kernel/serial.h"
#include <string.h>

typedef struct {
    tenant_t tenant;
    tenant_stats_t stats;
} tenant_entry_t;

static tenant_entry_t tenants[TENANT_MAX_TENANTS];
static tenant_global_stats_t global_stats = {0};
static uint32_t tenant_id_counter = 1;

/* Round-robin index for fair-share tiebreaking */
static uint32_t fair_rr_idx = 0;

static tenant_entry_t *find_tenant(uint32_t tenant_id)
{
    for (int i = 0; i < TENANT_MAX_TENANTS; i++) {
        if (tenants[i].tenant.in_use &&
            tenants[i].tenant.tenant_id == tenant_id)
            return &tenants[i];
    }
    return (void *)0;
}

void tenant_init(void)
{
    memset(tenants, 0, sizeof(tenants));
    memset(&global_stats, 0, sizeof(tenant_global_stats_t));
    tenant_id_counter = 1;
    fair_rr_idx = 0;

    serial_puts("[tenant] Multi-tenant resource manager initialized\n");
}

uint32_t tenant_create(const char *name, const tenant_quota_t *quota,
                       uint16_t priority_weight)
{
    if (!name || !quota) return 0;
    if (priority_weight == 0) priority_weight = 1;
    if (priority_weight > 100) priority_weight = 100;

    int free_idx = -1;
    for (int i = 0; i < TENANT_MAX_TENANTS; i++) {
        if (!tenants[i].tenant.in_use) {
            free_idx = i;
            break;
        }
    }
    if (free_idx < 0) return 0;

    tenant_entry_t *te = &tenants[free_idx];
    memset(te, 0, sizeof(tenant_entry_t));

    te->tenant.tenant_id = tenant_id_counter++;
    te->tenant.state = TENANT_ACTIVE;
    te->tenant.priority_weight = priority_weight;
    te->tenant.in_use = 1;

    memcpy(&te->tenant.quota, quota, sizeof(tenant_quota_t));

    uint32_t nlen = 0;
    while (name[nlen] && nlen < TENANT_MAX_NAME_LEN - 1) nlen++;
    memcpy(te->tenant.name, name, nlen);
    te->tenant.name[nlen] = '\0';

    te->stats.tenant_id = te->tenant.tenant_id;

    global_stats.active_tenants++;

    serial_printf("[tenant] Tenant '%s' created (id=%u weight=%u)\n",
                  te->tenant.name, te->tenant.tenant_id,
                  (unsigned)priority_weight);
    return te->tenant.tenant_id;
}

int tenant_destroy(uint32_t tenant_id)
{
    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    serial_printf("[tenant] Destroying tenant '%s' (id=%u)\n",
                  te->tenant.name, tenant_id);

    te->tenant.in_use = 0;
    te->tenant.state = TENANT_INACTIVE;

    if (global_stats.active_tenants > 0)
        global_stats.active_tenants--;

    return 0;
}

int tenant_get(uint32_t tenant_id, tenant_t *out)
{
    if (!out) return -1;
    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    memcpy(out, &te->tenant, sizeof(tenant_t));
    return 0;
}

uint32_t tenant_list(tenant_t *out, uint32_t max_tenants)
{
    if (!out || max_tenants == 0) return 0;

    uint32_t count = 0;
    for (int i = 0; i < TENANT_MAX_TENANTS && count < max_tenants; i++) {
        if (tenants[i].tenant.in_use) {
            memcpy(&out[count], &tenants[i].tenant, sizeof(tenant_t));
            count++;
        }
    }
    return count;
}

int tenant_set_quota(uint32_t tenant_id, const tenant_quota_t *quota)
{
    if (!quota) return -1;
    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    memcpy(&te->tenant.quota, quota, sizeof(tenant_quota_t));
    return 0;
}

int tenant_get_quota(uint32_t tenant_id, tenant_quota_t *out)
{
    if (!out) return -1;
    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    memcpy(out, &te->tenant.quota, sizeof(tenant_quota_t));
    return 0;
}

int tenant_check_quota(uint32_t tenant_id, tenant_resource_t resource,
                       uint32_t amount)
{
    if (resource >= TENANT_RES_COUNT) return 0;

    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return 0;

    if (te->tenant.state != TENANT_ACTIVE) return 0;

    global_stats.total_quota_checks++;

    uint32_t limit = te->tenant.quota.limits[resource];
    uint32_t current = te->tenant.usage.current[resource];

    if (current + amount > limit) {
        global_stats.total_quota_denials++;
        te->stats.quota_violations++;
        return 0;  /* Denied */
    }

    return 1;  /* Allowed */
}

int tenant_account_usage(uint32_t tenant_id, tenant_resource_t resource,
                         uint32_t amount)
{
    if (resource >= TENANT_RES_COUNT) return -1;

    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    te->tenant.usage.current[resource] += amount;
    te->tenant.usage.cumulative[resource] += amount;

    if (te->tenant.usage.current[resource] > te->tenant.usage.peak[resource])
        te->tenant.usage.peak[resource] = te->tenant.usage.current[resource];

    return 0;
}

int tenant_release_usage(uint32_t tenant_id, tenant_resource_t resource,
                         uint32_t amount)
{
    if (resource >= TENANT_RES_COUNT) return -1;

    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    if (te->tenant.usage.current[resource] >= amount)
        te->tenant.usage.current[resource] -= amount;
    else
        te->tenant.usage.current[resource] = 0;

    return 0;
}

int tenant_get_usage(uint32_t tenant_id, tenant_usage_t *out)
{
    if (!out) return -1;
    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    memcpy(out, &te->tenant.usage, sizeof(tenant_usage_t));
    return 0;
}

int tenant_set_state(uint32_t tenant_id, tenant_state_t state)
{
    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    te->tenant.state = state;
    serial_printf("[tenant] Tenant %u state -> %d\n", tenant_id, state);
    return 0;
}

int tenant_set_priority(uint32_t tenant_id, uint16_t weight)
{
    if (weight == 0) weight = 1;
    if (weight > 100) weight = 100;

    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    te->tenant.priority_weight = weight;
    return 0;
}

uint32_t tenant_select_fair(tenant_resource_t resource)
{
    if (resource >= TENANT_RES_COUNT) return 0;

    global_stats.total_fair_share_selections++;

    /*
     * Weighted fair-share: select the active tenant with the lowest
     * (current_usage / weight) ratio for the requested resource.
     * Ties broken by round-robin.
     */
    uint32_t best_id = 0;
    uint32_t best_score = 0xFFFFFFFF;
    int found = 0;

    for (int i = 0; i < TENANT_MAX_TENANTS; i++) {
        int idx = ((int)fair_rr_idx + i) % TENANT_MAX_TENANTS;
        tenant_t *t = &tenants[idx].tenant;

        if (!t->in_use || t->state != TENANT_ACTIVE) continue;
        if (t->priority_weight == 0) continue;

        /* Check the tenant still has quota headroom */
        uint32_t limit = t->quota.limits[resource];
        uint32_t current = t->usage.current[resource];
        if (current >= limit) continue;

        /* Score = current_usage * 100 / weight (lower = more deserving) */
        uint32_t score = (current * 100) / t->priority_weight;

        if (!found || score < best_score) {
            best_score = score;
            best_id = t->tenant_id;
            found = 1;
        }
    }

    fair_rr_idx = (fair_rr_idx + 1) % TENANT_MAX_TENANTS;

    return best_id;
}

int tenant_set_container(uint32_t tenant_id, uint32_t container_id)
{
    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    te->tenant.container_id = container_id;
    return 0;
}

int tenant_get_stats(uint32_t tenant_id, tenant_stats_t *out)
{
    if (!out) return -1;
    tenant_entry_t *te = find_tenant(tenant_id);
    if (!te) return -1;

    /* Populate from current usage */
    te->stats.active_models = te->tenant.usage.current[TENANT_RES_MODELS];
    te->stats.active_endpoints = te->tenant.usage.current[TENANT_RES_ENDPOINTS];
    te->stats.total_inferences = te->tenant.usage.cumulative[TENANT_RES_INFER_PER_SEC];
    te->stats.total_training_jobs = te->tenant.usage.cumulative[TENANT_RES_TRAINING_JOBS];
    te->stats.gpu_seconds_used = te->tenant.usage.cumulative[TENANT_RES_GPU];

    memcpy(out, &te->stats, sizeof(tenant_stats_t));
    return 0;
}

tenant_global_stats_t *tenant_get_global_stats(void)
{
    return &global_stats;
}

uint32_t tenant_get_count(void)
{
    return global_stats.active_tenants;
}
