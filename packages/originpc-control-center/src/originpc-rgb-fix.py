#!/usr/bin/env python3
"""
Minimal RGB Clear - Single Command Approach
===========================================
The simplest possible approach to clear kp_plus.
"""

import os
import time

def minimal_clear():
    """Send only the most basic clear command"""
    print("🔧 Minimal kp_plus clear - single command")
    
    try:
        # Wait for device to be ready
        print("   Waiting for device stability...")
        time.sleep(2)
        
        # Send single clear command to primary kp_plus location
        with open('/dev/hidraw0', 'wb') as device:
            data = bytes([0xCC, 0x01, 0x53, 0, 0, 0] + [0x00] * 10)
            device.write(data)
            device.flush()
        
        print("✅ Minimal clear sent successfully")
        return True
        
    except Exception as e:
        print(f"❌ Minimal clear failed: {e}")
        return False

def test_device_status():
    """Check if device is accessible"""
    try:
        # Just try to open the device
        with open('/dev/hidraw0', 'rb') as device:
            pass
        print("✅ Device is accessible for reading")
        
        with open('/dev/hidraw0', 'wb') as device:
            pass
        print("✅ Device is accessible for writing")
        return True
        
    except Exception as e:
        print(f"❌ Device access failed: {e}")
        return False

if __name__ == "__main__":
    print("🔧 Minimal RGB Clear Test")
    print("=" * 30)
    
    # Test device first
    if test_device_status():
        # Try minimal clear
        minimal_clear()
    else:
        print("❌ Device not ready for RGB commands")
        
    print("\nIf this fails with protocol error, the issue is:")
    print("1. Hardware-level RGB lock during lid events")
    print("2. Firmware exclusive control")
    print("3. Need to wait longer after lid events")
    print("\nTry running this 10-30 seconds after opening lid.")

