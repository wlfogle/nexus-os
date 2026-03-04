#!/bin/bash
# Create all Phase 1 files efficiently

mkdir -p kernel/arch kernel/irq kernel/mm

# Create paging.h
cat > include/kernel/paging.h << 'HEADER'
#ifndef KERNEL_PAGING_H
#define KERNEL_PAGING_H

#include "../libc/stdint.h"
#include "pmem.h"

#define PAGE_TABLE_ENTRIES 1024
#define PAGE_DIR_ENTRIES 1024

void paging_init(void);
void enable_paging(void);
uint32_t *get_page_directory(void);
void map_page(uint32_t virt, uint32_t phys, int flags);
void unmap_page(uint32_t virt);

#endif
HEADER

# Create paging.c
cat > kernel/mm/paging.c << 'PAGING'
#include "../../include/kernel/paging.h"
#include "../../include/kernel/serial.h"

#define PAGE_PRESENT 0x001
#define PAGE_WRITABLE 0x002
#define PAGE_USER 0x004

static uint32_t page_directory[PAGE_DIR_ENTRIES] __attribute__((aligned(4096)));
static uint32_t page_tables[4][PAGE_TABLE_ENTRIES] __attribute__((aligned(4096)));

void paging_init(void)
{
    for (int i = 0; i < PAGE_DIR_ENTRIES; i++) {
        page_directory[i] = 0;
    }
    
    for (int t = 0; t < 4; t++) {
        for (int i = 0; i < PAGE_TABLE_ENTRIES; i++) {
            page_tables[t][i] = 0;
        }
    }
    
    for (int i = 0; i < 4; i++) {
        page_directory[i] = (uint32_t)&page_tables[i] | PAGE_PRESENT | PAGE_WRITABLE;
    }
    
    for (int i = 0; i < 1024; i++) {
        page_tables[0][i] = (i * PAGE_SIZE) | PAGE_PRESENT | PAGE_WRITABLE;
    }
    
    serial_puts("Paging structures initialized\n");
}

void enable_paging(void)
{
    __asm__ volatile("mov %0, %%cr3" : : "r"((uint32_t)page_directory));
    uint32_t cr0;
    __asm__ volatile("mov %%cr0, %0" : "=r"(cr0));
    cr0 |= 0x80000000;
    __asm__ volatile("mov %0, %%cr0" : : "r"(cr0));
    serial_puts("Paging enabled\n");
}

uint32_t *get_page_directory(void)
{
    return page_directory;
}

void map_page(uint32_t virt, uint32_t phys, int flags)
{
    uint32_t dir_idx = virt / (PAGE_TABLE_ENTRIES * PAGE_SIZE);
    uint32_t table_idx = (virt / PAGE_SIZE) % PAGE_TABLE_ENTRIES;
    if (dir_idx < 4 && table_idx < PAGE_TABLE_ENTRIES) {
        page_tables[dir_idx][table_idx] = phys | flags | PAGE_PRESENT;
    }
}

void unmap_page(uint32_t virt)
{
    uint32_t dir_idx = virt / (PAGE_TABLE_ENTRIES * PAGE_SIZE);
    uint32_t table_idx = (virt / PAGE_SIZE) % PAGE_TABLE_ENTRIES;
    if (dir_idx < 4 && table_idx < PAGE_TABLE_ENTRIES) {
        page_tables[dir_idx][table_idx] = 0;
    }
}
PAGING

# Create PIC.h
cat > include/kernel/pic.h << 'PICH'
#ifndef KERNEL_PIC_H
#define KERNEL_PIC_H

#include "../libc/stdint.h"

void pic_init(void);
void pic_send_eoi(uint8_t irq);
void pic_disable_irq(uint8_t irq);
void pic_enable_irq(uint8_t irq);

#endif
PICH

# Create pic.c
cat > kernel/irq/pic.c << 'PICC'
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
PICC

echo "Phase 1 files created successfully"
