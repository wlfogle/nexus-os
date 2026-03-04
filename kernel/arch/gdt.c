#include "../../include/kernel/gdt.h"
#include "../../include/kernel/serial.h"

/* Global GDT */
static struct gdt_entry gdt[GDT_ENTRIES];
static struct gdt_ptr gdt_ptr;

/* Set a GDT entry */
static void gdt_set_entry(int index, uint32_t base, uint32_t limit, uint8_t access, uint8_t gran)
{
    gdt[index].base_low = (base & 0xFFFF);
    gdt[index].base_mid = (base >> 16) & 0xFF;
    gdt[index].base_high = (base >> 24) & 0xFF;

    gdt[index].limit_low = (limit & 0xFFFF);
    gdt[index].granularity = (limit >> 16) & 0x0F;

    gdt[index].access = access;
    gdt[index].granularity |= gran & 0xF0;
}

/* Initialize GDT */
void gdt_init(void)
{
    serial_puts("Initializing GDT...\n");

    /* Null descriptor (required) */
    gdt_set_entry(0, 0, 0, 0, 0);

    /* Kernel code segment (4GB, present, privilege 0, executable, readable) */
    gdt_set_entry(1, 0, 0xFFFFFFFF,
                  GDT_PRESENT | GDT_PRIVILEGE_KERNEL | GDT_CODE,
                  0xC0);  /* Granularity: 4KB pages, 32-bit */

    /* Kernel data segment (4GB, present, privilege 0, writable) */
    gdt_set_entry(2, 0, 0xFFFFFFFF,
                  GDT_PRESENT | GDT_PRIVILEGE_KERNEL | GDT_DATA,
                  0xC0);  /* Granularity: 4KB pages, 32-bit */

    /* User code segment (4GB, present, privilege 3, executable, readable) */
    gdt_set_entry(3, 0, 0xFFFFFFFF,
                  GDT_PRESENT | GDT_PRIVILEGE_USER | GDT_CODE,
                  0xC0);

    /* User data segment (4GB, present, privilege 3, writable) */
    gdt_set_entry(4, 0, 0xFFFFFFFF,
                  GDT_PRESENT | GDT_PRIVILEGE_USER | GDT_DATA,
                  0xC0);

    /* Set up the GDT pointer */
    gdt_ptr.limit = (sizeof(struct gdt_entry) * GDT_ENTRIES) - 1;
    gdt_ptr.base = (uint32_t)&gdt[0];

    /* Load the GDT into the CPU */
    gdt_load(&gdt_ptr);

    serial_puts("GDT initialized and loaded\n");
}
