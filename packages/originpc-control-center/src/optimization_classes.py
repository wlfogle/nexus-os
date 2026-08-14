#!/usr/bin/env python3
"""
Optimization Classes for OriginPC Enhanced Professional Control Center
=====================================================================
This module provides optimization classes that improve performance and add
new functionality to the Enhanced Control Center:

- RGBCommandBatcher: Batches RGB commands for improved performance
- SystemInfoCache: Caches system information to reduce system queries
- DeviceManager: Provides multi-device support for RGB control
- MacroRecorder: Records and plays back RGB effect macros
"""

import os
import time
import json
import queue
import threading
import subprocess
from pathlib import Path
import psutil
from typing import Dict, List, Tuple, Optional, Union, Callable
from datetime import datetime, timedelta
import pickle
import glob

class RGBCommandBatcher:
    """
    Batches RGB commands for improved performance.
    
    Instead of sending each RGB command individually, which introduces
    latency and overhead, this class batches commands and sends them
    in optimized groups for better performance and smoother effects.
    """
    
    def __init__(self, device_path: str = "/dev/hidraw0", batch_size: int = 16, max_delay: float = 0.05):
        """
        Initialize the RGB command batcher.
        
        Args:
            device_path: Path to the RGB device
            batch_size: Maximum number of commands to batch before sending
            max_delay: Maximum delay in seconds before sending a partial batch
        """
        self.device_path = device_path
        self.batch_size = batch_size
        self.max_delay = max_delay
        self.command_queue = queue.Queue()
        self.batch_thread = None
        self.running = False
        self.last_batch_time = time.time()
        self.lock = threading.Lock()
        self.alternate_devices = []
        self.fallback_device_paths = ["/dev/hidraw1", "/dev/hidraw2", "/dev/hidraw3"]
        self.current_device_index = 0
        self.device_write_failures = 0
        self.max_failures = 5
        
    def start(self):
        """Start the batch processing thread."""
        if not self.running:
            self.running = True
            self.batch_thread = threading.Thread(target=self._process_batch_queue, daemon=True)
            self.batch_thread.start()
            return True
        return False
    
    def stop(self):
        """Stop the batch processing thread."""
        self.running = False
        if self.batch_thread and self.batch_thread.is_alive():
            self.batch_thread.join(timeout=1.0)
        return True
    
    def add_command(self, key_index: int, red: int, green: int, blue: int, priority: int = 0):
        """
        Add an RGB command to the batch queue.
        
        Args:
            key_index: Index of the key to control
            red, green, blue: RGB color values (0-255)
            priority: Command priority (higher = processed first)
        
        Returns:
            True if command was added to queue
        """
        try:
            command = {
                'key_index': key_index,
                'red': max(0, min(255, red)),
                'green': max(0, min(255, green)),
                'blue': max(0, min(255, blue)),
                'priority': priority,
                'timestamp': time.time()
            }
            self.command_queue.put(command)
            
            # If queue gets too large, process immediately
            if self.command_queue.qsize() > self.batch_size * 2:
                self._process_batch_now()
                
            return True
        except Exception as e:
            print(f"Error adding RGB command: {e}")
            return False
    
    def add_key_color(self, key_name: str, keyboard_map: dict, red: int, green: int, blue: int, priority: int = 0):
        """
        Add a key color command using key name instead of index.
        
        Args:
            key_name: Name of the key (e.g., 'a', 'enter')
            keyboard_map: Dictionary mapping key names to indices
            red, green, blue: RGB color values (0-255)
            priority: Command priority
        
        Returns:
            True if command was added
        """
        key_lower = key_name.lower()
        if key_lower in keyboard_map:
            key_index = keyboard_map[key_lower]
            return self.add_command(key_index, red, green, blue, priority)
        return False
    
    def add_group_colors(self, key_group: List[str], keyboard_map: dict, red: int, green: int, blue: int, priority: int = 0):
        """
        Add color commands for a group of keys.
        
        Args:
            key_group: List of key names
            keyboard_map: Dictionary mapping key names to indices
            red, green, blue: RGB color values (0-255)
            priority: Command priority
        
        Returns:
            Number of commands successfully added
        """
        count = 0
        for key in key_group:
            if self.add_key_color(key, keyboard_map, red, green, blue, priority):
                count += 1
        return count
    
    def clear_queue(self):
        """Clear all pending commands in the queue."""
        with self.lock:
            while not self.command_queue.empty():
                try:
                    self.command_queue.get_nowait()
                    self.command_queue.task_done()
                except queue.Empty:
                    break
        return True
    
    def _process_batch_now(self):
        """Force immediate processing of the current batch."""
        self.last_batch_time = 0  # Force immediate processing
    
    def _process_batch_queue(self):
        """Process the command queue in batches."""
        while self.running:
            try:
                current_time = time.time()
                # Process batch if we have enough commands or enough time has passed
                if (self.command_queue.qsize() >= self.batch_size or 
                    (current_time - self.last_batch_time >= self.max_delay and not self.command_queue.empty())):
                    
                    batch = []
                    with self.lock:
                        # Get up to batch_size commands
                        for _ in range(min(self.batch_size, self.command_queue.qsize())):
                            try:
                                cmd = self.command_queue.get_nowait()
                                batch.append(cmd)
                                self.command_queue.task_done()
                            except queue.Empty:
                                break
                    
                    # Sort batch by priority (high to low) then by timestamp (old to new)
                    batch.sort(key=lambda x: (-x['priority'], x['timestamp']))
                    
                    # Send the batch
                    if batch:
                        self._send_batch(batch)
                        self.last_batch_time = current_time
                        
                # Small sleep to prevent CPU spinning
                time.sleep(0.005)
                
            except Exception as e:
                print(f"Error in batch processing: {e}")
                time.sleep(0.1)  # Sleep longer on error
    
    def _send_batch(self, batch: List[dict]):
        """
        Send a batch of RGB commands to the device.
        
        Args:
            batch: List of command dictionaries
        
        Returns:
            Number of commands successfully sent
        """
        success_count = 0
        device = None
        
        try:
            device_path = self._get_current_device_path()
            
            if not os.path.exists(device_path):
                self._try_fallback_device()
                device_path = self._get_current_device_path()
            
            if not os.path.exists(device_path):
                raise FileNotFoundError(f"No RGB devices found")
                
            try:
                device = open(device_path, 'wb')
                
                for cmd in batch:
                    try:
                        data = bytes([0xCC, 0x01, cmd['key_index'], 
                                    cmd['red'], cmd['green'], cmd['blue']] + [0x00] * 10)
                        device.write(data)
                        success_count += 1
                    except Exception as e:
                        print(f"Error sending RGB command: {e}")
                
                # Reset failure counter on success
                if success_count > 0:
                    self.device_write_failures = 0
                else:
                    self.device_write_failures += 1
                    
                # Try fallback device if too many failures
                if self.device_write_failures >= self.max_failures:
                    self._try_fallback_device()
                    
            except (PermissionError, IOError) as e:
                print(f"Device access error: {e}")
                self.device_write_failures += 1
                self._try_fallback_device()
                
        except Exception as e:
            print(f"Batch sending error: {e}")
            
        finally:
            if device:
                try:
                    device.close()
                except:
                    pass
                
        return success_count
    
    def _get_current_device_path(self):
        """Get the current device path to use."""
        if self.current_device_index == 0:
            return self.device_path
        else:
            idx = self.current_device_index - 1
            if idx < len(self.fallback_device_paths):
                return self.fallback_device_paths[idx]
        return self.device_path  # Default fallback
    
    def _try_fallback_device(self):
        """Try to switch to a fallback device."""
        self.current_device_index = (self.current_device_index + 1) % (len(self.fallback_device_paths) + 1)
        self.device_write_failures = 0
        print(f"Switching to device: {self._get_current_device_path()}")
    
    def discover_devices(self):
        """Discover available RGB devices."""
        devices = []
        for i in range(10):  # Check hidraw0 through hidraw9
            path = f"/dev/hidraw{i}"
            if os.path.exists(path) and os.access(path, os.R_OK | os.W_OK):
                devices.append(path)
        
        if devices:
            self.device_path = devices[0]  # Primary device
            self.fallback_device_paths = devices[1:]  # Fallback devices
            return devices
        return []


class SystemInfoCache:
    """
    Caches system information to reduce system queries.
    
    Instead of querying the system for information repeatedly, this class
    caches results for configurable time periods, reducing system load
    and improving performance.
    """
    
    def __init__(self, cache_dir: str = None):
        """
        Initialize the system information cache.
        
        Args:
            cache_dir: Directory to store persistent cache (optional)
        """
        self.cache = {}
        self.cache_times = {}
        self.ttl = {
            'cpu_info': 5,         # CPU info TTL: 5 seconds
            'memory_info': 2,      # Memory info TTL: 2 seconds
            'disk_info': 30,       # Disk info TTL: 30 seconds
            'temperature': 3,      # Temperature TTL: 3 seconds
            'fan_speeds': 5,       # Fan speeds TTL: 5 seconds
            'battery_info': 10,    # Battery info TTL: 10 seconds
            'system_load': 2,      # System load TTL: 2 seconds
            'process_list': 5,     # Process list TTL: 5 seconds
            'network_info': 2,     # Network info TTL: 2 seconds
            'hw_info': 3600,       # Hardware info TTL: 1 hour
            'user_info': 300,      # User info TTL: 5 minutes
            'device_info': 60,     # Device info TTL: 1 minute
        }
        self.locks = {key: threading.Lock() for key in self.ttl.keys()}
        self.cache_dir = cache_dir
        
        if cache_dir:
            os.makedirs(cache_dir, exist_ok=True)
            self._load_persistent_cache()
    
    def get(self, cache_key: str, fetcher_func: Callable, *args, **kwargs):
        """
        Get a cached value or fetch and cache it if not available.
        
        Args:
            cache_key: Key to identify the cache entry
            fetcher_func: Function to call if cache miss
            *args, **kwargs: Arguments to pass to fetcher_func
        
        Returns:
            Cached or freshly fetched value
        """
        # Determine cache category (for TTL)
        category = next((cat for cat in self.ttl.keys() if cat in cache_key), 'default')
        ttl = self.ttl.get(category, 5)  # Default TTL: 5 seconds
        
        # Check if we have a valid cached value
        with self.locks.get(category, threading.Lock()):
            current_time = time.time()
            
            if (cache_key in self.cache and 
                cache_key in self.cache_times and 
                current_time - self.cache_times[cache_key] < ttl):
                return self.cache[cache_key]
            
            # Cache miss or expired, fetch new value
            try:
                value = fetcher_func(*args, **kwargs)
                self.cache[cache_key] = value
                self.cache_times[cache_key] = current_time
                
                # Save to persistent cache if configured
                if self.cache_dir and category in ['hw_info', 'device_info']:
                    self._save_to_persistent_cache(cache_key, value)
                    
                return value
            except Exception as e:
                # On error, return cached value if available (even if expired)
                if cache_key in self.cache:
                    print(f"Error fetching {cache_key}, using expired cache: {e}")
                    return self.cache[cache_key]
                raise
    
    def invalidate(self, cache_key: str = None):
        """
        Invalidate a cache entry or the entire cache.
        
        Args:
            cache_key: Key to invalidate, or None to invalidate all
        """
        if cache_key is None:
            self.cache.clear()
            self.cache_times.clear()
        elif cache_key in self.cache:
            del self.cache[cache_key]
            if cache_key in self.cache_times:
                del self.cache_times[cache_key]
    
    def set_ttl(self, category: str, seconds: int):
        """
        Set time-to-live for a cache category.
        
        Args:
            category: Cache category name
            seconds: TTL in seconds
        """
        self.ttl[category] = seconds
    
    def _save_to_persistent_cache(self, cache_key: str, value):
        """Save a value to persistent cache."""
        if not self.cache_dir:
            return
            
        try:
            cache_file = os.path.join(self.cache_dir, f"{cache_key}.cache")
            with open(cache_file, 'wb') as f:
                pickle.dump({
                    'value': value,
                    'timestamp': time.time()
                }, f)
        except Exception as e:
            print(f"Error saving to persistent cache: {e}")
    
    def _load_persistent_cache(self):
        """Load values from persistent cache."""
        if not self.cache_dir or not os.path.exists(self.cache_dir):
            return
            
        try:
            for cache_file in glob.glob(os.path.join(self.cache_dir, "*.cache")):
                cache_key = os.path.basename(cache_file).replace(".cache", "")
                
                try:
                    with open(cache_file, 'rb') as f:
                        data = pickle.load(f)
                        
                    # Check if the persistent cache is still valid
                    category = next((cat for cat in self.ttl.keys() if cat in cache_key), 'default')
                    ttl = self.ttl.get(category, 5)
                    
                    if time.time() - data['timestamp'] < ttl:
                        self.cache[cache_key] = data['value']
                        self.cache_times[cache_key] = data['timestamp']
                except Exception as e:
                    print(f"Error loading cache file {cache_file}: {e}")
        except Exception as e:
            print(f"Error loading persistent cache: {e}")
    
    def get_cpu_info(self):
        """Get cached CPU information."""
        return self.get('cpu_info', self._fetch_cpu_info)
    
    def get_memory_info(self):
        """Get cached memory information."""
        return self.get('memory_info', psutil.virtual_memory)
    
    def get_disk_info(self, path='/'):
        """Get cached disk information."""
        return self.get(f'disk_info_{path}', psutil.disk_usage, path)
    
    def get_temperature(self):
        """Get cached temperature information."""
        return self.get('temperature', self._fetch_temperature)
    
    def get_fan_speeds(self):
        """Get cached fan speeds."""
        return self.get('fan_speeds', self._fetch_fan_speeds)
    
    def get_battery_info(self):
        """Get cached battery information."""
        return self.get('battery_info', self._fetch_battery_info)
    
    def get_system_load(self):
        """Get cached system load."""
        return self.get('system_load', os.getloadavg)
    
    def get_process_list(self):
        """Get cached process list."""
        return self.get('process_list', self._fetch_process_list)
    
    def get_network_info(self):
        """Get cached network information."""
        return self.get('network_info', psutil.net_io_counters)
    
    def _fetch_cpu_info(self):
        """Fetch detailed CPU information."""
        info = {
            'percent': psutil.cpu_percent(interval=0.1),
            'count': psutil.cpu_count(),
            'physical_count': psutil.cpu_count(logical=False),
            'freq': psutil.cpu_freq() if hasattr(psutil, 'cpu_freq') else None,
            'stats': psutil.cpu_stats() if hasattr(psutil, 'cpu_stats') else None,
            'load_avg': os.getloadavg() if hasattr(os, 'getloadavg') else (0, 0, 0)
        }
        
        # Try to get CPU model name
        try:
            if os.path.exists('/proc/cpuinfo'):
                with open('/proc/cpuinfo', 'r') as f:
                    for line in f:
                        if 'model name' in line:
                            info['model'] = line.split(':', 1)[1].strip()
                            break
        except:
            info['model'] = "Unknown CPU"
            
        return info
    
    def _fetch_temperature(self):
        """Fetch temperature information."""
        temps = {
            'cpu': [],
            'gpu': [],
            'disk': [],
            'other': []
        }
        
        # Method 1: psutil sensors
        if hasattr(psutil, 'sensors_temperatures'):
            try:
                sensors = psutil.sensors_temperatures()
                
                for name, entries in sensors.items():
                    for entry in entries:
                        if 'core' in name.lower() or 'cpu' in name.lower():
                            temps['cpu'].append({
                                'label': entry.label or f"{name} {entry.current}",
                                'temp': entry.current,
                                'high': entry.high,
                                'critical': entry.critical
                            })
                        elif 'gpu' in name.lower() or 'nvidia' in name.lower():
                            temps['gpu'].append({
                                'label': entry.label or f"{name} {entry.current}",
                                'temp': entry.current,
                                'high': entry.high,
                                'critical': entry.critical
                            })
                        elif 'nvme' in name.lower() or 'ssd' in name.lower():
                            temps['disk'].append({
                                'label': entry.label or f"{name} {entry.current}",
                                'temp': entry.current,
                                'high': entry.high,
                                'critical': entry.critical
                            })
                        else:
                            temps['other'].append({
                                'label': entry.label or f"{name} {entry.current}",
                                'temp': entry.current,
                                'high': entry.high,
                                'critical': entry.critical
                            })
            except:
                pass
        
        # Method 2: sensors command
        try:
            result = subprocess.run(['sensors', '-j'], capture_output=True, text=True, timeout=2)
            if result.returncode == 0:
                try:
                    data = json.loads(result.stdout)
                    self._parse_sensors_json(data, temps)
                except:
                    pass
        except:
            pass
            
        return temps
    
    def _parse_sensors_json(self, data, temps):
        """Parse JSON output from sensors command."""
        for chip, values in data.items():
            for key, subvalues in values.items():
                if isinstance(subvalues, dict):
                    for subkey, temp in subvalues.items():
                        if isinstance(temp, (int, float)) and '_input' in subkey:
                            label = subkey.replace('_input', '')
                            high = subvalues.get(f'{label}_max', None)
                            critical = subvalues.get(f'{label}_crit', None)
                            
                            temp_entry = {
                                'label': f"{chip} {label}",
                                'temp': temp,
                                'high': high,
                                'critical': critical
                            }
                            
                            if 'core' in chip.lower() or 'cpu' in chip.lower():
                                temps['cpu'].append(temp_entry)
                            elif 'gpu' in chip.lower() or 'nvidia' in chip.lower():
                                temps['gpu'].append(temp_entry)
                            elif 'nvme' in chip.lower() or 'ssd' in chip.lower():
                                temps['disk'].append(temp_entry)
                            else:
                                temps['other'].append(temp_entry)
    
    def _fetch_fan_speeds(self):
        """Fetch fan speed information."""
        fans = []
        
        # Method 1: psutil sensors
        if hasattr(psutil, 'sensors_fans'):
            try:
                sensors = psutil.sensors_fans()
                
                for name, entries in sensors.items():
                    for i, entry in enumerate(entries):
                        fans.append({
                            'name': entry.label or f"{name} Fan {i}",
                            'speed': entry.current,
                            'source': 'psutil'
                        })
            except:
                pass
        
        # Method 2: hwmon
        try:
            hwmon_dirs = glob.glob('/sys/class/hwmon/hwmon*')
            for hwmon_dir in hwmon_dirs:
                try:
                    name_file = os.path.join(hwmon_dir, 'name')
                    device_name = 'Unknown'
                    if os.path.exists(name_file):
                        with open(name_file, 'r') as f:
                            device_name = f.read().strip()
                    
                    # Find fan input files
                    fan_files = glob.glob(os.path.join(hwmon_dir, 'fan*_input'))
                    for fan_file in fan_files:
                        with open(fan_file, 'r') as f:
                            rpm = int(f.read().strip())
                            
                        fan_num = os.path.basename(fan_file).replace('fan', '').replace('_input', '')
                        
                        # Try to get fan label
                        label_file = fan_file.replace('_input', '_label')
                        fan_label = f'Fan {fan_num}'
                        if os.path.exists(label_file):
                            with open(label_file, 'r') as lf:
                                fan_label = lf.read().strip()
                        
                        fans.append({
                            'name': f'{device_name} {fan_label}',
                            'speed': rpm,
                            'source': 'hwmon'
                        })
                except:
                    pass
        except:
            pass
            
        return fans
    
    def _fetch_battery_info(self):
        """Fetch battery information."""
        battery = None
        
        # Method 1: psutil
        if hasattr(psutil, 'sensors_battery'):
            try:
                battery = psutil.sensors_battery()
                if battery:
                    return {
                        'percent': battery.percent,
                        'power_plugged': battery.power_plugged,
                        'secsleft': battery.secsleft,
                        'source': 'psutil'
                    }
            except:
                pass
        
        # Method 2: /sys/class/power_supply
        try:
            battery_dirs = glob.glob('/sys/class/power_supply/BAT*')
            if battery_dirs:
                battery_dir = battery_dirs[0]
                
                # Get capacity
                capacity_file = os.path.join(battery_dir, 'capacity')
                if os.path.exists(capacity_file):
                    with open(capacity_file, 'r') as f:
                        capacity = int(f.read().strip())
                
                # Get status
                status_file = os.path.join(battery_dir, 'status')
                if os.path.exists(status_file):
                    with open(status_file, 'r') as f:
                        status = f.read().strip()
                
                return {
                    'percent': capacity,
                    'power_plugged': status == 'Charging' or status == 'Full',
                    'status': status,
                    'source': 'sysfs'
                }
        except:
            pass
        
        # No battery found
        return {
            'percent': 0,
            'power_plugged': True,
            'source': 'none'
        }
    
    def _fetch_process_list(self):
        """Fetch process list."""
        processes = []
        
        try:
            for proc in psutil.process_iter(['pid', 'name', 'username', 'cpu_percent', 'memory_percent']):
                try:
                    pinfo = proc.info
                    pinfo['cpu_percent'] = proc.cpu_percent(interval=0.1)
                    processes.append(pinfo)
                except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                    pass
        except:
            pass
            
        return processes


class DeviceManager:
    """
    Provides multi-device support for RGB control.
    
    This class discovers and manages multiple RGB devices, allowing
    for synchronized lighting effects across multiple peripherals.
    """
    
    def __init__(self, config_dir: str = None):
        """
        Initialize the device manager.
        
        Args:
            config_dir: Directory to store device configuration
        """
        self.devices = {}
        self.device_status = {}
        self.config_dir = config_dir
        self.config_file = os.path.join(config_dir, 'devices.json') if config_dir else None
        self.lock = threading.Lock()
        self.last_discovery = 0
        self.discovery_interval = 30  # seconds
        
        # RGB device types and their detection patterns
        self.device_types = {
            'keyboard': {
                'patterns': ['keyboard', 'kbd'],
                'aliases': ['keyboard', 'kbd'],
                'icon': '⌨️'
            },
            'mouse': {
                'patterns': ['mouse', 'hid'],
                'aliases': ['mouse'],
                'icon': '🖱️'
            },
            'headset': {
                'patterns': ['headset', 'audio'],
                'aliases': ['headset', 'headphone'],
                'icon': '🎧'
            },
            'mousepad': {
                'patterns': ['mousepad', 'mouse_pad'],
                'aliases': ['mousepad'],
                'icon': '📟'
            },
            'fan': {
                'patterns': ['fan', 'cooler'],
                'aliases': ['fan', 'cooling'],
                'icon': '🌀'
            },
            'memory': {
                'patterns': ['memory', 'ram', 'dimm'],
                'aliases': ['memory', 'ram'],
                'icon': '📊'
            },
            'gpu': {
                'patterns': ['gpu', 'graphics', 'vga'],
                'aliases': ['graphics', 'gpu', 'vga'],
                'icon': '🎮'
            },
            'led_strip': {
                'patterns': ['led', 'strip', 'light'],
                'aliases': ['led_strip', 'strip'],
                'icon': '💡'
            },
            'unknown': {
                'patterns': [],
                'aliases': ['unknown', 'other'],
                'icon': '❓'
            }
        }
        
        self.commands = {
            'keyboard': {
                'set_key': [0xCC, 0x01],  # CC 01 [key_index] [r] [g] [b] 00...
                'set_mode': [0xCC, 0x02]   # CC 02 [mode] [speed] [brightness] 00...
            },
            'mouse': {
                'set_color': [0xCC, 0x01], # CC 01 [zone] [r] [g] [b] 00...
                'set_mode': [0xCC, 0x02]   # CC 02 [mode] [speed] [brightness] 00...
            },
            'default': {
                'set_color': [0xCC, 0x01], # Generic fallback
                'set_mode': [0xCC, 0x02]
            }
        }
        
        # Load saved devices
        if self.config_file and os.path.exists(self.config_file):
            self._load_devices()
    
    def discover_devices(self, force: bool = False):
        """
        Discover RGB devices connected to the system.
        
        Args:
            force: Force discovery even if within discovery interval
            
        Returns:
            Dictionary of discovered devices
        """
        current_time = time.time()
        
        # Skip discovery if done recently, unless forced
        if not force and current_time - self.last_discovery < self.discovery_interval:
            return self.devices
        
        with self.lock:
            self.last_discovery = current_time
            
            # Method 1: Standard hidraw devices
            self._discover_hidraw_devices()
            
            # Method 2: Check for OpenRGB support
            self._discover_openrgb_devices()
            
            # Method 3: Check for RGB Fusion and other vendor-specific SDKs
            self._discover_vendor_devices()
            
            # Method 4: Check for RGB device drivers
            self._discover_driver_devices()
            
            # Update device status
            self._update_device_status()
            
            # Save discovered devices
            if self.config_dir:
                self._save_devices()
            
        return self.devices
    
    def get_device(self, device_id: str):
        """
        Get a device by ID.
        
        Args:
            device_id: Device ID
            
        Returns:
            Device dictionary or None if not found
        """
        return self.devices.get(device_id)
    
    def get_device_path(self, device_id: str):
        """
        Get device path by ID.
        
        Args:
            device_id: Device ID
            
        Returns:
            Device path or None if not found
        """
        device = self.devices.get(device_id)
        return device.get('path') if device else None
    
    def get_devices_by_type(self, device_type: str):
        """
        Get devices by type.
        
        Args:
            device_type: Device type (e.g., 'keyboard', 'mouse')
            
        Returns:
            List of devices of the specified type
        """
        return {k: v for k, v in self.devices.items() if v.get('type') == device_type}
    
    def get_primary_keyboard(self):
        """
        Get the primary keyboard device.
        
        Returns:
            Primary keyboard device or None if not found
        """
        keyboards = self.get_devices_by_type('keyboard')
        
        if not keyboards:
            return None
            
        # Return the primary keyboard if set
        for device_id, device in keyboards.items():
            if device.get('is_primary', False):
                return device
        
        # Otherwise return the first keyboard
        return next(iter(keyboards.values()))
    
    def set_primary_device(self, device_id: str):
        """
        Set a device as primary for its type.
        
        Args:
            device_id: Device ID
            
        Returns:
            True if successful, False otherwise
        """
        with self.lock:
            if device_id not in self.devices:
                return False
                
            device_type = self.devices[device_id].get('type')
            
            # Clear primary flag from other devices of the same type
            for other_id, other_device in self.devices.items():
                if other_device.get('type') == device_type:
                    other_device['is_primary'] = False
            
            # Set this device as primary
            self.devices[device_id]['is_primary'] = True
            
            # Save changes
            if self.config_dir:
                self._save_devices()
                
            return True
    
    def set_device_type(self, device_id: str, device_type: str):
        """
        Set the type of a device.
        
        Args:
            device_id: Device ID
            device_type: Device type
            
        Returns:
            True if successful, False otherwise
        """
        with self.lock:
            if device_id not in self.devices:
                return False
                
            if device_type not in self.device_types:
                return False
                
            self.devices[device_id]['type'] = device_type
            
            # Save changes
            if self.config_dir:
                self._save_devices()
                
            return True
    
    def set_device_name(self, device_id: str, name: str):
        """
        Set the name of a device.
        
        Args:
            device_id: Device ID
            name: Device name
            
        Returns:
            True if successful, False otherwise
        """
        with self.lock:
            if device_id not in self.devices:
                return False
                
            self.devices[device_id]['name'] = name
            
            # Save changes
            if self.config_dir:
                self._save_devices()
                
            return True
    
    def send_command(self, device_id: str, command_type: str, *args):
        """
        Send a command to a device.
        
        Args:
            device_id: Device ID
            command_type: Command type (e.g., 'set_key', 'set_mode')
            *args: Command arguments
            
        Returns:
            True if successful, False otherwise
        """
        device = self.devices.get(device_id)
        if not device:
            return False
            
        device_type = device.get('type', 'unknown')
        device_path = device.get('path')
        
        if not device_path or not os.path.exists(device_path):
            return False
            
        # Get command bytes for this device type
        commands = self.commands.get(device_type, self.commands['default'])
        command_bytes = commands.get(command_type)
        
        if not command_bytes:
            return False
            
        # Construct full command with arguments
        full_command = command_bytes.copy()
        full_command.extend(args)
        
        # Pad to 16 bytes
        while len(full_command) < 16:
            full_command.append(0x00)
            
        try:
            with open(device_path, 'wb') as device_file:
                device_file.write(bytes(full_command))
            return True
        except Exception as e:
            print(f"Error sending command to device {device_id}: {e}")
            return False
    
    def _discover_hidraw_devices(self):
        """Discover hidraw devices."""
        for i in range(10):  # Check hidraw0 through hidraw9
            path = f"/dev/hidraw{i}"
            if os.path.exists(path) and os.access(path, os.R_OK | os.W_OK):
                device_id = f"hidraw{i}"
                
                # Check if we already know about this device
                if device_id in self.devices:
                    # Update path in case it changed
                    self.devices[device_id]['path'] = path
                    continue
                
                # Try to get more info about the device
                device_info = self._get_hidraw_info(path)
                
                # Determine device type
                device_type = self._determine_device_type(device_info)
                
                # Add to devices
                self.devices[device_id] = {
                    'id': device_id,
                    'path': path,
                    'type': device_type,
                    'name': device_info.get('name', f"RGB Device {i}"),
                    'info': device_info,
                    'commands': self.commands.get(device_type, self.commands['default']),
                    'is_primary': False,
                    'icon': self.device_types[device_type]['icon']
                }
    
    def _discover_openrgb_devices(self):
        """Discover devices via OpenRGB SDK."""
        try:
            # Check if OpenRGB is installed
            if subprocess.run(['which', 'openrgb'], capture_output=True).returncode != 0:
                return
                
            # Try to get device list from OpenRGB
            result = subprocess.run(['openrgb', '--list-devices'], capture_output=True, text=True, timeout=5)
            if result.returncode != 0:
                return
                
            # Parse output
            for line in result.stdout.splitlines():
                if ': ' in line:
                    index, device_info = line.split(': ', 1)
                    try:
                        index = int(index)
                        device_id = f"openrgb{index}"
                        
                        # Parse device type
                        device_type = 'unknown'
                        for dtype, info in self.device_types.items():
                            if any(pattern in device_info.lower() for pattern in info['patterns']):
                                device_type = dtype
                                break
                        
                        # Add to devices
                        self.devices[device_id] = {
                            'id': device_id,
                            'path': f"openrgb://{index}",
                            'type': device_type,
                            'name': device_info,
                            'info': {'source': 'openrgb'},
                            'commands': {'via': 'openrgb'},
                            'is_primary': False,
                            'icon': self.device_types[device_type]['icon']
                        }
                    except ValueError:
                        pass
        except:
            pass
    
    def _discover_vendor_devices(self):
        """Discover devices via vendor-specific SDKs."""
        # This would be expanded with vendor-specific detection logic
        pass
    
    def _discover_driver_devices(self):
        """Discover devices via RGB drivers."""
        # This would be expanded with driver-specific detection logic
        pass
    
    def _get_hidraw_info(self, path: str):
        """Get info about a hidraw device."""
        info = {
            'path': path,
            'name': f"RGB Device {path.replace('/dev/hidraw', '')}"
        }
        
        try:
            # Try to get vendor/product info from sysfs
            hidraw_num = path.replace('/dev/hidraw', '')
            sysfs_path = f"/sys/class/hidraw/hidraw{hidraw_num}/device"
            
            if os.path.exists(sysfs_path):
                # Try to get device name
                uevent_path = os.path.join(sysfs_path, 'uevent')
                if os.path.exists(uevent_path):
                    with open(uevent_path, 'r') as f:
                        for line in f:
                            if '=' in line:
                                key, value = line.strip().split('=', 1)
                                info[key.lower()] = value
                
                # Try to get vendor/product ID
                for parent_level in range(1, 4):  # Check up to 3 parent directories
                    parent_path = sysfs_path
                    for _ in range(parent_level):
                        parent_path = os.path.dirname(parent_path)
                    
                    id_vendor_path = os.path.join(parent_path, 'idVendor')
                    id_product_path = os.path.join(parent_path, 'idProduct')
                    
                    if os.path.exists(id_vendor_path) and os.path.exists(id_product_path):
                        try:
                            with open(id_vendor_path, 'r') as f:
                                info['vendor_id'] = f.read().strip()
                            with open(id_product_path, 'r') as f:
                                info['product_id'] = f.read().strip()
                            
                            # Try to get manufacturer and product strings
                            manufacturer_path = os.path.join(parent_path, 'manufacturer')
                            product_path = os.path.join(parent_path, 'product')
                            
                            if os.path.exists(manufacturer_path):
                                with open(manufacturer_path, 'r') as f:
                                    info['manufacturer'] = f.read().strip()
                            
                            if os.path.exists(product_path):
                                with open(product_path, 'r') as f:
                                    info['product'] = f.read().strip()
                                    info['name'] = info['product']
                            
                            break
                        except:
                            pass
        except:
            pass
            
        return info
    
    def _determine_device_type(self, device_info: dict):
        """Determine device type based on information."""
        # First, check if we have manufacturer/product info
        if 'manufacturer' in device_info and 'product' in device_info:
            manufacturer = device_info['manufacturer'].lower()
            product = device_info['product'].lower()
            
            combined = f"{manufacturer} {product}"
            
            for dtype, info in self.device_types.items():
                if any(pattern in combined for pattern in info['patterns']):
                    return dtype
        
        # Default to keyboard for hidraw0
        if device_info['path'] == '/dev/hidraw0':
            return 'keyboard'
            
        return 'unknown'
    
    def _update_device_status(self):
        """Update device status."""
        for device_id, device in self.devices.items():
            path = device.get('path')
            
            if path.startswith('/dev/'):
                if os.path.exists(path) and os.access(path, os.R_OK | os.W_OK):
                    self.device_status[device_id] = {
                        'connected': True,
                        'accessible': True,
                        'last_check': time.time()
                    }
                else:
                    self.device_status[device_id] = {
                        'connected': os.path.exists(path),
                        'accessible': False,
                        'last_check': time.time()
                    }
            elif path.startswith('openrgb://'):
                # For OpenRGB devices, we'd need to check differently
                self.device_status[device_id] = {
                    'connected': True,  # Assume connected
                    'accessible': True,  # Assume accessible
                    'last_check': time.time()
                }
    
    def _save_devices(self):
        """Save devices to config file."""
        if not self.config_dir or not self.config_file:
            return
            
        try:
            os.makedirs(self.config_dir, exist_ok=True)
            
            # Create a copy of the devices dict with only essential info
            devices_to_save = {}
            for device_id, device in self.devices.items():
                devices_to_save[device_id] = {
                    'id': device['id'],
                    'type': device['type'],
                    'name': device['name'],
                    'is_primary': device.get('is_primary', False)
                }
            
            with open(self.config_file, 'w') as f:
                json.dump(devices_to_save, f, indent=2)
        except Exception as e:
            print(f"Error saving devices: {e}")
    
    def _load_devices(self):
        """Load devices from config file."""
        if not self.config_file or not os.path.exists(self.config_file):
            return
            
        try:
            with open(self.config_file, 'r') as f:
                saved_devices = json.load(f)
            
            # Apply saved settings to discovered devices
            for device_id, saved_device in saved_devices.items():
                if device_id in self.devices:
                    self.devices[device_id]['type'] = saved_device.get('type', self.devices[device_id]['type'])
                    self.devices[device_id]['name'] = saved_device.get('name', self.devices[device_id]['name'])
                    self.devices[device_id]['is_primary'] = saved_device.get('is_primary', False)
        except Exception as e:
            print(f"Error loading devices: {e}")


class MacroRecorder:
    """
    Records and plays back RGB effect macros.
    
    This class allows recording sequences of RGB commands that can be
    saved, loaded, and played back to recreate complex lighting effects.
    """
    
    def __init__(self, config_dir: str = None):
        """
        Initialize the macro recorder.
        
        Args:
            config_dir: Directory to store macro files
        """
        self.config_dir = config_dir
        self.macros_dir = os.path.join(config_dir, 'macros') if config_dir else None
        self.recording = False
        self.current_macro = []
        self.current_macro_name = None
        self.playback_thread = None
        self.stop_playback = threading.Event()
        self.recording_start_time = 0
        self.playback_speed = 1.0
        self.loop_playback = False
        
        # Create macros directory if it doesn't exist
        if self.macros_dir:
            os.makedirs(self.macros_dir, exist_ok=True)
    
    def start_recording(self, macro_name: str):
        """
        Start recording a macro.
        
        Args:
            macro_name: Name of the macro
            
        Returns:
            True if recording started, False otherwise
        """
        if self.recording:
            return False
            
        self.recording = True
        self.current_macro = []
        self.current_macro_name = macro_name
        self.recording_start_time = time.time()
        
        return True
    
    def stop_recording(self):
        """
        Stop recording a macro.
        
        Returns:
            True if recording stopped, False if not recording
        """
        if not self.recording:
            return False
            
        self.recording = False
        
        # Save the macro
        if self.current_macro and self.current_macro_name and self.macros_dir:
            self.save_macro(self.current_macro_name)
            
        return True
    
    def record_command(self, command_type: str, key_name: str, red: int, green: int, blue: int):
        """
        Record an RGB command.
        
        Args:
            command_type: Type of command (e.g., 'key', 'group')
            key_name: Name of the key or group
            red, green, blue: RGB color values
            
        Returns:
            True if command recorded, False if not recording
        """
        if not self.recording:
            return False
            
        # Calculate time since recording started
        timestamp = time.time() - self.recording_start_time
        
        command = {
            'type': command_type,
            'key': key_name,
            'color': [red, green, blue],
            'timestamp': timestamp
        }
        
        self.current_macro.append(command)
        return True
    
    def save_macro(self, name: str = None):
        """
        Save the current macro.
        
        Args:
            name: Name to save the macro as (default: current name)
            
        Returns:
            True if saved, False otherwise
        """
        if not self.current_macro:
            return False
            
        if not self.macros_dir:
            return False
            
        # Use provided name or current name
        macro_name = name or self.current_macro_name
        if not macro_name:
            return False
            
        # Sanitize filename
        filename = ''.join(c if c.isalnum() else '_' for c in macro_name)
        filepath = os.path.join(self.macros_dir, f"{filename}.json")
        
        # Create macro object
        macro = {
            'name': macro_name,
            'created': time.time(),
            'duration': self.current_macro[-1]['timestamp'] if self.current_macro else 0,
            'command_count': len(self.current_macro),
            'commands': self.current_macro
        }
        
        try:
            with open(filepath, 'w') as f:
                json.dump(macro, f, indent=2)
            return True
        except Exception as e:
            print(f"Error saving macro: {e}")
            return False
    
    def load_macro(self, name: str):
        """
        Load a macro.
        
        Args:
            name: Name of the macro to load
            
        Returns:
            Loaded macro or None if not found
        """
        if not self.macros_dir:
            return None
            
        # Try exact filename match first
        filepath = os.path.join(self.macros_dir, f"{name}.json")
        
        # If not found, try case-insensitive search
        if not os.path.exists(filepath):
            # Sanitize filename for search
            sanitized = ''.join(c if c.isalnum() else '_' for c in name)
            
            # Search for matching files
            for filename in os.listdir(self.macros_dir):
                if filename.lower() == f"{sanitized.lower()}.json":
                    filepath = os.path.join(self.macros_dir, filename)
                    break
        
        if not os.path.exists(filepath):
            return None
            
        try:
            with open(filepath, 'r') as f:
                macro = json.load(f)
            
            self.current_macro = macro['commands']
            self.current_macro_name = macro['name']
            
            return macro
        except Exception as e:
            print(f"Error loading macro: {e}")
            return None
    
    def list_macros(self):
        """
        List all available macros.
        
        Returns:
            List of macro names and metadata
        """
        if not self.macros_dir or not os.path.exists(self.macros_dir):
            return []
            
        macros = []
        
        for filename in os.listdir(self.macros_dir):
            if filename.endswith('.json'):
                filepath = os.path.join(self.macros_dir, filename)
                
                try:
                    with open(filepath, 'r') as f:
                        macro = json.load(f)
                    
                    macros.append({
                        'name': macro.get('name', filename.replace('.json', '')),
                        'duration': macro.get('duration', 0),
                        'command_count': macro.get('command_count', 0),
                        'created': macro.get('created', 0),
                        'filename': filename
                    })
                except Exception as e:
                    print(f"Error reading macro {filename}: {e}")
        
        return macros
    
    def play_macro(self, rgb_controller, name: str = None, loop: bool = False, speed: float = 1.0):
        """
        Play a macro.
        
        Args:
            rgb_controller: RGB controller to send commands to
            name: Name of the macro to play (default: current macro)
            loop: Whether to loop playback
            speed: Playback speed multiplier
            
        Returns:
            True if playback started, False otherwise
        """
        # Stop any existing playback
        self.stop_playback_thread()
        
        # Load macro if name provided
        if name:
            if not self.load_macro(name):
                return False
        
        # Check if we have a macro to play
        if not self.current_macro:
            return False
        
        # Set playback options
        self.loop_playback = loop
        self.playback_speed = speed
        self.stop_playback.clear()
        
        # Start playback thread
        self.playback_thread = threading.Thread(
            target=self._playback_thread,
            args=(rgb_controller,),
            daemon=True
        )
        self.playback_thread.start()
        
        return True
    
    def stop_playback_thread(self):
        """
        Stop macro playback.
        
        Returns:
            True if playback was stopped, False if not playing
        """
        if not self.playback_thread or not self.playback_thread.is_alive():
            return False
            
        self.stop_playback.set()
        self.playback_thread.join(timeout=1.0)
        self.playback_thread = None
        
        return True
    
    def _playback_thread(self, rgb_controller):
        """Macro playback thread."""
        if not self.current_macro:
            return
            
        # Get keyboard map for key name resolution
        keyboard_map = getattr(rgb_controller, 'keyboard_map', {})
        
        while not self.stop_playback.is_set():
            last_time = time.time()
            
            for command in self.current_macro:
                # Check for stop request
                if self.stop_playback.is_set():
                    break
                    
                # Calculate wait time based on timestamps and speed
                wait_time = command['timestamp'] / self.playback_speed
                current_time = time.time()
                elapsed = current_time - last_time
                
                if elapsed < wait_time:
                    # Sleep for the remaining time
                    sleep_time = wait_time - elapsed
                    
                    # Use short sleeps to allow for responsive stopping
                    sleep_end = time.time() + sleep_time
                    while time.time() < sleep_end:
                        if self.stop_playback.is_set():
                            break
                        time.sleep(min(0.01, sleep_end - time.time()))
                
                # Execute the command
                command_type = command['type']
                key_name = command['key']
                color = command['color']
                
                try:
                    if command_type == 'key':
                        rgb_controller.set_key_color(key_name, *color)
                    elif command_type == 'group':
                        if hasattr(rgb_controller, 'key_groups') and key_name in rgb_controller.key_groups:
                            rgb_controller.set_group_color(key_name, *color)
                except Exception as e:
                    print(f"Error executing macro command: {e}")
                
                last_time = current_time
            
            # Exit if not looping
            if not self.loop_playback:
                break
                
            # Small pause between loops
            time.sleep(0.5)
