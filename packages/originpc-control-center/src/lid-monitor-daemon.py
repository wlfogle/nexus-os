#!/usr/bin/env python3
"""
OriginPC Lid Monitor Daemon - Systemctl Service
===============================================
Dedicated daemon for monitoring lid closure and clearing RGB keyboard
- Runs as systemctl service for reliable monitoring
- Enhanced keypad clearing (fixes kp_plus cyan issue)
- Multiple detection methods for maximum reliability
- Logging for debugging
"""

import os
import sys
import time
import subprocess
import logging
import signal
import threading
from pathlib import Path

class RGBController:
    """Simple RGB controller for daemon use"""
    
    def __init__(self, device_path="/dev/hidraw0"):
        self.device_path = device_path
        
        # Keypad indices that need special attention
        self.problem_keypad_indices = [
            0x53, 0x93, 0xB2, 0x50, 0x51, 0x52,  # Keypad area including kp_plus
            0x70, 0x71, 0x72, 0x90, 0x91, 0x92, 0xB1,
            0x30, 0x31, 0x32, 0x33  # Additional keypad area
        ]
    
    def send_key_command(self, key_index, red, green, blue):
        """Send RGB command for specific key"""
        try:
            data = bytes([0xCC, 0x01, key_index, red, green, blue] + [0x00] * 10)
            with open(self.device_path, 'wb') as device:
                device.write(data)
                device.flush()  # Ensure data is written immediately
            return True
        except Exception as e:
            logging.error(f"Failed to send key command {key_index}: {e}")
            return False
    
    def ultra_aggressive_clear(self):
        """ULTRA AGGRESSIVE keyboard clearing for lid closure"""
        logging.info("Starting ULTRA AGGRESSIVE keyboard clear")
        
        try:
            # Check device exists first
            if not os.path.exists(self.device_path):
                logging.error(f"Device {self.device_path} not found")
                # Try alternative hidraw devices if primary one isn't found
                for i in range(1, 5):
                    alt_path = f"/dev/hidraw{i}"
                    if os.path.exists(alt_path):
                        logging.info(f"Found alternative device: {alt_path}")
                        self.device_path = alt_path
                        break
                else:
                    return False
            
            # Pass 1-10: More intensive clearing with optimized batching
            for pass_num in range(1, 11):
                logging.info(f"Clear pass {pass_num}/10")
                
                # Clear entire range with device sync using batching for better performance
                try:
                    with open(self.device_path, 'wb') as device:
                        batch_size = 16  # Process keys in batches for better performance
                        for batch_start in range(0, 0xFF, batch_size):
                            batch_end = min(batch_start + batch_size, 0xFF)
                            for key_index in range(batch_start, batch_end):
                                data = bytes([0xCC, 0x01, key_index, 0, 0, 0] + [0x00] * 10)
                                device.write(data)
                            device.flush()  # Flush after each batch instead of each key
                except Exception as e:
                    logging.warning(f"Batch clearing failed: {e}")
                    # Fall back to individual key clearing if batch fails
                    for key_index in range(0x00, 0xFF):
                        success = self.send_key_command(key_index, 0, 0, 0)
                        if not success:
                            logging.warning(f"Failed to clear key {key_index:02X}")
                
                # Force device sync
                try:
                    with open(self.device_path, 'wb') as device:
                        # Send sync command
                        sync_data = bytes([0xCC, 0x00, 0x00, 0x00, 0x00, 0x00] + [0x00] * 10)
                        device.write(sync_data)
                        device.flush()
                        os.fsync(device.fileno())
                except Exception as e:
                    logging.warning(f"Device sync failed: {e}")
                
                # Extra attention to problem areas
                if pass_num >= 5:
                    for problem_idx in self.problem_keypad_indices:
                        # Clear surrounding area with wider range
                        for offset in range(-8, 9):
                            clear_idx = max(0, min(0xFF, problem_idx + offset))
                            self.send_key_command(clear_idx, 0, 0, 0)
                
                time.sleep(0.08 if pass_num < 6 else 0.15)
            
            # Final verification pass with force flush
            logging.info("Final verification pass with force flush")
            for key_index in range(0x00, 0xFF):
                self.send_key_command(key_index, 0, 0, 0)
            
            # Final device sync
            try:
                with open(self.device_path, 'wb') as device:
                    device.flush()
                    os.fsync(device.fileno())
            except Exception as e:
                logging.warning(f"Final sync failed: {e}")
            
            # MEGA-AGGRESSIVE KP_PLUS SPECIFIC CLEARING (nuclear option)
            logging.info("Executing MEGA-AGGRESSIVE kp_plus specific clearing")
            self._mega_aggressive_kp_plus_clear()
            
            logging.info("ULTRA AGGRESSIVE keyboard clear completed")
            return True
            
        except Exception as e:
            logging.error(f"Ultra aggressive clear failed: {e}")
            return False
    
    def _mega_aggressive_kp_plus_clear(self):
        """MEGA AGGRESSIVE kp_plus specific clearing - nuclear option"""
        kp_plus_indices = [
            0x53, 0x33, 0x73, 0x93, 0xB3, 0xD3, 0xF3, 0x13,  # Primary locations
            0x54, 0x34, 0x74, 0x94, 0xB4, 0xD4, 0xF4, 0x14,  # Adjacent locations
            0x52, 0x32, 0x72, 0x92, 0xB2, 0xD2, 0xF2, 0x12,  # More adjacent
            0x55, 0x35, 0x75, 0x95, 0xB5, 0xD5, 0xF5, 0x15   # Even more adjacent
        ]
        
        try:
            with open(self.device_path, 'wb') as device:
                # Phase 1: Direct kp_plus attack - 15 passes
                for pass_num in range(1, 16):
                    for idx in kp_plus_indices:
                        # Clear exact index
                        data = bytes([0xCC, 0x01, idx, 0, 0, 0] + [0x00] * 10)
                        device.write(data)
                        
                        # Clear wide surrounding area
                        for offset in range(-10, 11):
                            clear_idx = max(0, min(0xFF, idx + offset))
                            data = bytes([0xCC, 0x01, clear_idx, 0, 0, 0] + [0x00] * 10)
                            device.write(data)
                    
                    if pass_num % 5 == 0:
                        device.flush()
                        time.sleep(0.02)
                
                # Phase 2: Alternative command patterns
                alt_commands = [
                    [0xCC, 0x02],  # Alternative command prefix
                    [0xCC, 0x03],  # Another alternative
                    [0xDD, 0x01],  # Different command altogether
                ]
                
                for cmd_prefix in alt_commands:
                    for idx in kp_plus_indices:
                        for offset in range(-5, 6):
                            clear_idx = max(0, min(0xFF, idx + offset))
                            data = bytes(cmd_prefix + [clear_idx, 0, 0, 0] + [0x00] * 10)
                            device.write(data)
                
                # Phase 3: Force reset commands for kp_plus area
                reset_commands = [
                    bytes([0xCC, 0x00, 0x53, 0x00, 0x00, 0x00] + [0x00] * 10),  # Reset kp_plus
                    bytes([0xCC, 0xFF, 0x53, 0x00, 0x00, 0x00] + [0x00] * 10),  # Clear kp_plus
                ]
                
                for reset_cmd in reset_commands:
                    for _ in range(5):
                        device.write(reset_cmd)
                
                device.flush()
                os.fsync(device.fileno())
                logging.info("MEGA-AGGRESSIVE kp_plus clearing completed")
                
        except Exception as e:
            logging.warning(f"Mega-aggressive kp_plus clear failed: {e}")

class LidMonitorDaemon:
    """Dedicated lid monitoring daemon"""
    
    def __init__(self):
        self.rgb_controller = RGBController()
        self.running = False
        self.monitor_thread = None
        
        # Setup logging (user-writable XDG state dir; this daemon runs as a
        # systemd --user service, not root, since RGB access only needs the
        # hidraw udev permission, not root)
        state_dir = Path(os.environ.get('XDG_STATE_HOME', Path.home() / '.local' / 'state')) / 'originpc-control-center'
        state_dir.mkdir(parents=True, exist_ok=True)
        log_file = state_dir / 'lid-monitor.log'
        
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(log_file),
                logging.StreamHandler()
            ]
        )
        
        logging.info("Lid Monitor Daemon initialized")
    
    def start(self):
        """Start the daemon"""
        logging.info("Starting Lid Monitor Daemon")
        self.running = True
        
        # Start monitoring thread
        self.monitor_thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self.monitor_thread.start()
        
        # Setup signal handlers
        signal.signal(signal.SIGTERM, self._signal_handler)
        signal.signal(signal.SIGINT, self._signal_handler)
        
        logging.info("Daemon started successfully")
        
        # Keep main thread alive
        try:
            while self.running:
                time.sleep(1)
        except KeyboardInterrupt:
            self.stop()
    
    def stop(self):
        """Stop the daemon"""
        logging.info("Stopping Lid Monitor Daemon")
        self.running = False
        
        if self.monitor_thread and self.monitor_thread.is_alive():
            self.monitor_thread.join(timeout=5)
        
        logging.info("Daemon stopped")
    
    def _signal_handler(self, signum, frame):
        """Handle shutdown signals"""
        logging.info(f"Received signal {signum}, shutting down")
        self.stop()
        sys.exit(0)
    
    def _monitor_loop(self):
        """Main monitoring loop"""
        lid_was_open = True
        consecutive_closed = 0
        last_clear_time = 0
        
        logging.info("Lid monitoring loop started")
        
        while self.running:
            try:
                lid_open = self._check_lid_state()
                current_time = time.time()
                
                if not lid_open:
                    consecutive_closed += 1
                    if consecutive_closed >= 2 and lid_was_open:
                        # Prevent multiple clears in short time
                        if current_time - last_clear_time > 5:
                            logging.info("LID CLOSURE DETECTED - Killing RGB processes and clearing")
                            
                            # First: Kill all RGB effect processes
                            self._kill_rgb_processes()
                            
                            # Then: Clear RGB state
                            success = self.rgb_controller.ultra_aggressive_clear()
                            if success:
                                logging.info("RGB processes killed and keyboard cleared successfully")
                                last_clear_time = current_time
                            else:
                                logging.error("Keyboard clear failed")
                        lid_was_open = False
                else:
                    consecutive_closed = 0
                    if not lid_was_open:
                        logging.info("Lid reopened")
                        lid_was_open = True
                
                time.sleep(1)  # Check every second
                
            except Exception as e:
                logging.error(f"Monitor loop error: {e}")
                time.sleep(5)
    
    def _check_lid_state(self):
        """Check lid state using multiple reliable methods"""
        
        # Method 1: Check if display is off (most reliable for lid closure)
        try:
            result = subprocess.run(['xset', 'q'], capture_output=True, text=True, timeout=3)
            if result.returncode == 0:
                output = result.stdout.lower()
                if 'monitor is off' in output or 'monitor is in standby' in output:
                    logging.debug("Display detected as OFF - lid likely closed")
                    return False
        except Exception:
            pass
        
        # Method 2: Check session lock status
        try:
            result = subprocess.run(['loginctl', 'show-session', 'self'], 
                                   capture_output=True, text=True, timeout=3)
            if result.returncode == 0 and 'LockedHint=yes' in result.stdout:
                logging.debug("Session locked - lid likely closed")
                return False
        except Exception:
            pass
        
        # Method 3: ACPI lid button state
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
                            logging.debug(f"ACPI lid state: {content}")
                            return False
                except Exception:
                    pass
        
        # Method 4: Check for suspend preparation
        try:
            result = subprocess.run(['systemctl', 'is-active', 'suspend.target'], 
                                   capture_output=True, text=True, timeout=2)
            if result.returncode == 0 and 'active' in result.stdout:
                logging.debug("System is suspending - lid closed")
                return False
        except Exception:
            pass
        
        # Method 5: Manual test file
        test_file = Path('/tmp/test_lid_closed')
        if test_file.exists():
            logging.info("Manual test lid closure detected")
            return False
        
        return True  # Default to open
    
    def _kill_rgb_processes(self):
        """Kill all RGB effect processes to stop running effects"""
        try:
            import psutil
            
            killed_count = 0
            process_names = [
                'enhanced-professional-control-center',
                'rgb-lid-monitor',
                'openrgb', 
                'ckb-next',
                'python3'
            ]
            
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    if proc.info['cmdline']:
                        cmdline_str = ' '.join(proc.info['cmdline']).lower()
                        
                        # Kill RGB control processes
                        if any(name in cmdline_str for name in process_names):
                            # Don't kill ourselves (lid monitor daemon)
                            if 'lid-monitor-daemon' not in cmdline_str:
                                proc_info = f"PID {proc.info['pid']} - {' '.join(proc.info['cmdline'][:3])}"
                                logging.info(f"Killing RGB process: {proc_info}")
                                
                                try:
                                    proc.terminate()
                                    proc.wait(timeout=2)
                                    killed_count += 1
                                except (psutil.NoSuchProcess, psutil.TimeoutExpired):
                                    try:
                                        proc.kill()
                                        killed_count += 1
                                    except psutil.NoSuchProcess:
                                        pass
                                        
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            
            if killed_count > 0:
                logging.info(f"Killed {killed_count} RGB process(es)")
            else:
                logging.info("No RGB processes found to kill")
                
            # Also try to stop any RGB daemons
            try:
                subprocess.run(['pkill', '-f', 'rgb.*effect'], check=False, timeout=3)
                subprocess.run(['pkill', '-f', 'rainbow'], check=False, timeout=3)
            except Exception:
                pass
                
        except ImportError:
            # Fallback without psutil
            logging.warning("psutil not available, using fallback process killing")
            try:
                subprocess.run(['pkill', '-f', 'enhanced-professional-control-center'], check=False, timeout=3)
                subprocess.run(['pkill', '-f', 'rgb-lid-monitor'], check=False, timeout=3)
                subprocess.run(['pkill', '-f', 'openrgb'], check=False, timeout=3)
            except Exception as e:
                logging.error(f"Fallback process killing failed: {e}")

def main():
    """Main daemon entry point"""
    print("🔒 OriginPC Lid Monitor Daemon v1.0")
    print("===================================")
    
    # This daemon is designed to run unprivileged as a systemd --user service.
    # RGB access only requires the hidraw device to be world-writable (see
    # packaging/99-originpc-rgb.rules); root is neither required nor expected.
    if os.geteuid() == 0:
        print("⚠️  Warning: Running as root is unnecessary for this daemon.")
        print("   Recommended: systemctl --user enable --now originpc-lid-monitor")
    
    # Check RGB device
    if not os.path.exists('/dev/hidraw0'):
        print("❌ RGB device not found at /dev/hidraw0")
        sys.exit(1)
    
    print("✅ Starting lid monitoring daemon...")
    print("   Check status with: systemctl --user status originpc-lid-monitor")
    
    daemon = LidMonitorDaemon()
    
    try:
        daemon.start()
    except Exception as e:
        logging.error(f"Daemon failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

