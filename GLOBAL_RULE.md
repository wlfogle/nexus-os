# Global Rule: 100% Complete Code Only

## ABSOLUTE REQUIREMENT

**ALL code must be 100% complete and fully functional before any commit.**

## Zero Tolerances

- **NO TODO, FIXME, XXX, HACK, stub, or unimplemented comments**
- **NO incomplete functions or partial implementations**
- **NO zombie/dead code paths**
- **NO placeholder stubs or mock handlers**

## Requirements for Every Code Change

### Implementation
- EVERY function must have a complete implementation
- EVERY function must have full error handling
- EVERY function must have parameter validation
- EVERY function must handle all edge cases

### Testing & Verification
- Code MUST compile without errors
- Code MUST run correctly
- Code MUST be tested and verified to work
- Grep for TODO/FIXME/stub/unimplemented MUST return ZERO matches

### Pre-Commit Checklist
1. Audit all code for incomplete patterns
2. Verify no TODO/FIXME/stub comments exist
3. Verify all functions have complete bodies
4. Verify all error paths are handled
5. Verify all parameters are validated
6. Test that code actually works
7. Verify binary compiles and runs
8. Only then commit

## Examples

### ❌ REJECTED - Incomplete
```c
int read_file(const char *path) {
    // TODO: implement file reading
    return -1;
}
```

### ❌ REJECTED - Stub
```asm
.globl interrupt_handler
interrupt_handler:
    # stub handler
    iret
```

### ✅ ACCEPTED - Complete
```c
int read_file(const char *path) {
    if (!path) return -EINVAL;
    
    int fd = open(path, O_RDONLY);
    if (fd < 0) return fd;
    
    char buffer[4096];
    int bytes = read(fd, buffer, sizeof(buffer));
    
    if (bytes < 0) {
        close(fd);
        return bytes;
    }
    
    close(fd);
    return bytes;
}
```

### ✅ ACCEPTED - Production ISR
```asm
.globl interrupt_handler
interrupt_handler:
    pushal
    push %ds
    push %es
    push %fs
    push %gs
    
    mov $0x10, %eax
    mov %eax, %ds
    mov %eax, %es
    mov %eax, %fs
    mov %eax, %gs
    
    call handle_interrupt
    
    pop %gs
    pop %fs
    pop %es
    pop %ds
    popal
    iret
```

## Enforcement

This rule applies to **ALL work** on **ALL projects**. Never compromise on this standard.

Before any commit or code review, verify:
1. No stub code patterns exist
2. No TODO/FIXME markers found
3. All functions fully implemented
4. Code tested and working
5. Binary compiles without errors
