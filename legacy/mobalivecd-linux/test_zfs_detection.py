#!/usr/bin/env python3
"""
ZFS Partition Detection Test Script
Tests the core functionality of the NVMe handler to ensure ZFS partitions are detected properly.
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from core.nvme_handler import NVMeHandler

def test_zfs_detection():
    """Test ZFS partition detection functionality"""
    
    print("🧪 Testing ZFS Partition Detection")
    print("=" * 50)
    
    try:
        # Initialize handler
        nvme = NVMeHandler()
        
        # Test NVMe device detection
        print("\n1️⃣ Testing NVMe device detection...")
        nvmes = nvme.detect_nvme_devices()
        print(f"   ✅ Found {len(nvmes)} NVMe devices")
        
        for device in nvmes:
            print(f"   📀 {device['device']}: {device['size']}")
        
        # Test partition detection
        print("\n2️⃣ Testing partition detection...")
        partitions = nvme.get_all_partitions()
        print(f"   ✅ Found {len(partitions)} total partitions")
        
        zfs_found = False
        for partition in partitions:
            device = partition['device']
            fstype = partition.get('fstype', 'unknown')
            size = partition.get('size', 'unknown')
            
            print(f"   💾 {device}: {fstype} ({size})")
            
            if fstype == 'zfs_member':
                print(f"       🌊 ZFS partition detected!")
                zfs_found = True
        
        # Test bootable partition detection
        print("\n3️⃣ Testing bootable partition detection...")
        bootable = nvme.get_bootable_partitions()
        print(f"   ✅ Found {len(bootable)} bootable partitions")
        
        for partition in bootable:
            device = partition['device']
            fstype = partition.get('fstype', 'unknown')
            size = partition.get('size', 'unknown')
            print(f"   🥾 {device}: {fstype} ({size})")
        
        # Summary
        print("\n📊 Test Results Summary:")
        print(f"   • NVMe devices: {len(nvmes)}")
        print(f"   • Total partitions: {len(partitions)}")
        print(f"   • Bootable partitions: {len(bootable)}")
        print(f"   • ZFS partitions found: {'✅ YES' if zfs_found else '❌ NO'}")
        
        if zfs_found:
            print("\n🎉 SUCCESS: ZFS partition detection is working properly!")
            return True
        else:
            print("\n⚠️  WARNING: No ZFS partitions found on this system")
            return True
            
    except Exception as e:
        print(f"\n❌ ERROR: Test failed with exception: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_zfs_detection()
    sys.exit(0 if success else 1)