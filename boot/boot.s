# Multiboot 1 bootloader - 32-bit entry point
.code32

# Multiboot 1 header constants
.set ALIGN, 1<<0
.set MEMINFO, 1<<1
.set FLAGS, ALIGN | MEMINFO
.set MAGIC, 0x1BADB002
.set CHECKSUM, -(MAGIC + FLAGS)

# Multiboot 1 header
.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

# Reserve a stack for the initial thread
.section .bss
.align 16
stack_bottom:
.skip 32768  # 32 KiB stack
stack_top:

# The kernel entry point
.section .text
.global _start
.type _start, @function
_start:
    cli                          # Disable interrupts
    cld                          # Clear direction flag
    
    # Set up stack (EBP = 0 for initial frame)
    movl $stack_top, %esp
    xorl %ebp, %ebp
    
    # QEMU/Multiboot2 passes:
    # EAX = 0x36D76289 (magic)
    # EBX = pointer to multiboot info
    # Save them for kernel_main
    movl %eax, %esi              # Save magic
    movl %ebx, %edi              # Save mbi
    
    # Push arguments in cdecl order (right-to-left)
    pushl %esi                   # Magic
    pushl %edi                   # Multiboot info
    
    # Call kernel_main
    call kernel_main
    
    # If kernel_main returns, halt indefinitely
    cli
    hlt
    jmp _start

.size _start, . - _start
