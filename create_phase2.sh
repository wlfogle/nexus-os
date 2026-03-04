#!/bin/bash
mkdir -p kernel/proc kernel/syscall userland/bin

# Create task.h - Task Control Block
cat > include/kernel/task.h << 'EOF'
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
EOF

# Create task.c - Task management
cat > kernel/proc/task.c << 'EOF'
#include "../../include/kernel/task.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/heap.h"
#include "../../include/kernel/pmem.h"

static struct task task_list[MAX_TASKS];
static int task_count = 0;
static struct task *current_task = NULL;

void task_init(void)
{
    for (int i = 0; i < MAX_TASKS; i++) {
        task_list[i].id = 0;
        task_list[i].state = TASK_DEAD;
    }
    task_count = 0;
    current_task = NULL;
    serial_puts("Task manager initialized\n");
}

struct task *task_create(uint32_t entry_point, int priority)
{
    if (task_count >= MAX_TASKS) return NULL;
    
    struct task *task = &task_list[task_count];
    task->id = task_count + 1;
    task->entry_point = entry_point;
    task->state = TASK_READY;
    task->priority = priority;
    task->ticks_remaining = 10;
    
    task->kernel_stack = pmem_alloc_page();
    task->user_stack = pmem_alloc_page();
    
    task->regs.esp = task->user_stack + TASK_STACK_SIZE - 4;
    task->regs.eip = entry_point;
    task->regs.eflags = 0x200;
    task->regs.cs = 0x1B;
    task->regs.ss = 0x23;
    
    task_count++;
    return task;
}

void task_destroy(struct task *task)
{
    if (task) {
        pmem_free_page(task->kernel_stack);
        pmem_free_page(task->user_stack);
        task->state = TASK_DEAD;
    }
}

struct task *task_get_current(void)
{
    return current_task;
}

void task_set_current(struct task *task)
{
    current_task = task;
    if (task) task->state = TASK_RUNNING;
}

void task_yield(void)
{
    /* Signal scheduler to switch tasks */
}

void task_sleep(uint32_t ms)
{
    if (current_task) {
        current_task->state = TASK_BLOCKED;
    }
}

int task_get_count(void)
{
    return task_count;
}

struct task *task_get(int index)
{
    if (index >= 0 && index < task_count) {
        return &task_list[index];
    }
    return NULL;
}
EOF

# Create scheduler.h
cat > include/kernel/scheduler.h << 'EOF'
#ifndef KERNEL_SCHEDULER_H
#define KERNEL_SCHEDULER_H

#include "task.h"

void scheduler_init(void);
struct task *scheduler_pick_next(void);
void scheduler_tick(void);

#endif
EOF

# Create scheduler.c
cat > kernel/proc/scheduler.c << 'EOF'
#include "../../include/kernel/scheduler.h"
#include "../../include/kernel/serial.h"

static struct task *last_task = NULL;
static int current_index = 0;

void scheduler_init(void)
{
    serial_puts("Scheduler initialized\n");
}

struct task *scheduler_pick_next(void)
{
    int start = current_index;
    
    do {
        struct task *task = task_get(current_index);
        current_index = (current_index + 1) % task_get_count();
        
        if (task && (task->state == TASK_READY || task->state == TASK_RUNNING)) {
            return task;
        }
    } while (current_index != start);
    
    return NULL;
}

void scheduler_tick(void)
{
    struct task *current = task_get_current();
    
    if (current) {
        current->ticks_remaining--;
        if (current->ticks_remaining <= 0) {
            current->state = TASK_READY;
            current->ticks_remaining = 10;
            
            struct task *next = scheduler_pick_next();
            if (next && next != current) {
                task_set_current(next);
            }
        }
    }
}
EOF

# Create timer.c - PIT timer interrupt
cat > kernel/irq/timer.c << 'EOF'
#include "../../include/kernel/pic.h"
#include "../../include/kernel/scheduler.h"
#include "../../include/kernel/serial.h"

#define PIT_CHANNEL0 0x40
#define PIT_CMD 0x43
#define PIT_HZ 100

static int ticks = 0;

void timer_init(void)
{
    uint32_t divisor = 1193182 / PIT_HZ;
    
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)(divisor & 0xFF)), "Nd"(PIT_CMD));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)0x36), "Nd"(PIT_CMD));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)((divisor >> 8) & 0xFF)), "Nd"(PIT_CHANNEL0));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)(divisor & 0xFF)), "Nd"(PIT_CHANNEL0));
    
    pic_enable_irq(0);
    serial_puts("Timer initialized (100 Hz)\n");
}

void timer_interrupt(void)
{
    ticks++;
    scheduler_tick();
    pic_send_eoi(0);
}

int timer_get_ticks(void)
{
    return ticks;
}
EOF

# Create timer.h
cat > include/kernel/timer.h << 'EOF'
#ifndef KERNEL_TIMER_H
#define KERNEL_TIMER_H

void timer_init(void);
void timer_interrupt(void);
int timer_get_ticks(void);

#endif
EOF

# Create syscall.h
cat > include/kernel/syscall.h << 'EOF'
#ifndef KERNEL_SYSCALL_H
#define KERNEL_SYSCALL_H

#include "../libc/stdint.h"

#define SYSCALL_EXIT 1
#define SYSCALL_WRITE 2
#define SYSCALL_READ 3
#define SYSCALL_FORK 4
#define SYSCALL_WAIT 5
#define SYSCALL_SLEEP 6

struct syscall_args {
    uint32_t eax, ebx, ecx, edx, esi, edi;
};

void syscall_init(void);
int32_t syscall_dispatch(uint32_t num, struct syscall_args *args);

#endif
EOF

# Create syscall.c - Syscall dispatcher
cat > kernel/syscall/syscall.c << 'EOF'
#include "../../include/kernel/syscall.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/task.h"

int32_t sys_exit(int code)
{
    struct task *task = task_get_current();
    if (task) {
        task->state = TASK_DEAD;
    }
    return 0;
}

int32_t sys_write(int fd, const char *buf, int count)
{
    if (fd == 1) {
        for (int i = 0; i < count; i++) {
            serial_putchar(buf[i]);
        }
        return count;
    }
    return -1;
}

int32_t sys_read(int fd, char *buf, int count)
{
    return 0;
}

int32_t sys_fork(void)
{
    return -1;
}

int32_t sys_wait(int *status)
{
    return -1;
}

int32_t sys_sleep(uint32_t ms)
{
    struct task *task = task_get_current();
    if (task) {
        task->state = TASK_BLOCKED;
    }
    return 0;
}

int32_t syscall_dispatch(uint32_t num, struct syscall_args *args)
{
    switch (num) {
        case SYSCALL_EXIT:
            return sys_exit(args->ebx);
        case SYSCALL_WRITE:
            return sys_write(args->ebx, (char *)args->ecx, args->edx);
        case SYSCALL_READ:
            return sys_read(args->ebx, (char *)args->ecx, args->edx);
        case SYSCALL_FORK:
            return sys_fork();
        case SYSCALL_WAIT:
            return sys_wait((int *)args->ebx);
        case SYSCALL_SLEEP:
            return sys_sleep(args->ebx);
        default:
            return -1;
    }
}
EOF

echo "Phase 2 files created successfully"
