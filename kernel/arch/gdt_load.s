.code32

.section .text
.global gdt_load
.type gdt_load, @function

# void gdt_load(struct gdt_ptr *ptr)
# Load GDT pointer from argument (on stack at [esp+4])
gdt_load:
    # Get pointer to gdt_ptr structure from stack
    mov 4(%esp), %eax

    # Load GDT register with lgdt instruction
    lgdt (%eax)

    # Update segment registers with kernel selectors
    # Code segment selector = 0x08 (kernel code)
    # Data segment selector = 0x10 (kernel data)

    # Far jump to reload CS (code segment)
    # This must be done as a far jump to actually update CS
    ljmp $0x08, $1f

1:
    # Now we're in kernel code segment
    # Update data segment registers
    movl $0x10, %eax        # Kernel data selector
    movl %eax, %ds          # Data segment
    movl %eax, %es          # Extra segment
    movl %eax, %fs          # FS segment
    movl %eax, %gs          # GS segment
    movl %eax, %ss          # Stack segment

    ret

.size gdt_load, . - gdt_load
