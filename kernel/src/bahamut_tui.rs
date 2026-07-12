//! NexusOS Lite — Bahamut TUI
//!
//! Replaces the bare nexus> shell on the bahamut target with a proper
//! network-edge setup interface.  Runs as a kernel-mode task using the
//! PL011 UART for serial I/O (ANSI escape codes for layout).
//!
//! Menu:
//!   1  Network Configuration
//!   2  DNS  — AdGuard Home  (:53 / :8081)
//!   3  VPN  — WireGuard     (:51820/udp)
//!   4  Proxy — Caddy        (:80/:443 / DuckDNS TLS)
//!   5  Secrets — Vaultwarden(:8080)
//!   6  System Status
//!   7  Install to Disk
//!   S  Shell (expert mode)
//!   R  Reboot

use crate::kprintln;

// ── ANSI escape helpers ───────────────────────────────────────────────────────

/// Clear terminal + move cursor home.
macro_rules! ansi_clear {
    () => { "\x1b[2J\x1b[H" }
}
/// Bold.
macro_rules! ansi_bold  { () => { "\x1b[1m"  } }
/// Cyan.
macro_rules! ansi_cyan  { () => { "\x1b[36m" } }
/// Green.
macro_rules! ansi_green { () => { "\x1b[32m" } }
/// Yellow.
macro_rules! ansi_yellow{ () => { "\x1b[33m" } }
/// Magenta.
macro_rules! ansi_magenta{()=> { "\x1b[35m" } }
/// Reset.
macro_rules! ansi_reset { () => { "\x1b[0m"  } }

// ── Read a single char from UART ─────────────────────────────────────────────

#[cfg(target_arch = "aarch64")]
fn uart_getchar() -> u8 {
    crate::io::uart::read_char()
}

// ── Print helpers (write directly via kprintln / UART) ───────────────────────

fn print_banner() {
    kprintln!("{}{}", ansi_clear!(), "");
    kprintln!("{}{}╔══════════════════════════════════════════════════════╗{}", ansi_bold!(), ansi_cyan!(), ansi_reset!());
    kprintln!("{}{}║      NexusOS Lite  —  Bahamut Network Edge           ║{}", ansi_bold!(), ansi_cyan!(), ansi_reset!());
    kprintln!("{}{}║      World's First AI-Native OS   v0.6.2             ║{}", ansi_bold!(), ansi_cyan!(), ansi_reset!());
    kprintln!("{}{}╠══════════════════════════════════════════════════════╣{}", ansi_bold!(), ansi_cyan!(), ansi_reset!());
    kprintln!("{}{}║  Role: network-edge  │  Arch: aarch64  │ Pi 4       ║{}", ansi_cyan!(), ansi_bold!(), ansi_reset!());
    kprintln!("{}{}╚══════════════════════════════════════════════════════╝{}", ansi_bold!(), ansi_cyan!(), ansi_reset!());
    kprintln!("");
}

fn print_menu() {
    kprintln!("{}{}  Network Services{}                    {}Status{}", ansi_bold!(), ansi_yellow!(), ansi_reset!(), ansi_green!(), ansi_reset!());
    kprintln!("  ─────────────────────────────────────────────");
    kprintln!("  {}[1]{} Network Configuration              [ ] pending", ansi_bold!(), ansi_reset!());
    kprintln!("  {}[2]{} DNS          AdGuard Home  :53/:8081   [ ] pending", ansi_bold!(), ansi_reset!());
    kprintln!("  {}[3]{} VPN          WireGuard     :51820/udp  [ ] pending", ansi_bold!(), ansi_reset!());
    kprintln!("  {}[4]{} Proxy        Caddy         :80/:443    [ ] pending", ansi_bold!(), ansi_reset!());
    kprintln!("  {}[5]{} Secrets      Vaultwarden   :8080       [ ] pending", ansi_bold!(), ansi_reset!());
    kprintln!("  ─────────────────────────────────────────────");
    kprintln!("  {}[6]{} System Status", ansi_bold!(), ansi_reset!());
    kprintln!("  {}[7]{} {}Install to Disk{}", ansi_bold!(), ansi_reset!(), ansi_magenta!(), ansi_reset!());
    kprintln!("  {}[S]{} Shell  (expert mode)", ansi_bold!(), ansi_reset!());
    kprintln!("  {}[R]{} Reboot", ansi_bold!(), ansi_reset!());
    kprintln!("");
    kprintln!("  {}Enter selection: {}", ansi_bold!(), ansi_reset!());
}

fn print_network_config() {
    kprintln!("");
    kprintln!("{}Network Configuration{}", ansi_bold!(), ansi_reset!());
    kprintln!("  Static IP : 192.168.12.244/24");
    kprintln!("  Gateway   : 192.168.12.254");
    kprintln!("  DNS       : 127.0.0.1  (AdGuard, once installed)");
    kprintln!("  Interface : eth0");
    kprintln!("");
    kprintln!("  Note: Network stack (Phase 7) required for live config.");
    kprintln!("  Press any key to return...");
}

fn print_dns_info() {
    kprintln!("");
    kprintln!("{}DNS — AdGuard Home{}", ansi_bold!(), ansi_reset!());
    kprintln!("  Listens  : :53 (DNS)  :8081 (web admin)");
    kprintln!("  Upstream : Quad9 DoH, Cloudflare DoH, Google DoH");
    kprintln!("  Cache    : 32 MB");
    kprintln!("  Blocklists (7):");
    kprintln!("    AdGuard DNS filter, AdAway, EasyList, EasyPrivacy,");
    kprintln!("    StevenBlack Hosts, URLhaus Malware, HaGeZi Pro Mini");
    kprintln!("  Rewrites : *.tiamat.local → 192.168.12.30");
    kprintln!("             *.mediastack.lan → 192.168.12.30");
    kprintln!("  Admin    : adguard / adguard  (change after install)");
    kprintln!("");
    kprintln!("  Press any key to return...");
}

fn print_vpn_info() {
    kprintln!("");
    kprintln!("{}VPN — WireGuard / PiVPN{}", ansi_bold!(), ansi_reset!());
    kprintln!("  Port     : 51820/udp");
    kprintln!("  Subnet   : 10.92.29.0/24");
    kprintln!("  DNS      : 10.92.29.1 (Bahamut via WG)");
    kprintln!("  Tunnel   : full (AllowedIPs 0.0.0.0/0)");
    kprintln!("  Clients  : laptop, tiamat, openwrt, mediastack");
    kprintln!("  Manage   : pivpn add -n <name>");
    kprintln!("");
    kprintln!("  Press any key to return...");
}

fn print_proxy_info() {
    kprintln!("");
    kprintln!("{}Reverse Proxy — Caddy + DuckDNS{}", ansi_bold!(), ansi_reset!());
    kprintln!("  Ports    : :80 (HTTP redirect)  :443 (HTTPS)");
    kprintln!("  TLS      : DuckDNS DNS-01 wildcard cert (auto-renew)");
    kprintln!("  Domain   : *.lou-fogle-media-stack.duckdns.org");
    kprintln!("  Routes   :");
    kprintln!("    vaultwarden.*  → localhost:8080");
    kprintln!("    adguard.*      → localhost:8081");
    kprintln!("    ha.*           → 192.168.12.123:8123");
    kprintln!("");
    kprintln!("  Press any key to return...");
}

fn print_secrets_info() {
    kprintln!("");
    kprintln!("{}Secrets — Vaultwarden{}", ansi_bold!(), ansi_reset!());
    kprintln!("  Port     : :8080 (proxied by Caddy → HTTPS)");
    kprintln!("  Binary   : native aarch64-musl (no Docker)");
    kprintln!("  Data     : /opt/appdata/vaultwarden/");
    kprintln!("  Domain   : vaultwarden.lou-fogle-media-stack.duckdns.org");
    kprintln!("  Signups  : disabled (invite-only after install)");
    kprintln!("");
    kprintln!("  Press any key to return...");
}

fn print_status() {
    kprintln!("");
    kprintln!("{}System Status{}", ansi_bold!(), ansi_reset!());
    kprintln!("  Kernel   : NexusOS v0.6.2  [bahamut / aarch64]");
    kprintln!("  Role     : network-edge");
    kprintln!("  Memory   : heap online, paging active");
    kprintln!("  Disk     : checking...");

    let cap = crate::virtio_mmio::capacity();
    if cap > 0 {
        let gib = cap / (2 * 1024 * 1024);
        kprintln!("  Disk     : {} GiB VirtIO-MMIO ({} sectors)", gib, cap);
    } else {
        kprintln!("  Disk     : not detected");
    }

    let mounted = crate::fs::fat::is_mounted();
    kprintln!("  FS       : {}", if mounted { "FAT32 mounted (installed)" } else { "not mounted (boot ISO)" });
    kprintln!("  Network  : Phase 7 (not yet implemented)");
    kprintln!("  Services : pending network stack");
    kprintln!("");
    kprintln!("  Press any key to return...");
}

fn print_install_prompt() {
    kprintln!("");
    kprintln!("{}{}Install NexusOS Lite to Disk{}", ansi_bold!(), ansi_magenta!(), ansi_reset!());
    kprintln!("");

    let cap = crate::virtio_mmio::capacity();
    if cap == 0 {
        kprintln!("  {}ERROR: No disk detected.{}", ansi_yellow!(), ansi_reset!());
        kprintln!("  Attach a VirtIO disk and reboot.");
        kprintln!("");
        kprintln!("  Press any key to return...");
        return;
    }

    let gib = cap / (2 * 1024 * 1024);
    kprintln!("  Target   : VirtIO disk  ({} GiB)", gib);
    kprintln!("  Layout   : GPT + FAT32 ESP");
    kprintln!("  Installs : NexusOS Lite kernel + Limine bootloader");
    kprintln!("");
    kprintln!("  {}WARNING: This will ERASE the target disk!{}", ansi_yellow!(), ansi_reset!());
    kprintln!("  Type YES to confirm, or any other key to cancel: ");
}

// ── Subscreen wait-for-key ────────────────────────────────────────────────────

fn wait_key() {
    #[cfg(target_arch = "aarch64")]
    { uart_getchar(); }
}

// ── Install confirmation ──────────────────────────────────────────────────────

fn run_install_confirmed() {
    kprintln!("");
    kprintln!("  Starting installation...");
    crate::installer::task_installer_run();
}

// ── Main TUI loop ─────────────────────────────────────────────────────────────

/// Entry point — called from main.rs AArch64 boot path.
/// Runs indefinitely, replacing the bare nexus> shell on bahamut.
pub fn run() -> ! {
    // Brief pause for UART to stabilise
    for _ in 0..5_000_000u32 {
        core::hint::spin_loop();
    }

    loop {
        print_banner();
        print_menu();

        #[cfg(target_arch = "aarch64")]
        let ch = uart_getchar();
        #[cfg(not(target_arch = "aarch64"))]
        let ch = b'6'; // fallback for non-aarch64 builds

        match ch {
            b'1' => {
                print_banner();
                print_network_config();
                wait_key();
            }
            b'2' => {
                print_banner();
                print_dns_info();
                wait_key();
            }
            b'3' => {
                print_banner();
                print_vpn_info();
                wait_key();
            }
            b'4' => {
                print_banner();
                print_proxy_info();
                wait_key();
            }
            b'5' => {
                print_banner();
                print_secrets_info();
                wait_key();
            }
            b'6' => {
                print_banner();
                print_status();
                wait_key();
            }
            b'7' => {
                print_banner();
                print_install_prompt();

                let cap = crate::virtio_mmio::capacity();
                if cap == 0 {
                    wait_key();
                    continue;
                }

                // Read YES confirmation
                #[cfg(target_arch = "aarch64")]
                {
                    let mut buf = [0u8; 3];
                    buf[0] = uart_getchar();
                    buf[1] = uart_getchar();
                    buf[2] = uart_getchar();
                    kprintln!("");
                    if buf == *b"YES" {
                        run_install_confirmed();
                        kprintln!("  Press any key to return to menu...");
                        wait_key();
                    } else {
                        kprintln!("  Installation cancelled.");
                        wait_key();
                    }
                }
            }
            b's' | b'S' => {
                // Drop to expert shell (just loop with a minimal prompt)
                kprintln!("");
                kprintln!("  {}Expert shell — type 'menu' to return to TUI{}", ansi_yellow!(), ansi_reset!());
                kprintln!("  (no commands implemented yet — press Enter to return)");
                wait_key();
            }
            b'r' | b'R' => {
                kprintln!("");
                kprintln!("  Rebooting...");
                // Halt — Limine/QEMU will handle reset
                loop {
                    unsafe { core::arch::asm!("wfe"); }
                }
            }
            _ => {
                // Unknown key — redraw
            }
        }
    }
}
