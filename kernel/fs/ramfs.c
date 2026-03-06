#include "ramfs.h"
#include "../../include/kernel/heap.h"
#include "../../include/kernel/serial.h"
#include "../../include/libc/string.h"
#include <stddef.h>

/* ------------------------------------------------------------------ */
/* Static node pool                                                    */
/* ------------------------------------------------------------------ */

static ramfs_node_t node_pool[RAMFS_MAX_NODES];
static ramfs_node_t *root_node = NULL;

static ramfs_node_t *alloc_node(void)
{
    for (int i = 0; i < RAMFS_MAX_NODES; i++) {
        if (!node_pool[i].in_use) {
            memset(&node_pool[i], 0, sizeof(ramfs_node_t));
            node_pool[i].in_use = 1;
            return &node_pool[i];
        }
    }
    return NULL;
}

static void free_node(ramfs_node_t *n)
{
    if (!n) return;
    if (n->data) {
        kfree(n->data);
        n->data = NULL;
    }
    n->in_use = 0;
}

/* ------------------------------------------------------------------ */
/* Internal helpers                                                    */
/* ------------------------------------------------------------------ */

static int name_match(const char *a, const char *b)
{
    while (*a && *b) {
        if (*a != *b) return 0;
        a++; b++;
    }
    return *a == *b;
}

/* Find a direct child of 'dir' with the given name. */
static ramfs_node_t *find_child(ramfs_node_t *dir, const char *name)
{
    if (!dir || dir->type != RAMFS_DIR) return NULL;
    for (uint32_t i = 0; i < dir->child_count; i++) {
        if (dir->children[i] && name_match(dir->children[i]->name, name)) {
            return dir->children[i];
        }
    }
    return NULL;
}

/* Add a child to a directory. */
static int add_child(ramfs_node_t *dir, ramfs_node_t *child)
{
    if (!dir || dir->type != RAMFS_DIR) return -1;
    if (dir->child_count >= RAMFS_MAX_CHILDREN) return -1;
    dir->children[dir->child_count++] = child;
    child->parent = dir;
    return 0;
}

/* Remove a child from its parent's children array. */
static int detach_child(ramfs_node_t *node)
{
    ramfs_node_t *p = node->parent;
    if (!p) return -1;

    for (uint32_t i = 0; i < p->child_count; i++) {
        if (p->children[i] == node) {
            /* Shift remaining children down */
            for (uint32_t j = i; j + 1 < p->child_count; j++) {
                p->children[j] = p->children[j + 1];
            }
            p->children[--p->child_count] = NULL;
            node->parent = NULL;
            return 0;
        }
    }
    return -1;
}

/* ------------------------------------------------------------------ */
/* Path resolution                                                     */
/* ------------------------------------------------------------------ */

ramfs_node_t *ramfs_lookup(ramfs_node_t *base, const char *path)
{
    if (!path) return NULL;

    ramfs_node_t *cur;

    /* Absolute path starts from root */
    if (path[0] == '/') {
        cur = root_node;
        path++;
    } else {
        cur = base ? base : root_node;
    }

    if (!cur) return NULL;

    /* Empty remainder means we wanted the root / base itself */
    if (*path == '\0') return cur;

    /* Walk each component separated by '/' */
    char component[RAMFS_MAX_NAME];
    while (*path) {
        /* Skip leading slashes */
        while (*path == '/') path++;
        if (*path == '\0') break;

        /* Extract next component */
        int len = 0;
        while (*path && *path != '/' && len < RAMFS_MAX_NAME - 1) {
            component[len++] = *path++;
        }
        component[len] = '\0';

        /* Handle "." and ".." */
        if (name_match(component, ".")) {
            continue;
        }
        if (name_match(component, "..")) {
            if (cur->parent) cur = cur->parent;
            continue;
        }

        /* Must be a directory to descend into */
        if (cur->type != RAMFS_DIR) return NULL;

        ramfs_node_t *child = find_child(cur, component);
        if (!child) return NULL;
        cur = child;
    }

    return cur;
}

/* ------------------------------------------------------------------ */
/* Public API                                                          */
/* ------------------------------------------------------------------ */

int ramfs_init(void)
{
    memset(node_pool, 0, sizeof(node_pool));

    root_node = alloc_node();
    if (!root_node) return -1;

    strcpy(root_node->name, "/");
    root_node->type = RAMFS_DIR;
    root_node->parent = root_node;  /* root's parent is itself */

    serial_puts("ramfs: initialized (256 nodes, 64KB max file)\n");
    return 0;
}

ramfs_node_t *ramfs_root(void)
{
    return root_node;
}

ramfs_node_t *ramfs_create(ramfs_node_t *parent, const char *name, uint8_t type)
{
    if (!parent || parent->type != RAMFS_DIR) return NULL;
    if (!name || name[0] == '\0') return NULL;
    if (strlen(name) >= RAMFS_MAX_NAME) return NULL;

    /* Reject duplicates */
    if (find_child(parent, name)) return NULL;

    ramfs_node_t *node = alloc_node();
    if (!node) return NULL;

    strcpy(node->name, name);
    node->type = type;
    node->size = 0;
    node->data = NULL;
    node->child_count = 0;

    if (add_child(parent, node) != 0) {
        free_node(node);
        return NULL;
    }

    return node;
}

int ramfs_read(ramfs_node_t *node, void *buffer, uint32_t offset, uint32_t max_size)
{
    if (!node || node->type != RAMFS_FILE || !buffer) return -1;
    if (offset >= node->size) return 0;

    uint32_t avail = node->size - offset;
    uint32_t to_read = avail < max_size ? avail : max_size;

    if (node->data) {
        memcpy(buffer, node->data + offset, to_read);
    }
    return (int)to_read;
}

int ramfs_write(ramfs_node_t *node, const void *buffer, uint32_t offset, uint32_t size)
{
    if (!node || node->type != RAMFS_FILE || !buffer) return -1;

    uint32_t end = offset + size;
    if (end > RAMFS_MAX_FILE_SIZE) return -1;

    /* Grow the backing buffer if needed */
    if (end > node->size || !node->data) {
        uint32_t new_cap = end;
        /* Round up to 256-byte granularity */
        new_cap = (new_cap + 255) & ~(uint32_t)255;
        if (new_cap > RAMFS_MAX_FILE_SIZE) new_cap = RAMFS_MAX_FILE_SIZE;

        uint8_t *new_buf = (uint8_t *)kmalloc(new_cap);
        if (!new_buf) return -1;

        memset(new_buf, 0, new_cap);
        if (node->data && node->size > 0) {
            memcpy(new_buf, node->data, node->size);
            kfree(node->data);
        }
        node->data = new_buf;
    }

    memcpy(node->data + offset, buffer, size);
    if (end > node->size) {
        node->size = end;
    }
    return (int)size;
}

int ramfs_truncate(ramfs_node_t *node, uint32_t new_size)
{
    if (!node || node->type != RAMFS_FILE) return -1;
    if (new_size > RAMFS_MAX_FILE_SIZE) return -1;

    if (new_size == 0) {
        if (node->data) {
            kfree(node->data);
            node->data = NULL;
        }
        node->size = 0;
        return 0;
    }

    if (new_size < node->size) {
        /* Zero the truncated region */
        if (node->data) {
            memset(node->data + new_size, 0, node->size - new_size);
        }
        node->size = new_size;
    }
    return 0;
}

int ramfs_remove(ramfs_node_t *node)
{
    if (!node) return -1;
    /* Cannot remove root */
    if (node == root_node) return -1;
    /* Cannot remove non-empty directory */
    if (node->type == RAMFS_DIR && node->child_count > 0) return -1;

    detach_child(node);
    free_node(node);
    return 0;
}

int ramfs_get_path(ramfs_node_t *node, char *buf, uint32_t buflen)
{
    if (!node || !buf || buflen == 0) return -1;

    /* Build path by walking up to root, then reverse */
    char tmp[512];
    int pos = 0;

    /* Collect components in reverse */
    const char *parts[64];
    int depth = 0;
    ramfs_node_t *cur = node;

    while (cur && cur != root_node && depth < 64) {
        parts[depth++] = cur->name;
        cur = cur->parent;
    }

    /* Build forward path */
    tmp[pos++] = '/';
    for (int i = depth - 1; i >= 0; i--) {
        const char *p = parts[i];
        while (*p && pos < 510) {
            tmp[pos++] = *p++;
        }
        if (i > 0 && pos < 510) {
            tmp[pos++] = '/';
        }
    }
    tmp[pos] = '\0';

    /* Copy to output buffer */
    uint32_t copy_len = (uint32_t)pos < buflen - 1 ? (uint32_t)pos : buflen - 1;
    memcpy(buf, tmp, copy_len);
    buf[copy_len] = '\0';

    return 0;
}
