#!/usr/bin/env python3
"""
Gentle RGB Clear for EON17-X
=============================
A more gentle approach to clearing RGB that won't overwhelm the device.
Designed to work around protocol errors and hardware limitations.
"""

import os
import time

def gentle_kp_plus_clear():
    """Gentle but persistent kp_plus clearing"""
    print("🧹 Starting GENTLE kp_plus clearing...")
    
    # Primary kp_plus indices (most likely locations)
    primary_indices = [0x53, 0x93, 0xB3]
    
    # Extended indices (all possible locations)
    extended_indices = [0x33, 0x73, 0xD3, 0xF3, 0x13, 0x54, 0x34, 0x74, 0x94, 0xB4]
    
    device_path = '/dev/hidraw0'
    
    try:
        # Phase 1: Gentle primary clearing (multiple short sessions)
        for session in range(1, 6):
            print(f"   Session {session}/5 - Primary indices")
            
            with open(device_path, 'wb') as device:
                for idx in primary_indices:
                    # Clear exact index
                    data = bytes([0xCC, 0x01, idx, 0, 0, 0] + [0x00] * 10)
                    device.write(data)
                    time.sleep(0.01)  # Small delay between commands
                    
                    # Clear nearby indices gently
                    for offset in [-2, -1, 1, 2]:
                        clear_idx = max(0, min(0xFF, idx + offset))
                        data = bytes([0xCC, 0x01, clear_idx, 0, 0, 0] + [0x00] * 10)
                        device.write(data)
                        time.sleep(0.01)
                
                device.flush()
            
            # Wait between sessions to avoid overwhelming device
            time.sleep(0.2)
        
        # Phase 2: Extended clearing (even more gentle)
        print("   Extended clearing - all possible indices")
        
        with open(device_path, 'wb') as device:
            for idx in extended_indices:
                data = bytes([0xCC, 0x01, idx, 0, 0, 0] + [0x00] * 10)
                device.write(data)
                time.sleep(0.02)  # Longer delay for extended clearing
            
            device.flush()
        
        # Phase 3: Reset commands (very gentle)
        print("   Reset commands")
        
        time.sleep(0.5)  # Wait before reset commands
        
        with open(device_path, 'wb') as device:
            # Send gentle reset commands
            reset_commands = [
                bytes([0xCC, 0x00, 0x53, 0x00, 0x00, 0x00] + [0x00] * 10),  # Reset primary kp_plus
                bytes([0xCC, 0x01, 0x53, 0x00, 0x00, 0x00] + [0x00] * 10),  # Clear primary kp_plus
            ]
            
            for reset_cmd in reset_commands:
                device.write(reset_cmd)
                time.sleep(0.1)  # Longer delay for reset commands
                device.flush()
        
        print("✅ Gentle kp_plus clearing completed successfully")
        return True
        
    except Exception as e:
        print(f"❌ Gentle clearing failed: {e}")
        return False

def check_hardware_rgb_state():
    """Check if there are any hardware-level RGB settings"""
    print("🔍 Checking for hardware-level RGB settings...")
    
    # Check for UEFI variables related to RGB
    uefi_paths = [
        "/sys/firmware/efi/efivars",
        "/sys/firmware/efi/vars"
    ]
    
    for path in uefi_paths:
        if os.path.exists(path):
            print(f"   Found UEFI variables at: {path}")
            try:
                # Look for RGB-related UEFI variables
                files = os.listdir(path)
                rgb_files = [f for f in files if any(term in f.lower() for term in ['rgb', 'light', 'color', 'keyboard'])]
                if rgb_files:
                    print(f"   Found {len(rgb_files)} potentially RGB-related UEFI variables:")
                    for f in rgb_files[:5]:  # Show first 5
                        print(f"     {f}")
                else:
                    print("   No RGB-related UEFI variables found")
            except PermissionError:
                print("   Permission denied accessing UEFI variables")
            except Exception as e:
                print(f"   Error checking UEFI variables: {e}")
    
    # Check for kernel RGB modules
    print("🔍 Checking for kernel RGB modules...")
    try:
        result = os.popen("lsmod | grep -i rgb").read().strip()
        if result:
            print(f"   RGB kernel modules found:\n{result}")
        else:
            print("   No RGB kernel modules found")
    except:
        print("   Could not check kernel modules")
    
    # Check for ACPI RGB methods
    print("🔍 Checking for ACPI RGB methods...")
    acpi_paths = [
        "/proc/acpi",
        "/sys/firmware/acpi/tables"
    ]
    
    for path in acpi_paths:
        if os.path.exists(path):
            print(f"   ACPI interface available at: {path}")
            break
    else:
        print("   No ACPI interface found")

def test_device_communication():
    """Test basic device communication"""
    print("🧪 Testing device communication...")
    
    try:
        with open('/dev/hidraw0', 'wb') as device:
            # Send a simple test command
            test_data = bytes([0xCC, 0x01, 0x01, 0, 0, 0] + [0x00] * 10)
            device.write(test_data)
            device.flush()
            time.sleep(0.1)
            
            # Clear it
            clear_data = bytes([0xCC, 0x01, 0x01, 0, 0, 0] + [0x00] * 10)
            device.write(clear_data)
            device.flush()
        
        print("✅ Device communication test passed")
        return True
    except Exception as e:
        print(f"❌ Device communication test failed: {e}")
        return False

if __name__ == "__main__":
    print("🔧 Gentle RGB Clear for EON17-X")
    print("=" * 40)
    
    # Test device communication first
    if not test_device_communication():
        print("❌ Device communication failed. Exiting.")
        exit(1)
    
    # Check for hardware-level RGB settings
    check_hardware_rgb_state()
    print()
    
    # Perform gentle clearing
    success = gentle_kp_plus_clear()
    
    if success:
        print("\n✅ Gentle RGB clearing completed.")
        print("   If kp_plus is still cyan, this may be a hardware/firmware issue.")
    else:
        print("\n❌ Gentle RGB clearing failed.")
        print("   The device may have hardware-level RGB persistence.")

