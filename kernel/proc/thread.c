#include "../../include/kernel/thread.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/heap.h"
#include <string.h>

/* Global thread table */
static struct thread *thread_table[256];
static uint32_t next_thread_id = 1;

/* Initialize thread subsystem */
void thread_init(void)
{
    memset(thread_table, 0, sizeof(thread_table));
    serial_puts("[thread] Thread manager initialized\n");
}

/* Create a new thread within a task */
int thread_create(struct task *task, void (*entry)(void *), void *arg)
{
    if (!task || !entry) return -1;

    /* Find free thread slot */
    int thread_idx = -1;
    for (int i = 0; i < 256; i++) {
        if (!thread_table[i]) {
            thread_idx = i;
            break;
        }
    }

    if (thread_idx < 0) {
        serial_puts("[thread] Thread table full\n");
        return -1;
    }

    /* Allocate thread control block */
    struct thread *thread = (struct thread *)kmalloc(sizeof(struct thread));
    if (!thread) return -1;

    memset(thread, 0, sizeof(struct thread));

    /* Initialize thread structure */
    thread->id = next_thread_id++;
    thread->task_id = task->id;
    thread->state = THREAD_READY;
    thread->entry_point = entry;
    thread->arg = arg;
    thread->refcount = 1;

    /* Allocate stack */
    thread->stack_size = THREAD_STACK_SIZE;
    thread->stack = (uint32_t *)kmalloc(THREAD_STACK_SIZE);
    if (!thread->stack) {
        kfree(thread);
        return -1;
    }

    thread->stack_base = (uint32_t)thread->stack;

    /* Initialize stack pointer to top of stack (grows downward) */
    uint32_t *sp = (uint32_t *)(thread->stack_base + THREAD_STACK_SIZE - 4);
    
    /* Set up initial stack frame */
    *sp = (uint32_t)arg;                    /* arg parameter */
    thread->regs.esp = (uint32_t)(sp);
    thread->regs.eip = (uint32_t)entry;
    thread->regs.ebp = thread->regs.esp;
    thread->regs.eflags = 0x202;            /* IF flag enabled, IOPL=0 */

    /* Initialize TLS */
    thread->tls.thread_id = thread->id;
    thread->tls.errno_val = 0;

    /* Register thread */
    thread_table[thread_idx] = thread;

    serial_printf("[thread] Created thread %d for task %d\n", thread->id, task->id);
    return thread->id;
}

/* Join a thread (wait for it to exit) */
int thread_join(int thread_id, int *exit_code)
{
    struct thread *thread = thread_get(thread_id);
    if (!thread) return -1;

    /* Wait for thread to exit */
    while (thread->state != THREAD_EXITED) {
        /* Busy wait - in production would use condition variables */
        for (volatile int i = 0; i < 100000; i++);
    }

    if (exit_code) {
        *exit_code = thread->exit_code;
    }

    return 0;
}

/* Exit current thread */
int thread_exit(int exit_code)
{
    int thread_id = thread_self();
    struct thread *thread = thread_get(thread_id);
    
    if (!thread) return -1;

    thread->exit_code = exit_code;
    thread->state = THREAD_EXITED;

    serial_printf("[thread] Thread %d exited with code %d\n", thread->id, exit_code);

    return 0;
}

/* Get current thread ID */
int thread_self(void)
{
    /* In a full implementation, this would read from FS segment
     * For now, return 1 (main thread ID) */
    return 1;
}

/* Get thread by ID */
int thread_get_id(int thread_id)
{
    struct thread *thread = thread_get(thread_id);
    return thread ? (int)thread->id : -1;
}

/* Get thread control block */
struct thread *thread_get(int thread_id)
{
    for (int i = 0; i < 256; i++) {
        if (thread_table[i] && (int)thread_table[i]->id == thread_id) {
            return thread_table[i];
        }
    }
    return NULL;
}

/* Sleep thread for milliseconds */
void thread_sleep_ms(uint32_t ms)
{
    struct thread *thread = thread_get(thread_self());
    if (!thread) return;

    thread->state = THREAD_BLOCKED;
    
    /* Convert ms to timer ticks and block */
    (void)((ms + 9) / 10);  /* ticks - used for future timer-based sleep */
    
    /* Busy sleep - in production would use timer */
    uint32_t iterations = ms * 1000;
    for (volatile uint32_t i = 0; i < iterations; i++);

    thread->state = THREAD_READY;
}

/* Get thread-local storage value */
void *thread_get_tls(int slot)
{
    if (slot < 0 || slot >= 4) return NULL;

    struct thread *thread = thread_get(thread_self());
    if (!thread) return NULL;

    return thread->tls.user_data[slot];
}

/* Set thread-local storage value */
int thread_set_tls(int slot, void *value)
{
    if (slot < 0 || slot >= 4) return -1;

    struct thread *thread = thread_get(thread_self());
    if (!thread) return -1;

    thread->tls.user_data[slot] = value;
    return 0;
}

/* Get thread count for a task */
int thread_count_for_task(uint32_t task_id)
{
    int count = 0;
    for (int i = 0; i < 256; i++) {
        if (thread_table[i] && thread_table[i]->task_id == task_id) {
            count++;
        }
    }
    return count;
}
