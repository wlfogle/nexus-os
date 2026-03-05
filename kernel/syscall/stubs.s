.code32

.section .text
.global syscall_int80
.global irq0

# Syscall entry point via INT 0x80
syscall_int80:
    push %ebp
    push %edi
    push %esi
    push %edx
    push %ecx
    push %ebx
    push %eax
    
    mov %esp, %eax          # Args pointer
    mov %eax, %edx          # Save for later
    sub $4, %esp
    mov %edx, (%esp)        # Push args
    
    call syscall_dispatch
    
    add $8, %esp
    pop %eax
    pop %ebx
    pop %ecx
    pop %edx
    pop %esi
    pop %edi
    pop %ebp
    iret

# IRQ 0: Timer interrupt
irq0:
    push $0                 # No error code
    push $32                # IRQ 0 = INT 32
    jmp irq_common

irq_common:
    pusha
    push %ds
    push %es
    push %fs
    push %gs
    
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    
    call timer_interrupt
    
    pop %gs
    pop %fs
    pop %es
    pop %ds
    popa
    add $8, %esp
    iret
