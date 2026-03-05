#include "../../include/kernel/idt.h"
#include "../../include/kernel/serial.h"

/* Global IDT */
static struct idt_entry idt[256];
static struct idt_ptr idt_ptr;

/* Exception names for debugging */
static const char *exception_names[32] = {
    "Division by Zero",
    "Debug",
    "Non-Maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved (15)",
    "Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
    "Reserved (22)",
    "Reserved (23)",
    "Reserved (24)",
    "Reserved (25)",
    "Reserved (26)",
    "Reserved (27)",
    "Hypervisor Injection Exception",
    "VMM Communication Exception",
    "Security Exception",
    "Reserved (31)"
};

/* Set an IDT entry */
static void idt_set_entry(int index, uint32_t handler, uint16_t selector, uint8_t type_attr)
{
    idt[index].offset_low = handler & 0xFFFF;
    idt[index].offset_high = (handler >> 16) & 0xFFFF;
    idt[index].selector = selector;
    idt[index].reserved = 0;
    idt[index].type_attr = type_attr;
}

/* Print hex value to serial */
static void print_hex(uint32_t num)
{
    int i;
    const char hex_chars[] = "0123456789ABCDEF";
    
    serial_putchar('0');
    serial_putchar('x');
    
    for (i = 28; i >= 0; i -= 4) {
        uint8_t digit = (num >> i) & 0xF;
        serial_putchar(hex_chars[digit]);
    }
}

/* Default exception handler */
void exception_handler(uint32_t exception_num, uint32_t error_code)
{
    serial_puts("\n=== EXCEPTION OCCURRED ===");
    serial_puts("\nException Number: ");
    print_hex(exception_num);
    serial_puts(" - ");
    
    if (exception_num < 32) {
        serial_puts(exception_names[exception_num]);
    } else {
        serial_puts("Unknown Exception");
    }
    serial_puts("\n");

    /* Page fault: Print faulting address from CR2 */
    if (exception_num == EXC_PAGE_FAULT) {
        uint32_t cr2;
        __asm__ volatile("mov %%cr2, %0" : "=r"(cr2));
        serial_puts("Faulting Address: ");
        print_hex(cr2);
        serial_puts("\n");
        
        /* Decode page fault error code */
        serial_puts("Error Code Flags: ");
        if (error_code & 1) serial_puts("PRESENT ");
        if (error_code & 2) serial_puts("WRITE ");
        if (error_code & 4) serial_puts("USER ");
        if (error_code & 8) serial_puts("RESERVED ");
        if (error_code & 16) serial_puts("FETCH ");
        serial_puts("\n");
    } else {
        /* For other exceptions, show raw error code */
        serial_puts("Error Code: ");
        print_hex(error_code);
        serial_puts("\n");
    }
    
    serial_puts("System halted.\n");
    
    /* Halt indefinitely */
    while (1) {
        __asm__ volatile("hlt");
    }
}

/* External ISR stub symbols */
extern void isr0(void);
extern void isr1(void);
extern void isr2(void);
extern void isr3(void);
extern void isr4(void);
extern void isr5(void);
extern void isr6(void);
extern void isr7(void);
extern void isr8(void);
extern void isr9(void);
extern void isr10(void);
extern void isr11(void);
extern void isr12(void);
extern void isr13(void);
extern void isr14(void);
extern void isr15(void);
extern void isr16(void);
extern void isr17(void);
extern void isr18(void);
extern void isr19(void);
extern void isr20(void);
extern void isr21(void);
extern void isr22(void);
extern void isr23(void);
extern void isr24(void);
extern void isr25(void);
extern void isr26(void);
extern void isr27(void);
extern void isr28(void);
extern void isr29(void);
extern void isr30(void);
extern void isr31(void);

static void *isr_handlers[32] = {
    isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7,
    isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15,
    isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23,
    isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31
};

/* Initialize IDT */
void idt_init(void)
{
    serial_puts("Initializing IDT...\n");

    /* Clear entire IDT */
    int i;
    for (i = 0; i < 256; i++) {
        idt_set_entry(i, 0, 0, 0);
    }

    /* Set up exception handlers for first 32 exceptions */
    for (i = 0; i < NUM_EXCEPTIONS; i++) {
        idt_set_entry(i, (uint32_t)isr_handlers[i],
                      0x08,  /* Kernel code selector */
                      IDT_PRESENT | IDT_PRIVILEGE_KERNEL | IDT_TRAP_GATE);
    }

    /* Set up IDT pointer structure */
    idt_ptr.limit = (sizeof(struct idt_entry) * 256) - 1;
    idt_ptr.base = (uint32_t)&idt[0];

    /* Load IDT into CPU */
    idt_load(&idt_ptr);

    serial_puts("IDT initialized and loaded (256 entries, 32 exceptions)\n");
}

/* Register a custom handler for a specific interrupt/exception */
void idt_set_handler(uint8_t num, uint32_t handler, uint8_t type_attr)
{
    if (num < 256) {
        idt_set_entry(num, handler, 0x08,
                      IDT_PRESENT | IDT_PRIVILEGE_KERNEL | type_attr);
    }
}

/* Disable interrupts */
void interrupts_disable(void)
{
    __asm__ volatile("cli");
}

/* Enable interrupts */
void interrupts_enable(void)
{
    __asm__ volatile("sti");
}
