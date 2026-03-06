.section .text
.align 4

.extern exception_handler

/* Macro for exceptions without error code */
.macro ISR_NO_ERROR num
.globl isr\num
isr\num:
    pushl $0              /* Push dummy error code */
    pushl $\num           /* Push exception number */
    jmp isr_common
.endm

/* Macro for exceptions with error code */
.macro ISR_ERROR num
.globl isr\num
isr\num:
    pushl $\num           /* Push exception number */
    jmp isr_common
.endm

/* Exception stubs - exceptions 0-7 (no error code) */
ISR_NO_ERROR 0
ISR_NO_ERROR 1
ISR_NO_ERROR 2
ISR_NO_ERROR 3
ISR_NO_ERROR 4
ISR_NO_ERROR 5
ISR_NO_ERROR 6
ISR_NO_ERROR 7

/* Exception 8 - Double Fault (has error code) */
ISR_ERROR 8

/* Exception 9 (no error code) */
ISR_NO_ERROR 9

/* Exceptions 10-14 (have error codes) */
ISR_ERROR 10
ISR_ERROR 11
ISR_ERROR 12
ISR_ERROR 13
ISR_ERROR 14

/* Exceptions 15-31 (no error code) */
ISR_NO_ERROR 15
ISR_NO_ERROR 16
ISR_NO_ERROR 17
ISR_NO_ERROR 18
ISR_NO_ERROR 19
ISR_NO_ERROR 20
ISR_NO_ERROR 21
ISR_NO_ERROR 22
ISR_NO_ERROR 23
ISR_NO_ERROR 24
ISR_NO_ERROR 25
ISR_NO_ERROR 26
ISR_NO_ERROR 27
ISR_NO_ERROR 28
ISR_NO_ERROR 29
ISR_NO_ERROR 30
ISR_NO_ERROR 31

/* Common ISR handler */
isr_common:
    /* Stack now contains: exception_num, error_code */
    pushal                 /* Push all general registers */
    pushl %ds
    pushl %es
    pushl %fs
    pushl %gs
    
    movl $0x10, %eax       /* Load kernel data selector */
    movl %eax, %ds
    movl %eax, %es
    movl %eax, %fs
    movl %eax, %gs
    
    /* Call exception_handler(exception_num, error_code) */
    /* Arguments on stack: [esp+36]=error_code, [esp+40]=exception_num */
    movl 36(%esp), %eax    /* exception_num into eax */
    movl 40(%esp), %edx    /* error_code into edx */
    pushl %edx             /* Push error_code (2nd arg) */
    pushl %eax             /* Push exception_num (1st arg) */
    call exception_handler
    addl $8, %esp          /* Clean up arguments */
    
    popl %gs
    popl %fs
    popl %es
    popl %ds
    popal
    
    addl $8, %esp          /* Remove exception_num and error_code */
    iret
