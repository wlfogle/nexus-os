#ifndef KERNEL_VFS_H
#define KERNEL_VFS_H

#include <stdint.h>
#include "ramfs.h"

/* Filesystem types */
#define VFS_RAMFS 0
#define VFS_FAT12 1

/* File descriptor flags */
#define VFS_O_READ  0x01
#define VFS_O_WRITE 0x02
#define VFS_O_RDWR  (VFS_O_READ | VFS_O_WRITE)

/* Node types for stat */
#define VFS_TYPE_FILE 0
#define VFS_TYPE_DIR  1

#define VFS_MAX_FDS   32
#define VFS_MAX_PATH  256

typedef struct {
    uint8_t  type;          /* VFS_TYPE_FILE or VFS_TYPE_DIR */
    uint32_t size;          /* bytes for files */
    uint32_t child_count;   /* children for directories */
} vfs_stat_t;

/* ------------------------------------------------------------------ */
/* Core VFS API                                                        */
/* ------------------------------------------------------------------ */

/* Initialize VFS, mount ramfs at root, prepopulate directories */
int vfs_init(void);

/* Resolve a path (absolute or relative to cwd) to a ramfs node */
ramfs_node_t *vfs_resolve(const char *path);

/* File operations */
int vfs_open(const char *path, int flags);
int vfs_close(int fd);
int vfs_read(int fd, void *buffer, uint32_t size);
int vfs_write(int fd, const void *buffer, uint32_t size);

/* Convenience: read/write entire files by path */
int vfs_read_file(const char *path, void *buffer, uint32_t max_size);
int vfs_write_file(const char *path, const void *buffer, uint32_t size);

/* Directory and node operations */
int vfs_list_directory(const char *path);
int vfs_mkdir(const char *path);
int vfs_touch(const char *path);
int vfs_remove(const char *path);
int vfs_stat(const char *path, vfs_stat_t *out);
int vfs_copy(const char *src, const char *dst);

/* Working directory */
const char *vfs_getcwd(void);
int vfs_chdir(const char *path);

/* Legacy: mount a FAT12 block device at /mnt/disk */
int vfs_mount_fat(int block_device_id);

#endif /* KERNEL_VFS_H */
