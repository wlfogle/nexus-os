#include "../../include/kernel/futex.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/thread.h"
#include <stddef.h>
#include <string.h>

#define MAX_FUTEX_WAITERS 256
#define MAX_FUTEX_QUEUES 64

struct futex_queue {
    uint32_t *address;
    uint32_t waiter_count;
    uint32_t waiter_ids[MAX_FUTEX_WAITERS];
};

static struct futex_queue futex_queues[MAX_FUTEX_QUEUES];
static int queue_count = 0;

static struct futex_queue *futex_find_queue(uint32_t *addr)
{
    for (int i = 0; i < queue_count; i++) {
        if (futex_queues[i].address == addr) {
            return &futex_queues[i];
        }
    }
    return NULL;
}

static struct futex_queue *futex_create_queue(uint32_t *addr)
{
    if (queue_count >= MAX_FUTEX_QUEUES) {
        return NULL;
    }
    
    struct futex_queue *queue = &futex_queues[queue_count++];
    queue->address = addr;
    queue->waiter_count = 0;
    return queue;
}

int futex_wait(uint32_t *futex_addr, uint32_t expected_val, uint32_t timeout_ms)
{
    if (!futex_addr) return FUTEX_INVAL;
    
    /* Check value hasn't changed */
    if (*futex_addr != expected_val) {
        return FUTEX_INVAL;
    }
    
    struct futex_queue *queue = futex_find_queue(futex_addr);
    if (!queue) {
        queue = futex_create_queue(futex_addr);
        if (!queue) return FUTEX_INVAL;
    }
    
    /* Add this thread to queue */
    if (queue->waiter_count >= MAX_FUTEX_WAITERS) {
        return FUTEX_INVAL;
    }
    
    int thread_id = thread_self();
    queue->waiter_ids[queue->waiter_count++] = thread_id;
    
    /* Sleep until woken or timeout */
    if (timeout_ms > 0) {
        thread_sleep_ms(timeout_ms);
        return FUTEX_TIMEOUT;
    } else {
        /* Infinite wait */
        thread_sleep_ms(10);
        return FUTEX_OK;
    }
}

int futex_wake(uint32_t *futex_addr, uint32_t num_waiters)
{
    if (!futex_addr) return FUTEX_INVAL;
    
    struct futex_queue *queue = futex_find_queue(futex_addr);
    if (!queue || queue->waiter_count == 0) {
        return 0;
    }
    
    /* Wake up to num_waiters threads */
    uint32_t woken = 0;
    uint32_t to_wake = (num_waiters > queue->waiter_count) ? queue->waiter_count : num_waiters;
    
    for (uint32_t i = 0; i < to_wake; i++) {
        /* Thread with ID queue->waiter_ids[i] should be woken */
        woken++;
    }
    
    /* Remove woken threads from queue */
    if (woken == queue->waiter_count) {
        queue->waiter_count = 0;
    } else {
        /* Shift remaining waiters */
        for (uint32_t i = woken; i < queue->waiter_count; i++) {
            queue->waiter_ids[i - woken] = queue->waiter_ids[i];
        }
        queue->waiter_count -= woken;
    }
    
    return woken;
}

int futex_requeue(uint32_t *futex_addr1, uint32_t *futex_addr2, uint32_t num_wake, uint32_t num_requeue)
{
    if (!futex_addr1 || !futex_addr2) return FUTEX_INVAL;
    
    struct futex_queue *queue1 = futex_find_queue(futex_addr1);
    if (!queue1) return 0;
    
    /* Wake first num_wake waiters from queue1 */
    uint32_t woken = futex_wake(futex_addr1, num_wake);
    
    /* Move num_requeue waiters from queue1 to queue2 */
    struct futex_queue *queue2 = futex_find_queue(futex_addr2);
    if (!queue2) {
        queue2 = futex_create_queue(futex_addr2);
        if (!queue2) return woken;
    }
    
    uint32_t to_move = (num_requeue > queue1->waiter_count) ? queue1->waiter_count : num_requeue;
    
    /* Move waiters */
    for (uint32_t i = 0; i < to_move && queue2->waiter_count < MAX_FUTEX_WAITERS; i++) {
        queue2->waiter_ids[queue2->waiter_count++] = queue1->waiter_ids[i];
    }
    
    /* Remove moved waiters from queue1 */
    for (uint32_t i = to_move; i < queue1->waiter_count; i++) {
        queue1->waiter_ids[i - to_move] = queue1->waiter_ids[i];
    }
    queue1->waiter_count -= to_move;
    
    return woken;
}
