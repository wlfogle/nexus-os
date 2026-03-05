#ifndef USERLAND_LIBC_H
#define USERLAND_LIBC_H

#include <stdint.h>
#include <stddef.h>

/* Standard I/O */
int putchar(int c);
int getchar(void);
int puts(const char *s);
char *gets(char *s);
int printf(const char *fmt, ...);

/* String functions */
size_t strlen(const char *s);
char *strcpy(char *dst, const char *src);
int strcmp(const char *a, const char *b);
char *strchr(const char *s, int c);
char *strstr(const char *haystack, const char *needle);
char *strcat(char *dst, const char *src);
void *memset(void *s, int c, size_t n);
void *memcpy(void *dst, const void *src, size_t n);

/* Memory management */
void *malloc(size_t size);
void free(void *ptr);

/* Process/syscall wrappers */
void exit(int code);
int write(int fd, const char *buf, int count);
int read(int fd, char *buf, int count);
int fork(void);
int wait(int *status);
int sleep(uint32_t ms);
int open(const char *path, int flags);
int close(int fd);
int mkdir(const char *path);
int unlink(const char *path);
int lseek(int fd, int offset, int whence);
int exec(const char *filename, char *const argv[]);
int signal(int signum, uint32_t handler);

/* Utilities */
int atoi(const char *s);
void abort(void);

/* File descriptor constants */
#define STDIN_FILENO  0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

/* Open flags (basic) */
#define O_RDONLY 0
#define O_WRONLY 1
#define O_RDWR   2
#define O_CREAT  0x40
#define O_TRUNC  0x200

/* Seek modes */
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

/* Dummy FILE type for fflush */
typedef int FILE;
#define stdout ((FILE *)1)
#define stderr ((FILE *)2)

int fflush(FILE *stream);

#endif /* USERLAND_LIBC_H */
