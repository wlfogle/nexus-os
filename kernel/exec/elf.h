#ifndef KERNEL_ELF_H
#define KERNEL_ELF_H

#include <stdint.h>

int elf_validate(void *elf_data);
uint32_t elf_get_entry(void *elf_data);
int elf_load(void *elf_data, uint32_t max_size);
int elf_execute(void *elf_data, uint32_t max_size);

#endif
