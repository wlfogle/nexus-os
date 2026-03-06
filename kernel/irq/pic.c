#include "../../include/kernel/pic.h"
#include "../../include/kernel/serial.h"

#define PIC1 0x20
#define PIC2 0xA0
#define PIC1_CMD PIC1
#define PIC1_DATA (PIC1+1)
#define PIC2_CMD PIC2
#define PIC2_DATA (PIC2+1)

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

void pic_init(void)
{
    outb(PIC1_CMD, 0x11);
    outb(PIC2_CMD, 0x11);
    outb(PIC1_DATA, 0x20);
    outb(PIC2_DATA, 0x28);
    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);
    outb(PIC1_DATA, 0xFF);
    outb(PIC2_DATA, 0xFF);
    serial_puts("PIC initialized\n");
}

void pic_send_eoi(uint8_t irq)
{
    if (irq >= 8) outb(PIC2_CMD, 0x20);
    outb(PIC1_CMD, 0x20);
}

void pic_disable_irq(uint8_t irq)
{
    uint16_t port = (irq < 8) ? PIC1_DATA : PIC2_DATA;
    uint8_t val = inb(port) | (1 << (irq % 8));
    outb(port, val);
}

void pic_enable_irq(uint8_t irq)
{
    uint16_t port = (irq < 8) ? PIC1_DATA : PIC2_DATA;
    uint8_t val = inb(port) & ~(1 << (irq % 8));
    outb(port, val);
}
