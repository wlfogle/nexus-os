#include "../../include/kernel/irq.h"
#include "../../include/kernel/idt.h"
#include "../../include/kernel/pic.h"
#include "../../include/kernel/timer.h"
#include "../../include/kernel/keyboard.h"

/* Assembly stubs defined in kernel/syscall/stubs.s */
extern void irq0(void);
extern void irq1(void);
extern void irq2(void);
extern void irq3(void);
extern void irq4(void);
extern void irq5(void);
extern void irq6(void);
extern void irq7(void);
extern void irq8(void);
extern void irq9(void);
extern void irq10(void);
extern void irq11(void);
extern void irq12(void);
extern void irq13(void);
extern void irq14(void);
extern void irq15(void);
extern void syscall_int80(void);

void irq_init(void)
{
    /* Wire IRQ 0-15 into the IDT at INT vectors 32-47.
       Use interrupt gates (0x0E) so the CPU clears IF on entry,
       preventing nested hardware interrupts while we handle one. */
    idt_set_handler(32, (uint32_t)irq0,  IDT_INTERRUPT_GATE);
    idt_set_handler(33, (uint32_t)irq1,  IDT_INTERRUPT_GATE);
    idt_set_handler(34, (uint32_t)irq2,  IDT_INTERRUPT_GATE);
    idt_set_handler(35, (uint32_t)irq3,  IDT_INTERRUPT_GATE);
    idt_set_handler(36, (uint32_t)irq4,  IDT_INTERRUPT_GATE);
    idt_set_handler(37, (uint32_t)irq5,  IDT_INTERRUPT_GATE);
    idt_set_handler(38, (uint32_t)irq6,  IDT_INTERRUPT_GATE);
    idt_set_handler(39, (uint32_t)irq7,  IDT_INTERRUPT_GATE);
    idt_set_handler(40, (uint32_t)irq8,  IDT_INTERRUPT_GATE);
    idt_set_handler(41, (uint32_t)irq9,  IDT_INTERRUPT_GATE);
    idt_set_handler(42, (uint32_t)irq10, IDT_INTERRUPT_GATE);
    idt_set_handler(43, (uint32_t)irq11, IDT_INTERRUPT_GATE);
    idt_set_handler(44, (uint32_t)irq12, IDT_INTERRUPT_GATE);
    idt_set_handler(45, (uint32_t)irq13, IDT_INTERRUPT_GATE);
    idt_set_handler(46, (uint32_t)irq14, IDT_INTERRUPT_GATE);
    idt_set_handler(47, (uint32_t)irq15, IDT_INTERRUPT_GATE);

    /* INT 0x80 syscall gate — kernel-privilege only for now */
    idt_set_handler(0x80, (uint32_t)syscall_int80, IDT_TRAP_GATE);

    /* All hardware interrupts are now wired — enable them */
    __asm__ volatile("sti");
}

void irq_dispatch(uint32_t irq_num)
{
    switch (irq_num) {
    case 0:  timer_interrupt();    break;
    case 1:  keyboard_interrupt(); break;
    /* IRQ 2-15: no handler yet; EOI is still sent below so the PIC
       does not get stuck.  Add case entries as drivers are added. */
    default: break;
    }

    /* Send End-Of-Interrupt to the PIC so it can accept the next IRQ */
    pic_send_eoi((uint8_t)irq_num);
}
