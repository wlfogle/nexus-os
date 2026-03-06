.code32

.section .text
.global switch_to_task
.global get_esp

# void switch_to_task(struct task *task)
# esp+4 = task pointer
switch_to_task:
    push %ebp
    mov %esp, %ebp
    
    mov 8(%ebp), %eax       # Get task pointer
    
    # Load task registers
    mov 4(%eax), %ebx       # task->regs.ebx
    mov 8(%eax), %ecx       # task->regs.ecx
    mov 12(%eax), %edx      # task->regs.edx
    mov 16(%eax), %esi      # task->regs.esi
    mov 20(%eax), %edi      # task->regs.edi
    
    # Set ESP to task's stack
    mov 24(%eax), %esp      # task->regs.ebp
    mov 28(%eax), %esp      # task->regs.esp
    
    pop %ebp
    ret

# Get current ESP value
get_esp:
    mov %esp, %eax
    ret
