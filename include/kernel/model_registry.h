#ifndef KERNEL_MODEL_REGISTRY_H
#define KERNEL_MODEL_REGISTRY_H

#include <stdint.h>

/* Maximum limits */
#define REGISTRY_MAX_ENTRIES 64
#define REGISTRY_MAX_NAME_LEN 64
#define REGISTRY_MAX_VERSION_LEN 16
#define REGISTRY_MAX_TAGS 4
#define REGISTRY_MAX_TAG_LEN 32
#define REGISTRY_MAX_ACCESS_TENANTS 8
#define REGISTRY_HASH_SIZE 32

/* Model version state */
typedef enum {
    REGISTRY_PUBLISHED = 0,
    REGISTRY_DEPRECATED = 1,
    REGISTRY_ARCHIVED = 2
} registry_version_state_t;

/* Model format (mirrors model_runtime.h enum values) */
typedef enum {
    REG_FORMAT_ONNX = 1,
    REG_FORMAT_TENSORRT = 2,
    REG_FORMAT_TFLITE = 3,
    REG_FORMAT_NCNN = 4,
    REG_FORMAT_CUSTOM = 5
} registry_format_t;

/* Registry entry — one model version in the catalog */
typedef struct {
    uint32_t entry_id;
    char name[REGISTRY_MAX_NAME_LEN];
    char version[REGISTRY_MAX_VERSION_LEN];
    registry_format_t format;
    registry_version_state_t state;
    uint32_t owner_tenant_id;       /* Tenant that published the model */
    uint32_t model_size;            /* Size in bytes */
    uint8_t hash[REGISTRY_HASH_SIZE]; /* Content hash */
    char tags[REGISTRY_MAX_TAGS][REGISTRY_MAX_TAG_LEN];
    uint8_t tag_count;
    uint32_t access_tenants[REGISTRY_MAX_ACCESS_TENANTS]; /* Tenants with read access */
    uint8_t access_tenant_count;
    uint32_t download_count;
    uint32_t deploy_count;
    uint32_t created_time;
    uint32_t updated_time;
    uint8_t in_use;
} registry_entry_t;

/* Registry search query */
typedef struct {
    const char *name_prefix;        /* Match name starting with (NULL=any) */
    const char *tag;                /* Match entries with this tag (NULL=any) */
    registry_format_t format;       /* Match format (0=any) */
    uint32_t tenant_id;             /* Filter by owner tenant (0=any) */
    registry_version_state_t state; /* Filter by state */
    uint8_t include_deprecated;     /* Include deprecated entries */
    uint8_t include_archived;       /* Include archived entries */
} registry_query_t;

/* Global registry statistics */
typedef struct {
    uint32_t total_entries;
    uint32_t published_entries;
    uint32_t deprecated_entries;
    uint32_t archived_entries;
    uint32_t total_downloads;
    uint32_t total_deployments;
    uint32_t total_searches;
} registry_stats_t;

/* Core Model Registry APIs */

/**
 * Initialize model registry
 */
void model_registry_init(void);

/**
 * Publish a model to the registry
 */
uint32_t registry_publish(const char *name, const char *version,
                          registry_format_t format, uint32_t tenant_id,
                          uint32_t model_size, const uint8_t *hash);

/**
 * Deprecate a model entry (still accessible but marked for removal)
 */
int registry_deprecate(uint32_t entry_id);

/**
 * Archive a model entry (no longer available for deployment)
 */
int registry_archive(uint32_t entry_id);

/**
 * Restore an archived or deprecated entry to published state
 */
int registry_restore(uint32_t entry_id);

/**
 * Get entry by ID
 */
int registry_get(uint32_t entry_id, registry_entry_t *out);

/**
 * List entries for a tenant (0 = all tenants)
 */
uint32_t registry_list(uint32_t tenant_id, registry_entry_t *entries,
                       uint32_t max_entries);

/**
 * Search entries by tag
 */
uint32_t registry_search_by_tag(const char *tag, registry_entry_t *entries,
                                uint32_t max_entries);

/**
 * Search entries by name prefix
 */
uint32_t registry_search_by_name(const char *prefix, registry_entry_t *entries,
                                 uint32_t max_entries);

/**
 * Add a tag to an entry
 */
int registry_set_tag(uint32_t entry_id, const char *tag);

/**
 * Remove a tag from an entry
 */
int registry_remove_tag(uint32_t entry_id, const char *tag);

/**
 * Check if a tenant has access to an entry
 */
int registry_check_access(uint32_t entry_id, uint32_t tenant_id);

/**
 * Grant access to a tenant
 */
int registry_grant_access(uint32_t entry_id, uint32_t tenant_id);

/**
 * Revoke access from a tenant
 */
int registry_revoke_access(uint32_t entry_id, uint32_t tenant_id);

/**
 * Deploy model from registry to a serving endpoint
 * Returns the serving endpoint_id, or 0 on failure.
 */
uint32_t registry_deploy(uint32_t entry_id, uint16_t endpoint_port);

/**
 * Record a download of the model
 */
int registry_record_download(uint32_t entry_id);

/**
 * Get registry-wide statistics
 */
registry_stats_t *registry_get_stats(void);

/**
 * Get total entry count
 */
uint32_t registry_get_count(void);

/**
 * Delete an entry permanently
 */
int registry_delete(uint32_t entry_id);

#endif /* KERNEL_MODEL_REGISTRY_H */
