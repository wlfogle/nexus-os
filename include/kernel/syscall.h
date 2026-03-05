#ifndef KERNEL_SYSCALL_H
#define KERNEL_SYSCALL_H

#include "../libc/stdint.h"

#define SYSCALL_EXIT   1
#define SYSCALL_WRITE  2
#define SYSCALL_READ   3
#define SYSCALL_FORK   4
#define SYSCALL_WAIT   5
#define SYSCALL_SLEEP  6
#define SYSCALL_OPEN   7
#define SYSCALL_CLOSE  8
#define SYSCALL_MKDIR  9
#define SYSCALL_UNLINK 10
#define SYSCALL_LSEEK  11
#define SYSCALL_EXEC   12
#define SYSCALL_SIGNAL 13

struct syscall_args {
    uint32_t eax, ebx, ecx, edx, esi, edi;
};

int32_t sys_exit(int code);
int32_t sys_write(int fd, const char *buf, int count);
int32_t sys_open(const char *path, int flags);
int32_t sys_close(int fd);
int32_t sys_mkdir(const char *path);
int32_t sys_unlink(const char *path);
int32_t sys_lseek(int fd, int32_t offset, int whence);
int32_t sys_exec(const char *filename, char *const argv[]);
int32_t sys_signal(int signum, uint32_t handler);

void syscall_init(void);
int32_t syscall_dispatch(uint32_t num, struct syscall_args *args);

#endif
