#include "libc.h"
#include <stdarg.h>

/* ===== Syscall Interface ===== */

/* Inline syscall wrapper - calls INT 0x80 with syscall number in EAX */
static inline int32_t _syscall5(uint32_t num, uint32_t a, uint32_t b, uint32_t c, uint32_t d, uint32_t e)
{
    int32_t ret;
    asm volatile(
        "int $0x80"
        : "=a" (ret)
        : "a" (num), "b" (a), "c" (b), "d" (c), "S" (d), "D" (e)
        : "memory"
    );
    return ret;
}

static inline int32_t _syscall4(uint32_t num, uint32_t a, uint32_t b, uint32_t c, uint32_t d)
{
    return _syscall5(num, a, b, c, d, 0);
}

static inline int32_t _syscall3(uint32_t num, uint32_t a, uint32_t b, uint32_t c)
{
    return _syscall5(num, a, b, c, 0, 0);
}

static inline int32_t _syscall2(uint32_t num, uint32_t a, uint32_t b)
{
    return _syscall5(num, a, b, 0, 0, 0);
}

static inline int32_t _syscall1(uint32_t num, uint32_t a)
{
    return _syscall5(num, a, 0, 0, 0, 0);
}

static inline int32_t _syscall0(uint32_t num)
{
    return _syscall5(num, 0, 0, 0, 0, 0);
}

/* ===== Syscall definitions ===== */
#define SYS_EXIT   1
#define SYS_WRITE  2
#define SYS_READ   3
#define SYS_FORK   4
#define SYS_WAIT   5
#define SYS_SLEEP  6
#define SYS_OPEN   7
#define SYS_CLOSE  8
#define SYS_MKDIR  9
#define SYS_UNLINK 10
#define SYS_LSEEK  11
#define SYS_EXEC   12
#define SYS_SIGNAL 13

void exit(int code)
{
    _syscall1(SYS_EXIT, code);
    while(1);  /* Should not reach here, but safeguard */
}

int write(int fd, const char *buf, int count)
{
    return _syscall3(SYS_WRITE, fd, (uint32_t)buf, count);
}

int read(int fd, char *buf, int count)
{
    return _syscall3(SYS_READ, fd, (uint32_t)buf, count);
}

int fork(void)
{
    return _syscall0(SYS_FORK);
}

int wait(int *status)
{
    return _syscall1(SYS_WAIT, (uint32_t)status);
}

int sleep(uint32_t ms)
{
    return _syscall1(SYS_SLEEP, ms);
}

int open(const char *path, int flags)
{
    return _syscall2(SYS_OPEN, (uint32_t)path, flags);
}

int close(int fd)
{
    return _syscall1(SYS_CLOSE, fd);
}

int mkdir(const char *path)
{
    return _syscall1(SYS_MKDIR, (uint32_t)path);
}

int unlink(const char *path)
{
    return _syscall1(SYS_UNLINK, (uint32_t)path);
}

int lseek(int fd, int offset, int whence)
{
    return _syscall3(SYS_LSEEK, fd, (uint32_t)offset, whence);
}

int exec(const char *filename, char *const argv[])
{
    return _syscall2(SYS_EXEC, (uint32_t)filename, (uint32_t)argv);
}

int signal(int signum, uint32_t handler)
{
    return _syscall2(SYS_SIGNAL, signum, handler);
}

/* ===== Standard I/O ===== */

int putchar(int c)
{
    char ch = (char)c;
    return write(STDOUT_FILENO, &ch, 1);
}

int getchar(void)
{
    char ch;
    if (read(STDIN_FILENO, &ch, 1) == 1) {
        return (unsigned char)ch;
    }
    return -1;
}

int puts(const char *s)
{
    int count = 0;
    while (*s) {
        if (putchar(*s++) < 0) return -1;
        count++;
    }
    if (putchar('\n') < 0) return -1;
    return count + 1;
}

char *gets(char *s)
{
    char *p = s;
    int c;
    while ((c = getchar()) != '\n' && c != -1) {
        *p++ = (char)c;
    }
    *p = '\0';
    return s;
}

/* Basic printf with support for %s, %d, %x, %c */
int printf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    
    int count = 0;
    while (*fmt) {
        if (*fmt == '%' && *(fmt + 1)) {
            fmt++;
            switch (*fmt) {
                case 's': {
                    const char *s = va_arg(ap, const char *);
                    if (s) {
                        while (*s) {
                            putchar(*s++);
                            count++;
                        }
                    }
                    break;
                }
                case 'd': {
                    int n = va_arg(ap, int);
                    if (n < 0) {
                        putchar('-');
                        count++;
                        n = -n;
                    }
                    char buf[20], *p = buf + sizeof(buf) - 1;
                    *p = '\0';
                    if (n == 0) {
                        p--;
                        *p = '0';
                    } else {
                        while (n > 0) {
                            p--;
                            *p = '0' + (n % 10);
                            n /= 10;
                        }
                    }
                    while (*p) {
                        putchar(*p++);
                        count++;
                    }
                    break;
                }
                case 'x': {
                    unsigned int n = va_arg(ap, unsigned int);
                    const char *hex = "0123456789abcdef";
                    char buf[16], *p = buf + sizeof(buf) - 1;
                    *p = '\0';
                    if (n == 0) {
                        p--;
                        *p = '0';
                    } else {
                        while (n > 0) {
                            p--;
                            *p = hex[n & 0xf];
                            n >>= 4;
                        }
                    }
                    while (*p) {
                        putchar(*p++);
                        count++;
                    }
                    break;
                }
                case 'c': {
                    int c = va_arg(ap, int);
                    putchar(c);
                    count++;
                    break;
                }
                default:
                    putchar(*fmt);
                    count++;
                    break;
            }
            fmt++;
        } else {
            putchar(*fmt++);
            count++;
        }
    }
    
    va_end(ap);
    return count;
}

/* ===== String functions ===== */

size_t strlen(const char *s)
{
    size_t n = 0;
    while (*s++) n++;
    return n;
}

char *strcpy(char *dst, const char *src)
{
    char *d = dst;
    while ((*d++ = *src++));
    return dst;
}

int strcmp(const char *a, const char *b)
{
    while (*a && *a == *b) {
        a++;
        b++;
    }
    return (unsigned char)*a - (unsigned char)*b;
}

char *strchr(const char *s, int c)
{
    while (*s && *s != c) s++;
    return *s == c ? (char *)s : NULL;
}

char *strstr(const char *haystack, const char *needle)
{
    size_t nlen = strlen(needle);
    while (*haystack) {
        if (!memcmp(haystack, needle, nlen)) {
            return (char *)haystack;
        }
        haystack++;
    }
    return NULL;
}

char *strcat(char *dst, const char *src)
{
    char *d = dst + strlen(dst);
    while ((*d++ = *src++));
    return dst;
}

void *memset(void *s, int c, size_t n)
{
    unsigned char *p = (unsigned char *)s;
    while (n--) {
        *p++ = (unsigned char)c;
    }
    return s;
}

void *memcpy(void *dst, const void *src, size_t n)
{
    unsigned char *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;
    while (n--) {
        *d++ = *s++;
    }
    return dst;
}

int memcmp(const void *a, const void *b, size_t n)
{
    const unsigned char *ca = (const unsigned char *)a;
    const unsigned char *cb = (const unsigned char *)b;
    while (n--) {
        int cmp = *ca - *cb;
        if (cmp) return cmp;
        ca++;
        cb++;
    }
    return 0;
}

/* ===== Memory management ===== */

/* Simple bump allocator - just allocate from a fixed buffer */
#define HEAP_SIZE (64 * 1024)
static char heap[HEAP_SIZE];
static size_t heap_used = 0;

void *malloc(size_t size)
{
    if (heap_used + size > HEAP_SIZE) {
        return NULL;
    }
    void *ptr = &heap[heap_used];
    heap_used += size;
    return ptr;
}

void free(void *ptr)
{
    /* Bump allocator doesn't free - just a stub */
    (void)ptr;
}

/* ===== Utilities ===== */

int atoi(const char *s)
{
    int n = 0;
    int neg = 0;
    
    while (*s && (*s == ' ' || *s == '\t')) s++;
    
    if (*s == '-') {
        neg = 1;
        s++;
    } else if (*s == '+') {
        s++;
    }
    
    while (*s && *s >= '0' && *s <= '9') {
        n = n * 10 + (*s++ - '0');
    }
    
    return neg ? -n : n;
}

void abort(void)
{
    exit(-1);
}

int fflush(FILE *stream)
{
    /* No buffering in our simple printf, so nothing to flush */
    (void)stream;
    return 0;
}
