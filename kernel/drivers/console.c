#include "../../include/kernel/console.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/vga.h"
#include <stdarg.h>

void console_init(void)
{
    vga_init();
}

void console_putchar(char c)
{
    serial_putchar(c);
    vga_putchar(c);
}

void console_puts(const char *str)
{
    if (!str) return;
    while (*str) {
        console_putchar(*str++);
    }
}

void console_set_color(vga_color_t fg, vga_color_t bg)
{
    vga_set_color(fg, bg);
}

/* ------------------------------------------------------------------ */
/* console_printf — mirrors serial_printf format support               */
/* ------------------------------------------------------------------ */

static void cprint_decimal(int val)
{
    char buf[11];
    int  len = 0;

    if (val < 0) {
        console_putchar('-');
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
        console_putchar(buf[i]);
    }
}

static void cprint_udecimal(uint32_t val)
{
    char buf[11];
    int  len = 0;
    do {
        buf[len++] = (char)('0' + val % 10);
        val /= 10;
    } while (val > 0);
    for (int i = len - 1; i >= 0; i--) {
        console_putchar(buf[i]);
    }
}

static void cprint_hex(uint32_t val, int upper)
{
    const char *digits = upper ? "0123456789ABCDEF" : "0123456789abcdef";
    char buf[8];
    int  len = 0;
    do {
        buf[len++] = digits[val & 0xF];
        val >>= 4;
    } while (val > 0);
    for (int i = len - 1; i >= 0; i--) {
        console_putchar(buf[i]);
    }
}

void console_printf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);

    while (*fmt) {
        if (*fmt != '%') {
            console_putchar(*fmt++);
            continue;
        }
        fmt++; /* skip '%' */
        switch (*fmt) {
        case 'd':
        case 'i':
            cprint_decimal(va_arg(ap, int));
            break;
        case 'u':
            cprint_udecimal(va_arg(ap, uint32_t));
            break;
        case 'x':
            cprint_hex(va_arg(ap, uint32_t), 0);
            break;
        case 'X':
            cprint_hex(va_arg(ap, uint32_t), 1);
            break;
        case 's': {
            const char *s = va_arg(ap, const char *);
            console_puts(s ? s : "(null)");
            break;
        }
        case 'c':
            console_putchar((char)va_arg(ap, int));
            break;
        case '%':
            console_putchar('%');
            break;
        default:
            console_putchar('%');
            console_putchar(*fmt);
            break;
        }
        fmt++;
    }

    va_end(ap);
}
