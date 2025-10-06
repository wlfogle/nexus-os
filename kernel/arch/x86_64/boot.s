.section .text
.global _start
.code64

_start:
    # Clear interrupts
    cli
    
    # Set up kernel stack
    movq $kernel_stack_top, %rsp
    
    # Clear direction flag
    cld
    
    # Initialize basic GDT
    lgdt gdt_descriptor
    
    # Reload code segment
    pushq $0x08
    pushq $reload_cs
    retfq
    
reload_cs:
    # Reload data segments
    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    
    # Call kernel main
    call kernel_main
    
    # Halt if kernel returns
halt_loop:
    hlt
    jmp halt_loop

# GDT (Global Descriptor Table)
.section .data
.align 16
gdt_start:
    # Null descriptor
    .quad 0
    
    # Code segment (64-bit)
    .word 0          # Limit low
    .word 0          # Base low
    .byte 0          # Base middle
    .byte 0b10011010 # Access: present, ring 0, executable, readable
    .byte 0b10100000 # Granularity: 64-bit, not 32-bit
    .byte 0          # Base high
    
    # Data segment
    .word 0          # Limit low
    .word 0          # Base low
    .byte 0          # Base middle
    .byte 0b10010010 # Access: present, ring 0, writable
    .byte 0          # Granularity
    .byte 0          # Base high

gdt_descriptor:
    .word gdt_descriptor - gdt_start - 1
    .quad gdt_start

# Kernel stack (16KB)
.section .bss
.align 16
kernel_stack_bottom:
    .skip 16384
kernel_stack_top: