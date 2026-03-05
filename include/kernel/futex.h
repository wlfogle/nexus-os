#ifndef KERNEL_FUTEX_H
#define KERNEL_FUTEX_H

#include "../libc/stdint.h"

#define FUTEX_WAIT      0
#define FUTEX_WAKE      1
#define FUTEX_REQUEUE   2
#define FUTEX_CMP_REQUEUE 3

/* Futex operation result codes */
#define FUTEX_OK        0
#define FUTEX_TIMEOUT   -1
#define FUTEX_INVAL     -2

/* Futex syscall - atomic wait/wake on userspace address */
int futex_wait(uint32_t *futex_addr, uint32_t expected_val, uint32_t timeout_ms);
int futex_wake(uint32_t *futex_addr, uint32_t num_waiters);
int futex_requeue(uint32_t *futex_addr1, uint32_t *futex_addr2, uint32_t num_wake, uint32_t num_requeue);

#endif /* KERNEL_FUTEX_H */
