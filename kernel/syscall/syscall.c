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
    if (!buf || count <= 0) return -1;
    
    struct task *task = task_get_current();
    if (!task || fd < 0 || fd >= MAX_FD_PER_TASK) return -1;
    
    if (!task->fd_table[fd].in_use) return -1;
    
    /* fd=1 (stdout) and fd=2 (stderr) go to serial */
    if (fd == 1 || fd == 2) {
        for (int i = 0; i < count; i++) {
            serial_putchar(buf[i]);
        }
        return count;
    }
    
    /* File write would go here in Phase 5 */
    return -1;
}

int32_t sys_read(int fd, char *buf, int count)
{
    if (!buf || count <= 0) return -1;
    
    struct task *task = task_get_current();
    if (!task || fd < 0 || fd >= MAX_FD_PER_TASK) return -1;
    
    if (!task->fd_table[fd].in_use) return -1;
    
    /* fd=0 (stdin) - read from keyboard buffer (not implemented in Phase 5.1) */
    if (fd == 0) {
        /* Placeholder: return 0 for now */
        /* Will implement keyboard buffering in Phase 5.3 */
        return 0;
    }
    
    /* File read would go here in Phase 5 */
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

int32_t sys_open(const char *path, int flags)
{
    struct task *task = task_get_current();
    if (!task || !path) return -1;
    
    /* Find first free file descriptor */
    int fd = -1;
    for (int i = 3; i < MAX_FD_PER_TASK; i++) {
        if (!task->fd_table[i].in_use) {
            fd = i;
            break;
        }
    }
    
    if (fd < 0) return -1;  /* No free file descriptors */
    
    /* For Phase 5, file operations use VFS (stub for now) */
    /* In Phase 5.5, integrate with actual VFS/FAT driver */
    task->fd_table[fd].in_use = 1;
    task->fd_table[fd].vfs_handle = 0;  /* Placeholder */
    task->fd_table[fd].offset = 0;
    task->fd_table[fd].flags = flags;
    
    return fd;
}

int32_t sys_close(int fd)
{
    struct task *task = task_get_current();
    if (!task || fd < 0 || fd >= MAX_FD_PER_TASK) return -1;
    
    if (!task->fd_table[fd].in_use) return -1;
    
    /* Close the file descriptor */
    task->fd_table[fd].in_use = 0;
    task->fd_table[fd].vfs_handle = -1;
    
    return 0;
}

int32_t sys_mkdir(const char *path)
{
    /* File system operations deferred to Phase 5.5+ */
    (void)path;
    return -1;
}

int32_t sys_unlink(const char *path)
{
    /* File system operations deferred to Phase 5.5+ */
    (void)path;
    return -1;
}

int32_t sys_lseek(int fd, int32_t offset, int whence)
{
    struct task *task = task_get_current();
    if (!task || fd < 0 || fd >= MAX_FD_PER_TASK) return -1;
    
    if (!task->fd_table[fd].in_use) return -1;
    
    /* Implement seek logic */
    switch (whence) {
        case 0:  /* SEEK_SET */
            task->fd_table[fd].offset = offset;
            break;
        case 1:  /* SEEK_CUR */
            task->fd_table[fd].offset += offset;
            break;
        case 2:  /* SEEK_END */
            /* Would need file size from VFS */
            return -1;
        default:
            return -1;
    }
    
    return task->fd_table[fd].offset;
}

int32_t sys_exec(const char *filename, char *const argv[])
{
    /* exec syscall deferred to Phase 5.2 */
    /* For now, stub that returns error */
    (void)filename;
    (void)argv;
    return -1;
}

int32_t sys_signal(int signum, uint32_t handler)
{
    struct task *task = task_get_current();
    if (!task || signum < 0 || signum >= 32) return -1;
    
    task->signal_handlers[signum] = (signal_handler_t)handler;
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
        case SYSCALL_OPEN:
            return sys_open((const char *)args->ebx, args->ecx);
        case SYSCALL_CLOSE:
            return sys_close(args->ebx);
        case SYSCALL_MKDIR:
            return sys_mkdir((const char *)args->ebx);
        case SYSCALL_UNLINK:
            return sys_unlink((const char *)args->ebx);
        case SYSCALL_LSEEK:
            return sys_lseek(args->ebx, args->ecx, args->edx);
        case SYSCALL_EXEC:
            return sys_exec((const char *)args->ebx, (char *const *)args->ecx);
        case SYSCALL_SIGNAL:
            return sys_signal(args->ebx, args->ecx);
        default:
            return -1;
    }
}
