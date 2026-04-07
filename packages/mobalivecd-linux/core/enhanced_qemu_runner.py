"""
AI-Enhanced QEMU Runner for MobaLiveCD Linux
Advanced virtualization engine with intelligent optimization and hardware detection
"""

import os
import subprocess
import shutil
import tempfile
import json
import re
import threading
import time
try:
    import psutil
    PSUTIL_AVAILABLE = True
except ImportError:
    PSUTIL_AVAILABLE = False
    # Create dummy psutil for basic functionality
    class DummyPsutil:
        @staticmethod
        def cpu_count(logical=True):
            return 4 if logical else 2
        @staticmethod
        def virtual_memory():
            class Memory:
                total = 8 * 1024**3  # 8GB default
            return Memory()
    psutil = DummyPsutil()
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass

try:
    from .nvme_handler import NVMeHandler
    NVME_SUPPORT = True
except ImportError:
    NVME_SUPPORT = False
    print("Warning: NVMe support not available")

@dataclass
class SystemCapabilities:
    """System hardware capabilities"""
    cpu_cores: int
    cpu_threads: int
    memory_gb: float
    kvm_available: bool
    gpu_acceleration: bool
    nested_virtualization: bool
    cpu_flags: List[str]
    architecture: str

@dataclass
class ISOProfile:
    """ISO-specific optimization profile"""
    name: str
    category: str
    memory_min: str
    memory_recommended: str
    cpu_cores: int
    enable_3d: bool
    enable_audio: bool
    network_mode: str
    boot_priority: str
    special_flags: List[str]
    description: str

class AIEnhancedQEMURunner:
    """AI-powered QEMU runner with intelligent optimization"""
    
    def __init__(self):
        self.qemu_binary = self._find_optimal_qemu_binary()
        self.system_caps = self._analyze_system_capabilities()
        self.iso_profiles = self._load_iso_profiles()
        self.active_processes = {}
        self._init_performance_monitoring()
        
        # Initialize NVMe support if available
        if NVME_SUPPORT:
            self.nvme_handler = NVMeHandler()
        else:
            self.nvme_handler = None
        
    def _find_optimal_qemu_binary(self) -> str:
        """Find the best QEMU binary for the system"""
        candidates = [
            ('qemu-system-x86_64', 10),  # Preferred for 64-bit
            ('qemu-system-i386', 8),     # Fallback for 32-bit
            ('qemu', 5),                 # Generic
            ('qemu-kvm', 9)              # KVM-specific variant
        ]
        
        available = []
        for binary, priority in candidates:
            if shutil.which(binary):
                # Check if it actually works
                try:
                    result = subprocess.run(
                        [binary, '--version'], 
                        capture_output=True, 
                        text=True, 
                        timeout=5
                    )
                    if result.returncode == 0:
                        available.append((binary, priority))
                except:
                    continue
        
        if not available:
            raise RuntimeError("No working QEMU binary found. Install qemu-system-x86 package")
        
        # Return highest priority binary
        return max(available, key=lambda x: x[1])[0]
    
    def _analyze_system_capabilities(self) -> SystemCapabilities:
        """Analyze system hardware capabilities using AI-driven detection"""
        
        # CPU Information
        cpu_info = {}
        try:
            with open('/proc/cpuinfo', 'r') as f:
                content = f.read()
                cpu_info = self._parse_cpuinfo(content)
        except:
            cpu_info = {'cores': psutil.cpu_count(logical=False), 'threads': psutil.cpu_count()}
        
        # Memory Information
        memory = psutil.virtual_memory()
        memory_gb = memory.total / (1024**3)
        
        # KVM Support
        kvm_available = (
            os.path.exists('/dev/kvm') and 
            os.access('/dev/kvm', os.R_OK | os.W_OK)
        )
        
        # GPU Acceleration Detection
        gpu_acceleration = self._detect_gpu_acceleration()
        
        # Nested Virtualization
        nested_virt = self._check_nested_virtualization()
        
        # CPU Flags
        cpu_flags = cpu_info.get('flags', [])
        
        return SystemCapabilities(
            cpu_cores=cpu_info.get('cores', 1),
            cpu_threads=cpu_info.get('threads', 1),
            memory_gb=memory_gb,
            kvm_available=kvm_available,
            gpu_acceleration=gpu_acceleration,
            nested_virtualization=nested_virt,
            cpu_flags=cpu_flags,
            architecture=os.uname().machine
        )
    
    def _parse_cpuinfo(self, content: str) -> Dict:
        """Parse /proc/cpuinfo with AI pattern recognition"""
        lines = content.strip().split('\n')
        info = {'cores': 1, 'threads': 1, 'flags': []}
        
        processor_count = 0
        core_ids = set()
        
        for line in lines:
            if ':' in line:
                key, value = line.split(':', 1)
                key = key.strip()
                value = value.strip()
                
                if key == 'processor':
                    processor_count += 1
                elif key == 'core id':
                    core_ids.add(value)
                elif key == 'flags':
                    info['flags'] = value.split()
        
        info['threads'] = processor_count
        info['cores'] = len(core_ids) if core_ids else processor_count
        
        return info
    
    def _detect_gpu_acceleration(self) -> bool:
        """Detect GPU acceleration capabilities"""
        gpu_indicators = [
            'nvidia',
            'amd',
            'intel',
            'virgl',
            'virtio-gpu'
        ]
        
        try:
            # Check lspci for GPU
            result = subprocess.run(['lspci'], capture_output=True, text=True)
            if result.returncode == 0:
                output = result.stdout.lower()
                return any(indicator in output for indicator in gpu_indicators)
        except:
            pass
        
        return False
    
    def _check_nested_virtualization(self) -> bool:
        """Check if nested virtualization is enabled"""
        try:
            # Check Intel
            if os.path.exists('/sys/module/kvm_intel/parameters/nested'):
                with open('/sys/module/kvm_intel/parameters/nested', 'r') as f:
                    return f.read().strip() in ['1', 'Y', 'y']
            
            # Check AMD
            if os.path.exists('/sys/module/kvm_amd/parameters/nested'):
                with open('/sys/module/kvm_amd/parameters/nested', 'r') as f:
                    return f.read().strip() in ['1', 'Y', 'y']
        except:
            pass
        
        return False
    
    def _load_iso_profiles(self) -> Dict[str, ISOProfile]:
        """Load AI-curated ISO optimization profiles"""
        profiles = {
            'proxmox': ISOProfile(
                name="Proxmox VE",
                category="Virtualization",
                memory_min="2G",
                memory_recommended="8G",
                cpu_cores=4,
                enable_3d=False,
                enable_audio=False,
                network_mode="virtio",
                boot_priority="d",
                special_flags=["-machine", "type=q35", "-cpu", "host,+vmx"],
                description="Enterprise virtualization platform"
            ),
            'debian': ISOProfile(
                name="Debian Live",
                category="Linux Distribution",
                memory_min="512M",
                memory_recommended="2G",
                cpu_cores=2,
                enable_3d=True,
                enable_audio=True,
                network_mode="virtio",
                boot_priority="d",
                special_flags=["-machine", "type=pc"],
                description="Stable Linux distribution"
            ),
            'systemrescue': ISOProfile(
                name="SystemRescue",
                category="Recovery",
                memory_min="1G",
                memory_recommended="4G",
                cpu_cores=2,
                enable_3d=False,
                enable_audio=False,
                network_mode="virtio",
                boot_priority="d",
                special_flags=["-machine", "type=pc"],
                description="System rescue and recovery toolkit"
            ),
            'windows': ISOProfile(
                name="Windows",
                category="Operating System",
                memory_min="2G",
                memory_recommended="8G",
                cpu_cores=2,
                enable_3d=True,
                enable_audio=True,
                network_mode="e1000",
                boot_priority="d",
                special_flags=["-machine", "type=q35"],
                description="Microsoft Windows operating system"
            ),
            'gaming': ISOProfile(
                name="Gaming Linux",
                category="Gaming",
                memory_min="4G",
                memory_recommended="8G",
                cpu_cores=4,
                enable_3d=True,
                enable_audio=True,
                network_mode="virtio",
                boot_priority="d",
                special_flags=["-machine", "type=q35", "-vga", "virtio"],
                description="Gaming-optimized Linux distribution"
            ),
            'nvme_linux': ISOProfile(
                name="NVMe Linux Partition",
                category="Installed OS",
                memory_min="4G",
                memory_recommended="8G",
                cpu_cores=4,
                enable_3d=True,
                enable_audio=True,
                network_mode="virtio",
                boot_priority="c",  # Boot from hard disk
                special_flags=["-machine", "type=q35", "-bios", "/usr/share/ovmf/OVMF.fd"],
                description="Boot from NVMe partition with UEFI support"
            ),
            'nvme_windows': ISOProfile(
                name="NVMe Windows Partition",
                category="Installed OS",
                memory_min="4G",
                memory_recommended="12G",
                cpu_cores=4,
                enable_3d=True,
                enable_audio=True,
                network_mode="e1000",
                boot_priority="c",  # Boot from hard disk
                special_flags=["-machine", "type=q35", "-bios", "/usr/share/ovmf/OVMF.fd"],
                description="Boot from NVMe Windows partition with UEFI support"
            ),
            'nvme_generic': ISOProfile(
                name="NVMe Generic Partition",
                category="Installed OS",
                memory_min="2G",
                memory_recommended="8G",
                cpu_cores=2,
                enable_3d=True,
                enable_audio=True,
                network_mode="virtio",
                boot_priority="c",  # Boot from hard disk
                special_flags=["-machine", "type=q35"],
                description="Generic NVMe partition boot (Legacy BIOS)"
            )
        }
        
        return profiles
    
    def _init_performance_monitoring(self):
        """Initialize performance monitoring subsystem"""
        self.performance_thread = threading.Thread(
            target=self._monitor_performance,
            daemon=True
        )
        self.performance_thread.start()
    
    def _monitor_performance(self):
        """Background performance monitoring"""
        while True:
            try:
                # Monitor active QEMU processes
                for pid, info in list(self.active_processes.items()):
                    try:
                        process = psutil.Process(pid)
                        if not process.is_running():
                            del self.active_processes[pid]
                            continue
                        
                        # Update performance metrics
                        cpu_percent = process.cpu_percent()
                        memory_info = process.memory_info()
                        
                        info.update({
                            'cpu_percent': cpu_percent,
                            'memory_mb': memory_info.rss / (1024*1024),
                            'status': process.status()
                        })
                        
                    except psutil.NoSuchProcess:
                        if pid in self.active_processes:
                            del self.active_processes[pid]
                
                time.sleep(5)  # Check every 5 seconds
                
            except Exception as e:
                print(f"Performance monitoring error: {e}")
                time.sleep(10)
    
    def identify_boot_source(self, boot_path: str) -> Tuple[str, ISOProfile]:
        """AI-powered boot source identification and profile selection"""
        
        # Check if it's an NVMe partition
        if self._is_nvme_partition(boot_path):
            return self._identify_nvme_partition(boot_path)
        
        # Otherwise treat as ISO
        return self._identify_iso_file(boot_path)
    
    def _is_nvme_partition(self, path: str) -> bool:
        """Check if path is an NVMe partition"""
        return (path.startswith('/dev/nvme') and 'p' in path and 
                self.nvme_handler and self.nvme_handler.is_nvme_partition(path))
    
    def _identify_nvme_partition(self, nvme_path: str) -> Tuple[str, ISOProfile]:
        """Identify NVMe partition type and select appropriate profile"""
        if not self.nvme_handler:
            return 'nvme_generic', self.iso_profiles['nvme_generic']
        
        # Get partition info
        partition_info = self.nvme_handler.get_partition_info(nvme_path)
        if not partition_info:
            return 'nvme_generic', self.iso_profiles['nvme_generic']
        
        fstype = (partition_info.get('fstype') or '').lower()
        label = (partition_info.get('label') or '').lower()
        
        # Identify OS type based on filesystem and label
        if 'ntfs' in fstype or 'windows' in label or 'microsoft' in label:
            return 'nvme_windows', self.iso_profiles['nvme_windows']
        elif fstype in ['ext4', 'ext3', 'ext2', 'btrfs', 'xfs', 'zfs', 'zfs_member'] or 'linux' in label:
            return 'nvme_linux', self.iso_profiles['nvme_linux']
        else:
            # Default to generic for unknown filesystems
            return 'nvme_generic', self.iso_profiles['nvme_generic']
    
    def _identify_iso_file(self, iso_path: str) -> Tuple[str, ISOProfile]:
        """AI-powered ISO identification and profile selection"""
        iso_name = os.path.basename(iso_path).lower()
        
        # Pattern matching with AI logic
        patterns = {
            r'proxmox.*ve.*\d+': 'proxmox',
            r'debian.*live': 'debian',
            r'systemrescue': 'systemrescue',
            r'win.*\d+': 'windows',
            r'garuda.*gaming': 'gaming',
            r'fedora.*workstation': 'debian',  # Use Debian profile as base
            r'ubuntu.*desktop': 'debian',
            r'mint.*cinnamon': 'debian',
            r'rescue|recovery|repair': 'systemrescue'
        }
        
        for pattern, profile_key in patterns.items():
            if re.search(pattern, iso_name):
                return profile_key, self.iso_profiles[profile_key]
        
        # Default to Debian profile for unknown ISOs
        return 'debian', self.iso_profiles['debian']
    
    # Keep backward compatibility
    def identify_iso(self, iso_path: str) -> Tuple[str, ISOProfile]:
        """Backward compatibility wrapper"""
        return self.identify_boot_source(iso_path)
    
    def calculate_optimal_resources(self, profile: ISOProfile) -> Dict[str, str]:
        """Calculate optimal resource allocation based on system capabilities"""
        
        # Memory calculation
        available_memory_gb = self.system_caps.memory_gb * 0.7  # Leave 30% for host
        recommended_memory = self._parse_memory_string(profile.memory_recommended)
        min_memory = self._parse_memory_string(profile.memory_min)
        
        if available_memory_gb >= recommended_memory:
            memory = profile.memory_recommended
        elif available_memory_gb >= min_memory:
            memory = f"{int(available_memory_gb)}G"
        else:
            memory = profile.memory_min
        
        # CPU calculation
        available_cores = self.system_caps.cpu_cores
        recommended_cores = min(profile.cpu_cores, max(1, available_cores // 2))
        
        return {
            'memory': memory,
            'cpu_cores': str(recommended_cores),
            'enable_kvm': str(self.system_caps.kvm_available),
            'enable_gpu': str(profile.enable_3d and self.system_caps.gpu_acceleration)
        }
    
    def _parse_memory_string(self, memory_str: str) -> float:
        """Parse memory string to GB"""
        memory_str = memory_str.upper()
        if 'G' in memory_str:
            return float(memory_str.replace('G', ''))
        elif 'M' in memory_str:
            return float(memory_str.replace('M', '')) / 1024
        return 1.0  # Default
    
    def build_optimized_command(self, boot_path: str, **user_options) -> List[str]:
        """Build AI-optimized QEMU command for any boot source"""
        
        # Identify boot source and get profile
        profile_key, profile = self.identify_boot_source(boot_path)
        optimal_resources = self.calculate_optimal_resources(profile)
        
        # Start building command
        cmd = [self.qemu_binary]
        
        # Machine type from profile
        if profile.special_flags:
            cmd.extend(profile.special_flags)
        else:
            cmd.extend(['-machine', 'type=pc'])
        
        # Memory optimization
        memory = user_options.get('memory', optimal_resources['memory'])
        cmd.extend(['-m', memory])
        
        # CPU optimization
        cpu_cores = user_options.get('cpu_cores', optimal_resources['cpu_cores'])
        cmd.extend(['-smp', cpu_cores])
        
        # Acceleration
        if self.system_caps.kvm_available and user_options.get('enable_kvm', True):
            cmd.extend(['-accel', 'kvm'])
            cmd.extend(['-cpu', 'host'])
        else:
            cmd.extend(['-accel', 'tcg'])
        
        # Display optimization
        if profile.enable_3d and self.system_caps.gpu_acceleration:
            cmd.extend(['-vga', 'virtio'])
            cmd.extend(['-display', 'gtk,gl=on'])
        else:
            cmd.extend(['-vga', 'std'])
            cmd.extend(['-display', 'gtk'])
        
        # Audio
        if profile.enable_audio and user_options.get('enable_audio', True):
            cmd.extend(['-audiodev', 'alsa,id=audio0'])
            cmd.extend(['-device', 'AC97,audiodev=audio0'])
        
        # Network optimization
        if user_options.get('enable_network', True):
            if profile.network_mode == 'virtio':
                cmd.extend(['-netdev', 'user,id=net0'])
                cmd.extend(['-device', 'virtio-net,netdev=net0'])
            else:
                cmd.extend(['-netdev', 'user,id=net0'])
                cmd.extend(['-device', 'rtl8139,netdev=net0'])
        
        # USB and tablet for better mouse handling
        cmd.extend(['-usb', '-device', 'usb-tablet'])
        
        # Boot configuration based on source type
        if self._is_nvme_partition(boot_path):
            # NVMe partition - boot from hard disk
            cmd.extend(['-boot', profile.boot_priority])
            
            # Add NVMe partition as primary drive
            if 'nvme_linux' in profile_key or 'nvme_generic' in profile_key:
                # For Linux/ZFS, use virtio for better performance
                cmd.extend(['-drive', f'file={boot_path},format=raw,cache=none,if=virtio'])
            else:
                # For Windows, use IDE for better compatibility
                cmd.extend(['-drive', f'file={boot_path},format=raw,cache=none,if=ide'])
            
            # Add EFI support for UEFI boot (crucial for ZFS)
            if '-bios' not in ' '.join(profile.special_flags or []):
                # Check for OVMF availability (Arch/Garuda Linux paths)
                ovmf_paths = [
                    '/usr/share/edk2/x64/OVMF_CODE.4m.fd',  # Arch/Garuda Linux
                    '/usr/share/edk2/x64/OVMF.4m.fd',       # Arch/Garuda Linux alternative
                    '/usr/share/ovmf/OVMF.fd',              # Generic path
                    '/usr/share/edk2-ovmf/OVMF_CODE.fd',    # Ubuntu/Debian path
                    '/usr/share/qemu/edk2-x86_64-code.fd'   # CentOS/RHEL path
                ]
                for ovmf_path in ovmf_paths:
                    if os.path.exists(ovmf_path):
                        cmd.extend(['-bios', ovmf_path])
                        break
                else:
                    print("⚠️ Warning: OVMF UEFI firmware not found. Install ovmf package for UEFI support.")
            
            # Unmount partition if needed
            if self.nvme_handler:
                try:
                    self.nvme_handler.unmount_partition(boot_path)
                except Exception as e:
                    print(f"Warning: Could not unmount {boot_path}: {e}")
        else:
            # ISO file - boot from CD-ROM
            cmd.extend(['-boot', profile.boot_priority])
            cmd.extend(['-cdrom', boot_path])
        
        # Performance optimizations
        cmd.extend(['-no-reboot'])
        cmd.extend(['-rtc', 'base=localtime,clock=host'])
        
        return cmd
    
    def validate_boot_source(self, boot_path: str) -> tuple[bool, str]:
        """Validate a boot source (ISO file, USB device, or NVMe partition)"""
        
        if not os.path.exists(boot_path):
            source_type = "NVMe partition" if self._is_nvme_partition(boot_path) else "ISO file"
            return False, f"{source_type} not found: {boot_path}"
        
        # Validate NVMe partition if applicable
        if self._is_nvme_partition(boot_path) and self.nvme_handler:
            is_valid, message = self.nvme_handler.validate_nvme_partition(boot_path)
            return is_valid, message
        
        # For ISO files and other sources, just check if file exists
        return True, "Boot source is valid"
    
    def run_boot_source(self, boot_path: str, **options) -> int:
        """Run any boot source (ISO or NVMe partition) with AI optimization"""
        
        # Validate boot source first
        is_valid, message = self.validate_boot_source(boot_path)
        if not is_valid:
            raise RuntimeError(message)
        
        if self._is_nvme_partition(boot_path):
            print(f"✅ {message}")
        
        # Build optimized command
        cmd = self.build_optimized_command(boot_path, **options)
        
        # Check for sudo requirement for NVMe partitions
        if self._is_nvme_partition(boot_path):
            if not self._can_access_device(boot_path):
                print("🔐 NVMe partition requires elevated permissions, using sudo...")
                cmd = ['sudo'] + cmd
        
        # Log command for debugging
        print(f"🚀 AI-Optimized QEMU Command:")
        print(f"   {' '.join(cmd)}")
        
        # Get profile info for display
        profile_key, profile = self.identify_boot_source(boot_path)
        print(f"📊 Detected: {profile.name} ({profile.category})")
        print(f"💡 {profile.description}")
        
        # Special messaging for ZFS
        if self._is_nvme_partition(boot_path) and self.nvme_handler:
            partition_info = self.nvme_handler.get_partition_info(boot_path)
            fstype = (partition_info.get('fstype') or '').lower() if partition_info else ''
            if 'zfs' in fstype:  # Matches both 'zfs' and 'zfs_member'
                print("🌊 ZFS partition detected - UEFI boot enabled for compatibility")
        
        try:
            # Launch QEMU
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE if options.get('quiet', True) else None,
                stderr=subprocess.PIPE
            )
            
            # Give QEMU time to start
            time.sleep(2)
            
            if process.poll() is not None:
                # Process terminated - error occurred
                stdout, stderr = process.communicate()
                error_msg = stderr.decode('utf-8') if stderr else "QEMU failed to start"
                raise RuntimeError(f"QEMU startup failed: {error_msg}")
            
            # Add to monitoring
            self.active_processes[process.pid] = {
                'boot_path': boot_path,
                'profile': profile_key,
                'start_time': time.time(),
                'command': cmd,
                'is_nvme': self._is_nvme_partition(boot_path)
            }
            
            print(f"✅ QEMU started successfully (PID: {process.pid})")
            return process.pid
            
        except FileNotFoundError:
            raise RuntimeError(f"QEMU binary '{self.qemu_binary}' not found")
        except Exception as e:
            raise RuntimeError(f"Failed to start QEMU: {e}")
    
    def _can_access_device(self, device_path: str) -> bool:
        """Check if we can access the device without sudo"""
        try:
            with open(device_path, 'rb') as f:
                f.read(1)
            return True
        except (OSError, PermissionError):
            return False
    
    def run_optimized_iso(self, iso_path: str, **options) -> int:
        """Run ISO with AI optimization (backward compatibility wrapper)"""
        return self.run_boot_source(iso_path, **options)
    
    def get_system_diagnostics(self) -> Dict:
        """Get comprehensive system diagnostics"""
        return {
            'system_capabilities': {
                'cpu_cores': self.system_caps.cpu_cores,
                'cpu_threads': self.system_caps.cpu_threads,
                'memory_gb': round(self.system_caps.memory_gb, 2),
                'kvm_available': self.system_caps.kvm_available,
                'gpu_acceleration': self.system_caps.gpu_acceleration,
                'nested_virtualization': self.system_caps.nested_virtualization,
                'architecture': self.system_caps.architecture
            },
            'qemu_info': {
                'binary': self.qemu_binary,
                'version': self._get_qemu_version()
            },
            'active_vms': len(self.active_processes),
            'performance': {
                pid: info for pid, info in self.active_processes.items()
            }
        }
    
    def _get_qemu_version(self) -> str:
        """Get QEMU version"""
        try:
            result = subprocess.run(
                [self.qemu_binary, '--version'],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                return result.stdout.strip().split('\n')[0]
        except:
            pass
        return 'Unknown'
    
    def kill_vm(self, pid: int) -> bool:
        """Gracefully terminate a VM"""
        try:
            process = psutil.Process(pid)
            process.terminate()
            
            # Wait up to 10 seconds for graceful shutdown
            try:
                process.wait(timeout=10)
            except psutil.TimeoutExpired:
                # Force kill if needed
                process.kill()
            
            if pid in self.active_processes:
                del self.active_processes[pid]
            
            return True
        except psutil.NoSuchProcess:
            return True
        except Exception as e:
            print(f"Error killing VM {pid}: {e}")
            return False
