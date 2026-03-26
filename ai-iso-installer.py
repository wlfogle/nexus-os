#!/usr/bin/env python3
"""
AI NLP ISO Installer
Natural language OS search, download, and installation tool
"""

import os
import sys
import json
import subprocess
import requests
import re
import hashlib
from pathlib import Path
from urllib.parse import urlparse
import tkinter as tk
from tkinter import ttk, messagebox, filedialog
import threading
import time

class OSDatabase:
    """Database of downloadable operating systems"""
    
    def __init__(self):
        self.os_data = {
            "ubuntu": {
                "name": "Ubuntu",
                "versions": {
                    "24.04": "https://ubuntu.com/download/desktop/thank-you?version=24.04.1&architecture=amd64",
                    "22.04": "https://ubuntu.com/download/desktop/thank-you?version=22.04.3&architecture=amd64"
                },
                "keywords": ["ubuntu", "canonical", "linux desktop", "beginner linux"]
            },
            "debian": {
                "name": "Debian",
                "versions": {
                    "12": "https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/debian-12.7.0-amd64-netinst.iso",
                    "11": "https://cdimage.debian.org/debian-cd/11.11.0/amd64/iso-cd/debian-11.11.0-amd64-netinst.iso"
                },
                "keywords": ["debian", "stable linux", "server linux", "gnu linux"]
            },
            "fedora": {
                "name": "Fedora",
                "versions": {
                    "40": "https://download.fedoraproject.org/pub/fedora/linux/releases/40/Workstation/x86_64/iso/Fedora-Workstation-Live-x86_64-40-1.14.iso",
                    "39": "https://download.fedoraproject.org/pub/fedora/linux/releases/39/Workstation/x86_64/iso/Fedora-Workstation-Live-x86_64-39-1.5.iso"
                },
                "keywords": ["fedora", "red hat", "rpm", "cutting edge linux"]
            },
            "arch": {
                "name": "Arch Linux",
                "versions": {
                    "latest": "https://archlinux.org/download/"
                },
                "keywords": ["arch", "btw i use arch", "rolling release", "advanced linux"]
            },
            "kali": {
                "name": "Kali Linux",
                "versions": {
                    "2024.1": "https://cdimage.kali.org/kali-2024.1/kali-linux-2024.1-installer-amd64.iso"
                },
                "keywords": ["kali", "penetration testing", "hacking", "security", "ethical hacking"]
            },
            "windows": {
                "name": "Windows",
                "versions": {
                    "11": "https://www.microsoft.com/software-download/windows11",
                    "10": "https://www.microsoft.com/software-download/windows10"
                },
                "keywords": ["windows", "microsoft", "gaming", "office", "business"]
            },
            "garuda-dragonized": {
                "name": "Garuda Linux Dr460nized",
                "versions": {
                    "latest": "https://iso.garudalinux.org/iso/latest/garuda/dr460nized/"
                },
                "keywords": ["garuda", "garuda linux", "dr460nized", "gaming linux", "arch based", "beautiful linux", "customized linux", "kde plasma", "dragonized"]
            },
            "garuda-gaming": {
                "name": "Garuda Linux Gaming Edition",
                "versions": {
                    "latest": "https://iso.garudalinux.org/iso/latest/garuda/gaming/"
                },
                "keywords": ["garuda gaming", "gaming edition", "garuda linux gaming", "gaming linux", "performance gaming", "linux gaming", "gaming optimization"]
            },
            "garuda-gnome": {
                "name": "Garuda Linux GNOME",
                "versions": {
                    "latest": "https://iso.garudalinux.org/iso/latest/garuda/gnome/"
                },
                "keywords": ["garuda gnome", "gnome desktop", "garuda linux gnome", "clean desktop", "modern linux"]
            },
            "garuda-xfce": {
                "name": "Garuda Linux Xfce",
                "versions": {
                    "latest": "https://iso.garudalinux.org/iso/latest/garuda/xfce/"
                },
                "keywords": ["garuda xfce", "lightweight garuda", "xfce desktop", "efficient linux", "low resource"]
            },
            "garuda-cinnamon": {
                "name": "Garuda Linux Cinnamon",
                "versions": {
                    "latest": "https://iso.garudalinux.org/iso/latest/garuda/cinnamon/"
                },
                "keywords": ["garuda cinnamon", "cinnamon desktop", "familiar interface", "traditional desktop"]
            },
            "garuda-sway": {
                "name": "Garuda Linux Sway",
                "versions": {
                    "latest": "https://iso.garudalinux.org/iso/latest/garuda/sway/"
                },
                "keywords": ["garuda sway", "wayland compositor", "tiling window manager", "minimal desktop", "keyboard driven"]
            },
            "ai-powerhouse": {
                "name": "AI Powerhouse Garuda",
                "versions": {
                    "latest": "https://github.com/wlfogle/ai-powerhouse-setup"
                },
                "keywords": ["ai powerhouse", "development environment", "zfs", "cuda", "rust", "tauri", "react", "self-hosting", "virtualization", "ai development", "machine learning", "neural networks"]
            }
        }

    def search_os(self, query):
        """Natural language search for operating systems"""
        query = query.lower()
        matches = []
        
        for os_id, data in self.os_data.items():
            score = 0
            
            # Check direct name match
            if os_id in query or data["name"].lower() in query:
                score += 10
            
            # Check keyword matches
            for keyword in data["keywords"]:
                if keyword in query:
                    score += 5
            
            # Check use case matches
            # Gaming prioritization - Garuda Gaming gets highest score
            if any(word in query for word in ["gaming", "game", "games"]):
                if "garuda-gaming" in os_id:
                    score += 15  # Highest priority for Garuda Gaming
                elif "garuda" in os_id:
                    score += 12  # High priority for other Garuda editions
                elif "windows" in os_id:
                    score += 3   # Lower priority for Windows
            
            # Linux desktop prioritization
            if any(word in query for word in ["linux", "arch", "rolling"]):
                if "garuda" in os_id:
                    score += 10
                elif "arch" in os_id:
                    score += 8
            
            # Desktop environment specific
            if any(word in query for word in ["kde", "plasma", "dr460nized", "dragonized"]):
                if "garuda-dragonized" in os_id or "garuda-gaming" in os_id:
                    score += 12
            
            # Other use cases
            if any(word in query for word in ["work", "office"]) and "windows" in os_id:
                score += 3
            if any(word in query for word in ["server", "stable"]) and "debian" in os_id:
                score += 3
            if any(word in query for word in ["security", "hack", "pen"]) and "kali" in os_id:
                score += 5
            if any(word in query for word in ["beginner", "easy"]) and "ubuntu" in os_id:
                score += 3
            if any(word in query for word in ["ai", "development", "powerhouse", "zfs", "cuda", "rust", "ml", "neural"]) and "ai-powerhouse" in os_id:
                score += 8
            
            if score > 0:
                matches.append((os_id, data, score))
        
        # Sort by relevance score
        matches.sort(key=lambda x: x[2], reverse=True)
        return matches

class DiskManager:
    """Handle disk operations and partition management"""
    
    @staticmethod
    def get_available_disks():
        """Get list of available disks with enhanced information"""
        try:
            # Get basic disk info
            result = subprocess.run(['lsblk', '-J', '-o', 'NAME,SIZE,TYPE,MOUNTPOINT,TRAN,MODEL'], 
                                   capture_output=True, text=True)
            data = json.loads(result.stdout)
            
            disks = []
            for device in data['blockdevices']:
                disk_info = {
                    'name': device['name'],
                    'size': device['size'],
                    'type': device['type'],
                    'mountpoint': device.get('mountpoint'),
                    'path': f"/dev/{device['name']}",
                    'transport': device.get('tran', ''),
                    'model': device.get('model', '')
                }
                disks.append(disk_info)
                
                # Add partitions (still collected but filtered out later)
                if 'children' in device:
                    for child in device['children']:
                        part_info = {
                            'name': child['name'],
                            'size': child['size'],
                            'type': child['type'],
                            'mountpoint': child.get('mountpoint'),
                            'path': f"/dev/{child['name']}",
                            'transport': device.get('tran', ''),
                            'model': device.get('model', '')
                        }
                        disks.append(part_info)
            
            return disks
        except Exception as e:
            print(f"Error getting disk info: {e}")
            return []

    @staticmethod
    def create_bootable_usb(iso_path, device_path):
        """Create bootable USB from ISO"""
        try:
            # Unmount if mounted
            subprocess.run(['sudo', 'umount', device_path], capture_output=True)
            
            # Use dd to write ISO
            cmd = [
                'sudo', 'dd',
                f'if={iso_path}',
                f'of={device_path}',
                'bs=4M',
                'status=progress'
            ]
            
            process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            
            return process
            
        except Exception as e:
            raise Exception(f"Failed to create bootable USB: {e}")

class AIPowerhouseInstaller:
    """Handle AI Powerhouse Setup installation"""
    
    @staticmethod
    def clone_and_setup():
        """Clone AI Powerhouse Setup repository and prepare installation"""
        try:
            install_dir = Path.home() / "Downloads" / "ai-powerhouse-setup"
            
            # Clone repository if not exists
            if not install_dir.exists():
                subprocess.run([
                    'git', 'clone', 
                    'https://github.com/wlfogle/ai-powerhouse-setup.git',
                    str(install_dir)
                ], check=True)
            else:
                # Update existing repository
                subprocess.run(['git', 'pull'], cwd=str(install_dir), check=True)
            
            return str(install_dir)
            
        except subprocess.CalledProcessError as e:
            raise Exception(f"Failed to clone AI Powerhouse Setup: {e}")
    
    @staticmethod
    def build_custom_iso(progress_callback=None):
        """Build custom AI Powerhouse ISO"""
        try:
            install_dir = AIPowerhouseInstaller.clone_and_setup()
            
            # Execute the build script
            build_script = Path(install_dir) / "installation" / "build-custom-iso.sh"
            
            if not build_script.exists():
                raise Exception("Build script not found in AI Powerhouse Setup")
            
            # Make script executable
            subprocess.run(['chmod', '+x', str(build_script)], check=True)
            
            # Run build script with sudo
            process = subprocess.Popen(
                ['sudo', str(build_script)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                cwd=str(install_dir)
            )
            
            return process
            
        except Exception as e:
            raise Exception(f"Failed to build AI Powerhouse ISO: {e}")
    
    @staticmethod
    def get_built_iso_path():
        """Get path to built AI Powerhouse ISO"""
        # Common locations where the ISO might be built
        possible_paths = [
            Path.home() / "Downloads" / "ai-powerhouse-setup" / "build" / "ai-powerhouse-garuda.iso",
            Path("/tmp") / "ai-powerhouse-garuda.iso",
            Path("./build") / "ai-powerhouse-garuda.iso"
        ]
        
        for path in possible_paths:
            if path.exists():
                return str(path)
        
        return None

class DownloadManager:
    """Handle ISO downloads with progress tracking"""
    
    def __init__(self, progress_callback=None):
        self.progress_callback = progress_callback
        self.cancelled = False
    
    def download_file(self, url, destination):
        """Download file with progress tracking"""
        try:
            response = requests.get(url, stream=True)
            response.raise_for_status()
            
            total_size = int(response.headers.get('content-length', 0))
            downloaded_size = 0
            
            with open(destination, 'wb') as f:
                for chunk in response.iter_content(chunk_size=8192):
                    if self.cancelled:
                        break
                        
                    if chunk:
                        f.write(chunk)
                        downloaded_size += len(chunk)
                        
                        if self.progress_callback and total_size > 0:
                            progress = (downloaded_size / total_size) * 100
                            self.progress_callback(progress)
            
            return not self.cancelled
            
        except Exception as e:
            raise Exception(f"Download failed: {e}")
    
    def cancel(self):
        """Cancel current download"""
        self.cancelled = True

class AIISOInstaller:
    """Main AI ISO Installer application"""
    
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("AI NLP ISO Installer")
        self.root.geometry("800x600")
        
        self.os_db = OSDatabase()
        self.disk_manager = DiskManager()
        self.download_manager = None
        
        self.setup_ui()
        
    def setup_ui(self):
        """Setup the user interface"""
        # Main frame
        main_frame = ttk.Frame(self.root, padding="10")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Search frame
        search_frame = ttk.LabelFrame(main_frame, text="What do you want to install?", padding="10")
        search_frame.grid(row=0, column=0, columnspan=2, sticky=(tk.W, tk.E), pady=(0, 10))
        
        self.search_var = tk.StringVar()
        search_entry = ttk.Entry(search_frame, textvariable=self.search_var, font=("Arial", 12))
        search_entry.grid(row=0, column=0, sticky=(tk.W, tk.E), padx=(0, 10))
        search_entry.bind('<Return>', self.search_os)
        
        search_btn = ttk.Button(search_frame, text="Search", command=self.search_os)
        search_btn.grid(row=0, column=1)
        
        search_frame.columnconfigure(0, weight=1)
        
        # Example queries
        examples = ttk.Label(search_frame, 
                           text="Examples: 'Ubuntu for beginners', 'Garuda gaming', 'Kali for pentesting', 'AI Powerhouse development'",
                           font=("Arial", 9), foreground="gray")
        examples.grid(row=1, column=0, columnspan=2, sticky=tk.W, pady=(5, 0))
        
        # Local ISO file selection
        local_frame = ttk.LabelFrame(main_frame, text="Or select local ISO file", padding="10")
        local_frame.grid(row=1, column=0, columnspan=2, sticky=(tk.W, tk.E), pady=(0, 10))
        
        self.local_iso_var = tk.StringVar()
        local_iso_entry = ttk.Entry(local_frame, textvariable=self.local_iso_var, font=("Arial", 10))
        local_iso_entry.grid(row=0, column=0, sticky=(tk.W, tk.E), padx=(0, 10))
        
        browse_btn = ttk.Button(local_frame, text="Browse ISO...", command=self.browse_local_iso)
        browse_btn.grid(row=0, column=1)
        
        local_frame.columnconfigure(0, weight=1)
        
        # Results frame
        results_frame = ttk.LabelFrame(main_frame, text="Search Results", padding="10")
        results_frame.grid(row=2, column=0, columnspan=2, sticky=(tk.W, tk.E, tk.N, tk.S), pady=(0, 10))
        
        # Results treeview
        columns = ('OS', 'Version', 'Match')
        self.results_tree = ttk.Treeview(results_frame, columns=columns, show='headings', height=8)
        
        self.results_tree.heading('OS', text='Operating System')
        self.results_tree.heading('Version', text='Available Versions')
        self.results_tree.heading('Match', text='Relevance')
        
        self.results_tree.column('OS', width=200)
        self.results_tree.column('Version', width=150)
        self.results_tree.column('Match', width=100)
        
        self.results_tree.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Scrollbar for results
        scrollbar = ttk.Scrollbar(results_frame, orient=tk.VERTICAL, command=self.results_tree.yview)
        scrollbar.grid(row=0, column=1, sticky=(tk.N, tk.S))
        self.results_tree.configure(yscrollcommand=scrollbar.set)
        
        results_frame.columnconfigure(0, weight=1)
        results_frame.rowconfigure(0, weight=1)
        
        # Installation type frame
        install_type_frame = ttk.LabelFrame(main_frame, text="Installation Type", padding="10")
        install_type_frame.grid(row=3, column=0, columnspan=2, sticky=(tk.W, tk.E), pady=(0, 10))
        
        self.install_type_var = tk.StringVar(value="live")
        
        live_radio = ttk.Radiobutton(install_type_frame, text="Create Live USB (boot from ISO)", 
                                    variable=self.install_type_var, value="live")
        live_radio.grid(row=0, column=0, sticky=tk.W, pady=(0, 5))
        
        install_radio = ttk.Radiobutton(install_type_frame, text="Install OS to USB (persistent installation)", 
                                       variable=self.install_type_var, value="install")
        install_radio.grid(row=1, column=0, sticky=tk.W)
        
        # Download frame
        download_frame = ttk.LabelFrame(main_frame, text="Target Device", padding="10")
        download_frame.grid(row=4, column=0, columnspan=2, sticky=(tk.W, tk.E), pady=(0, 10))
        
        # Target disk selection
        ttk.Label(download_frame, text="Target Device:").grid(row=0, column=0, sticky=tk.W, padx=(0, 10))
        
        self.disk_var = tk.StringVar()
        self.disk_combo = ttk.Combobox(download_frame, textvariable=self.disk_var, state="readonly")
        self.disk_combo.grid(row=0, column=1, sticky=(tk.W, tk.E), padx=(0, 10))
        
        refresh_btn = ttk.Button(download_frame, text="Refresh", command=self.refresh_disks)
        refresh_btn.grid(row=0, column=2)
        
        # Progress bar
        self.progress_var = tk.DoubleVar()
        self.progress_bar = ttk.Progressbar(download_frame, variable=self.progress_var, maximum=100)
        self.progress_bar.grid(row=1, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(10, 0))
        
        # Status label
        self.status_var = tk.StringVar(value="Ready to search and install")
        status_label = ttk.Label(download_frame, textvariable=self.status_var)
        status_label.grid(row=2, column=0, columnspan=3, sticky=tk.W, pady=(5, 0))
        
        download_frame.columnconfigure(1, weight=1)
        
        # Action buttons
        button_frame = ttk.Frame(main_frame)
        button_frame.grid(row=5, column=0, columnspan=2, sticky=(tk.W, tk.E))
        
        self.download_btn = ttk.Button(button_frame, text="Create Bootable USB", command=self.start_installation)
        self.download_btn.grid(row=0, column=0, padx=(0, 10))
        self.download_btn.configure(state='disabled')
        
        self.cancel_btn = ttk.Button(button_frame, text="Cancel", command=self.cancel_operation)
        self.cancel_btn.grid(row=0, column=1, padx=(0, 10))
        self.cancel_btn.configure(state='disabled')
        
        test_btn = ttk.Button(button_frame, text="Test with QEMU", command=self.test_with_qemu)
        test_btn.grid(row=0, column=2)
        
        main_frame.columnconfigure(0, weight=1)
        main_frame.rowconfigure(2, weight=1)
        
        # Initialize
        self.refresh_disks()
        
    def search_os(self, event=None):
        """Search for operating systems based on natural language query"""
        query = self.search_var.get().strip()
        if not query:
            return
        
        self.status_var.set(f"Searching for: {query}")
        
        # Clear previous results
        for item in self.results_tree.get_children():
            self.results_tree.delete(item)
        
        # Search database
        matches = self.os_db.search_os(query)
        
        if not matches:
            self.status_var.set("No matches found. Try different keywords.")
            return
        
        # Populate results
        for os_id, data, score in matches:
            versions = ", ".join(data["versions"].keys())
            match_percent = min(100, score * 10)
            
            self.results_tree.insert('', 'end', values=(data["name"], versions, f"{match_percent}%"))
        
        self.status_var.set(f"Found {len(matches)} matches")
        self.download_btn.configure(state='normal')
        
    def browse_local_iso(self):
        """Browse for local ISO file"""
        filetypes = [
            ('ISO files', '*.iso'),
            ('All files', '*.*')
        ]
        
        filename = filedialog.askopenfilename(
            title="Select ISO file",
            filetypes=filetypes,
            initialdir=str(Path.home() / "Downloads")
        )
        
        if filename:
            self.local_iso_var.set(filename)
            # Clear search results when local ISO is selected
            for item in self.results_tree.get_children():
                self.results_tree.delete(item)
            
            # Add local ISO to results
            iso_name = Path(filename).stem
            self.results_tree.insert('', 'end', values=(f"Local: {iso_name}", "Selected", "100%"))
            
            self.status_var.set(f"Selected local ISO: {Path(filename).name}")
            self.download_btn.configure(state='normal')
    
    def refresh_disks(self):
        """Refresh available disk list with enhanced information"""
        disks = self.disk_manager.get_available_disks()
        disk_options = []
        
        for disk in disks:
            # Only show whole disks (not partitions) for bootable USB creation
            if disk['type'] == 'disk':
                # Build descriptive label
                label_parts = [disk['path'], f"({disk['size']})"]
                
                # Add transport type if available
                if disk.get('transport'):
                    if disk['transport'] == 'usb':
                        label_parts.append('[USB]')
                    elif disk['transport'] == 'sata':
                        label_parts.append('[SATA]')
                    elif disk['transport'] == 'nvme':
                        label_parts.append('[NVMe]')
                    else:
                        label_parts.append(f"[{disk['transport'].upper()}]")
                
                # Add model if available
                if disk.get('model') and disk['model'].strip():
                    label_parts.append(f"- {disk['model'].strip()}")
                
                # Mark if disk has mounted partitions (warning)
                has_mounted = False
                if 'children' in disks or any(d['name'].startswith(disk['name']) and d.get('mountpoint') 
                                              for d in disks if d['type'] == 'part'):
                    # Check if any partition of this disk is mounted
                    for other_disk in disks:
                        if (other_disk['name'].startswith(disk['name']) and 
                            other_disk['type'] == 'part' and 
                            other_disk.get('mountpoint')):
                            has_mounted = True
                            break
                
                if has_mounted:
                    label_parts.append('⚠️ HAS MOUNTED PARTITIONS')
                
                disk_options.append(' '.join(label_parts))
        
        # Sort USB devices first for convenience
        disk_options.sort(key=lambda x: (0 if '[USB]' in x else 1, x))
        
        self.disk_combo['values'] = disk_options
        if disk_options:
            self.disk_combo.current(0)
    
    def start_installation(self):
        """Start the download and installation process"""
        # Check for local ISO first
        local_iso_path = self.local_iso_var.get().strip()
        if local_iso_path:
            # Local ISO installation
            target_disk = self.disk_var.get()
            if not target_disk:
                messagebox.showwarning("Warning", "Please select a target device")
                return
            
            # Validate ISO file exists
            if not Path(local_iso_path).exists():
                messagebox.showerror("Error", "Selected ISO file does not exist")
                return
            
            # Extract device path
            device_path = target_disk.split(' ')[0]
            
            # Confirm installation
            iso_name = Path(local_iso_path).name
            if not messagebox.askyesno("Confirm Installation", 
                                     f"Install {iso_name} to {device_path}?\nThis will ERASE all data on the device.\nContinue?"):
                return
            
            # Start installation with local ISO
            threading.Thread(target=self.install_local_iso, 
                            args=(local_iso_path, device_path), daemon=True).start()
            return
        
        # Standard online installation
        selection = self.results_tree.selection()
        if not selection:
            messagebox.showwarning("Warning", "Please select an operating system or browse for a local ISO file")
            return
        
        target_disk = self.disk_var.get()
        if not target_disk:
            messagebox.showwarning("Warning", "Please select a target device")
            return
        
        # Extract device path
        device_path = target_disk.split(' ')[0]
        
        # Confirm installation
        if not messagebox.askyesno("Confirm Installation", 
                                 f"This will ERASE all data on {device_path}.\nContinue?"):
            return
        
        # Get selected OS info
        item = self.results_tree.item(selection[0])
        os_name = item['values'][0]
        
        # Start download in separate thread
        threading.Thread(target=self.download_and_install, 
                        args=(os_name, device_path), daemon=True).start()
    
    def install_local_iso(self, iso_path, device_path):
        """Install local ISO file to USB device"""
        try:
            self.download_btn.configure(state='disabled')
            self.cancel_btn.configure(state='normal')
            
            iso_name = Path(iso_path).name
            self.status_var.set(f"Installing {iso_name} to {device_path}...")
            self.progress_var.set(0)
            
            # Create bootable USB directly from local ISO
            process = self.disk_manager.create_bootable_usb(iso_path, device_path)
            
            # Monitor dd progress (simplified)
            while process.poll() is None:
                time.sleep(1)
                current_progress = self.progress_var.get()
                if current_progress < 95:
                    self.progress_var.set(current_progress + 2)
            
            if process.returncode == 0:
                self.status_var.set(f"Successfully installed {iso_name} to {device_path}")
                self.progress_var.set(100)
                messagebox.showinfo("Success", 
                                  f"Bootable USB created successfully!\n"
                                  f"ISO: {iso_name}\n"
                                  f"Device: {device_path}")
            else:
                stderr_output = process.stderr.read() if process.stderr else "Unknown error"
                raise Exception(f"USB creation failed: {stderr_output}")
                
        except Exception as e:
            self.status_var.set(f"Error: {e}")
            messagebox.showerror("Error", f"Failed to install local ISO: {str(e)}")
        finally:
            self.download_btn.configure(state='normal')
            self.cancel_btn.configure(state='disabled')
    
    def download_and_install(self, os_name, device_path):
        """Download ISO and create bootable USB"""
        try:
            self.download_btn.configure(state='disabled')
            self.cancel_btn.configure(state='normal')
            
            # Find OS data
            os_data = None
            for os_id, data in self.os_db.os_data.items():
                if data['name'] == os_name:
                    os_data = data
                    break
            
            if not os_data:
                raise Exception("OS data not found")
            
            # Get latest version URL
            latest_version = list(os_data['versions'].keys())[0]
            download_url = os_data['versions'][latest_version]
            
            # Special handling for AI Powerhouse Setup
            if os_name == "AI Powerhouse Garuda":
                self.status_var.set("Setting up AI Powerhouse Build Environment...")
                self.progress_var.set(10)
                
                # Clone and setup AI Powerhouse
                install_dir = AIPowerhouseInstaller.clone_and_setup()
                self.progress_var.set(30)
                
                self.status_var.set("Building Custom AI Powerhouse ISO...")
                
                # Build custom ISO
                build_process = AIPowerhouseInstaller.build_custom_iso()
                
                # Monitor build progress
                while build_process.poll() is None:
                    time.sleep(2)
                    current_progress = self.progress_var.get()
                    if current_progress < 85:
                        self.progress_var.set(current_progress + 1)
                
                if build_process.returncode != 0:
                    stderr = build_process.stderr.read()
                    raise Exception(f"AI Powerhouse ISO build failed: {stderr}")
                
                self.progress_var.set(90)
                
                # Find the built ISO
                iso_path = AIPowerhouseInstaller.get_built_iso_path()
                if not iso_path:
                    raise Exception("Could not find built AI Powerhouse ISO")
                
                self.status_var.set("AI Powerhouse ISO built successfully!")
                
            else:
                # Standard ISO download process
                download_dir = Path.home() / "Downloads" / "ai-iso-installer"
                download_dir.mkdir(exist_ok=True)
                
                iso_filename = f"{os_name.replace(' ', '_')}_{latest_version}.iso"
                iso_path = download_dir / iso_filename
                
                self.status_var.set(f"Downloading {os_name} {latest_version}...")
                
                # Download ISO
                self.download_manager = DownloadManager(self.update_progress)
                success = self.download_manager.download_file(download_url, str(iso_path))
                
                if not success:
                    self.status_var.set("Download cancelled")
                    return
            
            self.status_var.set("Creating bootable USB...")
            self.progress_var.set(0)
            
            # Create bootable USB
            process = self.disk_manager.create_bootable_usb(str(iso_path), device_path)
            
            # Monitor dd progress (simplified)
            while process.poll() is None:
                time.sleep(1)
                self.progress_var.set(self.progress_var.get() + 1)
                if self.progress_var.get() >= 95:
                    self.progress_var.set(95)
            
            if process.returncode == 0:
                self.status_var.set(f"Successfully installed {os_name} to {device_path}")
                self.progress_var.set(100)
                messagebox.showinfo("Success", f"Bootable USB created successfully!\nDevice: {device_path}")
            else:
                raise Exception("USB creation failed")
                
        except Exception as e:
            self.status_var.set(f"Error: {e}")
            messagebox.showerror("Error", str(e))
        finally:
            self.download_btn.configure(state='normal')
            self.cancel_btn.configure(state='disabled')
    
    def update_progress(self, progress):
        """Update progress bar"""
        self.progress_var.set(progress)
    
    def cancel_operation(self):
        """Cancel current operation"""
        if self.download_manager:
            self.download_manager.cancel()
        self.status_var.set("Operation cancelled")
        self.cancel_btn.configure(state='disabled')
    
    def test_with_qemu(self):
        """Test USB with QEMU (requires MobaLiveCD Linux)"""
        target_disk = self.disk_var.get()
        if not target_disk:
            messagebox.showwarning("Warning", "Please select a device to test")
            return
        
        device_path = target_disk.split(' ')[0]
        
        try:
            # Launch QEMU to test the USB device
            cmd = [
                'qemu-system-x86_64',
                '-m', '2048',
                '-boot', 'order=d',
                '-drive', f'file={device_path},format=raw,if=ide',
                '-enable-kvm'
            ]
            
            self.status_var.set(f"Testing {device_path} with QEMU...")
            subprocess.Popen(cmd)
            
        except Exception as e:
            messagebox.showerror("Error", f"Failed to launch QEMU: {e}")
    
    def run(self):
        """Start the application"""
        self.root.mainloop()

def main():
    """Main entry point"""
    if os.geteuid() != 0:
        print("This application requires root privileges for disk operations.")
        print("Please run with: sudo python3 ai-iso-installer.py")
        sys.exit(1)
    
    app = AIISOInstaller()
    app.run()

if __name__ == "__main__":
    main()
