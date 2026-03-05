#include "../../include/kernel/vga.h"
#include <stdarg.h>

/* VGA text buffer base address (physical) */
#define VGA_BUFFER    ((volatile uint16_t *)0xB8000)

/* CRTC register ports for hardware cursor */
#define VGA_CRTC_INDEX 0x3D4
#define VGA_CRTC_DATA  0x3D5

static int     cursor_x     = 0;
static int     cursor_y     = 0;
static uint8_t cur_color    = 0;   /* (bg << 4) | fg */

/* ------------------------------------------------------------------ */
/* I/O helpers                                                         */
/* ------------------------------------------------------------------ */

static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile("outb %0, %1" : : "a"(val), "Nd"(port));
}

/* ------------------------------------------------------------------ */
/* Internal helpers                                                    */
/* ------------------------------------------------------------------ */

static inline uint16_t vga_entry(char c, uint8_t color)
{
    return (uint16_t)(unsigned char)c | ((uint16_t)color << 8);
}

static void vga_update_hw_cursor(void)
{
    uint16_t pos = (uint16_t)(cursor_y * VGA_WIDTH + cursor_x);
    outb(VGA_CRTC_INDEX, 14);
    outb(VGA_CRTC_DATA,  (uint8_t)((pos >> 8) & 0xFF));
    outb(VGA_CRTC_INDEX, 15);
    outb(VGA_CRTC_DATA,  (uint8_t)(pos & 0xFF));
}

static void vga_scroll_up(void)
{
    int x, y;

    /* Move every row one line up */
    for (y = 0; y < VGA_HEIGHT - 1; y++) {
        for (x = 0; x < VGA_WIDTH; x++) {
            VGA_BUFFER[y * VGA_WIDTH + x] =
                VGA_BUFFER[(y + 1) * VGA_WIDTH + x];
        }
    }

    /* Clear the last row */
    for (x = 0; x < VGA_WIDTH; x++) {
        VGA_BUFFER[(VGA_HEIGHT - 1) * VGA_WIDTH + x] =
            vga_entry(' ', cur_color);
    }

    cursor_y = VGA_HEIGHT - 1;
}

/* ------------------------------------------------------------------ */
/* Public API                                                          */
/* ------------------------------------------------------------------ */

void vga_init(void)
{
    cur_color = (uint8_t)((VGA_BLACK << 4) | VGA_LIGHT_GREY);
    vga_clear();
}

void vga_clear(void)
{
    int x, y;
    for (y = 0; y < VGA_HEIGHT; y++) {
        for (x = 0; x < VGA_WIDTH; x++) {
            VGA_BUFFER[y * VGA_WIDTH + x] = vga_entry(' ', cur_color);
        }
    }
    cursor_x = 0;
    cursor_y = 0;
    vga_update_hw_cursor();
}

void vga_set_color(vga_color_t fg, vga_color_t bg)
{
    cur_color = (uint8_t)(((uint8_t)bg << 4) | (uint8_t)fg);
}

void vga_putchar(char c)
{
    switch (c) {
    case '\n':
        cursor_x = 0;
        cursor_y++;
        break;
    case '\r':
        cursor_x = 0;
        break;
    case '\b':
        if (cursor_x > 0) {
            cursor_x--;
            VGA_BUFFER[cursor_y * VGA_WIDTH + cursor_x] =
                vga_entry(' ', cur_color);
        }
        break;
    case '\t':
        cursor_x = (cursor_x + 8) & ~7;
        break;
    default:
        VGA_BUFFER[cursor_y * VGA_WIDTH + cursor_x] =
            vga_entry(c, cur_color);
        cursor_x++;
        break;
    }

    if (cursor_x >= VGA_WIDTH) {
        cursor_x = 0;
        cursor_y++;
    }
    if (cursor_y >= VGA_HEIGHT) {
        vga_scroll_up();
    }

    vga_update_hw_cursor();
}

void vga_puts(const char *str)
{
    if (!str) return;
    while (*str) {
        vga_putchar(*str++);
    }
}

void vga_set_cursor(int x, int y)
{
    if (x >= 0 && x < VGA_WIDTH && y >= 0 && y < VGA_HEIGHT) {
        cursor_x = x;
        cursor_y = y;
        vga_update_hw_cursor();
    }
}

void vga_get_cursor(int *x, int *y)
{
    if (x) *x = cursor_x;
    if (y) *y = cursor_y;
}

/* ------------------------------------------------------------------ */
/* vga_printf — supports %d %i %u %x %X %s %c %%                     */
/* ------------------------------------------------------------------ */

static void print_decimal(int val)
{
    char buf[11];
    int  len = 0;

    if (val < 0) {
        vga_putchar('-');
        /* Handle INT_MIN safely by working in unsigned */
        uint32_t uval = (uint32_t)(-(val + 1)) + 1U;
        do {
            buf[len++] = (char)('0' + uval % 10);
            uval /= 10;
        } while (uval > 0);
    } else {
        uint32_t uval = (uint32_t)val;
        do {
            buf[len++] = (char)('0' + uval % 10);
            uval /= 10;
        } while (uval > 0);
    }

    for (int i = len - 1; i >= 0; i--) {
        vga_putchar(buf[i]);
    }
}

static void print_udecimal(uint32_t val)
{
    char buf[11];
    int  len = 0;
    do {
        buf[len++] = (char)('0' + val % 10);
        val /= 10;
    } while (val > 0);
    for (int i = len - 1; i >= 0; i--) {
        vga_putchar(buf[i]);
    }
}

static void print_hex(uint32_t val, int upper)
{
    const char *digits = upper ? "0123456789ABCDEF" : "0123456789abcdef";
    char buf[8];
    int  len = 0;
    do {
        buf[len++] = digits[val & 0xF];
        val >>= 4;
    } while (val > 0);
    for (int i = len - 1; i >= 0; i--) {
        vga_putchar(buf[i]);
    }
}

void vga_printf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);

    while (*fmt) {
        if (*fmt != '%') {
            vga_putchar(*fmt++);
            continue;
        }
        fmt++; /* skip '%' */
        switch (*fmt) {
        case 'd':
        case 'i':
            print_decimal(va_arg(ap, int));
            break;
        case 'u':
            print_udecimal(va_arg(ap, uint32_t));
            break;
        case 'x':
            print_hex(va_arg(ap, uint32_t), 0);
            break;
        case 'X':
            print_hex(va_arg(ap, uint32_t), 1);
            break;
        case 's': {
            const char *s = va_arg(ap, const char *);
            vga_puts(s ? s : "(null)");
            break;
        }
        case 'c':
            vga_putchar((char)va_arg(ap, int));
            break;
        case '%':
            vga_putchar('%');
            break;
        default:
            vga_putchar('%');
            vga_putchar(*fmt);
            break;
        }
        fmt++;
    }

    va_end(ap);
}
