#!/bin/bash
# =============================================================================
# fix-stack.sh — Tiamat Full Stack Fix
# Fixes: WireGuard kill-switch, DNS fallback, sysctl, I/O, watchdog
# Based on awesome-stack-optimization-suite patterns
# =============================================================================

set -euo pipefail
LOG=/var/log/fix-stack.log
exec > >(tee -a $LOG) 2>&1
echo "=== fix-stack.sh started $(date) ==="

# -----------------------------------------------------------------------------
# 1. SYSCTL — apply awesome-stack optimizations to Tiamat host
# -----------------------------------------------------------------------------
echo "[1/7] Applying sysctl optimizations..."
cat > /etc/sysctl.d/99-tiamat-optimized.conf << 'EOF'
# Tiamat optimizations — from awesome-stack-optimization-suite
vm.swappiness = 1
vm.vfs_cache_pressure = 30
vm.dirty_ratio = 10
vm.dirty_background_ratio = 3
vm.overcommit_memory = 1
vm.overcommit_ratio = 90
vm.min_free_kbytes = 131072
net.core.rmem_default = 524288
net.core.rmem_max = 134217728
net.core.wmem_default = 524288
net.core.wmem_max = 134217728
net.core.netdev_max_backlog = 10000
net.core.somaxconn = 16384
net.ipv4.tcp_rmem = 8192 131072 134217728
net.ipv4.tcp_wmem = 8192 131072 134217728
net.ipv4.tcp_congestion_control = bbr
net.ipv4.tcp_fastopen = 3
net.ipv4.tcp_slow_start_after_idle = 0
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 3
net.ipv4.tcp_keepalive_intvl = 30
net.ipv4.ip_local_port_range = 1024 65000
net.ipv4.tcp_max_syn_backlog = 16384
net.netfilter.nf_conntrack_max = 1048576
net.netfilter.nf_conntrack_tcp_timeout_established = 600
fs.file-max = 2097152
fs.inotify.max_user_watches = 1048576
fs.inotify.max_user_instances = 8192
kernel.pid_max = 4194304
user.max_user_namespaces = 65536
EOF
modprobe tcp_bbr 2>/dev/null || true
sysctl -p /etc/sysctl.d/99-tiamat-optimized.conf
echo "  [OK] sysctl applied"

# -----------------------------------------------------------------------------
# 2. I/O SCHEDULER — mq-deadline for NVMe (awesome-stack pattern)
# -----------------------------------------------------------------------------
echo "[2/7] Setting I/O scheduler..."
cat > /etc/udev/rules.d/99-io-scheduler.rules << 'EOF'
ACTION=="add|change", KERNEL=="nvme[0-9]*", ATTR{queue/scheduler}="mq-deadline"
ACTION=="add|change", KERNEL=="sd[a-z]", ATTR{queue/rotational}=="0", ATTR{queue/scheduler}="mq-deadline"
ACTION=="add|change", KERNEL=="sd[a-z]", ATTR{queue/rotational}=="1", ATTR{queue/scheduler}="bfq"
EOF
for dev in /sys/block/nvme* /sys/block/sd*; do
    [ -f "$dev/queue/scheduler" ] && echo mq-deadline > "$dev/queue/scheduler" 2>/dev/null || true
done
echo "  [OK] I/O scheduler set"

# -----------------------------------------------------------------------------
# 3. DNS FALLBACK — add 8.8.8.8 fallback to all running LXC containers
# Fixes Jellyseerr slowness when AdGuard on Bahamut hiccups
# -----------------------------------------------------------------------------
echo "[3/7] Fixing DNS fallback in all containers..."
CONTAINERS=$(pct list | awk 'NR>1 && $2=="running" {print $1}')
for CT in $CONTAINERS; do
    CONF=$(pct exec $CT -- cat /etc/resolv.conf 2>/dev/null || echo "")
    if echo "$CONF" | grep -q "8.8.8.8"; then
        echo "  CT-$CT: DNS fallback already present"
        continue
    fi
    # Append fallback DNS — keep existing nameservers first
    pct exec $CT -- sh -c '
        echo "" >> /etc/resolv.conf
        echo "# fallback DNS — added by fix-stack.sh" >> /etc/resolv.conf
        echo "nameserver 8.8.8.8" >> /etc/resolv.conf
        echo "nameserver 1.1.1.1" >> /etc/resolv.conf
        echo "options timeout:1 attempts:2" >> /etc/resolv.conf
    ' 2>/dev/null && echo "  CT-$CT: [OK] DNS fallback added" || echo "  CT-$CT: [SKIP] could not update"
done

# -----------------------------------------------------------------------------
# 4. WIREGUARD SERVER — CT-100 (Alpine)
# -----------------------------------------------------------------------------
echo "[4/7] Fixing WireGuard server (CT-100)..."
pct exec 100 -- sh -c '
    apk update -q 2>/dev/null || true
    apk add -q wireguard-tools 2>/dev/null || true
    wg-quick down wg0 2>/dev/null || true
    sleep 1
    wg-quick up wg0 2>/dev/null && echo "  CT-100: [OK] wg0 up" || echo "  CT-100: [WARN] wg-quick up failed — check /etc/wireguard/wg0.conf"
    wg show
'

# -----------------------------------------------------------------------------
# 5. WIREGUARD CLIENT + TINYPROXY — CT-101 (Alpine)
# -----------------------------------------------------------------------------
echo "[5/7] Fixing WireGuard client + TinyProxy (CT-101)..."
pct exec 101 -- sh -c '
    apk update -q 2>/dev/null || true
    apk add -q wireguard-tools tinyproxy curl 2>/dev/null || true

    # Restart WireGuard tunnel
    wg-quick down wg0 2>/dev/null || true
    sleep 1
    wg-quick up wg0 2>/dev/null && echo "  CT-101: [OK] wg0 tunnel up" || echo "  CT-101: [WARN] tunnel failed — check /etc/wireguard/wg0.conf exists"
    wg show

    # Restart TinyProxy
    rc-service tinyproxy restart 2>/dev/null || tinyproxy 2>/dev/null || true
    sleep 2
    pgrep tinyproxy > /dev/null && echo "  CT-101: [OK] TinyProxy running" || echo "  CT-101: [WARN] TinyProxy not running"

    # Auto-start on boot
    rc-update add wg-quick.wg0 default 2>/dev/null || true
    rc-update add tinyproxy default 2>/dev/null || true

    # Test proxy
    RESULT=$(curl -s --max-time 10 -x http://127.0.0.1:8888 https://icanhazip.com 2>/dev/null || echo "FAILED")
    echo "  CT-101: Proxy test external IP: $RESULT"
'

# -----------------------------------------------------------------------------
# 6. WIREGUARD WATCHDOG — auto-restart tunnel if it drops
# -----------------------------------------------------------------------------
echo "[6/7] Installing WireGuard watchdog on CT-101..."
pct exec 101 -- sh -c '
cat > /usr/local/bin/wg-watchdog.sh << '"'"'WDOG'"'"'
#!/bin/sh
# WireGuard watchdog — restart tunnel if ping to server fails
if ! ping -c 2 -W 3 10.0.0.1 > /dev/null 2>&1; then
    logger -t wg-watchdog "WireGuard tunnel down — restarting"
    wg-quick down wg0 2>/dev/null || true
    sleep 2
    wg-quick up wg0
    sleep 3
    rc-service tinyproxy restart 2>/dev/null || true
    logger -t wg-watchdog "WireGuard tunnel restarted"
fi
WDOG
chmod +x /usr/local/bin/wg-watchdog.sh

# Run every 5 minutes via cron
if ! crontab -l 2>/dev/null | grep -q wg-watchdog; then
    (crontab -l 2>/dev/null; echo "*/5 * * * * /usr/local/bin/wg-watchdog.sh") | crontab -
fi
rc-service crond start 2>/dev/null || true
rc-update add crond default 2>/dev/null || true
echo "  CT-101: [OK] watchdog installed"
'

# -----------------------------------------------------------------------------
# 7. LIMITS — raise file descriptor limits for media services on host
# -----------------------------------------------------------------------------
echo "[7/7] Applying system limits..."
cat > /etc/security/limits.d/99-tiamat-media.conf << 'EOF'
* soft nofile 1048576
* hard nofile 1048576
* soft nproc  1048576
* hard nproc  1048576
root soft nofile 1048576
root hard nofile 1048576
EOF
echo "  [OK] limits applied"

# -----------------------------------------------------------------------------
# FINAL VERIFICATION
# -----------------------------------------------------------------------------
echo ""
echo "=== VERIFICATION ==="
echo "WireGuard server (CT-100):"
pct exec 100 -- sh -c 'wg show 2>/dev/null || echo "wg0 not running"'

echo ""
echo "WireGuard client + proxy (CT-101):"
pct exec 101 -- sh -c '
    wg show 2>/dev/null || echo "wg0 not running"
    echo "TinyProxy: $(pgrep tinyproxy > /dev/null && echo RUNNING || echo STOPPED)"
'

echo ""
echo "Proxy test from host (through CT-101):"
curl -s --max-time 10 -x http://192.168.12.101:8888 https://icanhazip.com && echo "" || echo "PROXY UNREACHABLE"

echo ""
echo "=== fix-stack.sh completed $(date) ==="
echo "Log saved to $LOG"
