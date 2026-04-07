#!/bin/bash

echo "🚀 Installing HDHomeRun TV Viewer..."

# Check if python3 is installed
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 is required but not installed. Installing..."
    sudo pacman -S python python-pip
fi

# Check if VLC is installed
if ! command -v vlc &> /dev/null; then
    echo "📺 Installing VLC media player..."
    sudo pacman -S vlc
fi

# Install Python VLC bindings
echo "📦 Installing Python VLC bindings..."
pip install --user python-vlc

# Make the TV viewer executable
chmod +x hdhomerun_tv_gui.py

echo ""
echo "✅ Installation complete!"
echo ""
echo "🎯 To run the HDHomeRun TV Viewer:"
echo "   python3 hdhomerun_tv_gui.py"
echo ""
echo "📺 Features:"
echo "   • Automatic HDHomeRun detection"
echo "   • Live TV guide with current programs"
echo "   • Working Watch buttons"
echo "   • 200% volume by default"
echo "   • All your scanned channels pre-loaded"
echo ""
echo "🔧 The app uses your HDHomeRun at 192.168.12.215"
echo "   If your IP is different, edit the hdhomerun_tv_gui.py file"
echo ""
