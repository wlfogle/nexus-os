#include "../include/kernel/serial.h"

/* Serial port base address */
#define SERIAL_BASE COM1_PORT

/* UART register offsets */
#define THR 0    /* Transmitter Holding Register */
#define RBR 0    /* Receiver Buffer Register */
#define DLL 0    /* Divisor Latch Low */
#define DLH 1    /* Divisor Latch High */
#define IER 1    /* Interrupt Enable Register */
#define IIR 2    /* Interrupt Identification Register */
#define FCR 2    /* FIFO Control Register */
#define LCR 3    /* Line Control Register */
#define MCR 4    /* Modem Control Register */
#define LSR 5    /* Line Status Register */
#define MSR 6    /* Modem Status Register */

/* Line Status Register bits */
#define LSR_THRE 0x20  /* Transmitter Holding Register Empty */

/* Inline assembly for I/O port access */
static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile("outb %0, %1" : : "a"(val), "Nd"(port));
}

static inline uint8_t inb(uint16_t port)
{
    uint8_t ret;
    __asm__ volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

/* Check if transmitter is ready */
static int serial_ready(void)
{
    return (inb(SERIAL_BASE + LSR) & LSR_THRE) != 0;
}

/* Initialize serial port */
void serial_init(void)
{
    /* Set DLAB (Divisor Latch Access Bit) to access baud rate divisor */
    outb(SERIAL_BASE + LCR, 0x80);

    /* Set baud rate to 115200 (divisor = 1 for 115200) */
    outb(SERIAL_BASE + DLL, 1);
    outb(SERIAL_BASE + DLH, 0);

    /* Clear DLAB and set 8N1 (8 data bits, no parity, 1 stop bit) */
    outb(SERIAL_BASE + LCR, 0x03);

    /* Enable FIFO with 14-byte threshold, clear buffers */
    outb(SERIAL_BASE + FCR, 0xC7);

    /* Enable interrupts (optional - not used in basic version) */
    outb(SERIAL_BASE + IER, 0x00);
}

/* Write single character to serial */
void serial_putchar(char c)
{
    /* Wait until transmitter is ready */
    while (!serial_ready())
        __asm__ volatile("pause");

    /* Send character */
    outb(SERIAL_BASE + THR, (unsigned char)c);
}

/* Write string to serial */
void serial_puts(const char *str)
{
    if (!str)
        return;

    while (*str) {
        if (*str == '\n')
            serial_putchar('\r');
        serial_putchar(*str++);
    }
}

/* Simple printf-like formatter */
void serial_printf(const char *fmt, ...)
{
    /* TODO: Implement proper printf parsing */
    /* For now, just output the format string */
    serial_puts(fmt);
}
