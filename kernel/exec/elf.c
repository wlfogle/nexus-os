#include "elf.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define ELF_MAGIC 0x464C457F

typedef struct {
    uint32_t magic;
    uint8_t ei_class;
    uint8_t ei_data;
    uint8_t ei_version;
    uint8_t ei_osabi;
    uint8_t ei_abiversion;
    uint8_t pad[7];
    uint16_t e_type;
    uint16_t e_machine;
    uint32_t e_version;
    uint32_t e_entry;
    uint32_t e_phoff;
    uint32_t e_shoff;
    uint32_t e_flags;
    uint16_t e_ehsize;
    uint16_t e_phentsize;
    uint16_t e_phnum;
    uint16_t e_shentsize;
    uint16_t e_shnum;
    uint16_t e_shstrndx;
} __attribute__((packed)) elf_header_t;

typedef struct {
    uint32_t p_type;
    uint32_t p_offset;
    uint32_t p_vaddr;
    uint32_t p_paddr;
    uint32_t p_filesz;
    uint32_t p_memsz;
    uint32_t p_flags;
    uint32_t p_align;
} __attribute__((packed)) elf_program_header_t;

#define PT_LOAD 1

int elf_validate(void *elf_data) {
    if (!elf_data) return -1;
    
    elf_header_t *header = (elf_header_t *)elf_data;
    
    if (header->magic != ELF_MAGIC) {
        serial_puts("ERROR: Invalid ELF magic\n");
        return -1;
    }
    
    if (header->ei_class != 1) {
        serial_puts("ERROR: Not 32-bit ELF\n");
        return -1;
    }
    
    if (header->e_machine != 3) {
        serial_puts("ERROR: Not x86 ELF\n");
        return -1;
    }
    
    return 0;
}

uint32_t elf_get_entry(void *elf_data) {
    if (!elf_data) return 0;
    elf_header_t *header = (elf_header_t *)elf_data;
    return header->e_entry;
}

int elf_load(void *elf_data, uint32_t max_size) {
    if (!elf_data) return -1;
    
    if (elf_validate(elf_data) != 0) return -1;
    
    elf_header_t *header = (elf_header_t *)elf_data;
    
    if (header->e_phoff + (header->e_phnum * sizeof(elf_program_header_t)) > max_size) {
        serial_puts("ERROR: ELF corrupted\n");
        return -1;
    }
    
    uint8_t *elf_bytes = (uint8_t *)elf_data;
    elf_program_header_t *pheaders = (elf_program_header_t *)(elf_bytes + header->e_phoff);
    
    for (int i = 0; i < header->e_phnum; i++) {
        if (pheaders[i].p_type != PT_LOAD) continue;
        
        if (pheaders[i].p_offset + pheaders[i].p_filesz > max_size) {
            serial_puts("ERROR: ELF segment corrupted\n");
            return -1;
        }
        
        uint8_t *segment_data = elf_bytes + pheaders[i].p_offset;
        uint32_t vaddr = pheaders[i].p_vaddr;
        uint32_t filesz = pheaders[i].p_filesz;
        uint32_t memsz = pheaders[i].p_memsz;
        
        memcpy((void *)vaddr, segment_data, filesz);
        
        if (memsz > filesz) {
            memset((void *)(vaddr + filesz), 0, memsz - filesz);
        }
    }
    
    return 0;
}

typedef int (*elf_entry_fn)(void);

int elf_execute(void *elf_data, uint32_t max_size) {
    if (!elf_data) return -1;
    
    if (elf_load(elf_data, max_size) != 0) {
        serial_puts("ERROR: Failed to load ELF\n");
        return -1;
    }
    
    uint32_t entry = elf_get_entry(elf_data);
    if (entry == 0) {
        serial_puts("ERROR: Invalid ELF entry point\n");
        return -1;
    }
    
    elf_entry_fn fn = (elf_entry_fn)entry;
    int result = fn();
    
    return result;
}
