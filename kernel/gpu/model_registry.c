#include "../../include/kernel/model_registry.h"
#include "../../include/kernel/model_serving.h"
#include "../../include/kernel/serial.h"
#include <string.h>

static registry_entry_t entries[REGISTRY_MAX_ENTRIES];
static registry_stats_t stats = {0};
static uint32_t entry_id_counter = 1;

static registry_entry_t *find_entry(uint32_t entry_id)
{
    for (int i = 0; i < REGISTRY_MAX_ENTRIES; i++) {
        if (entries[i].in_use && entries[i].entry_id == entry_id)
            return &entries[i];
    }
    return (void *)0;
}

/* Simple string prefix match */
static int str_starts_with(const char *str, const char *prefix)
{
    while (*prefix) {
        if (*str != *prefix) return 0;
        str++;
        prefix++;
    }
    return 1;
}

/* Simple string equality */
static int str_equal(const char *a, const char *b)
{
    while (*a && *b) {
        if (*a != *b) return 0;
        a++;
        b++;
    }
    return (*a == *b);
}

static int entry_has_tag(registry_entry_t *e, const char *tag)
{
    for (uint8_t t = 0; t < e->tag_count; t++) {
        if (str_equal(e->tags[t], tag))
            return 1;
    }
    return 0;
}

void model_registry_init(void)
{
    memset(entries, 0, sizeof(entries));
    memset(&stats, 0, sizeof(registry_stats_t));
    entry_id_counter = 1;

    serial_puts("[registry] Model registry initialized\n");
}

uint32_t registry_publish(const char *name, const char *version,
                          registry_format_t format, uint32_t tenant_id,
                          uint32_t model_size, const uint8_t *hash)
{
    if (!name || !version) return 0;

    int free_idx = -1;
    for (int i = 0; i < REGISTRY_MAX_ENTRIES; i++) {
        if (!entries[i].in_use) {
            free_idx = i;
            break;
        }
    }
    if (free_idx < 0) return 0;

    registry_entry_t *e = &entries[free_idx];
    memset(e, 0, sizeof(registry_entry_t));

    e->entry_id = entry_id_counter++;
    e->format = format;
    e->state = REGISTRY_PUBLISHED;
    e->owner_tenant_id = tenant_id;
    e->model_size = model_size;
    e->in_use = 1;

    /* Copy name */
    uint32_t nlen = 0;
    while (name[nlen] && nlen < REGISTRY_MAX_NAME_LEN - 1) nlen++;
    memcpy(e->name, name, nlen);
    e->name[nlen] = '\0';

    /* Copy version */
    uint32_t vlen = 0;
    while (version[vlen] && vlen < REGISTRY_MAX_VERSION_LEN - 1) vlen++;
    memcpy(e->version, version, vlen);
    e->version[vlen] = '\0';

    /* Copy hash */
    if (hash) {
        memcpy(e->hash, hash, REGISTRY_HASH_SIZE);
    }

    /* Owner tenant always has access */
    if (tenant_id > 0) {
        e->access_tenants[0] = tenant_id;
        e->access_tenant_count = 1;
    }

    stats.total_entries++;
    stats.published_entries++;

    serial_printf("[registry] Published '%s' v%s (id=%u tenant=%u format=%d "
                  "size=%u)\n",
                  e->name, e->version, e->entry_id, tenant_id,
                  (int)format, model_size);
    return e->entry_id;
}

int registry_deprecate(uint32_t entry_id)
{
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;
    if (e->state != REGISTRY_PUBLISHED) return -2;

    e->state = REGISTRY_DEPRECATED;
    if (stats.published_entries > 0) stats.published_entries--;
    stats.deprecated_entries++;

    serial_printf("[registry] Deprecated '%s' v%s (id=%u)\n",
                  e->name, e->version, entry_id);
    return 0;
}

int registry_archive(uint32_t entry_id)
{
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;

    if (e->state == REGISTRY_PUBLISHED && stats.published_entries > 0)
        stats.published_entries--;
    else if (e->state == REGISTRY_DEPRECATED && stats.deprecated_entries > 0)
        stats.deprecated_entries--;

    e->state = REGISTRY_ARCHIVED;
    stats.archived_entries++;

    serial_printf("[registry] Archived '%s' v%s (id=%u)\n",
                  e->name, e->version, entry_id);
    return 0;
}

int registry_restore(uint32_t entry_id)
{
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;
    if (e->state == REGISTRY_PUBLISHED) return 0;  /* Already published */

    if (e->state == REGISTRY_DEPRECATED && stats.deprecated_entries > 0)
        stats.deprecated_entries--;
    else if (e->state == REGISTRY_ARCHIVED && stats.archived_entries > 0)
        stats.archived_entries--;

    e->state = REGISTRY_PUBLISHED;
    stats.published_entries++;

    return 0;
}

int registry_get(uint32_t entry_id, registry_entry_t *out)
{
    if (!out) return -1;
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;

    memcpy(out, e, sizeof(registry_entry_t));
    return 0;
}

uint32_t registry_list(uint32_t tenant_id, registry_entry_t *out,
                       uint32_t max_entries)
{
    if (!out || max_entries == 0) return 0;

    uint32_t count = 0;
    for (int i = 0; i < REGISTRY_MAX_ENTRIES && count < max_entries; i++) {
        if (!entries[i].in_use) continue;

        if (tenant_id > 0 && entries[i].owner_tenant_id != tenant_id) {
            /* Check access list */
            if (!registry_check_access(entries[i].entry_id, tenant_id))
                continue;
        }

        memcpy(&out[count], &entries[i], sizeof(registry_entry_t));
        count++;
    }
    return count;
}

uint32_t registry_search_by_tag(const char *tag, registry_entry_t *out,
                                uint32_t max_entries)
{
    if (!tag || !out || max_entries == 0) return 0;

    stats.total_searches++;
    uint32_t count = 0;

    for (int i = 0; i < REGISTRY_MAX_ENTRIES && count < max_entries; i++) {
        if (!entries[i].in_use) continue;
        if (entry_has_tag(&entries[i], tag)) {
            memcpy(&out[count], &entries[i], sizeof(registry_entry_t));
            count++;
        }
    }
    return count;
}

uint32_t registry_search_by_name(const char *prefix, registry_entry_t *out,
                                 uint32_t max_entries)
{
    if (!prefix || !out || max_entries == 0) return 0;

    stats.total_searches++;
    uint32_t count = 0;

    for (int i = 0; i < REGISTRY_MAX_ENTRIES && count < max_entries; i++) {
        if (!entries[i].in_use) continue;
        if (str_starts_with(entries[i].name, prefix)) {
            memcpy(&out[count], &entries[i], sizeof(registry_entry_t));
            count++;
        }
    }
    return count;
}

int registry_set_tag(uint32_t entry_id, const char *tag)
{
    if (!tag) return -1;
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;

    if (e->tag_count >= REGISTRY_MAX_TAGS) return -2;

    /* Check duplicate */
    if (entry_has_tag(e, tag)) return 0;

    uint32_t tlen = 0;
    while (tag[tlen] && tlen < REGISTRY_MAX_TAG_LEN - 1) tlen++;
    memcpy(e->tags[e->tag_count], tag, tlen);
    e->tags[e->tag_count][tlen] = '\0';
    e->tag_count++;

    return 0;
}

int registry_remove_tag(uint32_t entry_id, const char *tag)
{
    if (!tag) return -1;
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;

    for (uint8_t t = 0; t < e->tag_count; t++) {
        if (str_equal(e->tags[t], tag)) {
            /* Shift remaining tags */
            for (uint8_t j = t; j < e->tag_count - 1; j++) {
                memcpy(e->tags[j], e->tags[j + 1], REGISTRY_MAX_TAG_LEN);
            }
            e->tag_count--;
            return 0;
        }
    }
    return -1;  /* Tag not found */
}

int registry_check_access(uint32_t entry_id, uint32_t tenant_id)
{
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return 0;

    /* Owner always has access */
    if (e->owner_tenant_id == tenant_id) return 1;

    /* Check access list */
    for (uint8_t i = 0; i < e->access_tenant_count; i++) {
        if (e->access_tenants[i] == tenant_id) return 1;
    }

    return 0;  /* No access */
}

int registry_grant_access(uint32_t entry_id, uint32_t tenant_id)
{
    if (tenant_id == 0) return -1;
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;

    /* Already has access? */
    if (registry_check_access(entry_id, tenant_id)) return 0;

    if (e->access_tenant_count >= REGISTRY_MAX_ACCESS_TENANTS) return -2;

    e->access_tenants[e->access_tenant_count] = tenant_id;
    e->access_tenant_count++;

    serial_printf("[registry] Granted tenant %u access to entry %u\n",
                  tenant_id, entry_id);
    return 0;
}

int registry_revoke_access(uint32_t entry_id, uint32_t tenant_id)
{
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;

    /* Cannot revoke owner access */
    if (e->owner_tenant_id == tenant_id) return -2;

    for (uint8_t i = 0; i < e->access_tenant_count; i++) {
        if (e->access_tenants[i] == tenant_id) {
            for (uint8_t j = i; j < e->access_tenant_count - 1; j++) {
                e->access_tenants[j] = e->access_tenants[j + 1];
            }
            e->access_tenant_count--;
            return 0;
        }
    }
    return -1;
}

uint32_t registry_deploy(uint32_t entry_id, uint16_t endpoint_port)
{
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return 0;

    if (e->state != REGISTRY_PUBLISHED) {
        serial_printf("[registry] Cannot deploy entry %u: not published\n",
                      entry_id);
        return 0;
    }

    /*
     * Deploy to model serving gateway. Use entry_id as model_id
     * (in a real system, this would load the model first).
     */
    uint32_t ep_id = serving_register_endpoint(e->entry_id, endpoint_port,
                                                e->name);
    if (ep_id == 0) {
        serial_printf("[registry] Failed to deploy entry %u to port %u\n",
                      entry_id, (unsigned)endpoint_port);
        return 0;
    }

    e->deploy_count++;
    stats.total_deployments++;

    serial_printf("[registry] Deployed '%s' v%s -> endpoint %u (port %u)\n",
                  e->name, e->version, ep_id, (unsigned)endpoint_port);
    return ep_id;
}

int registry_record_download(uint32_t entry_id)
{
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;

    e->download_count++;
    stats.total_downloads++;
    return 0;
}

registry_stats_t *registry_get_stats(void)
{
    return &stats;
}

uint32_t registry_get_count(void)
{
    return stats.total_entries;
}

int registry_delete(uint32_t entry_id)
{
    registry_entry_t *e = find_entry(entry_id);
    if (!e) return -1;

    serial_printf("[registry] Deleted '%s' v%s (id=%u)\n",
                  e->name, e->version, entry_id);

    if (e->state == REGISTRY_PUBLISHED && stats.published_entries > 0)
        stats.published_entries--;
    else if (e->state == REGISTRY_DEPRECATED && stats.deprecated_entries > 0)
        stats.deprecated_entries--;
    else if (e->state == REGISTRY_ARCHIVED && stats.archived_entries > 0)
        stats.archived_entries--;

    if (stats.total_entries > 0)
        stats.total_entries--;

    e->in_use = 0;
    return 0;
}
