#ifndef KERNEL_FAT_H
#define KERNEL_FAT_H

#include <stdint.h>

int fat_init(int block_device_id);
int fat_read_file(const char *path, void *buffer, uint32_t max_size);
int fat_write_file(const char *path, void *buffer, uint32_t size);
int fat_list_directory(void);

#endif
