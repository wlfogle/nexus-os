#include "../../include/kernel/kshell.h"
#include "../../include/kernel/console.h"
#include "../../include/kernel/keyboard.h"
#include "../../include/kernel/task.h"
#include "../../include/kernel/timer.h"
#include "../../include/kernel/pmem.h"
#include "../../include/kernel/vga.h"
#include <string.h>

#define CMD_BUF_SIZE  256
#define MAX_ARGS      16

/* ------------------------------------------------------------------ */
/* Helpers                                                             */
/* ------------------------------------------------------------------ */

static int streq(const char *a, const char *b)
{
    return strcmp(a, b) == 0;
}

/* Simple strtok-style splitter (modifies cmd in-place) */
static int parse_args(char *cmd, char *argv[])
{
    int argc = 0;
    char *p = cmd;

    while (*p && (*p == ' ' || *p == '\t')) p++;

    while (*p && argc < MAX_ARGS - 1) {
        argv[argc++] = p;
        while (*p && *p != ' ' && *p != '\t') p++;
        if (*p) {
            *p++ = '\0';
            while (*p && (*p == ' ' || *p == '\t')) p++;
        }
    }
    argv[argc] = NULL;
    return argc;
}

/* ------------------------------------------------------------------ */
/* Built-in commands                                                   */
/* ------------------------------------------------------------------ */

static void cmd_help(void)
{
    console_puts("NexusOS Kernel Shell -- built-in commands:\n");
    console_puts("  help     - Show this message\n");
    console_puts("  echo     - Print arguments\n");
    console_puts("  clear    - Clear the screen\n");
    console_puts("  ps       - List tasks\n");
    console_puts("  uptime   - Show system uptime\n");
    console_puts("  mem      - Show memory statistics\n");
    console_puts("  version  - Print NexusOS version\n");
    console_puts("  reboot   - Reboot the system\n");
}

static void cmd_echo(int argc, char *argv[])
{
    for (int i = 1; i < argc; i++) {
        if (i > 1) console_putchar(' ');
        console_puts(argv[i]);
    }
    console_putchar('\n');
}

static void cmd_clear(void)
{
    vga_clear();
}

static const char *state_name(task_state_t s)
{
    switch (s) {
    case TASK_READY:   return "READY";
    case TASK_RUNNING: return "RUNNING";
    case TASK_BLOCKED: return "BLOCKED";
    case TASK_DEAD:    return "DEAD";
    default:           return "?";
    }
}

static void cmd_ps(void)
{
    int count = task_get_count();
    console_puts("PID  STATE    PRI\n");
    for (int i = 0; i < count; i++) {
        struct task *t = task_get(i);
        if (t) {
            console_printf("%d\t%s\t%d\n", t->id, state_name(t->state), t->priority);
        }
    }
}

static void cmd_uptime(void)
{
    int ticks = timer_get_ticks();
    /* Timer fires at 100 Hz (10 ms per tick) */
    int secs = ticks / 100;
    int frac = ticks % 100;
    console_printf("Uptime: %d.%d%d seconds (%d ticks)\n",
                   secs, frac / 10, frac % 10, ticks);
}

static void cmd_mem(void)
{
    uint32_t free_pages  = pmem_get_free_pages();
    uint32_t total_pages = pmem_get_total_pages();
    uint32_t used_pages  = total_pages - free_pages;

    console_printf("Physical memory:\n");
    console_printf("  Total : %u KB (%u pages)\n", total_pages * 4, total_pages);
    console_printf("  Used  : %u KB (%u pages)\n", used_pages  * 4, used_pages);
    console_printf("  Free  : %u KB (%u pages)\n", free_pages  * 4, free_pages);
}

static void cmd_version(void)
{
    console_puts("NexusOS v0.1.0  (Phases 0-13)\n");
    console_puts("x86 32-bit, multiboot, kernel-mode shell\n");
}

static void cmd_reboot(void)
{
    console_puts("Rebooting...\n");
    /* Triple-fault: load a zero-length IDT and trigger an interrupt */
    struct { uint16_t limit; uint32_t base; } __attribute__((packed)) null_idt = {0, 0};
    __asm__ volatile("lidt %0" : : "m"(null_idt));
    __asm__ volatile("int $3");
    /* Should never reach here */
    while (1) __asm__ volatile("hlt");
}

/* ------------------------------------------------------------------ */
/* Command dispatch                                                    */
/* ------------------------------------------------------------------ */

static void dispatch(char *line)
{
    char *argv[MAX_ARGS];
    int argc = parse_args(line, argv);
    if (argc == 0) return;

    if      (streq(argv[0], "help"))    cmd_help();
    else if (streq(argv[0], "echo"))    cmd_echo(argc, argv);
    else if (streq(argv[0], "clear"))   cmd_clear();
    else if (streq(argv[0], "ps"))      cmd_ps();
    else if (streq(argv[0], "uptime"))  cmd_uptime();
    else if (streq(argv[0], "mem"))     cmd_mem();
    else if (streq(argv[0], "version")) cmd_version();
    else if (streq(argv[0], "reboot"))  cmd_reboot();
    else {
        console_printf("unknown command: %s\n", argv[0]);
    }
}

/* ------------------------------------------------------------------ */
/* Main shell loop                                                     */
/* ------------------------------------------------------------------ */

void kshell_run(void)
{
    char buf[CMD_BUF_SIZE];
    int  pos = 0;

    console_set_color(VGA_LIGHT_GREEN, VGA_BLACK);
    console_puts("\n===== NexusOS Kernel Shell =====\n");
    console_puts("Type 'help' for available commands.\n\n");
    console_set_color(VGA_LIGHT_GREY, VGA_BLACK);

    console_puts("nexus> ");

    while (1) {
        /* Wait for the next interrupt (keyboard, timer, …) */
        __asm__ volatile("hlt");

        /* Drain all pending characters from the keyboard buffer */
        while (keyboard_has_input()) {
            int ch = keyboard_getchar();
            if (ch < 0) break;

            if (ch == '\n') {
                console_putchar('\n');
                buf[pos] = '\0';
                dispatch(buf);
                pos = 0;
                console_puts("nexus> ");
            } else if (ch == '\b') {
                if (pos > 0) {
                    pos--;
                    console_putchar('\b');
                }
            } else {
                if (pos < CMD_BUF_SIZE - 1) {
                    buf[pos++] = (char)ch;
                    console_putchar((char)ch);
                }
            }
        }
    }
}
