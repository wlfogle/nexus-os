#!/bin/bash
# USB Rescue System Performance Optimizer
# Safe optimizations that preserve rescue capabilities

# Apply I/O optimizations for all USB devices
for device in /sys/block/sd*; do
    if [ -d "$device" ]; then
        # Check if it's a USB device
        if [[ $(udevadm info --query=property --path="$device" 2>/dev/null | grep -i "ID_BUS=usb") ]]; then
            dev_name=$(basename "$device")
            echo "Optimizing USB device: $dev_name"
            
            # Set optimal scheduler for USB
            echo mq-deadline > "$device/queue/scheduler" 2>/dev/null || true
            
            # Reduce queue depth to prevent I/O congestion
            echo 32 > "$device/queue/nr_requests" 2>/dev/null || true
            
            # Optimize readahead for USB
            echo 128 > "$device/queue/read_ahead_kb" 2>/dev/null || true
            
            # Set optimal rotational flag
            echo 0 > "$device/queue/rotational" 2>/dev/null || true
        fi
    fi
done

# Optimize zram compression if available
if [ -e /sys/block/zram0/comp_algorithm ]; then
    echo lz4 > /sys/block/zram0/comp_algorithm 2>/dev/null || true
fi

# Apply CPU governor optimizations for desktop responsiveness
if [ -e /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]; then
    echo ondemand > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true
fi

logger "USB Rescue System Performance Optimizer completed"
