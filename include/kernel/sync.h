#ifndef KERNEL_SYNC_H
#define KERNEL_SYNC_H

#include "../libc/stdint.h"

/* Mutex state */
typedef struct {
    uint32_t lock;              /* 0=unlocked, != 0=owner thread ID */
    uint32_t count;             /* Recursion count */
    uint32_t waiter_count;      /* Threads waiting on this mutex */
} mutex_t;

/* Semaphore state */
typedef struct {
    uint32_t value;             /* Semaphore counter */
    uint32_t waiter_count;      /* Threads waiting */
} semaphore_t;

/* Condition variable state */
typedef struct {
    uint32_t waiter_count;      /* Threads waiting on condition */
    mutex_t *mutex;             /* Associated mutex */
} cond_t;

/* Reader-writer lock */
typedef struct {
    uint32_t readers;           /* Number of active readers */
    uint32_t writers;           /* Number of active writers */
    uint32_t write_waiters;     /* Threads waiting to write */
    mutex_t lock;               /* Protects this structure */
} rwlock_t;

/* Mutex operations */
int mutex_init(mutex_t *mutex);
int mutex_lock(mutex_t *mutex);
int mutex_trylock(mutex_t *mutex);
int mutex_unlock(mutex_t *mutex);
int mutex_destroy(mutex_t *mutex);

/* Semaphore operations */
int semaphore_init(semaphore_t *sem, uint32_t initial_value);
int semaphore_wait(semaphore_t *sem);
int semaphore_post(semaphore_t *sem);
int semaphore_destroy(semaphore_t *sem);

/* Condition variable operations */
int cond_init(cond_t *cond, mutex_t *mutex);
int cond_wait(cond_t *cond);
int cond_signal(cond_t *cond);
int cond_broadcast(cond_t *cond);
int cond_destroy(cond_t *cond);

/* Reader-writer lock operations */
int rwlock_init(rwlock_t *rwlock);
int rwlock_read_lock(rwlock_t *rwlock);
int rwlock_read_unlock(rwlock_t *rwlock);
int rwlock_write_lock(rwlock_t *rwlock);
int rwlock_write_unlock(rwlock_t *rwlock);
int rwlock_destroy(rwlock_t *rwlock);

/* Barrier (for synchronizing multiple threads) */
typedef struct {
    uint32_t count;             /* Number of threads to wait for */
    uint32_t current;           /* Current number waiting */
    mutex_t lock;               /* Protects this structure */
} barrier_t;

int barrier_init(barrier_t *barrier, uint32_t count);
int barrier_wait(barrier_t *barrier);
int barrier_destroy(barrier_t *barrier);

#endif /* KERNEL_SYNC_H */
