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
    if (!buf || count <= 0) return -1;
    
    if (fd == 0) {
        /* Stdin - read from keyboard (not implemented in this phase) */
        /* Return 0 bytes read for now */
        return 0;
    }
    
    return -1;
}

int32_t sys_fork(void)
{
    struct task *current = task_get_current();
    if (!current) return -1;
    
    /* Create new task with same entry point */
    struct task *child = task_create(current->entry_point, current->priority);
    if (!child) return -1;
    
    /* Copy parent's register state to child */
    child->regs = current->regs;
    
    /* Return child's task ID to parent, 0 to child */
    return child->id;
}

int32_t sys_wait(int *status)
{
    if (!status) return -1;
    
    struct task *current = task_get_current();
    if (!current) return -1;
    
    /* Find first dead child task */
    for (int i = 0; i < task_get_count(); i++) {
        struct task *task = task_get(i);
        if (task && task->state == TASK_DEAD) {
            int child_id = task->id;
            *status = 0;
            task_destroy(task);
            return child_id;
        }
    }
    
    /* No dead children - block until one exits */
    current->state = TASK_BLOCKED;
    return 0;
}

int32_t sys_sleep(uint32_t ms)
{
    struct task *task = task_get_current();
    if (!task) return -1;
    
    /* Convert ms to timer ticks (100 Hz = 10ms per tick) */
    uint32_t ticks = (ms + 9) / 10;  /* Round up */
    
    /* Save wake-up time in task (simplified: just block) */
    task->state = TASK_BLOCKED;
    task->ticks_remaining = ticks;
    
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
