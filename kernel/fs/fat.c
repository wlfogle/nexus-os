#include "fat.h"
#include "../../include/kernel/block.h"
#include "../../include/kernel/serial.h"
#include "../../include/libc/string.h"
#include <stddef.h>

#define SECTOR_SIZE 512
#define FAT12_CLUSTER_SIZE 1024
#define ROOT_ENTRIES 224

typedef struct {
    uint8_t boot_jump[3];
    uint8_t oem[8];
    uint16_t bytes_per_sector;
    uint8_t sectors_per_cluster;
    uint16_t reserved_sectors;
    uint8_t num_fats;
    uint16_t root_entries;
    uint16_t total_sectors;
    uint8_t media;
    uint16_t sectors_per_fat;
    uint16_t sectors_per_track;
    uint16_t num_heads;
    uint32_t hidden_sectors;
    uint32_t large_sectors;
    uint8_t drive_number;
    uint8_t reserved;
    uint8_t boot_sig;
    uint32_t serial;
    uint8_t label[11];
    uint8_t fstype[8];
} __attribute__((packed)) fat_boot_sector_t;

typedef struct {
    uint8_t name[8];
    uint8_t ext[3];
    uint8_t attr;
    uint8_t reserved;
    uint8_t create_time_fine;
    uint16_t create_time;
    uint16_t create_date;
    uint16_t access_date;
    uint16_t cluster_high;
    uint16_t write_time;
    uint16_t write_date;
    uint16_t cluster_low;
    uint32_t size;
} __attribute__((packed)) fat_dirent_t;

typedef struct {
    int block_device_id;
    fat_boot_sector_t boot;
    uint32_t fat_start;
    uint32_t root_start;
    uint32_t data_start;
    uint32_t fat_sectors;
} fat_volume_t;

static fat_volume_t volume = {0};
static int fat_initialized = 0;

int fat_init(int block_device_id) {
    fat_boot_sector_t boot_buf;
    
    if (block_device_read(block_device_id, 0, 1, &boot_buf) != 1) {
        serial_puts("ERROR: Failed to read FAT boot sector\n");
        return -1;
    }
    
    if (boot_buf.bytes_per_sector != SECTOR_SIZE) {
        serial_puts("ERROR: Invalid sector size\n");
        return -1;
    }
    
    volume.block_device_id = block_device_id;
    volume.boot = boot_buf;
    volume.fat_start = boot_buf.reserved_sectors;
    volume.root_start = boot_buf.reserved_sectors + (boot_buf.num_fats * boot_buf.sectors_per_fat);
    volume.data_start = volume.root_start + ((boot_buf.root_entries * 32 + SECTOR_SIZE - 1) / SECTOR_SIZE);
    volume.fat_sectors = boot_buf.sectors_per_fat;
    
    fat_initialized = 1;
    serial_puts("FAT12 filesystem initialized\n");
    return 0;
}

static uint16_t fat_get_cluster(uint16_t cluster) {
    if (!fat_initialized || cluster == 0) return 0;
    
    uint32_t fat_offset = (cluster * 3) / 2;
    uint32_t fat_sector = volume.fat_start + (fat_offset / SECTOR_SIZE);
    uint16_t offset_in_sector = fat_offset % SECTOR_SIZE;
    
    uint8_t fat_buf[SECTOR_SIZE];
    if (block_device_read(volume.block_device_id, fat_sector, 1, fat_buf) != 1) {
        return 0;
    }
    
    uint16_t result = *(uint16_t *)&fat_buf[offset_in_sector];
    if (cluster & 1) {
        result = (result >> 4) & 0xFFF;
    } else {
        result = result & 0xFFF;
    }
    
    return result;
}

static int fat_read_cluster(uint16_t cluster, void *buffer) {
    if (!fat_initialized || cluster < 2 || cluster > 0xFF0) return -1;
    
    uint32_t sector = volume.data_start + ((cluster - 2) * volume.boot.sectors_per_cluster);
    return block_device_read(volume.block_device_id, sector, 
                            volume.boot.sectors_per_cluster, buffer);
}

int fat_read_file(const char *path, void *buffer, uint32_t max_size) {
    if (!fat_initialized || !path || !buffer) return -1;
    
    uint8_t root_buf[SECTOR_SIZE];
    if (block_device_read(volume.block_device_id, volume.root_start, 1, root_buf) != 1) {
        return -1;
    }
    
    fat_dirent_t *entries = (fat_dirent_t *)root_buf;
    int entry_count = (SECTOR_SIZE / sizeof(fat_dirent_t));
    
    uint16_t start_cluster = 0;
    uint32_t file_size = 0;
    
    for (int i = 0; i < entry_count && i < ROOT_ENTRIES; i++) {
        if (entries[i].name[0] == 0) break;
        if (entries[i].name[0] == 0xE5) continue;
        if (entries[i].attr & 0x10) continue;
        
        uint8_t fname[9] = {0};
        uint8_t fext[4] = {0};
        
        for (int j = 0; j < 8 && entries[i].name[j] != ' '; j++) {
            fname[j] = entries[i].name[j];
        }
        for (int j = 0; j < 3 && entries[i].ext[j] != ' '; j++) {
            fext[j] = entries[i].ext[j];
        }
        
        char full_name[13] = {0};
        strcpy(full_name, (char *)fname);
        if (fext[0] != 0) {
            strcat(full_name, ".");
            strcat(full_name, (char *)fext);
        }
        
        if (strcmp(full_name, path) == 0) {
            start_cluster = entries[i].cluster_low | (entries[i].cluster_high << 16);
            file_size = entries[i].size;
            break;
        }
    }
    
    if (start_cluster == 0) return -1;
    
    uint8_t *out = (uint8_t *)buffer;
    uint32_t bytes_read = 0;
    uint16_t current_cluster = start_cluster;
    
    while (current_cluster >= 2 && current_cluster < 0xFF0 && bytes_read < file_size) {
        uint8_t cluster_buf[FAT12_CLUSTER_SIZE];
        if (fat_read_cluster(current_cluster, cluster_buf) <= 0) return -1;
        
        uint32_t to_copy = (file_size - bytes_read) > FAT12_CLUSTER_SIZE ? 
                          FAT12_CLUSTER_SIZE : (file_size - bytes_read);
        
        if (bytes_read + to_copy > max_size) {
            to_copy = max_size - bytes_read;
        }
        
        memcpy(&out[bytes_read], cluster_buf, to_copy);
        bytes_read += to_copy;
        
        current_cluster = fat_get_cluster(current_cluster);
    }
    
    return bytes_read;
}

int fat_write_file(const char *path, void *buffer, uint32_t size) {
    if (!fat_initialized || !path || !buffer || size == 0) return -1;
    
    uint8_t root_buf[SECTOR_SIZE];
    if (block_device_read(volume.block_device_id, volume.root_start, 1, root_buf) != 1) {
        return -1;
    }
    
    fat_dirent_t *entries = (fat_dirent_t *)root_buf;
    int entry_count = SECTOR_SIZE / sizeof(fat_dirent_t);
    
    int empty_entry = -1;
    for (int i = 0; i < entry_count && i < ROOT_ENTRIES; i++) {
        if (entries[i].name[0] == 0 || entries[i].name[0] == 0xE5) {
            empty_entry = i;
            break;
        }
    }
    
    if (empty_entry == -1) return -1;
    
    uint8_t fname[9] = {0};
    uint8_t fext[4] = {0};
    const char *dot = strchr(path, '.');
    
    if (dot) {
        int name_len = dot - path;
        strncpy((char *)fname, path, name_len < 8 ? name_len : 8);
        strcpy((char *)fext, dot + 1);
    } else {
        strncpy((char *)fname, path, 8);
    }
    
    memset(entries[empty_entry].name, ' ', 8);
    memset(entries[empty_entry].ext, ' ', 3);
    
    for (int i = 0; fname[i] && i < 8; i++) {
        entries[empty_entry].name[i] = fname[i];
    }
    for (int i = 0; fext[i] && i < 3; i++) {
        entries[empty_entry].ext[i] = fext[i];
    }
    
    entries[empty_entry].attr = 0x20;
    entries[empty_entry].size = size;
    entries[empty_entry].cluster_low = 0x003;
    entries[empty_entry].cluster_high = 0;
    
    if (block_device_write(volume.block_device_id, volume.root_start, 1, root_buf) != 1) {
        return -1;
    }
    
    uint32_t sector_offset = (0x003 - 2) * volume.boot.sectors_per_cluster;
    uint32_t start_sector = volume.data_start + sector_offset;
    
    uint32_t bytes_written = 0;
    uint8_t *src = (uint8_t *)buffer;
    
    while (bytes_written < size) {
        uint32_t to_write = (size - bytes_written) > FAT12_CLUSTER_SIZE ? 
                           FAT12_CLUSTER_SIZE : (size - bytes_written);
        
        if (block_device_write(volume.block_device_id, start_sector, 
                              volume.boot.sectors_per_cluster, &src[bytes_written]) != 
            volume.boot.sectors_per_cluster) {
            return -1;
        }
        
        bytes_written += to_write;
        start_sector += volume.boot.sectors_per_cluster;
    }
    
    return bytes_written;
}

int fat_list_directory(void) {
    if (!fat_initialized) return -1;
    
    uint8_t root_buf[SECTOR_SIZE];
    if (block_device_read(volume.block_device_id, volume.root_start, 1, root_buf) != 1) {
        return -1;
    }
    
    fat_dirent_t *entries = (fat_dirent_t *)root_buf;
    int count = 0;
    
    for (int i = 0; i < ROOT_ENTRIES; i++) {
        if (entries[i].name[0] == 0) break;
        if (entries[i].name[0] == 0xE5) continue;
        
        uint8_t fname[9] = {0};
        uint8_t fext[4] = {0};
        
        for (int j = 0; j < 8 && entries[i].name[j] != ' '; j++) {
            fname[j] = entries[i].name[j];
        }
        for (int j = 0; j < 3 && entries[i].ext[j] != ' '; j++) {
            fext[j] = entries[i].ext[j];
        }
        
        serial_printf("%s", fname);
        if (fext[0]) {
            serial_printf(".%s", fext);
        }
        serial_printf(" (%u bytes)\n", entries[i].size);
        count++;
    }
    
    return count;
}
