#ifndef KERNEL_TASK_H
#define KERNEL_TASK_H

#include "../libc/stdint.h"

#define MAX_TASKS 32
#define TASK_STACK_SIZE 4096
#define MAX_FD_PER_TASK 32
#define MAX_OPEN_FILES 256

typedef enum {
    TASK_READY,
    TASK_RUNNING,
    TASK_BLOCKED,
    TASK_DEAD
} task_state_t;

/* File descriptor entry */
struct file_descriptor {
    int vfs_handle;          /* Handle from VFS layer */
    uint32_t flags;          /* O_RDONLY, O_WRONLY, etc */
    uint32_t offset;         /* Current file position */
    uint8_t in_use;          /* 1 if allocated, 0 if free */
};

/* Open file table (shared across tasks) */
struct open_file {
    char path[256];
    uint32_t inode;          /* VFS inode number */
    uint32_t ref_count;      /* Number of open file descriptors pointing here */
    uint32_t size;           /* File size in bytes */
};

/* Signal handler */
typedef void (*signal_handler_t)(int);

typedef enum {
    SIGCHLD = 17,
    SIGTERM = 15
} signal_t;

struct registers {
    uint32_t eax, ebx, ecx, edx, esi, edi, ebp, esp;
    uint32_t eip, eflags, cs, ss;
};

struct task {
    uint32_t id;
    task_state_t state;
    struct registers regs;
    uint32_t *page_directory;
    uint32_t kernel_stack;
    uint32_t user_stack;
    uint32_t entry_point;
    int priority;
    int ticks_remaining;
    
    /* File descriptors */
    struct file_descriptor fd_table[MAX_FD_PER_TASK];
    
    /* Signal handling */
    signal_handler_t signal_handlers[32];
    int pending_signals;     /* Bitmask of pending signals */
    
    /* Parent/child tracking */
    uint32_t parent_id;
    int exit_code;
};

void task_init(void);
struct task *task_create(uint32_t entry_point, int priority);
void task_destroy(struct task *task);
struct task *task_get_current(void);
void task_set_current(struct task *task);
void task_yield(void);
void task_sleep(uint32_t ms);
int task_get_count(void);
struct task *task_get(int index);

#endif
