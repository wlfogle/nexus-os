#ifndef KERNEL_BLOCK_H
#define KERNEL_BLOCK_H

#include <stdint.h>

typedef int (*block_read_fn)(uint32_t sector, uint32_t count, void *buffer);
typedef int (*block_write_fn)(uint32_t sector, uint32_t count, void *buffer);

typedef struct {
    const char *name;
    uint32_t sector_size;
    block_read_fn read;
    block_write_fn write;
} block_device_t;

int block_device_register(const char *name, uint32_t sector_size,
                          block_read_fn read_fn, block_write_fn write_fn);
block_device_t *block_device_get(int id);
int block_device_read(int id, uint32_t sector, uint32_t count, void *buffer);
int block_device_write(int id, uint32_t sector, uint32_t count, void *buffer);

#endif
