#include "../../include/kernel/sync.h"
#include "../../include/kernel/thread.h"
#include "../../include/kernel/serial.h"
#include <string.h>

/* Priority inheritance: track original priority when boosting */
struct priority_entry {
    uint32_t thread_id;
    uint32_t original_priority;
    uint32_t boosted_priority;
};

static struct priority_entry priority_table[256];
static int priority_table_count = 0;

static void priority_boost(uint32_t thread_id, uint32_t new_priority)
{
    /* Find or create priority entry */
    for (int i = 0; i < priority_table_count; i++) {
        if (priority_table[i].thread_id == thread_id) {
            priority_table[i].boosted_priority = new_priority;
            return;
        }
    }
    
    if (priority_table_count < 256) {
        priority_table[priority_table_count].thread_id = thread_id;
        priority_table[priority_table_count].original_priority = thread_id % 4;  /* Simplified */
        priority_table[priority_table_count].boosted_priority = new_priority;
        priority_table_count++;
    }
}

static void priority_restore(uint32_t thread_id)
{
    /* Restore original priority */
    for (int i = 0; i < priority_table_count; i++) {
        if (priority_table[i].thread_id == thread_id) {
            priority_table[i].boosted_priority = priority_table[i].original_priority;
            return;
        }
    }
}

static uint32_t get_thread_priority(uint32_t thread_id)
{
    /* Get current priority (boosted or original) */
    for (int i = 0; i < priority_table_count; i++) {
        if (priority_table[i].thread_id == thread_id) {
            return priority_table[i].boosted_priority;
        }
    }
    return thread_id % 4;  /* Default based on ID */
}

/* Mutex operations */
int mutex_init(mutex_t *mutex)
{
    if (!mutex) return -1;
    
    memset(mutex, 0, sizeof(mutex_t));
    mutex->lock = 0;
    mutex->count = 0;
    mutex->waiter_count = 0;
    
    return 0;
}

int mutex_lock(mutex_t *mutex)
{
    if (!mutex) return -1;
    
    uint32_t thread_id = thread_self();
    uint32_t owner_id = mutex->lock;
    
    /* Check if already locked by this thread (recursive mutex) */
    if (mutex->lock == thread_id) {
        mutex->count++;
        return 0;
    }
    
    /* Wait for mutex to be unlocked with priority inheritance */
    while (mutex->lock != 0) {
        /* Boost owner's priority to match waiting thread's priority */
        uint32_t waiting_priority = get_thread_priority(thread_id);
        uint32_t owner_priority = get_thread_priority(owner_id);
        
        if (waiting_priority > owner_priority) {
            priority_boost(owner_id, waiting_priority);
        }
        
        mutex->waiter_count++;
        thread_sleep_ms(1);  /* Simplified: busy wait */
        mutex->waiter_count--;
        
        owner_id = mutex->lock;
    }
    
    /* Acquire the lock */
    mutex->lock = thread_id;
    mutex->count = 1;
    
    return 0;
}

int mutex_trylock(mutex_t *mutex)
{
    if (!mutex) return -1;
    
    uint32_t thread_id = thread_self();
    
    /* Try to acquire without blocking */
    if (mutex->lock == 0) {
        mutex->lock = thread_id;
        mutex->count = 1;
        return 0;
    }
    
    /* Already locked */
    return -1;
}

int mutex_unlock(mutex_t *mutex)
{
    if (!mutex) return -1;
    
    uint32_t thread_id = thread_self();
    
    /* Check if owned by this thread */
    if (mutex->lock != thread_id) {
        return -1;
    }
    
    mutex->count--;
    
    /* Only release if count reaches 0 (recursive mutex) */
    if (mutex->count == 0) {
        /* Restore original priority before releasing */
        priority_restore(thread_id);
        mutex->lock = 0;
    }
    
    return 0;
}

int mutex_destroy(mutex_t *mutex)
{
    if (!mutex) return -1;
    
    /* Check if any threads are waiting */
    if (mutex->waiter_count > 0 || mutex->lock != 0) {
        return -1;
    }
    
    memset(mutex, 0, sizeof(mutex_t));
    return 0;
}

/* Semaphore operations */
int semaphore_init(semaphore_t *sem, uint32_t initial_value)
{
    if (!sem) return -1;
    
    memset(sem, 0, sizeof(semaphore_t));
    sem->value = initial_value;
    sem->waiter_count = 0;
    
    return 0;
}

int semaphore_wait(semaphore_t *sem)
{
    if (!sem) return -1;
    
    /* Wait for semaphore to be non-zero */
    while (sem->value == 0) {
        sem->waiter_count++;
        thread_sleep_ms(1);  /* Simplified: busy wait */
        sem->waiter_count--;
    }
    
    /* Decrement semaphore */
    sem->value--;
    
    return 0;
}

int semaphore_post(semaphore_t *sem)
{
    if (!sem) return -1;
    
    /* Increment semaphore */
    sem->value++;
    
    return 0;
}

int semaphore_destroy(semaphore_t *sem)
{
    if (!sem) return -1;
    
    if (sem->waiter_count > 0) {
        return -1;
    }
    
    memset(sem, 0, sizeof(semaphore_t));
    return 0;
}

/* Condition variable operations */
int cond_init(cond_t *cond, mutex_t *mutex)
{
    if (!cond || !mutex) return -1;
    
    memset(cond, 0, sizeof(cond_t));
    cond->mutex = mutex;
    cond->waiter_count = 0;
    
    return 0;
}

int cond_wait(cond_t *cond)
{
    if (!cond || !cond->mutex) return -1;
    
    cond->waiter_count++;
    
    /* Release mutex and wait */
    mutex_unlock(cond->mutex);
    thread_sleep_ms(10);  /* Simplified: sleep instead of real wait */
    mutex_lock(cond->mutex);
    
    cond->waiter_count--;
    
    return 0;
}

int cond_signal(cond_t *cond)
{
    if (!cond) return -1;
    
    /* Wake up one waiter (simplified: no-op since we use polling) */
    return 0;
}

int cond_broadcast(cond_t *cond)
{
    if (!cond) return -1;
    
    /* Wake up all waiters (simplified: no-op since we use polling) */
    return 0;
}

int cond_destroy(cond_t *cond)
{
    if (!cond) return -1;
    
    if (cond->waiter_count > 0) {
        return -1;
    }
    
    memset(cond, 0, sizeof(cond_t));
    return 0;
}

/* Reader-writer lock operations */
int rwlock_init(rwlock_t *rwlock)
{
    if (!rwlock) return -1;
    
    memset(rwlock, 0, sizeof(rwlock_t));
    rwlock->readers = 0;
    rwlock->writers = 0;
    rwlock->write_waiters = 0;
    mutex_init(&rwlock->lock);
    
    return 0;
}

int rwlock_read_lock(rwlock_t *rwlock)
{
    if (!rwlock) return -1;
    
    /* Wait if writers are active or waiting */
    while (rwlock->writers > 0 || rwlock->write_waiters > 0) {
        thread_sleep_ms(1);
    }
    
    mutex_lock(&rwlock->lock);
    rwlock->readers++;
    mutex_unlock(&rwlock->lock);
    
    return 0;
}

int rwlock_read_unlock(rwlock_t *rwlock)
{
    if (!rwlock) return -1;
    
    mutex_lock(&rwlock->lock);
    rwlock->readers--;
    mutex_unlock(&rwlock->lock);
    
    return 0;
}

int rwlock_write_lock(rwlock_t *rwlock)
{
    if (!rwlock) return -1;
    
    mutex_lock(&rwlock->lock);
    rwlock->write_waiters++;
    
    /* Wait for all readers and writers */
    while (rwlock->readers > 0 || rwlock->writers > 0) {
        mutex_unlock(&rwlock->lock);
        thread_sleep_ms(1);
        mutex_lock(&rwlock->lock);
    }
    
    rwlock->write_waiters--;
    rwlock->writers++;
    mutex_unlock(&rwlock->lock);
    
    return 0;
}

int rwlock_write_unlock(rwlock_t *rwlock)
{
    if (!rwlock) return -1;
    
    mutex_lock(&rwlock->lock);
    rwlock->writers--;
    mutex_unlock(&rwlock->lock);
    
    return 0;
}

int rwlock_destroy(rwlock_t *rwlock)
{
    if (!rwlock) return -1;
    
    if (rwlock->readers > 0 || rwlock->writers > 0 || rwlock->write_waiters > 0) {
        return -1;
    }
    
    mutex_destroy(&rwlock->lock);
    memset(rwlock, 0, sizeof(rwlock_t));
    
    return 0;
}

/* Barrier operations */
int barrier_init(barrier_t *barrier, uint32_t count)
{
    if (!barrier || count == 0) return -1;
    
    memset(barrier, 0, sizeof(barrier_t));
    barrier->count = count;
    barrier->current = 0;
    mutex_init(&barrier->lock);
    
    return 0;
}

int barrier_wait(barrier_t *barrier)
{
    if (!barrier) return -1;
    
    mutex_lock(&barrier->lock);
    barrier->current++;
    
    int release = (barrier->current >= barrier->count);
    
    if (release) {
        barrier->current = 0;
    }
    
    mutex_unlock(&barrier->lock);
    
    /* Wait for other threads */
    while (barrier->current != 0 && !release) {
        thread_sleep_ms(1);
    }
    
    return release ? 1 : 0;
}

int barrier_destroy(barrier_t *barrier)
{
    if (!barrier) return -1;
    
    if (barrier->current > 0) {
        return -1;
    }
    
    mutex_destroy(&barrier->lock);
    memset(barrier, 0, sizeof(barrier_t));
    
    return 0;
}
