; NexusOS reference user program — hello.asm
; =============================================================================
; A real, static ELF64 executable (not a flat blob) that the ring-3 shell can
; load from the FAT32 disk via `run HELLO.ELF` (SYS_EXEC).
;
; Build (see kernel/build.rs):
;   nasm -f elf64 -o hello.o hello.asm
;   ld -N -Ttext=0x8040000000 -e _start -o hello.elf hello.o
;
; Linked at 0x80_4000_0000 (PML4[1], PDPT[1]) so it never collides with the
; resident shell at 0x80_0000_0000.  Uses only the stable NexusOS syscall ABI:
;   rax = number, rdi/rsi/rdx = args, return in rax.
;
;   SYS_WRITE = 2   write(fd=1, buf, len)
;   SYS_EXIT  = 1   exit(code)
; =============================================================================

BITS 64
%define SYS_EXIT   1
%define SYS_WRITE  2

global _start
section .text

_start:
    ; write(1, msg, msg_len)
    mov  rax, SYS_WRITE
    mov  rdi, 1
    lea  rsi, [rel msg]
    mov  rdx, msg_len
    syscall

    ; exit(0)
    mov  rax, SYS_EXIT
    xor  rdi, rdi
    syscall

.hang:
    jmp  .hang                      ; unreachable — kernel marks us Dead

msg:
    db  "Hello from a real ELF64 program running in ring 3 on NexusOS!", 13, 10
    db  "Loaded from FAT32 by SYS_EXEC. Phase 6 program execution works.", 13, 10
msg_len equ $ - msg
