#include "block.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

#define MAX_BLOCK_DEVICES 4

typedef struct {
    block_device_t device;
    int in_use;
} block_device_entry_t;

static block_device_entry_t devices[MAX_BLOCK_DEVICES] = {0};

int block_device_register(const char *name, uint32_t sector_size, 
                          block_read_fn read_fn, block_write_fn write_fn) {
    if (!name || !read_fn || !write_fn || sector_size == 0) return -1;
    
    for (int i = 0; i < MAX_BLOCK_DEVICES; i++) {
        if (!devices[i].in_use) {
            devices[i].device.name = name;
            devices[i].device.sector_size = sector_size;
            devices[i].device.read = read_fn;
            devices[i].device.write = write_fn;
            devices[i].in_use = 1;
            return i;
        }
    }
    return -1;
}

block_device_t *block_device_get(int id) {
    if (id < 0 || id >= MAX_BLOCK_DEVICES) return NULL;
    return devices[id].in_use ? &devices[id].device : NULL;
}

int block_device_read(int id, uint32_t sector, uint32_t count, void *buffer) {
    block_device_t *dev = block_device_get(id);
    if (!dev || !buffer || count == 0) return -1;
    return dev->read(sector, count, buffer);
}

int block_device_write(int id, uint32_t sector, uint32_t count, void *buffer) {
    block_device_t *dev = block_device_get(id);
    if (!dev || !buffer || count == 0) return -1;
    return dev->write(sector, count, buffer);
}
