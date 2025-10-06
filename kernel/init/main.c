#include <stdint.h>
#include <stddef.h>
#include "../include/pkg_compat.h"

// VGA text mode constants
#define VGA_WIDTH 80
#define VGA_HEIGHT 25
#define VGA_MEMORY 0xB8000

// VGA colors
enum vga_color {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_BLUE = 1,
    VGA_COLOR_GREEN = 2,
    VGA_COLOR_CYAN = 3,
    VGA_COLOR_RED = 4,
    VGA_COLOR_MAGENTA = 5,
    VGA_COLOR_BROWN = 6,
    VGA_COLOR_LIGHT_GREY = 7,
    VGA_COLOR_DARK_GREY = 8,
    VGA_COLOR_LIGHT_BLUE = 9,
    VGA_COLOR_LIGHT_GREEN = 10,
    VGA_COLOR_LIGHT_CYAN = 11,
    VGA_COLOR_LIGHT_RED = 12,
    VGA_COLOR_LIGHT_MAGENTA = 13,
    VGA_COLOR_LIGHT_BROWN = 14,
    VGA_COLOR_WHITE = 15,
};

static uint8_t vga_entry_color(enum vga_color fg, enum vga_color bg) {
    return fg | bg << 4;
}

static uint16_t vga_entry(unsigned char uc, uint8_t color) {
    return (uint16_t) uc | (uint16_t) color << 8;
}

// Terminal state
static size_t terminal_row;
static size_t terminal_column;
static uint8_t terminal_color;
static uint16_t* terminal_buffer;

void terminal_initialize(void) {
    terminal_row = 0;
    terminal_column = 0;
    terminal_color = vga_entry_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_buffer = (uint16_t*) VGA_MEMORY;
    for (size_t y = 0; y < VGA_HEIGHT; y++) {
        for (size_t x = 0; x < VGA_WIDTH; x++) {
            const size_t index = y * VGA_WIDTH + x;
            terminal_buffer[index] = vga_entry(' ', terminal_color);
        }
    }
}

void terminal_setcolor(uint8_t color) {
    terminal_color = color;
}

void terminal_putentryat(char c, uint8_t color, size_t x, size_t y) {
    const size_t index = y * VGA_WIDTH + x;
    terminal_buffer[index] = vga_entry(c, color);
}

void terminal_scroll(void) {
    for (size_t y = 0; y < VGA_HEIGHT - 1; y++) {
        for (size_t x = 0; x < VGA_WIDTH; x++) {
            terminal_buffer[y * VGA_WIDTH + x] = terminal_buffer[(y + 1) * VGA_WIDTH + x];
        }
    }
    
    // Clear last line
    for (size_t x = 0; x < VGA_WIDTH; x++) {
        terminal_buffer[(VGA_HEIGHT - 1) * VGA_WIDTH + x] = vga_entry(' ', terminal_color);
    }
    
    terminal_row = VGA_HEIGHT - 1;
    terminal_column = 0;
}

void terminal_putchar(char c) {
    if (c == '\n') {
        terminal_column = 0;
        if (++terminal_row == VGA_HEIGHT) {
            terminal_scroll();
        }
        return;
    }
    
    terminal_putentryat(c, terminal_color, terminal_column, terminal_row);
    if (++terminal_column == VGA_WIDTH) {
        terminal_column = 0;
        if (++terminal_row == VGA_HEIGHT) {
            terminal_scroll();
        }
    }
}

void terminal_write(const char* data, size_t size) {
    for (size_t i = 0; i < size; i++)
        terminal_putchar(data[i]);
}

void terminal_writestring(const char* data) {
    size_t len = 0;
    while (data[len]) len++; // strlen
    terminal_write(data, len);
}

// Port I/O functions
static inline void outb(uint16_t port, uint8_t val) {
    asm volatile ("outb %0, %1" : : "a"(val), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    asm volatile ("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

// PIC (Programmable Interrupt Controller) initialization
void init_pic(void) {
    // ICW1: Initialize command
    outb(0x20, 0x11);
    outb(0xA0, 0x11);
    
    // ICW2: Vector offset
    outb(0x21, 0x20); // Master PIC offset to 32
    outb(0xA1, 0x28); // Slave PIC offset to 40
    
    // ICW3: Cascade
    outb(0x21, 0x04); // Master has slave at IRQ2
    outb(0xA1, 0x02); // Slave cascade identity
    
    // ICW4: Environment
    outb(0x21, 0x01);
    outb(0xA1, 0x01);
    
    // Mask all interrupts initially
    outb(0x21, 0xFF);
    outb(0xA1, 0xFF);
}

// IDT (Interrupt Descriptor Table) entry
struct idt_entry {
    uint16_t base_low;
    uint16_t selector;
    uint8_t ist;
    uint8_t flags;
    uint16_t base_mid;
    uint32_t base_high;
    uint32_t reserved;
} __attribute__((packed));

struct idt_ptr {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed));

#define IDT_ENTRIES 256
static struct idt_entry idt[IDT_ENTRIES];
static struct idt_ptr idtp;

// Default interrupt handler
void default_handler(void) {
    terminal_writestring("Interrupt received!\n");
}

void set_idt_entry(int num, uint64_t handler, uint16_t selector, uint8_t flags) {
    idt[num].base_low = handler & 0xFFFF;
    idt[num].base_mid = (handler >> 16) & 0xFFFF;
    idt[num].base_high = (handler >> 32) & 0xFFFFFFFF;
    idt[num].selector = selector;
    idt[num].flags = flags;
    idt[num].ist = 0;
    idt[num].reserved = 0;
}

void init_idt(void) {
    idtp.limit = sizeof(idt) - 1;
    idtp.base = (uint64_t)&idt;
    
    // Initialize all entries to default handler
    for (int i = 0; i < IDT_ENTRIES; i++) {
        set_idt_entry(i, (uint64_t)default_handler, 0x08, 0x8E);
    }
    
    // Load IDT
    asm volatile ("lidt %0" : : "m"(idtp));
}

// Timer (PIT) initialization
void init_timer(void) {
    // Set PIT frequency to 100 Hz
    uint32_t divisor = 1193180 / 100;
    
    outb(0x43, 0x36); // Command byte
    outb(0x40, divisor & 0xFF);
    outb(0x40, divisor >> 8);
}

// Keyboard initialization
void init_keyboard(void) {
    // Enable keyboard
    outb(0x21, inb(0x21) & ~0x02); // Unmask IRQ1 (keyboard)
}

// Simple physical memory manager
static uint8_t* memory_bitmap;
static uint64_t total_pages;
static uint64_t used_pages;

#define PAGE_SIZE 4096
#define BITMAP_START 0x100000  // 1MB

void init_memory(void) {
    // Assume 64MB of RAM for simplicity
    total_pages = (64 * 1024 * 1024) / PAGE_SIZE;
    used_pages = 0;
    
    memory_bitmap = (uint8_t*)BITMAP_START;
    
    // Clear bitmap
    for (uint64_t i = 0; i < total_pages / 8; i++) {
        memory_bitmap[i] = 0;
    }
    
    // Mark first 2MB as used (kernel space)
    for (uint64_t i = 0; i < (2 * 1024 * 1024) / PAGE_SIZE; i++) {
        memory_bitmap[i / 8] |= (1 << (i % 8));
        used_pages++;
    }
    
    terminal_writestring("Memory manager initialized\n");
}

void* allocate_page(void) {
    for (uint64_t i = 0; i < total_pages; i++) {
        if (!(memory_bitmap[i / 8] & (1 << (i % 8)))) {
            memory_bitmap[i / 8] |= (1 << (i % 8));
            used_pages++;
            return (void*)(i * PAGE_SIZE);
        }
    }
    return NULL; // Out of memory
}

void free_page(void* page) {
    uint64_t page_num = (uint64_t)page / PAGE_SIZE;
    if (page_num < total_pages) {
        memory_bitmap[page_num / 8] &= ~(1 << (page_num % 8));
        used_pages--;
    }
}

// Main kernel function
void kernel_main(void) {
    // Initialize terminal
    terminal_initialize();
    
    terminal_setcolor(vga_entry_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK));
    terminal_writestring("NexusOS Kernel v1.0\n");
    terminal_writestring("====================\n\n");
    
    // Initialize hardware
    terminal_writestring("Initializing PIC...\n");
    init_pic();
    
    terminal_writestring("Initializing IDT...\n");
    init_idt();
    
    terminal_writestring("Initializing timer...\n");
    init_timer();
    
    terminal_writestring("Initializing keyboard...\n");
    init_keyboard();
    
    terminal_writestring("Initializing memory manager...\n");
    init_memory();
    
    // Enable interrupts
    terminal_writestring("Enabling interrupts...\n");
    asm volatile ("sti");
    
    terminal_setcolor(vga_entry_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK));
    terminal_writestring("\nNexusOS kernel initialization complete!\n");
    
    // Test memory allocation
    terminal_writestring("Testing memory allocation...\n");
    void* page1 = allocate_page();
    void* page2 = allocate_page();
    terminal_writestring("Allocated pages at: ");
    
    // Simple hex print
    char hex_buf[20];
    uint64_t addr = (uint64_t)page1;
    hex_buf[0] = '0'; hex_buf[1] = 'x';
    for (int i = 15; i >= 0; i--) {
        uint8_t digit = (addr >> (i * 4)) & 0xF;
        hex_buf[17-i] = (digit < 10) ? ('0' + digit) : ('A' + digit - 10);
    }
    hex_buf[18] = '\n';
    hex_buf[19] = '\0';
    terminal_write(hex_buf, 19);
    
    terminal_setcolor(vga_entry_color(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK));
    terminal_writestring("\nKernel is now running. System ready.\n");
    
    // Main kernel loop
    while (1) {
        asm volatile ("hlt"); // Wait for interrupts
    }
}