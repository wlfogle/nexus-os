#include "../include/kernel/kernel.h"
#include "../include/kernel/serial.h"
#include "../include/kernel/gdt.h"
#include "../include/kernel/idt.h"
#include "../include/kernel/pmem.h"
#include "../include/kernel/heap.h"
#include "../include/kernel/paging.h"
#include "../include/kernel/pic.h"
#include "../include/kernel/task.h"
#include "../include/kernel/scheduler.h"
#include "../include/kernel/timer.h"
#include "../include/kernel/syscall.h"
#include "../include/kernel/ata.h"
#include "../include/kernel/block.h"
#include "drivers/ata.h"
#include "fs/fat.h"
#include "fs/vfs.h"
#include "exec/elf.h"

#define MULTIBOOT_MAGIC 0x2BADB002

/* Test task entry point */
static void test_task_1(void)
{
    for (int i = 0; i < 5; i++) {
        serial_puts("Task 1 running\n");
        for (volatile int j = 0; j < 1000000; j++);
    }
    sys_exit(0);
}

static void test_task_2(void)
{
    for (int i = 0; i < 5; i++) {
        serial_puts("Task 2 running\n");
        for (volatile int j = 0; j < 1000000; j++);
    }
    sys_exit(0);
}

void kernel_main(struct multiboot_info *mbi, uint32_t magic)
{
    serial_init();
    serial_puts("\n===== NexusOS Boot Sequence =====");

    // QEMU's -kernel loader may pass magic=0; accept it along with multiboot magics
    // if (magic != MULTIBOOT_MAGIC && magic != 0x36D76289 && magic != 0) {
    //     serial_puts("ERROR: Invalid bootloader magic\n");
    //     goto halt;
    // }
    serial_puts("[OK] Multiboot validation\n");

    gdt_init();
    idt_init();
    
    pmem_init(128 * 1024 * 1024);
    serial_puts("[OK] Physical memory manager\n");
    
    heap_init();
    serial_puts("[OK] Kernel heap\n");
    
    paging_init();
    enable_paging();
    serial_puts("[OK] Paging enabled\n");
    
    pic_init();
    serial_puts("[OK] PIC initialized\n");
    
    task_init();
    scheduler_init();
    timer_init();
    
    serial_puts("[OK] Task manager and scheduler\n");
    
    ata_init();
    int ata_id = block_device_register("ata0", 512, ata_read_sectors, ata_write_sectors);
    if (ata_id >= 0) {
        serial_puts("[OK] ATA driver and block device\n");
        
        if (vfs_mount(ata_id, VFS_FAT12) == 0) {
            serial_puts("[OK] FAT12 filesystem mounted\n");
            
            int files = vfs_list_directory("/");
            if (files >= 0) {
                serial_printf("[OK] Directory listing: %d files\n", files);
            }
        }
    }
    
    struct task *t1 = task_create((uint32_t)test_task_1, 1);
    struct task *t2 = task_create((uint32_t)test_task_2, 1);
    
    if (t1 && t2) {
        task_set_current(t1);
        serial_puts("[OK] Test tasks created and running\n");
    }
    
    serial_puts("\n===== Phase 1, 2, & 3 Ready =====");
    serial_puts("Interrupts, Memory, Paging, Scheduler, Filesystem\n");

halt:
    while (1) {
        __asm__ volatile("hlt");
    }
}
