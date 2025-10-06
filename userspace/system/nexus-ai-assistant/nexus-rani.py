#!/usr/bin/env python3
"""
NexusOS AI Assistant (Nexus-Rani)
Similar to Garuda-Rani but integrated with awesome-stack and universal package management
"""

import sys
import os
import json
import subprocess
import argparse
from typing import Dict, List, Optional, Any
import requests
from pathlib import Path
import yaml
import time
from datetime import datetime

class NexusRani:
    def __init__(self):
        self.version = "1.0.0"
        self.awesome_stack_path = "/run/media/garuda/34c008f3-1990-471c-bd80-c72985c7dc5c/@home/lou/Repos/github/awesome-stack"
        self.config_path = Path.home() / ".config" / "nexus-rani"
        self.config_path.mkdir(exist_ok=True)
        self.config_file = self.config_path / "config.yaml"
        self.load_config()
        
    def load_config(self):
        """Load configuration from file"""
        default_config = {
            'ai_features': True,
            'awesome_stack_integration': True,
            'auto_optimize': False,
            'tray_notifications': True,
            'update_check_interval': 3600,
            'debug_mode': False,
            'supported_formats': [
                'deb', 'rpm', 'zst', 'flatpak', 'snap', 'appimage', 
                'pip', 'npm', 'cargo', 'docker'
            ]
        }
        
        if self.config_file.exists():
            with open(self.config_file, 'r') as f:
                self.config = yaml.safe_load(f) or default_config
        else:
            self.config = default_config
            self.save_config()
    
    def save_config(self):
        """Save configuration to file"""
        with open(self.config_file, 'w') as f:
            yaml.dump(self.config, f, default_flow_style=False)
    
    def print_banner(self):
        """Print NexusOS AI Assistant banner"""
        print(f"""
╔═══════════════════════════════════════════════════════════════╗
║                    🚀 NexusOS AI Assistant                   ║
║                      (Nexus-Rani v{self.version})                      ║
║                                                               ║
║  🎯 Universal Package Management  🛡️  Digital Fortress       ║
║  🎬 Media Stack Automation       🔐  Vaultwarden Security   ║
║  🏠 Self-Hosting Infrastructure  📦  Awesome-Stack Power    ║
╚═══════════════════════════════════════════════════════════════╝
        """)
    
    def run_command(self, command: str, capture=True, timeout=30) -> subprocess.CompletedProcess:
        """Run shell command safely with timeout"""
        try:
            if capture:
                result = subprocess.run(
                    command, shell=True, capture_output=True, 
                    text=True, timeout=timeout
                )
            else:
                result = subprocess.run(command, shell=True, timeout=timeout)
            return result
        except subprocess.TimeoutExpired:
            print(f"⚠️  Command timed out: {command}")
            return subprocess.CompletedProcess(command, 124, "", "Timeout")
        except Exception as e:
            print(f"❌ Command failed: {command} - {e}")
            return subprocess.CompletedProcess(command, 1, "", str(e))
    
    def check_system_status(self) -> Dict[str, Any]:
        """Check overall system status"""
        status = {
            'nexuspkg': False,
            'awesome_stack': False,
            'digital_fortress': False,
            'vaultwarden': False,
            'docker': False,
            'services_running': []
        }
        
        # Check NexusPkg
        result = self.run_command("nexuspkg status")
        status['nexuspkg'] = result.returncode == 0
        
        # Check Awesome Stack
        status['awesome_stack'] = Path(self.awesome_stack_path).exists()
        
        # Check Digital Fortress
        result = self.run_command("systemctl is-active digital-fortress")
        status['digital_fortress'] = result.stdout.strip() == "active"
        
        # Check Vaultwarden
        result = self.run_command("systemctl --user is-active vaultwarden")
        status['vaultwarden'] = result.stdout.strip() == "active"
        
        # Check Docker
        result = self.run_command("docker ps")
        status['docker'] = result.returncode == 0
        if status['docker']:
            containers = result.stdout.strip().split('\n')[1:]  # Skip header
            status['services_running'] = [line.split()[-1] for line in containers if line.strip()]
        
        return status
    
    def install_package(self, package: str, format_hint: Optional[str] = None) -> bool:
        """Install package using NexusPkg universal installer"""
        print(f"🚀 Installing {package}...")
        
        if format_hint:
            if format_hint == 'flatpak':
                cmd = f"nexuspkg flatpak {package}"
            elif format_hint == 'snap':
                cmd = f"nexuspkg snap {package}"
            elif format_hint == 'deb':
                cmd = f"nexuspkg deb install {package}"
            elif format_hint == 'rpm':
                cmd = f"nexuspkg rpm install {package}"
            elif format_hint == 'pip':
                cmd = f"nexuspkg pip {package}"
            elif format_hint == 'npm':
                cmd = f"nexuspkg npm {package}"
            elif format_hint == 'cargo':
                cmd = f"nexuspkg cargo {package}"
            else:
                cmd = f"nexuspkg install {package}"
        else:
            cmd = f"nexuspkg install {package}"
        
        result = self.run_command(cmd, capture=False)
        
        if result.returncode == 0:
            print(f"✅ Successfully installed {package}")
            return True
        else:
            print(f"❌ Failed to install {package}")
            return False
    
    def setup_media_stack(self) -> bool:
        """Setup complete media stack using awesome-stack"""
        if not Path(self.awesome_stack_path).exists():
            print("❌ Awesome Stack not found!")
            return False
        
        print("🎬 Setting up media stack...")
        
        # Use awesome-stack setup script
        cmd = f"cd {self.awesome_stack_path} && ./setup-complete-firetv-stack.sh"
        result = self.run_command(cmd, capture=False, timeout=300)
        
        if result.returncode == 0:
            print("✅ Media stack setup completed!")
            return True
        else:
            print("❌ Media stack setup failed!")
            return False
    
    def enable_digital_fortress(self) -> bool:
        """Enable Digital Fortress security suite"""
        if not Path(self.awesome_stack_path).exists():
            print("❌ Awesome Stack not found!")
            return False
        
        print("🛡️ Enabling Digital Fortress...")
        
        # Install and enable Digital Fortress
        install_cmd = f"cd {self.awesome_stack_path}/ghost-mode && ./install-ghost-mode.sh"
        result = self.run_command(install_cmd, capture=False, timeout=120)
        
        if result.returncode == 0:
            # Enable systemd service
            service_cmd = "sudo systemctl enable --now digital-fortress.service"
            service_result = self.run_command(service_cmd, capture=False)
            
            if service_result.returncode == 0:
                print("✅ Digital Fortress enabled and active!")
                return True
        
        print("❌ Failed to enable Digital Fortress!")
        return False
    
    def setup_vaultwarden(self) -> bool:
        """Setup Vaultwarden password manager"""
        print("🔐 Setting up Vaultwarden...")
        
        # Create vaultwarden user
        user_cmd = "sudo useradd -r -s /bin/false vaultwarden"
        self.run_command(user_cmd)
        
        # Create directories
        dirs = ["/opt/vaultwarden", "/var/lib/vaultwarden", "/var/log/vaultwarden"]
        for dir_path in dirs:
            self.run_command(f"sudo mkdir -p {dir_path}")
            self.run_command(f"sudo chown vaultwarden:vaultwarden {dir_path}")
        
        # Download and install Vaultwarden
        download_cmd = """
        cd /tmp &&
        wget https://github.com/dani-garcia/vaultwarden/releases/latest/download/vaultwarden-linux-x86_64.tar.gz &&
        sudo tar -xzf vaultwarden-linux-x86_64.tar.gz -C /opt/vaultwarden &&
        sudo chown vaultwarden:vaultwarden /opt/vaultwarden/vaultwarden &&
        sudo chmod +x /opt/vaultwarden/vaultwarden
        """
        
        result = self.run_command(download_cmd, timeout=60)
        
        if result.returncode == 0:
            # Enable systemd service
            service_cmd = "sudo systemctl enable --now vaultwarden.service"
            service_result = self.run_command(service_cmd)
            
            if service_result.returncode == 0:
                print("✅ Vaultwarden setup completed!")
                print("🌐 Access at: http://localhost:8080")
                return True
        
        print("❌ Vaultwarden setup failed!")
        return False
    
    def optimize_system(self) -> bool:
        """Optimize system using awesome-stack optimization"""
        if not Path(self.awesome_stack_path).exists():
            print("❌ Awesome Stack not found!")
            return False
        
        print("⚡ Optimizing system...")
        
        cmd = f"cd {self.awesome_stack_path} && sudo ./hardware_optimization_vm.sh"
        result = self.run_command(cmd, capture=False, timeout=180)
        
        if result.returncode == 0:
            print("✅ System optimization completed!")
            return True
        else:
            print("❌ System optimization failed!")
            return False
    
    def get_recommendations(self) -> List[str]:
        """Get AI-powered package recommendations based on system analysis"""
        recommendations = []
        
        status = self.check_system_status()
        
        if not status['nexuspkg']:
            recommendations.append("📦 Install NexusPkg universal package manager")
        
        if not status['awesome_stack']:
            recommendations.append("🚀 Mount awesome-stack repository for full features")
        
        if not status['digital_fortress']:
            recommendations.append("🛡️ Enable Digital Fortress for ultimate privacy")
        
        if not status['vaultwarden']:
            recommendations.append("🔐 Setup Vaultwarden for secure password management")
        
        if not status['docker']:
            recommendations.append("🐳 Install Docker for containerized services")
        
        # AI-based recommendations based on usage patterns
        if len(status['services_running']) == 0:
            recommendations.extend([
                "🎬 Setup media stack (Plex, Sonarr, Radarr)",
                "🏠 Install Home Assistant for smart home",
                "📊 Setup monitoring (Grafana, Prometheus)"
            ])
        
        return recommendations
    
    def interactive_mode(self):
        """Interactive assistant mode"""
        self.print_banner()
        
        while True:
            print("\n🤖 What can I help you with?")
            print("1. 📊 System Status")
            print("2. 📦 Install Package")
            print("3. 🎬 Setup Media Stack")
            print("4. 🛡️ Enable Digital Fortress")
            print("5. 🔐 Setup Vaultwarden")
            print("6. ⚡ Optimize System")
            print("7. 🎯 Get Recommendations")
            print("8. ⚙️ Configuration")
            print("9. ❌ Exit")
            
            try:
                choice = input("\n👉 Enter your choice (1-9): ").strip()
                
                if choice == '1':
                    self.show_system_status()
                elif choice == '2':
                    self.interactive_package_install()
                elif choice == '3':
                    self.setup_media_stack()
                elif choice == '4':
                    self.enable_digital_fortress()
                elif choice == '5':
                    self.setup_vaultwarden()
                elif choice == '6':
                    self.optimize_system()
                elif choice == '7':
                    self.show_recommendations()
                elif choice == '8':
                    self.configure_assistant()
                elif choice == '9':
                    print("👋 Goodbye!")
                    break
                else:
                    print("❌ Invalid choice. Please try again.")
                    
            except KeyboardInterrupt:
                print("\n👋 Goodbye!")
                break
            except Exception as e:
                print(f"❌ Error: {e}")
    
    def show_system_status(self):
        """Show detailed system status"""
        print("\n📊 System Status:")
        print("=" * 50)
        
        status = self.check_system_status()
        
        def status_icon(active: bool) -> str:
            return "✅" if active else "❌"
        
        print(f"{status_icon(status['nexuspkg'])} NexusPkg Universal Package Manager")
        print(f"{status_icon(status['awesome_stack'])} Awesome Stack Repository")
        print(f"{status_icon(status['digital_fortress'])} Digital Fortress Security")
        print(f"{status_icon(status['vaultwarden'])} Vaultwarden Password Manager")
        print(f"{status_icon(status['docker'])} Docker Container Runtime")
        
        if status['services_running']:
            print(f"\n🐳 Running Services ({len(status['services_running'])}):")
            for service in status['services_running'][:10]:  # Show max 10
                print(f"   • {service}")
            if len(status['services_running']) > 10:
                print(f"   ... and {len(status['services_running']) - 10} more")
    
    def interactive_package_install(self):
        """Interactive package installation"""
        package = input("📦 Package name or file: ").strip()
        if not package:
            print("❌ Package name required!")
            return
        
        print("\n📋 Available formats:")
        formats = ['auto'] + self.config['supported_formats']
        for i, fmt in enumerate(formats, 1):
            print(f"{i}. {fmt}")
        
        try:
            choice = input(f"\n👉 Select format (1-{len(formats)}, default=auto): ").strip()
            if choice and choice.isdigit():
                format_hint = formats[int(choice) - 1] if int(choice) <= len(formats) else 'auto'
            else:
                format_hint = 'auto'
            
            if format_hint == 'auto':
                format_hint = None
            
            self.install_package(package, format_hint)
            
        except (ValueError, IndexError):
            print("❌ Invalid choice, using auto-detection")
            self.install_package(package)
    
    def show_recommendations(self):
        """Show AI recommendations"""
        recommendations = self.get_recommendations()
        
        if recommendations:
            print("\n🎯 AI Recommendations:")
            print("=" * 50)
            for i, rec in enumerate(recommendations, 1):
                print(f"{i}. {rec}")
        else:
            print("\n✅ Your system is optimally configured!")
    
    def configure_assistant(self):
        """Configure assistant settings"""
        print("\n⚙️ Configuration:")
        print("=" * 30)
        print(f"1. AI Features: {'✅' if self.config['ai_features'] else '❌'}")
        print(f"2. Awesome Stack: {'✅' if self.config['awesome_stack_integration'] else '❌'}")
        print(f"3. Auto Optimize: {'✅' if self.config['auto_optimize'] else '❌'}")
        print(f"4. Tray Notifications: {'✅' if self.config['tray_notifications'] else '❌'}")
        print(f"5. Debug Mode: {'✅' if self.config['debug_mode'] else '❌'}")
        
        try:
            choice = input("\n👉 Toggle setting (1-5) or 0 to exit: ").strip()
            if choice == '1':
                self.config['ai_features'] = not self.config['ai_features']
            elif choice == '2':
                self.config['awesome_stack_integration'] = not self.config['awesome_stack_integration']
            elif choice == '3':
                self.config['auto_optimize'] = not self.config['auto_optimize']
            elif choice == '4':
                self.config['tray_notifications'] = not self.config['tray_notifications']
            elif choice == '5':
                self.config['debug_mode'] = not self.config['debug_mode']
            elif choice == '0':
                return
            else:
                print("❌ Invalid choice")
                return
                
            self.save_config()
            print("✅ Configuration saved!")
            
        except Exception as e:
            print(f"❌ Error: {e}")

def main():
    parser = argparse.ArgumentParser(description="NexusOS AI Assistant (Nexus-Rani)")
    parser.add_argument('--version', action='version', version=f'Nexus-Rani 1.0.0')
    parser.add_argument('--status', action='store_true', help='Show system status')
    parser.add_argument('--install', type=str, help='Install package')
    parser.add_argument('--format', type=str, help='Package format hint')
    parser.add_argument('--media-stack', action='store_true', help='Setup media stack')
    parser.add_argument('--digital-fortress', action='store_true', help='Enable Digital Fortress')
    parser.add_argument('--vaultwarden', action='store_true', help='Setup Vaultwarden')
    parser.add_argument('--optimize', action='store_true', help='Optimize system')
    parser.add_argument('--recommendations', action='store_true', help='Get AI recommendations')
    parser.add_argument('--daemon', action='store_true', help='Run as daemon service')
    
    args = parser.parse_args()
    
    rani = NexusRani()
    
    # Handle command line arguments
    if args.status:
        rani.show_system_status()
    elif args.install:
        rani.install_package(args.install, args.format)
    elif args.media_stack:
        rani.setup_media_stack()
    elif args.digital_fortress:
        rani.enable_digital_fortress()
    elif args.vaultwarden:
        rani.setup_vaultwarden()
    elif args.optimize:
        rani.optimize_system()
    elif args.recommendations:
        rani.show_recommendations()
    elif args.daemon:
        print("🤖 Starting Nexus-Rani daemon...")
        # TODO: Implement daemon mode
        print("📝 Daemon mode not yet implemented")
    else:
        # Interactive mode
        rani.interactive_mode()

if __name__ == "__main__":
    main()