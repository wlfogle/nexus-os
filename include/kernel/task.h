#ifndef KERNEL_TASK_H
#define KERNEL_TASK_H

#include "../libc/stdint.h"

#define MAX_TASKS 32
#define TASK_STACK_SIZE 4096

typedef enum {
    TASK_READY,
    TASK_RUNNING,
    TASK_BLOCKED,
    TASK_DEAD
} task_state_t;

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
};

void task_init(void);
struct task *task_create(uint32_t entry_point, int priority);
void task_destroy(struct task *task);
struct task *task_get_current(void);
void task_set_current(struct task *task);
void task_yield(void);
void task_sleep(uint32_t ms);

#endif
