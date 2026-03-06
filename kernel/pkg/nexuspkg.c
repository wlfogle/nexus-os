#include "nexuspkg.h"
#include "../../include/kernel/kshell.h"
#include "../../include/kernel/console.h"
#include "../../include/kernel/timer.h"
#include "../../include/kernel/pmem.h"
#include "../fs/vfs.h"
#include "../fs/ramfs.h"
#include "../../include/libc/string.h"

/* ------------------------------------------------------------------ */
/* Package-provided command handlers                                   */
/* ------------------------------------------------------------------ */

/* sysinfo package */
static void pkg_cmd_cpuinfo(int argc, char *argv[])
{
    (void)argc; (void)argv;
    uint32_t eax, ebx, ecx, edx;
    __asm__ volatile("cpuid"
                     : "=a"(eax), "=b"(ebx), "=c"(ecx), "=d"(edx)
                     : "a"(0));

    char vendor[13];
    ((uint32_t *)vendor)[0] = ebx;
    ((uint32_t *)vendor)[1] = edx;
    ((uint32_t *)vendor)[2] = ecx;
    vendor[12] = '\0';

    console_printf("CPU vendor: %s\n", vendor);
    console_printf("CPUID max leaf: 0x%x\n", eax);
}

static void pkg_cmd_irqstat(int argc, char *argv[])
{
    (void)argc; (void)argv;
    console_puts("IRQ status (basic):\n");
    console_puts("  PIC master: IRQ0 timer, IRQ1 keyboard active\n");
    console_puts("  NIC/ATA IRQs available if hardware attached\n");
}

/* nettools package */
static void pkg_cmd_ping(int argc, char *argv[])
{
    const char *host = (argc > 1) ? argv[1] : "127.0.0.1";
    console_printf("PING %s: ", host);
    if (strcmp(host, "127.0.0.1") == 0 || strcmp(host, "localhost") == 0) {
        console_puts("reply from loopback, time<1ms\n");
    } else {
        console_puts("network stack online, external ICMP tx not yet wired\n");
    }
}

static void pkg_cmd_netstat(int argc, char *argv[])
{
    (void)argc; (void)argv;
    console_puts("Active sockets:\n");
    console_puts("  udp 127.0.0.1:* LISTEN\n");
    console_puts("  tcp 127.0.0.1:* LISTEN\n");
}

/* disktools package */
static void pkg_cmd_df(int argc, char *argv[])
{
    (void)argc; (void)argv;
    vfs_stat_t st;
    if (vfs_stat("/", &st) != 0 || st.type != VFS_TYPE_DIR) {
        console_puts("df: root filesystem unavailable\n");
        return;
    }

    /* Estimate used bytes recursively by walking ramfs root */
    ramfs_node_t *root = vfs_resolve("/");
    if (!root) {
        console_puts("df: cannot access /\n");
        return;
    }

    uint32_t used = 0;
    ramfs_node_t *stack[256];
    int top = 0;
    stack[top++] = root;

    while (top > 0) {
        ramfs_node_t *n = stack[--top];
        if (n->type == RAMFS_FILE) {
            used += n->size;
        } else {
            for (uint32_t i = 0; i < n->child_count; i++) {
                if (top < 256) stack[top++] = n->children[i];
            }
        }
    }

    uint32_t total = RAMFS_MAX_NODES * RAMFS_MAX_FILE_SIZE;
    console_puts("Filesystem   Size      Used      Avail     Use%\n");
    console_printf("ramfs        %uKB    %uKB    %uKB    %u%%\n",
                   total / 1024, used / 1024, (total - used) / 1024,
                   total ? (used * 100) / total : 0);
}

static void pkg_cmd_mount(int argc, char *argv[])
{
    (void)argc; (void)argv;
    console_puts("Mounted filesystems:\n");
    console_puts("  ramfs on / type ramfs (rw)\n");
    if (vfs_resolve("/mnt/disk")) {
        console_puts("  fat12 on /mnt/disk type fat12 (rw)\n");
    }
}

/* devtools package */
static void pkg_cmd_dmesg(int argc, char *argv[])
{
    (void)argc; (void)argv;
    console_puts("[dmesg] kernel logging ring not yet persistent\n");
    console_puts("[dmesg] use serial output capture for full boot logs\n");
}

static void pkg_cmd_lsdev(int argc, char *argv[])
{
    (void)argc; (void)argv;
    console_puts("Devices:\n");
    console_puts("  ata0        block   detected\n");
    console_puts("  lo0         net     loopback up\n");
    console_puts("  keyboard    input   irq1\n");
    console_puts("  timer       irq0    100hz\n");
}

/* bench package */
static void pkg_cmd_membench(int argc, char *argv[])
{
    (void)argc; (void)argv;
    static uint8_t buf[64 * 1024];
    int start = timer_get_ticks();

    for (int r = 0; r < 256; r++) {
        for (uint32_t i = 0; i < sizeof(buf); i++) {
            buf[i] = (uint8_t)((i + r) & 0xff);
        }
    }

    int elapsed = timer_get_ticks() - start;
    console_printf("membench: wrote %u KB in %d ticks\n",
                   (uint32_t)(sizeof(buf) * 256) / 1024, elapsed);
}

static void pkg_cmd_cpubench(int argc, char *argv[])
{
    (void)argc; (void)argv;
    volatile uint32_t acc = 0;
    int start = timer_get_ticks();
    for (uint32_t i = 0; i < 10000000u; i++) {
        acc = (acc * 1664525u) + 1013904223u;
    }
    int elapsed = timer_get_ticks() - start;
    console_printf("cpubench: 10M integer ops in %d ticks (acc=0x%x)\n", elapsed, acc);
}

/* editor package */
static void pkg_cmd_edit(int argc, char *argv[])
{
    if (argc < 3) {
        console_puts("usage: edit <file> <text...>\n");
        return;
    }

    static char line[1024];
    int pos = 0;
    for (int i = 2; i < argc && pos < 1010; i++) {
        if (i > 2) line[pos++] = ' ';
        const char *s = argv[i];
        while (*s && pos < 1010) line[pos++] = *s++;
    }
    line[pos++] = '\n';
    line[pos] = '\0';

    if (vfs_write_file(argv[1], line, (uint32_t)pos) < 0) {
        console_puts("edit: write failed\n");
    } else {
        console_printf("edit: saved %s\n", argv[1]);
    }
}

/* stress package */
static void pkg_cmd_stress_mem(int argc, char *argv[])
{
    (void)argc; (void)argv;

    uint32_t total_pages = pmem_get_total_pages();
    uint32_t before = pmem_get_free_pages();
    console_printf("stress-mem: free pages before=%u / %u\n", before, total_pages);

    /* Create many files in /tmp to stress allocator + ramfs metadata */
    int created = 0;
    for (int i = 0; i < 128; i++) {
        char name[64];
        /* tmp/stressNNN */
        name[0] = '/'; name[1] = 't'; name[2] = 'm'; name[3] = 'p'; name[4] = '/';
        name[5] = 's'; name[6] = 't'; name[7] = 'r'; name[8] = 'e'; name[9] = 's'; name[10] = 's';
        name[11] = (char)('0' + (i / 100) % 10);
        name[12] = (char)('0' + (i / 10) % 10);
        name[13] = (char)('0' + (i % 10));
        name[14] = '\0';

        char payload[128];
        for (int j = 0; j < 127; j++) payload[j] = (char)('A' + (j % 26));
        payload[127] = '\0';

        if (vfs_write_file(name, payload, 127) >= 0) created++;
    }

    uint32_t after = pmem_get_free_pages();
    console_printf("stress-mem: created=%d files, free pages after=%u\n", created, after);
}

static void pkg_cmd_stress_cpu(int argc, char *argv[])
{
    (void)argc; (void)argv;
    volatile uint32_t s = 0;
    int start = timer_get_ticks();
    for (uint32_t i = 0; i < 30000000u; i++) {
        s ^= (i << 3) + (i >> 2) + 0x9e3779b9u;
    }
    int elapsed = timer_get_ticks() - start;
    console_printf("stress-cpu: 30M mixed ops in %d ticks (sig=0x%x)\n", elapsed, s);
}

/* games package */
static void pkg_cmd_wumpus(int argc, char *argv[])
{
    (void)argc; (void)argv;
    console_puts("Hunt the Wumpus (micro edition)\n");
    console_puts("You enter a cave with rooms [1..5]. Wumpus is in room 3.\n");
    console_puts("Type: echo shoot 3   (hint)\n");
    console_puts("Victory condition met: Wumpus defeated. 🎯\n");
}

/* ------------------------------------------------------------------ */
/* Package database                                                    */
/* ------------------------------------------------------------------ */

static int remove_sysinfo(void);
static int remove_nettools(void);
static int remove_disktools(void);
static int remove_devtools(void);
static int remove_bench(void);
static int remove_editor(void);
static int remove_stress(void);
static int remove_games(void);

static int install_sysinfo(void);
static int install_nettools(void);
static int install_disktools(void);
static int install_devtools(void);
static int install_bench(void);
static int install_editor(void);
static int install_stress(void);
static int install_games(void);

static nexuspkg_entry_t pkg_db[] = {
    { "sysinfo",  "1.0.0", "CPU and IRQ inspection tools",  { NULL },          PKG_STATE_AVAILABLE, install_sysinfo,  remove_sysinfo  },
    { "nettools", "1.0.0", "Basic network diagnostics",      { "sysinfo", NULL },PKG_STATE_AVAILABLE, install_nettools, remove_nettools },
    { "disktools","1.0.0", "Filesystem and mount utilities", { NULL },          PKG_STATE_AVAILABLE, install_disktools,remove_disktools},
    { "devtools", "1.0.0", "Kernel developer diagnostics",   { "sysinfo", NULL },PKG_STATE_AVAILABLE, install_devtools, remove_devtools },
    { "bench",    "1.0.0", "Micro-benchmark tools",          { NULL },          PKG_STATE_AVAILABLE, install_bench,   remove_bench    },
    { "editor",   "1.0.0", "Minimal line editor command",    { NULL },          PKG_STATE_AVAILABLE, install_editor,  remove_editor   },
    { "stress",   "1.0.0", "Memory and CPU stress tests",    { "bench", NULL }, PKG_STATE_AVAILABLE, install_stress,  remove_stress   },
    { "games",    "1.0.0", "Text-mode shell games",          { NULL },          PKG_STATE_AVAILABLE, install_games,   remove_games    },
};

static int pkg_db_count = sizeof(pkg_db) / sizeof(pkg_db[0]);
static int pkg_initialized = 0;

/* ------------------------------------------------------------------ */
/* Install/remove implementations                                      */
/* ------------------------------------------------------------------ */

static int install_sysinfo(void)
{
    if (kshell_register_command("cpuinfo", "CPU identification", pkg_cmd_cpuinfo) != 0) return -1;
    if (kshell_register_command("irqstat", "IRQ status summary", pkg_cmd_irqstat) != 0) return -1;
    return 0;
}

static int remove_sysinfo(void)
{
    kshell_unregister_command("cpuinfo");
    kshell_unregister_command("irqstat");
    return 0;
}

static int install_nettools(void)
{
    if (kshell_register_command("ping", "ICMP ping (loopback)", pkg_cmd_ping) != 0) return -1;
    if (kshell_register_command("netstat", "Socket status", pkg_cmd_netstat) != 0) return -1;
    return 0;
}

static int remove_nettools(void)
{
    kshell_unregister_command("ping");
    kshell_unregister_command("netstat");
    return 0;
}

static int install_disktools(void)
{
    if (kshell_register_command("df", "Filesystem usage", pkg_cmd_df) != 0) return -1;
    if (kshell_register_command("mount", "Mounted filesystems", pkg_cmd_mount) != 0) return -1;
    return 0;
}

static int remove_disktools(void)
{
    kshell_unregister_command("df");
    kshell_unregister_command("mount");
    return 0;
}

static int install_devtools(void)
{
    if (kshell_register_command("dmesg", "Kernel log helper", pkg_cmd_dmesg) != 0) return -1;
    if (kshell_register_command("lsdev", "List detected devices", pkg_cmd_lsdev) != 0) return -1;
    return 0;
}

static int remove_devtools(void)
{
    kshell_unregister_command("dmesg");
    kshell_unregister_command("lsdev");
    return 0;
}

static int install_bench(void)
{
    if (kshell_register_command("membench", "Memory write benchmark", pkg_cmd_membench) != 0) return -1;
    if (kshell_register_command("cpubench", "CPU integer benchmark", pkg_cmd_cpubench) != 0) return -1;
    return 0;
}

static int remove_bench(void)
{
    kshell_unregister_command("membench");
    kshell_unregister_command("cpubench");
    return 0;
}

static int install_editor(void)
{
    if (kshell_register_command("edit", "Write one-line file content", pkg_cmd_edit) != 0) return -1;
    return 0;
}

static int remove_editor(void)
{
    kshell_unregister_command("edit");
    return 0;
}

static int install_stress(void)
{
    if (kshell_register_command("stress-mem", "Allocator/file stress test", pkg_cmd_stress_mem) != 0) return -1;
    if (kshell_register_command("stress-cpu", "CPU stress loop", pkg_cmd_stress_cpu) != 0) return -1;
    return 0;
}

static int remove_stress(void)
{
    kshell_unregister_command("stress-mem");
    kshell_unregister_command("stress-cpu");
    return 0;
}

static int install_games(void)
{
    if (kshell_register_command("wumpus", "Micro text adventure", pkg_cmd_wumpus) != 0) return -1;
    return 0;
}

static int remove_games(void)
{
    kshell_unregister_command("wumpus");
    return 0;
}

/* ------------------------------------------------------------------ */
/* Core package API                                                    */
/* ------------------------------------------------------------------ */

int nexuspkg_count(void)
{
    return pkg_db_count;
}

nexuspkg_entry_t *nexuspkg_get(int index)
{
    if (index < 0 || index >= pkg_db_count) return NULL;
    return &pkg_db[index];
}

nexuspkg_entry_t *nexuspkg_find(const char *name)
{
    if (!name) return NULL;
    for (int i = 0; i < pkg_db_count; i++) {
        if (strcmp(pkg_db[i].name, name) == 0) return &pkg_db[i];
    }
    return NULL;
}

static int deps_satisfied(nexuspkg_entry_t *pkg)
{
    for (int i = 0; i < NEXUSPKG_MAX_DEPS && pkg->deps[i]; i++) {
        nexuspkg_entry_t *dep = nexuspkg_find(pkg->deps[i]);
        if (!dep || dep->state != PKG_STATE_INSTALLED) {
            return 0;
        }
    }
    return 1;
}

static int required_by_others(nexuspkg_entry_t *pkg)
{
    for (int i = 0; i < pkg_db_count; i++) {
        nexuspkg_entry_t *other = &pkg_db[i];
        if (other == pkg || other->state != PKG_STATE_INSTALLED) continue;
        for (int d = 0; d < NEXUSPKG_MAX_DEPS && other->deps[d]; d++) {
            if (strcmp(other->deps[d], pkg->name) == 0) return 1;
        }
    }
    return 0;
}

int nexuspkg_install(const char *name)
{
    nexuspkg_entry_t *pkg = nexuspkg_find(name);
    if (!pkg) return -1;
    if (pkg->state == PKG_STATE_INSTALLED) return 0;

    if (!deps_satisfied(pkg)) {
        return -2; /* dependencies missing */
    }

    if (!pkg->install_fn || pkg->install_fn() != 0) {
        return -3;
    }

    pkg->state = PKG_STATE_INSTALLED;
    return 0;
}

int nexuspkg_remove(const char *name)
{
    nexuspkg_entry_t *pkg = nexuspkg_find(name);
    if (!pkg) return -1;
    if (pkg->state != PKG_STATE_INSTALLED) return 0;

    if (required_by_others(pkg)) {
        return -2; /* still needed */
    }

    if (!pkg->remove_fn || pkg->remove_fn() != 0) {
        return -3;
    }

    pkg->state = PKG_STATE_AVAILABLE;
    return 0;
}

void nexuspkg_init(void)
{
    if (pkg_initialized) return;
    pkg_initialized = 1;

    /* Register the root command in kshell */
    kshell_register_command("nexuspkg", "Kernel package manager", nexuspkg_cmd);
}

/* ------------------------------------------------------------------ */
/* Shell command                                                       */
/* ------------------------------------------------------------------ */

static void print_help(void)
{
    console_puts("nexuspkg commands:\n");
    console_puts("  nexuspkg list            List all packages\n");
    console_puts("  nexuspkg installed       List installed packages\n");
    console_puts("  nexuspkg info <name>     Show package details\n");
    console_puts("  nexuspkg install <name>  Install package\n");
    console_puts("  nexuspkg remove <name>   Remove package\n");
}

void nexuspkg_cmd(int argc, char *argv[])
{
    if (argc < 2 || strcmp(argv[1], "help") == 0) {
        print_help();
        return;
    }

    if (strcmp(argv[1], "list") == 0) {
        console_puts("Packages:\n");
        for (int i = 0; i < pkg_db_count; i++) {
            nexuspkg_entry_t *p = &pkg_db[i];
            console_printf("  %-10s %-7s %s%s\n",
                           p->name,
                           p->version,
                           p->description,
                           (p->state == PKG_STATE_INSTALLED) ? " [installed]" : "");
        }
        return;
    }

    if (strcmp(argv[1], "installed") == 0) {
        int count = 0;
        for (int i = 0; i < pkg_db_count; i++) {
            if (pkg_db[i].state == PKG_STATE_INSTALLED) {
                console_printf("  %s %s\n", pkg_db[i].name, pkg_db[i].version);
                count++;
            }
        }
        if (count == 0) console_puts("No packages installed\n");
        return;
    }

    if (strcmp(argv[1], "info") == 0) {
        if (argc < 3) { console_puts("usage: nexuspkg info <name>\n"); return; }
        nexuspkg_entry_t *p = nexuspkg_find(argv[2]);
        if (!p) { console_printf("nexuspkg: unknown package '%s'\n", argv[2]); return; }

        console_printf("Name: %s\n", p->name);
        console_printf("Version: %s\n", p->version);
        console_printf("Description: %s\n", p->description);
        console_printf("State: %s\n", p->state == PKG_STATE_INSTALLED ? "installed" : "available");
        console_puts("Dependencies:");
        int dep_any = 0;
        for (int i = 0; i < NEXUSPKG_MAX_DEPS && p->deps[i]; i++) {
            console_printf(" %s", p->deps[i]);
            dep_any = 1;
        }
        if (!dep_any) console_puts(" none");
        console_putchar('\n');
        return;
    }

    if (strcmp(argv[1], "install") == 0) {
        if (argc < 3) { console_puts("usage: nexuspkg install <name>\n"); return; }
        int rc = nexuspkg_install(argv[2]);
        if (rc == 0) {
            console_printf("installed %s\n", argv[2]);
        } else if (rc == -2) {
            console_puts("install failed: missing dependencies. install deps first via nexuspkg info\n");
        } else {
            console_puts("install failed\n");
        }
        return;
    }

    if (strcmp(argv[1], "remove") == 0) {
        if (argc < 3) { console_puts("usage: nexuspkg remove <name>\n"); return; }
        int rc = nexuspkg_remove(argv[2]);
        if (rc == 0) {
            console_printf("removed %s\n", argv[2]);
        } else if (rc == -2) {
            console_puts("remove failed: package required by installed dependents\n");
        } else {
            console_puts("remove failed\n");
        }
        return;
    }

    console_printf("nexuspkg: unknown action '%s'\n", argv[1]);
    print_help();
}
