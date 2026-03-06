#ifndef KERNEL_ATA_H
#define KERNEL_ATA_H

#include <stdint.h>

#define ATA_PRIMARY_BASE 0x1F0

typedef struct {
    uint16_t total_sectors;
} ata_drive_info_t;

void ata_init(void);
int ata_read_sectors(uint32_t lba, uint32_t count, void *buffer);
int ata_write_sectors(uint32_t lba, uint32_t count, void *buffer);
ata_drive_info_t *ata_get_drive_info(void);

#endif
