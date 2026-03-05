/*
 * stubs.s -- x86 32-bit interrupt and syscall entry points
 *
 * Hardware IRQ entry (IRQ_STUB macro)
 * ------------------------------------
 * Each IRQ stub pushes its line number then jumps to irq_common_handler.
 * Stack layout on arrival at irq_common_handler (offsets from ESP after
 * all saves):
 *
 *   [esp+ 0]  GS           (pushed by irq_common_handler)
 *   [esp+ 4]  FS
 *   [esp+ 8]  ES
 *   [esp+12]  DS
 *   [esp+16]  EDI  \
 *   [esp+20]  ESI   |
 *   [esp+24]  EBP   |
 *   [esp+28]  ESP*  |  pusha block  (*value of ESP before pusha)
 *   [esp+32]  EBX   |
 *   [esp+36]  EDX   |
 *   [esp+40]  ECX   |
 *   [esp+44]  EAX  /
 *   [esp+48]  IRQ line number  (pushed by stub)
 *   [esp+52]  EIP              (pushed by CPU)
 *   [esp+56]  CS
 *   [esp+60]  EFLAGS
 *
 * Syscall entry (INT 0x80)
 * -------------------------
 * EAX = syscall number, EBX = arg1, ECX = arg2, EDX = arg3,
 * ESI = arg4, EDI = arg5.
 * Builds struct syscall_args on the stack and calls
 *   syscall_dispatch(uint32_t num, struct syscall_args *args)
 * Return value in EAX is forwarded to the caller via iret.
 */

.code32
.section .text

/* ------------------------------------------------------------------ */
/* IRQ stubs (IRQ 0-15 -> INT 32-47)                                  */
/* ------------------------------------------------------------------ */

.macro IRQ_STUB num
.globl irq\num
irq\num:
    pushl $\num
    jmp   irq_common_handler
.endm

IRQ_STUB 0
IRQ_STUB 1
IRQ_STUB 2
IRQ_STUB 3
IRQ_STUB 4
IRQ_STUB 5
IRQ_STUB 6
IRQ_STUB 7
IRQ_STUB 8
IRQ_STUB 9
IRQ_STUB 10
IRQ_STUB 11
IRQ_STUB 12
IRQ_STUB 13
IRQ_STUB 14
IRQ_STUB 15

/* ------------------------------------------------------------------ */
/* Common IRQ handler                                                  */
/* ------------------------------------------------------------------ */

.extern irq_dispatch

irq_common_handler:
    pusha                       /* EDI,ESI,EBP,ESP,EBX,EDX,ECX,EAX (32 bytes) */
    push %ds
    push %es
    push %fs
    push %gs

    /* Switch to kernel data segment (GDT selector 0x10) */
    movl  $0x10, %eax
    movw  %ax, %ds
    movw  %ax, %es
    movw  %ax, %fs
    movw  %ax, %gs

    /* IRQ number sits at [esp+48]; read it before altering the stack */
    movl  48(%esp), %eax
    pushl %eax                  /* irq_dispatch(uint32_t irq_num) */
    call  irq_dispatch
    addl  $4, %esp              /* discard argument */

    /* Restore saved state in reverse order */
    pop   %gs
    pop   %fs
    pop   %es
    pop   %ds
    popa
    addl  $4, %esp              /* discard the IRQ number pushed by the stub */
    iret

/* ------------------------------------------------------------------ */
/* Syscall gate -- INT 0x80                                            */
/* ------------------------------------------------------------------ */

.extern syscall_dispatch

.globl syscall_int80
syscall_int80:
    /*
     * Push registers in reverse field order so that after all six pushes
     * ESP points to a valid struct syscall_args { eax, ebx, ecx, edx, esi, edi }:
     *
     *   push edi  -> [esp+20] once all done
     *   push esi  -> [esp+16]
     *   push edx  -> [esp+12]
     *   push ecx  -> [esp+ 8]
     *   push ebx  -> [esp+ 4]
     *   push eax  -> [esp+ 0]  <- struct base pointer, eax = syscall number
     */
    pushl %edi
    pushl %esi
    pushl %edx
    pushl %ecx
    pushl %ebx
    pushl %eax          /* EAX still contains the syscall number */

    /*
     * syscall_dispatch(uint32_t num, struct syscall_args *args)
     * cdecl: push args right-to-left (arg2 then arg1).
     */
    movl  %esp, %ecx    /* ecx = &args (= current ESP) */
    pushl %ecx          /* arg2: pointer to struct syscall_args */
    pushl %eax          /* arg1: syscall number (EAX unchanged) */
    call  syscall_dispatch
    addl  $8, %esp      /* discard the two call arguments */

    /*
     * EAX = syscall return value from C.
     * Skip the struct.eax slot (original syscall number) to avoid
     * overwriting the return value, then restore the other registers.
     */
    addl  $4, %esp      /* skip struct.eax */
    popl  %ebx
    popl  %ecx
    popl  %edx
    popl  %esi
    popl  %edi
    iret
