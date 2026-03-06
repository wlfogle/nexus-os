#ifndef KERNEL_SECURITY_H
#define KERNEL_SECURITY_H

#include <stdint.h>

/* Capability flags for access control */
typedef enum {
    CAP_NONE = 0x0,
    CAP_GPU_READ = 0x1,        /* Read GPU memory */
    CAP_GPU_WRITE = 0x2,       /* Write GPU memory */
    CAP_GPU_EXECUTE = 0x4,     /* Execute GPU kernels */
    CAP_MODEL_LOAD = 0x8,      /* Load models */
    CAP_MODEL_INFER = 0x10,    /* Run inference */
    CAP_MEMORY_ALLOC = 0x20,   /* Allocate memory */
    CAP_NETWORK_TX = 0x40,     /* Network transmit */
    CAP_NETWORK_RX = 0x80,     /* Network receive */
    CAP_FILESYSTEM_READ = 0x100,
    CAP_FILESYSTEM_WRITE = 0x200,
    CAP_CONTAINER_CREATE = 0x400,
    CAP_CONTAINER_DESTROY = 0x800,
    CAP_ALL = 0xFFF
} capability_t;

/* ACL entry for subject access control */
typedef struct {
    uint32_t subject_id;        /* Container or process ID */
    uint32_t object_id;         /* Resource ID (GPU, model, etc) */
    uint8_t object_type;        /* Resource type (0=GPU, 1=model, 2=memory, etc) */
    capability_t permissions;   /* Granted capabilities */
    uint32_t created_time;
    uint8_t active;
} acl_entry_t;

/* Model attestation certificate */
typedef struct {
    uint32_t model_id;
    uint8_t hash[32];           /* SHA256 hash */
    uint32_t size;
    uint8_t verified;
    uint32_t verified_time;
    uint8_t allow_execution;
} model_attestation_t;

/* Security context for container */
typedef struct {
    uint32_t container_id;
    capability_t capabilities;
    uint8_t enforced;
    uint8_t allow_syscalls[256];  /* Syscall whitelist */
    uint32_t memory_limit;
    uint32_t created_time;
} security_context_t;

/* Audit log entry */
typedef struct {
    uint32_t subject_id;
    uint32_t action;            /* Action code */
    uint32_t object_id;
    uint8_t allowed;            /* Whether action was allowed */
    uint32_t timestamp;
} audit_log_t;

/* Core Security APIs */

/**
 * Initialize security subsystem
 */
void security_init(void);

/**
 * Grant capability to subject for object
 */
int acl_grant(uint32_t subject_id, uint32_t object_id, uint8_t object_type,
              capability_t capability);

/**
 * Revoke capability from subject
 */
int acl_revoke(uint32_t subject_id, uint32_t object_id, capability_t capability);

/**
 * Check if subject has capability for object
 */
int acl_check(uint32_t subject_id, uint32_t object_id, uint8_t object_type,
              capability_t capability);

/**
 * Get all capabilities for a subject
 */
capability_t acl_get_capabilities(uint32_t subject_id, uint32_t object_id);

/**
 * Create security context for container
 */
uint32_t security_create_context(uint32_t container_id, capability_t caps);

/**
 * Destroy security context
 */
int security_destroy_context(uint32_t context_id);

/**
 * Enable syscall enforcement for context
 */
int security_enable_syscall_filtering(uint32_t context_id);

/**
 * Add syscall to whitelist
 */
int security_whitelist_syscall(uint32_t context_id, uint8_t syscall_num);

/**
 * Remove syscall from whitelist
 */
int security_blacklist_syscall(uint32_t context_id, uint8_t syscall_num);

/**
 * Check if syscall is allowed
 */
int security_check_syscall(uint32_t context_id, uint8_t syscall_num);

/**
 * Register model attestation
 */
int security_attest_model(uint32_t model_id, uint8_t *hash, uint32_t size);

/**
 * Verify model attestation
 */
int security_verify_model(uint32_t model_id, uint8_t *hash);

/**
 * Get model attestation status
 */
int security_get_model_attestation(uint32_t model_id, model_attestation_t *out);

/**
 * Log security audit event
 */
int security_audit_log(uint32_t subject_id, uint32_t action, 
                       uint32_t object_id, uint8_t allowed);

/**
 * Get audit log entries
 */
uint32_t security_get_audit_log(audit_log_t *entries, uint32_t max_entries);

/**
 * Clear audit log
 */
int security_clear_audit_log(void);

/**
 * Set global security enforcement level
 */
int security_set_enforcement_level(uint8_t level);

/**
 * Get current enforcement level
 */
uint8_t security_get_enforcement_level(void);

/**
 * Enable/disable capability enforcement
 */
int security_enable_capability_enforcement(int enabled);

/**
 * Get security statistics
 */
int security_get_stats(uint32_t *denied_count, uint32_t *allowed_count);

/**
 * Create default security policy
 */
int security_apply_default_policy(uint32_t container_id);

/**
 * Isolate container network
 */
int security_isolate_network(uint32_t container_id);

/**
 * Check container isolation status
 */
int security_is_isolated(uint32_t container_id);

#endif /* KERNEL_SECURITY_H */
