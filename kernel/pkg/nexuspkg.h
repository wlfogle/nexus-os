#ifndef KERNEL_NEXUSPKG_H
#define KERNEL_NEXUSPKG_H

#include <stdint.h>

#define NEXUSPKG_MAX_PACKAGES 16
#define NEXUSPKG_MAX_NAME     32
#define NEXUSPKG_MAX_DESC     128
#define NEXUSPKG_MAX_DEPS     4

/* Package states */
#define PKG_STATE_AVAILABLE  0
#define PKG_STATE_INSTALLED  1

typedef struct nexuspkg_entry {
    const char *name;
    const char *version;
    const char *description;
    const char *deps[NEXUSPKG_MAX_DEPS];  /* NULL-terminated dependency names */
    int         state;
    int (*install_fn)(void);
    int (*remove_fn)(void);
} nexuspkg_entry_t;

/* Initialize the package database */
void nexuspkg_init(void);

/* Shell command handler for "nexuspkg" — registered as a kshell command */
void nexuspkg_cmd(int argc, char *argv[]);

/* Query API */
int nexuspkg_count(void);
nexuspkg_entry_t *nexuspkg_get(int index);
nexuspkg_entry_t *nexuspkg_find(const char *name);

/* Operations */
int nexuspkg_install(const char *name);
int nexuspkg_remove(const char *name);

#endif /* KERNEL_NEXUSPKG_H */
