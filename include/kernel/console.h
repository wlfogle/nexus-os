#ifndef KERNEL_CONSOLE_H
#define KERNEL_CONSOLE_H

#include "vga.h"

/* Initialize the console (calls vga_init) */
void console_init(void);

/* Write a single character to both serial and VGA */
void console_putchar(char c);

/* Write a null-terminated string to both serial and VGA */
void console_puts(const char *str);

/* Printf-style formatted output to both serial and VGA */
void console_printf(const char *fmt, ...);

/* Set the VGA text color for subsequent console output */
void console_set_color(vga_color_t fg, vga_color_t bg);

#endif /* KERNEL_CONSOLE_H */
