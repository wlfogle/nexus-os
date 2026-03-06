#!/usr/bin/env bash
# =============================================================================
# NexusOS Security Hardening Script
# Hardens a NexusOS system with firewall, intrusion prevention,
# SSH lockdown, and kernel security parameters.
# =============================================================================
set -euo pipefail

readonly SCRIPT_NAME="nexus-harden"
readonly LOG_DIR="/var/log/nexus-os"
readonly LOG_FILE="${LOG_DIR}/hardening.log"
readonly BACKUP_DIR="/etc/nexus-os/security-backup"
readonly SSHD_CONFIG="/etc/ssh/sshd_config"
readonly SSHD_NEXUS="/etc/ssh/sshd_config.d/99-nexus-hardened.conf"
readonly FAIL2BAN_NEXUS="/etc/fail2ban/jail.d/nexus.conf"
readonly SYSCTL_SECURITY="/etc/sysctl.d/99-nexus-security.conf"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log() { printf "[%s] %s\n" "$(date '+%Y-%m-%d %H:%M:%S')" "$*" | tee -a "$LOG_FILE"; }
warn() { log "WARN: $*"; }
die() { log "FATAL: $*"; exit 1; }

need_root() {
    [[ $EUID -eq 0 ]] || die "This script must be run as root (sudo $0)"
}

backup_file() {
    local src="$1"
    if [[ -f "$src" ]]; then
        mkdir -p "$BACKUP_DIR"
        cp -a "$src" "${BACKUP_DIR}/$(basename "$src").$(date +%s).bak"
        log "Backed up $src"
    fi
}

# ---------------------------------------------------------------------------
# UFW Firewall
# ---------------------------------------------------------------------------
setup_ufw() {
    log "=== Configuring UFW Firewall ==="

    if ! command -v ufw &>/dev/null; then
        log "Installing UFW..."
        nala install -y ufw
    fi

    # Reset to clean state (non-interactive)
    ufw --force reset

    # Default policies
    ufw default deny incoming
    ufw default allow outgoing

    # SSH
    ufw allow 22/tcp comment 'SSH'

    # AI Services (localhost only for orchestrator, stella, maxjr)
    ufw allow from 127.0.0.0/8 to any port 8600 proto tcp comment 'Orchestrator'
    ufw allow from 127.0.0.0/8 to any port 8601 proto tcp comment 'Stella AI'
    ufw allow from 127.0.0.0/8 to any port 8602 proto tcp comment 'MaxJr AI'

    # Media Stack — allow LAN access to dashboards
    ufw allow from 192.168.0.0/16 to any port 8200 proto tcp comment 'Jellyfin'
    ufw allow from 192.168.0.0/16 to any port 8201 proto tcp comment 'Plex'
    ufw allow from 192.168.0.0/16 to any port 8540 proto tcp comment 'Organizr'
    ufw allow from 192.168.0.0/16 to any port 8541 proto tcp comment 'Homarr'
    ufw allow from 192.168.0.0/16 to any port 8500 proto tcp comment 'Portainer'
    ufw allow from 10.0.0.0/8 to any port 8200 proto tcp comment 'Jellyfin-10net'
    ufw allow from 10.0.0.0/8 to any port 8201 proto tcp comment 'Plex-10net'
    ufw allow from 10.0.0.0/8 to any port 8540 proto tcp comment 'Organizr-10net'
    ufw allow from 10.0.0.0/8 to any port 8541 proto tcp comment 'Homarr-10net'
    ufw allow from 10.0.0.0/8 to any port 8500 proto tcp comment 'Portainer-10net'

    # Plex discovery (GDM)
    ufw allow 32400/tcp comment 'Plex-direct'

    # Docker — let Docker manage its own iptables but allow forwarding
    ufw allow in on docker0

    # Enable
    ufw --force enable
    ufw status verbose | tee -a "$LOG_FILE"

    log "UFW configured and enabled"
}

# ---------------------------------------------------------------------------
# Fail2Ban
# ---------------------------------------------------------------------------
setup_fail2ban() {
    log "=== Configuring Fail2Ban ==="

    if ! command -v fail2ban-client &>/dev/null; then
        log "Installing fail2ban..."
        nala install -y fail2ban
    fi

    backup_file "$FAIL2BAN_NEXUS" 2>/dev/null || true

    cat > "$FAIL2BAN_NEXUS" << 'FAIL2BAN_EOF'
# NexusOS Fail2Ban Configuration
# Protects SSH and common services from brute-force attacks

[DEFAULT]
bantime  = 3600
findtime = 600
maxretry = 5
banaction = ufw

[sshd]
enabled  = true
port     = ssh
filter   = sshd
logpath  = /var/log/auth.log
maxretry = 3
bantime  = 7200
findtime = 600

[sshd-ddos]
enabled  = true
port     = ssh
filter   = sshd-ddos
logpath  = /var/log/auth.log
maxretry = 6
bantime  = 3600
findtime = 60

[recidive]
enabled  = true
filter   = recidive
logpath  = /var/log/fail2ban.log
bantime  = 604800
findtime = 86400
maxretry = 3
FAIL2BAN_EOF

    systemctl enable fail2ban
    systemctl restart fail2ban
    fail2ban-client status | tee -a "$LOG_FILE"

    log "Fail2ban configured and running"
}

# ---------------------------------------------------------------------------
# SSH Hardening
# ---------------------------------------------------------------------------
setup_ssh() {
    log "=== Hardening SSH ==="

    if ! command -v sshd &>/dev/null; then
        log "OpenSSH server not installed — skipping SSH hardening"
        return 0
    fi

    backup_file "$SSHD_CONFIG"

    mkdir -p "$(dirname "$SSHD_NEXUS")"
    cat > "$SSHD_NEXUS" << 'SSH_EOF'
# NexusOS SSH Hardening
# Drop-in config — overrides defaults from sshd_config

# Disable root login
PermitRootLogin no

# Disable password auth (key-only recommended — uncomment after adding keys)
# PasswordAuthentication no

# Disable empty passwords
PermitEmptyPasswords no

# Strong key exchange and ciphers
KexAlgorithms curve25519-sha256,curve25519-sha256@libssh.org,diffie-hellman-group16-sha512,diffie-hellman-group18-sha512
Ciphers chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com
MACs hmac-sha2-512-etm@openssh.com,hmac-sha2-256-etm@openssh.com

# Limit authentication attempts
MaxAuthTries 3
MaxSessions 4
LoginGraceTime 30

# Disable unused auth methods
ChallengeResponseAuthentication no
KerberosAuthentication no
GSSAPIAuthentication no

# Disable X11 and agent forwarding by default
X11Forwarding no
AllowAgentForwarding no
AllowTcpForwarding no

# Log more detail
LogLevel VERBOSE

# Idle timeout — disconnect after 10 minutes of inactivity
ClientAliveInterval 300
ClientAliveCountMax 2
SSH_EOF

    # Validate config before restarting
    if sshd -t -f "$SSHD_CONFIG" 2>/dev/null; then
        systemctl reload sshd 2>/dev/null || systemctl reload ssh 2>/dev/null || true
        log "SSH hardened and reloaded"
    else
        warn "SSH config validation failed — reverting"
        rm -f "$SSHD_NEXUS"
        return 1
    fi
}

# ---------------------------------------------------------------------------
# Kernel Security Parameters
# ---------------------------------------------------------------------------
setup_sysctl_security() {
    log "=== Applying Kernel Security Parameters ==="

    backup_file "$SYSCTL_SECURITY" 2>/dev/null || true

    cat > "$SYSCTL_SECURITY" << 'SYSCTL_EOF'
# NexusOS Kernel Security Hardening
# Applied via sysctl — complements core/config/sysctl-nexus.conf (performance)

# --- Network hardening ---
# Disable IP source routing
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0
net.ipv6.conf.all.accept_source_route = 0
net.ipv6.conf.default.accept_source_route = 0

# Disable ICMP redirects (prevent MITM)
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.default.accept_redirects = 0
net.ipv6.conf.all.accept_redirects = 0
net.ipv6.conf.default.accept_redirects = 0
net.ipv4.conf.all.send_redirects = 0
net.ipv4.conf.default.send_redirects = 0

# Enable SYN flood protection
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_max_syn_backlog = 4096

# Log martian packets (impossible source addresses)
net.ipv4.conf.all.log_martians = 1
net.ipv4.conf.default.log_martians = 1

# Ignore ICMP broadcast requests
net.ipv4.icmp_echo_ignore_broadcasts = 1

# Ignore bogus ICMP error responses
net.ipv4.icmp_ignore_bogus_error_responses = 1

# Enable reverse path filtering (anti-spoofing)
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1

# --- Kernel hardening ---
# Restrict dmesg access to root
kernel.dmesg_restrict = 1

# Restrict kernel pointer exposure
kernel.kptr_restrict = 2

# Restrict perf_event access
kernel.perf_event_paranoid = 3

# Disable magic SysRq (except sync+reboot: 176)
kernel.sysrq = 176

# Restrict ptrace to direct child processes only
kernel.yama.ptrace_scope = 1

# Restrict unprivileged BPF
kernel.unprivileged_bpf_disabled = 1

# Harden BPF JIT
net.core.bpf_jit_harden = 2

# --- Filesystem hardening ---
# Restrict access to kernel logs
fs.protected_hardlinks = 1
fs.protected_symlinks = 1

# Restrict core dumps
fs.suid_dumpable = 0
SYSCTL_EOF

    sysctl --system 2>&1 | tail -5 | tee -a "$LOG_FILE"

    log "Kernel security parameters applied"
}

# ---------------------------------------------------------------------------
# Additional Hardening
# ---------------------------------------------------------------------------
setup_misc() {
    log "=== Additional Security Measures ==="

    # Disable core dumps via limits
    if ! grep -q 'hard core 0' /etc/security/limits.d/nexus-security.conf 2>/dev/null; then
        cat > /etc/security/limits.d/nexus-security.conf << 'LIMITS_EOF'
# NexusOS — disable core dumps for security
*    hard    core    0
*    soft    core    0
LIMITS_EOF
        log "Core dumps disabled"
    fi

    # Secure shared memory
    if ! grep -q '/run/shm' /etc/fstab; then
        printf "\n# NexusOS — restrict shared memory\ntmpfs /run/shm tmpfs defaults,noexec,nosuid 0 0\n" >> /etc/fstab
        mount -o remount /run/shm 2>/dev/null || true
        log "Shared memory restricted"
    fi

    # Ensure automatic security updates are enabled
    if command -v unattended-upgrades &>/dev/null; then
        dpkg-reconfigure -plow unattended-upgrades 2>/dev/null || true
        log "Unattended security upgrades verified"
    else
        nala install -y unattended-upgrades
        dpkg-reconfigure -plow unattended-upgrades 2>/dev/null || true
        log "Unattended security upgrades installed and enabled"
    fi

    # Set restrictive umask for root
    if ! grep -q 'umask 077' /root/.bashrc 2>/dev/null; then
        echo 'umask 077' >> /root/.bashrc
        log "Root umask set to 077"
    fi
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print_summary() {
    log ""
    log "============================================"
    log "  NexusOS Security Hardening Complete"
    log "============================================"
    log "  UFW Firewall:     ENABLED (deny incoming)"
    log "  Fail2Ban:         ENABLED (SSH + recidive)"
    log "  SSH:              HARDENED (no root, strong crypto)"
    log "  Kernel Params:    APPLIED (anti-spoofing, restricted ptrace)"
    log "  Core Dumps:       DISABLED"
    log "  Shared Memory:    RESTRICTED"
    log "  Auto-Updates:     ENABLED (unattended-upgrades)"
    log ""
    log "  Log: $LOG_FILE"
    log "  Backups: $BACKUP_DIR"
    log ""
    log "  IMPORTANT: To enable key-only SSH auth, add your"
    log "  public key to ~/.ssh/authorized_keys, then uncomment"
    log "  PasswordAuthentication in $SSHD_NEXUS"
    log "============================================"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    need_root
    mkdir -p "$LOG_DIR"

    log "Starting NexusOS security hardening..."

    setup_ufw
    setup_fail2ban
    setup_ssh
    setup_sysctl_security
    setup_misc
    print_summary
}

main "$@"
