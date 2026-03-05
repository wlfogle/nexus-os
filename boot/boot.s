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
    
    # Early serial output - test if we're here
    # Write 'B' to COM1 at 0x3F8 (data register)
    mov $0x3F8, %dx
    mov $0x42, %al               # 'B' = 0x42
    out %al, (%dx)               # Send to serial
    
    # Set up stack (EBP = 0 for initial frame)
    movl $stack_top, %esp
    xorl %ebp, %ebp
    
    # Write 'S' to serial
    mov $0x3F8, %dx
    mov $0x53, %al               # 'S'
    out %al, (%dx)
    
    # QEMU/Multiboot passes:
    # EAX = magic
    # EBX = pointer to multiboot info
    # kernel_main signature: void kernel_main(struct multiboot_info *mbi, uint32_t magic)
    # In cdecl, we push right-to-left: magic first, then mbi
    
    # Write 'T' to serial
    mov $0x3F8, %dx
    mov $0x54, %al               # 'T'
    out %al, (%dx)
    
    # Push arguments in cdecl order (right-to-left): magic, then mbi
    pushl %eax                   # Magic (rightmost arg, pushed first)
    pushl %ebx                   # Multiboot info (leftmost arg, pushed second)
    
    # Write 'C' to serial
    mov $0x3F8, %dx
    mov $0x43, %al               # 'C'
    out %al, (%dx)
    
    # Call kernel_main
    call kernel_main
    
    # Write 'H' to serial (shouldn't reach)
    mov $0x3F8, %dx
    mov $0x48, %al               # 'H'
    out %al, (%dx)
    
    # If kernel_main returns, halt indefinitely
    cli
    hlt
    jmp _start

.size _start, . - _start
