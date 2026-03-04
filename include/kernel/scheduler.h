#ifndef KERNEL_SCHEDULER_H
#define KERNEL_SCHEDULER_H

#include "task.h"

void scheduler_init(void);
struct task *scheduler_pick_next(void);
void scheduler_tick(void);

#endif
