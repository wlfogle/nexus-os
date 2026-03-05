.code32

.section .text
.global _start
.type _start, @function

_start:
    /* Load base pointer and setup stack frame */
    xor %ebp, %ebp
    
    /* argc and argv would be passed by kernel, but for now simplified */
    /* Stack: [return address] [argc] [argv]  */
    /* We'll get them from the stack if kernel passes them */
    
    /* Get argc from stack at ESP (first arg to _start) */
    mov (%esp), %eax        /* eax = argc (if kernel put it there) */
    mov %eax, %ebx
    
    /* For now, assume argc=0, argv=NULL - can be enhanced later */
    xor %eax, %eax          /* eax = argc = 0 */
    xor %ecx, %ecx          /* ecx = argv = NULL */
    
    /* Call main(argc, argv) */
    push %ecx               /* push argv */
    push %eax               /* push argc */
    call main
    
    /* main() returns in eax - this is the exit code */
    mov %eax, %ebx          /* move exit code to ebx for exit() call */
    
    /* Call exit(return_code) which will syscall to kernel */
    push %ebx
    call exit
    
    /* Should never reach here */
    jmp .

.size _start, . - _start
