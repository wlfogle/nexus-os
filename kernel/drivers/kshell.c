#include "../../include/kernel/kshell.h"
#include "../../include/kernel/console.h"
#include "../../include/kernel/keyboard.h"
#include "../../include/kernel/task.h"
#include "../../include/kernel/timer.h"
#include "../../include/kernel/pmem.h"
#include "../../include/kernel/vga.h"
#include "../fs/vfs.h"
#include <string.h>

#define CMD_BUF_SIZE  256
#define MAX_ARGS      16
#define MAX_DYN_CMDS  64

/* ------------------------------------------------------------------ */
/* Dynamic command registry                                            */
/* ------------------------------------------------------------------ */

typedef struct {
    const char   *name;
    const char   *desc;
    kshell_cmd_fn fn;
    int           in_use;
} dyn_cmd_t;

static dyn_cmd_t dyn_cmds[MAX_DYN_CMDS];

int kshell_register_command(const char *name, const char *desc, kshell_cmd_fn fn)
{
    if (!name || !fn) return -1;
    for (int i = 0; i < MAX_DYN_CMDS; i++) {
        if (!dyn_cmds[i].in_use) {
            dyn_cmds[i].name   = name;
            dyn_cmds[i].desc   = desc;
            dyn_cmds[i].fn     = fn;
            dyn_cmds[i].in_use = 1;
            return 0;
        }
    }
    return -1;
}

int kshell_unregister_command(const char *name)
{
    if (!name) return -1;
    for (int i = 0; i < MAX_DYN_CMDS; i++) {
        if (dyn_cmds[i].in_use && strcmp(dyn_cmds[i].name, name) == 0) {
            dyn_cmds[i].in_use = 0;
            return 0;
        }
    }
    return -1;
}

static dyn_cmd_t *find_dyn_cmd(const char *name)
{
    for (int i = 0; i < MAX_DYN_CMDS; i++) {
        if (dyn_cmds[i].in_use && strcmp(dyn_cmds[i].name, name) == 0) {
            return &dyn_cmds[i];
        }
    }
    return NULL;
}

/* ------------------------------------------------------------------ */
/* Helpers                                                             */
/* ------------------------------------------------------------------ */

static int streq(const char *a, const char *b)
{
    return strcmp(a, b) == 0;
}

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
/* Built-in: system commands                                           */
/* ------------------------------------------------------------------ */

static void cmd_help(void)
{
    console_puts("NexusOS Kernel Shell\n");
    console_puts("\nSystem:\n");
    console_puts("  help      Show this message\n");
    console_puts("  echo      Print arguments\n");
    console_puts("  clear     Clear the screen\n");
    console_puts("  ps        List tasks\n");
    console_puts("  uptime    Show system uptime\n");
    console_puts("  mem       Show memory statistics\n");
    console_puts("  version   Print NexusOS version\n");
    console_puts("  reboot    Reboot the system\n");
    console_puts("\nFiles:\n");
    console_puts("  ls        List directory contents\n");
    console_puts("  cat       Display file contents\n");
    console_puts("  write     Write text to a file\n");
    console_puts("  touch     Create an empty file\n");
    console_puts("  rm        Remove file or empty dir\n");
    console_puts("  mkdir     Create a directory\n");
    console_puts("  stat      Show file/dir info\n");
    console_puts("  pwd       Print working directory\n");
    console_puts("  cd        Change directory\n");
    console_puts("  cp        Copy a file\n");
    console_puts("  hexdump   Hex dump of a file\n");
    console_puts("  tree      Recursive directory tree\n");
    console_puts("\nPackages:\n");
    console_puts("  nexuspkg  Package manager (try 'nexuspkg help')\n");

    /* List dynamic commands */
    int has_dyn = 0;
    for (int i = 0; i < MAX_DYN_CMDS; i++) {
        if (dyn_cmds[i].in_use) {
            if (!has_dyn) {
                console_puts("\nInstalled:\n");
                has_dyn = 1;
            }
            console_printf("  %-10s%s\n", dyn_cmds[i].name,
                           dyn_cmds[i].desc ? dyn_cmds[i].desc : "");
        }
    }
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
    struct { uint16_t limit; uint32_t base; } __attribute__((packed)) null_idt = {0, 0};
    __asm__ volatile("lidt %0" : : "m"(null_idt));
    __asm__ volatile("int $3");
    while (1) __asm__ volatile("hlt");
}

/* ------------------------------------------------------------------ */
/* Built-in: file management commands                                  */
/* ------------------------------------------------------------------ */

static void cmd_ls(int argc, char *argv[])
{
    const char *path = (argc > 1) ? argv[1] : ".";
    int n = vfs_list_directory(path);
    if (n < 0) {
        console_printf("ls: cannot list '%s'\n", path);
    } else if (n == 0) {
        console_puts("  (empty)\n");
    }
}

static void cmd_cat(int argc, char *argv[])
{
    if (argc < 2) { console_puts("usage: cat <file>\n"); return; }

    static char buf[4096];
    int n = vfs_read_file(argv[1], buf, sizeof(buf) - 1);
    if (n < 0) {
        console_printf("cat: cannot read '%s'\n", argv[1]);
        return;
    }
    buf[n] = '\0';
    console_puts(buf);
    /* Ensure trailing newline */
    if (n > 0 && buf[n - 1] != '\n') console_putchar('\n');
}

static void cmd_write(int argc, char *argv[])
{
    if (argc < 3) { console_puts("usage: write <file> <text...>\n"); return; }

    /* Concatenate all arguments after the filename */
    static char buf[4096];
    int pos = 0;
    for (int i = 2; i < argc && pos < 4090; i++) {
        if (i > 2 && pos < 4090) buf[pos++] = ' ';
        const char *s = argv[i];
        while (*s && pos < 4090) buf[pos++] = *s++;
    }
    buf[pos++] = '\n';
    buf[pos] = '\0';

    int n = vfs_write_file(argv[1], buf, (uint32_t)pos);
    if (n < 0) {
        console_printf("write: failed to write '%s'\n", argv[1]);
    } else {
        console_printf("wrote %d bytes to %s\n", pos, argv[1]);
    }
}

static void cmd_touch(int argc, char *argv[])
{
    if (argc < 2) { console_puts("usage: touch <file>\n"); return; }
    if (vfs_touch(argv[1]) != 0) {
        console_printf("touch: failed '%s'\n", argv[1]);
    }
}

static void cmd_rm(int argc, char *argv[])
{
    if (argc < 2) { console_puts("usage: rm <path>\n"); return; }
    if (vfs_remove(argv[1]) != 0) {
        console_printf("rm: failed '%s' (non-empty dir?)\n", argv[1]);
    }
}

static void cmd_mkdir(int argc, char *argv[])
{
    if (argc < 2) { console_puts("usage: mkdir <dir>\n"); return; }
    if (vfs_mkdir(argv[1]) != 0) {
        console_printf("mkdir: failed '%s'\n", argv[1]);
    }
}

static void cmd_stat(int argc, char *argv[])
{
    if (argc < 2) { console_puts("usage: stat <path>\n"); return; }

    vfs_stat_t st;
    if (vfs_stat(argv[1], &st) != 0) {
        console_printf("stat: not found '%s'\n", argv[1]);
        return;
    }
    console_printf("  Path: %s\n", argv[1]);
    console_printf("  Type: %s\n", st.type == VFS_TYPE_DIR ? "directory" : "file");
    if (st.type == VFS_TYPE_DIR) {
        console_printf("  Children: %u\n", st.child_count);
    } else {
        console_printf("  Size: %u bytes\n", st.size);
    }
}

static void cmd_pwd(void)
{
    console_printf("%s\n", vfs_getcwd());
}

static void cmd_cd(int argc, char *argv[])
{
    const char *path = (argc > 1) ? argv[1] : "/";
    if (vfs_chdir(path) != 0) {
        console_printf("cd: not a directory '%s'\n", path);
    }
}

static void cmd_cp(int argc, char *argv[])
{
    if (argc < 3) { console_puts("usage: cp <src> <dst>\n"); return; }
    if (vfs_copy(argv[1], argv[2]) != 0) {
        console_printf("cp: failed\n");
    }
}

static void cmd_hexdump(int argc, char *argv[])
{
    if (argc < 2) { console_puts("usage: hexdump <file>\n"); return; }

    static char buf[4096];
    int n = vfs_read_file(argv[1], buf, sizeof(buf));
    if (n < 0) {
        console_printf("hexdump: cannot read '%s'\n", argv[1]);
        return;
    }

    for (int off = 0; off < n; off += 16) {
        console_printf("%04x  ", off);

        /* Hex bytes */
        for (int j = 0; j < 16; j++) {
            if (off + j < n) {
                console_printf("%02x ", (uint8_t)buf[off + j]);
            } else {
                console_puts("   ");
            }
            if (j == 7) console_putchar(' ');
        }

        console_puts(" |");
        /* ASCII */
        for (int j = 0; j < 16 && off + j < n; j++) {
            char c = buf[off + j];
            console_putchar((c >= 0x20 && c < 0x7f) ? c : '.');
        }
        console_puts("|\n");
    }
    console_printf("%04x  (%d bytes)\n", n, n);
}

static void tree_recurse(ramfs_node_t *node, int depth)
{
    if (!node) return;

    for (uint32_t i = 0; i < node->child_count; i++) {
        ramfs_node_t *ch = node->children[i];
        if (!ch) continue;

        /* Indent */
        for (int d = 0; d < depth; d++) console_puts("  ");

        if (ch->type == RAMFS_DIR) {
            console_printf("%s/\n", ch->name);
            tree_recurse(ch, depth + 1);
        } else {
            console_printf("%s (%u)\n", ch->name, ch->size);
        }
    }
}

static void cmd_tree(int argc, char *argv[])
{
    const char *path = (argc > 1) ? argv[1] : ".";
    ramfs_node_t *node = vfs_resolve(path);
    if (!node || node->type != RAMFS_DIR) {
        console_printf("tree: not a directory '%s'\n", path);
        return;
    }

    console_printf("%s\n", path);
    tree_recurse(node, 1);
}

/* ------------------------------------------------------------------ */
/* Command dispatch                                                    */
/* ------------------------------------------------------------------ */

static void dispatch(char *line)
{
    char *argv[MAX_ARGS];
    int argc = parse_args(line, argv);
    if (argc == 0) return;

    /* Check dynamic commands first (packages can override) */
    dyn_cmd_t *dc = find_dyn_cmd(argv[0]);
    if (dc) { dc->fn(argc, argv); return; }

    /* Built-in system commands */
    if      (streq(argv[0], "help"))    cmd_help();
    else if (streq(argv[0], "echo"))    cmd_echo(argc, argv);
    else if (streq(argv[0], "clear"))   cmd_clear();
    else if (streq(argv[0], "ps"))      cmd_ps();
    else if (streq(argv[0], "uptime"))  cmd_uptime();
    else if (streq(argv[0], "mem"))     cmd_mem();
    else if (streq(argv[0], "version")) cmd_version();
    else if (streq(argv[0], "reboot"))  cmd_reboot();
    /* File commands */
    else if (streq(argv[0], "ls"))      cmd_ls(argc, argv);
    else if (streq(argv[0], "cat"))     cmd_cat(argc, argv);
    else if (streq(argv[0], "write"))   cmd_write(argc, argv);
    else if (streq(argv[0], "touch"))   cmd_touch(argc, argv);
    else if (streq(argv[0], "rm"))      cmd_rm(argc, argv);
    else if (streq(argv[0], "mkdir"))   cmd_mkdir(argc, argv);
    else if (streq(argv[0], "stat"))    cmd_stat(argc, argv);
    else if (streq(argv[0], "pwd"))     cmd_pwd();
    else if (streq(argv[0], "cd"))      cmd_cd(argc, argv);
    else if (streq(argv[0], "cp"))      cmd_cp(argc, argv);
    else if (streq(argv[0], "hexdump")) cmd_hexdump(argc, argv);
    else if (streq(argv[0], "tree"))    cmd_tree(argc, argv);
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

    /* Show /etc/motd */
    {
        static char motd[512];
        int n = vfs_read_file("/etc/motd", motd, sizeof(motd) - 1);
        if (n > 0) { motd[n] = '\0'; console_puts(motd); console_putchar('\n'); }
    }

    console_puts("nexus> ");

    while (1) {
        __asm__ volatile("hlt");

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
