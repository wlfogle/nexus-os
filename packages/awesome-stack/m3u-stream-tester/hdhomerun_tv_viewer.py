#!/usr/bin/env python3
"""
HDHomeRun TV Viewer GUI Application
A native desktop GUI for watching HDHomeRun channels with embedded VLC player
"""

import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
import vlc
import threading
import time
import requests
import json
import os
import sys
from datetime import datetime
import subprocess

class HDHomeRunTVViewer:
    def __init__(self, root):
        self.root = root
        self.root.title("HDHomeRun TV Viewer")
        self.root.geometry("1200x800")
        self.root.configure(bg='#2b2b2b')
        
        # VLC instance and player
        self.vlc_instance = None
        self.vlc_player = None
        self.current_channel = None
        
        # Channel data
        self.channels = []
        self.epg_data = {}
        
        # HDHomeRun discovery
        self.hdhomerun_ip = None
        self.device_id = None
        
        # Setup GUI
        self.setup_gui()
        
        # Initialize VLC
        self.init_vlc()
        
        # Discover HDHomeRun device
        self.discover_hdhomerun()
        
        # Load channels
        self.load_channels()
        
        # Start EPG update thread
        self.epg_update_thread = threading.Thread(target=self.update_epg_loop, daemon=True)
        self.epg_update_thread.start()
        
        # Bind close event
        self.root.protocol("WM_DELETE_WINDOW", self.on_closing)

    def setup_gui(self):
        # Main frame
        main_frame = tk.Frame(self.root, bg='#2b2b2b')
        main_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        
        # Left pane - Channel list
        left_frame = tk.Frame(main_frame, bg='#3b3b3b', width=400)
        left_frame.pack(side=tk.LEFT, fill=tk.BOTH, padx=(0, 10))
        left_frame.pack_propagate(False)
        
        # Channel list title
        tk.Label(left_frame, text="Channels", font=('Arial', 14, 'bold'), 
                bg='#3b3b3b', fg='white').pack(pady=(10, 5))
        
        # Channel list with scrollbar
        list_frame = tk.Frame(left_frame, bg='#3b3b3b')
        list_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=(0, 10))
        
        self.channel_listbox = tk.Listbox(list_frame, bg='#4b4b4b', fg='white', 
                                         selectbackground='#007acc', font=('Arial', 10))
        scrollbar = tk.Scrollbar(list_frame, orient=tk.VERTICAL, command=self.channel_listbox.yview)
        self.channel_listbox.config(yscrollcommand=scrollbar.set)
        
        self.channel_listbox.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        
        # Watch button
        self.watch_button = tk.Button(left_frame, text="Watch Selected Channel", 
                                     command=self.watch_selected_channel,
                                     bg='#007acc', fg='white', font=('Arial', 12, 'bold'),
                                     pady=10)
        self.watch_button.pack(pady=(0, 10), padx=10, fill=tk.X)
        
        # Right pane - Video player
        right_frame = tk.Frame(main_frame, bg='#1b1b1b')
        right_frame.pack(side=tk.RIGHT, fill=tk.BOTH, expand=True)
        
        # Video frame
        video_frame = tk.Frame(right_frame, bg='#000000', height=480)
        video_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        video_frame.pack_propagate(False)
        
        # Video widget (for VLC embedding)
        self.video_widget = tk.Frame(video_frame, bg='#000000')
        self.video_widget.pack(fill=tk.BOTH, expand=True)
        
        # Controls frame
        controls_frame = tk.Frame(right_frame, bg='#2b2b2b')
        controls_frame.pack(fill=tk.X, padx=10, pady=(0, 10))
        
        # Volume control
        volume_frame = tk.Frame(controls_frame, bg='#2b2b2b')
        volume_frame.pack(fill=tk.X, pady=5)
        
        tk.Label(volume_frame, text="Volume:", bg='#2b2b2b', fg='white').pack(side=tk.LEFT)
        self.volume_scale = tk.Scale(volume_frame, from_=0, to=200, orient=tk.HORIZONTAL,
                                   command=self.set_volume, bg='#2b2b2b', fg='white',
                                   highlightbackground='#2b2b2b', troughcolor='#4b4b4b')
        self.volume_scale.set(200)  # Default to 200%
        self.volume_scale.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(5, 0))
        
        # Current channel info
        self.current_info_frame = tk.Frame(controls_frame, bg='#2b2b2b')
        self.current_info_frame.pack(fill=tk.X, pady=5)
        
        self.current_channel_label = tk.Label(self.current_info_frame, text="No channel selected",
                                            bg='#2b2b2b', fg='white', font=('Arial', 12, 'bold'))
        self.current_channel_label.pack(anchor=tk.W)
        
        self.current_program_label = tk.Label(self.current_info_frame, text="",
                                            bg='#2b2b2b', fg='#cccccc', font=('Arial', 10))
        self.current_program_label.pack(anchor=tk.W)
        
        # Status bar
        self.status_bar = tk.Label(self.root, text="Ready", bg='#1b1b1b', fg='white',
                                  anchor=tk.W, font=('Arial', 9))
        self.status_bar.pack(side=tk.BOTTOM, fill=tk.X)

    def init_vlc(self):
        """Initialize VLC media player"""
        try:
            # Create VLC instance
            vlc_args = [
                '--intf', 'dummy',
                '--no-video-title-show',
                '--network-caching=1000',
                '--no-osd'
            ]
            self.vlc_instance = vlc.Instance(vlc_args)
            self.vlc_player = self.vlc_instance.media_player_new()
            
            # Set the video widget as the output
            if sys.platform == "linux":
                self.vlc_player.set_xwindow(self.video_widget.winfo_id())
            
            self.update_status("VLC initialized successfully")
            
        except Exception as e:
            self.update_status(f"Error initializing VLC: {str(e)}")
            messagebox.showerror("VLC Error", f"Failed to initialize VLC: {str(e)}")

    def discover_hdhomerun(self):
        """Discover HDHomeRun device on network"""
        try:
            self.update_status("Discovering HDHomeRun device...")
            
            # Try to use hdhomerun_config discover
            result = subprocess.run(['hdhomerun_config', 'discover'], 
                                  capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0 and result.stdout:
                lines = result.stdout.strip().split('\n')
                for line in lines:
                    if 'device type' in line.lower() and 'tuner' in line.lower():
                        # Extract device ID and IP
                        parts = line.split()
                        if len(parts) >= 4:
                            self.device_id = parts[1]
                            self.hdhomerun_ip = parts[3]
                            self.update_status(f"Found HDHomeRun device: {self.device_id} at {self.hdhomerun_ip}")
                            return
            
            # Fallback: try common IP ranges
            self.update_status("Scanning for HDHomeRun devices...")
            import socket
            
            # Get local network
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            s.connect(("8.8.8.8", 80))
            local_ip = s.getsockname()[0]
            s.close()
            
            network_base = '.'.join(local_ip.split('.')[:-1])
            
            for i in range(1, 255):
                ip = f"{network_base}.{i}"
                try:
                    response = requests.get(f"http://{ip}/discover.json", timeout=2)
                    if response.status_code == 200:
                        data = response.json()
                        if 'DeviceID' in data:
                            self.device_id = data['DeviceID']
                            self.hdhomerun_ip = ip
                            self.update_status(f"Found HDHomeRun device: {self.device_id} at {self.hdhomerun_ip}")
                            return
                except:
                    continue
            
            self.update_status("No HDHomeRun device found")
            messagebox.showwarning("Device Not Found", "Could not find HDHomeRun device on network")
            
        except Exception as e:
            self.update_status(f"Error discovering HDHomeRun: {str(e)}")

    def load_channels(self):
        """Load channel lineup from HDHomeRun device"""
        if not self.hdhomerun_ip:
            return
            
        try:
            self.update_status("Loading channel lineup...")
            
            # Get channel lineup
            response = requests.get(f"http://{self.hdhomerun_ip}/lineup.json", timeout=10)
            if response.status_code == 200:
                lineup = response.json()
                
                self.channels = []
                for channel in lineup:
                    if channel.get('URL'):
                        self.channels.append({
                            'number': channel.get('GuideNumber', 'Unknown'),
                            'name': channel.get('GuideName', 'Unknown Channel'),
                            'url': channel.get('URL'),
                            'program': 'Loading...'
                        })
                
                self.update_channel_list()
                self.update_status(f"Loaded {len(self.channels)} channels")
                
            else:
                self.update_status("Failed to load channel lineup")
                
        except Exception as e:
            self.update_status(f"Error loading channels: {str(e)}")

    def update_channel_list(self):
        """Update the channel list display"""
        self.channel_listbox.delete(0, tk.END)
        
        for channel in self.channels:
            display_text = f"{channel['number']} - {channel['name']}"
            if channel['program'] != 'Loading...':
                display_text += f"\n    {channel['program']}"
            
            self.channel_listbox.insert(tk.END, display_text)

    def watch_selected_channel(self):
        """Watch the selected channel"""
        selection = self.channel_listbox.curselection()
        if not selection:
            messagebox.showwarning("No Selection", "Please select a channel to watch")
            return
            
        channel_index = selection[0]
        channel = self.channels[channel_index]
        
        self.watch_channel(channel)

    def watch_channel(self, channel):
        """Start watching a specific channel"""
        if not self.vlc_player:
            messagebox.showerror("VLC Error", "VLC player not initialized")
            return
            
        try:
            self.update_status(f"Starting channel {channel['number']} - {channel['name']}")
            
            # Stop current playback
            if self.vlc_player.is_playing():
                self.vlc_player.stop()
            
            # Create new media
            media = self.vlc_instance.media_new(channel['url'])
            self.vlc_player.set_media(media)
            
            # Set volume to 200%
            self.vlc_player.audio_set_volume(200)
            
            # Start playback
            self.vlc_player.play()
            
            # Update current channel info
            self.current_channel = channel
            self.current_channel_label.config(text=f"Channel {channel['number']} - {channel['name']}")
            self.current_program_label.config(text=channel['program'])
            
            self.update_status(f"Playing channel {channel['number']} - {channel['name']}")
            
        except Exception as e:
            self.update_status(f"Error playing channel: {str(e)}")
            messagebox.showerror("Playback Error", f"Failed to play channel: {str(e)}")

    def set_volume(self, value):
        """Set playback volume"""
        if self.vlc_player:
            volume = int(value)
            self.vlc_player.audio_set_volume(volume)
            self.update_status(f"Volume set to {volume}%")

    def update_epg_loop(self):
        """Background thread to update EPG data"""
        while True:
            try:
                self.update_epg()
                time.sleep(60)  # Update every minute
            except Exception as e:
                print(f"EPG update error: {e}")
                time.sleep(60)

    def update_epg(self):
        """Update Electronic Program Guide information"""
        if not self.hdhomerun_ip:
            return
            
        try:
            # Get current time
            current_time = datetime.now()
            
            # Update program info for each channel
            for channel in self.channels:
                try:
                    # Try to get EPG data from HDHomeRun
                    # Note: This is a simplified approach - real EPG would require more complex parsing
                    channel['program'] = f"Live TV - {current_time.strftime('%I:%M %p')}"
                    
                except Exception as e:
                    channel['program'] = "Program information unavailable"
            
            # Update UI on main thread
            self.root.after(0, self.update_channel_list)
            
            # Update current channel info if watching something
            if self.current_channel:
                for channel in self.channels:
                    if channel['number'] == self.current_channel['number']:
                        self.root.after(0, lambda: self.current_program_label.config(text=channel['program']))
                        break
                        
        except Exception as e:
            print(f"EPG update error: {e}")

    def update_status(self, message):
        """Update status bar message"""
        self.status_bar.config(text=message)
        self.root.update_idletasks()

    def on_closing(self):
        """Handle application closing"""
        try:
            if self.vlc_player:
                self.vlc_player.stop()
                self.vlc_player.release()
            
            if self.vlc_instance:
                self.vlc_instance.release()
                
        except Exception as e:
            print(f"Cleanup error: {e}")
        
        self.root.destroy()

def main():
    """Main application entry point"""
    # Check if VLC is available
    try:
        import vlc
    except ImportError:
        print("Error: python-vlc not installed. Please install it with:")
        print("yay -S python-vlc")
        sys.exit(1)
    
    # Create and run application
    root = tk.Tk()
    app = HDHomeRunTVViewer(root)
    root.mainloop()

if __name__ == "__main__":
    main()
