#!/usr/bin/env python3
"""
HDHomeRun TV Viewer - Embedded VLC Version
Uses python-vlc bindings to embed VLC player internally
"""

import tkinter as tk
from tkinter import ttk, messagebox
import subprocess
import threading
import webbrowser
from datetime import datetime
import os
import sys
import vlc
import platform

class SimpleHDHomeRunTVViewer:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("HDHomeRun TV Viewer")
        self.root.geometry("1200x800")
        self.root.configure(bg='#2c3e50')
        
        # HDHomeRun device info
        self.device_ip = "192.168.12.215"
        
        # VLC setup with improved initialization
        vlc_args = [
            '--no-xlib',
            '--intf', 'dummy',
            '--quiet',
            '--no-video-title-show',
            '--network-caching=1000'
        ]
        self.vlc_instance = vlc.Instance(vlc_args)
        self.media_player = self.vlc_instance.media_player_new()
        # Set initial volume to 80%
        self.media_player.audio_set_volume(80)
        self.current_media = None
        self.is_fullscreen = False
        self.fullscreen_window = None
        
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
        
        self.setup_gui()
        self.update_epg()
        
    def setup_gui(self):
        """Create the main GUI layout"""
        # Title
        title_frame = tk.Frame(self.root, bg='#2c3e50')
        title_frame.pack(fill=tk.X, padx=10, pady=10)
        
        title_label = tk.Label(title_frame, text="📺 HDHomeRun TV Guide", 
                              font=('Arial', 18, 'bold'), 
                              fg='#3498db', bg='#2c3e50')
        title_label.pack()
        
        # Status
        self.status_var = tk.StringVar()
        self.status_var.set(f"Ready! {len(self.channels)} channels available")
        status_label = tk.Label(title_frame, textvariable=self.status_var,
                               font=('Arial', 10), fg='#27ae60', bg='#2c3e50')
        status_label.pack()
        
        # Main frame with two panels
        main_frame = tk.Frame(self.root, bg='#2c3e50')
        main_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        
        # Left panel - Channel list (fixed width)
        left_panel = tk.Frame(main_frame, bg='#2c3e50', width=400)
        left_panel.pack(side=tk.LEFT, fill=tk.Y, padx=(0, 10))
        left_panel.pack_propagate(False)  # Keep fixed width
        
        # Right panel - Video player (expandable)
        right_panel = tk.Frame(main_frame, bg='#1a1a1a', relief=tk.RAISED, bd=2)
        right_panel.pack(side=tk.RIGHT, fill=tk.BOTH, expand=True)
        
        # Video frame
        self.video_frame = tk.Frame(right_panel, bg='#000000')
        self.video_frame.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)
        
        # Placeholder text
        self.placeholder_label = tk.Label(self.video_frame, 
                                        text="📺 Select a channel to start watching\n\nClick 'Watch' button to play a channel\n\nUse volume control and fullscreen toggle below",
                                        font=('Arial', 16),
                                        fg='#95a5a6', bg='#000000',
                                        justify=tk.CENTER)
        self.placeholder_label.place(relx=0.5, rely=0.5, anchor=tk.CENTER)
        
        # Video controls
        controls_frame = tk.Frame(right_panel, bg='#2c3e50')
        controls_frame.pack(fill=tk.X, padx=5, pady=5)
        
        # Control buttons
        self.stop_btn = tk.Button(controls_frame, text="⏹️ Stop",
                                 font=('Arial', 10, 'bold'),
                                 bg='#e74c3c', fg='white',
                                 activebackground='#c0392b',
                                 relief=tk.FLAT, bd=0,
                                 padx=10, pady=5,
                                 command=self.stop_playback)
        self.stop_btn.pack(side=tk.LEFT, padx=5)
        
        # Fullscreen toggle button
        self.fullscreen_btn = tk.Button(controls_frame, text="🔳 Fullscreen",
                                       font=('Arial', 10, 'bold'),
                                       bg='#3498db', fg='white',
                                       activebackground='#2980b9',
                                       relief=tk.FLAT, bd=0,
                                       padx=10, pady=5,
                                       command=self.toggle_fullscreen)
        self.fullscreen_btn.pack(side=tk.LEFT, padx=5)
        
        # Volume control
        volume_frame = tk.Frame(controls_frame, bg='#2c3e50')
        volume_frame.pack(side=tk.RIGHT, padx=5)
        
        tk.Label(volume_frame, text="Volume:", bg='#2c3e50', fg='white').pack(side=tk.LEFT)
        self.volume_var = tk.IntVar(value=80)
        self.volume_scale = tk.Scale(volume_frame, from_=0, to=200, orient=tk.HORIZONTAL,
                                   variable=self.volume_var, bg='#2c3e50', fg='white',
                                   highlightthickness=0, length=150,
                                   command=self.update_volume)
        self.volume_scale.pack(side=tk.LEFT, padx=5)
        
        # Channel list
        self.setup_channel_list(left_panel)
        
    def setup_channel_list(self, parent):
        """Create scrollable channel list"""
        # Header
        header_label = tk.Label(parent, text="Choose a channel to watch:", 
                               font=('Arial', 12, 'bold'), 
                               fg='#ecf0f1', bg='#2c3e50')
        header_label.pack(pady=(0, 10))
        
        # Scrollable frame
        canvas = tk.Canvas(parent, bg='#34495e', highlightthickness=0, height=500)
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
        
        # Add channels
        for i, channel in enumerate(self.channels):
            self.create_channel_item(scrollable_frame, channel, i)
            
        # Mouse wheel scrolling
        def _on_mousewheel(event):
            canvas.yview_scroll(int(-1*(event.delta/120)), "units")
        canvas.bind_all("<MouseWheel>", _on_mousewheel)
            
    def create_channel_item(self, parent, channel, index):
        """Create individual channel item"""
        # Channel frame
        channel_frame = tk.Frame(parent, bg='#2c3e50', relief=tk.RAISED, bd=1)
        channel_frame.pack(fill=tk.X, padx=5, pady=2)
        
        # Left side - channel info
        info_frame = tk.Frame(channel_frame, bg='#2c3e50')
        info_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True, padx=15, pady=10)
        
        # Channel number and name
        channel_text = f"{channel['number']} - {channel['name']}"
        channel_label = tk.Label(info_frame, text=channel_text,
                                font=('Arial', 11, 'bold'),
                                fg='#3498db', bg='#2c3e50', anchor='w')
        channel_label.pack(anchor='w')
        
        # Current program (EPG)
        current_time = datetime.now().strftime("%H:%M")
        epg_text = f"{current_time} - {self.get_current_program(channel)}"
        epg_label = tk.Label(info_frame, text=epg_text,
                            font=('Arial', 9),
                            fg='#95a5a6', bg='#2c3e50', anchor='w')
        epg_label.pack(anchor='w')
        
        # Right side - buttons
        button_frame = tk.Frame(channel_frame, bg='#2c3e50')
        button_frame.pack(side=tk.RIGHT, padx=15, pady=10)
        
        # Watch button
        watch_btn = tk.Button(button_frame, text="▶️ Watch",
                             font=('Arial', 9, 'bold'),
                             bg='#27ae60', fg='white',
                             activebackground='#2ecc71',
                             relief=tk.FLAT, bd=0,
                             padx=15, pady=5,
                             command=lambda ch=channel: self.watch_channel(ch))
        watch_btn.pack(side=tk.TOP, pady=(0, 2))
        
        # Info button
        info_btn = tk.Button(button_frame, text="ℹ️ Info",
                            font=('Arial', 8),
                            bg='#34495e', fg='white',
                            activebackground='#5d6d7e',
                            relief=tk.FLAT, bd=0,
                            padx=15, pady=2,
                            command=lambda ch=channel: self.show_channel_info(ch))
        info_btn.pack(side=tk.TOP)
        
        # Store EPG label for updates
        setattr(self, f'epg_label_{index}', epg_label)
        
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
        """Start watching a channel using embedded VLC"""
        try:
            self.status_var.set(f"Starting {channel['number']} - {channel['name']}...")
            self.root.update()
            
            # Create stream URL
            stream_url = f"http://{self.device_ip}:5004/auto/v{channel['number']}"
            
            # Stop any current playback
            if self.current_media:
                self.media_player.stop()
            
            # Create new media
            self.current_media = self.vlc_instance.media_new(stream_url)
            self.media_player.set_media(self.current_media)
            
            # Set the video output to our frame
            if platform.system() == "Linux":
                self.media_player.set_xwindow(self.video_frame.winfo_id())
            elif platform.system() == "Windows":
                self.media_player.set_hwnd(self.video_frame.winfo_id())
            elif platform.system() == "Darwin":  # macOS
                self.media_player.set_nsobject(self.video_frame.winfo_id())
            
            # Start playback
            self.media_player.play()
            
            self.status_var.set(f"Playing: {channel['number']} - {channel['name']} (Volume: {self.volume_var.get()}%)")
            
            print(f"Playing: {stream_url}")
            
        except Exception as e:
            messagebox.showerror("Playback Error", f"Failed to play channel: {str(e)}")
            self.status_var.set("Error playing channel")
            
    def show_channel_info(self, channel):
        """Show detailed channel information"""
        info_text = f"""Channel Information:

Number: {channel['number']}
Name: {channel['name']}
Program: {channel['program']}
Current: {self.get_current_program(channel)}

Stream URL:
http://{self.device_ip}:5004/auto/v{channel['number']}

Note: HDHomeRun has 2 tuners. If you get an error,
another device may be using both tuners."""
        
        messagebox.showinfo(f"Channel {channel['number']}", info_text)
        
    def update_epg(self):
        """Update EPG information periodically"""
        def update_loop():
            while True:
                try:
                    current_time = datetime.now().strftime("%H:%M")
                    for i, channel in enumerate(self.channels):
                        epg_label = getattr(self, f'epg_label_{i}', None)
                        if epg_label and epg_label.winfo_exists():
                            epg_text = f"{current_time} - {self.get_current_program(channel)}"
                            epg_label.config(text=epg_text)
                    
                    # Update every minute
                    threading.Event().wait(60)
                except Exception as e:
                    print(f"EPG update error: {e}")
                    threading.Event().wait(60)
                    
        # Start EPG update thread
        epg_thread = threading.Thread(target=update_loop, daemon=True)
        epg_thread.start()
        
    def stop_playback(self):
        """Stop current playback"""
        if self.current_media:
            self.media_player.stop()
            self.status_var.set("Playback stopped")
    
    def update_volume(self, value):
        """Update VLC volume"""
        volume = int(value)
        # Set volume on the media player (VLC supports 0-200 range for amplification)
        self.media_player.audio_set_volume(volume)
        if self.current_media and self.media_player.is_playing():
            # Update status to show current volume
            current_status = self.status_var.get()
            if "Volume:" in current_status:
                base_status = current_status.split(" (Volume:")[0]
                self.status_var.set(f"{base_status} (Volume: {volume}%)")
        # Show amplification warning for volumes above 100%
        if volume > 100:
            print(f"Volume set to: {volume}% (AMPLIFIED - may cause distortion)")
        else:
            print(f"Volume set to: {volume}%")
    
    def toggle_fullscreen(self):
        """Toggle fullscreen mode with separate window"""
        if not self.is_fullscreen:
            self.enter_fullscreen()
        else:
            self.exit_fullscreen()
    
    def enter_fullscreen(self):
        """Enter fullscreen mode with separate window"""
        if not self.current_media or not self.media_player.is_playing():
            messagebox.showwarning("No Video", "Please start playing a channel first.")
            return
            
        self.is_fullscreen = True
        
        # Get current playback position and volume
        current_position = self.media_player.get_position()
        current_volume = self.volume_var.get()
        
        # Pause the embedded player
        self.media_player.pause()
        
        # Create fullscreen window
        self.fullscreen_window = tk.Toplevel(self.root)
        self.fullscreen_window.title("VLC Fullscreen")
        self.fullscreen_window.configure(bg='black')
        
        # Make it truly fullscreen
        self.fullscreen_window.attributes('-fullscreen', True)
        self.fullscreen_window.attributes('-topmost', True)
        
        # Create video frame for fullscreen
        self.fullscreen_video_frame = tk.Frame(self.fullscreen_window, bg='black')
        self.fullscreen_video_frame.pack(fill=tk.BOTH, expand=True)
        
        # Create separate VLC media player for fullscreen
        self.fullscreen_media_player = self.vlc_instance.media_player_new()
        
        # Set the video output to fullscreen frame
        self.fullscreen_window.update()  # Ensure window is rendered
        if platform.system() == "Linux":
            self.fullscreen_media_player.set_xwindow(self.fullscreen_video_frame.winfo_id())
        elif platform.system() == "Windows":
            self.fullscreen_media_player.set_hwnd(self.fullscreen_video_frame.winfo_id())
        elif platform.system() == "Darwin":  # macOS
            self.fullscreen_media_player.set_nsobject(self.fullscreen_video_frame.winfo_id())
        
        # Set the same media and start playback
        self.fullscreen_media_player.set_media(self.current_media)
        self.fullscreen_media_player.play()
        
        # Restore position and volume
        self.fullscreen_media_player.set_position(current_position)
        self.fullscreen_media_player.audio_set_volume(current_volume)
        
        # Bind escape key to exit fullscreen
        self.fullscreen_window.bind('<Escape>', self.exit_fullscreen)
        self.fullscreen_window.bind('<Button-1>', self.exit_fullscreen)  # Click to exit
        self.fullscreen_window.focus_set()
        
        # Update button text
        self.fullscreen_btn.config(text="🔲 Exit Fullscreen")
        
        # Handle window close
        self.fullscreen_window.protocol("WM_DELETE_WINDOW", self.exit_fullscreen)
        
        print("Entered fullscreen mode")
    
    def exit_fullscreen(self, event=None):
        """Exit fullscreen mode"""
        if not self.is_fullscreen or not hasattr(self, 'fullscreen_window'):
            return
            
        self.is_fullscreen = False
        
        # Get current position and volume from fullscreen player
        current_position = self.fullscreen_media_player.get_position()
        current_volume = self.fullscreen_media_player.audio_get_volume()
        
        # Stop fullscreen player
        self.fullscreen_media_player.stop()
        
        # Destroy fullscreen window
        self.fullscreen_window.destroy()
        
        # Resume embedded player
        self.media_player.set_position(current_position)
        self.media_player.audio_set_volume(current_volume)
        self.media_player.play()
        
        # Update button text
        self.fullscreen_btn.config(text="🔳 Fullscreen")
        
        # Clean up
        if hasattr(self, 'fullscreen_media_player'):
            del self.fullscreen_media_player
        if hasattr(self, 'fullscreen_window'):
            del self.fullscreen_window
        
        print("Exited fullscreen mode")

    def on_closing(self):
        """Handle application closing"""
        if self.current_media:
            self.media_player.stop()
        self.root.destroy()
        
    def run(self):
        """Start the application"""
        print("🚀 Starting Simple HDHomeRun TV Viewer...")
        print(f"📡 Using HDHomeRun at: {self.device_ip}")
        print(f"📺 {len(self.channels)} channels available")
        print("💡 Click 'Watch' to start a channel in VLC")
        
        self.root.protocol("WM_DELETE_WINDOW", self.on_closing)
        
        try:
            self.root.mainloop()
        except KeyboardInterrupt:
            self.on_closing()

if __name__ == "__main__":
    # Check if VLC is available
    try:
        subprocess.run(['vlc', '--version'], 
                      stdout=subprocess.DEVNULL, 
                      stderr=subprocess.DEVNULL, 
                      check=True)
    except (FileNotFoundError, subprocess.CalledProcessError):
        print("❌ VLC is required but not found.")
        print("   Install it with: sudo pacman -S vlc")
        sys.exit(1)
    
    app = SimpleHDHomeRunTVViewer()
    app.run()
