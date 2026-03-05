#ifndef KERNEL_DEVICE_H
#define KERNEL_DEVICE_H

#include "../libc/stdint.h"

/* Device class/type definitions */
typedef enum {
    DEVICE_CLASS_BLOCK = 1,      /* Block devices (disks, partitions) */
    DEVICE_CLASS_CHAR = 2,       /* Character devices (keyboards, terminals) */
    DEVICE_CLASS_NET = 3,        /* Network devices (NICs) */
    DEVICE_CLASS_BUS = 4,        /* Bus devices (PCI, USB) */
    DEVICE_CLASS_MISC = 5        /* Miscellaneous devices */
} device_class_t;

/* Device state */
typedef enum {
    DEVICE_REGISTERED = 1,
    DEVICE_ACTIVE = 2,
    DEVICE_INACTIVE = 3,
    DEVICE_REMOVED = 4
} device_state_t;

/* Device metadata structure */
typedef struct {
    uint32_t id;                 /* Unique device ID */
    const char *name;            /* Device name (e.g. "sda", "eth0") */
    device_class_t device_class; /* Device class */
    device_state_t state;        /* Current device state */
    void *driver_data;           /* Private driver data */
    uint32_t flags;              /* Device flags */
} device_t;

/* Device registry statistics */
typedef struct {
    uint32_t total_registered;   /* Total devices ever registered */
    uint32_t lookups_by_id;      /* Successful lookups by ID */
    uint32_t lookups_by_name;    /* Successful lookups by name */
    uint32_t lookup_failures;    /* Failed lookups */
} device_stats_t;

/* Device registration API */
int device_register(const char *name, device_class_t device_class, void *driver_data);
int device_unregister(uint32_t device_id);

/* Device lookup API */
device_t *device_get_by_id(uint32_t device_id);
device_t *device_get_by_name(const char *name);

/* Device state management */
int device_set_state(uint32_t device_id, device_state_t state);
device_state_t device_get_state(uint32_t device_id);

/* Device enumeration */
device_t *device_get_by_class(device_class_t device_class, int index);
int device_count_by_class(device_class_t device_class);

/* Statistics and diagnostics */
device_stats_t *device_get_stats(void);
void device_init(void);

#endif /* KERNEL_DEVICE_H */
