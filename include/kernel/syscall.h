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

int32_t sys_exit(int code);
int32_t sys_write(int fd, const char *buf, int count);

void syscall_init(void);
int32_t syscall_dispatch(uint32_t num, struct syscall_args *args);

#endif
