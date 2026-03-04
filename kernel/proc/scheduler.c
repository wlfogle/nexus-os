#include "../../include/kernel/scheduler.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

static int current_index = 0;

void scheduler_init(void)
{
    serial_puts("Scheduler initialized\n");
}

struct task *scheduler_pick_next(void)
{
    if (task_get_count() == 0) return NULL;
    
    int start = current_index;
    
    do {
        struct task *task = task_get(current_index);
        current_index = (current_index + 1) % task_get_count();
        
        if (task && (task->state == TASK_READY || task->state == TASK_RUNNING)) {
            return task;
        }
    } while (current_index != start);
    
    return NULL;
}

void scheduler_tick(void)
{
    struct task *current = task_get_current();
    
    if (current) {
        current->ticks_remaining--;
        if (current->ticks_remaining <= 0) {
            current->state = TASK_READY;
            current->ticks_remaining = 10;
            
            struct task *next = scheduler_pick_next();
            if (next && next != current) {
                task_set_current(next);
            }
        }
    }
}
