#!/usr/bin/env python3
"""
OriginPC Enhanced Professional Control Center v5.1 - Ultimate Edition
=====================================================================
Fully integrated RGB control center with all system management features
- Complete lid monitoring with automatic keyboard clearing
- Integrated fan control (fancontrol-gui replacement)
- Real-time temperature monitoring with sensors
- Power management and TLP integration
- Enhanced keypad clearing (fixes kp_plus cyan issue)
- Nyx-inspired professional system monitoring
- Advanced threading model for smooth performance
- Professional dark theme with high DPI scaling
- Persistent effects that survive reboots
- System tray integration
- Self-contained - no external dependencies needed
"""

import sys
import os
import time
import random
import math
import subprocess
import json
import pickle
import threading
import signal
import psutil
# import numpy as np  # Not needed, removing to avoid dependency
import glob
import shutil
from pathlib import Path
from datetime import datetime
from PyQt5.QtWidgets import *
from PyQt5.QtCore import *
from PyQt5.QtGui import *
from PyQt5.QtCore import QSharedMemory

# Import optimization modules for advanced performance
try:
    from optimization_classes import RGBCommandBatcher, SystemInfoCache, DeviceManager, MacroRecorder
    from hardware_optimizations import ThreadPool, HardwareOptimizer, RGBDeviceOptimizer, SystemMonitor, PowerOptimizer
    from core_system import SystemManager, ConfigManager
    OPTIMIZATIONS_AVAILABLE = True
except ImportError as e:
    print(f"Warning: Optimization modules not available: {e}")
    OPTIMIZATIONS_AVAILABLE = False

# Import AI enhancement modules for intelligent features
try:
    from ai_effects_engine import ColorTheoryEngine, AdaptiveLearningEngine
    from system_intelligence import AdvancedSystemMonitor, IntelligentResourceManager, PredictiveAnalytics
    AI_ENHANCEMENTS_AVAILABLE = True
    print("✨ AI Enhancement modules loaded successfully")
except ImportError as e:
    print(f"Warning: AI Enhancement modules not available: {e}")
    AI_ENHANCEMENTS_AVAILABLE = False

# High DPI Support (Nyx-inspired)
if hasattr(Qt, 'AA_EnableHighDpiScaling'):
    QApplication.setAttribute(Qt.ApplicationAttribute.AA_EnableHighDpiScaling, True)
if hasattr(Qt, 'AA_UseHighDpiPixmaps'):
    QApplication.setAttribute(Qt.ApplicationAttribute.AA_UseHighDpiPixmaps, True)
os.environ["QT_AUTO_SCREEN_SCALE_FACTOR"] = "1"

# Configuration directory
CONFIG_DIR = Path.home() / '.config' / 'enhanced-originpc-control'
CONFIG_DIR.mkdir(parents=True, exist_ok=True)
EFFECT_STATE_FILE = CONFIG_DIR / 'effect_state.json'
SETTINGS_FILE = CONFIG_DIR / 'settings.json'
AI_USAGE_FILE = CONFIG_DIR / 'ai_usage.json'
PROFILES_FILE = CONFIG_DIR / 'profiles.json'

class SystemDataUpdater(QThread):
    """Professional threading model for real-time system data updates (Nyx-inspired)"""
    data_updated = pyqtSignal(dict)
    
    def __init__(self, update_interval=1000):
        super().__init__()
        self.update_interval = update_interval
        self.running = False
        self.sensors_data = {}
        
    def run(self):
        self.running = True
        while self.running:
            try:
                # Gather comprehensive system data
                data = {
                    'cpu_percent': psutil.cpu_percent(interval=0.1),
                    'cpu_temp': self.get_cpu_temperature(),
                    'memory': psutil.virtual_memory(),
                    'disk': psutil.disk_usage('/'),
                    'network': psutil.net_io_counters(),
                    'boot_time': psutil.boot_time(),
                    'load_avg': os.getloadavg() if hasattr(os, 'getloadavg') else (0, 0, 0),
                    'timestamp': time.time()
                }
                
                # Add GPU data if available
                try:
                    import GPUtil
                    gpus = GPUtil.getGPUs()
                    if gpus:
                        gpu = gpus[0]
                        data['gpu_load'] = gpu.load * 100
                        data['gpu_temp'] = gpu.temperature
                        data['gpu_memory'] = gpu.memoryUtil * 100
                except ImportError:
                    data['gpu_load'] = 0
                    data['gpu_temp'] = 0
                    data['gpu_memory'] = 0
                
                self.data_updated.emit(data)
                self.msleep(self.update_interval)
                
            except Exception as e:
                print(f"SystemDataUpdater error: {e}")
                self.msleep(5000)  # Wait 5 seconds on error
                
    def get_comprehensive_sensors_data(self):
        """Get comprehensive sensor data including temperatures, fans, voltages"""
        sensors_data = {
            'cpu_temps': [],
            'gpu_temps': [],
            'nvme_temps': [],
            'memory_temps': [],
            'fan_speeds': [],
            'voltages': [],
            'power': []
        }
        
        try:
            # Get all sensor data using psutil
            if hasattr(psutil, 'sensors_temperatures'):
                temps = psutil.sensors_temperatures()
                
                # CPU temperatures
                if 'coretemp' in temps:
                    sensors_data['cpu_temps'] = [(t.label or f'Core {i}', t.current) 
                                                for i, t in enumerate(temps['coretemp'])]
                
                # NVME/Storage temperatures
                if 'nvme' in temps:
                    sensors_data['nvme_temps'] = [(t.label or f'NVME {i}', t.current) 
                                                 for i, t in enumerate(temps['nvme'])]
                
                # Memory temperatures
                if 'spd5118' in temps:
                    sensors_data['memory_temps'] = [(t.label or f'Memory {i}', t.current) 
                                                   for i, t in enumerate(temps['spd5118'])]
            
            # Fan speeds
            if hasattr(psutil, 'sensors_fans'):
                fans = psutil.sensors_fans()
                for sensor_name, fan_list in fans.items():
                    for i, fan in enumerate(fan_list):
                        sensors_data['fan_speeds'].append((fan.label or f'{sensor_name} Fan {i}', fan.current))
            
            # Try to get additional sensor data via subprocess if available
            try:
                result = subprocess.run(['sensors', '-A'], capture_output=True, text=True, timeout=3)
                if result.returncode == 0:
                    self._parse_sensors_output(result.stdout, sensors_data)
            except (subprocess.TimeoutExpired, FileNotFoundError):
                pass
                
        except Exception as e:
            print(f"Sensors error: {e}")
        
        return sensors_data
    
    def _parse_sensors_output(self, sensors_output, sensors_data):
        """Parse sensors command output for additional data"""
        lines = sensors_output.split('\n')
        current_chip = None
        
        for line in lines:
            line = line.strip()
            if line and not line.startswith('+') and ':' not in line and line.endswith(':') == False:
                current_chip = line
            elif ':' in line and current_chip:
                parts = line.split(':')
                if len(parts) >= 2:
                    name = parts[0].strip()
                    value_part = parts[1].strip()
                    
                    # Parse temperature values
                    if '°C' in value_part and 'temp' in name.lower():
                        try:
                            temp_val = float(value_part.split('°C')[0].split()[-1])
                            if 'cpu' in current_chip.lower() or 'core' in current_chip.lower():
                                sensors_data['cpu_temps'].append((f"{current_chip} {name}", temp_val))
                            elif 'gpu' in current_chip.lower():
                                sensors_data['gpu_temps'].append((f"{current_chip} {name}", temp_val))
                        except ValueError:
                            pass
                    
                    # Parse fan speeds
                    elif 'rpm' in value_part.lower() and 'fan' in name.lower():
                        try:
                            rpm_val = float(value_part.split('RPM')[0].strip())
                            sensors_data['fan_speeds'].append((f"{current_chip} {name}", rpm_val))
                        except ValueError:
                            pass
    
    def get_cpu_temperature(self):
        """Get highest CPU temperature from comprehensive data"""
        sensors = self.get_comprehensive_sensors_data()
        if sensors['cpu_temps']:
            return max([temp for _, temp in sensors['cpu_temps']])
        return 0
    
    def stop(self):
        self.running = False

class QRoundProgressBar(QWidget):
    """Custom round progress bar widget (Nyx-inspired)"""
    
    def __init__(self, parent=None, font_size=20, default_color=None, 
                 progress_color=None, inner_background_color=None, width=0.2):
        super().__init__(parent)
        
        self.font_size = font_size
        self.width = width
        self.value = 0
        self.max_value = 100
        
        # Color scheme
        self.default_color = default_color or QColor(60, 63, 65, 255)
        self.progress_color = progress_color or QColor(0, 150, 255, 255)
        self.inner_bg_color = inner_background_color or QColor(40, 42, 45, 255)
        
        # Set minimum size
        self.setMinimumSize(100, 100)
        
    def set_value(self, value):
        """Set progress value (0-100)"""
        self.value = max(0, min(100, value))
        self.update()
        
    def paintEvent(self, event):
        """Custom paint event for round progress bar"""
        painter = QPainter(self)
        painter.setRenderHint(QPainter.Antialiasing)
        
        # Calculate dimensions
        rect = self.rect()
        size = min(rect.width(), rect.height())
        x = (rect.width() - size) // 2
        y = (rect.height() - size) // 2
        
        # Create square rect for circle
        square_rect = QRect(x, y, size, size)
        inner_rect = QRect(x + size//4, y + size//4, size//2, size//2)
        
        # Draw background circle
        painter.setBrush(QBrush(self.default_color))
        painter.setPen(Qt.NoPen)
        painter.drawEllipse(square_rect)
        
        # Draw progress arc
        if self.value > 0:
            painter.setBrush(QBrush(self.progress_color))
            angle = int(360 * 16 * (self.value / 100))  # Convert to 16ths of degree
            painter.drawPie(square_rect, 90 * 16, -angle)  # Start from top
        
        # Draw inner circle
        painter.setBrush(QBrush(self.inner_bg_color))
        painter.drawEllipse(inner_rect)
        
        # Draw text
        painter.setPen(QColor(255, 255, 255))
        font = QFont()
        font.setPointSize(self.font_size)
        font.setBold(True)
        painter.setFont(font)
        
        text = f"{int(self.value)}%"
        painter.drawText(square_rect, Qt.AlignCenter, text)

class EnhancedRGBController:
    """Enhanced RGB Controller with multi-device support"""
    
    def __init__(self, device_path="/dev/hidraw0"):
        self.device_path = device_path
        self.last_effect_time = 0
        
        # Complete keyboard mapping with enhanced keypad support
        self.keyboard_map = {
            # Special keys
            'esc': 0x00,
            
            # Function keys
            'f1': 0x01, 'f2': 0x02, 'f3': 0x03, 'f4': 0x04,
            'f5': 0x05, 'f6': 0x06, 'f7': 0x07, 'f8': 0x08,
            'f9': 0x09, 'f10': 0x0A, 'f11': 0x0B, 'f12': 0x0C,
            'prtsc': 0x0D, 'scroll': 0x0E, 'pause': 0x0F,
            
            # Navigation keys
            'home': 0x10, 'ins': 0x11, 'pgup': 0x12, 'pgdn': 0x13, 'del': 0x14, 'end': 0x15,
            
            # Number row
            'grave': 0x20, '`': 0x20,
            '1': 0x21, '2': 0x22, '3': 0x23, '4': 0x24, '5': 0x25,
            '6': 0x26, '7': 0x27, '8': 0x28, '9': 0x29, '0': 0x2A,
            'minus': 0x2B, '-': 0x2B, 'equals': 0x2D, '=': 0x2D,
            'backspace': 0x2E, 'bksp': 0x2E,
            
            # Keypad - Enhanced mapping
            'numlock': 0x30, 'kp_divide': 0x31, 'kp_multiply': 0x32, 'kp_minus': 0x33,
            'kp_7': 0x50, 'kp_8': 0x51, 'kp_9': 0x52, 'kp_plus': 0x53,
            'kp_4': 0x70, 'kp_5': 0x71, 'kp_6': 0x72,
            'kp_1': 0x90, 'kp_2': 0x91, 'kp_3': 0x92, 'kp_enter': 0x93,
            'kp_0': 0xB1, 'kp_period': 0xB2, 'kp_dot': 0xB2,
            
            # QWERTY row
            'tab': 0x40, 'q': 0x42, 'w': 0x43, 'e': 0x44, 'r': 0x45,
            't': 0x46, 'y': 0x47, 'u': 0x48, 'i': 0x49, 'o': 0x4A,
            'p': 0x4B, 'lbracket': 0x4C, '[': 0x4C, 'rbracket': 0x4D, ']': 0x4D,
            'backslash': 0x4E, '\\': 0x4E,
            
            # ASDF row
            'capslock': 0x60, 'caps': 0x60, 'a': 0x62, 's': 0x63, 'd': 0x64,
            'f': 0x65, 'g': 0x66, 'h': 0x67, 'j': 0x68, 'k': 0x69,
            'l': 0x6A, 'semicolon': 0x6B, ';': 0x6B, 'quote': 0x6C, "'": 0x6C,
            'enter': 0x6E, 'return': 0x6E,
            
            # ZXCV row
            'lshift': 0x80, 'lshft': 0x80, 'z': 0x83, 'x': 0x84, 'c': 0x85,
            'v': 0x86, 'b': 0x87, 'n': 0x88, 'm': 0x89, 'comma': 0x8A, ',': 0x8A,
            'period': 0x8B, '.': 0x8B, 'slash': 0x8C, '/': 0x8C,
            'rshift': 0x8D, 'rshft': 0x8D,
            
            # Arrow keys
            'up': 0x8F, 'up_arrow': 0x8F, 'left': 0xAE, 'left_arrow': 0xAE,
            'down': 0xAF, 'down_arrow': 0xAF, 'right': 0xB0, 'right_arrow': 0xB0,
            
            # Bottom row modifiers and spacebar
            'lctrl': 0xA0, 'lcontrol': 0xA0, 'fn': 0xA2, 'super': 0xA3, 'win': 0xA3,
            'lalt': 0xA4, 'space_left': 0xA5, 'space_center': 0xA6,
            'space': 0xA8, 'spacebar': 0xA8, 'space_right': 0xA8, 'space_far_right': 0xA9,
            'ralt': 0xAA, 'menu': 0xAB, 'rctrl': 0xAC, 'rcontrol': 0xAC,
        }
        
        # Spatial layout for advanced wave effects
        self.spatial_layout = [
            ['esc', 'f1', 'f2', 'f3', 'f4', 'f5', 'f6', 'f7', 'f8', 'f9', 'f10', 'f11', 'f12', 'prtsc', 'scroll', 'pause', 'numlock', 'kp_divide', 'kp_multiply', 'kp_minus'],
            ['`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', 'bksp', 'ins', 'home', 'pgup', 'kp_7', 'kp_8', 'kp_9', 'kp_plus'],
            ['tab', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\\', 'del', 'end', 'pgdn', 'kp_4', 'kp_5', 'kp_6'],
            ['caps', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', "'", 'enter', '', '', '', 'kp_1', 'kp_2', 'kp_3', 'kp_enter'],
            ['lshift', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', 'rshift', 'up', '', '', '', 'kp_0', '', 'kp_period'],
            ['lctrl', 'fn', 'super', 'lalt', 'space_left', 'space_center', 'space', 'space_far_right', 'ralt', 'menu', 'rctrl', 'left', 'down', 'right']
        ]
        
        # Key groups for effects
        self.key_groups = {
            'function_keys': ['f1', 'f2', 'f3', 'f4', 'f5', 'f6', 'f7', 'f8', 'f9', 'f10', 'f11', 'f12'],
            'number_row': ['`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '='],
            'qwerty_row': ['tab', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\\'],
            'asdf_row': ['caps', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', "'", 'enter'],
            'zxcv_row': ['lshift', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', 'rshift'],
            'bottom_row': ['lctrl', 'fn', 'super', 'lalt', 'space_left', 'space_center', 'space', 'space_far_right', 'ralt', 'menu', 'rctrl'],
            'spacebar_full': ['space_left', 'space_center', 'space', 'space_far_right'],
            'arrow_keys': ['up', 'left', 'down', 'right'],
            'keypad': ['numlock', 'kp_divide', 'kp_multiply', 'kp_minus', 'kp_7', 'kp_8', 'kp_9', 'kp_plus',
                      'kp_4', 'kp_5', 'kp_6', 'kp_1', 'kp_2', 'kp_3', 'kp_enter', 'kp_0', 'kp_period'],
            'letters': ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'z', 'x', 'c', 'v', 'b', 'n', 'm'],
            'navigation': ['ins', 'home', 'pgup', 'del', 'end', 'pgdn'],
            'special': ['esc', 'prtsc', 'scroll', 'pause'],
            'all_keys': list(self.keyboard_map.keys())
        }
    
    def check_permissions(self):
        """Check device permissions"""
        if not os.path.exists(self.device_path):
            return False, f"Device {self.device_path} does not exist"
        
        if not os.access(self.device_path, os.R_OK | os.W_OK):
            return False, f"No read/write permission for {self.device_path}"
            
        return True, "Permissions OK"
    
    def send_key_command(self, key_index, red, green, blue):
        """Send RGB command for specific key"""
        try:
            data = bytes([0xCC, 0x01, key_index, red, green, blue] + [0x00] * 10)
            with open(self.device_path, 'wb') as device:
                device.write(data)
            return True
        except (PermissionError, FileNotFoundError, Exception):
            return False
    
    def set_key_color(self, key_name, red, green, blue):
        """Set color for a specific key"""
        key_lower = key_name.lower()
        if key_lower in self.keyboard_map:
            key_index = self.keyboard_map[key_lower]
            return self.send_key_command(key_index, red, green, blue)
        return False
    
    def set_group_color(self, group_name, red, green, blue):
        """Set color for a group of keys"""
        if group_name in self.key_groups:
            success = True
            for key in self.key_groups[group_name]:
                if not self.set_key_color(key, red, green, blue):
                    success = False
                time.sleep(0.002)  # Minimal delay for smoother effects
            return success
        return False
    
    def clear_all_keys(self):
        """SUPER-enhanced key clearing with AGGRESSIVE kp_plus targeting"""
        # Standard clear - 2 passes
        for _ in range(2):
            for key_index in range(0x00, 0xFF):
                self.send_key_command(key_index, 0, 0, 0)
        
        # SUPER AGGRESSIVE kp_plus clearing (this key is stubborn!)
        kp_plus_indices = [0x53, 0x33, 0x73, 0x93, 0xB3, 0xD3, 0xF3]  # All possible kp_plus locations
        
        for _ in range(8):  # 8 passes for kp_plus specifically
            for idx in kp_plus_indices:
                # Clear exact index
                self.send_key_command(idx, 0, 0, 0)
                
                # Clear surrounding area (wide net)
                for offset in range(-8, 9):
                    clear_idx = max(0, min(0xFF, idx + offset))
                    self.send_key_command(clear_idx, 0, 0, 0)
                
                time.sleep(0.01)  # Small delay between attempts
        
        # Enhanced keypad clearing (other problematic keys)
        other_problematic = ['kp_enter', 'kp_period', 'kp_0', 'kp_1', 'kp_2', 'kp_3', 
                           'kp_4', 'kp_5', 'kp_6', 'kp_7', 'kp_8', 'kp_9', 'kp_divide', 'kp_multiply', 'kp_minus']
        
        for _ in range(4):  # 4 passes for other keypad keys
            for key in other_problematic:
                if key in self.keyboard_map:
                    key_index = self.keyboard_map[key]
                    for offset in range(-4, 5):  # Clear surrounding area
                        clear_index = max(0, min(0xFF, key_index + offset))
                        self.send_key_command(clear_index, 0, 0, 0)
                    time.sleep(0.003)
        
        return True
    
    def force_clear_device(self, device_path):
        """Force clear a specific device by sending clear commands to entire range"""
        try:
            with open(device_path, 'wb') as device:
                for i in range(0x00, 0xFF):
                    data = bytes([0xCC, 0x01, i, 0, 0, 0] + [0x00] * 10)
                    device.write(data)
            return True
        except Exception:
            return False
    
    def advanced_wave_effect(self, duration=20, wave_type='diagonal', stop_event=None):
        """Advanced wave effects with multiple patterns"""
        start_time = time.time()
        wave_position = 0
        
        # Create position mapping based on wave type
        key_positions = {}
        for row_idx, row in enumerate(self.spatial_layout):
            for col_idx, key in enumerate(row):
                if key and key in self.keyboard_map:
                    if wave_type == 'diagonal':
                        key_positions[key] = row_idx + col_idx
                    elif wave_type == 'horizontal':
                        key_positions[key] = row_idx
                    elif wave_type == 'vertical':
                        key_positions[key] = col_idx
                    elif wave_type == 'radial':
                        center_row, center_col = 3, 10  # Center of keyboard
                        distance = math.sqrt((row_idx - center_row)**2 + (col_idx - center_col)**2)
                        key_positions[key] = distance
        
        max_position = max(key_positions.values()) if key_positions else 20
        
        while time.time() - start_time < duration:
            if stop_event and stop_event.is_set():
                break
            
            for key, position in key_positions.items():
                # Calculate wave distance
                wave_distance = abs(position - wave_position)
                
                if wave_distance <= 3:  # Wave thickness
                    # Calculate brightness and color
                    brightness = max(0, 1.0 - (wave_distance / 3.0))
                    
                    # Dynamic color based on position and time
                    hue = (wave_position * 20 + position * 30 + time.time() * 50) % 360
                    r, g, b = self.hsv_to_rgb(hue, 1.0, brightness)
                    self.set_key_color(key, r, g, b)
                else:
                    # Fade out keys outside wave
                    self.set_key_color(key, 0, 0, 0)
            
            # Advance wave
            wave_position = (wave_position + 0.4) % (max_position + 6)
            time.sleep(0.04)  # Smooth animation
    
    @staticmethod
    def hsv_to_rgb(h, s, v):
        """Convert HSV to RGB with enhanced precision"""
        h = h / 360.0
        i = int(h * 6.0)
        f = (h * 6.0) - i
        p = v * (1.0 - s)
        q = v * (1.0 - s * f)
        t = v * (1.0 - s * (1.0 - f))
        
        i = i % 6
        if i == 0:
            r, g, b = v, t, p
        elif i == 1:
            r, g, b = q, v, p
        elif i == 2:
            r, g, b = p, v, t
        elif i == 3:
            r, g, b = p, q, v
        elif i == 4:
            r, g, b = t, p, v
        elif i == 5:
            r, g, b = v, p, q
        
        return int(r * 255), int(g * 255), int(b * 255)

class LidMonitor(QObject):
    """Monitor laptop lid state and clear keyboard on closure - ENHANCED CLEARING"""
    lid_status = pyqtSignal(str)
    
    def __init__(self, rgb_controller, main_window=None):
        super().__init__()
        self.rgb_controller = rgb_controller
        self.main_window = main_window  # Reference to main window for effect control
        self.monitoring = False
        self.monitor_thread = None
        self.last_clear_time = 0
        
    def start_monitoring(self):
        """Start lid monitoring with enhanced detection"""
        if not self.monitoring:
            self.monitor_thread = threading.Thread(target=self._monitor_lid, daemon=True)
            self.monitor_thread.start()
            self.monitoring = True
            self.lid_status.emit("LID MONITORING STARTED - Enhanced detection active")
    
    def _monitor_lid(self):
        """Monitor lid state with multiple detection methods"""
        lid_was_open = True
        consecutive_closed = 0
        
        while self.monitoring:
            try:
                lid_open = self._check_lid_state()
                
                if not lid_open:
                    consecutive_closed += 1
                    if consecutive_closed >= 2 and lid_was_open:  # Require 2 consecutive readings
                        self.lid_status.emit("LID CLOSURE CONFIRMED - Executing keyboard clear")
                        self._aggressive_keyboard_clear()
                        lid_was_open = False
                else:
                    consecutive_closed = 0
                    if not lid_was_open:
                        self.lid_status.emit("Lid reopened")
                        lid_was_open = True
                
                time.sleep(1)  # Check every second
                
            except Exception as e:
                self.lid_status.emit(f"Lid monitor error: {e}")
                time.sleep(5)
    
    def _check_lid_state(self):
        """Check lid state using multiple methods - Enhanced for EON17-X"""
        try:
            # Method 1: ACPI lid button (most reliable for EON17-X)
            acpi_paths = [
                '/proc/acpi/button/lid/LID0/state',
                '/proc/acpi/button/lid/LID/state'
            ]
            
            for path in acpi_paths:
                if os.path.exists(path):
                    try:
                        with open(path, 'r') as f:
                            content = f.read().strip().lower()
                            if 'closed' in content:
                                self.lid_status.emit(f"ACPI lid CLOSED detected: {path}")
                                return False
                    except Exception:
                        pass
            
            # Method 2: Check DPMS (display power management)
            try:
                result = subprocess.run(['xset', 'q'], capture_output=True, text=True, timeout=2)
                if result.returncode == 0:
                    if 'Monitor is Off' in result.stdout or 'Standby' in result.stdout or 'Suspend' in result.stdout:
                        self.lid_status.emit("Display OFF/Standby/Suspend - lid closed")
                        return False
            except Exception:
                pass
            
            # Method 3: Check if we're in a locked session (works on some systems)
            try:
                # Try different session detection methods
                session_cmds = [
                    ['loginctl', 'show-session', '$(loginctl show-user $USER -p Sessions --value | head -1)', '-p', 'LockedHint'],
                    ['gnome-screensaver-command', '-q'],
                    ['qdbus', 'org.freedesktop.ScreenSaver', '/ScreenSaver', 'GetActive']
                ]
                
                for cmd in session_cmds:
                    try:
                        result = subprocess.run(cmd, capture_output=True, text=True, timeout=2, shell=True)
                        if result.returncode == 0:
                            output = result.stdout.lower()
                            if 'lockedhint=yes' in output or 'is active: true' in output or 'true' in output.strip():
                                self.lid_status.emit(f"Screen locked detected via {cmd[0]}")
                                return False
                    except Exception:
                        continue
            except Exception:
                pass
            
            # Method 4: Test file for manual triggering (for testing)
            test_lid_file = Path('/tmp/test_lid_closed')
            if test_lid_file.exists():
                self.lid_status.emit("TEST LID CLOSURE FILE DETECTED")
                return False
            
            # Method 5: Check systemd logind (if available)
            try:
                result = subprocess.run(['busctl', 'get-property', 'org.freedesktop.login1', 
                                       '/org/freedesktop/login1', 'org.freedesktop.login1.Manager', 
                                       'LidClosed'], capture_output=True, text=True, timeout=2)
                if result.returncode == 0 and 'true' in result.stdout.lower():
                    self.lid_status.emit("systemd-logind reports lid closed")
                    return False
            except Exception:
                pass
            
            return True  # Default to open
            
        except Exception as e:
            self.lid_status.emit(f"Lid check error: {e}")
            return True
    
    def _aggressive_keyboard_clear(self):
        """AGGRESSIVE keyboard and effects clearing on lid close (EON-17x optimized)"""
        try:
            if self.rgb_controller:
                self.lid_status.emit("EXECUTING AGGRESSIVE KEYBOARD & EFFECTS CLEAR")
                
                # First: ULTRA-AGGRESSIVE effect stopping
                if self.main_window:
                    try:
                        # STEP 1: Set all stop flags immediately
                        self.main_window.effect_stop_event.set()
                        if hasattr(self.main_window, 'persistent_effects_running'):
                            self.main_window.persistent_effects_running = False
                        
                        # STEP 2: Force kill effect thread with aggressive timeout
                        if hasattr(self.main_window, 'current_effect_thread') and self.main_window.current_effect_thread:
                            if self.main_window.current_effect_thread.is_alive():
                                # Try graceful stop first
                                self.main_window.current_effect_thread.join(timeout=0.1)
                                
                                # If still alive, kill it aggressively
                                if self.main_window.current_effect_thread.is_alive():
                                    try:
                                        import ctypes
                                        thread_id = self.main_window.current_effect_thread.ident
                                        if thread_id:
                                            # Force terminate thread
                                            ctypes.pythonapi.PyThreadState_SetAsyncExc(
                                                ctypes.c_long(thread_id), ctypes.py_object(SystemExit)
                                            )
                                    except Exception:
                                        pass
                        
                        self.main_window.current_effect_thread = None
                        
                        # STEP 3: Clear all possible RGB state immediately
                        for _ in range(3):  # Triple clear
                            self.rgb_controller.clear_all_keys()
                            time.sleep(0.05)
                        
                        self.lid_status.emit("ULTRA-AGGRESSIVELY stopped all effects + cleared RGB")
                        
                    except Exception as e:
                        self.lid_status.emit(f"Effect stop error: {e}")
                
                # Second: Kill any external RGB processes/daemons
                try:
                    subprocess.run(['sudo', 'pkill', '-f', 'rgb.*daemon'], check=False, timeout=2)
                    subprocess.run(['sudo', 'pkill', '-f', 'openrgb'], check=False, timeout=2)
                    subprocess.run(['sudo', 'systemctl', 'stop', 'rgb-daemon'], check=False, timeout=2)
                    self.lid_status.emit("Stopped external RGB processes")
                    time.sleep(0.3)
                except Exception:
                    pass
                
            # Third: Super aggressive clearing - 8 full sweeps for EON-17x
            for sweep in range(8):
                self.lid_status.emit(f"EON-17x clear sweep {sweep + 1}/8")
                
                # Clear entire range
                for key_index in range(0x00, 0xFF):
                    self.rgb_controller.send_key_command(key_index, 0, 0, 0)
                
                time.sleep(0.05)  # Slightly longer delay for EON-17x
                
                # Fourth: MEGA aggressive kp_plus clearing (this key is SUPER stubborn on EON-17x)
                kp_plus_indices = [
                    0x53, 0x33, 0x73, 0x93, 0xB3, 0xD3, 0xF3,  # All possible kp_plus locations
                    0x54, 0x34, 0x74, 0x94, 0xB4, 0xD4, 0xF4   # Adjacent indices too
                ]
                
                for _ in range(12):  # 12 passes specifically for kp_plus on EON-17x
                    for idx in kp_plus_indices:
                        # Clear wide surrounding area
                        for offset in range(-10, 11):
                            clear_idx = max(0, min(0xFF, idx + offset))
                            self.rgb_controller.send_key_command(clear_idx, 0, 0, 0)
                    time.sleep(0.03)
                
                # Fifth: Clear all other keypad keys with extra passes
                other_problem_indices = [
                    0x50, 0x51, 0x52, 0x70, 0x71, 0x72, 0x90, 0x91, 0x92, 0xB1, 0xB2,  # Keypad
                    0x30, 0x31, 0x32  # Additional numlock area
                ]
                
                for _ in range(6):  # 6 additional passes for other problem keys
                    for idx in other_problem_indices:
                        # Clear surrounding area
                        for offset in range(-6, 7):
                            clear_idx = max(0, min(0xFF, idx + offset))
                            self.rgb_controller.send_key_command(clear_idx, 0, 0, 0)
                    time.sleep(0.02)
                
                # Sixth: Force clear multiple device paths for EON-17x
                device_paths = ['/dev/hidraw0', '/dev/hidraw1', '/dev/hidraw2', '/dev/hidraw3']
                for device_path in device_paths:
                    if os.path.exists(device_path):
                        try:
                            self.rgb_controller.force_clear_device(device_path)
                            self.lid_status.emit(f"Force cleared {device_path}")
                        except Exception:
                            pass
                
                self.lid_status.emit("EON-17x AGGRESSIVE CLEAR COMPLETED - Effects stopped")
                
        except Exception as e:
            self.lid_status.emit(f"Clear error: {e}")

class FanController(QObject):
    """Integrated fan control system for EON17-X laptop"""
    fan_status = pyqtSignal(str)
    
    def __init__(self):
        super().__init__()
        self.fan_data = {}
        self.nbfc_available = self.check_nbfc_available()
    
    def check_nbfc_available(self):
        """Check if NBFC is available for fan control"""
        try:
            result = subprocess.run(['nbfc', 'status'], capture_output=True, text=True, timeout=3)
            return result.returncode == 0
        except (subprocess.TimeoutExpired, FileNotFoundError):
            return False
    
    def get_fan_speeds(self):
        """Get current fan speeds from all available sources - EON-17x OPTIMIZED"""
        fans = []
        
        try:
            # Method 1: NBFC fan status (best for Clevo laptops)
            if self.nbfc_available:
                try:
                    result = subprocess.run(['nbfc', 'status'], capture_output=True, text=True, timeout=5)
                    if result.returncode == 0:
                        nbfc_fans = self._parse_nbfc_output(result.stdout)
                        fans.extend(nbfc_fans)
                        if nbfc_fans:
                            self.fan_status.emit(f"Found {len(nbfc_fans)} NBFC-controlled fans")
                except (subprocess.TimeoutExpired, FileNotFoundError):
                    pass
            
            # Method 2: Try ACPI thermal zones for EON-17x
            try:
                thermal_fans = self._get_thermal_zone_fans()
                fans.extend(thermal_fans)
            except Exception as e:
                self.fan_status.emit(f"Thermal zone scan error: {e}")
            
            # Method 3: Try alternative ACPI methods for Clevo
            try:
                acpi_fans = self._get_acpi_fans()
                fans.extend(acpi_fans)
            except Exception:
                pass
            
            # Method 1.5: EON-17x ACPI Thermal Zone fans
            try:
                thermal_zones = glob.glob('/sys/class/thermal/thermal_zone*')
                for zone in thermal_zones:
                    try:
                        with open(f'{zone}/type', 'r') as f:
                            zone_type = f.read().strip()
                        
                        # Look for cooling devices
                        cooling_devices = glob.glob('/sys/class/thermal/cooling_device*')
                        for device in cooling_devices:
                            try:
                                with open(f'{device}/type', 'r') as f:
                                    device_type = f.read().strip()
                                
                                if 'fan' in device_type.lower() or 'thermal' in device_type.lower():
                                    with open(f'{device}/cur_state', 'r') as f:
                                        state = int(f.read().strip())
                                    
                                    if state > 0:
                                        fans.append({
                                            'name': f'EON-17x {device_type}',
                                            'rpm': f'Level {state}',
                                            'source': 'thermal'
                                        })
                            except (IOError, ValueError):
                                pass
                    except IOError:
                        pass
            except Exception:
                pass
            
            # Method 2: psutil sensors
            if hasattr(psutil, 'sensors_fans'):
                try:
                    fan_sensors = psutil.sensors_fans()
                    for sensor_name, fan_list in fan_sensors.items():
                        for i, fan in enumerate(fan_list):
                            fans.append({
                                'name': fan.label or f'{sensor_name} Fan {i+1}',
                                'rpm': fan.current,
                                'source': 'psutil'
                            })
                except Exception:
                    pass
            
            # Method 3: Enhanced /sys/class/hwmon scanning (EON-17x optimized)
            hwmon_dirs = glob.glob('/sys/class/hwmon/hwmon*')
            for hwmon_dir in hwmon_dirs:
                try:
                    # Try to get device name
                    name_file = os.path.join(hwmon_dir, 'name')
                    device_name = 'Unknown'
                    if os.path.exists(name_file):
                        try:
                            with open(name_file, 'r') as f:
                                device_name = f.read().strip()
                        except IOError:
                            pass
                    
                    # EON-17x specific device name mapping
                    if 'acpi' in device_name.lower():
                        device_name = 'EON-17x ACPI'
                    elif 'coretemp' in device_name.lower():
                        device_name = 'EON-17x CPU'
                    elif any(x in device_name.lower() for x in ['it87', 'nct', 'w83']):
                        device_name = 'EON-17x System'
                    
                    # Find fan input files
                    fan_files = glob.glob(os.path.join(hwmon_dir, 'fan*_input'))
                    for fan_file in fan_files:
                        try:
                            with open(fan_file, 'r') as f:
                                rpm = int(f.read().strip())
                                if rpm > 0:  # Only add fans that are actually spinning
                                    fan_num = os.path.basename(fan_file).replace('fan', '').replace('_input', '')
                                    
                                    # Try to get fan label
                                    label_file = fan_file.replace('_input', '_label')
                                    fan_label = f'Fan {fan_num}'
                                    if os.path.exists(label_file):
                                        try:
                                            with open(label_file, 'r') as lf:
                                                fan_label = lf.read().strip()
                                        except IOError:
                                            pass
                                    
                                    # EON-17x specific fan naming
                                    if fan_num == '1':
                                        fan_label = 'CPU Fan'
                                    elif fan_num == '2':
                                        fan_label = 'System Fan'
                                    elif fan_num == '3':
                                        fan_label = 'GPU Fan'
                                    
                                    fans.append({
                                        'name': f'{device_name} {fan_label}',
                                        'rpm': rpm,
                                        'source': 'hwmon'
                                    })
                        except (ValueError, IOError):
                            pass
                except OSError:
                    pass
            
            # Method 3.5: EON-17x specific PWM fan detection
            try:
                pwm_files = glob.glob('/sys/class/hwmon/hwmon*/pwm*')
                for pwm_file in pwm_files:
                    try:
                        if pwm_file.endswith('_enable'):
                            continue
                        
                        with open(pwm_file, 'r') as f:
                            pwm_value = int(f.read().strip())
                        
                        if pwm_value > 0:
                            pwm_num = os.path.basename(pwm_file).replace('pwm', '')
                            fan_name = f'EON-17x PWM Fan {pwm_num}'
                            
                            # Try to calculate approximate RPM from PWM
                            estimated_rpm = int((pwm_value / 255.0) * 3000)  # Rough estimate
                            
                            fans.append({
                                'name': fan_name,
                                'rpm': f'{estimated_rpm} RPM (PWM: {pwm_value})',
                                'source': 'pwm'
                            })
                    except (ValueError, IOError):
                        pass
            except Exception:
                pass
            
            # Method 4: Try ACPI fan info
            try:
                if os.path.exists('/proc/acpi/fan'):
                    fan_dirs = glob.glob('/proc/acpi/fan/*')
                    for fan_dir in fan_dirs:
                        state_file = os.path.join(fan_dir, 'state')
                        if os.path.exists(state_file):
                            try:
                                with open(state_file, 'r') as f:
                                    content = f.read().strip()
                                    if 'on' in content.lower():
                                        fan_name = os.path.basename(fan_dir)
                                        fans.append({
                                            'name': f'ACPI {fan_name}',
                                            'rpm': 'Active',
                                            'source': 'acpi'
                                        })
                            except IOError:
                                pass
            except Exception:
                pass
            
            # Remove duplicates based on name
            seen_names = set()
            unique_fans = []
            for fan in fans:
                if fan['name'] not in seen_names:
                    seen_names.add(fan['name'])
                    unique_fans.append(fan)
            
            fans = unique_fans
        
        except Exception as e:
            self.fan_status.emit(f"Fan detection error: {e}")
        
        return fans
    
    def set_fan_mode(self, mode):
        """Set fan control mode using NBFC (not fancontrol)"""
        try:
            if mode == 'auto':
                # Use NBFC auto mode instead of fancontrol
                subprocess.run(['nbfc', 'set', '-a'], check=False, timeout=5)
                self.fan_status.emit("NBFC automatic fan control enabled")
            elif mode == 'performance':
                # Set fans to high speed (if possible)
                self.fan_status.emit("Performance mode activated (manual control)")
            elif mode == 'silent':
                # Set fans to low speed using NBFC if available
                try:
                    # Try to set NBFC to low profile or manual low speed
                    subprocess.run(['nbfc', 'set', '-s', '30'], check=False, timeout=5)  # 30% speed
                    self.fan_status.emit("Silent mode activated - fans set to 30% speed")
                except Exception:
                    # Fallback silent mode message
                    self.fan_status.emit("Silent mode activated (manual control - adjust via NBFC)")
        except Exception as e:
            self.fan_status.emit(f"Fan control error: {e}")
    
    def launch_fancontrol_gui(self):
        """Launch integrated fan control or external GUI"""
        # Try to launch integrated fan control window first
        try:
            self.show_integrated_fan_control()
            self.fan_status.emit("Launched integrated fan control")
            return True
        except Exception as e:
            self.fan_status.emit(f"Integrated fan control error: {e}")
        
        # Fallback to external applications
        apps_to_try = [
            (['fancontrol-gui'], 'Fan Control GUI'),
            (['qfancontrol'], 'QFanControl'),
            (['konsole', '-e', 'watch', '-n', '1', 'sensors'], 'Sensors Monitor'),
            (['gnome-terminal', '--', 'watch', '-n', '1', 'sensors'], 'Sensors Monitor'),
            (['xterm', '-e', 'watch', '-n', '1', 'sensors'], 'Sensors Monitor')
        ]
        
        for cmd, app_name in apps_to_try:
            try:
                # Check if first command exists
                if subprocess.run(['which', cmd[0]], capture_output=True, timeout=1).returncode == 0:
                    subprocess.Popen(cmd, start_new_session=True, 
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    self.fan_status.emit(f"Launched {app_name}")
                    return True
            except Exception:
                continue
        
        self.fan_status.emit("No fan control applications found")
        return False
    
    def show_integrated_fan_control(self):
        """Show integrated fan control window"""
        from PyQt5.QtWidgets import QDialog, QVBoxLayout, QHBoxLayout, QLabel, QTextEdit, QPushButton
        from PyQt5.QtCore import QTimer
        from PyQt5.QtCore import Qt
        
        dialog = QDialog(self.parent() if hasattr(self, 'parent') else None)
        dialog.setWindowTitle("Integrated Fan Control")
        dialog.setMinimumSize(500, 400)
        
        layout = QVBoxLayout(dialog)
        
        # Header
        header = QLabel("Fan Control & Monitoring")
        header.setObjectName("panel_header")
        header.setAlignment(Qt.AlignCenter)
        layout.addWidget(header)
        
        # Fan speeds display
        fan_text = QTextEdit()
        fan_text.setObjectName("system_info")
        fan_text.setReadOnly(True)
        layout.addWidget(fan_text)
        
        # Update fan info
        def update_fan_info():
            fans = self.get_fan_speeds()
            if fans:
                info = "Current Fan Speeds:\n\n"
                for fan in fans:
                    info += f"{fan['name']}: {fan['rpm']} RPM\n"
            else:
                info = "No fan sensors detected\n\nTry installing lm-sensors:\nsudo pacman -S lm-sensors\nsudo sensors-detect"
            fan_text.setPlainText(info)
        
        # Control buttons
        btn_layout = QHBoxLayout()
        
        refresh_btn = QPushButton("Refresh")
        refresh_btn.setObjectName("group_button")
        refresh_btn.clicked.connect(update_fan_info)
        btn_layout.addWidget(refresh_btn)
        
        close_btn = QPushButton("Close")
        close_btn.setObjectName("clear_button")
        close_btn.clicked.connect(dialog.close)
        btn_layout.addWidget(close_btn)
        
        layout.addLayout(btn_layout)
        
        # Initial update
        update_fan_info()
        
        # Auto-refresh timer
        timer = QTimer(dialog)
        timer.timeout.connect(update_fan_info)
        timer.start(2000)
        
        dialog.exec_()
    
    def _parse_nbfc_output(self, nbfc_output):
        """Parse NBFC status output for fan information"""
        fans = []
        try:
            lines = nbfc_output.split('\n')
            current_fan_data = {}
            
            for line in lines:
                line = line.strip()
                
                # Parse NBFC status format
                if 'Fan Display Name' in line and ':' in line:
                    current_fan_data['name'] = line.split(':', 1)[1].strip()
                elif 'Current Fan Speed' in line and ':' in line:
                    speed_str = line.split(':', 1)[1].strip()
                    current_fan_data['speed'] = speed_str
                elif 'Temperature' in line and ':' in line:
                    temp_str = line.split(':', 1)[1].strip()
                    current_fan_data['temp'] = temp_str
                
                # If we have enough data, add the fan
                if 'name' in current_fan_data and 'speed' in current_fan_data:
                    fans.append({
                        'name': f"NBFC {current_fan_data['name']}",
                        'rpm': f"{current_fan_data['speed']}% (Temp: {current_fan_data.get('temp', 'N/A')})",
                        'source': 'nbfc'
                    })
                    current_fan_data = {}  # Reset for next fan
                    
        except Exception as e:
            self.fan_status.emit(f"NBFC parse error: {e}")
        return fans
    
    def _get_thermal_zone_fans(self):
        """Get fan information from thermal zones"""
        fans = []
        try:
            thermal_zones = glob.glob('/sys/class/thermal/thermal_zone*')
            for zone in thermal_zones:
                try:
                    with open(f'{zone}/type', 'r') as f:
                        zone_type = f.read().strip()
                    
                    # Look for cooling devices associated with this zone
                    cooling_devices = glob.glob('/sys/class/thermal/cooling_device*')
                    for device in cooling_devices:
                        try:
                            with open(f'{device}/type', 'r') as f:
                                device_type = f.read().strip()
                            
                            if 'fan' in device_type.lower() or 'thermal' in device_type.lower():
                                with open(f'{device}/cur_state', 'r') as f:
                                    state = int(f.read().strip())
                                
                                if state > 0:
                                    fans.append({
                                        'name': f'EON-17x {device_type}',
                                        'rpm': f'Level {state}',
                                        'source': 'thermal'
                                    })
                        except (IOError, ValueError):
                            pass
                except IOError:
                    pass
        except Exception:
            pass
        return fans
    
    def _get_acpi_fans(self):
        """Get fan information from ACPI"""
        fans = []
        try:
            if os.path.exists('/proc/acpi/fan'):
                fan_dirs = glob.glob('/proc/acpi/fan/*')
                for fan_dir in fan_dirs:
                    state_file = os.path.join(fan_dir, 'state')
                    if os.path.exists(state_file):
                        try:
                            with open(state_file, 'r') as f:
                                content = f.read().strip()
                                if 'on' in content.lower():
                                    fan_name = os.path.basename(fan_dir)
                                    fans.append({
                                        'name': f'ACPI {fan_name}',
                                        'rpm': 'Active',
                                        'source': 'acpi'
                                    })
                        except IOError:
                            pass
        except Exception:
            pass
        return fans

class PowerManager(QObject):
    """Power management integration"""
    power_status = pyqtSignal(str)
    
    def __init__(self):
        super().__init__()
    
    def get_power_info(self):
        """Get comprehensive power information"""
        power_info = {
            'ac_connected': False,
            'battery_percent': 0,
            'power_profile': 'unknown',
            'cpu_freq': {},
            'tlp_status': 'unknown'
        }
        
        try:
            # Battery information
            if hasattr(psutil, 'sensors_battery'):
                battery = psutil.sensors_battery()
                if battery:
                    power_info['battery_percent'] = battery.percent
                    power_info['ac_connected'] = battery.power_plugged
            
            # CPU frequency
            try:
                cpu_freq = psutil.cpu_freq()
                if cpu_freq:
                    power_info['cpu_freq'] = {
                        'current': cpu_freq.current,
                        'min': cpu_freq.min,
                        'max': cpu_freq.max
                    }
            except AttributeError:
                pass
            
            # TLP status
            try:
                result = subprocess.run(['tlp-stat', '-s'], capture_output=True, text=True, timeout=10)
                if result.returncode == 0:
                    if 'TLP status' in result.stdout:
                        power_info['tlp_status'] = 'active'
                    for line in result.stdout.split('\n'):
                        if 'Mode' in line and '=' in line:
                            power_info['power_profile'] = line.split('=')[1].strip()
                            break
            except (subprocess.TimeoutExpired, FileNotFoundError):
                pass
        
        except Exception as e:
            self.power_status.emit(f"Power info error: {e}")
        
        return power_info
    
    def set_power_profile(self, profile):
        """Set power profile"""
        try:
            if profile == 'performance':
                subprocess.run(['sudo', 'tlp', 'start'], check=False, timeout=5)
                subprocess.run(['sudo', 'cpupower', 'frequency-set', '-g', 'performance'], check=False, timeout=5)
                self.power_status.emit("Performance profile activated")
            elif profile == 'balanced':
                subprocess.run(['sudo', 'tlp', 'start'], check=False, timeout=5)
                subprocess.run(['sudo', 'cpupower', 'frequency-set', '-g', 'ondemand'], check=False, timeout=5)
                self.power_status.emit("Balanced profile activated")
            elif profile == 'powersave':
                subprocess.run(['sudo', 'tlp', 'start'], check=False, timeout=5)
                subprocess.run(['sudo', 'cpupower', 'frequency-set', '-g', 'powersave'], check=False, timeout=5)
                self.power_status.emit("Power save profile activated")
        except Exception as e:
            self.power_status.emit(f"Profile change error: {e}")
    
    def show_tlp_stats(self):
        """Show TLP statistics in external terminal with keep-open"""
        # Create a script that shows TLP stats and keeps terminal open
        script_content = '''#!/bin/bash
echo "=== TLP Power Management Statistics ==="
echo "Press any key to exit after viewing"
echo
sudo tlp-stat
echo
echo "Press any key to exit..."
read -n 1'''
        
        script_path = '/tmp/tlp_stats.sh'
        try:
            with open(script_path, 'w') as f:
                f.write(script_content)
            os.chmod(script_path, 0o755)
            
            # Try different terminals
            terminals = [
                ['konsole', '-e', script_path],
                ['gnome-terminal', '--', script_path],
                ['xterm', '-e', script_path],
                ['x-terminal-emulator', '-e', script_path]
            ]
            
            for terminal_cmd in terminals:
                try:
                    subprocess.Popen(terminal_cmd, start_new_session=True)
                    self.power_status.emit(f"Launched TLP stats in {terminal_cmd[0]}")
                    return True
                except FileNotFoundError:
                    continue
            
            self.power_status.emit("No suitable terminal found for TLP stats")
            return False
            
        except Exception as e:
            self.power_status.emit(f"TLP stats error: {e}")
            return False

class TemperatureMonitor(QObject):
    """Comprehensive temperature monitoring"""
    temp_updated = pyqtSignal(dict)
    
    def __init__(self):
        super().__init__()
        self.monitoring = False
        self.monitor_thread = None
    
    def start_monitoring(self):
        """Start temperature monitoring"""
        if not self.monitoring:
            self.monitor_thread = threading.Thread(target=self._monitor_loop, daemon=True)
            self.monitor_thread.start()
            self.monitoring = True
    
    def _monitor_loop(self):
        """Continuous temperature monitoring loop"""
        while self.monitoring:
            try:
                temp_data = self.get_all_temperatures()
                self.temp_updated.emit(temp_data)
                time.sleep(2)  # Update every 2 seconds
            except Exception as e:
                print(f"Temperature monitor error: {e}")
                time.sleep(5)
    
    def get_all_temperatures(self):
        """Get all available temperature sensors"""
        temps = {
            'cpu_cores': [],
            'gpu': [],
            'storage': [],
            'memory': [],
            'motherboard': []
        }
        
        try:
            # Method 1: sensors command (most comprehensive)
            try:
                result = subprocess.run(['sensors'], capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    self._parse_sensors_temperatures(result.stdout, temps)
            except (subprocess.TimeoutExpired, FileNotFoundError):
                pass
            
            # Method 2: psutil
            if hasattr(psutil, 'sensors_temperatures'):
                psutil_temps = psutil.sensors_temperatures()
                
                if 'coretemp' in psutil_temps:
                    for sensor in psutil_temps['coretemp']:
                        temps['cpu_cores'].append((sensor.label or 'CPU Core', sensor.current))
                
                if 'nvme' in psutil_temps:
                    for sensor in psutil_temps['nvme']:
                        temps['storage'].append((sensor.label or 'NVME', sensor.current))
            
            # Method 3: Direct hwmon reading
            self._read_hwmon_temperatures(temps)
        
        except Exception as e:
            print(f"Temperature reading error: {e}")
        
        return temps
    
    def _parse_sensors_temperatures(self, sensors_output, temps):
        """Parse sensors command output for temperatures"""
        lines = sensors_output.split('\n')
        current_chip = None
        
        for line in lines:
            line = line.strip()
            if line and not line.startswith('+') and ':' not in line and not line.startswith('Adapter'):
                current_chip = line
            elif ':' in line and current_chip:
                parts = line.split(':')
                if len(parts) >= 2:
                    name = parts[0].strip()
                    value_part = parts[1].strip()
                    
                    if '°C' in value_part and ('temp' in name.lower() or 'core' in name.lower()):
                        try:
                            temp_val = float(value_part.split('°C')[0].split()[-1])
                            
                            if 'cpu' in current_chip.lower() or 'core' in current_chip.lower():
                                temps['cpu_cores'].append((f"{current_chip} {name}", temp_val))
                            elif 'gpu' in current_chip.lower() or 'nvidia' in current_chip.lower():
                                temps['gpu'].append((f"{current_chip} {name}", temp_val))
                            elif 'nvme' in current_chip.lower() or 'ssd' in current_chip.lower():
                                temps['storage'].append((f"{current_chip} {name}", temp_val))
                            elif 'memory' in current_chip.lower() or 'dimm' in current_chip.lower():
                                temps['memory'].append((f"{current_chip} {name}", temp_val))
                            else:
                                temps['motherboard'].append((f"{current_chip} {name}", temp_val))
                        except ValueError:
                            pass
    
    def _read_hwmon_temperatures(self, temps):
        """Read temperatures directly from hwmon"""
        hwmon_dirs = glob.glob('/sys/class/hwmon/hwmon*')
        for hwmon_dir in hwmon_dirs:
            try:
                temp_files = glob.glob(os.path.join(hwmon_dir, 'temp*_input'))
                for temp_file in temp_files:
                    try:
                        with open(temp_file, 'r') as f:
                            temp_millidegrees = int(f.read().strip())
                            temp_celsius = temp_millidegrees / 1000.0
                            
                            # Try to get label
                            label_file = temp_file.replace('_input', '_label')
                            label = f"Sensor {os.path.basename(temp_file)}"
                            if os.path.exists(label_file):
                                try:
                                    with open(label_file, 'r') as lf:
                                        label = lf.read().strip()
                                except IOError:
                                    pass
                            
                            temps['motherboard'].append((label, temp_celsius))
                    except (ValueError, IOError):
                        pass
            except OSError:
                pass
    
    def show_temperature_monitor(self):
        """Show temperature monitor in external terminal"""
        try:
            # Create a temporary script for temperature monitoring
            script_content = '''#!/bin/bash
echo "=== OriginPC EON17-X System Monitor ==="
echo "Press Ctrl+C to exit"
echo

while true; do
    clear
    echo "=== OriginPC EON17-X System Monitor ==="
    echo "$(date)"
    echo
    
    echo "=== CPU Temperatures ==="
    sensors coretemp-isa-0000 | grep -E "Package|Core" 2>/dev/null || echo "No coretemp sensors found"
    echo
    
    echo "=== Storage Temperatures ==="
    sensors | grep -E "nvme|Composite" 2>/dev/null || echo "No storage sensors found"
    echo
    
    echo "=== Memory Temperatures ==="
    sensors | grep -E "spd5118" 2>/dev/null || echo "No memory sensors found"
    echo
    
    echo "=== System Load ==="
    uptime
    echo
    
    echo "=== CPU Frequency ==="
    grep MHz /proc/cpuinfo | head -8 2>/dev/null || echo "CPU frequency info unavailable"
    echo
    
    sleep 2
done
'''
            
            script_path = '/tmp/temp_monitor.sh'
            with open(script_path, 'w') as f:
                f.write(script_content)
            os.chmod(script_path, 0o755)
            
            subprocess.Popen(['konsole', '-e', script_path], start_new_session=True)
            return True
        except FileNotFoundError:
            try:
                subprocess.Popen(['gnome-terminal', '--', script_path], start_new_session=True)
                return True
            except FileNotFoundError:
                return False
        except Exception as e:
            print(f"Temperature monitor launch error: {e}")
            return False

class EnhancedControlCenter(QMainWindow):
    """Main Enhanced Control Center Application"""
    
    def __init__(self):
        super().__init__()
        self.setup_application()
        self.init_components()
        self.create_ui()
        self.setup_system_tray_actual()
        self.start_monitoring()
        
        # Auto-start lid monitoring after 2 seconds
        QTimer.singleShot(2000, self.auto_start_lid_monitoring)
        
        # Load and restore previous effect state
        QTimer.singleShot(5000, self.load_effect_state)
        
    def setup_application(self):
        """Initialize application settings and variables"""
        self.setWindowTitle("Enhanced OriginPC Control Center v5.0")
        self.setMinimumSize(1200, 800)
        self.setStyleSheet(self.get_professional_stylesheet())
        
        # Application state
        self.current_color = (255, 102, 0)  # TCC Orange
        self.rgb_connected = False
        self.effect_stop_event = threading.Event()
        self.current_effect_thread = None
        self.tray_icon = None
        
        # System monitoring
        self.system_data = {}
        self.monitoring_active = False
        
    def init_components(self):
        """Initialize core components"""
        # RGB Controller
        self.rgb_controller = EnhancedRGBController()
        self.check_rgb_connection()
        
        # System data updater
        self.system_updater = SystemDataUpdater(update_interval=1000)
        self.system_updater.data_updated.connect(self.update_system_data)
        
    def create_ui(self):
        """Create the main user interface with menu system"""
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        # Main layout - horizontal split
        main_layout = QHBoxLayout(central_widget)
        main_layout.setContentsMargins(10, 10, 10, 10)
        main_layout.setSpacing(10)
        
        # Left side - Always visible monitoring panel
        self.create_monitoring_panel(main_layout)
        
        # Right side - Tabbed interface for other controls
        self.create_tabbed_interface(main_layout)
        
        # Fix font issues
        font = QFont()
        font.setFamily("Arial,Helvetica,sans-serif")
        font.setPointSize(10)
        self.setFont(font)
        
        # Status bar with tray button
        self.statusBar().showMessage("Enhanced Control Center Ready")
        
        # Add minimize to tray button in status bar
        tray_button = QPushButton("📱 Minimize to Tray")
        tray_button.setObjectName("tray_button")
        tray_button.clicked.connect(self.minimize_to_tray)
        tray_button.setMaximumWidth(150)
        self.statusBar().addPermanentWidget(tray_button)
        
    def create_monitoring_panel(self, parent_layout):
        """Create system monitoring panel (Nyx-inspired)"""
        panel = QWidget()
        panel.setObjectName("monitoring_panel")
        panel.setMinimumWidth(350)
        panel.setMaximumWidth(350)
        
        layout = QVBoxLayout(panel)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Header
        header = QLabel("System Monitor")
        header.setObjectName("panel_header")
        header.setAlignment(Qt.AlignCenter)
        layout.addWidget(header)
        
        # Progress bars section
        progress_section = QWidget()
        progress_layout = QGridLayout(progress_section)
        
        # CPU Progress
        cpu_label = QLabel("CPU Usage")
        cpu_label.setAlignment(Qt.AlignCenter)
        cpu_label.setObjectName("monitor_label")
        self.cpu_progress = QRoundProgressBar(font_size=16)
        
        progress_layout.addWidget(cpu_label, 0, 0)
        progress_layout.addWidget(self.cpu_progress, 1, 0)
        
        # Memory Progress
        mem_label = QLabel("Memory Usage")
        mem_label.setAlignment(Qt.AlignCenter)
        mem_label.setObjectName("monitor_label")
        self.memory_progress = QRoundProgressBar(font_size=16, 
            progress_color=QColor(255, 150, 0, 255))
        
        progress_layout.addWidget(mem_label, 0, 1)
        progress_layout.addWidget(self.memory_progress, 1, 1)
        
        layout.addWidget(progress_section)
        
        # Temperature section
        temp_section = QWidget()
        temp_layout = QVBoxLayout(temp_section)
        
        temp_header = QLabel("Temperature Monitoring")
        temp_header.setObjectName("section_header")
        temp_layout.addWidget(temp_header)
        
        self.cpu_temp_label = QLabel("CPU: --°C")
        self.cpu_temp_label.setObjectName("temp_label")
        temp_layout.addWidget(self.cpu_temp_label)
        
        self.gpu_temp_label = QLabel("GPU: --°C")
        self.gpu_temp_label.setObjectName("temp_label")
        temp_layout.addWidget(self.gpu_temp_label)
        
        layout.addWidget(temp_section)
        
        # System info section
        info_section = QWidget()
        info_layout = QVBoxLayout(info_section)
        
        info_header = QLabel("System Information")
        info_header.setObjectName("section_header")
        info_layout.addWidget(info_header)
        
        self.system_info_text = QTextEdit()
        self.system_info_text.setObjectName("system_info")
        self.system_info_text.setMaximumHeight(150)
        self.system_info_text.setReadOnly(True)
        info_layout.addWidget(self.system_info_text)
        
        layout.addWidget(info_section)
        layout.addStretch()
        
        parent_layout.addWidget(panel)
        
    def create_control_panel(self, parent_layout):
        """Create RGB control panel"""
        panel = QWidget()
        panel.setObjectName("control_panel")
        panel.setMinimumWidth(400)
        
        layout = QVBoxLayout(panel)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Header
        header = QLabel("RGB Lighting Control")
        header.setObjectName("panel_header")
        header.setAlignment(Qt.AlignCenter)
        layout.addWidget(header)
        
        # Connection status
        self.connection_label = QLabel("Checking device...")
        self.connection_label.setObjectName("connection_status")
        layout.addWidget(self.connection_label)
        
        # Color picker
        color_section = QWidget()
        color_layout = QVBoxLayout(color_section)
        
        color_header = QLabel("Color Selection")
        color_header.setObjectName("section_header")
        color_layout.addWidget(color_header)
        
        # Color wheel or picker
        self.color_picker = QColorDialog()
        color_button = QPushButton("Choose Color")
        color_button.setObjectName("color_button")
        color_button.clicked.connect(self.choose_color)
        color_layout.addWidget(color_button)
        
        # Current color display
        self.color_display = QLabel()
        self.color_display.setObjectName("color_display")
        self.color_display.setMinimumHeight(50)
        self.color_display.setStyleSheet(f"background-color: rgb{self.current_color}; border: 2px solid #444;")
        color_layout.addWidget(self.color_display)
        
        layout.addWidget(color_section)
        
        # Key groups
        groups_section = QWidget()
        groups_layout = QVBoxLayout(groups_section)
        
        groups_header = QLabel("Key Groups")
        groups_header.setObjectName("section_header")
        groups_layout.addWidget(groups_header)
        
        # Create group buttons
        groups = [
            ("All Keys", "all_keys"),
            ("Function Keys", "function_keys"),
            ("WASD", "wasd"),
            ("Arrow Keys", "arrow_keys"),
            ("Number Pad", "keypad"),
            ("Spacebar", "spacebar_full")
        ]
        
        for group_name, group_key in groups:
            btn = QPushButton(group_name)
            btn.setObjectName("group_button")
            btn.clicked.connect(lambda checked, gk=group_key: self.set_group_color(gk))
            groups_layout.addWidget(btn)
        
        layout.addWidget(groups_section)
        
        # Control buttons
        control_section = QWidget()
        control_layout = QVBoxLayout(control_section)
        
        clear_btn = QPushButton("Clear All Keys")
        clear_btn.setObjectName("clear_button")
        clear_btn.clicked.connect(self.clear_all_keys)
        control_layout.addWidget(clear_btn)
        
        layout.addWidget(control_section)
        layout.addStretch()
        
        parent_layout.addWidget(panel)
        
    def create_tabbed_interface(self, parent_layout):
        """Create tabbed interface for organized controls"""
        tab_widget = QTabWidget()
        tab_widget.setObjectName("main_tabs")
        
        # RGB Control Tab
        rgb_tab = QWidget()
        self.create_control_panel_content(rgb_tab)
        tab_widget.addTab(rgb_tab, "🎨 RGB Control")
        
        # System Integration Tab  
        system_tab = QWidget()
        self.create_system_panel_content(system_tab)
        tab_widget.addTab(system_tab, "🔧 System")
        
        # Effects Tab
        effects_tab = QWidget()
        self.create_effects_panel_content(effects_tab)
        tab_widget.addTab(effects_tab, "✨ Effects")
        
        # Key Binding Tab (new organized layout)
        bindings_tab = QWidget()
        self.create_key_bindings_panel(bindings_tab)
        tab_widget.addTab(bindings_tab, "⌨️ Key Bindings")
        
        parent_layout.addWidget(tab_widget)
        
    def create_control_panel_content(self, parent_widget):
        """Create RGB control content for tab"""
        layout = QVBoxLayout(parent_widget)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Connection status
        self.connection_label = QLabel("Checking device...")
        self.connection_label.setObjectName("connection_status")
        layout.addWidget(self.connection_label)
        
        # Color picker section with better spacing
        color_section = QGroupBox("Color Selection")
        color_layout = QVBoxLayout(color_section)
        
        # Color wheel or picker
        color_button = QPushButton("Choose Color")
        color_button.setObjectName("color_button")
        color_button.clicked.connect(self.choose_color)
        color_button.setMinimumHeight(40)
        color_layout.addWidget(color_button)
        
        # Current color display with better sizing
        self.color_display = QLabel()
        self.color_display.setObjectName("color_display")
        self.color_display.setMinimumHeight(60)
        self.color_display.setStyleSheet(f"background-color: rgb{self.current_color}; border: 2px solid #444;")
        color_layout.addWidget(self.color_display)
        
        # Quick color presets
        presets_widget = QWidget()
        presets_layout = QGridLayout(presets_widget)
        presets_layout.setSpacing(5)
        
        color_presets = [
            ("Red", (255, 0, 0)),
            ("Green", (0, 255, 0)),
            ("Blue", (0, 0, 255)),
            ("Orange", (255, 102, 0)),
            ("Purple", (128, 0, 128)),
            ("Cyan", (0, 255, 255)),
            ("Yellow", (255, 255, 0)),
            ("White", (255, 255, 255))
        ]
        
        for i, (name, color) in enumerate(color_presets):
            row = i // 4
            col = i % 4
            preset_btn = QPushButton(name)
            preset_btn.setObjectName("preset_button")
            preset_btn.setMinimumHeight(25)
            preset_btn.setStyleSheet(f"background-color: rgb{color}; color: {'black' if sum(color) > 400 else 'white'};")
            preset_btn.clicked.connect(lambda checked, c=color: self.set_current_color_immediate(c))
            presets_layout.addWidget(preset_btn, row, col)
        
        color_layout.addWidget(presets_widget)
        
        layout.addWidget(color_section)
        
        # Key groups with better organization
        groups_section = QGroupBox("Quick Key Groups")
        groups_layout = QGridLayout(groups_section)
        
        # Create group buttons in a grid for better space usage
        groups = [
            ("All Keys", "all_keys"),
            ("Function Keys", "function_keys"),
            ("WASD", "wasd"),
            ("Arrow Keys", "arrow_keys"),
            ("Number Pad", "keypad"),
            ("Spacebar", "spacebar_full")
        ]
        
        for i, (group_name, group_key) in enumerate(groups):
            row = i // 2
            col = i % 2
            btn = QPushButton(group_name)
            btn.setObjectName("group_button")
            btn.setMinimumHeight(35)
            btn.clicked.connect(lambda checked, gk=group_key: self.set_group_color(gk))
            groups_layout.addWidget(btn, row, col)
        
        layout.addWidget(groups_section)
        
        # Control buttons
        control_section = QGroupBox("Controls")
        control_layout = QVBoxLayout(control_section)
        
        clear_btn = QPushButton("Clear All Keys")
        clear_btn.setObjectName("clear_button")
        clear_btn.setMinimumHeight(40)
        clear_btn.clicked.connect(self.clear_all_keys)
        control_layout.addWidget(clear_btn)
        
        layout.addWidget(control_section)
        layout.addStretch()
        
    def create_system_panel_content(self, parent_widget):
        """Create system integration content for tab"""
        layout = QVBoxLayout(parent_widget)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Initialize system components
        self.fan_controller = FanController()
        self.power_manager = PowerManager()
        self.temp_monitor = TemperatureMonitor()
        self.lid_monitor = LidMonitor(self.rgb_controller, self)
        
        # Connect signals
        self.fan_controller.fan_status.connect(lambda msg: self.statusBar().showMessage(f"Fan: {msg}"))
        self.power_manager.power_status.connect(lambda msg: self.statusBar().showMessage(f"Power: {msg}"))
        self.lid_monitor.lid_status.connect(lambda msg: self.statusBar().showMessage(f"Lid: {msg}"))
        
        # Fan Control Section - Compact
        fan_section = QGroupBox("🌀 Fan Control")
        fan_layout = QVBoxLayout(fan_section)
        
        # Fan speed display - more compact
        self.fan_display = QLabel("Checking fans...")
        self.fan_display.setObjectName("temp_label")
        self.fan_display.setMinimumHeight(40)
        fan_layout.addWidget(self.fan_display)
        
        # Fan control buttons - horizontal layout
        fan_buttons = QWidget()
        fan_btn_layout = QHBoxLayout(fan_buttons)
        
        fan_auto_btn = QPushButton("Auto")
        fan_auto_btn.setObjectName("group_button")
        fan_auto_btn.setMinimumHeight(30)
        fan_auto_btn.clicked.connect(lambda: self.fan_controller.set_fan_mode('auto'))
        
        fan_silent_btn = QPushButton("Silent")
        fan_silent_btn.setObjectName("group_button")
        fan_silent_btn.setMinimumHeight(30)
        fan_silent_btn.clicked.connect(lambda: self.fan_controller.set_fan_mode('silent'))
        
        fan_gui_btn = QPushButton("Fan GUI")
        fan_gui_btn.setObjectName("effect_start_button")
        fan_gui_btn.setMinimumHeight(30)
        fan_gui_btn.clicked.connect(self.fan_controller.launch_fancontrol_gui)
        
        fan_btn_layout.addWidget(fan_auto_btn)
        fan_btn_layout.addWidget(fan_silent_btn)
        fan_btn_layout.addWidget(fan_gui_btn)
        fan_layout.addWidget(fan_buttons)
        
        layout.addWidget(fan_section)
        
        # Power Management Section - Compact
        power_section = QGroupBox("⚡ Power Management")
        power_layout = QVBoxLayout(power_section)
        
        # Power info display - more compact
        self.power_display = QLabel("Checking power...")
        self.power_display.setObjectName("temp_label")
        self.power_display.setMinimumHeight(40)
        power_layout.addWidget(self.power_display)
        
        # Power profile buttons - grid layout for all modes
        power_buttons = QWidget()
        power_btn_layout = QGridLayout(power_buttons)
        
        power_perf_btn = QPushButton("Performance")
        power_perf_btn.setObjectName("group_button")
        power_perf_btn.setMinimumHeight(25)
        power_perf_btn.clicked.connect(lambda: self.power_manager.set_power_profile('performance'))
        
        power_bal_btn = QPushButton("Balanced")
        power_bal_btn.setObjectName("group_button")
        power_bal_btn.setMinimumHeight(25)
        power_bal_btn.clicked.connect(lambda: self.power_manager.set_power_profile('balanced'))
        
        power_save_btn = QPushButton("Power Save")
        power_save_btn.setObjectName("group_button")
        power_save_btn.setMinimumHeight(25)
        power_save_btn.clicked.connect(lambda: self.power_manager.set_power_profile('powersave'))
        
        tlp_btn = QPushButton("TLP Stats")
        tlp_btn.setObjectName("effect_start_button")
        tlp_btn.setMinimumHeight(25)
        tlp_btn.clicked.connect(self.power_manager.show_tlp_stats)
        
        power_btn_layout.addWidget(power_perf_btn, 0, 0)
        power_btn_layout.addWidget(power_bal_btn, 0, 1)
        power_btn_layout.addWidget(power_save_btn, 1, 0)
        power_btn_layout.addWidget(tlp_btn, 1, 1)
        power_layout.addWidget(power_buttons)
        
        layout.addWidget(power_section)
        
        # Temperature Monitor Section - Compact
        temp_section = QGroupBox("🌡️ Temperature Monitor")
        temp_layout = QVBoxLayout(temp_section)
        
        # Temperature display - more compact
        self.detailed_temp_display = QTextEdit()
        self.detailed_temp_display.setObjectName("system_info")
        self.detailed_temp_display.setMinimumHeight(80)
        self.detailed_temp_display.setMaximumHeight(100)
        self.detailed_temp_display.setReadOnly(True)
        temp_layout.addWidget(self.detailed_temp_display)
        
        # Temperature monitor button - smaller
        temp_monitor_btn = QPushButton("Temp Monitor")
        temp_monitor_btn.setObjectName("effect_start_button")
        temp_monitor_btn.setMinimumHeight(30)
        temp_monitor_btn.clicked.connect(self.temp_monitor.show_temperature_monitor)
        temp_layout.addWidget(temp_monitor_btn)
        
        layout.addWidget(temp_section)
        
        # Note: Lid monitoring runs automatically in background - no UI needed
        layout.addStretch()
        
        # Start system data updates
        self.start_system_updates()
    
    def auto_start_lid_monitoring(self):
        """Auto-start lid monitoring when program starts"""
        try:
            if hasattr(self, 'lid_monitor') and self.lid_monitor:
                self.lid_monitor.start_monitoring()
                self.statusBar().showMessage("Lid monitoring auto-started")
                # Update status display if it exists
                if hasattr(self, 'lid_status_display'):
                    self.lid_status_display.setText("Lid monitoring: ACTIVE (Auto-started)")
                    self.lid_status_display.setStyleSheet("color: #4CAF50;")
        except Exception as e:
            self.statusBar().showMessage(f"Failed to auto-start lid monitoring: {e}")
            if hasattr(self, 'lid_status_display'):
                self.lid_status_display.setText(f"Lid monitoring: ERROR - {str(e)[:50]}")
                self.lid_status_display.setStyleSheet("color: #F44336;")
        
        except AttributeError:
            # Initialize lid monitor if not already done
            self.lid_monitor = LidMonitor(self.rgb_controller, self)
            self.lid_monitor.lid_status.connect(lambda msg: self.statusBar().showMessage(f"Lid: {msg}"))
            self.lid_monitor.start_monitoring()
            self.statusBar().showMessage("Lid monitoring initialized and started")
            # Update status display if it exists
            if hasattr(self, 'lid_status_display'):
                self.lid_status_display.setText("Lid monitoring: ACTIVE (Auto-started)")
                self.lid_status_display.setStyleSheet("color: #4CAF50;")
        
    def create_effects_panel_content(self, parent_widget):
        """Create effects content for tab"""
        layout = QVBoxLayout(parent_widget)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Effects list with better organization
        effects_scroll = QScrollArea()
        effects_widget = QWidget()
        effects_layout = QVBoxLayout(effects_widget)
        
        effects = [
            ("🌈 Rainbow Diagonal Wave", "rainbow_diagonal", "Diagonal wave starting from ESC"),
            ("🌊 Horizontal Wave", "rainbow_horizontal", "Wave moving across rows"),
            ("🎯 Radial Wave", "rainbow_radial", "Wave radiating from center"),
            ("💨 Breathing Effect", "breathing", "Smooth breathing animation"),
            ("⚡ Static Color", "static", "Solid color lighting"),
            ("🔥 Reactive Typing", "reactive", "Keys light up on activity"),
            ("🎮 Gaming Mode", "gaming", "WASD and arrow key highlights"),
            ("🌟 Starfield", "starfield", "Random twinkling effect")
        ]
        
        for name, effect_id, description in effects:
            effect_card = self.create_improved_effect_card(name, effect_id, description)
            effects_layout.addWidget(effect_card)
        
        effects_scroll.setWidget(effects_widget)
        effects_scroll.setWidgetResizable(True)
        layout.addWidget(effects_scroll)
        
        # Control buttons with better spacing
        control_widget = QWidget()
        control_layout = QHBoxLayout(control_widget)
        
        stop_btn = QPushButton("Stop Effects")
        stop_btn.setObjectName("stop_button")
        stop_btn.setMinimumHeight(45)
        stop_btn.clicked.connect(self.stop_current_effect)
        
        control_layout.addWidget(stop_btn)
        layout.addWidget(control_widget)
        
    def create_key_bindings_panel(self, parent_widget):
        """Create organized key groups panel (no individual keys)"""
        layout = QVBoxLayout(parent_widget)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Header
        header = QLabel("Key Group Controls")
        header.setObjectName("section_header")
        header.setAlignment(Qt.AlignCenter)
        layout.addWidget(header)
        
        # Description
        desc = QLabel("Select a key group to apply the current color:")
        desc.setObjectName("temp_label")
        desc.setWordWrap(True)
        layout.addWidget(desc)
        
        # Key groups with better organization and scroll area
        groups_section = QGroupBox("Available Key Groups")
        groups_scroll = QScrollArea()
        groups_widget = QWidget()
        groups_layout = QGridLayout(groups_widget)
        
        # Create group buttons in a grid for better space usage
        groups = [
            ("All Keys", "all_keys", "Apply color to entire keyboard"),
            ("Function Keys (F1-F12)", "function_keys", "F1 through F12 keys"),
            ("Number Row (1-0)", "number_row", "Number keys 1 through 0"),
            ("QWERTY Row", "qwerty_row", "Q, W, E, R, T, Y, U, I, O, P keys"),
            ("ASDF Row", "asdf_row", "A, S, D, F, G, H, J, K, L keys"),
            ("ZXCV Row", "zxcv_row", "Z, X, C, V, B, N, M keys"),
            ("WASD Keys", "wasd", "Gaming WASD keys"),
            ("Arrow Keys", "arrow_keys", "Up, Down, Left, Right arrows"),
            ("Number Pad", "keypad", "Numeric keypad including operators"),
            ("Spacebar", "spacebar_full", "Complete spacebar area"),
            ("Letter Keys Only", "letters", "All alphabet letters A-Z"),
            ("Navigation Keys", "navigation", "Insert, Home, Page Up/Down, Delete, End")
        ]
        
        for i, (group_name, group_key, description) in enumerate(groups):
            row = i // 2
            col = i % 2
            
            # Create a mini-card for each group
            group_card = QWidget()
            group_card.setObjectName("effect_card")
            card_layout = QVBoxLayout(group_card)
            card_layout.setContentsMargins(10, 10, 10, 10)
            
            # Group name
            name_label = QLabel(group_name)
            name_label.setObjectName("effect_name")
            name_label.setWordWrap(True)
            card_layout.addWidget(name_label)
            
            # Description
            desc_label = QLabel(description)
            desc_label.setObjectName("effect_description")
            desc_label.setWordWrap(True)
            card_layout.addWidget(desc_label)
            
            # Color picker and apply buttons
            btn_layout = QHBoxLayout()
            
            # Color picker button
            color_btn = QPushButton("Pick Color")
            color_btn.setObjectName("group_button")
            color_btn.setMinimumHeight(30)
            color_btn.clicked.connect(lambda checked, gk=group_key: self.pick_color_for_group(gk))
            btn_layout.addWidget(color_btn)
            
            # Apply current color button
            apply_btn = QPushButton("Apply Current")
            apply_btn.setObjectName("effect_start_button")
            apply_btn.setMinimumHeight(30)
            apply_btn.clicked.connect(lambda checked, gk=group_key: self.set_group_color(gk))
            btn_layout.addWidget(apply_btn)
            
            card_layout.addLayout(btn_layout)
            
            groups_layout.addWidget(group_card, row, col)
        
        # Set up scroll area
        groups_scroll.setWidget(groups_widget)
        groups_scroll.setWidgetResizable(True)
        groups_scroll.setMinimumHeight(400)
        
        # Add scroll area to section
        groups_section_layout = QVBoxLayout(groups_section)
        groups_section_layout.addWidget(groups_scroll)
        
        layout.addWidget(groups_section)
        layout.addStretch()
        
    def create_improved_effect_card(self, name, effect_id, description):
        """Create an improved effect card widget with better spacing"""
        card = QWidget()
        card.setObjectName("effect_card")
        card.setMinimumHeight(80)
        layout = QHBoxLayout(card)
        layout.setContentsMargins(15, 15, 15, 15)
        
        # Info section
        info_layout = QVBoxLayout()
        
        name_label = QLabel(name)
        name_label.setObjectName("effect_name")
        
        desc_label = QLabel(description)
        desc_label.setObjectName("effect_description")
        desc_label.setWordWrap(True)
        
        info_layout.addWidget(name_label)
        info_layout.addWidget(desc_label)
        
        # Start button with better sizing
        start_btn = QPushButton("Start")
        start_btn.setObjectName("effect_start_button")
        start_btn.setMinimumHeight(40)
        start_btn.setMinimumWidth(80)
        start_btn.clicked.connect(lambda: self.start_effect(effect_id))
        
        layout.addLayout(info_layout)
        layout.addStretch()
        layout.addWidget(start_btn)
        
        return card
        
    def set_individual_key(self, key_name):
        """Set color for individual key"""
        if self.rgb_connected:
            success = self.rgb_controller.set_key_color(key_name, *self.current_color)
            if success:
                self.statusBar().showMessage(f"Applied color to {key_name}")
            else:
                self.statusBar().showMessage(f"Failed to set color for {key_name}")
        else:
            self.statusBar().showMessage("RGB device not connected")
        
    def setup_system_tray_integration(self):
        """Create system integration panel with fan, power, and temperature controls"""
        panel = QWidget()
        panel.setObjectName("system_panel")
        panel.setMinimumWidth(350)
        panel.setMaximumWidth(350)
        
        layout = QVBoxLayout(panel)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Header
        header = QLabel("System Integration")
        header.setObjectName("panel_header")
        header.setAlignment(Qt.AlignCenter)
        layout.addWidget(header)
        
        # Initialize system components
        self.fan_controller = FanController()
        self.power_manager = PowerManager()
        self.temp_monitor = TemperatureMonitor()
        self.lid_monitor = LidMonitor(self.rgb_controller, self)
        
        # Connect signals
        self.fan_controller.fan_status.connect(lambda msg: self.statusBar().showMessage(f"Fan: {msg}"))
        self.power_manager.power_status.connect(lambda msg: self.statusBar().showMessage(f"Power: {msg}"))
        self.lid_monitor.lid_status.connect(lambda msg: self.statusBar().showMessage(f"Lid: {msg}"))
        
        # Fan Control Section
        fan_section = QWidget()
        fan_layout = QVBoxLayout(fan_section)
        
        fan_header = QLabel("🌀 Fan Control")
        fan_header.setObjectName("section_header")
        fan_layout.addWidget(fan_header)
        
        # Fan speed display
        self.fan_display = QLabel("Fan speeds: Checking...")
        self.fan_display.setObjectName("temp_label")
        fan_layout.addWidget(self.fan_display)
        
        # Fan control buttons
        fan_buttons = QWidget()
        fan_btn_layout = QHBoxLayout(fan_buttons)
        
        fan_auto_btn = QPushButton("Auto")
        fan_auto_btn.setObjectName("group_button")
        fan_auto_btn.clicked.connect(lambda: self.fan_controller.set_fan_mode('auto'))
        
        fan_perf_btn = QPushButton("Performance")
        fan_perf_btn.setObjectName("group_button")
        fan_perf_btn.clicked.connect(lambda: self.fan_controller.set_fan_mode('performance'))
        
        fan_silent_btn = QPushButton("Silent")
        fan_silent_btn.setObjectName("group_button")
        fan_silent_btn.clicked.connect(lambda: self.fan_controller.set_fan_mode('silent'))
        
        fan_gui_btn = QPushButton("Launch Fan GUI")
        fan_gui_btn.setObjectName("effect_start_button")
        fan_gui_btn.clicked.connect(self.fan_controller.launch_fancontrol_gui)
        
        fan_btn_layout.addWidget(fan_auto_btn)
        fan_btn_layout.addWidget(fan_perf_btn)
        fan_btn_layout.addWidget(fan_silent_btn)
        fan_layout.addWidget(fan_buttons)
        fan_layout.addWidget(fan_gui_btn)
        
        layout.addWidget(fan_section)
        
        # Power Management Section
        power_section = QWidget()
        power_layout = QVBoxLayout(power_section)
        
        power_header = QLabel("⚡ Power Management")
        power_header.setObjectName("section_header")
        power_layout.addWidget(power_header)
        
        # Power info display
        self.power_display = QLabel("Power info: Checking...")
        self.power_display.setObjectName("temp_label")
        power_layout.addWidget(self.power_display)
        
        # Power profile buttons
        power_buttons = QWidget()
        power_btn_layout = QHBoxLayout(power_buttons)
        
        power_perf_btn = QPushButton("Performance")
        power_perf_btn.setObjectName("group_button")
        power_perf_btn.clicked.connect(lambda: self.power_manager.set_power_profile('performance'))
        
        power_balanced_btn = QPushButton("Balanced")
        power_balanced_btn.setObjectName("group_button")
        power_balanced_btn.clicked.connect(lambda: self.power_manager.set_power_profile('balanced'))
        
        power_save_btn = QPushButton("Power Save")
        power_save_btn.setObjectName("group_button")
        power_save_btn.clicked.connect(lambda: self.power_manager.set_power_profile('powersave'))
        
        power_btn_layout.addWidget(power_perf_btn)
        power_btn_layout.addWidget(power_balanced_btn)
        power_btn_layout.addWidget(power_save_btn)
        power_layout.addWidget(power_buttons)
        
        # TLP Stats button
        tlp_btn = QPushButton("Show TLP Stats")
        tlp_btn.setObjectName("effect_start_button")
        tlp_btn.clicked.connect(self.power_manager.show_tlp_stats)
        power_layout.addWidget(tlp_btn)
        
        layout.addWidget(power_section)
        
        # Temperature Monitor Section
        temp_section = QWidget()
        temp_layout = QVBoxLayout(temp_section)
        
        temp_header = QLabel("🌡️ Temperature Monitor")
        temp_header.setObjectName("section_header")
        temp_layout.addWidget(temp_header)
        
        # Temperature display
        self.detailed_temp_display = QTextEdit()
        self.detailed_temp_display.setObjectName("system_info")
        self.detailed_temp_display.setMaximumHeight(120)
        self.detailed_temp_display.setReadOnly(True)
        temp_layout.addWidget(self.detailed_temp_display)
        
        # Temperature monitor button
        temp_monitor_btn = QPushButton("Launch Temperature Monitor")
        temp_monitor_btn.setObjectName("effect_start_button")
        temp_monitor_btn.clicked.connect(self.temp_monitor.show_temperature_monitor)
        temp_layout.addWidget(temp_monitor_btn)
        
        layout.addWidget(temp_section)
        
        # Lid Monitor Section
        lid_section = QWidget()
        lid_layout = QVBoxLayout(lid_section)
        
        lid_header = QLabel("💻 Lid Monitor (Enhanced)")
        lid_header.setObjectName("section_header")
        lid_layout.addWidget(lid_header)
        
        # Lid status display
        self.lid_status_display = QLabel("Lid monitoring: Not started")
        self.lid_status_display.setObjectName("temp_label")
        lid_layout.addWidget(self.lid_status_display)
        
        # Lid monitor controls
        lid_controls = QWidget()
        lid_ctrl_layout = QHBoxLayout(lid_controls)
        
        lid_start_btn = QPushButton("Start Monitor")
        lid_start_btn.setObjectName("group_button")
        lid_start_btn.clicked.connect(self.start_lid_monitoring)
        
        lid_test_btn = QPushButton("Test Clear")
        lid_test_btn.setObjectName("group_button")
        lid_test_btn.clicked.connect(self.test_lid_clear)
        
        lid_ctrl_layout.addWidget(lid_start_btn)
        lid_ctrl_layout.addWidget(lid_test_btn)
        lid_layout.addWidget(lid_controls)
        
        layout.addWidget(lid_section)
        
        layout.addStretch()
        parent_layout.addWidget(panel)
        return panel
        
        # Start system data updates
        self.start_system_updates()
        
    def create_effects_panel_deprecated(self, parent_layout):
        """Create effects control panel"""
        panel = QWidget()
        panel.setObjectName("effects_panel")
        panel.setMinimumWidth(350)
        
        layout = QVBoxLayout(panel)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Header
        header = QLabel("Lighting Effects")
        header.setObjectName("panel_header")
        header.setAlignment(Qt.AlignCenter)
        layout.addWidget(header)
        
        # Effects list
        effects_scroll = QScrollArea()
        effects_widget = QWidget()
        effects_layout = QVBoxLayout(effects_widget)
        
        effects = [
            ("🌈 Rainbow Diagonal Wave", "rainbow_diagonal", "Diagonal wave starting from ESC"),
            ("🌊 Horizontal Wave", "rainbow_horizontal", "Wave moving across rows"),
            ("🎯 Radial Wave", "rainbow_radial", "Wave radiating from center"),
            ("💨 Breathing Effect", "breathing", "Smooth breathing animation"),
            ("⚡ Static Color", "static", "Solid color lighting"),
            ("🔥 Reactive Typing", "reactive", "Keys light up on activity"),
            ("🎮 Gaming Mode", "gaming", "WASD and arrow key highlights"),
            ("🌟 Starfield", "starfield", "Random twinkling effect")
        ]
        
        for name, effect_id, description in effects:
            effect_card = self.create_effect_card(name, effect_id, description)
            effects_layout.addWidget(effect_card)
        
        effects_scroll.setWidget(effects_widget)
        effects_scroll.setWidgetResizable(True)
        layout.addWidget(effects_scroll)
        
        # Control buttons
        control_widget = QWidget()
        control_layout = QHBoxLayout(control_widget)
        
        stop_btn = QPushButton("Stop Effects")
        stop_btn.setObjectName("stop_button")
        stop_btn.clicked.connect(self.stop_current_effect)
        
        control_layout.addWidget(stop_btn)
        layout.addWidget(control_widget)
        
        parent_layout.addWidget(panel)
        
    def create_effect_card(self, name, effect_id, description):
        """Create an effect card widget"""
        card = QWidget()
        card.setObjectName("effect_card")
        layout = QHBoxLayout(card)
        layout.setContentsMargins(15, 15, 15, 15)
        
        # Info section
        info_layout = QVBoxLayout()
        
        name_label = QLabel(name)
        name_label.setObjectName("effect_name")
        
        desc_label = QLabel(description)
        desc_label.setObjectName("effect_description")
        desc_label.setWordWrap(True)
        
        info_layout.addWidget(name_label)
        info_layout.addWidget(desc_label)
        
        # Start button
        start_btn = QPushButton("Start")
        start_btn.setObjectName("effect_start_button")
        start_btn.clicked.connect(lambda: self.start_effect(effect_id))
        
        layout.addLayout(info_layout)
        layout.addStretch()
        layout.addWidget(start_btn)
        
        return card
        
    def setup_system_tray_actual(self):
        """Setup system tray functionality (Nyx-inspired) with robust handling"""
        if QSystemTrayIcon.isSystemTrayAvailable():
            try:
                self.tray_icon = QSystemTrayIcon(self)
                
                # Set icon
                icon = self.style().standardIcon(QStyle.SP_ComputerIcon)
                self.tray_icon.setIcon(icon)
                
                # Create tray menu with reconnection
                tray_menu = QMenu()
                
                show_action = tray_menu.addAction("Show Control Center")
                show_action.triggered.connect(self.show_window)
                
                tray_menu.addSeparator()
                
                quick_clear = tray_menu.addAction("Quick Clear All")
                quick_clear.triggered.connect(self.clear_all_keys)
                
                tray_menu.addSeparator()
                
                exit_action = tray_menu.addAction("Exit")
                exit_action.triggered.connect(self.quit_application)
                
                self.tray_icon.setContextMenu(tray_menu)
                self.tray_icon.activated.connect(self.show_window)
                
                # Show tray icon safely
                self.tray_icon.show()
                self.tray_icon.showMessage(
                    "Enhanced Control Center",
                    "Running in system tray",
                    QSystemTrayIcon.Information,
                    3000
                )
            except Exception as e:
                print(f"⚠️  Tray setup error: {e}")
                # Retry logic for tray setup
                self.tray_icon.deleteLater()
                QTimer.singleShot(1000, self.setup_system_tray)
            
    def show_window(self):
        """Show main window and bring to foreground with forced activation"""
        self.setWindowState(self.windowState() & ~Qt.WindowMinimized)
        self.showNormal()
        self.raise_()
        self.activateWindow()
        print("🖥️ Control Center shown")
    
    def tray_activated(self, reason):
        """Handle tray icon activation robustly with specific checks"""
        if reason in (QSystemTrayIcon.Trigger, QSystemTrayIcon.DoubleClick, QSystemTrayIcon.Context):
            self.show_window()
    
    def start_monitoring(self):
        """Start system monitoring"""
        if not self.monitoring_active:
            self.system_updater.start()
            self.monitoring_active = True
            
    def check_rgb_connection(self):
        """Check RGB device connection"""
        success, message = self.rgb_controller.check_permissions()
        self.rgb_connected = success
        
        if hasattr(self, 'connection_label'):
            if success:
                self.connection_label.setText("✅ RGB Device Connected")
                self.connection_label.setStyleSheet("color: #4CAF50;")
            else:
                self.connection_label.setText(f"❌ {message}")
                self.connection_label.setStyleSheet("color: #F44336;")
    
    def update_system_data(self, data):
        """Update system monitoring data"""
        self.system_data = data
        
        # Update progress bars
        if hasattr(self, 'cpu_progress'):
            self.cpu_progress.set_value(int(data.get('cpu_percent', 0)))
            
        if hasattr(self, 'memory_progress'):
            memory = data.get('memory')
            if memory:
                self.memory_progress.set_value(int(memory.percent))
        
        # Update temperature labels
        if hasattr(self, 'cpu_temp_label'):
            cpu_temp = data.get('cpu_temp', 0)
            self.cpu_temp_label.setText(f"CPU: {cpu_temp:.1f}°C")
            
            # Color-code temperature
            if cpu_temp > 80:
                self.cpu_temp_label.setStyleSheet("color: #F44336;")  # Red
            elif cpu_temp > 65:
                self.cpu_temp_label.setStyleSheet("color: #FF9800;")  # Orange
            else:
                self.cpu_temp_label.setStyleSheet("color: #4CAF50;")  # Green
        
        if hasattr(self, 'gpu_temp_label'):
            gpu_temp = data.get('gpu_temp', 0)
            self.gpu_temp_label.setText(f"GPU: {gpu_temp:.1f}°C")
        
        # Update system info
        if hasattr(self, 'system_info_text'):
            memory = data.get('memory')
            disk = data.get('disk')
            load_avg = data.get('load_avg', (0, 0, 0))
            
            info_text = f"""Memory: {memory.percent:.1f}% ({memory.used / 1024**3:.1f}GB / {memory.total / 1024**3:.1f}GB)
Disk: {disk.percent:.1f}% ({disk.used / 1024**3:.1f}GB / {disk.total / 1024**3:.1f}GB)
Load Average: {load_avg[0]:.2f}, {load_avg[1]:.2f}, {load_avg[2]:.2f}
Uptime: {self.format_uptime(data.get('boot_time', 0))}"""
            
            self.system_info_text.setPlainText(info_text)
    
    def format_uptime(self, boot_time):
        """Format system uptime"""
        if boot_time:
            uptime_seconds = time.time() - boot_time
            days = int(uptime_seconds // 86400)
            hours = int((uptime_seconds % 86400) // 3600)
            minutes = int((uptime_seconds % 3600) // 60)
            return f"{days}d {hours}h {minutes}m"
        return "Unknown"
    
    def choose_color(self):
        """Open color picker dialog with immediate preview"""
        color = QColorDialog.getColor(QColor(*self.current_color), self)
        if color.isValid():
            new_color = (color.red(), color.green(), color.blue())
            self.set_current_color_immediate(new_color)
    
    def set_group_color(self, group_key):
        """Set color for a key group"""
        if self.rgb_connected and group_key in self.rgb_controller.key_groups:
            success = self.rgb_controller.set_group_color(group_key, *self.current_color)
            if success:
                self.statusBar().showMessage(f"Applied color to {group_key}")
            else:
                self.statusBar().showMessage("Failed to apply color")
        else:
            self.statusBar().showMessage("RGB device not connected")
    
    def clear_all_keys(self):
        """Clear all keyboard lighting - enhanced daemon clearing"""
        if self.rgb_connected:
            # Stop any running effects first
            self.stop_current_effect()
            
            # Clear through controller
            success = self.rgb_controller.clear_all_keys()
            
            # Also clear daemon if running
            try:
                subprocess.run(['sudo', 'pkill', '-f', 'rgb.*daemon'], check=False, timeout=3)
                subprocess.run(['sudo', 'systemctl', 'stop', 'rgb-daemon'], check=False, timeout=3)
                time.sleep(0.5)
            except Exception:
                pass
            
            # Force clear all possible key mappings
            try:
                for device_path in ['/dev/hidraw0', '/dev/hidraw1', '/dev/hidraw2']:
                    if os.path.exists(device_path):
                        self.rgb_controller.force_clear_device(device_path)
            except Exception:
                pass
                
            if success:
                self.statusBar().showMessage("All keys cleared (including daemon)")
            else:
                self.statusBar().showMessage("Failed to clear some keys")
        else:
            self.statusBar().showMessage("RGB device not connected")
    
    def start_system_updates(self):
        """Start updating system data for the system panel"""
        self.system_update_timer = QTimer()
        self.system_update_timer.timeout.connect(self.update_system_panel_data)
        self.system_update_timer.start(2000)  # Update every 2 seconds
        
        # Initial update
        self.update_system_panel_data()
    
    def update_system_panel_data(self):
        """Update system panel displays with current data"""
        try:
            # Update fan speeds
            if hasattr(self, 'fan_controller') and hasattr(self, 'fan_display'):
                fans = self.fan_controller.get_fan_speeds()
                if fans:
                    fan_text = "\n".join([f"{fan['name']}: {fan['rpm']} RPM" for fan in fans[:3]])
                    self.fan_display.setText(fan_text)
                else:
                    self.fan_display.setText("No fans detected")
            
            # Update power information
            if hasattr(self, 'power_manager') and hasattr(self, 'power_display'):
                power_info = self.power_manager.get_power_info()
                power_text = f"""Battery: {power_info['battery_percent']:.0f}%
{"AC Connected" if power_info['ac_connected'] else "On Battery"}
Profile: {power_info['power_profile']}
TLP: {power_info['tlp_status']}"""
                self.power_display.setText(power_text)
            
            # Update detailed temperature display
            if hasattr(self, 'temp_monitor') and hasattr(self, 'detailed_temp_display'):
                temp_data = self.temp_monitor.get_all_temperatures()
                temp_text = ""
                
                if temp_data['cpu_cores']:
                    temp_text += "CPU:\n"
                    for name, temp in temp_data['cpu_cores'][:4]:  # Show first 4
                        temp_text += f"  {name}: {temp:.1f}°C\n"
                
                if temp_data['storage']:
                    temp_text += "Storage:\n"
                    for name, temp in temp_data['storage'][:2]:  # Show first 2
                        temp_text += f"  {name}: {temp:.1f}°C\n"
                
                if temp_data['gpu']:
                    temp_text += "GPU:\n"
                    for name, temp in temp_data['gpu'][:2]:  # Show first 2
                        temp_text += f"  {name}: {temp:.1f}°C\n"
                
                if not temp_text:
                    temp_text = "No temperature sensors found"
                
                self.detailed_temp_display.setPlainText(temp_text.strip())
            
        except Exception as e:
            print(f"System panel update error: {e}")
    
    def start_lid_monitoring(self):
        """Start enhanced lid monitoring"""
        if hasattr(self, 'lid_monitor'):
            self.lid_monitor.start_monitoring()
            if hasattr(self, 'lid_status_display'):
                self.lid_status_display.setText("Lid monitoring: ACTIVE (Enhanced)")
                self.lid_status_display.setStyleSheet("color: #4CAF50;")
    
    def test_lid_clear(self):
        """Test the enhanced lid clearing functionality"""
        if hasattr(self, 'lid_monitor'):
            self.lid_monitor._aggressive_keyboard_clear()
            self.statusBar().showMessage("Test clear executed")
    
    def start_effect(self, effect_id):
        """Start a lighting effect with persistence"""
        if not self.rgb_connected:
            self.statusBar().showMessage("RGB device not connected")
            return
        
        # Stop current effect
        self.stop_current_effect()
        
        # Save effect state for persistence
        self.save_effect_state(effect_id)
        
        # Start new effect
        self.effect_stop_event.clear()
        
        if effect_id == "rainbow_diagonal":
            self.current_effect_thread = threading.Thread(
                target=self.persistent_effect_wrapper,
                args=(self.rgb_controller.advanced_wave_effect, (30, 'diagonal', self.effect_stop_event)),
                daemon=True
            )
        elif effect_id == "rainbow_horizontal":
            self.current_effect_thread = threading.Thread(
                target=self.persistent_effect_wrapper,
                args=(self.rgb_controller.advanced_wave_effect, (30, 'horizontal', self.effect_stop_event)),
                daemon=True
            )
        elif effect_id == "rainbow_radial":
            self.current_effect_thread = threading.Thread(
                target=self.persistent_effect_wrapper,
                args=(self.rgb_controller.advanced_wave_effect, (30, 'radial', self.effect_stop_event)),
                daemon=True
            )
        elif effect_id == "static":
            self.rgb_controller.set_group_color("all_keys", *self.current_color)
            self.statusBar().showMessage("Static effect applied (persistent)")
            return
        elif effect_id == "breathing":
            self.current_effect_thread = threading.Thread(
                target=self.persistent_effect_wrapper,
                args=(self.breathing_effect, ()),
                daemon=True
            )
        elif effect_id == "gaming":
            self.gaming_mode_effect()
            self.statusBar().showMessage("Gaming mode activated (persistent)")
            return
        
        if self.current_effect_thread:
            self.current_effect_thread.start()
            self.statusBar().showMessage(f"Started {effect_id} effect (persistent)")
    
    def stop_current_effect(self):
        """Stop the current lighting effect and clear all keys"""
        # Stop persistent effects flag
        self.persistent_effects_running = False
        
        # Set stop event
        self.effect_stop_event.set()
        
        # Force join thread
        if self.current_effect_thread and self.current_effect_thread.is_alive():
            self.current_effect_thread.join(timeout=1.0)
        self.current_effect_thread = None
        
        # Automatically clear all keys when stopping effect
        if self.rgb_connected:
            self.rgb_controller.clear_all_keys()
            self.statusBar().showMessage("Effect stopped - Keys cleared")
        else:
            self.statusBar().showMessage("Effect stopped")
    
    def pick_color_for_group(self, group_key):
        """Pick a specific color for a key group"""
        color = QColorDialog.getColor(QColor(*self.current_color), self)
        if color.isValid():
            new_color = (color.red(), color.green(), color.blue())
            if self.rgb_connected and group_key in self.rgb_controller.key_groups:
                success = self.rgb_controller.set_group_color(group_key, *new_color)
                if success:
                    self.statusBar().showMessage(f"Applied picked color to {group_key}")
                else:
                    self.statusBar().showMessage("Failed to apply color")
            else:
                self.statusBar().showMessage("RGB device not connected")
    
    def set_current_color(self, color):
        """Set the current color from preset"""
        self.current_color = color
        if hasattr(self, 'color_display'):
            self.color_display.setStyleSheet(
                f"background-color: rgb{self.current_color}; border: 2px solid #444;"
            )
        self.statusBar().showMessage(f"Color set to {color}")
    
    def set_current_color_immediate(self, color):
        """Set color and immediately apply to all keys"""
        self.current_color = color
        if hasattr(self, 'color_display'):
            self.color_display.setStyleSheet(
                f"background-color: rgb{self.current_color}; border: 2px solid #444;"
            )
        
        # Immediately apply to all keys if connected
        if self.rgb_connected:
            # Stop any running effects first
            self.stop_current_effect()
            # Apply color to all keys
            self.rgb_controller.set_group_color("all_keys", *self.current_color)
            self.statusBar().showMessage(f"Color {color} applied to all keys")
        else:
            self.statusBar().showMessage(f"Color set to {color} (device not connected)")
    
    def save_effect_state(self, effect_id):
        """Save current effect state for persistence across restarts"""
        try:
            effect_state = {
                'effect_id': effect_id,
                'color': self.current_color,
                'timestamp': time.time()
            }
            with open(EFFECT_STATE_FILE, 'w') as f:
                json.dump(effect_state, f)
        except Exception as e:
            print(f"Failed to save effect state: {e}")
    
    def load_effect_state(self):
        """Load previous effect state (color only, no auto-start)"""
        try:
            if EFFECT_STATE_FILE.exists():
                with open(EFFECT_STATE_FILE, 'r') as f:
                    effect_state = json.load(f)
                
                # Check if state is recent (within 24 hours)
                if time.time() - effect_state.get('timestamp', 0) < 86400:
                    # Only restore color, not the effect itself
                    self.current_color = tuple(effect_state.get('color', (255, 102, 0)))
                    
                    # Update color display if it exists
                    if hasattr(self, 'color_display'):
                        self.color_display.setStyleSheet(
                            f"background-color: rgb{self.current_color}; border: 2px solid #444;"
                        )
                    
                    effect_id = effect_state.get('effect_id')
                    if effect_id:
                        self.statusBar().showMessage(f"Previous session: {effect_id} effect settings loaded (not auto-started)")
                    else:
                        self.statusBar().showMessage("Previous color settings restored")
        except Exception as e:
            print(f"Failed to load effect state: {e}")
    
    def persistent_effect_wrapper(self, effect_func, args):
        """Wrapper to make effects persistent and restart if interrupted"""
        self.persistent_effects_running = True
        while not self.effect_stop_event.is_set() and getattr(self, 'persistent_effects_running', True):
            try:
                if args:
                    effect_func(*args)
                else:
                    effect_func()
                
                # If effect completed normally, restart it for persistence
                if not self.effect_stop_event.is_set() and getattr(self, 'persistent_effects_running', True):
                    time.sleep(1)  # Brief pause before restart
            except Exception as e:
                print(f"Effect error, restarting: {e}")
                time.sleep(2)  # Wait before retry
        
        # Ensure effects are fully stopped
        self.persistent_effects_running = False
    
    def breathing_effect(self):
        """Breathing effect implementation"""
        while not self.effect_stop_event.is_set():
            for brightness in range(0, 101, 5):
                if self.effect_stop_event.is_set():
                    break
                
                # Calculate color with breathing brightness
                r = int(self.current_color[0] * brightness / 100)
                g = int(self.current_color[1] * brightness / 100)
                b = int(self.current_color[2] * brightness / 100)
                
                self.rgb_controller.set_group_color("all_keys", r, g, b)
                time.sleep(0.05)
            
            for brightness in range(100, -1, -5):
                if self.effect_stop_event.is_set():
                    break
                
                r = int(self.current_color[0] * brightness / 100)
                g = int(self.current_color[1] * brightness / 100)
                b = int(self.current_color[2] * brightness / 100)
                
                self.rgb_controller.set_group_color("all_keys", r, g, b)
                time.sleep(0.05)
    
    def gaming_mode_effect(self):
        """Gaming mode with WASD and arrow highlights"""
        # Clear all first
        self.rgb_controller.clear_all_keys()
        
        # Highlight WASD keys
        wasd_keys = ['w', 'a', 's', 'd']
        for key in wasd_keys:
            self.rgb_controller.set_key_color(key, 0, 255, 0)  # Green
        
        # Highlight arrow keys
        for key in self.rgb_controller.key_groups['arrow_keys']:
            self.rgb_controller.set_key_color(key, 255, 0, 0)  # Red
        
        # Highlight spacebar
        for key in self.rgb_controller.key_groups['spacebar_full']:
            self.rgb_controller.set_key_color(key, 0, 0, 255)  # Blue
    
    def show_window(self):
        """Show main window"""
        self.showNormal()
        self.activateWindow()
        self.raise_()
    
    def tray_activated(self, reason):
        """Handle tray icon activation"""
        if reason == QSystemTrayIcon.DoubleClick:
            self.show_window()
    
    def minimize_to_tray(self):
        """Minimize window to system tray"""
        if self.tray_icon and self.tray_icon.isVisible():
            self.hide()
            self.tray_icon.showMessage(
                "Enhanced Control Center",
                "Application minimized to tray",
                QSystemTrayIcon.Information,
                2000
            )
        else:
            self.showMinimized()
    
    def quit_application(self):
        """Quit the application with comprehensive cleanup"""
        print("🛑 Shutting down Enhanced Control Center...")
        
        try:
            # STEP 1: Hide and cleanup tray icon IMMEDIATELY
            if hasattr(self, 'tray_icon') and self.tray_icon:
                self.tray_icon.hide()
                self.tray_icon.setVisible(False)
                self.tray_icon.deleteLater()
                print("✅ Tray icon hidden and deleted")
            
            # STEP 2: Stop all effects and monitoring
            self.stop_current_effect()
            if hasattr(self, 'monitoring_active') and self.monitoring_active:
                self.monitoring_active = False
                if hasattr(self, 'system_updater'):
                    self.system_updater.stop()
            
            # STEP 3: Stop lid monitoring
            if hasattr(self, 'lid_monitor') and self.lid_monitor:
                self.lid_monitor.monitoring = False
            
            # STEP 4: Clear RGB completely
            if hasattr(self, 'rgb_controller'):
                for _ in range(2):
                    self.rgb_controller.clear_all_keys()
            
            print("✅ Core cleanup completed")
            
        except Exception as e:
            print(f"⚠️ Cleanup error: {e}")
        
        # STEP 5: Kill other instances
        try:
            self.kill_all_instances()
        except:
            pass
        
        print("🚪 Force exiting...")
        
        # STEP 6: Exit application with force
        QApplication.quit()
        
        # Force exit with delay if needed
        import threading
        def force_exit():
            time.sleep(0.5)
            print("💥 FORCE EXIT")
            os._exit(0)
        
        threading.Thread(target=force_exit, daemon=True).start()
    
    def kill_all_instances(self):
        """Kill all instances of this program to prevent orphaned processes"""
        try:
            import psutil
            current_pid = os.getpid()
            program_name = "enhanced-professional-control-center"
            
            print(f"🔍 Searching for instances of {program_name}...")
            
            killed_count = 0
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    # Check if this is our program
                    if proc.info['cmdline']:
                        cmdline_str = ' '.join(proc.info['cmdline'])
                        if program_name in cmdline_str and 'python' in cmdline_str.lower():
                            pid = proc.info['pid']
                            
                            # Don't kill ourselves (we'll exit normally)
                            if pid != current_pid:
                                print(f"🎯 Killing orphaned instance: PID {pid}")
                                try:
                                    proc.terminate()
                                    proc.wait(timeout=3)
                                    killed_count += 1
                                except (psutil.NoSuchProcess, psutil.TimeoutExpired):
                                    # Force kill if terminate doesn't work
                                    try:
                                        proc.kill()
                                        killed_count += 1
                                    except psutil.NoSuchProcess:
                                        pass
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            
            if killed_count > 0:
                print(f"✅ Killed {killed_count} orphaned instance(s)")
            else:
                print("✅ No orphaned instances found")
                
        except Exception as e:
            print(f"⚠️  Error during cleanup: {e}")
            # Fallback: try pkill command
            try:
                subprocess.run(['pkill', '-f', 'enhanced-professional-control-center'], 
                             check=False, timeout=5)
                print("✅ Fallback cleanup completed")
            except Exception:
                pass
    
    def closeEvent(self, event):
        """Handle close event - minimize to tray or quit properly"""
        if self.tray_icon and self.tray_icon.isVisible():
            self.hide()
            event.ignore()
        else:
            # Proper cleanup on close
            self.quit_application()
            event.accept()
    
    def get_professional_stylesheet(self):
        """Professional dark theme stylesheet"""
        return """
        /* Main Application */
        QMainWindow {
            background-color: #1e1e1e;
            color: #ffffff;
        }
        
        /* Panels */
        QWidget#monitoring_panel, QWidget#control_panel, QWidget#system_panel, QWidget#effects_panel {
            background-color: #2d2d2d;
            border-right: 1px solid #404040;
        }
        
        /* Headers */
        QLabel#panel_header {
            font-size: 18px;
            font-weight: bold;
            color: #ffffff;
            padding: 10px;
            background-color: #3d3d3d;
            border-radius: 5px;
            margin-bottom: 15px;
        }
        
        QLabel#section_header {
            font-size: 14px;
            font-weight: bold;
            color: #cccccc;
            padding: 8px;
            background-color: #404040;
            border-radius: 3px;
            margin: 5px 0;
        }
        
        /* Labels */
        QLabel#monitor_label {
            color: #cccccc;
            font-size: 12px;
            padding: 5px;
        }
        
        QLabel#temp_label {
            color: #ffffff;
            font-size: 13px;
            padding: 3px;
            background-color: #353535;
            border-radius: 3px;
            margin: 2px;
        }
        
        QLabel#connection_status {
            font-size: 12px;
            padding: 8px;
            background-color: #404040;
            border-radius: 5px;
            margin: 5px 0;
        }
        
        /* Buttons */
        QPushButton#group_button, QPushButton#color_button, QPushButton#clear_button {
            background-color: #4a90e2;
            color: white;
            border: none;
            padding: 10px 15px;
            border-radius: 5px;
            font-size: 12px;
            margin: 2px;
        }
        
        QPushButton#group_button:hover, QPushButton#color_button:hover, QPushButton#clear_button:hover {
            background-color: #357abd;
        }
        
        QPushButton#group_button:pressed, QPushButton#color_button:pressed, QPushButton#clear_button:pressed {
            background-color: #2968a3;
        }
        
        QPushButton#effect_start_button {
            background-color: #5cb85c;
            color: white;
            border: none;
            padding: 8px 15px;
            border-radius: 4px;
            font-size: 11px;
        }
        
        QPushButton#effect_start_button:hover {
            background-color: #449d44;
        }
        
        QPushButton#stop_button {
            background-color: #d9534f;
            color: white;
            border: none;
            padding: 10px 15px;
            border-radius: 5px;
            font-size: 12px;
        }
        
        QPushButton#stop_button:hover {
            background-color: #c9302c;
        }
        
        /* Effect Cards */
        QWidget#effect_card {
            background-color: #3a3a3a;
            border: 1px solid #505050;
            border-radius: 8px;
            margin: 5px;
            padding: 5px;
        }
        
        QLabel#effect_name {
            font-size: 13px;
            font-weight: bold;
            color: #ffffff;
        }
        
        QLabel#effect_description {
            font-size: 11px;
            color: #cccccc;
        }
        
        /* Color Display */
        QLabel#color_display {
            border: 2px solid #666666;
            border-radius: 5px;
        }
        /* System Info */
        QTextEdit#system_info {
            background-color: #2a2a2a;
            border: 1px solid #505050;
            border-radius: 5px;
            color: #cccccc;
            font-family: monospace;
            font-size: 11px;
            padding: 8px;
        }
        
        /* Preset Buttons */
        QPushButton#preset_button {
            border: 1px solid #666666;
            border-radius: 3px;
            padding: 5px;
            font-size: 10px;
            font-weight: bold;
        }
        
        QPushButton#preset_button:hover {
            border: 2px solid #ffffff;
        }
        
        /* Scroll Areas */
        QScrollArea {
            border: none;
            background-color: transparent;
        }
        
        QScrollBar:vertical {
            background: #404040;
            width: 12px;
            border-radius: 6px;
        }
        
        QScrollBar::handle:vertical {
            background: #606060;
            border-radius: 6px;
            min-height: 20px;
        }
        
        QScrollBar::handle:vertical:hover {
            background: #707070;
        }
        /* Status Bar */
        QStatusBar {
            background-color: #2d2d2d;
            color: #cccccc;
            border-top: 1px solid #404040;
        }
        
        /* Tray Button */
        QPushButton#tray_button {
            background-color: #f0ad4e;
            color: white;
            border: none;
            padding: 5px 10px;
            border-radius: 3px;
            font-size: 11px;
        }
        
        QPushButton#tray_button:hover {
            background-color: #ec971f;
        }
        
        /* Tab Widget */
        QTabWidget#main_tabs {
            background-color: #2d2d2d;
        }
        
        QTabWidget#main_tabs::pane {
            border: 1px solid #404040;
            background-color: #2d2d2d;
        }
        
        QTabWidget#main_tabs::tab-bar {
            alignment: left;
        }
        
        QTabBar::tab {
            background-color: #404040;
            color: #cccccc;
            border: 1px solid #606060;
            padding: 8px 16px;
            margin-right: 2px;
        }
        
        QTabBar::tab:selected {
            background-color: #4a90e2;
            color: white;
        }
        
        QTabBar::tab:hover {
            background-color: #505050;
        }
        
        /* Group Boxes */
        QGroupBox {
            font-weight: bold;
            border: 2px solid #505050;
            border-radius: 5px;
            margin-top: 10px;
            padding-top: 10px;
            color: #ffffff;
        }
        
        QGroupBox::title {
            subcontrol-origin: margin;
            left: 10px;
            padding: 0 5px 0 5px;
        }
        """

def main():
    """Main application entry point"""
    print("🚀 Enhanced OriginPC Professional Control Center v5.0")
    print("Ultimate Edition with Advanced System Monitoring")
    print("================================================")
    
    # Parse command line arguments
    import argparse
    parser = argparse.ArgumentParser(description='Enhanced OriginPC Control Center')
    parser.add_argument('--minimized', action='store_true', help='Start minimized')
    parser.add_argument('--tray', action='store_true', help='Start to system tray')
    args = parser.parse_args()
    
    # Check device permissions
    if not os.path.exists('/dev/hidraw0'):
        print("⚠️  RGB device not found at /dev/hidraw0")
        print("   The application will launch but RGB features may not work.")
    elif not os.access('/dev/hidraw0', os.R_OK | os.W_OK):
        print("⚠️  Insufficient permissions for RGB device.")
        print("   Run: sudo chmod 666 /dev/hidraw0")
    else:
        print("✅ RGB device accessible")
    
    # Create and run the application
    app = QApplication(sys.argv)
    app.setApplicationName("Enhanced OriginPC Control Center")
    app.setApplicationVersion("5.0")
    app.setQuitOnLastWindowClosed(False)  # Keep running in tray
    
    # Set application icon
    app.setWindowIcon(app.style().standardIcon(QStyle.SP_ComputerIcon))
    
    # Check for single instance
    app_id = "enhanced_originpc_control_center"
    shared_memory = QSharedMemory(app_id)
    
    if shared_memory.attach(QSharedMemory.ReadOnly):
        print("Another instance is already running!")
        sys.exit(1)
    
    if not shared_memory.create(1):
        print("Failed to create shared memory!")
        sys.exit(1)
    
    # Create main window
    window = EnhancedControlCenter()
    
    # Handle startup options
    if args.tray:
        print("🔍 Starting to system tray...")
        # Don't show window, just start in tray
        if window.tray_icon:
            window.tray_icon.showMessage(
                "Enhanced Control Center",
                "Started in system tray",
                QApplication.style().standardIcon(QStyle.SP_ComputerIcon),
                3000
            )
    elif args.minimized:
        print("🔽 Starting minimized...")
        window.showMinimized()
    else:
        window.show()
    
    print("🎨 Enhanced Control Center is now running!")
    print("Features: System Monitor | RGB Control | Effects | Tray Icon")
    if args.tray:
        print("Running in system tray - right-click tray icon to access")
    elif args.minimized:
        print("Started minimized - restore from taskbar")
    else:
        print("Minimize to system tray for background operation")
    print("Usage: python enhanced-professional-control-center.py [--minimized] [--tray]")
    
    # Handle SIGINT and SIGTERM gracefully
    def signal_handler(sig, frame):
        print(f"\n🛑 Received signal {sig} - Shutting down Enhanced Control Center...")
        window.quit_application()
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Register cleanup function for normal exit
    import atexit
    def cleanup_on_exit():
        if hasattr(window, 'kill_all_instances'):
            window.kill_all_instances()
    
    atexit.register(cleanup_on_exit)
    
    sys.exit(app.exec_())

if __name__ == "__main__":
    main()

