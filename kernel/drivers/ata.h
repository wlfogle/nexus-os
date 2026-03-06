#ifndef KERNEL_ATA_H
#define KERNEL_ATA_H

#include "../../include/libc/stdint.h"

#define ATA_PRIMARY_BASE 0x1F0
#define ATA_PRIMARY_CTRL 0x3F6
#define ATA_SECTOR_SIZE 512

typedef struct {
    uint16_t cylinders;
    uint16_t heads;
    uint16_t sectors;
    uint64_t total_sectors;
} ata_drive_info_t;

void ata_init(void);
int ata_read_sectors(uint32_t lba, uint32_t count, void *buffer);
int ata_write_sectors(uint32_t lba, uint32_t count, void *buffer);
ata_drive_info_t *ata_get_drive_info(void);

#endif
