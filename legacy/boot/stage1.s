.code16
.section .text
.global _start16

_start16:
    # Stage 1 Bootloader - MBR compatible
    # Loaded at 0x7C00 by BIOS
    
    # Clear registers
    xor %ax, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    mov $0x7C00, %sp
    
    # Enable A20 line
    call enable_a20
    
    # Load GDT
    lgdt gdtr - stage1_start + 0x7C00
    
    # Enter protected mode
    mov %cr0, %eax
    or $1, %eax
    mov %eax, %cr0
    
    # Far jump to 32-bit code
    ljmp $0x08, $(stage2_32 - stage1_start + 0x7C00)

enable_a20:
    # Try fast method first
    in $0x92, %al
    or $2, %al
    out %al, $0x92
    ret

# GDT for protected mode
gdtr:
    .word gdt_end - gdt - 1
    .long gdt - stage1_start + 0x7C00

gdt:
    # Null descriptor
    .quad 0x0000000000000000
    
    # Code descriptor (0x08)
    .word 0xFFFF           # Limit low
    .word 0x0000           # Base low
    .byte 0x00             # Base mid
    .byte 0x9A             # Access byte
    .byte 0xCF             # Granularity
    .byte 0x00             # Base high
    
    # Data descriptor (0x10)
    .word 0xFFFF           # Limit low
    .word 0x0000           # Base low
    .byte 0x00             # Base mid
    .byte 0x92             # Access byte
    .byte 0xCF             # Granularity
    .byte 0x00             # Base high

gdt_end:

# Padding to make bootable
.org 0x1FE
.word 0xAA55
