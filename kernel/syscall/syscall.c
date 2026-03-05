#include "../../include/kernel/syscall.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/task.h"
#include "../../include/kernel/tcp.h"
#include "../../include/kernel/udp.h"
#include "../../include/kernel/netdev.h"
#include "../../include/kernel/thread.h"
#include "../../include/kernel/sync.h"
#include "../../include/kernel/futex.h"
#include "../../include/kernel/paging.h"

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

/* Socket syscalls */
int32_t sys_socket(int domain, int type, int protocol)
{
    (void)domain;  /* AF_INET implied */
    (void)protocol;  /* Determined by type */
    
    struct task *task = task_get_current();
    if (!task) return -1;
    
    int socket_id = -1;
    
    if (type == 1) {  /* SOCK_STREAM = TCP */
        socket_id = tcp_socket_create();
    } else if (type == 2) {  /* SOCK_DGRAM = UDP */
        socket_id = udp_socket_create(0);  /* Ephemeral port */
    } else {
        return -1;
    }
    
    if (socket_id < 0) return -1;
    
    /* Find free fd and store socket_id */
    int fd = -1;
    for (int i = 3; i < MAX_FD_PER_TASK; i++) {
        if (!task->fd_table[i].in_use) {
            fd = i;
            break;
        }
    }
    
    if (fd < 0) return -1;
    
    task->fd_table[fd].in_use = 1;
    task->fd_table[fd].vfs_handle = socket_id;  /* Store socket ID in vfs_handle */
    task->fd_table[fd].flags = type;  /* Store socket type */
    
    return fd;
}

int32_t sys_bind(int sockfd, const struct sockaddr *addr, int addrlen)
{
    struct task *task = task_get_current();
    if (!task || sockfd < 0 || sockfd >= MAX_FD_PER_TASK) return -1;
    if (!task->fd_table[sockfd].in_use) return -1;
    
    int socket_id = task->fd_table[sockfd].vfs_handle;
    int socket_type = task->fd_table[sockfd].flags;
    
    if (addrlen < 8) return -1;  /* Need at least addr_family + port */
    
    uint16_t port = ((uint8_t *)addr)[2] << 8 | ((uint8_t *)addr)[3];
    
    if (socket_type == 1) {  /* TCP */
        return tcp_socket_listen(socket_id, port);
    } else if (socket_type == 2) {  /* UDP */
        /* UDP binding handled at socket creation */
        return 0;
    }
    
    return -1;
}

int32_t sys_listen(int sockfd, int backlog)
{
    struct task *task = task_get_current();
    if (!task || sockfd < 0 || sockfd >= MAX_FD_PER_TASK) return -1;
    if (!task->fd_table[sockfd].in_use) return -1;
    
    (void)backlog;  /* Simplified: we accept any backlog */
    
    /* For TCP, listen is already called in bind */
    return 0;
}

int32_t sys_connect(int sockfd, const struct sockaddr *addr, int addrlen)
{
    struct task *task = task_get_current();
    if (!task || sockfd < 0 || sockfd >= MAX_FD_PER_TASK) return -1;
    if (!task->fd_table[sockfd].in_use) return -1;
    
    int socket_id = task->fd_table[sockfd].vfs_handle;
    int socket_type = task->fd_table[sockfd].flags;
    
    if (addrlen < 8) return -1;
    
    /* Extract IP and port from sockaddr_in */
    ipv4_addr_t dest_ip;
    dest_ip.addr[0] = ((uint8_t *)addr)[4];
    dest_ip.addr[1] = ((uint8_t *)addr)[5];
    dest_ip.addr[2] = ((uint8_t *)addr)[6];
    dest_ip.addr[3] = ((uint8_t *)addr)[7];
    uint16_t port = ((uint8_t *)addr)[2] << 8 | ((uint8_t *)addr)[3];
    
    if (socket_type == 1) {  /* TCP */
        return tcp_socket_connect(socket_id, &dest_ip, port);
    }
    
    return -1;
}

int32_t sys_accept(int sockfd, struct sockaddr *addr, int *addrlen)
{
    struct task *task = task_get_current();
    if (!task || sockfd < 0 || sockfd >= MAX_FD_PER_TASK) return -1;
    if (!task->fd_table[sockfd].in_use) return -1;
    
    int socket_id = task->fd_table[sockfd].vfs_handle;
    int socket_type = task->fd_table[sockfd].flags;
    
    if (socket_type != 1) return -1;  /* Only TCP */
    
    int client_socket_id = tcp_socket_accept(socket_id);
    if (client_socket_id < 0) return -1;  /* No pending connections */
    
    /* Find free fd for client */
    int fd = -1;
    for (int i = 3; i < MAX_FD_PER_TASK; i++) {
        if (!task->fd_table[i].in_use) {
            fd = i;
            break;
        }
    }
    
    if (fd < 0) return -1;
    
    task->fd_table[fd].in_use = 1;
    task->fd_table[fd].vfs_handle = client_socket_id;
    task->fd_table[fd].flags = socket_type;
    
    (void)addr;  /* Simplified: don't fill addr */
    (void)addrlen;
    
    return fd;
}

int32_t sys_send(int sockfd, const void *buf, int len, int flags)
{
    struct task *task = task_get_current();
    if (!task || sockfd < 0 || sockfd >= MAX_FD_PER_TASK) return -1;
    if (!task->fd_table[sockfd].in_use || !buf || len <= 0) return -1;
    
    int socket_id = task->fd_table[sockfd].vfs_handle;
    int socket_type = task->fd_table[sockfd].flags;
    
    (void)flags;  /* Simplified: ignore flags for now */
    
    if (socket_type == 1) {  /* TCP */
        return tcp_socket_send(socket_id, (const uint8_t *)buf, len);
    }
    
    return -1;
}

int32_t sys_recv(int sockfd, void *buf, int len, int flags)
{
    struct task *task = task_get_current();
    if (!task || sockfd < 0 || sockfd >= MAX_FD_PER_TASK) return -1;
    if (!task->fd_table[sockfd].in_use || !buf || len <= 0) return -1;
    
    int socket_id = task->fd_table[sockfd].vfs_handle;
    int socket_type = task->fd_table[sockfd].flags;
    
    (void)flags;  /* Simplified: ignore flags for now */
    
    if (socket_type == 1) {  /* TCP */
        return tcp_socket_recv(socket_id, (uint8_t *)buf, len);
    }
    
    return -1;
}

/* Thread syscalls */
int32_t sys_clone(int flags, void *stack, int (*fn)(void *), void *arg, int *parent_tid)
{
    (void)flags;  /* CLONE_THREAD, CLONE_VM, etc - simplified for now */
    (void)stack;  /* Child stack - using allocated stack */
    (void)parent_tid;
    
    struct task *task = task_get_current();
    if (!task || !fn) return -1;
    
    /* Create new thread in current task */
    int thread_id = thread_create(task, (void (*)(void *))fn, arg);
    return thread_id;
}

int32_t sys_thread_join(int thread_id, int *exit_code)
{
    if (thread_id <= 0) return -1;
    
    return thread_join(thread_id, exit_code);
}

int32_t sys_mutex_lock(void *mutex_addr)
{
    if (!mutex_addr) return -1;
    
    mutex_t *mutex = (mutex_t *)mutex_addr;
    return mutex_lock(mutex);
}

int32_t sys_mutex_unlock(void *mutex_addr)
{
    if (!mutex_addr) return -1;
    
    mutex_t *mutex = (mutex_t *)mutex_addr;
    return mutex_unlock(mutex);
}

/* Futex syscalls */
int32_t sys_futex_wait(uint32_t *futex_addr, uint32_t expected_val, uint32_t timeout_ms)
{
    if (!futex_addr) return -1;
    
    return futex_wait(futex_addr, expected_val, timeout_ms);
}

int32_t sys_futex_wake(uint32_t *futex_addr, uint32_t num_waiters)
{
    if (!futex_addr) return -1;
    
    return futex_wake(futex_addr, num_waiters);
}

int32_t sys_futex_requeue(uint32_t *futex_addr1, uint32_t *futex_addr2, uint32_t num_wake, uint32_t num_requeue)
{
    if (!futex_addr1 || !futex_addr2) return -1;
    
    return futex_requeue(futex_addr1, futex_addr2, num_wake, num_requeue);
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
        case SYSCALL_SOCKET:
            return sys_socket(args->ebx, args->ecx, args->edx);
        case SYSCALL_BIND:
            return sys_bind(args->ebx, (struct sockaddr *)args->ecx, args->edx);
        case SYSCALL_LISTEN:
            return sys_listen(args->ebx, args->ecx);
        case SYSCALL_CONNECT:
            return sys_connect(args->ebx, (struct sockaddr *)args->ecx, args->edx);
        case SYSCALL_ACCEPT:
            return sys_accept(args->ebx, (struct sockaddr *)args->ecx, (int *)args->edx);
        case SYSCALL_SEND:
            return sys_send(args->ebx, (void *)args->ecx, args->edx, args->esi);
        case SYSCALL_RECV:
            return sys_recv(args->ebx, (void *)args->ecx, args->edx, args->esi);
        case SYSCALL_CLONE:
            return sys_clone(args->ebx, (void *)args->ecx, (int (*)(void *))args->edx, (void *)args->esi, (int *)args->edi);
        case SYSCALL_THREAD_JOIN:
            return sys_thread_join(args->ebx, (int *)args->ecx);
        case SYSCALL_MUTEX_LOCK:
            return sys_mutex_lock((void *)args->ebx);
        case SYSCALL_MUTEX_UNLOCK:
            return sys_mutex_unlock((void *)args->ebx);
        case SYSCALL_FUTEX_WAIT:
            return sys_futex_wait((uint32_t *)args->ebx, args->ecx, args->edx);
        case SYSCALL_FUTEX_WAKE:
            return sys_futex_wake((uint32_t *)args->ebx, args->ecx);
        case SYSCALL_FUTEX_REQUEUE:
            return sys_futex_requeue((uint32_t *)args->ebx, (uint32_t *)args->ecx, args->edx, args->esi);
        default:
            return -1;
    }
}
