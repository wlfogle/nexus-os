#include "../../include/kernel/security.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_ACL_ENTRIES 128
#define MAX_SECURITY_CONTEXTS 32
#define MAX_MODEL_ATTESTATIONS 64
#define MAX_AUDIT_ENTRIES 256

typedef struct {
    acl_entry_t entry;
    int in_use;
} acl_slot_t;

typedef struct {
    security_context_t context;
    int in_use;
} context_slot_t;

typedef struct {
    model_attestation_t attestation;
    int in_use;
} attestation_slot_t;

typedef struct {
    audit_log_t log_entry;
    int in_use;
} audit_slot_t;

static acl_slot_t acl_table[MAX_ACL_ENTRIES];
static context_slot_t contexts[MAX_SECURITY_CONTEXTS];
static attestation_slot_t attestations[MAX_MODEL_ATTESTATIONS];
static audit_slot_t audit_log[MAX_AUDIT_ENTRIES];
static uint32_t audit_index = 0;
static uint8_t enforcement_level = 1;  /* 0=off, 1=warn, 2=enforce */
static int cap_enforcement_enabled = 1;
static uint32_t denied_count = 0;
static uint32_t allowed_count = 0;

void security_init(void)
{
    memset(acl_table, 0, sizeof(acl_table));
    memset(contexts, 0, sizeof(contexts));
    memset(attestations, 0, sizeof(attestations));
    memset(audit_log, 0, sizeof(audit_log));
    
    enforcement_level = 1;
    cap_enforcement_enabled = 1;
    denied_count = 0;
    allowed_count = 0;
    audit_index = 0;
    
    serial_puts("[security] Security subsystem initialized\n");
}

int acl_grant(uint32_t subject_id, uint32_t object_id, uint8_t object_type,
              capability_t capability)
{
    if (subject_id == 0 || object_id == 0) return -1;
    
    /* Find free ACL slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_ACL_ENTRIES; i++) {
        if (!acl_table[i].in_use) {
            free_idx = i;
            break;
        }
        /* Update existing entry */
        if (acl_table[i].entry.subject_id == subject_id &&
            acl_table[i].entry.object_id == object_id &&
            acl_table[i].entry.object_type == object_type &&
            acl_table[i].in_use) {
            acl_table[i].entry.permissions |= capability;
            return 0;
        }
    }
    
    if (free_idx < 0) return -1;  /* ACL table full */
    
    acl_slot_t *slot = &acl_table[free_idx];
    slot->entry.subject_id = subject_id;
    slot->entry.object_id = object_id;
    slot->entry.object_type = object_type;
    slot->entry.permissions = capability;
    slot->entry.created_time = 0;  /* Would be set from timer */
    slot->entry.active = 1;
    slot->in_use = 1;
    
    serial_printf("[security] ACL: Granted capability 0x%x to subject %d for object %d\n",
                 capability, subject_id, object_id);
    
    return 0;
}

int acl_revoke(uint32_t subject_id, uint32_t object_id, capability_t capability)
{
    if (subject_id == 0 || object_id == 0) return -1;
    
    for (int i = 0; i < MAX_ACL_ENTRIES; i++) {
        if (acl_table[i].in_use &&
            acl_table[i].entry.subject_id == subject_id &&
            acl_table[i].entry.object_id == object_id) {
            acl_table[i].entry.permissions &= ~capability;
            
            if (acl_table[i].entry.permissions == CAP_NONE) {
                acl_table[i].in_use = 0;
            }
            
            return 0;
        }
    }
    
    return -1;
}

int acl_check(uint32_t subject_id, uint32_t object_id, uint8_t object_type,
              capability_t capability)
{
    if (!cap_enforcement_enabled) return 1;  /* Enforcement disabled */
    
    for (int i = 0; i < MAX_ACL_ENTRIES; i++) {
        if (acl_table[i].in_use &&
            acl_table[i].entry.subject_id == subject_id &&
            acl_table[i].entry.object_id == object_id &&
            acl_table[i].entry.object_type == object_type) {
            
            int has_cap = (acl_table[i].entry.permissions & capability) ? 1 : 0;
            
            if (has_cap) {
                allowed_count++;
            } else {
                denied_count++;
            }
            
            if (enforcement_level > 1 && !has_cap) {
                security_audit_log(subject_id, 1, object_id, 0);  /* action=1=access */
                return 0;  /* Deny */
            }
            
            security_audit_log(subject_id, 1, object_id, has_cap);
            return has_cap;
        }
    }
    
    /* No ACL entry found - deny by default */
    denied_count++;
    security_audit_log(subject_id, 1, object_id, 0);
    return 0;
}

capability_t acl_get_capabilities(uint32_t subject_id, uint32_t object_id)
{
    if (subject_id == 0 || object_id == 0) return CAP_NONE;
    
    for (int i = 0; i < MAX_ACL_ENTRIES; i++) {
        if (acl_table[i].in_use &&
            acl_table[i].entry.subject_id == subject_id &&
            acl_table[i].entry.object_id == object_id) {
            return acl_table[i].entry.permissions;
        }
    }
    
    return CAP_NONE;
}

uint32_t security_create_context(uint32_t container_id, capability_t caps)
{
    if (container_id == 0) return 0;
    
    /* Find free context slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_SECURITY_CONTEXTS; i++) {
        if (!contexts[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) return 0;  /* No free contexts */
    
    context_slot_t *slot = &contexts[free_idx];
    slot->context.container_id = container_id;
    slot->context.capabilities = caps;
    slot->context.enforced = 0;
    slot->context.memory_limit = 64 * 1024 * 1024;  /* 64MB default */
    slot->context.created_time = 0;
    
    memset(slot->context.allow_syscalls, 0, 256);
    slot->in_use = 1;
    
    serial_printf("[security] Created security context for container %d\n", container_id);
    
    return free_idx + 1;  /* Return 1-based context ID */
}

int security_destroy_context(uint32_t context_id)
{
    if (context_id == 0 || context_id > MAX_SECURITY_CONTEXTS) return -1;
    
    context_slot_t *slot = &contexts[context_id - 1];
    if (!slot->in_use) return -1;
    
    slot->in_use = 0;
    
    return 0;
}

int security_enable_syscall_filtering(uint32_t context_id)
{
    if (context_id == 0 || context_id > MAX_SECURITY_CONTEXTS) return -1;
    
    context_slot_t *slot = &contexts[context_id - 1];
    if (!slot->in_use) return -1;
    
    slot->context.enforced = 1;
    
    serial_printf("[security] Enabled syscall filtering for context %d\n", context_id);
    
    return 0;
}

int security_whitelist_syscall(uint32_t context_id, uint8_t syscall_num)
{
    if (context_id == 0 || context_id > MAX_SECURITY_CONTEXTS) return -1;
    
    context_slot_t *slot = &contexts[context_id - 1];
    if (!slot->in_use) return -1;
    
    slot->context.allow_syscalls[syscall_num] = 1;
    
    return 0;
}

int security_blacklist_syscall(uint32_t context_id, uint8_t syscall_num)
{
    if (context_id == 0 || context_id > MAX_SECURITY_CONTEXTS) return -1;
    
    context_slot_t *slot = &contexts[context_id - 1];
    if (!slot->in_use) return -1;
    
    slot->context.allow_syscalls[syscall_num] = 0;
    
    return 0;
}

int security_check_syscall(uint32_t context_id, uint8_t syscall_num)
{
    if (context_id == 0 || context_id > MAX_SECURITY_CONTEXTS) return -1;
    
    context_slot_t *slot = &contexts[context_id - 1];
    if (!slot->in_use) return -1;
    
    if (!slot->context.enforced) return 1;  /* Not enforced */
    
    int allowed = slot->context.allow_syscalls[syscall_num];
    
    if (!allowed) {
        denied_count++;
        security_audit_log(slot->context.container_id, 2, syscall_num, 0);  /* action=2=syscall */
    } else {
        allowed_count++;
    }
    
    return allowed;
}

int security_attest_model(uint32_t model_id, uint8_t *hash, uint32_t size)
{
    if (model_id == 0 || !hash || size == 0) return -1;
    
    /* Find free attestation slot */
    int free_idx = -1;
    for (int i = 0; i < MAX_MODEL_ATTESTATIONS; i++) {
        if (!attestations[i].in_use) {
            free_idx = i;
            break;
        }
        /* Update existing */
        if (attestations[i].attestation.model_id == model_id && attestations[i].in_use) {
            memcpy(attestations[i].attestation.hash, hash, 32);
            attestations[i].attestation.size = size;
            return 0;
        }
    }
    
    if (free_idx < 0) return -1;  /* Too many attestations */
    
    attestation_slot_t *slot = &attestations[free_idx];
    slot->attestation.model_id = model_id;
    memcpy(slot->attestation.hash, hash, 32);
    slot->attestation.size = size;
    slot->attestation.verified = 0;
    slot->attestation.verified_time = 0;
    slot->attestation.allow_execution = 0;
    slot->in_use = 1;
    
    serial_printf("[security] Registered attestation for model %d (size=%d)\n",
                 model_id, size);
    
    return 0;
}

int security_verify_model(uint32_t model_id, uint8_t *hash)
{
    if (model_id == 0 || !hash) return -1;
    
    for (int i = 0; i < MAX_MODEL_ATTESTATIONS; i++) {
        if (attestations[i].in_use &&
            attestations[i].attestation.model_id == model_id) {
            
            /* Simple byte-by-byte comparison */
            int match = 1;
            for (int j = 0; j < 32; j++) {
                if (attestations[i].attestation.hash[j] != hash[j]) {
                    match = 0;
                    break;
                }
            }
            
            if (match) {
                attestations[i].attestation.verified = 1;
                attestations[i].attestation.allow_execution = 1;
                serial_printf("[security] Model %d verified\n", model_id);
                return 0;
            }
            
            return -1;  /* Hash mismatch */
        }
    }
    
    return -1;  /* No attestation found */
}

int security_get_model_attestation(uint32_t model_id, model_attestation_t *out)
{
    if (model_id == 0 || !out) return -1;
    
    for (int i = 0; i < MAX_MODEL_ATTESTATIONS; i++) {
        if (attestations[i].in_use &&
            attestations[i].attestation.model_id == model_id) {
            memcpy(out, &attestations[i].attestation, sizeof(model_attestation_t));
            return 0;
        }
    }
    
    return -1;
}

int security_audit_log(uint32_t subject_id, uint32_t action,
                       uint32_t object_id, uint8_t allowed)
{
    if (audit_index >= MAX_AUDIT_ENTRIES) {
        audit_index = 0;  /* Wrap around */
    }
    
    audit_slot_t *slot = &audit_log[audit_index];
    slot->log_entry.subject_id = subject_id;
    slot->log_entry.action = action;
    slot->log_entry.object_id = object_id;
    slot->log_entry.allowed = allowed;
    slot->log_entry.timestamp = 0;  /* Would be set from timer */
    slot->in_use = 1;
    
    audit_index++;
    
    return 0;
}

uint32_t security_get_audit_log(audit_log_t *entries, uint32_t max_entries)
{
    if (!entries || max_entries == 0) return 0;
    
    uint32_t count = 0;
    
    for (int i = 0; i < MAX_AUDIT_ENTRIES && count < max_entries; i++) {
        if (audit_log[i].in_use) {
            memcpy(&entries[count], &audit_log[i].log_entry, sizeof(audit_log_t));
            count++;
        }
    }
    
    return count;
}

int security_clear_audit_log(void)
{
    memset(audit_log, 0, sizeof(audit_log));
    audit_index = 0;
    
    return 0;
}

int security_set_enforcement_level(uint8_t level)
{
    if (level > 2) return -1;
    
    enforcement_level = level;
    
    serial_printf("[security] Enforcement level set to %d\n", level);
    
    return 0;
}

uint8_t security_get_enforcement_level(void)
{
    return enforcement_level;
}

int security_enable_capability_enforcement(int enabled)
{
    cap_enforcement_enabled = enabled ? 1 : 0;
    
    serial_printf("[security] Capability enforcement %s\n",
                 enabled ? "enabled" : "disabled");
    
    return 0;
}

int security_get_stats(uint32_t *denied_count_out, uint32_t *allowed_count_out)
{
    if (!denied_count_out || !allowed_count_out) return -1;
    
    *denied_count_out = denied_count;
    *allowed_count_out = allowed_count;
    
    return 0;
}

int security_apply_default_policy(uint32_t container_id)
{
    if (container_id == 0) return -1;
    
    /* Apply standard capabilities for containers */
    capability_t default_caps = CAP_GPU_READ | CAP_GPU_WRITE | CAP_GPU_EXECUTE |
                               CAP_MODEL_INFER | CAP_MEMORY_ALLOC |
                               CAP_FILESYSTEM_READ;
    
    uint32_t ctx_id = security_create_context(container_id, default_caps);
    if (ctx_id == 0) return -1;
    
    return 0;
}

int security_isolate_network(uint32_t container_id)
{
    if (container_id == 0) return -1;
    
    /* Revoke network capabilities */
    for (int i = 0; i < MAX_SECURITY_CONTEXTS; i++) {
        if (contexts[i].in_use &&
            contexts[i].context.container_id == container_id) {
            contexts[i].context.capabilities &= ~(CAP_NETWORK_TX | CAP_NETWORK_RX);
            return 0;
        }
    }
    
    return -1;
}

int security_is_isolated(uint32_t container_id)
{
    if (container_id == 0) return -1;
    
    for (int i = 0; i < MAX_SECURITY_CONTEXTS; i++) {
        if (contexts[i].in_use &&
            contexts[i].context.container_id == container_id) {
            
            /* Check if network capabilities are revoked */
            return ((contexts[i].context.capabilities & CAP_NETWORK_TX) == 0 &&
                    (contexts[i].context.capabilities & CAP_NETWORK_RX) == 0) ? 1 : 0;
        }
    }
    
    return -1;
}
