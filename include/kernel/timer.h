#ifndef KERNEL_TIMER_H
#define KERNEL_TIMER_H

void timer_init(void);
void timer_interrupt(void);
int timer_get_ticks(void);

#endif
