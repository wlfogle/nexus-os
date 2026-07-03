; NexusOS Linux ABI smoke-test program — linux_hello.asm
; =============================================================================
; Static ELF64 linked at the conventional Linux base (0x400000). It does NOT use
; NexusOS syscall numbers: it uses Linux x86_64 syscall IDs directly:
;   write = 1, exit = 60
; Running this from the shell (`run LINUX.ELF`) proves the Linux personality
; dispatch path and private low user address space work.
; =============================================================================

BITS 64
%define LINUX_SYS_WRITE  1
%define LINUX_SYS_EXIT   60

global _start
section .text

_start:
    mov  rax, LINUX_SYS_WRITE
    mov  rdi, 1
    lea  rsi, [rel msg]
    mov  rdx, msg_len
    syscall

    mov  rax, LINUX_SYS_EXIT
    xor  rdi, rdi
    syscall

.hang:
    jmp  .hang

msg:
    db "Hello from a Linux ABI ELF64 using Linux syscalls on NexusOS!", 13, 10
    db "write=1 and exit=60 translated by the Linux personality.", 13, 10
msg_len equ $ - msg
