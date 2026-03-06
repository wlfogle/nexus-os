#include "../../include/kernel/task.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/heap.h"
#include "../../include/kernel/pmem.h"
#include <stddef.h>

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
    
    /* Initialize file descriptors (0=stdin, 1=stdout, 2=stderr) */
    for (int i = 0; i < MAX_FD_PER_TASK; i++) {
        task->fd_table[i].in_use = 0;
        task->fd_table[i].vfs_handle = -1;
        task->fd_table[i].offset = 0;
        task->fd_table[i].flags = 0;
    }
    
    /* Reserve standard file descriptors */
    task->fd_table[0].in_use = 1;  /* stdin */
    task->fd_table[0].vfs_handle = 0;  /* keyboard */
    task->fd_table[1].in_use = 1;  /* stdout */
    task->fd_table[1].vfs_handle = 1;  /* serial */
    task->fd_table[2].in_use = 1;  /* stderr */
    task->fd_table[2].vfs_handle = 1;  /* serial */
    
    /* Initialize signal handlers and tracking */
    for (int i = 0; i < 32; i++) {
        task->signal_handlers[i] = NULL;
    }
    task->pending_signals = 0;
    task->parent_id = 0;
    task->exit_code = 0;
    
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
    (void)ms;
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
