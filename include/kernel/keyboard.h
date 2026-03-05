#ifndef KERNEL_KEYBOARD_H
#define KERNEL_KEYBOARD_H

/* Initialize the PS/2 keyboard and unmask IRQ1 */
void keyboard_init(void);

/* Called from the IRQ1 handler — reads scan code, converts to ASCII,
   and stores in the input ring buffer */
void keyboard_interrupt(void);

/* Read one character from the ring buffer.
   Returns the ASCII value, or -1 if the buffer is empty. */
int keyboard_getchar(void);

/* Returns 1 if there is at least one character waiting, 0 otherwise */
int keyboard_has_input(void);

#endif /* KERNEL_KEYBOARD_H */
