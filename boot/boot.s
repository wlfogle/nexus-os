# Multiboot header for x86_64 compatibility
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

# Reserve a stack for the initial thread (increased size for x86_64)
.section .bss
.align 16
stack_bottom:
.skip 32768 # 32 KiB (larger stack for x86_64)
stack_top:

# The kernel entry point
.section .text
.global _start
.type _start, @function
_start:
    # Initialize the stack pointer
    mov $stack_top, %esp
    
    # Call the kernel main function
    call kernel_main
    
    # If kernel_main returns, put the computer into an infinite loop
    cli
1:  hlt
    jmp 1b

# Set the size of the _start symbol to the current location '.' minus its start
.size _start, . - _start