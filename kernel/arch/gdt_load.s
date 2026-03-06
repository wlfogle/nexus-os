.code32

.section .text
.global gdt_load
.type gdt_load, @function

gdt_load:
    # Load GDT descriptor from argument on stack
    mov 4(%esp), %eax
    lgdt (%eax)
    
    # Far jump to reload CS register
    # ljmp syntax: ljmp $segment, $offset
    # GAS AT&T doesn't allow $ on the offset, use indirect form instead
    # Push selector and offset, use lret
    pushl $0x08             # Push code segment selector
    pushl $.Lgdt_flush      # Push return address
    lret                    # Long return (equivalent to ljmp)

.Lgdt_flush:
    # Now executing with new code segment
    # Update all data segments to kernel data selector
    mov $0x10, %eax         # Kernel data segment selector (0x10)
    mov %eax, %ds
    mov %eax, %es
    mov %eax, %fs
    mov %eax, %gs
    mov %eax, %ss
    
    ret

.size gdt_load, . - gdt_load
