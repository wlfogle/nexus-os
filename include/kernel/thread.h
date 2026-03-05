#ifndef KERNEL_THREAD_H
#define KERNEL_THREAD_H

#include "../libc/stdint.h"
#include "task.h"

#define MAX_THREADS_PER_TASK 16
#define THREAD_STACK_SIZE 4096

/* Thread states */
typedef enum {
    THREAD_CREATED = 0,
    THREAD_READY = 1,
    THREAD_RUNNING = 2,
    THREAD_BLOCKED = 3,
    THREAD_EXITED = 4
} thread_state_t;

/* Thread-local storage (TLS) area */
struct thread_tls {
    uint32_t thread_id;
    uint32_t errno_val;
    void *user_data[4];  /* User-accessible TLS slots */
};

/* Thread control block */
struct thread {
    uint32_t id;
    uint32_t task_id;           /* Parent task ID */
    thread_state_t state;
    
    /* Stack management */
    uint32_t *stack;
    uint32_t stack_base;
    uint32_t stack_size;
    
    /* CPU state */
    struct registers regs;
    
    /* Entry point and argument */
    void (*entry_point)(void *);
    void *arg;
    
    /* Return value */
    int exit_code;
    
    /* TLS */
    struct thread_tls tls;
    
    /* Synchronization */
    uint32_t join_waiter_id;    /* Thread waiting on join */
    uint32_t refcount;          /* Reference count for cleanup */
};

/* Thread management operations */
int thread_create(struct task *task, void (*entry)(void *), void *arg);
int thread_join(int thread_id, int *exit_code);
int thread_exit(int exit_code);

/* Thread utilities */
int thread_self(void);
int thread_get_id(int thread_id);
struct thread *thread_get(int thread_id);
void thread_sleep_ms(uint32_t ms);

/* Thread-local storage */
void *thread_get_tls(int slot);
int thread_set_tls(int slot, void *value);

#endif /* KERNEL_THREAD_H */
