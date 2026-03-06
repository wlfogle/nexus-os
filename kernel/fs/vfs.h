#ifndef KERNEL_VFS_H
#define KERNEL_VFS_H

#include <stdint.h>

#define VFS_FAT12 1

int vfs_mount(int block_device_id, int fs_type);
int vfs_open(const char *path);
int vfs_close(int fd);
int vfs_read(int fd, void *buffer, uint32_t size);
int vfs_write(int fd, void *buffer, uint32_t size);
int vfs_read_file(const char *path, void *buffer, uint32_t max_size);
int vfs_write_file(const char *path, void *buffer, uint32_t size);
int vfs_list_directory(const char *path);

#endif
