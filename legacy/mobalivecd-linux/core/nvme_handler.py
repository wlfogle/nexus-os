"""
NVMe partition handler for MobaLiveCD Linux
Handles detection and validation of NVMe partitions for booting
"""

import os
import subprocess
import json
import re
from pathlib import Path


class NVMeHandler:
    """Handles NVMe partition detection and validation"""
    
    def __init__(self):
        self.nvme_devices = []
        self.nvme_partitions = []
    
    def detect_nvme_devices(self):
        """Detect all NVMe devices and their partitions"""
        self.nvme_devices = []
        self.nvme_partitions = []
        
        try:
            # Use lsblk to get NVMe device information
            result = subprocess.run([
                'lsblk', '-J', '-o', 'NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE,LABEL,UUID',
                '-d', '-e7'  # Exclude loop devices
            ], capture_output=True, text=True, check=True)
            
            data = json.loads(result.stdout)
            
            for device in data.get('blockdevices', []):
                device_name = device.get('name', '')
                if device_name.startswith('nvme'):
                    nvme_info = {
                        'device': f"/dev/{device_name}",
                        'name': device_name,
                        'size': device.get('size', 'Unknown'),
                        'partitions': []
                    }
                    
                    # Get partitions for this NVMe device
                    partitions = self._get_nvme_partitions(device_name)
                    nvme_info['partitions'] = partitions
                    self.nvme_partitions.extend(partitions)
                    
                    self.nvme_devices.append(nvme_info)
                    
        except (subprocess.CalledProcessError, json.JSONDecodeError, FileNotFoundError) as e:
            print(f"Warning: Could not detect NVMe devices: {e}")
            # Fallback method
            self._detect_nvme_fallback()
        
        return self.nvme_devices
    
    def _get_nvme_partitions(self, device_name):
        """Get partitions for a specific NVMe device"""
        partitions = []
        
        try:
            # Get detailed partition info for this device
            result = subprocess.run([
                'lsblk', '-J', '-o', 'NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE,LABEL,UUID,PARTUUID',
                f"/dev/{device_name}"
            ], capture_output=True, text=True, check=True)
            
            data = json.loads(result.stdout)
            
            for device in data.get('blockdevices', []):
                children = device.get('children', [])
                for partition in children:
                    part_name = partition.get('name', '')
                    if part_name.startswith('nvme') and 'p' in part_name:
                        part_info = {
                            'device': f"/dev/{part_name}",
                            'name': part_name,
                            'size': partition.get('size', 'Unknown'),
                            'fstype': partition.get('fstype', ''),
                            'label': partition.get('label', ''),
                            'uuid': partition.get('uuid', ''),
                            'partuuid': partition.get('partuuid', ''),
                            'mountpoint': partition.get('mountpoint', ''),
                            'parent_device': f"/dev/{device_name}",
                            'bootable': self._check_bootable(part_name, partition)
                        }
                        partitions.append(part_info)
                        
        except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
            print(f"Warning: Could not get partitions for {device_name}: {e}")
        
        return partitions
    
    def _detect_nvme_fallback(self):
        """Fallback NVMe detection method"""
        try:
            # Look for NVMe devices in /dev/
            for device_path in Path('/dev').glob('nvme*n*'):
                if device_path.is_block_device():
                    device_name = device_path.name
                    nvme_info = {
                        'device': str(device_path),
                        'name': device_name,
                        'size': self._get_device_size(str(device_path)),
                        'partitions': []
                    }
                    
                    # Find partitions
                    for part_path in Path('/dev').glob(f'{device_name}p*'):
                        if part_path.is_block_device():
                            part_info = {
                                'device': str(part_path),
                                'name': part_path.name,
                                'size': self._get_device_size(str(part_path)),
                                'fstype': '',
                                'label': '',
                                'uuid': '',
                                'partuuid': '',
                                'mountpoint': '',
                                'parent_device': str(device_path),
                                'bootable': self._check_bootable_fallback(str(part_path))
                            }
                            nvme_info['partitions'].append(part_info)
                            self.nvme_partitions.append(part_info)
                    
                    self.nvme_devices.append(nvme_info)
                    
        except Exception as e:
            print(f"Warning: Fallback NVMe detection failed: {e}")
    
    def _get_device_size(self, device_path):
        """Get device size using blockdev"""
        try:
            result = subprocess.run([
                'blockdev', '--getsize64', device_path
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                size_bytes = int(result.stdout.strip())
                return self._format_size(size_bytes)
        except:
            pass
        
        return "Unknown"
    
    def _format_size(self, size_bytes):
        """Format size in human readable format"""
        for unit in ['B', 'K', 'M', 'G', 'T']:
            if size_bytes < 1024.0:
                return f"{size_bytes:.1f}{unit}"
            size_bytes /= 1024.0
        return f"{size_bytes:.1f}P"
    
    def _check_bootable(self, part_name, partition_info):
        """Check if a partition is potentially bootable"""
        fstype = (partition_info.get('fstype') or '').lower()
        label = (partition_info.get('label') or '').lower()
        
        # Common bootable filesystem types
        bootable_fs = ['fat32', 'vfat', 'fat16', 'ntfs', 'ext2', 'ext3', 'ext4', 'btrfs', 'xfs']
        
        # Check filesystem type
        if fstype in bootable_fs:
            return True
        
        # Check for common boot partition labels
        boot_labels = ['boot', 'efi', 'system', 'recovery', 'windows', 'linux']
        if any(boot_label in label for boot_label in boot_labels):
            return True
        
        # Check if it's an EFI system partition (typically around 100-512MB)
        size_str = partition_info.get('size', '')
        if 'M' in size_str:
            try:
                size_mb = float(size_str.replace('M', ''))
                if 50 <= size_mb <= 1024 and fstype in ['fat32', 'vfat']:
                    return True
            except:
                pass
        
        return False
    
    def _check_bootable_fallback(self, device_path):
        """Fallback method to check if partition is bootable"""
        try:
            # Try to read the first sector
            with open(device_path, 'rb') as f:
                first_sector = f.read(512)
                # Check for boot signature (0x55AA at the end)
                if len(first_sector) >= 512 and first_sector[510:512] == b'\x55\xAA':
                    return True
        except:
            pass
        
        return False
    
    def get_bootable_partitions(self):
        """Get list of potentially bootable NVMe partitions"""
        if not self.nvme_partitions:
            self.detect_nvme_devices()
        
        return [p for p in self.nvme_partitions if p.get('bootable', False)]
    
    def get_all_partitions(self):
        """Get all NVMe partitions"""
        if not self.nvme_partitions:
            self.detect_nvme_devices()
        
        return self.nvme_partitions
    
    def is_nvme_partition(self, device_path):
        """Check if a device path is an NVMe partition"""
        return device_path.startswith('/dev/nvme') and 'p' in device_path
    
    def get_partition_info(self, device_path):
        """Get detailed information about a specific partition"""
        if not self.nvme_partitions:
            self.detect_nvme_devices()
        
        for partition in self.nvme_partitions:
            if partition['device'] == device_path:
                return partition
        
        return None
    
    def validate_nvme_partition(self, device_path):
        """Validate an NVMe partition for booting"""
        if not self.is_nvme_partition(device_path):
            return False, "Not an NVMe partition"
        
        if not os.path.exists(device_path):
            return False, "Partition does not exist"
        
        try:
            # Check if it's a block device
            import stat
            st = os.stat(device_path)
            if not stat.S_ISBLK(st.st_mode):
                return False, "Not a valid block device"
            
            # Get partition info
            partition_info = self.get_partition_info(device_path)
            if not partition_info:
                return False, "Could not get partition information"
            
            # Check if partition has some size
            size_str = partition_info.get('size', '')
            if size_str == 'Unknown' or size_str == '0B':
                return False, "Partition appears to be empty"
            
            # Check if it's mounted (warn but don't prevent)
            mountpoint = partition_info.get('mountpoint', '')
            if mountpoint:
                return True, f"Valid NVMe partition (currently mounted at {mountpoint})"
            
            return True, "Valid NVMe partition"
            
        except OSError as e:
            return False, f"Cannot access partition: {e}"
    
    def unmount_partition(self, device_path):
        """Safely unmount a partition if it's mounted"""
        try:
            partition_info = self.get_partition_info(device_path)
            if partition_info and partition_info.get('mountpoint'):
                print(f"Unmounting {device_path}...")
                result = subprocess.run(['sudo', 'umount', device_path], 
                                      capture_output=True, text=True)
                if result.returncode != 0:
                    print(f"Warning: Could not unmount {device_path}: {result.stderr}")
                else:
                    print(f"Successfully unmounted {device_path}")
        except Exception as e:
            print(f"Warning: Error during unmount of {device_path}: {e}")