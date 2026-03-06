#include "../../include/kernel/pic.h"
#include "../../include/kernel/scheduler.h"
#include "../../include/kernel/serial.h"

#define PIT_CHANNEL0 0x40   /* PIT channel 0 data port       */
#define PIT_CMD      0x43   /* PIT mode/command register     */
#define PIT_HZ       100    /* Desired timer frequency in Hz */

static volatile int ticks = 0;

static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile("outb %0, %1" : : "a"(val), "Nd"(port));
}

void timer_init(void)
{
    /*
     * Program PIT channel 0 in mode 3 (square wave), binary counting:
     *   Command byte 0x36 = 0011 0110
     *     bits 7-6: 00  = channel 0
     *     bits 5-4: 11  = access lobyte then hibyte
     *     bits 3-1: 011 = mode 3 (square wave)
     *     bit  0:   0   = binary (not BCD)
     *
     * Divisor = 1193182 / PIT_HZ  (PIT base clock = 1.193182 MHz)
     */
    uint32_t divisor = 1193182 / PIT_HZ;

    outb(PIT_CMD,     0x36);
    outb(PIT_CHANNEL0, (uint8_t)( divisor       & 0xFF));  /* LSB */
    outb(PIT_CHANNEL0, (uint8_t)((divisor >> 8) & 0xFF));  /* MSB */

    pic_enable_irq(0);
    serial_puts("[OK] PIT timer (100 Hz, IRQ0)\n");
}

void timer_interrupt(void)
{
    ticks++;
    scheduler_tick();
    /* EOI is sent by irq_dispatch() after this function returns */
}

int timer_get_ticks(void)
{
    return ticks;
}
