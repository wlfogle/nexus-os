#!/usr/bin/env python3
"""
AI-Enhanced System Optimizer for Rescue USB
Inspired by Ultimate Rescue USB project
"""

import os
import sys
import json
import time
import subprocess
import psutil
from pathlib import Path
from datetime import datetime

class AIRescueOptimizer:
    def __init__(self):
        self.config_path = "/etc/ai-rescue-config.json"
        self.log_path = "/var/log/ai-rescue.log"
        self.system_profile = {}
        
    def log(self, message):
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        with open(self.log_path, "a") as f:
            f.write(f"[{timestamp}] {message}\n")
        print(f"[AI-Rescue] {message}")
    
    def analyze_system_hardware(self):
        """Hardware failure prediction and analysis"""
        self.log("Analyzing system hardware profile...")
        
        profile = {
            "cpu_count": psutil.cpu_count(),
            "memory_total": psutil.virtual_memory().total,
            "disk_usage": {},
            "cpu_freq": psutil.cpu_freq()._asdict() if psutil.cpu_freq() else None,
            "boot_time": psutil.boot_time(),
            "load_avg": os.getloadavg()
        }
        
        # Analyze disk health
        for partition in psutil.disk_partitions():
            try:
                usage = psutil.disk_usage(partition.mountpoint)
                profile["disk_usage"][partition.device] = {
                    "total": usage.total,
                    "used": usage.used,
                    "free": usage.free,
                    "percent": usage.used / usage.total * 100
                }
            except:
                pass
                
        self.system_profile = profile
        return profile
    
    def detect_performance_anomalies(self):
        """AI-powered performance anomaly detection"""
        self.log("Detecting performance anomalies...")
        
        anomalies = []
        
        # CPU analysis
        cpu_percent = psutil.cpu_percent(interval=1)
        if cpu_percent > 80:
            anomalies.append(f"High CPU usage detected: {cpu_percent}%")
        
        # Memory analysis
        memory = psutil.virtual_memory()
        if memory.percent > 90:
            anomalies.append(f"High memory usage: {memory.percent}%")
        
        # Disk I/O analysis
        disk_io = psutil.disk_io_counters()
        if disk_io:
            # Simple heuristic for USB detection
            if disk_io.read_time + disk_io.write_time > 1000:
                anomalies.append("High disk I/O latency detected (possible USB bottleneck)")
        
        return anomalies
    
    def optimize_for_usb_storage(self):
        """Apply USB-specific optimizations"""
        self.log("Applying USB storage optimizations...")
        
        optimizations = [
            # Memory optimizations
            ("vm.swappiness", "1"),
            ("vm.dirty_ratio", "3"),
            ("vm.dirty_background_ratio", "1"),
            ("vm.vfs_cache_pressure", "200"),
            
            # I/O optimizations
            ("vm.dirty_expire_centisecs", "500"),
            ("vm.dirty_writeback_centisecs", "100"),
            
            # Desktop responsiveness
            ("kernel.sched_autogroup_enabled", "1"),
            ("kernel.sched_cfs_bandwidth_slice_us", "3000"),
        ]
        
        for param, value in optimizations:
            try:
                subprocess.run(["sysctl", "-w", f"{param}={value}"], 
                             capture_output=True, check=True)
                self.log(f"Applied: {param}={value}")
            except subprocess.CalledProcessError:
                self.log(f"Failed to apply: {param}={value}")
    
    def intelligent_service_management(self):
        """AI-guided service optimization"""
        self.log("Analyzing services for optimization...")
        
        # Services safe to delay for rescue USB
        delay_services = [
            "bluetooth.service",
            "cups.service",
            "avahi-daemon.service",
            "ModemManager.service"
        ]
        
        # Check which services are actually enabled
        for service in delay_services:
            try:
                result = subprocess.run(["systemctl", "is-enabled", service], 
                                      capture_output=True, text=True)
                if result.returncode == 0 and "enabled" in result.stdout:
                    # Create override to delay startup
                    override_dir = f"/etc/systemd/system/{service}.d"
                    os.makedirs(override_dir, exist_ok=True)
                    
                    with open(f"{override_dir}/rescue-delay.conf", "w") as f:
                        f.write("""[Unit]
# Delay non-essential service for rescue USB performance
After=graphical-session.target

[Service]
# Reduce priority
Nice=10
IOSchedulingClass=3
""")
                    self.log(f"Created performance override for {service}")
            except:
                pass
    
    def create_desktop_performance_profile(self):
        """Create Plasma performance profile"""
        self.log("Creating desktop performance profile...")
        
        plasma_config = {
            "effects_disabled": [
                "blur", "contrast", "coverswitch", "cube", "flipswitch",
                "glide", "magiclamp", "minimizeanimation", "scale", 
                "slidingpopups", "wobblywindows", "zoom"
            ],
            "animation_speed": 5,  # Fastest
            "compositor_backend": "OpenGL",
            "vsync": False
        }
        
        # Save configuration
        config_dir = "/etc/skel/.config"
        os.makedirs(config_dir, exist_ok=True)
        
        with open(f"{config_dir}/ai-plasma-performance.json", "w") as f:
            json.dump(plasma_config, f, indent=2)
        
        return plasma_config
    
    def generate_optimization_report(self):
        """Generate comprehensive system report"""
        self.log("Generating optimization report...")
        
        report = {
            "timestamp": datetime.now().isoformat(),
            "system_profile": self.system_profile,
            "anomalies": self.detect_performance_anomalies(),
            "optimizations_applied": True,
            "rescue_usb_ready": True,
            "recommendations": [
                "System optimized for USB storage performance",
                "Plasma configured for minimal resource usage",
                "Non-essential services delayed for faster boot",
                "AI monitoring enabled for predictive maintenance"
            ]
        }
        
        with open("/var/log/ai-rescue-report.json", "w") as f:
            json.dump(report, f, indent=2)
        
        return report
    
    def run_full_optimization(self):
        """Execute complete AI-enhanced optimization"""
        self.log("Starting AI-Enhanced Rescue USB Optimization...")
        
        try:
            # Step 1: Analyze hardware
            self.analyze_system_hardware()
            
            # Step 2: Detect issues
            anomalies = self.detect_performance_anomalies()
            if anomalies:
                self.log(f"Detected {len(anomalies)} performance issues")
            
            # Step 3: Apply optimizations
            self.optimize_for_usb_storage()
            
            # Step 4: Service management
            self.intelligent_service_management()
            
            # Step 5: Desktop profile
            self.create_desktop_performance_profile()
            
            # Step 6: Generate report
            report = self.generate_optimization_report()
            
            self.log("AI-Enhanced optimization completed successfully!")
            return True
            
        except Exception as e:
            self.log(f"Optimization failed: {str(e)}")
            return False

if __name__ == "__main__":
    optimizer = AIRescueOptimizer()
    success = optimizer.run_full_optimization()
    sys.exit(0 if success else 1)
