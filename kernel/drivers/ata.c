#include "ata.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

#define REG_DATA 0x00
#define REG_ERROR 0x01
#define REG_COUNT 0x02
#define REG_LBA0 0x03
#define REG_LBA1 0x04
#define REG_LBA2 0x05
#define REG_DEVICE 0x06
#define REG_STATUS 0x07
#define REG_CMD 0x07
#define REG_CTRL 0x06

#define CMD_READ 0x20
#define CMD_WRITE 0x30
#define CMD_IDENTIFY 0xEC

#define STATUS_BUSY 0x80
#define STATUS_READY 0x40
#define STATUS_ERROR 0x01
#define STATUS_DRQ 0x08

static ata_drive_info_t drive_info = {0};
static int drive_initialized = 0;

static inline void outb(uint16_t port, uint8_t val) {
    __asm__ volatile("outb %0, %1" : : "a"(val), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    __asm__ volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static inline uint16_t inw(uint16_t port) {
    uint16_t ret;
    __asm__ volatile("inw %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static int ata_wait_busy(int timeout) {
    for (int i = 0; i < timeout; i++) {
        uint8_t status = inb(ATA_PRIMARY_BASE + REG_STATUS);
        if (!(status & STATUS_BUSY)) return 0;
        for (volatile int j = 0; j < 1000; j++);
    }
    return -1;
}

static int ata_wait_ready(int timeout) {
    for (int i = 0; i < timeout; i++) {
        uint8_t status = inb(ATA_PRIMARY_BASE + REG_STATUS);
        if ((status & STATUS_READY)) return 0;
        for (volatile int j = 0; j < 1000; j++);
    }
    return -1;
}

static int ata_wait_data(int timeout) {
    for (int i = 0; i < timeout; i++) {
        uint8_t status = inb(ATA_PRIMARY_BASE + REG_STATUS);
        if ((status & STATUS_DRQ)) return 0;
        if ((status & STATUS_ERROR)) return -1;
        for (volatile int j = 0; j < 1000; j++);
    }
    return -1;
}

void ata_init(void) {
    serial_puts("Initializing ATA driver...\n");
    
    if (ata_wait_busy(100000) != 0) {
        serial_puts("ERROR: ATA device busy\n");
        return;
    }
    
    /* Send IDENTIFY command */
    outb(ATA_PRIMARY_BASE + REG_DEVICE, 0xA0);
    outb(ATA_PRIMARY_BASE + REG_CMD, CMD_IDENTIFY);
    
    if (ata_wait_data(100000) != 0) {
        serial_puts("ERROR: ATA identify failed\n");
        return;
    }
    
    /* Read IDENTIFY data */
    uint16_t *buf = (uint16_t *)&drive_info;
    for (int i = 0; i < 256; i++) {
        uint16_t word = inw(ATA_PRIMARY_BASE + REG_DATA);
        if (i < sizeof(ata_drive_info_t) / 2) {
            buf[i] = word;
        }
    }
    
    drive_info.total_sectors = 0x100000;
    drive_initialized = 1;
    serial_puts("ATA drive initialized\n");
}

int ata_read_sectors(uint32_t lba, uint16_t count, void *buffer) {
    if (!drive_initialized || !buffer || count == 0) return -1;
    
    if (ata_wait_busy(10000) != 0) return -1;
    
    uint8_t *buf = (uint8_t *)buffer;
    
    for (uint16_t i = 0; i < count; i++) {
        uint32_t current_lba = lba + i;
        
        outb(ATA_PRIMARY_BASE + REG_COUNT, 1);
        outb(ATA_PRIMARY_BASE + REG_LBA0, current_lba & 0xFF);
        outb(ATA_PRIMARY_BASE + REG_LBA1, (current_lba >> 8) & 0xFF);
        outb(ATA_PRIMARY_BASE + REG_LBA2, (current_lba >> 16) & 0xFF);
        outb(ATA_PRIMARY_BASE + REG_DEVICE, 0xA0 | ((current_lba >> 24) & 0x0F));
        outb(ATA_PRIMARY_BASE + REG_CMD, CMD_READ);
        
        if (ata_wait_data(10000) != 0) return -1;
        
        for (int j = 0; j < 256; j++) {
            uint16_t word = inw(ATA_PRIMARY_BASE + REG_DATA);
            buf[i * 512 + j * 2] = word & 0xFF;
            buf[i * 512 + j * 2 + 1] = (word >> 8) & 0xFF;
        }
    }
    
    return count;
}

int ata_write_sectors(uint32_t lba, uint16_t count, void *buffer) {
    if (!drive_initialized || !buffer || count == 0) return -1;
    
    if (ata_wait_busy(10000) != 0) return -1;
    
    uint8_t *buf = (uint8_t *)buffer;
    
    for (uint16_t i = 0; i < count; i++) {
        uint32_t current_lba = lba + i;
        
        outb(ATA_PRIMARY_BASE + REG_COUNT, 1);
        outb(ATA_PRIMARY_BASE + REG_LBA0, current_lba & 0xFF);
        outb(ATA_PRIMARY_BASE + REG_LBA1, (current_lba >> 8) & 0xFF);
        outb(ATA_PRIMARY_BASE + REG_LBA2, (current_lba >> 16) & 0xFF);
        outb(ATA_PRIMARY_BASE + REG_DEVICE, 0xA0 | ((current_lba >> 24) & 0x0F));
        outb(ATA_PRIMARY_BASE + REG_CMD, CMD_WRITE);
        
        if (ata_wait_data(10000) != 0) return -1;
        
        for (int j = 0; j < 256; j++) {
            uint16_t word = buf[i * 512 + j * 2] | (buf[i * 512 + j * 2 + 1] << 8);
            outb(ATA_PRIMARY_BASE + REG_DATA, word & 0xFF);
            outb(ATA_PRIMARY_BASE + REG_DATA, (word >> 8) & 0xFF);
        }
    }
    
    return count;
}

ata_drive_info_t *ata_get_drive_info(void) {
    return drive_initialized ? &drive_info : NULL;
}
