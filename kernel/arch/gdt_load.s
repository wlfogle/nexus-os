.code32

.section .text
.global gdt_load
.type gdt_load, @function

gdt_load:
    mov 4(%esp), %eax
    lgdt (%eax)
    
    # Perform a far jump using ljmp
    # In AT&T syntax: ljmp $code_sel, $address
    # But address must be resolved at link time
    # Use a trick: use absolute address form
    jmp 1f  # Skip the ljmp target
    
    # Define the far jump target here (unreachable but defines label)
.globl gdt_load_continue
gdt_load_continue:
    # Now in kernel code segment
    mov $0x10, %eax
    mov %eax, %ds
    mov %eax, %es
    mov %eax, %fs
    mov %eax, %gs
    mov %eax, %ss
    ret
    
1:  # Do the actual far jump
    # Manual ljmp: push far return address then jump
    lea gdt_load_continue, %eax
    push $0x08              # Code segment selector
    push %eax               # Offset
    lret                    # Long return (far jump)

.size gdt_load, . - gdt_load
