#include "vfs.h"
#include "ramfs.h"
#include "fat.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/console.h"
#include "../../include/kernel/heap.h"
#include "../../include/libc/string.h"
#include <stddef.h>

/* ------------------------------------------------------------------ */
/* File descriptor table                                               */
/* ------------------------------------------------------------------ */

typedef struct {
    ramfs_node_t *node;
    uint32_t      offset;
    int           flags;
    int           in_use;
} vfs_fd_t;

static vfs_fd_t fd_table[VFS_MAX_FDS];

/* ------------------------------------------------------------------ */
/* Current working directory                                           */
/* ------------------------------------------------------------------ */

static ramfs_node_t *cwd_node = NULL;
static char cwd_path[VFS_MAX_PATH] = "/";

/* ------------------------------------------------------------------ */
/* Internal helpers                                                    */
/* ------------------------------------------------------------------ */

/* Split 'path' into parent-path and basename.
   Writes parent portion into parent_buf, returns pointer to basename. */
static const char *split_path(const char *path, char *parent_buf, uint32_t buflen)
{
    uint32_t len = strlen(path);

    /* Strip trailing slashes */
    while (len > 1 && path[len - 1] == '/') len--;

    /* Find last '/' */
    int last_slash = -1;
    for (int i = (int)len - 1; i >= 0; i--) {
        if (path[i] == '/') { last_slash = i; break; }
    }

    if (last_slash < 0) {
        /* No slash — parent is cwd */
        parent_buf[0] = '\0';
        return path;
    }

    /* Copy parent portion */
    uint32_t plen = (uint32_t)last_slash;
    if (plen == 0) plen = 1;  /* root "/" */
    if (plen >= buflen) plen = buflen - 1;
    memcpy(parent_buf, path, plen);
    parent_buf[plen] = '\0';

    return path + last_slash + 1;
}

static int alloc_fd(void)
{
    for (int i = 0; i < VFS_MAX_FDS; i++) {
        if (!fd_table[i].in_use) return i;
    }
    return -1;
}

/* ------------------------------------------------------------------ */
/* Public API                                                          */
/* ------------------------------------------------------------------ */

ramfs_node_t *vfs_resolve(const char *path)
{
    if (!path) return NULL;
    if (path[0] == '/') {
        return ramfs_lookup(NULL, path);
    }
    return ramfs_lookup(cwd_node, path);
}

int vfs_init(void)
{
    memset(fd_table, 0, sizeof(fd_table));

    /* Initialize the in-memory filesystem */
    if (ramfs_init() != 0) return -1;

    cwd_node = ramfs_root();
    strcpy(cwd_path, "/");

    /* Create standard directory structure */
    ramfs_node_t *root = ramfs_root();
    ramfs_create(root, "etc",  RAMFS_DIR);
    ramfs_create(root, "tmp",  RAMFS_DIR);
    ramfs_create(root, "bin",  RAMFS_DIR);
    ramfs_create(root, "home", RAMFS_DIR);
    ramfs_create(root, "mnt",  RAMFS_DIR);
    ramfs_create(root, "var",  RAMFS_DIR);

    /* Create /etc/motd with a welcome message */
    ramfs_node_t *etc = ramfs_lookup(root, "etc");
    if (etc) {
        ramfs_node_t *motd = ramfs_create(etc, "motd", RAMFS_FILE);
        if (motd) {
            const char *msg =
                "Welcome to NexusOS v0.1.0\n"
                "AI-Native Operating System (Phases 0-13)\n"
                "Type 'help' for available commands.\n"
                "Type 'nexuspkg list' to see available packages.\n";
            ramfs_write(motd, msg, 0, strlen(msg));
        }

        /* Create /etc/hostname */
        ramfs_node_t *hostname = ramfs_create(etc, "hostname", RAMFS_FILE);
        if (hostname) {
            const char *name = "nexus\n";
            ramfs_write(hostname, name, 0, strlen(name));
        }

        /* Create /etc/version */
        ramfs_node_t *ver = ramfs_create(etc, "version", RAMFS_FILE);
        if (ver) {
            const char *v = "0.1.0\n";
            ramfs_write(ver, v, 0, strlen(v));
        }
    }

    serial_puts("vfs: initialized, ramfs mounted at /\n");
    return 0;
}

/* ---- File descriptor operations ---- */

int vfs_open(const char *path, int flags)
{
    if (!path) return -1;

    ramfs_node_t *node = vfs_resolve(path);
    if (!node) return -1;
    if (node->type != RAMFS_FILE) return -1;

    int fd = alloc_fd();
    if (fd < 0) return -1;

    fd_table[fd].node   = node;
    fd_table[fd].offset = 0;
    fd_table[fd].flags  = flags;
    fd_table[fd].in_use = 1;
    return fd;
}

int vfs_close(int fd)
{
    if (fd < 0 || fd >= VFS_MAX_FDS) return -1;
    if (!fd_table[fd].in_use) return -1;

    fd_table[fd].in_use = 0;
    fd_table[fd].node   = NULL;
    return 0;
}

int vfs_read(int fd, void *buffer, uint32_t size)
{
    if (fd < 0 || fd >= VFS_MAX_FDS) return -1;
    if (!fd_table[fd].in_use || !buffer) return -1;

    int n = ramfs_read(fd_table[fd].node, buffer, fd_table[fd].offset, size);
    if (n > 0) fd_table[fd].offset += (uint32_t)n;
    return n;
}

int vfs_write(int fd, const void *buffer, uint32_t size)
{
    if (fd < 0 || fd >= VFS_MAX_FDS) return -1;
    if (!fd_table[fd].in_use || !buffer) return -1;
    if (!(fd_table[fd].flags & VFS_O_WRITE)) return -1;

    int n = ramfs_write(fd_table[fd].node, buffer, fd_table[fd].offset, size);
    if (n > 0) fd_table[fd].offset += (uint32_t)n;
    return n;
}

/* ---- Convenience file operations ---- */

int vfs_read_file(const char *path, void *buffer, uint32_t max_size)
{
    if (!path || !buffer) return -1;
    ramfs_node_t *node = vfs_resolve(path);
    if (!node || node->type != RAMFS_FILE) return -1;
    return ramfs_read(node, buffer, 0, max_size);
}

int vfs_write_file(const char *path, const void *buffer, uint32_t size)
{
    if (!path || !buffer) return -1;

    ramfs_node_t *node = vfs_resolve(path);

    /* If the file doesn't exist, create it */
    if (!node) {
        char parent_path[VFS_MAX_PATH];
        const char *name = split_path(path, parent_path, VFS_MAX_PATH);

        ramfs_node_t *parent;
        if (parent_path[0] == '\0') {
            parent = cwd_node;
        } else {
            parent = vfs_resolve(parent_path);
        }
        if (!parent || parent->type != RAMFS_DIR) return -1;

        node = ramfs_create(parent, name, RAMFS_FILE);
        if (!node) return -1;
    }

    if (node->type != RAMFS_FILE) return -1;

    /* Truncate before overwriting */
    ramfs_truncate(node, 0);
    return ramfs_write(node, buffer, 0, size);
}

/* ---- Directory operations ---- */

int vfs_list_directory(const char *path)
{
    ramfs_node_t *dir = vfs_resolve(path);
    if (!dir || dir->type != RAMFS_DIR) return -1;

    for (uint32_t i = 0; i < dir->child_count; i++) {
        ramfs_node_t *ch = dir->children[i];
        if (!ch) continue;

        if (ch->type == RAMFS_DIR) {
            console_printf("  %s/\n", ch->name);
        } else {
            console_printf("  %-20s %u bytes\n", ch->name, ch->size);
        }
    }
    return (int)dir->child_count;
}

int vfs_mkdir(const char *path)
{
    if (!path) return -1;

    char parent_path[VFS_MAX_PATH];
    const char *name = split_path(path, parent_path, VFS_MAX_PATH);

    ramfs_node_t *parent;
    if (parent_path[0] == '\0') {
        parent = cwd_node;
    } else {
        parent = vfs_resolve(parent_path);
    }
    if (!parent || parent->type != RAMFS_DIR) return -1;

    return ramfs_create(parent, name, RAMFS_DIR) ? 0 : -1;
}

int vfs_touch(const char *path)
{
    if (!path) return -1;

    /* If it already exists, just return success */
    if (vfs_resolve(path)) return 0;

    char parent_path[VFS_MAX_PATH];
    const char *name = split_path(path, parent_path, VFS_MAX_PATH);

    ramfs_node_t *parent;
    if (parent_path[0] == '\0') {
        parent = cwd_node;
    } else {
        parent = vfs_resolve(parent_path);
    }
    if (!parent || parent->type != RAMFS_DIR) return -1;

    return ramfs_create(parent, name, RAMFS_FILE) ? 0 : -1;
}

int vfs_remove(const char *path)
{
    if (!path) return -1;
    ramfs_node_t *node = vfs_resolve(path);
    if (!node) return -1;
    return ramfs_remove(node);
}

int vfs_stat(const char *path, vfs_stat_t *out)
{
    if (!path || !out) return -1;
    ramfs_node_t *node = vfs_resolve(path);
    if (!node) return -1;

    out->type = node->type;
    out->size = node->size;
    out->child_count = (node->type == RAMFS_DIR) ? node->child_count : 0;
    return 0;
}

int vfs_copy(const char *src, const char *dst)
{
    if (!src || !dst) return -1;

    ramfs_node_t *src_node = vfs_resolve(src);
    if (!src_node || src_node->type != RAMFS_FILE) return -1;

    /* Read source content */
    uint8_t *buf = NULL;
    uint32_t sz = src_node->size;

    if (sz > 0) {
        buf = (uint8_t *)kmalloc(sz);
        if (!buf) return -1;
        int rd = ramfs_read(src_node, buf, 0, sz);
        if (rd < 0) { kfree(buf); return -1; }
        sz = (uint32_t)rd;
    }

    /* Write to destination (creates if needed) */
    int result;
    if (sz > 0) {
        result = vfs_write_file(dst, buf, sz);
        kfree(buf);
    } else {
        result = vfs_touch(dst);
    }
    return result >= 0 ? 0 : -1;
}

/* ---- Working directory ---- */

const char *vfs_getcwd(void)
{
    return cwd_path;
}

int vfs_chdir(const char *path)
{
    if (!path) return -1;

    ramfs_node_t *node = vfs_resolve(path);
    if (!node || node->type != RAMFS_DIR) return -1;

    cwd_node = node;
    ramfs_get_path(node, cwd_path, VFS_MAX_PATH);
    return 0;
}

/* ---- Legacy FAT12 mount ---- */

int vfs_mount_fat(int block_device_id)
{
    if (block_device_id < 0) return -1;

    /* Try to initialize FAT12 on the block device */
    if (fat_init(block_device_id) != 0) return -1;

    /* Create /mnt/disk mount point if it doesn't exist */
    ramfs_node_t *mnt = ramfs_lookup(NULL, "/mnt");
    if (mnt) {
        ramfs_create(mnt, "disk", RAMFS_DIR);
    }

    serial_puts("vfs: FAT12 mounted at /mnt/disk\n");
    return 0;
}
