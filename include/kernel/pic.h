#ifndef KERNEL_PIC_H
#define KERNEL_PIC_H

#include "../libc/stdint.h"

void pic_init(void);
void pic_send_eoi(uint8_t irq);
void pic_disable_irq(uint8_t irq);
void pic_enable_irq(uint8_t irq);

#endif
