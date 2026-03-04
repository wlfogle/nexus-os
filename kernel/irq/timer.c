#include "../../include/kernel/pic.h"
#include "../../include/kernel/scheduler.h"
#include "../../include/kernel/serial.h"

#define PIT_CHANNEL0 0x40
#define PIT_CMD 0x43
#define PIT_HZ 100

static int ticks = 0;

void timer_init(void)
{
    uint32_t divisor = 1193182 / PIT_HZ;
    
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)(divisor & 0xFF)), "Nd"(PIT_CMD));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)0x36), "Nd"(PIT_CMD));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)((divisor >> 8) & 0xFF)), "Nd"(PIT_CHANNEL0));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)(divisor & 0xFF)), "Nd"(PIT_CHANNEL0));
    
    pic_enable_irq(0);
    serial_puts("Timer initialized (100 Hz)\n");
}

void timer_interrupt(void)
{
    ticks++;
    scheduler_tick();
    pic_send_eoi(0);
}

int timer_get_ticks(void)
{
    return ticks;
}
