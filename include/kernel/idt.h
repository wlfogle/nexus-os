#ifndef KERNEL_IDT_H
#define KERNEL_IDT_H

#include "../libc/stdint.h"

/* IDT Entry - 8 bytes */
struct idt_entry {
    uint16_t offset_low;     /* Handler offset (bits 0-15) */
    uint16_t selector;       /* Code segment selector */
    uint8_t reserved;        /* Reserved (zero) */
    uint8_t type_attr;       /* Type and attributes */
    uint16_t offset_high;    /* Handler offset (bits 16-31) */
} __attribute__((packed));

/* IDT Pointer - passed to LIDT instruction */
struct idt_ptr {
    uint16_t limit;          /* Size of IDT - 1 */
    uint32_t base;           /* Base address of IDT */
} __attribute__((packed));

/* IDT Attributes */
#define IDT_PRESENT          0x80  /* Segment present */
#define IDT_PRIVILEGE_KERNEL 0x00  /* Privilege level 0 */
#define IDT_PRIVILEGE_USER   0x60  /* Privilege level 3 */
#define IDT_INTERRUPT_GATE   0x0E  /* Interrupt gate (32-bit) */
#define IDT_TRAP_GATE        0x0F  /* Trap gate (32-bit) */

/* Number of exception handlers */
#define NUM_EXCEPTIONS 32

/* Exception numbers */
#define EXC_DIVIDE_BY_ZERO       0
#define EXC_DEBUG                1
#define EXC_NMI                  2
#define EXC_BREAKPOINT           3
#define EXC_OVERFLOW             4
#define EXC_BOUND_RANGE          5
#define EXC_INVALID_OPCODE       6
#define EXC_DEVICE_NOT_AVAIL     7
#define EXC_DOUBLE_FAULT         8
#define EXC_COPROC_SEG_OVERRUN   9
#define EXC_INVALID_TSS          10
#define EXC_SEGMENT_NOT_PRESENT  11
#define EXC_STACK_SEGMENT_FAULT  12
#define EXC_GENERAL_PROTECTION   13
#define EXC_PAGE_FAULT           14
#define EXC_RESERVED             15
#define EXC_FPU_EXCEPTION        16
#define EXC_ALIGNMENT_CHECK      17
#define EXC_MACHINE_CHECK        18
#define EXC_SIMD_FP_EXCEPTION    19

/* Initialize IDT and load it */
void idt_init(void);

/* Load IDT into CPU (in assembly) */
extern void idt_load(struct idt_ptr *ptr);

/* Register a handler for a specific interrupt/exception */
void idt_set_handler(uint8_t num, uint32_t handler, uint8_t type_attr);

/* Default exception handler */
void exception_handler(uint32_t exception_num, uint32_t error_code);

#endif /* KERNEL_IDT_H */
