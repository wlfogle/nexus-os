#!/bin/bash
# Universal USB Device Detection Utility
# Works with any USB storage device, not hardcoded to specific paths

detect_usb_devices() {
    local usb_devices=""
    
    for device in /sys/block/sd*; do
        if [ -d "$device" ]; then
            # Check if it's a USB device using multiple methods
            local dev_name=$(basename "$device")
            
            # Method 1: Check udev properties
            if udevadm info --query=property --path="$device" 2>/dev/null | grep -q "ID_BUS=usb"; then
                usb_devices="$usb_devices $dev_name"
                continue
            fi
            
            # Method 2: Check if device path contains 'usb'
            if [ -L "$device" ]; then
                local real_path=$(readlink -f "$device")
                if echo "$real_path" | grep -q "usb"; then
                    usb_devices="$usb_devices $dev_name"
                    continue
                fi
            fi
            
            # Method 3: Check device speed (USB devices typically have specific speeds)
            if [ -f "$device/queue/max_sectors_kb" ]; then
                local max_sectors=$(cat "$device/queue/max_sectors_kb" 2>/dev/null)
                # USB devices often have lower max_sectors (common values: 240, 480, 960)
                if [[ "$max_sectors" -le 1024 ]]; then
                    # Additional check: verify it's not an NVMe or internal SATA
                    if ! echo "$device" | grep -E "(nvme|mmc)" && [ -f "/sys/block/$dev_name/removable" ]; then
                        local removable=$(cat "/sys/block/$dev_name/removable" 2>/dev/null)
                        if [[ "$removable" == "1" ]]; then
                            usb_devices="$usb_devices $dev_name"
                        fi
                    fi
                fi
            fi
        fi
    done
    
    echo "$usb_devices"
}

# Get the root device (device we're currently running from)
detect_root_device() {
    local root_device=""
    
    # Method 1: Check what device root is mounted from
    local root_mount=$(findmnt -n -o SOURCE /)
    if [[ ! -z "$root_mount" ]]; then
        # Extract device name (handle various formats like /dev/sda1, overlay, etc.)
        if echo "$root_mount" | grep -q "^/dev/"; then
            root_device=$(echo "$root_mount" | sed 's/[0-9]*$//' | sed 's|/dev/||')
        fi
    fi
    
    # Method 2: Check boot device from /proc/cmdline
    if [[ -z "$root_device" ]] && [ -f /proc/cmdline ]; then
        local cmdline_root=$(grep -o 'root=[^[:space:]]*' /proc/cmdline | cut -d= -f2)
        if echo "$cmdline_root" | grep -q "^/dev/"; then
            root_device=$(echo "$cmdline_root" | sed 's/[0-9]*$//' | sed 's|/dev/||')
        fi
    fi
    
    echo "$root_device"
}

# Get current USB device we're running from (if any)
get_current_usb_device() {
    local root_dev=$(detect_root_device)
    local usb_devices=$(detect_usb_devices)
    
    for usb_dev in $usb_devices; do
        if [[ "$usb_dev" == "$root_dev" ]]; then
            echo "$usb_dev"
            return 0
        fi
    done
    
    # If no match, return the first USB device found
    echo $usb_devices | awk '{print $1}'
}

# Main function for command-line usage
main() {
    case "${1:-detect}" in
        "detect"|"list")
            echo "USB devices found: $(detect_usb_devices)"
            ;;
        "root")
            echo "Root device: $(detect_root_device)"
            ;;
        "current")
            echo "Current USB device: $(get_current_usb_device)"
            ;;
        "help"|"-h"|"--help")
            echo "Usage: $0 [detect|list|root|current|help]"
            echo "  detect/list - List all USB storage devices"
            echo "  root        - Show root filesystem device"
            echo "  current     - Show USB device we're running from"
            echo "  help        - Show this help"
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# If script is executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi