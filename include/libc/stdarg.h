#ifndef LIBC_STDARG_H
#define LIBC_STDARG_H

/* Standard varargs for i386 */
typedef unsigned char *va_list;

#define va_start(ap, last) \
    ((ap) = (va_list)&(last) + sizeof(last))

#define va_end(ap) \
    ((ap) = 0)

#define va_arg(ap, type) \
    (*(type *)((ap) += sizeof(type), (ap) - sizeof(type)))

#endif /* LIBC_STDARG_H */
