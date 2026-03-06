#ifndef KERNEL_VGA_H
#define KERNEL_VGA_H

#include "../libc/stdint.h"

/* VGA text mode dimensions */
#define VGA_WIDTH  80
#define VGA_HEIGHT 25

/* VGA text mode color codes */
typedef enum {
    VGA_BLACK         = 0,
    VGA_BLUE          = 1,
    VGA_GREEN         = 2,
    VGA_CYAN          = 3,
    VGA_RED           = 4,
    VGA_MAGENTA       = 5,
    VGA_BROWN         = 6,
    VGA_LIGHT_GREY    = 7,
    VGA_DARK_GREY     = 8,
    VGA_LIGHT_BLUE    = 9,
    VGA_LIGHT_GREEN   = 10,
    VGA_LIGHT_CYAN    = 11,
    VGA_LIGHT_RED     = 12,
    VGA_LIGHT_MAGENTA = 13,
    VGA_YELLOW        = 14,
    VGA_WHITE         = 15,
} vga_color_t;

/* Initialize VGA text mode (clears screen, sets default colors) */
void vga_init(void);

/* Clear the entire screen */
void vga_clear(void);

/* Set foreground and background color for subsequent output */
void vga_set_color(vga_color_t fg, vga_color_t bg);

/* Write a single character at the current cursor position */
void vga_putchar(char c);

/* Write a null-terminated string */
void vga_puts(const char *str);

/* Printf-style formatted output; supports %d %u %x %X %s %c %% */
void vga_printf(const char *fmt, ...);

/* Move the hardware cursor to column x, row y (0-indexed) */
void vga_set_cursor(int x, int y);

/* Read the current cursor position into *x and *y */
void vga_get_cursor(int *x, int *y);

#endif /* KERNEL_VGA_H */
