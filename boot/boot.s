# Multiboot bootloader - 32-bit entry point
.code32

.set ALIGN,    1<<0             # align loaded modules on page boundaries
.set MEMINFO,  1<<1             # provide memory map
.set FLAGS,    ALIGN | MEMINFO  # this is the Multiboot 'flag' field
.set MAGIC,    0x1BADB002       # 'magic number' lets bootloader find the header
.set CHECKSUM, -(MAGIC + FLAGS) # checksum of above, to prove we are multiboot

# Declare a multiboot header that marks the program as a kernel
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
    cli
    cld
    
    # Set up stack
    movl $stack_top, %esp
    
    # Push multiboot arguments
    pushl %eax              # Magic
    pushl %ebx              # Multiboot info
    
    # Call kernel
    call kernel_main
    
    # Halt
    cli
    hlt

.size _start, . - _start
