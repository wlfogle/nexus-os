#include "vfs.h"
#include "fat.h"
#include "../../include/kernel/serial.h"

typedef struct {
    int block_device_id;
    int fs_type;
} vfs_mount_t;

static vfs_mount_t mounts[4] = {0};

int vfs_mount(int block_device_id, int fs_type) {
    if (block_device_id < 0 || fs_type < 0) return -1;
    
    for (int i = 0; i < 4; i++) {
        if (mounts[i].block_device_id == 0) {
            mounts[i].block_device_id = block_device_id + 1;
            mounts[i].fs_type = fs_type;
            
            if (fs_type == VFS_FAT12) {
                return fat_init(block_device_id);
            }
            
            return 0;
        }
    }
    
    return -1;
}

int vfs_open(const char *path) {
    if (!path) return -1;
    return 0;
}

int vfs_close(int fd) {
    if (fd < 0) return -1;
    return 0;
}

int vfs_read(int fd, void *buffer, uint32_t size) {
    if (fd < 0 || !buffer) return -1;
    return 0;
}

int vfs_write(int fd, void *buffer, uint32_t size) {
    if (fd < 0 || !buffer) return -1;
    return 0;
}

int vfs_read_file(const char *path, void *buffer, uint32_t max_size) {
    if (!path || !buffer) return -1;
    
    for (int i = 0; i < 4; i++) {
        if (mounts[i].block_device_id > 0) {
            if (mounts[i].fs_type == VFS_FAT12) {
                return fat_read_file(path, buffer, max_size);
            }
        }
    }
    
    return -1;
}

int vfs_write_file(const char *path, void *buffer, uint32_t size) {
    if (!path || !buffer) return -1;
    
    for (int i = 0; i < 4; i++) {
        if (mounts[i].block_device_id > 0) {
            if (mounts[i].fs_type == VFS_FAT12) {
                return fat_write_file(path, buffer, size);
            }
        }
    }
    
    return -1;
}

int vfs_list_directory(const char *path) {
    if (!path) return -1;
    
    for (int i = 0; i < 4; i++) {
        if (mounts[i].block_device_id > 0) {
            if (mounts[i].fs_type == VFS_FAT12) {
                return fat_list_directory();
            }
        }
    }
    
    return -1;
}
