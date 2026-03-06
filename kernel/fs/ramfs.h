#ifndef KERNEL_RAMFS_H
#define KERNEL_RAMFS_H

#include <stdint.h>

#define RAMFS_MAX_NODES     256
#define RAMFS_MAX_NAME      64
#define RAMFS_MAX_CHILDREN  32
#define RAMFS_MAX_FILE_SIZE (64 * 1024)  /* 64 KB */

#define RAMFS_FILE 0
#define RAMFS_DIR  1

typedef struct ramfs_node {
    char     name[RAMFS_MAX_NAME];
    uint8_t  type;                          /* RAMFS_FILE or RAMFS_DIR */
    uint8_t  in_use;
    uint32_t size;                          /* bytes for files, child count for dirs */
    uint8_t *data;                          /* file content (kmalloc'd) */
    struct ramfs_node *parent;
    struct ramfs_node *children[RAMFS_MAX_CHILDREN];
    uint32_t child_count;
} ramfs_node_t;

/* Initialize the ramfs and create the root node "/" */
int ramfs_init(void);

/* Get the root node */
ramfs_node_t *ramfs_root(void);

/* Resolve a path starting from 'base' (NULL = root).
   Returns the node, or NULL if not found. */
ramfs_node_t *ramfs_lookup(ramfs_node_t *base, const char *path);

/* Create a file or directory under 'parent'.
   type = RAMFS_FILE or RAMFS_DIR.
   Returns the new node, or NULL on error. */
ramfs_node_t *ramfs_create(ramfs_node_t *parent, const char *name, uint8_t type);

/* Read up to 'max_size' bytes from a file node into 'buffer'.
   Returns bytes read, or -1 on error. */
int ramfs_read(ramfs_node_t *node, void *buffer, uint32_t offset, uint32_t max_size);

/* Write 'size' bytes from 'buffer' into a file node at 'offset'.
   Grows the file if needed.  Returns bytes written, or -1 on error. */
int ramfs_write(ramfs_node_t *node, const void *buffer, uint32_t offset, uint32_t size);

/* Truncate a file to 'new_size' bytes (0 to empty it). */
int ramfs_truncate(ramfs_node_t *node, uint32_t new_size);

/* Remove a node (file or empty directory).  Returns 0 on success. */
int ramfs_remove(ramfs_node_t *node);

/* Build the full path string for a node into 'buf' (max 'buflen' chars). */
int ramfs_get_path(ramfs_node_t *node, char *buf, uint32_t buflen);

#endif /* KERNEL_RAMFS_H */
