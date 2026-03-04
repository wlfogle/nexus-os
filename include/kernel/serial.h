#ifndef KERNEL_SERIAL_H
#define KERNEL_SERIAL_H

#include "../libc/stdint.h"

/* Serial port I/O */
#define COM1_PORT 0x3F8

/* Initialize serial port for output */
void serial_init(void);

/* Write a single character to serial port */
void serial_putchar(char c);

/* Write a string to serial port */
void serial_puts(const char *str);

/* Print formatted output to serial (printf-like) */
void serial_printf(const char *fmt, ...);

#endif /* KERNEL_SERIAL_H */
