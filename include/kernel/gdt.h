#ifndef KERNEL_GDT_H
#define KERNEL_GDT_H

#include "../libc/stdint.h"

/* GDT Entry - 8 bytes */
struct gdt_entry {
    uint16_t limit_low;      /* Segment limit (bits 0-15) */
    uint16_t base_low;       /* Base address (bits 0-15) */
    uint8_t base_mid;        /* Base address (bits 16-23) */
    uint8_t access;          /* Access byte (present, privilege, type) */
    uint8_t granularity;     /* Granularity and limit (bits 16-19) */
    uint8_t base_high;       /* Base address (bits 24-31) */
} __attribute__((packed));

/* GDT Pointer - passed to LGDT instruction */
struct gdt_ptr {
    uint16_t limit;          /* Size of GDT - 1 */
    uint32_t base;           /* Base address of GDT */
} __attribute__((packed));

/* GDT segment descriptors */
#define GDT_ENTRIES 5

/* Segment selector indices */
#define KERNEL_CODE_SELECTOR 0x08  /* Selector for kernel code */
#define KERNEL_DATA_SELECTOR 0x10  /* Selector for kernel data */
#define USER_CODE_SELECTOR   0x1B  /* Selector for user code (privilege level 3) */
#define USER_DATA_SELECTOR   0x23  /* Selector for user data (privilege level 3) */

/* Access byte flags */
#define GDT_PRESENT          0x80  /* Segment present */
#define GDT_PRIVILEGE_KERNEL 0x00  /* Privilege level 0 (kernel) */
#define GDT_PRIVILEGE_USER   0x60  /* Privilege level 3 (user) */
#define GDT_CODE             0x1A  /* Code segment (S=1, executable, readable) */
#define GDT_DATA             0x12  /* Data segment (S=1, writable) */
#define GDT_TSS              0x09  /* Task State Segment */

/* Initialize GDT and load it into CPU */
void gdt_init(void);

/* Load GDT into CPU (in assembly) */
extern void gdt_load(struct gdt_ptr *ptr);

#endif /* KERNEL_GDT_H */
