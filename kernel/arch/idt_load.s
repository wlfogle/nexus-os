.code32

.section .text
.global idt_load
.type idt_load, @function

# void idt_load(struct idt_ptr *ptr)
# Load IDT pointer from argument (on stack at [esp+4])
idt_load:
    push %ebp
    mov %esp, %ebp

    # Get pointer to idt_ptr structure from stack argument
    mov 8(%ebp), %eax

    # Load IDT register with lidt instruction
    lidt (%eax)

    pop %ebp
    ret

.size idt_load, . - idt_load
