#!/usr/bin/env python3
"""
WireGuard System Tray Widget for Garuda Linux
Provides quick access to WireGuard VPN, API masking, and dashboard controls
"""

import sys
import subprocess
import json
import time
import threading
import requests
from PyQt5.QtWidgets import (QApplication, QSystemTrayIcon, QMenu, QAction, 
                           QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                           QPushButton, QTextEdit, QDialog, QMessageBox,
                           QProgressBar, QCheckBox, QSpinBox)
from PyQt5.QtCore import QTimer, QThread, pyqtSignal, Qt
from PyQt5.QtGui import QIcon, QPixmap, QFont, QPainter, QColor
import webbrowser

class StatusChecker(QThread):
    """Background thread for checking service status"""
    statusUpdated = pyqtSignal(dict)
    
    def __init__(self):
        super().__init__()
        self.running = True
        self.garuda_host = "127.0.0.1"
        self.dashboard_port = "8081"
        self.api_proxy_port = "8080"
        
    def run(self):
        while self.running:
            status = self.check_all_services()
            self.statusUpdated.emit(status)
            time.sleep(10)  # Check every 10 seconds
            
    def stop(self):
        self.running = False
        self.quit()
        self.wait()
    
    def check_all_services(self):
        """Check status of all services"""
        status = {
            'wireguard': False,
            'api_proxy': False,
            'dashboard': False,
            'external_ip': 'Unknown',
            'vpn_interface': None,
            'last_rotation': 'Unknown'
        }
        
        try:
            # Check WireGuard VPN
            result = subprocess.run(['pgrep', '-f', 'wg-quick'], 
                                  capture_output=True, text=True, timeout=5)
            status['wireguard'] = result.returncode == 0
            
            if status['wireguard']:
                # Get VPN interface info
                result = subprocess.run(['wg', 'show'], 
                                      capture_output=True, text=True, timeout=5)
                if result.returncode == 0 and result.stdout:
                    for line in result.stdout.split('\n'):
                        if line.startswith('interface:'):
                            status['vpn_interface'] = line.split(':')[1].strip()
                            break
        except:
            pass
            
        try:
            # Check API Masking Proxy
            response = requests.get(f"http://127.0.0.1:{self.api_proxy_port}/health", 
                                  timeout=2)
            status['api_proxy'] = response.status_code == 200
        except:
            pass
            
        try:
            # Check WGDashboard
            response = requests.get(f"http://{self.proxmox_host}:{self.dashboard_port}/", 
                                  timeout=5)
            status['dashboard'] = response.status_code == 200
        except:
            pass
            
        try:
            # Get external IP
            response = requests.get("http://ifconfig.me", timeout=5)
            if response.status_code == 200:
                status['external_ip'] = response.text.strip()
        except:
            pass
            
        return status

class ControlPanel(QDialog):
    """Main control panel dialog"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("🛡️ WireGuard Control Panel")
        self.setFixedSize(600, 500)
        self.proxmox_host = "192.168.122.9"
        self.proxmox_user = "root"
        self.dashboard_port = "10086"
        self.api_proxy_port = "8080"
        
        self.setup_ui()
        
    def setup_ui(self):
        layout = QVBoxLayout()
        
        # Header
        header = QLabel("🛡️ WireGuard VPN Manager")
        header.setFont(QFont("Arial", 16, QFont.Bold))
        header.setAlignment(Qt.AlignCenter)
        layout.addWidget(header)
        
        # Status Section
        status_group = QWidget()
        status_layout = QVBoxLayout()
        
        self.status_label = QLabel("📊 Service Status")
        self.status_label.setFont(QFont("Arial", 12, QFont.Bold))
        status_layout.addWidget(self.status_label)
        
        self.vpn_status = QLabel("🔄 VPN: Checking...")
        self.proxy_status = QLabel("🔄 API Proxy: Checking...")
        self.dashboard_status = QLabel("🔄 Dashboard: Checking...")
        self.ip_status = QLabel("🔄 External IP: Checking...")
        
        for label in [self.vpn_status, self.proxy_status, self.dashboard_status, self.ip_status]:
            label.setFont(QFont("Consolas", 10))
            status_layout.addWidget(label)
            
        status_group.setLayout(status_layout)
        layout.addWidget(status_group)
        
        # Control Buttons
        buttons_group = QWidget()
        buttons_layout = QVBoxLayout()
        
        # VPN Controls
        vpn_row = QHBoxLayout()
        self.rotate_btn = QPushButton("🔄 Rotate VPN")
        self.rotate_btn.clicked.connect(self.rotate_vpn)
        self.rotate_btn.setStyleSheet("background-color: #2196F3; color: white; font-weight: bold;")
        
        self.dashboard_btn = QPushButton("🌐 Dashboard")
        self.dashboard_btn.clicked.connect(self.open_dashboard)
        self.dashboard_btn.setStyleSheet("background-color: #4CAF50; color: white; font-weight: bold;")
        
        vpn_row.addWidget(self.rotate_btn)
        vpn_row.addWidget(self.dashboard_btn)
        buttons_layout.addLayout(vpn_row)
        
        # API Proxy Controls
        proxy_row = QHBoxLayout()
        self.start_proxy_btn = QPushButton("🚀 Start Proxy")
        self.start_proxy_btn.clicked.connect(self.start_api_proxy)
        self.start_proxy_btn.setStyleSheet("background-color: #FF9800; color: white; font-weight: bold;")
        
        self.stop_proxy_btn = QPushButton("🛑 Stop Proxy")
        self.stop_proxy_btn.clicked.connect(self.stop_api_proxy)
        self.stop_proxy_btn.setStyleSheet("background-color: #F44336; color: white; font-weight: bold;")
        
        proxy_row.addWidget(self.start_proxy_btn)
        proxy_row.addWidget(self.stop_proxy_btn)
        buttons_layout.addLayout(proxy_row)
        
        # Testing Controls
        test_row = QHBoxLayout()
        self.test_ai_btn = QPushButton("🤖 Test AI Access")
        self.test_ai_btn.clicked.connect(self.test_ai_access)
        self.test_ai_btn.setStyleSheet("background-color: #9C27B0; color: white; font-weight: bold;")
        
        self.stealth_btn = QPushButton("🥷 Stealth Mode")
        self.stealth_btn.clicked.connect(self.toggle_stealth_mode)
        self.stealth_btn.setStyleSheet("background-color: #607D8B; color: white; font-weight: bold;")
        
        test_row.addWidget(self.test_ai_btn)
        test_row.addWidget(self.stealth_btn)
        buttons_layout.addLayout(test_row)
        
        buttons_group.setLayout(buttons_layout)
        layout.addWidget(buttons_group)
        
        # Auto-rotation settings
        auto_group = QWidget()
        auto_layout = QHBoxLayout()
        
        self.auto_rotate_cb = QCheckBox("Auto-rotate every")
        self.auto_rotate_cb.stateChanged.connect(self.toggle_auto_rotate)
        
        self.rotation_interval = QSpinBox()
        self.rotation_interval.setRange(5, 120)
        self.rotation_interval.setValue(30)
        self.rotation_interval.setSuffix(" minutes")
        
        auto_layout.addWidget(self.auto_rotate_cb)
        auto_layout.addWidget(self.rotation_interval)
        auto_layout.addStretch()
        
        auto_group.setLayout(auto_layout)
        layout.addWidget(auto_group)
        
        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        layout.addWidget(self.progress_bar)
        
        # Log output
        self.log_output = QTextEdit()
        self.log_output.setMaximumHeight(120)
        self.log_output.setFont(QFont("Consolas", 8))
        layout.addWidget(QLabel("📋 Output Log:"))
        layout.addWidget(self.log_output)
        
        self.setLayout(layout)
        
        # Auto-rotation timer
        self.auto_rotation_timer = QTimer()
        self.auto_rotation_timer.timeout.connect(self.auto_rotate_vpn)
        
        self.stealth_mode_active = False
        
    def update_status(self, status):
        """Update status labels"""
        # VPN Status
        if status['wireguard']:
            interface = status['vpn_interface'] or "wg0"
            self.vpn_status.setText(f"✅ VPN: Active ({interface})")
            self.vpn_status.setStyleSheet("color: green; font-weight: bold;")
        else:
            self.vpn_status.setText("❌ VPN: Inactive")
            self.vpn_status.setStyleSheet("color: red; font-weight: bold;")
            
        # API Proxy Status
        if status['api_proxy']:
            self.proxy_status.setText(f"✅ API Proxy: Active (:{self.api_proxy_port})")
            self.proxy_status.setStyleSheet("color: green; font-weight: bold;")
        else:
            self.proxy_status.setText("❌ API Proxy: Inactive")
            self.proxy_status.setStyleSheet("color: red; font-weight: bold;")
            
        # Dashboard Status
        if status['dashboard']:
            self.dashboard_status.setText(f"✅ Dashboard: Accessible")
            self.dashboard_status.setStyleSheet("color: green; font-weight: bold;")
        else:
            self.dashboard_status.setText("❌ Dashboard: Not accessible")
            self.dashboard_status.setStyleSheet("color: red; font-weight: bold;")
            
        # External IP
        self.ip_status.setText(f"📡 External IP: {status['external_ip']}")
        self.ip_status.setStyleSheet("color: blue; font-weight: bold;")
        
    def log_message(self, message):
        """Add message to log output"""
        timestamp = time.strftime("%H:%M:%S")
        self.log_output.append(f"[{timestamp}] {message}")
        self.log_output.ensureCursorVisible()
        
    def run_command(self, command, success_msg, error_msg):
        """Run shell command and show progress"""
        self.progress_bar.setVisible(True)
        self.progress_bar.setRange(0, 0)  # Indeterminate progress
        
        try:
            result = subprocess.run(command, shell=True, capture_output=True, 
                                  text=True, timeout=30)
            if result.returncode == 0:
                self.log_message(f"✅ {success_msg}")
                if result.stdout:
                    self.log_message(result.stdout.strip())
            else:
                self.log_message(f"❌ {error_msg}")
                if result.stderr:
                    self.log_message(result.stderr.strip())
        except subprocess.TimeoutExpired:
            self.log_message(f"⏰ Command timed out: {command}")
        except Exception as e:
            self.log_message(f"❌ Error: {str(e)}")
        finally:
            self.progress_bar.setVisible(False)
            
    def rotate_vpn(self):
        """Rotate WireGuard configuration"""
        self.log_message("🔄 Starting VPN rotation...")
        cmd = f'ssh -o ConnectTimeout=5 {self.proxmox_user}@{self.proxmox_host} "/root/wireguard-rotate.sh garuda-host"'
        self.run_command(cmd, "VPN rotation completed", "VPN rotation failed")
        
    def open_dashboard(self):
        """Open WireGuard dashboard in browser"""
        url = f"http://{self.proxmox_host}:{self.dashboard_port}"
        try:
            webbrowser.open(url)
            self.log_message(f"🌐 Opened dashboard: {url}")
        except Exception as e:
            self.log_message(f"❌ Failed to open dashboard: {str(e)}")
            
    def start_api_proxy(self):
        """Start API masking proxy"""
        self.log_message("🚀 Starting API masking proxy...")
        cmd = f'nohup python3 /root/api-mask-proxy.py > /var/log/api-proxy.log 2>&1 &'
        self.run_command(cmd, "API proxy started", "Failed to start API proxy")
        
    def stop_api_proxy(self):
        """Stop API masking proxy"""
        self.log_message("🛑 Stopping API masking proxy...")
        cmd = 'pkill -f "api-mask-proxy.py"'
        self.run_command(cmd, "API proxy stopped", "Failed to stop API proxy")
        
    def test_ai_access(self):
        """Test AI service accessibility"""
        self.log_message("🤖 Testing AI service access...")
        
        ai_services = [
            "claude.ai", "api.openai.com", "chat.openai.com",
            "api.anthropic.com", "api.cohere.ai", "api.mistral.ai"
        ]
        
        for service in ai_services:
            try:
                response = requests.get(f"https://{service}", timeout=5)
                self.log_message(f"✅ {service}: Accessible")
            except:
                self.log_message(f"❌ {service}: Blocked/Unavailable")
                
    def toggle_stealth_mode(self):
        """Toggle stealth mode (rapid rotation)"""
        if not self.stealth_mode_active:
            self.stealth_mode_active = True
            self.stealth_btn.setText("🛑 Stop Stealth")
            self.stealth_btn.setStyleSheet("background-color: #F44336; color: white; font-weight: bold;")
            self.log_message("🥷 Stealth mode activated - rotating every 30 minutes")
            
            # Start aggressive rotation
            self.auto_rotation_timer.start(30 * 60 * 1000)  # 30 minutes
        else:
            self.stealth_mode_active = False
            self.stealth_btn.setText("🥷 Stealth Mode")
            self.stealth_btn.setStyleSheet("background-color: #607D8B; color: white; font-weight: bold;")
            self.auto_rotation_timer.stop()
            self.log_message("🛑 Stealth mode deactivated")
            
    def toggle_auto_rotate(self, state):
        """Toggle auto-rotation"""
        if state == Qt.Checked:
            interval_ms = self.rotation_interval.value() * 60 * 1000
            self.auto_rotation_timer.start(interval_ms)
            self.log_message(f"⏰ Auto-rotation enabled: every {self.rotation_interval.value()} minutes")
        else:
            self.auto_rotation_timer.stop()
            self.log_message("⏰ Auto-rotation disabled")
            
    def auto_rotate_vpn(self):
        """Automatic VPN rotation"""
        self.log_message("⏰ Auto-rotation triggered")
        self.rotate_vpn()

class WireGuardTrayWidget(QWidget):
    """Main system tray application"""
    
    def __init__(self):
        super().__init__()
        
        # Check if system tray is available
        if not QSystemTrayIcon.isSystemTrayAvailable():
            QMessageBox.critical(None, "System Tray", 
                               "System tray is not available on this system.")
            sys.exit(1)
            
        self.control_panel = None
        self.status_checker = None
        self.current_status = {}
        
        self.create_tray_icon()
        self.create_actions()
        self.create_menu()
        
        # Start status checker
        self.status_checker = StatusChecker()
        self.status_checker.statusUpdated.connect(self.update_tray_status)
        self.status_checker.start()
        
        self.tray_icon.show()
        self.update_tooltip("WireGuard Manager - Initializing...")
        
    def create_tray_icon(self):
        """Create system tray icon"""
        self.tray_icon = QSystemTrayIcon(self)
        
        # Create custom icon
        pixmap = QPixmap(16, 16)
        pixmap.fill(QColor(0, 0, 0, 0))  # Transparent background
        painter = QPainter(pixmap)
        painter.setRenderHint(QPainter.Antialiasing)
        
        # Draw VPN shield icon
        painter.setBrush(QColor(33, 150, 243))  # Blue
        painter.setPen(QColor(255, 255, 255))
        painter.drawRoundedRect(2, 2, 12, 12, 2, 2)
        
        # Add "W" for WireGuard
        painter.setPen(QColor(255, 255, 255))
        painter.setFont(QFont("Arial", 8, QFont.Bold))
        painter.drawText(4, 12, "W")
        
        painter.end()
        
        icon = QIcon(pixmap)
        self.tray_icon.setIcon(icon)
        
    def create_actions(self):
        """Create menu actions"""
        self.control_panel_action = QAction("🎛️ Control Panel", self)
        self.control_panel_action.triggered.connect(self.show_control_panel)
        
        self.quick_rotate_action = QAction("🔄 Quick Rotate", self)
        self.quick_rotate_action.triggered.connect(self.quick_rotate)
        
        self.dashboard_action = QAction("🌐 Dashboard", self)
        self.dashboard_action.triggered.connect(self.open_dashboard)
        
        self.toggle_proxy_action = QAction("🚀 Toggle API Proxy", self)
        self.toggle_proxy_action.triggered.connect(self.toggle_api_proxy)
        
        self.status_action = QAction("📊 Status", self)
        self.status_action.triggered.connect(self.show_status)
        
        self.quit_action = QAction("❌ Quit", self)
        self.quit_action.triggered.connect(self.quit_application)
        
    def create_menu(self):
        """Create context menu"""
        self.tray_menu = QMenu()
        
        self.tray_menu.addAction(self.control_panel_action)
        self.tray_menu.addSeparator()
        self.tray_menu.addAction(self.quick_rotate_action)
        self.tray_menu.addAction(self.dashboard_action)
        self.tray_menu.addAction(self.toggle_proxy_action)
        self.tray_menu.addSeparator()
        self.tray_menu.addAction(self.status_action)
        self.tray_menu.addSeparator()
        self.tray_menu.addAction(self.quit_action)
        
        self.tray_icon.setContextMenu(self.tray_menu)
        self.tray_icon.activated.connect(self.on_tray_icon_activated)
        
    def on_tray_icon_activated(self, reason):
        """Handle tray icon activation"""
        if reason == QSystemTrayIcon.DoubleClick:
            self.show_control_panel()
        elif reason == QSystemTrayIcon.Trigger:
            self.show_control_panel()
            
    def show_control_panel(self):
        """Show main control panel"""
        if self.control_panel is None:
            self.control_panel = ControlPanel()
            if self.status_checker:
                self.status_checker.statusUpdated.connect(self.control_panel.update_status)
                
        self.control_panel.show()
        self.control_panel.raise_()
        self.control_panel.activateWindow()
        
    def quick_rotate(self):
        """Quick VPN rotation"""
        try:
            subprocess.Popen([
                'ssh', '-o', 'ConnectTimeout=5',
                f'root@192.168.122.9',
                '/root/wireguard-rotate.sh garuda-host'
            ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            self.tray_icon.showMessage("WireGuard", "VPN rotation started...", 
                                     QSystemTrayIcon.Information, 3000)
        except Exception as e:
            self.tray_icon.showMessage("WireGuard", f"Rotation failed: {str(e)}", 
                                     QSystemTrayIcon.Critical, 3000)
            
    def open_dashboard(self):
        """Open dashboard in browser"""
        url = "http://192.168.122.9:10086"
        try:
            webbrowser.open(url)
            self.tray_icon.showMessage("WireGuard", "Dashboard opened in browser", 
                                     QSystemTrayIcon.Information, 2000)
        except Exception as e:
            self.tray_icon.showMessage("WireGuard", f"Failed to open dashboard: {str(e)}", 
                                     QSystemTrayIcon.Critical, 3000)
            
    def toggle_api_proxy(self):
        """Toggle API masking proxy"""
        if self.current_status.get('api_proxy', False):
            # Stop proxy
            subprocess.Popen(['pkill', '-f', 'api-mask-proxy.py'], 
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            self.tray_icon.showMessage("WireGuard", "API proxy stopped", 
                                     QSystemTrayIcon.Information, 2000)
        else:
            # Start proxy
            try:
                subprocess.Popen([
                    'nohup', 'python3', '/root/api-mask-proxy.py'
                ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                self.tray_icon.showMessage("WireGuard", "API proxy started", 
                                         QSystemTrayIcon.Information, 2000)
            except Exception as e:
                self.tray_icon.showMessage("WireGuard", f"Failed to start proxy: {str(e)}", 
                                         QSystemTrayIcon.Critical, 3000)
                
    def show_status(self):
        """Show quick status in notification"""
        status = self.current_status
        vpn_status = "✅ Active" if status.get('wireguard', False) else "❌ Inactive"
        proxy_status = "✅ Active" if status.get('api_proxy', False) else "❌ Inactive"
        dashboard_status = "✅ Available" if status.get('dashboard', False) else "❌ Unavailable"
        
        message = f"VPN: {vpn_status}\nAPI Proxy: {proxy_status}\nDashboard: {dashboard_status}\nIP: {status.get('external_ip', 'Unknown')}"
        
        self.tray_icon.showMessage("WireGuard Status", message, 
                                 QSystemTrayIcon.Information, 5000)
        
    def update_tray_status(self, status):
        """Update tray icon based on status"""
        self.current_status = status
        
        # Update tooltip
        vpn_status = "Active" if status.get('wireguard', False) else "Inactive"
        ip = status.get('external_ip', 'Unknown')
        self.update_tooltip(f"WireGuard: {vpn_status} | IP: {ip}")
        
        # Update icon color based on VPN status
        if status.get('wireguard', False):
            # Green for active VPN
            self.update_icon_color(QColor(76, 175, 80))  # Green
        else:
            # Red for inactive VPN  
            self.update_icon_color(QColor(244, 67, 54))   # Red
            
    def update_icon_color(self, color):
        """Update tray icon color"""
        pixmap = QPixmap(16, 16)
        pixmap.fill(QColor(0, 0, 0, 0))
        painter = QPainter(pixmap)
        painter.setRenderHint(QPainter.Antialiasing)
        
        painter.setBrush(color)
        painter.setPen(QColor(255, 255, 255))
        painter.drawRoundedRect(2, 2, 12, 12, 2, 2)
        
        painter.setPen(QColor(255, 255, 255))
        painter.setFont(QFont("Arial", 8, QFont.Bold))
        painter.drawText(4, 12, "W")
        
        painter.end()
        
        self.tray_icon.setIcon(QIcon(pixmap))
        
    def update_tooltip(self, text):
        """Update tray icon tooltip"""
        self.tray_icon.setToolTip(text)
        
    def quit_application(self):
        """Clean up and quit"""
        if self.status_checker:
            self.status_checker.stop()
        if self.control_panel:
            self.control_panel.close()
        QApplication.quit()

def main():
    app = QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(False)  # Keep running when windows are closed
    
    # Set application info
    app.setApplicationName("WireGuard Manager")
    app.setApplicationVersion("1.0")
    app.setOrganizationName("Garuda Tools")
    
    widget = WireGuardTrayWidget()
    
    sys.exit(app.exec_())

if __name__ == '__main__':
    main()
