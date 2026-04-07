#!/bin/bash
# 🚀 Garuda Linux Nested Virtualization Optimization Script
# Optimizes system for running Proxmox VM with nested LXC containers

set -e

echo "🚀 Starting Garuda Linux Optimization for Nested Virtualization..."

# Create backup directory
BACKUP_DIR="/home/lou/system-backups/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"
echo "📂 Backup directory: $BACKUP_DIR"

# Function to backup files
backup_file() {
    local file="$1"
    if [[ -f "$file" ]]; then
        cp "$file" "$BACKUP_DIR/$(basename "$file").backup"
        echo "✅ Backed up: $file"
    fi
}

# 1. GRUB Configuration for Nested Virtualization
echo "🔧 Configuring GRUB for optimal virtualization..."
backup_file "/etc/default/grub"

# Enhanced GRUB configuration
cat > /tmp/grub_cmdline_append << 'EOF'
intel_iommu=on
iommu=pt
kvm.ignore_msrs=1
kvm_intel.nested=1
kvm_intel.enable_shadow_vmcs=1
kvm_intel.enable_apicv=1
kvm_intel.ept=1
transparent_hugepage=madvise
processor.max_cstate=1
intel_idle.max_cstate=0
isolcpus=nohz_full,domain,managed_irq:16-31
rcu_nocbs=16-31
nohz_full=16-31
hugepagesz=1G
hugepages=8
default_hugepagesz=1G
mitigations=off
EOF

# Update GRUB
CURRENT_CMDLINE=$(grep "^GRUB_CMDLINE_LINUX_DEFAULT=" /etc/default/grub | cut -d'"' -f2)
NEW_CMDLINE="$CURRENT_CMDLINE $(cat /tmp/grub_cmdline_append | tr '\n' ' ')"

sudo sed -i "s/^GRUB_CMDLINE_LINUX_DEFAULT=.*/GRUB_CMDLINE_LINUX_DEFAULT=\"$NEW_CMDLINE\"/" /etc/default/grub

# 2. KVM Module Configuration
echo "🔧 Optimizing KVM modules..."
backup_file "/etc/modprobe.d/kvm.conf"

sudo tee /etc/modprobe.d/kvm.conf > /dev/null << 'EOF'
# Intel KVM optimizations
options kvm_intel nested=1
options kvm_intel enable_shadow_vmcs=1
options kvm_intel enable_apicv=1
options kvm_intel ept=1
options kvm_intel vpid=1
options kvm_intel emulate_invalid_guest_state=0
options kvm_intel flexpriority=1
options kvm_intel unrestricted_guest=1

# General KVM optimizations
options kvm ignore_msrs=1
options kvm report_ignored_msrs=0
options kvm halt_poll_ns=200000
options kvm halt_poll_ns_grow=10
options kvm halt_poll_ns_shrink=4
EOF

# 3. CPU Governor and Performance
echo "🔧 Setting CPU governor to performance..."
sudo tee /etc/systemd/system/cpu-performance.service > /dev/null << 'EOF'
[Unit]
Description=Set CPU governor to performance
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/usr/bin/cpupower frequency-set -g performance
ExecStart=/bin/bash -c 'echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable cpu-performance.service

# 4. Memory and Kernel Optimizations
echo "🔧 Optimizing kernel parameters..."
backup_file "/etc/sysctl.conf"

sudo tee -a /etc/sysctl.conf > /dev/null << 'EOF'

# === Nested Virtualization Optimizations ===

# Memory management for VMs
vm.swappiness=1
vm.vfs_cache_pressure=50
vm.dirty_ratio=5
vm.dirty_background_ratio=2
vm.dirty_expire_centisecs=1000
vm.dirty_writeback_centisecs=100
vm.overcommit_memory=1
vm.overcommit_ratio=80

# Huge pages support
vm.nr_hugepages=2048
vm.hugetlb_shm_group=0

# Network optimizations for VMs
net.core.rmem_max=67108864
net.core.wmem_max=67108864
net.core.netdev_max_backlog=5000
net.ipv4.tcp_rmem=4096 87380 67108864
net.ipv4.tcp_wmem=4096 65536 67108864
net.ipv4.tcp_congestion_control=bbr
net.core.default_qdisc=fq

# Scheduler optimizations
kernel.sched_migration_cost_ns=5000000
kernel.sched_autogroup_enabled=0
kernel.sched_cfs_bandwidth_slice_us=3000

# File system optimizations
fs.file-max=2097152
fs.nr_open=1048576

# Security optimizations (balanced)
kernel.kptr_restrict=1
kernel.dmesg_restrict=0
EOF

# 5. I/O Scheduler Optimization
echo "🔧 Optimizing I/O schedulers..."
sudo tee /etc/udev/rules.d/60-ioschedulers.rules > /dev/null << 'EOF'
# NVMe drives - use none (for low latency)
ACTION=="add|change", KERNEL=="nvme[0-9]*", ATTR{queue/scheduler}="none"

# SSD drives - use mq-deadline
ACTION=="add|change", KERNEL=="sd[a-z]", ATTR{queue/rotational}=="0", ATTR{queue/scheduler}="mq-deadline"

# HDD drives - use bfq
ACTION=="add|change", KERNEL=="sd[a-z]", ATTR{queue/rotational}=="1", ATTR{queue/scheduler}="bfq"
EOF

# 6. IRQ Balancing for Performance
echo "🔧 Optimizing IRQ handling..."
sudo tee /etc/systemd/system/irq-balance-optimize.service > /dev/null << 'EOF'
[Unit]
Description=Optimize IRQ balancing for virtualization
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/bin/bash -c 'echo 2 > /proc/irq/default_smp_affinity'
ExecStart=/bin/bash -c 'for irq in /proc/irq/*/smp_affinity; do echo 00ff > "$irq" 2>/dev/null || true; done'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable irq-balance-optimize.service

# 7. Transparent Huge Pages
echo "🔧 Configuring Transparent Huge Pages..."
sudo tee /etc/systemd/system/hugepages-setup.service > /dev/null << 'EOF'
[Unit]
Description=Setup Huge Pages for VMs
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/bin/bash -c 'echo madvise > /sys/kernel/mm/transparent_hugepage/enabled'
ExecStart=/bin/bash -c 'echo madvise > /sys/kernel/mm/transparent_hugepage/defrag'
ExecStart=/bin/bash -c 'echo 0 > /sys/kernel/mm/transparent_hugepage/khugepaged/defrag'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable hugepages-setup.service

# 8. Storage Mount Optimizations
echo "🔧 Optimizing storage mounts..."
backup_file "/etc/fstab"

# Add noatime and other optimizations to existing mounts
sudo sed -i 's/defaults/defaults,noatime,relatime/' /etc/fstab

# Mount the 342GB partition for AI storage
if ! grep -q "/mnt/ai-storage" /etc/fstab; then
    echo "/dev/nvme0n1p2 /mnt/ai-storage ext4 defaults,noatime,relatime,user_xattr 0 2" | sudo tee -a /etc/fstab
    sudo mkdir -p /mnt/ai-storage
fi

# 9. Network Optimizations for VMs
echo "🔧 Optimizing network for VMs..."
sudo tee /etc/systemd/system/network-optimize.service > /dev/null << 'EOF'
[Unit]
Description=Network optimizations for virtualization
After=network.target

[Service]
Type=oneshot
ExecStart=/bin/bash -c 'echo 32768 > /proc/sys/net/core/rps_sock_flow_entries'
ExecStart=/bin/bash -c 'for rx in /sys/class/net/*/queues/rx-*/rps_cpus; do echo ff > "$rx" 2>/dev/null || true; done'
ExecStart=/bin/bash -c 'for tx in /sys/class/net/*/queues/tx-*/xps_cpus; do echo ff > "$tx" 2>/dev/null || true; done'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable network-optimize.service

# 10. Memory Ballooning and ZRAM Optimization
echo "🔧 Optimizing ZRAM configuration..."
sudo tee /etc/systemd/system/zram-optimize.service > /dev/null << 'EOF'
[Unit]
Description=Optimize ZRAM for virtualization workloads
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/bin/bash -c 'echo 1 > /sys/block/zram0/recompress'
ExecStart=/bin/bash -c 'echo lz4 > /sys/block/zram0/comp_algorithm'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable zram-optimize.service

# 11. Process and Thread Limits
echo "🔧 Optimizing process limits..."
backup_file "/etc/security/limits.conf"

sudo tee -a /etc/security/limits.conf > /dev/null << 'EOF'

# === Virtualization Limits ===
* soft nofile 1048576
* hard nofile 1048576
* soft nproc 1048576  
* hard nproc 1048576
root soft nofile 1048576
root hard nofile 1048576
root soft nproc unlimited
root hard nproc unlimited
EOF

# 12. Disable Unnecessary Services
echo "🔧 Disabling unnecessary services..."
SERVICES_TO_DISABLE=(
    "bluetooth.service"
    "cups.service" 
    "avahi-daemon.service"
    "ModemManager.service"
)

for service in "${SERVICES_TO_DISABLE[@]}"; do
    if systemctl is-enabled "$service" >/dev/null 2>&1; then
        sudo systemctl disable "$service"
        echo "✅ Disabled: $service"
    fi
done

# 13. GPU Optimization for Passthrough Preparation
echo "🔧 Preparing GPU optimization..."
sudo tee /etc/modprobe.d/nvidia-vm.conf > /dev/null << 'EOF'
# NVIDIA optimizations for VM host
options nvidia NVreg_DeviceFileGID=44
options nvidia NVreg_DeviceFileMode=0664
options nvidia NVreg_DeviceFileUID=0
options nvidia NVreg_ModifyDeviceFiles=1
options nvidia NVreg_PreserveVideoMemoryAllocations=1
EOF

# 14. Create monitoring script
echo "🔧 Creating system monitoring script..."
sudo tee /usr/local/bin/vm-monitor > /dev/null << 'EOF'
#!/bin/bash
# VM Performance Monitor

echo "=== System Performance for VMs ==="
echo "CPU Usage: $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)%"
echo "Memory: $(free -h | awk '/^Mem:/ {printf "Used: %s/%s (%.1f%%), Available: %s", $3, $2, ($3/$2)*100, $7}')"
echo "Load Average: $(uptime | awk -F'load average:' '{print $2}')"
echo "VM Processes: $(ps aux | grep -E '(qemu|kvm|lxc)' | wc -l)"
echo "Huge Pages: $(cat /proc/meminfo | grep -E 'HugePages_(Total|Free):')"
echo "Network Connections: $(ss -tuln | wc -l)"
echo "Disk I/O: $(iostat -x 1 1 | tail -n +4 | head -n -1)"
EOF

sudo chmod +x /usr/local/bin/vm-monitor

# 15. Create optimization status checker
sudo tee /usr/local/bin/check-vm-optimizations > /dev/null << 'EOF'
#!/bin/bash
# Check VM Optimization Status

echo "🔍 VM Optimization Status Check"
echo "================================"

# Check nested virtualization
if [[ $(cat /sys/module/kvm_intel/parameters/nested) == "Y" ]]; then
    echo "✅ Nested virtualization: Enabled"
else
    echo "❌ Nested virtualization: Disabled"
fi

# Check CPU governor
current_governor=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor)
echo "🔧 CPU Governor: $current_governor"

# Check huge pages
hugepages_total=$(cat /proc/meminfo | grep HugePages_Total | awk '{print $2}')
echo "💾 Huge Pages: $hugepages_total configured"

# Check IOMMU
if dmesg | grep -q "IOMMU enabled"; then
    echo "✅ IOMMU: Enabled"
else
    echo "❌ IOMMU: Check required"
fi

# Check kernel parameters
echo "🔧 Key kernel parameters:"
echo "  - vm.swappiness: $(cat /proc/sys/vm/swappiness)"
echo "  - vm.overcommit_memory: $(cat /proc/sys/vm/overcommit_memory)"
echo "  - transparent_hugepage: $(cat /sys/kernel/mm/transparent_hugepage/enabled)"

# Check services
echo "🔧 VM optimization services:"
for service in cpu-performance irq-balance-optimize hugepages-setup network-optimize zram-optimize; do
    if systemctl is-enabled "$service.service" >/dev/null 2>&1; then
        echo "  ✅ $service: Enabled"
    else
        echo "  ❌ $service: Not enabled"
    fi
done
EOF

sudo chmod +x /usr/local/bin/check-vm-optimizations

# 16. Update GRUB and initramfs
echo "🔧 Updating bootloader configuration..."
sudo update-grub
sudo mkinitcpio -P

# 17. Create reboot recommendation
echo "
🎉 OPTIMIZATION COMPLETE!

📊 Summary of optimizations applied:
✅ GRUB configured for nested virtualization
✅ KVM modules optimized
✅ CPU governor set to performance
✅ Memory management tuned for VMs
✅ I/O schedulers optimized
✅ IRQ balancing configured
✅ Huge pages enabled
✅ Network stack optimized
✅ Process limits increased
✅ Unnecessary services disabled
✅ Monitoring tools installed

📁 Backups saved to: $BACKUP_DIR

🔧 Monitoring commands:
- vm-monitor              # Check current VM performance
- check-vm-optimizations  # Verify optimization status

⚠️  REBOOT REQUIRED to apply all optimizations!

🚀 After reboot, your system will be optimized for:
- Running Proxmox VM efficiently
- Supporting nested LXC containers
- Maximum performance for AI workloads
- Optimal resource allocation

Reboot now? (y/N): "

read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    echo "🔄 Rebooting system..."
    sudo reboot
else
    echo "📝 Remember to reboot when convenient to apply all optimizations!"
fi
EOF
