; NexusOS Ring-3 Shell — shell_init.asm
; =============================================================================
; Assembled with: nasm -f bin -o shell_init.bin shell_init.asm
; Produces a flat, position-independent 64-bit binary.
; The kernel copies the binary to USER_CODE_BASE and jumps to byte 0.
;
; Syscall ABI (matches kernel/src/syscall/mod.rs):
;   rax = syscall number
;   rdi = arg1,  rsi = arg2,  rdx = arg3
;   Return value in rax.
;   After syscall: rcx and r11 are clobbered by the CPU (sysretq).
;                  rdi/rsi/rdx are clobbered by kernel argument shuffling.
;   All callee-saved registers (rbx, rbp, r12-r15) are preserved by the
;   kernel's entry stub (push/pop in _nexus_syscall_entry).
;
; Syscalls used:
;   SYS_EXIT=1   exit(code)
;   SYS_WRITE=2  write(fd=1, buf, len) -> bytes_written
;   SYS_GETPID=3 -> pid
;   SYS_SLEEP=9  sleep(ticks)
;   SYS_READ_CHAR=13  -> u8 char (blocking, woken by IRQ1)
;
; Memory layout:
;   Code page (read+exec, USER_CODE_BASE):   this binary
;   Stack page (read+write, USER_STACK_TOP): cmd_buf, num_buf, call frames
;
; Register conventions in _start:
;   r14  = cmd_buf base pointer (stack, persistent across calls)
;   r15  = current input cursor / length (reset each line)
;   Both are callee-saved by the x86_64 ABI and are not modified by any
;   called function in this binary.
; =============================================================================

BITS 64

; ── Syscall numbers ──────────────────────────────────────────────────────────
%define SYS_EXIT       1
%define SYS_WRITE      2
%define SYS_GETPID     3
%define SYS_SLEEP      9
%define SYS_READ_CHAR  13

; ── Buffer sizes (must both fit on one 4 096-byte stack page) ────────────────
%define CMD_BUF_SIZE   256
%define NUM_BUF_SIZE    32

; =============================================================================
; _start — ring-3 entry point.  RSP = USER_STACK_TOP on first call.
; =============================================================================
_start:
    ; Allocate two mutable buffers on the stack.
    ;   [rsp + 0 .. rsp+CMD_BUF_SIZE)            -> cmd_buf
    ;   [rsp + CMD_BUF_SIZE .. rsp+CMD_BUF_SIZE+NUM_BUF_SIZE)  -> num_buf
    sub  rsp, CMD_BUF_SIZE + NUM_BUF_SIZE
    mov  r14, rsp                       ; r14 = &cmd_buf (persistent)

    ; ── Boot banner ──────────────────────────────────────────────────────────
    lea  rsi, [rel str_banner]
    mov  rdx, str_banner_len
    call fn_write

; ── Main REPL ────────────────────────────────────────────────────────────────
.repl:
    lea  rsi, [rel str_prompt]
    mov  rdx, str_prompt_len
    call fn_write

    xor  r15, r15                       ; r15 = input length = 0

.rdch:
    ; Block until a keystroke arrives (IRQ1 → wake_blocked_on_key → scheduler)
    mov  rax, SYS_READ_CHAR
    syscall                             ; returns char in al (low byte of rax)

    cmp  al, 13                         ; Carriage Return
    je   .eol
    cmp  al, 10                         ; Line Feed
    je   .eol
    cmp  al, 8                          ; Backspace (ASCII BS)
    je   .bs
    cmp  al, 127                        ; DEL (terminal sends this for Backspace)
    je   .bs

    ; Reject everything outside printable ASCII 0x20–0x7E
    cmp  al, 0x20
    jl   .rdch
    cmp  al, 0x7E
    jg   .rdch

    ; Drop character if buffer is full (leave one byte for the null terminator)
    cmp  r15, CMD_BUF_SIZE - 1
    jge  .rdch

    ; Store and echo the character
    mov  [r14 + r15], al
    inc  r15
    lea  rsi, [r14 + r15 - 1]          ; address of the stored character
    mov  rdx, 1
    call fn_write
    jmp  .rdch

.bs:
    cmp  r15, 0
    je   .rdch                          ; nothing to erase
    dec  r15
    lea  rsi, [rel str_bs_seq]
    mov  rdx, 3                         ; BS SPACE BS
    call fn_write
    jmp  .rdch

.eol:
    ; Null-terminate the command string
    mov  byte [r14 + r15], 0
    ; Echo newline
    lea  rsi, [rel str_nl]
    mov  rdx, 1
    call fn_write
    ; Dispatch: rdi = cmd_buf, rsi = num_buf (immediately after cmd_buf)
    mov  rdi, r14
    lea  rsi, [r14 + CMD_BUF_SIZE]
    call fn_dispatch
    jmp  .repl

; =============================================================================
; fn_write — write rsi[0..rdx) to fd 1 (serial / console)
; Clobbers: rax only.  All other registers preserved (push/pop rdi).
; =============================================================================
fn_write:
    push rdi
    mov  rax, SYS_WRITE
    mov  rdi, 1
    syscall
    pop  rdi
    ret

; =============================================================================
; fn_strlen — length of null-terminated string
; In:  rsi = string pointer
; Out: rcx = byte count (excluding null terminator)
; Clobbers rcx only.
; =============================================================================
fn_strlen:
    xor  rcx, rcx
.lp:
    cmp  byte [rsi + rcx], 0
    je   .dn
    inc  rcx
    jmp  .lp
.dn:
    ret

; =============================================================================
; fn_match — test whether the input at rdi starts with keyword at rsi
;            (rdx bytes long) and is followed by NUL or SPACE.
; In:  rdi = input string
;      rsi = keyword literal (not null-terminated)
;      rdx = keyword length in bytes
; Out: rax = 0 (matched), 1 (not matched)
; Preserves all registers except rax (push/pop rbx, rcx, rdi, rsi).
; =============================================================================
fn_match:
    push rbx
    push rcx
    push rdi
    push rsi
    mov  rcx, rdx               ; rcx = comparison count
    repe cmpsb                  ; advance rdi/rsi while bytes match
    jne  .no                    ; mismatch
    ; All rdx bytes matched.  rdi now points to input[keyword_len].
    ; Accept only if that byte is NUL (end of word) or SPACE (followed by args).
    cmp  byte [rdi], 0
    je   .yes
    cmp  byte [rdi], ' '
    je   .yes
.no:
    pop  rsi
    pop  rdi
    pop  rcx
    pop  rbx
    mov  rax, 1
    ret
.yes:
    pop  rsi
    pop  rdi
    pop  rcx
    pop  rbx
    xor  rax, rax
    ret

; =============================================================================
; fn_u64dec — format a u64 as a decimal ASCII string
; In:  rax = number to format
;      rsi = output buffer (at least 20 bytes, writable)
; Out: rdx = number of bytes written to buffer
; Clobbers rax, rdx.  Preserves rbx, rcx, r8, r9 (push/pop).
; =============================================================================
fn_u64dec:
    push rbx
    push rcx
    push r8
    push r9
    mov  rbx, rsi               ; rbx = output buffer
    xor  r9, r9                 ; r9  = digit count

    ; Special case: input is zero
    test rax, rax
    jnz  .digs
    mov  byte [rbx], '0'
    mov  rdx, 1
    jmp  .ret

.digs:
    ; Extract digits in least-significant-first order into buffer
    test rax, rax
    jz   .rev
    xor  rdx, rdx               ; zero high half of dividend before div
    mov  r8, 10
    div  r8                     ; rax = quotient, rdx = remainder
    add  dl, '0'
    mov  [rbx + r9], dl
    inc  r9
    jmp  .digs

.rev:
    ; Reverse the digit array in [rbx, rbx + r9) in-place
    xor  r8, r8                 ; r8  = left index
    mov  rcx, r9
    dec  rcx                    ; rcx = right index
.rvlp:
    cmp  r8, rcx
    jge  .rvdn
    movzx  eax, byte [rbx + r8]    ; al = byte at left  (no high-reg, no REX clash)
    movzx  edx, byte [rbx + rcx]   ; dl = byte at right
    mov  [rbx + r8], dl            ; store right → left
    mov  [rbx + rcx], al           ; store left  → right
    inc  r8
    dec  rcx
    jmp  .rvlp

.rvdn:
    mov  rdx, r9                ; return digit count

.ret:
    pop  r9
    pop  r8
    pop  rcx
    pop  rbx
    ret

; =============================================================================
; fn_dispatch — parse and execute one command line
; In:  rdi = null-terminated command string
;      rsi = scratch buffer for number formatting (NUM_BUF_SIZE bytes, writable)
; Preserves rbx, r12, r13 (push/pop).
; Does NOT modify r14 or r15 (the _start-level registers).
; =============================================================================
fn_dispatch:
    push rbx
    push r12
    push r13
    mov  r12, rdi               ; r12 = command string
    mov  r13, rsi               ; r13 = num_buf

    ; ── Skip leading whitespace ───────────────────────────────────────────────
.sp:
    cmp  byte [r12], ' '
    jne  .nsp
    inc  r12
    jmp  .sp
.nsp:
    ; Empty line after stripping — just re-prompt
    cmp  byte [r12], 0
    je   .fin

    ; ── help ─────────────────────────────────────────────────────────────────
    mov  rdi, r12
    lea  rsi, [rel kw_help]
    mov  rdx, 4
    call fn_match
    test rax, rax
    jnz  .nh
    lea  rsi, [rel str_help]
    mov  rdx, str_help_len
    call fn_write
    jmp  .fin
.nh:

    ; ── version ──────────────────────────────────────────────────────────────
    mov  rdi, r12
    lea  rsi, [rel kw_version]
    mov  rdx, 7
    call fn_match
    test rax, rax
    jnz  .nv
    lea  rsi, [rel str_version]
    mov  rdx, str_version_len
    call fn_write
    jmp  .fin
.nv:

    ; ── uname ────────────────────────────────────────────────────────────────
    mov  rdi, r12
    lea  rsi, [rel kw_uname]
    mov  rdx, 5
    call fn_match
    test rax, rax
    jnz  .nu
    lea  rsi, [rel str_uname]
    mov  rdx, str_uname_len
    call fn_write
    jmp  .fin
.nu:

    ; ── clear ────────────────────────────────────────────────────────────────
    mov  rdi, r12
    lea  rsi, [rel kw_clear]
    mov  rdx, 5
    call fn_match
    test rax, rax
    jnz  .nc
    lea  rsi, [rel str_clear]
    mov  rdx, str_clear_len
    call fn_write
    jmp  .fin
.nc:

    ; ── echo <text> ──────────────────────────────────────────────────────────
    mov  rdi, r12
    lea  rsi, [rel kw_echo]
    mov  rdx, 4
    call fn_match
    test rax, rax
    jnz  .ne
    ; Locate argument: advance past "echo", then skip one optional space
    lea  rsi, [r12 + 4]
    cmp  byte [rsi], ' '
    jne  .earg
    inc  rsi
.earg:
    call fn_strlen              ; rcx = byte count of argument text
    mov  rdx, rcx
    call fn_write               ; print argument (rsi still points to it)
    lea  rsi, [rel str_nl]
    mov  rdx, 1
    call fn_write
    jmp  .fin
.ne:

    ; ── ps ───────────────────────────────────────────────────────────────────
    mov  rdi, r12
    lea  rsi, [rel kw_ps]
    mov  rdx, 2
    call fn_match
    test rax, rax
    jnz  .np
    ; Header
    lea  rsi, [rel str_ps_hdr]
    mov  rdx, str_ps_hdr_len
    call fn_write
    ; Our PID
    mov  rax, SYS_GETPID
    syscall                     ; rax = PID  (rsi/rdx clobbered after syscall)
    mov  rsi, r13               ; reload rsi = num_buf
    call fn_u64dec              ; rdx = digit count, digits written to r13
    mov  rsi, r13               ; reload rsi (fn_u64dec preserves it, but be explicit)
    call fn_write               ; print PID digits
    ; Process name
    lea  rsi, [rel str_ps_row]
    mov  rdx, str_ps_row_len
    call fn_write
    jmp  .fin
.np:

    ; ── reboot ───────────────────────────────────────────────────────────────
    mov  rdi, r12
    lea  rsi, [rel kw_reboot]
    mov  rdx, 6
    call fn_match
    test rax, rax
    jnz  .nr
    lea  rsi, [rel str_reboot]
    mov  rdx, str_reboot_len
    call fn_write
    ; Signal the kernel to exit this process (scheduler drops to idle / reboot)
    mov  rax, SYS_EXIT
    mov  rdi, 0xFF              ; 0xFF = reboot intent
    syscall
    jmp  $                      ; unreachable — kernel marks process Dead
.nr:

    ; ── Unknown command ───────────────────────────────────────────────────────
    lea  rsi, [rel str_unk_pfx]
    mov  rdx, str_unk_pfx_len
    call fn_write
    mov  rsi, r12               ; the unrecognised command text
    call fn_strlen              ; rcx = length
    mov  rdx, rcx
    call fn_write
    lea  rsi, [rel str_nl]
    mov  rdx, 1
    call fn_write

.fin:
    pop  r13
    pop  r12
    pop  rbx
    ret

; =============================================================================
; Read-only data — all referenced via RIP-relative [rel label] addressing.
; These bytes live in the code page (read+exec, no write) which is fine since
; they are never written to; all mutable state lives on the writable stack.
; =============================================================================

str_banner:
    db  "======================================", 13, 10
    db  "  NexusOS v0.6 -- AI-Native OS", 13, 10
    db  "======================================", 13, 10
    db  "Type 'help' for available commands.", 13, 10
    db  13, 10
str_banner_len equ $ - str_banner

str_prompt:
    db  "nexus> "
str_prompt_len equ $ - str_prompt

str_nl:
    db  10

str_bs_seq:
    db  8, ' ', 8                               ; BS SPACE BS — erase-char

str_help:
    db  "Commands:", 13, 10
    db  "  help     - this help text", 13, 10
    db  "  version  - OS version string", 13, 10
    db  "  uname    - system information", 13, 10
    db  "  echo <x> - print argument to screen", 13, 10
    db  "  ps       - list running processes", 13, 10
    db  "  clear    - clear the screen", 13, 10
    db  "  reboot   - exit shell and reboot", 13, 10
str_help_len equ $ - str_help

str_version:
    db  "NexusOS v0.6.0", 13, 10
    db  "Kernel:  nexus-kernel (Rust, Limine boot protocol)", 13, 10
    db  "Phase:   5  -- Ring-3 userspace | IPC | Syscall interface", 13, 10
    db  "Arch:    x86_64", 13, 10
str_version_len equ $ - str_version

str_uname:
    db  "NexusOS nexus-kernel 0.6.0 x86_64", 13, 10
str_uname_len equ $ - str_uname

str_clear:
    db  13, 10, "[screen clear not supported on framebuffer console]", 13, 10
str_clear_len equ $ - str_clear

str_ps_hdr:
    db  "  PID   NAME", 13, 10
    db  "  ---   ----", 13, 10
    db  "  "
str_ps_hdr_len equ $ - str_ps_hdr

str_ps_row:
    db  "      nexus-shell", 13, 10
str_ps_row_len equ $ - str_ps_row

str_reboot:
    db  "System going down for reboot now...", 13, 10
str_reboot_len equ $ - str_reboot

str_unk_pfx:
    db  "nexus: command not found: "
str_unk_pfx_len equ $ - str_unk_pfx

; ── Command keyword literals (length matched, no null terminator needed) ─────
kw_help:    db  "help"
kw_version: db  "version"
kw_uname:   db  "uname"
kw_clear:   db  "clear"
kw_echo:    db  "echo"
kw_ps:      db  "ps"
kw_reboot:  db  "reboot"
