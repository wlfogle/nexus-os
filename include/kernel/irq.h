#ifndef KERNEL_IRQ_H
#define KERNEL_IRQ_H

#include "../libc/stdint.h"

/* Register IRQ0 (timer) and IRQ1 (keyboard) into the IDT at INT 32/33,
   register INT 0x80 for syscalls, then enable hardware interrupts (sti). */
void irq_init(void);

/* Called from the IRQ common assembly stub with the IRQ line number (0-15).
   Dispatches to the correct C handler, then sends PIC EOI. */
void irq_dispatch(uint32_t irq_num);

#endif /* KERNEL_IRQ_H */
