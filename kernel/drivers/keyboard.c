#include "../../include/kernel/keyboard.h"
#include "../../include/kernel/pic.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

#define KB_DATA_PORT   0x60
#define KB_STATUS_PORT 0x64
#define KB_BUF_SIZE    256

/* ------------------------------------------------------------------ */
/* Scan code set 1 → ASCII tables (US QWERTY)                         */
/* ------------------------------------------------------------------ */

/* Unshifted characters */
static const char sc_ascii[128] = {
/* 00 */  0,
/* 01 */  27,          /* ESC            */
/* 02 */  '1','2','3','4','5','6','7','8','9','0','-','=',   /* 02-0D */
/* 0E */  '\b',        /* Backspace      */
/* 0F */  '\t',        /* Tab            */
/* 10 */  'q','w','e','r','t','y','u','i','o','p','[',']',   /* 10-1B */
/* 1C */  '\n',        /* Enter          */
/* 1D */  0,           /* Left Ctrl      */
/* 1E */  'a','s','d','f','g','h','j','k','l',';','\'','`',  /* 1E-29 */
/* 2A */  0,           /* Left Shift     */
/* 2B */  '\\',
/* 2C */  'z','x','c','v','b','n','m',',','.','/',           /* 2C-35 */
/* 36 */  0,           /* Right Shift    */
/* 37 */  '*',         /* Keypad *       */
/* 38 */  0,           /* Left Alt       */
/* 39 */  ' ',         /* Space          */
/* 3A */  0,           /* Caps Lock      */
/* 3B */  0,0,0,0,0,0,0,0,0,0,    /* F1-F10         */
/* 45 */  0,           /* Num Lock       */
/* 46 */  0,           /* Scroll Lock    */
/* 47 */  '7','8','9','-','4','5','6','+','1','2','3','0','.', /* keypad 47-53 */
/* 54 */  0,0,0,0,0,0,0,0,0,0,0,0,
          0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
          0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0  /* 54-7F */
};

/* Shifted characters */
static const char sc_ascii_shift[128] = {
/* 00 */  0,
/* 01 */  27,          /* ESC            */
/* 02 */  '!','@','#','$','%','^','&','*','(',')','_','+',   /* 02-0D */
/* 0E */  '\b',
/* 0F */  '\t',
/* 10 */  'Q','W','E','R','T','Y','U','I','O','P','{','}',   /* 10-1B */
/* 1C */  '\n',
/* 1D */  0,
/* 1E */  'A','S','D','F','G','H','J','K','L',':','"','~',   /* 1E-29 */
/* 2A */  0,
/* 2B */  '|',
/* 2C */  'Z','X','C','V','B','N','M','<','>','?',           /* 2C-35 */
/* 36 */  0,
/* 37 */  '*',
/* 38 */  0,
/* 39 */  ' ',
/* 3A */  0,
/* 3B */  0,0,0,0,0,0,0,0,0,0,
/* 45 */  0,
/* 46 */  0,
/* 47 */  '7','8','9','-','4','5','6','+','1','2','3','0','.',
/* 54 */  0,0,0,0,0,0,0,0,0,0,0,0,
          0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
          0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
};

/* ------------------------------------------------------------------ */
/* Driver state                                                        */
/* ------------------------------------------------------------------ */

static volatile int shift_down = 0;
static volatile int ctrl_down  = 0;
static volatile int alt_down   = 0;
static volatile int caps_lock  = 0;

/* Ring buffer (written by interrupt handler, read by kernel/user) */
static volatile char kb_buf[KB_BUF_SIZE];
static volatile int  kb_head = 0;   /* next write position */
static volatile int  kb_tail = 0;   /* next read position  */

/* ------------------------------------------------------------------ */
/* I/O helper                                                          */
/* ------------------------------------------------------------------ */

static inline uint8_t inb(uint16_t port)
{
    uint8_t ret;
    __asm__ volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

/* ------------------------------------------------------------------ */
/* Ring buffer operations                                              */
/* ------------------------------------------------------------------ */

static void kb_buf_push(char c)
{
    int next = (kb_head + 1) % KB_BUF_SIZE;
    if (next != kb_tail) {          /* drop if full */
        kb_buf[kb_head] = c;
        kb_head = next;
    }
}

/* ------------------------------------------------------------------ */
/* Public API                                                          */
/* ------------------------------------------------------------------ */

void keyboard_init(void)
{
    int drain = 512;

    /* Drain any stale bytes already in the controller output buffer */
    while ((inb(KB_STATUS_PORT) & 0x01) && drain-- > 0) {
        inb(KB_DATA_PORT);
    }

    shift_down = 0;
    ctrl_down  = 0;
    alt_down   = 0;
    caps_lock  = 0;
    kb_head    = 0;
    kb_tail    = 0;

    pic_enable_irq(1);          /* Unmask IRQ1 in the 8259 PIC */
    serial_puts("[OK] Keyboard initialized (PS/2, IRQ1)\n");
}

void keyboard_interrupt(void)
{
    uint8_t scan = inb(KB_DATA_PORT);

    /* Key release: top bit set */
    if (scan & 0x80) {
        uint8_t released = scan & 0x7F;
        if (released == 0x2A || released == 0x36) shift_down = 0;
        if (released == 0x1D)                      ctrl_down  = 0;
        if (released == 0x38)                      alt_down   = 0;
        return;
    }

    /* Key press: handle modifier keys first */
    if (scan == 0x2A || scan == 0x36) { shift_down = 1; return; }
    if (scan == 0x1D)                  { ctrl_down  = 1; return; }
    if (scan == 0x38)                  { alt_down   = 1; return; }
    if (scan == 0x3A)                  { caps_lock ^= 1; return; }

    /* Ignore extended/unknown scan codes */
    if (scan >= 128) return;

    /* Determine effective shift state; caps lock only affects letters */
    int use_shift = shift_down;
    char base = sc_ascii[scan];
    if (caps_lock && base >= 'a' && base <= 'z') {
        use_shift ^= 1;
    }

    char c = use_shift ? sc_ascii_shift[scan] : sc_ascii[scan];
    if (c == 0) return;             /* unmapped key */

    kb_buf_push(c);
}

int keyboard_getchar(void)
{
    if (kb_tail == kb_head) return -1;      /* buffer empty */
    char c    = kb_buf[kb_tail];
    kb_tail   = (kb_tail + 1) % KB_BUF_SIZE;
    return (int)(unsigned char)c;
}

int keyboard_has_input(void)
{
    return kb_head != kb_tail;
}
