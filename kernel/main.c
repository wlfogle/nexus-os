#include "../include/kernel/kernel.h"
#include "../include/kernel/serial.h"
#include "../include/kernel/gdt.h"
#include "../include/kernel/idt.h"
#include "../include/kernel/pmem.h"
#include "../include/kernel/heap.h"
#include "../include/kernel/paging.h"
#include "../include/kernel/pic.h"
#include "../include/kernel/irq.h"
#include "../include/kernel/console.h"
#include "../include/kernel/keyboard.h"
#include "../include/kernel/kshell.h"
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
#include "../include/kernel/netdev.h"
#include "../include/kernel/ethernet.h"
#include "../include/kernel/arp.h"
#include "../include/kernel/ipv4.h"
#include "../include/kernel/icmp.h"
#include "../include/kernel/udp.h"
#include "../include/kernel/tcp.h"
#include "../include/kernel/device.h"
#include "../include/kernel/model_serving.h"
#include "../include/kernel/autoscale.h"
#include "../include/kernel/pipeline.h"
#include "../include/kernel/tenant.h"
#include "../include/kernel/federated.h"
#include "../include/kernel/model_registry.h"

#define MULTIBOOT_MAGIC 0x2BADB002

/* Initialize network stack */
static void network_init(void)
{
    /* Create loopback device for testing */
    struct netdev loopback;
    loopback.dev_id = 0;
    loopback.name[0] = 'l';
    loopback.name[1] = 'o';
    loopback.name[2] = '0';
    loopback.name[3] = '\0';
    
    /* Loopback address: 127.0.0.1 */
    loopback.ip_addr.addr[0] = 127;
    loopback.ip_addr.addr[1] = 0;
    loopback.ip_addr.addr[2] = 0;
    loopback.ip_addr.addr[3] = 1;
    
    /* Loopback MAC (dummy) */
    loopback.mac_addr.addr[0] = 0x00;
    loopback.mac_addr.addr[1] = 0x00;
    loopback.mac_addr.addr[2] = 0x00;
    loopback.mac_addr.addr[3] = 0x00;
    loopback.mac_addr.addr[4] = 0x00;
    loopback.mac_addr.addr[5] = 0x01;
    
    loopback.mtu = 65535;
    loopback.flags = IFF_UP | IFF_LOOPBACK | IFF_RUNNING;
    loopback.ops = NULL;
    loopback.priv = NULL;
    
    if (netdev_register(&loopback) == 0) {
        serial_puts("[OK] Loopback device registered (127.0.0.1)\n");
    }
    
    /* Initialize UDP */
    udp_init();
    serial_puts("[OK] UDP protocol initialized\n");
}

void kernel_main(struct multiboot_info *mbi __attribute__((unused)),
                 uint32_t magic __attribute__((unused)))
{
    /* --- Stage 1: Serial-only (VGA not ready yet) --- */
    serial_init();
    serial_puts("\n===== NexusOS Boot Sequence =====\n");

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
    irq_init();          /* Wire IRQ 0-15 + INT 0x80 into IDT; enables interrupts */
    serial_puts("[OK] PIC + IRQ vectors\n");

    /* --- Stage 2: Console ready (serial + VGA) --- */
    console_init();      /* Calls vga_init() */
    keyboard_init();     /* Unmasks IRQ1 */
    console_puts("[OK] Console (VGA + keyboard)\n");

    task_init();
    scheduler_init();
    timer_init();
    console_puts("[OK] Task manager and scheduler\n");

    device_init();
    console_puts("[OK] Device registry\n");

    network_init();
    console_puts("[OK] Network stack\n");

    ata_init();
    int ata_id = block_device_register("ata0", 512, ata_read_sectors, ata_write_sectors);
    if (ata_id >= 0) {
        console_puts("[OK] ATA driver and block device\n");

        if (vfs_mount(ata_id, VFS_FAT12) == 0) {
            console_puts("[OK] FAT12 filesystem mounted\n");

            int files = vfs_list_directory("/");
            if (files >= 0) {
                console_printf("[OK] Directory listing: %d files\n", files);
            }
        }
    }

    /* Phase 12: Cloud-Native ML Platform */
    model_serving_init();
    autoscale_init();
    pipeline_init();
    console_puts("[OK] Cloud-native ML platform\n");

    /* Phase 13: Multi-Tenant ML Orchestration */
    tenant_init();
    federated_init();
    model_registry_init();
    console_puts("[OK] Multi-tenant orchestration\n");

    console_puts("\n===== NexusOS Ready (Phases 0-13) =====\n");

    /* Enter the interactive kernel shell (does not return) */
    kshell_run();
}
