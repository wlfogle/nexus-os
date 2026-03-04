#ifndef KERNEL_HEAP_H
#define KERNEL_HEAP_H

#include "../libc/stdint.h"

void heap_init(void);
void *kmalloc(uint32_t size);
void kfree(void *ptr);

#endif
