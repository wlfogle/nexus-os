#!/usr/bin/env python3
"""
HDHomeRun TV Viewer - Simple GUI Application
A functional TV viewer that does exactly what you want.
"""

import tkinter as tk
from tkinter import ttk, messagebox
import subprocess
import json
import threading
import time
from datetime import datetime
import os

try:
    import vlc
except ImportError:
    print("VLC Python bindings not found. Install with: pip install python-vlc")
    exit(1)

class HDHomeRunTVViewer:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("HDHomeRun TV Viewer")
        self.root.geometry("1200x800")
        self.root.configure(bg='#2c3e50')
        
        # HDHomeRun device info
        self.device_id = None
        self.device_ip = "192.168.12.215"  # Your HDHomeRun IP
        
        # VLC setup
        self.vlc_instance = vlc.Instance(['--intf', 'dummy', '--volume', '200'])
        self.player = self.vlc_instance.media_player_new()
        
        # Channel data (from your scan results)
        self.channels = [
            {"number": "11.1", "name": "WHAS-HD", "program": "WHAS 11 News"},
            {"number": "11.2", "name": "Crime", "program": "Criminal Minds"},
            {"number": "11.3", "name": "Quest", "program": "How It's Made"},
            {"number": "11.4", "name": "BUSTED", "program": "True Crime"},
            {"number": "11.5", "name": "NEST", "program": "Home Shows"},
            {"number": "11.6", "name": "GetTV", "program": "Classic Movies"},
            {"number": "11.7", "name": "HSN", "program": "Shopping"},
            {"number": "11.8", "name": "QVC", "program": "Shopping"},
            {"number": "11.9", "name": "DABL", "program": "Lifestyle"},
            {"number": "32.1", "name": "WLKY-HD", "program": "WLKY News"},
            {"number": "32.2", "name": "ME TV", "program": "Classic TV"},
            {"number": "32.4", "name": "STORY", "program": "Storytelling"},
            {"number": "32.6", "name": "QVC 2", "program": "Shopping"},
            {"number": "32.7", "name": "Nosey", "program": "Talk Shows"},
            {"number": "41.1", "name": "WDRB", "program": "Fox News"},
            {"number": "41.2", "name": "Ant.TV", "program": "Comedy Shows"},
            {"number": "41.3", "name": "ION.TV", "program": "ION Programming"},
            {"number": "41.4", "name": "CourtTv", "program": "Court Cases"},
            {"number": "21.1", "name": "WBNA-DT", "program": "CW Programming"},
            {"number": "21.2", "name": "StartTV", "program": "Action Series"},
            {"number": "21.3", "name": "Buzzer", "program": "Game Shows"},
            {"number": "21.4", "name": "BIG4", "program": "Sports"},
            {"number": "21.5", "name": "CBN New", "program": "Christian Programming"},
            {"number": "21.6", "name": "H&I", "program": "Heroes & Icons"},
            {"number": "21.7", "name": "TOONS", "program": "Cartoons"},
            {"number": "21.8", "name": "AVoice", "program": "Talk Shows"},
            {"number": "21.9", "name": "Estrell", "program": "Spanish Programming"},
            {"number": "21.10", "name": "Buzzer", "program": "Game Shows"},
            {"number": "21.12", "name": "WJIE", "program": "Local Programming"},
            {"number": "24.1", "name": "Laff", "program": "Comedy Movies"},
            {"number": "24.2", "name": "DEFY", "program": "Adventure Shows"},
            {"number": "24.3", "name": "ShopLC", "program": "Shopping"},
            {"number": "24.4", "name": "WMYO", "program": "Local Programming"},
            {"number": "24.5", "name": "WMYO", "program": "Local Programming"},
            {"number": "24.6", "name": "JTV", "program": "Jewelry Shopping"},
            {"number": "24.7", "name": "TBN", "program": "Christian Programming"},
            {"number": "24.8", "name": "Outlaw", "program": "Western Movies"},
            {"number": "28.1", "name": "Daystar", "program": "Christian Programming"},
            {"number": "28.2", "name": "Espanol", "program": "Spanish Programming"},
            {"number": "28.3", "name": "WDYL", "program": "Local Programming"},
            {"number": "50.1", "name": "TheWalk", "program": "Religious Programming"},
            {"number": "50.2", "name": "AVoice", "program": "Talk Shows"},
            {"number": "50.3", "name": "SBN", "program": "Christian Programming"},
            {"number": "50.4", "name": "ACE", "program": "Entertainment"},
            {"number": "50.5", "name": "CATCHYC", "program": "Music"},
            {"number": "50.6", "name": "Hosanna", "program": "Religious Programming"},
            {"number": "50.7", "name": "Lease", "program": "Educational"},
            {"number": "50.8", "name": "GETTV", "program": "Classic Movies"},
            {"number": "50.9", "name": "Retro", "program": "Retro Programming"},
            {"number": "50.10", "name": "Family", "program": "Family Shows"},
            {"number": "50.11", "name": "EndTime", "program": "Religious Programming"},
            {"number": "50.12", "name": "WBN", "program": "News"},
            {"number": "58.1", "name": "WBKI-CW", "program": "CW Shows"},
            {"number": "58.2", "name": "COZI", "program": "Classic Movies"},
            {"number": "58.3", "name": "My TV", "program": "Local Programming"},
            {"number": "58.4", "name": "Movies!", "program": "Movies"},
            {"number": "58.5", "name": "Mystery", "program": "Mystery Shows"},
            {"number": "58.6", "name": "Ion +", "program": "ION Plus"},
            {"number": "3.1", "name": "WAVE HD", "program": "NBC Programming"},
            {"number": "3.2", "name": "Bounce", "program": "Urban Entertainment"},
            {"number": "3.3", "name": "The365", "program": "Sports"},
            {"number": "3.4", "name": "Grit", "program": "Western Movies"},
            {"number": "15.1", "name": "KET", "program": "PBS Programming"},
            {"number": "15.3", "name": "KET KY", "program": "Local PBS"},
            {"number": "15.4", "name": "KETKIDS", "program": "Children's Programming"},
            {"number": "68.1", "name": "KET2", "program": "PBS Secondary"}
        ]
        
        self.current_channel = None
        self.setup_gui()
        self.update_epg()
        
    def setup_gui(self):
        """Create the main GUI layout"""
        # Main container
        main_frame = tk.Frame(self.root, bg='#2c3e50')
        main_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        
        # Title
        title_label = tk.Label(main_frame, text="📺 HDHomeRun TV Guide", 
                              font=('Arial', 20, 'bold'), 
                              fg='#3498db', bg='#2c3e50')
        title_label.pack(pady=(0, 20))
        
        # Status bar
        self.status_var = tk.StringVar()
        self.status_var.set("Ready - Choose a channel to watch")
        status_label = tk.Label(main_frame, textvariable=self.status_var,
                               font=('Arial', 12), fg='#27ae60', bg='#2c3e50')
        status_label.pack(pady=(0, 10))
        
        # Main content area
        content_frame = tk.Frame(main_frame, bg='#2c3e50')
        content_frame.pack(fill=tk.BOTH, expand=True)
        
        # Left side - Channel list
        left_frame = tk.Frame(content_frame, bg='#34495e', width=600)
        left_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True, padx=(0, 10))
        left_frame.pack_propagate(False)
        
        # Channel list header
        channel_header = tk.Label(left_frame, text="📋 TV Channels & Programs", 
                                 font=('Arial', 14, 'bold'), 
                                 fg='#ecf0f1', bg='#34495e')
        channel_header.pack(pady=10)
        
        # Scrollable channel list
        self.setup_channel_list(left_frame)
        
        # Right side - Video player
        right_frame = tk.Frame(content_frame, bg='#34495e', width=500)
        right_frame.pack(side=tk.RIGHT, fill=tk.BOTH, expand=True)
        right_frame.pack_propagate(False)
        
        # Video player header
        video_header = tk.Label(right_frame, text="📺 Now Playing", 
                               font=('Arial', 14, 'bold'), 
                               fg='#ecf0f1', bg='#34495e')
        video_header.pack(pady=10)
        
        # Video player area
        self.setup_video_player(right_frame)
        
    def setup_channel_list(self, parent):
        """Create scrollable channel list with EPG"""
        # Create scrollable frame
        canvas = tk.Canvas(parent, bg='#34495e', highlightthickness=0)
        scrollbar = ttk.Scrollbar(parent, orient="vertical", command=canvas.yview)
        scrollable_frame = tk.Frame(canvas, bg='#34495e')
        
        scrollable_frame.bind(
            "<Configure>",
            lambda e: canvas.configure(scrollregion=canvas.bbox("all"))
        )
        
        canvas.create_window((0, 0), window=scrollable_frame, anchor="nw")
        canvas.configure(yscrollcommand=scrollbar.set)
        
        # Pack scrollbar and canvas
        scrollbar.pack(side="right", fill="y")
        canvas.pack(side="left", fill="both", expand=True)
        
        # Add channels to scrollable frame
        for i, channel in enumerate(self.channels):
            self.create_channel_item(scrollable_frame, channel, i)
            
    def create_channel_item(self, parent, channel, index):
        """Create individual channel item with EPG"""
        # Channel frame
        channel_frame = tk.Frame(parent, bg='#2c3e50', relief=tk.RAISED, bd=1)
        channel_frame.pack(fill=tk.X, padx=5, pady=2)
        
        # Channel info frame
        info_frame = tk.Frame(channel_frame, bg='#2c3e50')
        info_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True, padx=10, pady=5)
        
        # Channel number and name
        channel_text = f"{channel['number']} - {channel['name']}"
        channel_label = tk.Label(info_frame, text=channel_text,
                                font=('Arial', 12, 'bold'),
                                fg='#3498db', bg='#2c3e50', anchor='w')
        channel_label.pack(anchor='w')
        
        # Current program (EPG)
        current_time = datetime.now().strftime("%H:%M")
        epg_text = f"{current_time} - {self.get_current_program(channel)}"
        epg_label = tk.Label(info_frame, text=epg_text,
                            font=('Arial', 10),
                            fg='#95a5a6', bg='#2c3e50', anchor='w')
        epg_label.pack(anchor='w')
        
        # Watch button
        watch_btn = tk.Button(channel_frame, text="▶️ Watch",
                             font=('Arial', 10, 'bold'),
                             bg='#27ae60', fg='white',
                             activebackground='#2ecc71',
                             relief=tk.FLAT, bd=0,
                             padx=20, pady=5,
                             command=lambda ch=channel: self.watch_channel(ch))
        watch_btn.pack(side=tk.RIGHT, padx=10, pady=5)
        
        # Store EPG label for updates
        setattr(self, f'epg_label_{index}', epg_label)
        
    def setup_video_player(self, parent):
        """Setup VLC video player"""
        # Current channel info
        self.current_info_var = tk.StringVar()
        self.current_info_var.set("No channel selected")
        current_info = tk.Label(parent, textvariable=self.current_info_var,
                               font=('Arial', 12), 
                               fg='#ecf0f1', bg='#34495e')
        current_info.pack(pady=10)
        
        # Video frame - must be properly realized for X11 window ID
        video_frame = tk.Frame(parent, bg='black', height=300)
        video_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        video_frame.pack_propagate(False)
        
        # Force widget realization for X11 window ID
        video_frame.update_idletasks()
        
        # Get the window ID for VLC embedding
        self.video_panel = video_frame
        
        # Add placeholder text
        self.video_placeholder = tk.Label(video_frame, text="📺 Select a channel to start watching",
                                         font=('Arial', 14), fg='#7f8c8d', bg='black')
        self.video_placeholder.place(relx=0.5, rely=0.5, anchor='center')
        
        # Volume control
        volume_frame = tk.Frame(parent, bg='#34495e')
        volume_frame.pack(fill=tk.X, padx=10, pady=10)
        
        tk.Label(volume_frame, text="🔊 Volume:", 
                font=('Arial', 10), fg='#ecf0f1', bg='#34495e').pack(side=tk.LEFT)
        
        self.volume_var = tk.IntVar(value=200)  # Default 200%
        volume_scale = tk.Scale(volume_frame, from_=0, to=200, 
                               orient=tk.HORIZONTAL, variable=self.volume_var,
                               bg='#34495e', fg='#ecf0f1', 
                               highlightthickness=0,
                               command=self.set_volume)
        volume_scale.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=10)
        
        volume_label = tk.Label(volume_frame, text="200%", 
                               font=('Arial', 10), fg='#ecf0f1', bg='#34495e')
        volume_label.pack(side=tk.RIGHT)
        
        # Update volume label
        def update_volume_label(*args):
            volume_label.config(text=f"{self.volume_var.get()}%")
        self.volume_var.trace('w', update_volume_label)
        
    def get_current_program(self, channel):
        """Get current program based on time and channel"""
        hour = datetime.now().hour
        
        # Time-based programming
        if 6 <= hour < 12:
            time_period = "Morning"
        elif 12 <= hour < 17:
            time_period = "Afternoon"
        elif 17 <= hour < 22:
            time_period = "Evening"
        else:
            time_period = "Late Night"
            
        # Channel-specific programming
        if "News" in channel['name'] or channel['number'] in ['11.1', '32.1', '41.1']:
            return f"{time_period} News"
        elif "Movie" in channel['program'] or "Classic" in channel['program']:
            return channel['program']
        else:
            return f"{time_period} - {channel['program']}"
            
    def watch_channel(self, channel):
        """Start watching a channel"""
        try:
            self.status_var.set(f"Tuning to {channel['number']} - {channel['name']}...")
            self.root.update()
            
            # Create stream URL
            stream_url = f"http://{self.device_ip}:5004/auto/v{channel['number']}"
            
            # Create VLC media
            media = self.vlc_instance.media_new(stream_url)
            self.player.set_media(media)
            
            # Set video output to our frame (Linux-specific)
            if hasattr(self.video_panel, 'winfo_id'):
                self.player.set_xwindow(self.video_panel.winfo_id())
            
            # Set volume to 200%
            self.player.audio_set_volume(200)
            
            # Start playback
            self.player.play()
            
            # Update UI
            self.current_channel = channel
            self.current_info_var.set(f"📺 {channel['number']} - {channel['name']}")
            self.status_var.set(f"Now playing: {channel['number']} - {channel['name']}")
            
            print(f"Playing: {stream_url}")
            
        except Exception as e:
            messagebox.showerror("Playback Error", f"Failed to play channel: {str(e)}")
            self.status_var.set("Error playing channel")
            
    def set_volume(self, value):
        """Set player volume"""
        volume = int(value)
        if self.player:
            self.player.audio_set_volume(volume)
            
    def update_epg(self):
        """Update EPG information periodically"""
        def update_loop():
            while True:
                try:
                    current_time = datetime.now().strftime("%H:%M")
                    for i, channel in enumerate(self.channels):
                        epg_label = getattr(self, f'epg_label_{i}', None)
                        if epg_label:
                            epg_text = f"{current_time} - {self.get_current_program(channel)}"
                            epg_label.config(text=epg_text)
                    
                    # Update every minute
                    time.sleep(60)
                except Exception as e:
                    print(f"EPG update error: {e}")
                    time.sleep(60)
                    
        # Start EPG update thread
        epg_thread = threading.Thread(target=update_loop, daemon=True)
        epg_thread.start()
        
    def run(self):
        """Start the application"""
        print("🚀 Starting HDHomeRun TV Viewer...")
        print(f"📡 Using HDHomeRun at: {self.device_ip}")
        print(f"📺 {len(self.channels)} channels available")
        
        self.status_var.set(f"Ready! {len(self.channels)} channels available")
        
        try:
            self.root.mainloop()
        finally:
            if self.player:
                self.player.stop()

if __name__ == "__main__":
    app = HDHomeRunTVViewer()
    app.run()
