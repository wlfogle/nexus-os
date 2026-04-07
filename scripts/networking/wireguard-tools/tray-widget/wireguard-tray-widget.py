#!/usr/bin/env python3
"""
WireGuard System Tray Widget for Pop!_OS
Provides quick access to WireGuard VPN, API masking, and dashboard controls
Uses local Docker containers instead of remote Proxmox SSH
"""

import sys
import subprocess
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
import os

TOOLS_DIR = os.path.expanduser("~/wireguard-tools")
WG_CONTAINER = "wireguard"
API_PROXY_PORT = "8080"
DASHBOARD_PORT = "52821"


class StatusChecker(QThread):
    """Background thread for checking service status"""
    statusUpdated = pyqtSignal(dict)

    def __init__(self):
        super().__init__()
        self.running = True

    def run(self):
        while self.running:
            status = self.check_all_services()
            self.statusUpdated.emit(status)
            time.sleep(10)

    def stop(self):
        self.running = False
        self.quit()
        self.wait()

    def check_all_services(self):
        status = {
            'wireguard': False,
            'api_proxy': False,
            'dashboard': False,
            'external_ip': 'Unknown',
            'vpn_interface': None,
            'peer_count': 0
        }

        try:
            result = subprocess.run(
                ['docker', 'ps', '--format', '{{.Names}}'],
                capture_output=True, text=True, timeout=5
            )
            containers = result.stdout.strip().split('\n')
            status['wireguard'] = WG_CONTAINER in containers
            status['dashboard'] = 'wg-easy' in containers

            if status['wireguard']:
                result = subprocess.run(
                    ['docker', 'exec', WG_CONTAINER, 'wg', 'show'],
                    capture_output=True, text=True, timeout=5
                )
                if result.returncode == 0 and result.stdout:
                    status['vpn_interface'] = 'wg0'
                    status['peer_count'] = result.stdout.count('peer:')
        except Exception:
            pass

        try:
            response = requests.get(
                f"http://127.0.0.1:{API_PROXY_PORT}/health", timeout=2
            )
            status['api_proxy'] = response.status_code == 200
        except Exception:
            pass

        try:
            response = requests.get("http://ifconfig.me", timeout=5)
            if response.status_code == 200:
                status['external_ip'] = response.text.strip()
        except Exception:
            pass

        return status


class ControlPanel(QDialog):
    """Main control panel dialog"""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("🛡️ WireGuard Control Panel - Pop!_OS")
        self.setFixedSize(600, 500)
        self.setup_ui()

    def setup_ui(self):
        layout = QVBoxLayout()

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
            label.setFont(QFont("Monospace", 10))
            status_layout.addWidget(label)

        status_group.setLayout(status_layout)
        layout.addWidget(status_group)

        # Control Buttons
        buttons_group = QWidget()
        buttons_layout = QVBoxLayout()

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

        # Auto-rotation
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

        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        layout.addWidget(self.progress_bar)

        self.log_output = QTextEdit()
        self.log_output.setMaximumHeight(120)
        self.log_output.setFont(QFont("Monospace", 8))
        layout.addWidget(QLabel("📋 Output Log:"))
        layout.addWidget(self.log_output)

        self.setLayout(layout)

        self.auto_rotation_timer = QTimer()
        self.auto_rotation_timer.timeout.connect(self.auto_rotate_vpn)
        self.stealth_mode_active = False

    def update_status(self, status):
        if status['wireguard']:
            iface = status['vpn_interface'] or "wg0"
            peers = status.get('peer_count', 0)
            self.vpn_status.setText(f"✅ VPN: Active ({iface}, {peers} peers)")
            self.vpn_status.setStyleSheet("color: green; font-weight: bold;")
        else:
            self.vpn_status.setText("❌ VPN: Container not running")
            self.vpn_status.setStyleSheet("color: red; font-weight: bold;")

        if status['api_proxy']:
            self.proxy_status.setText(f"✅ API Proxy: Active (:{API_PROXY_PORT})")
            self.proxy_status.setStyleSheet("color: green; font-weight: bold;")
        else:
            self.proxy_status.setText("❌ API Proxy: Inactive")
            self.proxy_status.setStyleSheet("color: red; font-weight: bold;")

        if status['dashboard']:
            self.dashboard_status.setText("✅ WG-Easy Dashboard: Running")
            self.dashboard_status.setStyleSheet("color: green; font-weight: bold;")
        else:
            self.dashboard_status.setText("❌ WG-Easy Dashboard: Not running")
            self.dashboard_status.setStyleSheet("color: red; font-weight: bold;")

        self.ip_status.setText(f"📡 External IP: {status['external_ip']}")
        self.ip_status.setStyleSheet("color: blue; font-weight: bold;")

    def log_message(self, message):
        timestamp = time.strftime("%H:%M:%S")
        self.log_output.append(f"[{timestamp}] {message}")
        self.log_output.ensureCursorVisible()

    def run_command(self, command, success_msg, error_msg):
        self.progress_bar.setVisible(True)
        self.progress_bar.setRange(0, 0)
        try:
            result = subprocess.run(command, shell=True, capture_output=True,
                                  text=True, timeout=30)
            if result.returncode == 0:
                self.log_message(f"✅ {success_msg}")
                if result.stdout:
                    self.log_message(result.stdout.strip()[:200])
            else:
                self.log_message(f"❌ {error_msg}")
                if result.stderr:
                    self.log_message(result.stderr.strip()[:200])
        except subprocess.TimeoutExpired:
            self.log_message(f"⏰ Command timed out")
        except Exception as e:
            self.log_message(f"❌ Error: {str(e)}")
        finally:
            self.progress_bar.setVisible(False)

    def rotate_vpn(self):
        self.log_message("🔄 Starting VPN rotation...")
        cmd = f'bash {TOOLS_DIR}/server/wireguard-rotate.sh rotate pop-os-client'
        self.run_command(cmd, "VPN rotation completed", "VPN rotation failed")

    def open_dashboard(self):
        url = f"http://localhost:{DASHBOARD_PORT}"
        try:
            webbrowser.open(url)
            self.log_message(f"🌐 Opened dashboard: {url}")
        except Exception as e:
            self.log_message(f"❌ Failed to open dashboard: {str(e)}")

    def start_api_proxy(self):
        self.log_message("🚀 Starting API masking proxy...")
        cmd = f'nohup python3 {TOOLS_DIR}/api-masking/api-mask-proxy.py > {TOOLS_DIR}/api-proxy.log 2>&1 &'
        self.run_command(cmd, "API proxy started", "Failed to start API proxy")

    def stop_api_proxy(self):
        self.log_message("🛑 Stopping API masking proxy...")
        cmd = 'pkill -f "api-mask-proxy.py"'
        self.run_command(cmd, "API proxy stopped", "Failed to stop API proxy")

    def test_ai_access(self):
        self.log_message("🤖 Testing AI service access...")
        services = ["claude.ai", "api.openai.com", "api.anthropic.com",
                    "api.cohere.ai", "api.mistral.ai"]
        for service in services:
            try:
                requests.get(f"https://{service}", timeout=5)
                self.log_message(f"✅ {service}: Accessible")
            except Exception:
                self.log_message(f"❌ {service}: Blocked/Unavailable")

    def toggle_stealth_mode(self):
        if not self.stealth_mode_active:
            self.stealth_mode_active = True
            self.stealth_btn.setText("🛑 Stop Stealth")
            self.stealth_btn.setStyleSheet("background-color: #F44336; color: white; font-weight: bold;")
            self.log_message("🥷 Stealth mode activated - rotating every 30 minutes")
            self.auto_rotation_timer.start(30 * 60 * 1000)
        else:
            self.stealth_mode_active = False
            self.stealth_btn.setText("🥷 Stealth Mode")
            self.stealth_btn.setStyleSheet("background-color: #607D8B; color: white; font-weight: bold;")
            self.auto_rotation_timer.stop()
            self.log_message("🛑 Stealth mode deactivated")

    def toggle_auto_rotate(self, state):
        if state == Qt.Checked:
            interval_ms = self.rotation_interval.value() * 60 * 1000
            self.auto_rotation_timer.start(interval_ms)
            self.log_message(f"⏰ Auto-rotation enabled: every {self.rotation_interval.value()} minutes")
        else:
            self.auto_rotation_timer.stop()
            self.log_message("⏰ Auto-rotation disabled")

    def auto_rotate_vpn(self):
        self.log_message("⏰ Auto-rotation triggered")
        self.rotate_vpn()


class WireGuardTrayWidget(QWidget):
    """Main system tray application"""

    def __init__(self):
        super().__init__()

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

        self.status_checker = StatusChecker()
        self.status_checker.statusUpdated.connect(self.update_tray_status)
        self.status_checker.start()

        self.tray_icon.show()
        self.update_tooltip("WireGuard Manager - Initializing...")

    def create_tray_icon(self):
        self.tray_icon = QSystemTrayIcon(self)
        self.update_icon_color(QColor(33, 150, 243))

    def create_actions(self):
        self.control_panel_action = QAction("🎛️ Control Panel", self)
        self.control_panel_action.triggered.connect(self.show_control_panel)

        self.quick_rotate_action = QAction("🔄 Quick Rotate", self)
        self.quick_rotate_action.triggered.connect(self.quick_rotate)

        self.dashboard_action = QAction("🌐 WG-Easy Dashboard", self)
        self.dashboard_action.triggered.connect(self.open_dashboard)

        self.toggle_proxy_action = QAction("🚀 Toggle API Proxy", self)
        self.toggle_proxy_action.triggered.connect(self.toggle_api_proxy)

        self.status_action = QAction("📊 Status", self)
        self.status_action.triggered.connect(self.show_status)

        self.quit_action = QAction("❌ Quit", self)
        self.quit_action.triggered.connect(self.quit_application)

    def create_menu(self):
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
        if reason in (QSystemTrayIcon.DoubleClick, QSystemTrayIcon.Trigger):
            self.show_control_panel()

    def show_control_panel(self):
        if self.control_panel is None:
            self.control_panel = ControlPanel()
            if self.status_checker:
                self.status_checker.statusUpdated.connect(self.control_panel.update_status)
        self.control_panel.show()
        self.control_panel.raise_()
        self.control_panel.activateWindow()

    def quick_rotate(self):
        try:
            subprocess.Popen(
                ['bash', f'{TOOLS_DIR}/server/wireguard-rotate.sh', 'rotate', 'quick-rotate'],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
            )
            self.tray_icon.showMessage("WireGuard", "VPN rotation started...",
                                     QSystemTrayIcon.Information, 3000)
        except Exception as e:
            self.tray_icon.showMessage("WireGuard", f"Rotation failed: {str(e)}",
                                     QSystemTrayIcon.Critical, 3000)

    def open_dashboard(self):
        try:
            webbrowser.open(f"http://localhost:{DASHBOARD_PORT}")
            self.tray_icon.showMessage("WireGuard", "Dashboard opened",
                                     QSystemTrayIcon.Information, 2000)
        except Exception as e:
            self.tray_icon.showMessage("WireGuard", f"Failed: {str(e)}",
                                     QSystemTrayIcon.Critical, 3000)

    def toggle_api_proxy(self):
        if self.current_status.get('api_proxy', False):
            subprocess.Popen(['pkill', '-f', 'api-mask-proxy.py'],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            self.tray_icon.showMessage("WireGuard", "API proxy stopped",
                                     QSystemTrayIcon.Information, 2000)
        else:
            try:
                subprocess.Popen(
                    ['python3', f'{TOOLS_DIR}/api-masking/api-mask-proxy.py'],
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
                )
                self.tray_icon.showMessage("WireGuard", "API proxy started",
                                         QSystemTrayIcon.Information, 2000)
            except Exception as e:
                self.tray_icon.showMessage("WireGuard", f"Failed: {str(e)}",
                                         QSystemTrayIcon.Critical, 3000)

    def show_status(self):
        s = self.current_status
        vpn = "✅ Active" if s.get('wireguard') else "❌ Inactive"
        proxy = "✅ Active" if s.get('api_proxy') else "❌ Inactive"
        dash = "✅ Running" if s.get('dashboard') else "❌ Down"
        msg = f"VPN: {vpn}\nAPI Proxy: {proxy}\nDashboard: {dash}\nIP: {s.get('external_ip', 'Unknown')}"
        self.tray_icon.showMessage("WireGuard Status", msg,
                                 QSystemTrayIcon.Information, 5000)

    def update_tray_status(self, status):
        self.current_status = status
        vpn_st = "Active" if status.get('wireguard') else "Inactive"
        ip = status.get('external_ip', 'Unknown')
        self.update_tooltip(f"WireGuard: {vpn_st} | IP: {ip}")

        if status.get('wireguard'):
            self.update_icon_color(QColor(76, 175, 80))
        else:
            self.update_icon_color(QColor(244, 67, 54))

    def update_icon_color(self, color):
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
        self.tray_icon.setToolTip(text)

    def quit_application(self):
        if self.status_checker:
            self.status_checker.stop()
        if self.control_panel:
            self.control_panel.close()
        QApplication.quit()


def main():
    app = QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(False)
    app.setApplicationName("WireGuard Manager")
    app.setApplicationVersion("1.0")
    app.setOrganizationName("Pop!_OS Tools")

    widget = WireGuardTrayWidget()
    sys.exit(app.exec_())


if __name__ == '__main__':
    main()
