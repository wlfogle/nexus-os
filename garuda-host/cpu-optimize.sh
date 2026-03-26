#!/bin/bash

# CPU Performance Optimization Script for AI/ML Workloads

# Set CPU governor to performance for cores 0-23 (main work cores)
for cpu in {0..23}; do
    echo performance > /sys/devices/system/cpu/cpu${cpu}/cpufreq/scaling_governor 2>/dev/null || true
done

# Set CPU governor to ondemand for cores 24-31 (isolated cores)
for cpu in {24..31}; do
    echo ondemand > /sys/devices/system/cpu/cpu${cpu}/cpufreq/scaling_governor 2>/dev/null || true
done

# Disable CPU C-states for better latency on main cores
for cpu in {0..23}; do
    for state in /sys/devices/system/cpu/cpu${cpu}/cpuidle/state*/disable; do
        [ -f "$state" ] && echo 1 > "$state" 2>/dev/null || true
    done
done

# Set CPU affinity for IRQs to first 4 cores to avoid interference
echo f > /proc/irq/default_smp_affinity 2>/dev/null || true

# Optimize scheduler
echo 1 > /sys/kernel/debug/sched_features 2>/dev/null || true

# Set I/O scheduler to mq-deadline for NVMe drives
for nvme in /sys/block/nvme*; do
    if [ -d "$nvme" ]; then
        echo mq-deadline > "$nvme/queue/scheduler" 2>/dev/null || true
        echo 2 > "$nvme/queue/nr_requests" 2>/dev/null || true
    fi
done

# Disable swap readahead for better SSD performance
echo 1 > /proc/sys/vm/page-cluster 2>/dev/null || true

# Set transparent hugepages to madvise for better control
echo madvise > /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null || true
echo madvise > /sys/kernel/mm/transparent_hugepage/defrag 2>/dev/null || true

# Enable BBR congestion control
modprobe tcp_bbr 2>/dev/null || true

echo "CPU optimization completed at $(date)"
