#include "../include/kernel/serial.h"

void kernel_main(struct multiboot_info *mbi, uint32_t magic)
{
    serial_init();
    serial_puts("Kernel started!\n");
    serial_puts("If you see this, boot sequence works.\n");
    
    while(1) {
        __asm__ volatile("hlt");
    }
}
