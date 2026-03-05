#include "../../include/kernel/device.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>
#include <string.h>

#define MAX_DEVICES 128

typedef struct {
    device_t device;
    int in_use;
} device_entry_t;

static device_entry_t device_registry[MAX_DEVICES];
static device_stats_t device_stats = {0};
static uint32_t next_device_id = 1;

void device_init(void)
{
    memset(device_registry, 0, sizeof(device_registry));
    memset(&device_stats, 0, sizeof(device_stats));
    next_device_id = 1;
    
    serial_puts("[device] Device registry initialized\n");
}

int device_register(const char *name, device_class_t device_class, void *driver_data)
{
    if (!name || device_class < 1 || device_class > 5) {
        return -1;
    }
    
    /* Find free entry */
    int free_idx = -1;
    for (int i = 0; i < MAX_DEVICES; i++) {
        if (!device_registry[i].in_use) {
            free_idx = i;
            break;
        }
    }
    
    if (free_idx < 0) {
        serial_puts("[device] Registry full\n");
        return -1;
    }
    
    /* Initialize device */
    device_entry_t *entry = &device_registry[free_idx];
    entry->device.id = next_device_id++;
    entry->device.name = name;
    entry->device.device_class = device_class;
    entry->device.state = DEVICE_REGISTERED;
    entry->device.driver_data = driver_data;
    entry->device.flags = 0;
    entry->in_use = 1;
    
    device_stats.total_registered++;
    
    serial_printf("[device] Registered device %d: %s (class %d)\n", 
                  entry->device.id, name, device_class);
    
    return entry->device.id;
}

int device_unregister(uint32_t device_id)
{
    if (device_id == 0) return -1;
    
    for (int i = 0; i < MAX_DEVICES; i++) {
        if (device_registry[i].in_use && device_registry[i].device.id == device_id) {
            device_registry[i].device.state = DEVICE_REMOVED;
            device_registry[i].in_use = 0;
            
            serial_printf("[device] Unregistered device %d\n", device_id);
            return 0;
        }
    }
    
    return -1;
}

device_t *device_get_by_id(uint32_t device_id)
{
    if (device_id == 0) {
        device_stats.lookup_failures++;
        return NULL;
    }
    
    for (int i = 0; i < MAX_DEVICES; i++) {
        if (device_registry[i].in_use && device_registry[i].device.id == device_id) {
            device_stats.lookups_by_id++;
            return &device_registry[i].device;
        }
    }
    
    device_stats.lookup_failures++;
    return NULL;
}

device_t *device_get_by_name(const char *name)
{
    if (!name) {
        device_stats.lookup_failures++;
        return NULL;
    }
    
    for (int i = 0; i < MAX_DEVICES; i++) {
        if (device_registry[i].in_use && device_registry[i].device.name) {
            if (strcmp(device_registry[i].device.name, name) == 0) {
                device_stats.lookups_by_name++;
                return &device_registry[i].device;
            }
        }
    }
    
    device_stats.lookup_failures++;
    return NULL;
}

int device_set_state(uint32_t device_id, device_state_t state)
{
    if (device_id == 0 || state < 1 || state > 4) return -1;
    
    device_t *dev = device_get_by_id(device_id);
    if (!dev) return -1;
    
    dev->state = state;
    return 0;
}

device_state_t device_get_state(uint32_t device_id)
{
    device_t *dev = device_get_by_id(device_id);
    return dev ? dev->state : DEVICE_REGISTERED;
}

device_t *device_get_by_class(device_class_t device_class, int index)
{
    if (device_class < 1 || device_class > 5 || index < 0) {
        return NULL;
    }
    
    int count = 0;
    for (int i = 0; i < MAX_DEVICES; i++) {
        if (device_registry[i].in_use && 
            device_registry[i].device.device_class == device_class) {
            if (count == index) {
                return &device_registry[i].device;
            }
            count++;
        }
    }
    
    return NULL;
}

int device_count_by_class(device_class_t device_class)
{
    if (device_class < 1 || device_class > 5) return -1;
    
    int count = 0;
    for (int i = 0; i < MAX_DEVICES; i++) {
        if (device_registry[i].in_use && 
            device_registry[i].device.device_class == device_class) {
            count++;
        }
    }
    
    return count;
}

device_stats_t *device_get_stats(void)
{
    return &device_stats;
}
