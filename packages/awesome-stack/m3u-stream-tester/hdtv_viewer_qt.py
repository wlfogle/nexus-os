#!/usr/bin/env python3
"""
HDHomeRun TV Viewer - PyQt5 Version
Uses PyQt5 and python-vlc for better video integration
"""

import sys
import vlc
from PyQt5 import QtWidgets, QtGui, QtCore
from PyQt5.QtCore import Qt, QTimer, QThread, pyqtSignal
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, 
                           QHBoxLayout, QSplitter, QListWidget, QListWidgetItem,
                           QPushButton, QSlider, QLabel, QFrame, QMessageBox,
                           QScrollArea)
from datetime import datetime
import threading

class HDHomeRunTVViewer(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("📺 HDHomeRun TV Viewer - PyQt5")
        self.setGeometry(100, 100, 1200, 800)
        
        # HDHomeRun device info
        self.device_ip = "192.168.12.215"
        
        # VLC setup
        vlc_args = [
            '--no-xlib',
            '--intf', 'dummy',
            '--quiet',
            '--no-video-title-show',
            '--network-caching=1000'
        ]
        self.vlc_instance = vlc.Instance(vlc_args)
        self.media_player = self.vlc_instance.media_player_new()
        self.current_media = None
        self.is_fullscreen = False
        self.current_channel_index = 0  # Track currently playing channel
        
        # Channel data
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
        
        self.setup_ui()
        
    def setup_ui(self):
        """Setup the main UI"""
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        # Main layout
        main_layout = QHBoxLayout(central_widget)
        
        # Create splitter for resizable panels
        splitter = QSplitter(Qt.Horizontal)
        main_layout.addWidget(splitter)
        
        # Left panel - Channel list
        self.setup_channel_panel(splitter)
        
        # Right panel - Video player
        self.setup_video_panel(splitter)
        
        # Set splitter proportions (30% channels, 70% video)
        splitter.setSizes([600, 600])
        
        # Status bar
        self.statusBar().showMessage(f"Ready! {len(self.channels)} channels available")
        
    def setup_channel_panel(self, parent):
        """Setup the channel list panel"""
        channel_widget = QWidget()
        channel_layout = QVBoxLayout(channel_widget)
        
        # Header
        header_label = QLabel("📺 Choose a channel to watch:")
        header_label.setStyleSheet("font-size: 14px; font-weight: bold; color: #3498db; margin: 10px;")
        channel_layout.addWidget(header_label)
        
        # Channel list
        self.channel_list = QListWidget()
        self.channel_list.setStyleSheet("""
            QListWidget {
                background-color: #2c3e50;
                border: none;
                color: white;
            }
            QListWidget::item {
                padding: 8px;
                border-bottom: 1px solid #34495e;
                min-height: 45px;
            }
            QListWidget::item:hover {
                background-color: #34495e;
            }
            QListWidget::item:selected {
                background-color: #3498db;
            }
        """)
        
        # Populate channel list
        for channel in self.channels:
            item = QListWidgetItem()
            item_widget = self.create_channel_item(channel)
            item.setSizeHint(item_widget.sizeHint())
            self.channel_list.addItem(item)
            self.channel_list.setItemWidget(item, item_widget)
            
        channel_layout.addWidget(self.channel_list)
        parent.addWidget(channel_widget)
        
    def create_channel_item(self, channel):
        """Create a channel item widget"""
        widget = QWidget()
        layout = QHBoxLayout(widget)
        
        # Channel info
        info_layout = QVBoxLayout()
        
        channel_label = QLabel(f"{channel['number']} - {channel['name']}")
        channel_label.setStyleSheet("font-weight: bold; color: #3498db; font-size: 13px;")
        info_layout.addWidget(channel_label)
        
        current_time = datetime.now().strftime("%H:%M")
        program_label = QLabel(f"{current_time} - {self.get_current_program(channel)}")
        program_label.setStyleSheet("color: #95a5a6; font-size: 11px;")
        info_layout.addWidget(program_label)
        
        layout.addLayout(info_layout)
        layout.addStretch()
        
        # Watch button
        watch_btn = QPushButton("▶️ Watch")
        watch_btn.setStyleSheet("""
            QPushButton {
                background-color: #27ae60;
                color: white;
                border: none;
                padding: 8px 15px;
                border-radius: 4px;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #2ecc71;
            }
        """)
        watch_btn.clicked.connect(lambda: self.watch_channel(channel))
        layout.addWidget(watch_btn)
        
        return widget
        
    def get_current_program(self, channel):
        """Get current program based on time and channel"""
        hour = datetime.now().hour
        
        if 6 <= hour < 12:
            time_period = "Morning"
        elif 12 <= hour < 17:
            time_period = "Afternoon"
        elif 17 <= hour < 22:
            time_period = "Evening"
        else:
            time_period = "Late Night"
            
        if "News" in channel['name'] or channel['number'] in ['11.1', '32.1', '41.1']:
            return f"{time_period} News"
        elif "Movie" in channel['program'] or "Classic" in channel['program']:
            return channel['program']
        else:
            return f"{time_period} - {channel['program']}"
            
    def setup_video_panel(self, parent):
        """Setup the video player panel"""
        video_widget = QWidget()
        video_layout = QVBoxLayout(video_widget)
        
        # Video frame
        self.video_frame = QFrame()
        self.video_frame.setStyleSheet("background-color: black;")
        self.video_frame.setMinimumSize(640, 480)
        video_layout.addWidget(self.video_frame, 1)
        
        # Placeholder label
        self.placeholder_label = QLabel("📺 Select a channel to start watching\n\nClick 'Watch' button to play a channel")
        self.placeholder_label.setAlignment(Qt.AlignCenter)
        self.placeholder_label.setStyleSheet("""
            color: #95a5a6;
            font-size: 16px;
            background-color: black;
        """)
        self.placeholder_label.setParent(self.video_frame)
        
        # Controls
        controls_layout = QHBoxLayout()
        
        # Stop button
        self.stop_btn = QPushButton("⏹️ Stop")
        self.stop_btn.setStyleSheet("""
            QPushButton {
                background-color: #e74c3c;
                color: white;
                border: none;
                padding: 8px 15px;
                border-radius: 4px;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #c0392b;
            }
        """)
        self.stop_btn.clicked.connect(self.stop_playback)
        controls_layout.addWidget(self.stop_btn)
        
        # Fullscreen button
        self.fullscreen_btn = QPushButton("🔳 Fullscreen")
        self.fullscreen_btn.setStyleSheet("""
            QPushButton {
                background-color: #3498db;
                color: white;
                border: none;
                padding: 8px 15px;
                border-radius: 4px;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #2980b9;
            }
        """)
        self.fullscreen_btn.clicked.connect(self.toggle_fullscreen)
        controls_layout.addWidget(self.fullscreen_btn)
        
        controls_layout.addStretch()
        
        # Volume control
        volume_label = QLabel("Volume:")
        volume_label.setStyleSheet("color: white; font-weight: bold;")
        controls_layout.addWidget(volume_label)
        
        self.volume_slider = QSlider(Qt.Horizontal)
        self.volume_slider.setRange(0, 200)
        self.volume_slider.setValue(80)
        self.volume_slider.setFixedWidth(150)
        self.volume_slider.valueChanged.connect(self.update_volume)
        controls_layout.addWidget(self.volume_slider)
        
        self.volume_label = QLabel("80%")
        self.volume_label.setStyleSheet("color: white; font-weight: bold; min-width: 35px;")
        controls_layout.addWidget(self.volume_label)
        
        video_layout.addLayout(controls_layout)
        parent.addWidget(video_widget)
        
    def watch_channel(self, channel):
        """Start watching a channel"""
        try:
            self.statusBar().showMessage(f"Starting {channel['number']} - {channel['name']}...")
            
            # Create stream URL
            stream_url = f"http://{self.device_ip}:5004/auto/v{channel['number']}"
            
            # Stop any current playback
            if self.current_media:
                self.media_player.stop()
            
            # Create new media
            self.current_media = self.vlc_instance.media_new(stream_url)
            self.media_player.set_media(self.current_media)
            
            # Set video output to frame
            self.media_player.set_xwindow(int(self.video_frame.winId()))
            
            # Set initial volume
            self.media_player.audio_set_volume(self.volume_slider.value())
            
            # Start playback
            self.media_player.play()
            
            # Update current channel index
            self.current_channel_index = next((i for i, ch in enumerate(self.channels) if ch['number'] == channel['number']), 0)
            
            # Hide placeholder
            self.placeholder_label.hide()
            
            self.statusBar().showMessage(f"Playing: {channel['number']} - {channel['name']} (Volume: {self.volume_slider.value()}%)")
            
            print(f"Playing: {stream_url}")
            
        except Exception as e:
            QMessageBox.critical(self, "Playback Error", f"Failed to play channel: {str(e)}")
            self.statusBar().showMessage("Error playing channel")
            
    def stop_playback(self):
        """Stop current playback"""
        if self.current_media:
            self.media_player.stop()
            self.placeholder_label.show()
            self.statusBar().showMessage("Playback stopped")
            
    def update_volume(self, value):
        """Update volume"""
        self.media_player.audio_set_volume(value)
        self.volume_label.setText(f"{value}%")
        
        if value > 100:
            print(f"Volume set to: {value}% (AMPLIFIED - may cause distortion)")
        else:
            print(f"Volume set to: {value}%")
            
    def toggle_fullscreen(self):
        """Toggle fullscreen mode"""
        if not self.current_media or not self.media_player.is_playing():
            QMessageBox.warning(self, "No Video", "Please start playing a channel first.")
            return
            
        if not self.is_fullscreen:
            self.enter_fullscreen()
        else:
            self.exit_fullscreen()
            
    def enter_fullscreen(self):
        """Enter fullscreen mode"""
        if not self.current_media or not self.media_player.is_playing():
            return
            
        self.is_fullscreen = True
        
        # Create fullscreen window
        self.fullscreen_window = QWidget()
        self.fullscreen_window.setWindowTitle("VLC Fullscreen")
        self.fullscreen_window.setStyleSheet("background-color: black;")
        self.fullscreen_window.showFullScreen()
        self.fullscreen_window.setMouseTracking(True)
        
        # Create layout for fullscreen window
        fullscreen_layout = QVBoxLayout(self.fullscreen_window)
        fullscreen_layout.setContentsMargins(0, 0, 0, 0)
        
        # Video frame for fullscreen
        self.fullscreen_video_frame = QFrame()
        self.fullscreen_video_frame.setStyleSheet("background-color: black;")
        fullscreen_layout.addWidget(self.fullscreen_video_frame, 1)
        
        # Get current state
        current_position = self.media_player.get_position()
        current_volume = self.media_player.audio_get_volume()
        
        # Create new media player for fullscreen
        self.fullscreen_media_player = self.vlc_instance.media_player_new()
        self.fullscreen_media_player.set_media(self.current_media)
        
        # Overlay controls (initially hidden)
        self.create_fullscreen_controls(fullscreen_layout, current_volume)
        
        # Wait for window to be shown then set video output
        QApplication.processEvents()
        self.fullscreen_media_player.set_xwindow(int(self.fullscreen_video_frame.winId()))
        
        # Pause main player and start fullscreen player
        self.media_player.pause()
        self.fullscreen_media_player.play()
        
        # Restore state
        self.fullscreen_media_player.set_position(current_position)
        self.fullscreen_media_player.audio_set_volume(current_volume)
        self.fullscreen_window.mouseMoveEvent = self.fullscreen_mouse_move
        
        # Timer to hide controls
        self.controls_timer = QTimer()
        self.controls_timer.timeout.connect(self.hide_fullscreen_controls)
        self.controls_timer.setSingleShot(True)
        
        self.fullscreen_btn.setText("🔲 Exit Fullscreen")
        print("Entered fullscreen mode")
        
    def create_fullscreen_controls(self, layout, current_volume):
        """Create overlay controls for fullscreen"""
        self.controls_widget = QWidget()
        self.controls_widget.setStyleSheet(
            "background-color: rgba(0, 0, 0, 0.7); padding: 10px;"
        )
        controls_layout = QHBoxLayout(self.controls_widget)
        
        # Fullscreen exit button
        exit_btn = QPushButton("🔲 Exit Fullscreen")
        exit_btn.setStyleSheet("color: white; font-weight: bold;")
        exit_btn.clicked.connect(self.exit_fullscreen)
        controls_layout.addWidget(exit_btn)
        
        # Channel picker
        self.channel_picker = QtWidgets.QComboBox()
        self.channel_picker.setStyleSheet("color: white; background-color: #34495e;")
        for channel in self.channels:
            self.channel_picker.addItem(f"{channel['number']} - {channel['name']}", userData=channel)
        self.channel_picker.setCurrentIndex(self.current_channel_index)
        self.channel_picker.activated.connect(self.select_channel)
        # Keep controls visible when dropdown is open
        self.channel_picker.showPopup = self.show_channel_popup
        controls_layout.addWidget(self.channel_picker)
        
        # Spacer
        controls_layout.addStretch()
        
        # Volume control
        volume_label = QLabel("Volume:")
        volume_label.setStyleSheet("color: white;")
        controls_layout.addWidget(volume_label)

        volume_slider = QSlider(Qt.Horizontal)
        volume_slider.setRange(0, 200)
        volume_slider.setValue(current_volume)
        volume_slider.valueChanged.connect(self.fullscreen_volume_change)
        controls_layout.addWidget(volume_slider)

        layout.addWidget(self.controls_widget, alignment=Qt.AlignBottom)
        self.controls_widget.hide()
        
    def fullscreen_volume_change(self, value):
        """Change volume in fullscreen mode"""
        self.fullscreen_media_player.audio_set_volume(value)
        print(f"Fullscreen volume set to: {value}%")
    
    def previous_channel(self):
        """Switch to previous channel"""
        if self.current_channel_index > 0:
            self.current_channel_index -= 1
        else:
            self.current_channel_index = len(self.channels) - 1
        
        channel = self.channels[self.current_channel_index]
        self.switch_channel_fullscreen(channel)
    
    def next_channel(self):
        """Switch to next channel"""
        if self.current_channel_index < len(self.channels) - 1:
            self.current_channel_index += 1
        else:
            self.current_channel_index = 0
        
        channel = self.channels[self.current_channel_index]
        self.switch_channel_fullscreen(channel)
    
    def select_channel(self, index):
        """Select a channel from the picker"""
        channel = self.channels[index]
        self.switch_channel_fullscreen(channel)

    def switch_channel_fullscreen(self, channel):
        """Switch channel while in fullscreen mode"""
        try:
            # Create new stream URL
            stream_url = f"http://{self.device_ip}:5004/auto/v{channel['number']}"
            
            # Get current volume
            current_volume = self.fullscreen_media_player.audio_get_volume()
            
            # Stop current playback
            self.fullscreen_media_player.stop()
            
            # Create new media
            new_media = self.vlc_instance.media_new(stream_url)
            self.fullscreen_media_player.set_media(new_media)
            
            # Start new playback
            self.fullscreen_media_player.play()
            
            # Restore volume
            self.fullscreen_media_player.audio_set_volume(current_volume)
            
            # Update current media for main player too
            self.current_media = new_media
            
            # Update current channel index
            self.current_channel_index = next((i for i, ch in enumerate(self.channels) if ch['number'] == channel['number']), 0)
            
            print(f"Switched to: {channel['number']} - {channel['name']}")
            
        except Exception as e:
            print(f"Error switching channel: {str(e)}")

    def show_channel_popup(self):
        """Show channel picker popup and keep controls visible"""
        # Stop the timer to prevent hiding controls
        self.controls_timer.stop()
        # Show the popup
        QtWidgets.QComboBox.showPopup(self.channel_picker)
        # Start a longer timer when popup closes
        self.controls_timer.start(10000)  # 10 seconds

    def fullscreen_mouse_move(self, event):
        """Show overlay controls on mouse move"""
        self.controls_widget.show()
        # Only start timer if channel picker popup is not open
        if not self.channel_picker.view().isVisible():
            self.controls_timer.start(3000)  # 3 seconds

    def hide_fullscreen_controls(self):
        """Hide overlay controls for fullscreen"""
        self.controls_widget.hide()

    def exit_fullscreen(self):
        """Exit fullscreen mode"""
        if not self.is_fullscreen or not hasattr(self, 'fullscreen_window'):
            return
            
        self.is_fullscreen = False
        
        # Get current state from fullscreen player
        current_position = self.fullscreen_media_player.get_position()
        current_volume = self.fullscreen_media_player.audio_get_volume()
        
        # Stop fullscreen player
        self.fullscreen_media_player.stop()
        
        # Close fullscreen window
        self.fullscreen_window.close()
        
        # Resume main player
        self.media_player.set_position(current_position)
        self.media_player.audio_set_volume(current_volume)
        self.media_player.play()
        
        # Clean up
        if hasattr(self, 'controls_timer'):
            self.controls_timer.stop()
            del self.controls_timer
        del self.fullscreen_media_player
        del self.fullscreen_window
        del self.controls_widget
        
        self.fullscreen_btn.setText("🔳 Fullscreen")
        print("Exited fullscreen mode")
        
    def fullscreen_key_handler(self, event):
        """Handle key press events in fullscreen mode"""
        if event.key() == Qt.Key_Escape:
            self.exit_fullscreen()
    
    def keyPressEvent(self, event):
        """Handle key press events"""
        if event.key() == Qt.Key_Escape and self.is_fullscreen:
            self.exit_fullscreen()
        super().keyPressEvent(event)
        
    def resizeEvent(self, event):
        """Handle resize events"""
        super().resizeEvent(event)
        # Position placeholder label in center of video frame
        if hasattr(self, 'placeholder_label'):
            frame_rect = self.video_frame.geometry()
            label_size = self.placeholder_label.sizeHint()
            x = (frame_rect.width() - label_size.width()) // 2
            y = (frame_rect.height() - label_size.height()) // 2
            self.placeholder_label.move(x, y)
            
    def closeEvent(self, event):
        """Handle application closing"""
        if self.current_media:
            self.media_player.stop()
        event.accept()

def main():
    app = QApplication(sys.argv)
    app.setApplicationName("HDHomeRun TV Viewer")
    
    # Set application style
    app.setStyleSheet("""
        QMainWindow {
            background-color: #2c3e50;
        }
        QWidget {
            background-color: #2c3e50;
            color: white;
        }
        QStatusBar {
            background-color: #34495e;
            color: #27ae60;
            font-weight: bold;
        }
    """)
    
    viewer = HDHomeRunTVViewer()
    viewer.show()
    
    print("🚀 Starting HDHomeRun TV Viewer (PyQt5)...")
    print(f"📡 Using HDHomeRun at: 192.168.12.215")
    print(f"📺 {len(viewer.channels)} channels available")
    print("💡 Click 'Watch' to start a channel")
    
    sys.exit(app.exec_())

if __name__ == "__main__":
    main()
