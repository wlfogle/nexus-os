#!/bin/bash

# The Walking Dead Webisodes Downloader - GUI Launcher
# Simple launcher script for the GUI version

echo "Starting The Walking Dead Webisodes Downloader GUI..."

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Change to the script directory
cd "$SCRIPT_DIR"

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is required but not found. Please install Python 3."
    exit 1
fi

# Check if tkinter is available (required for GUI)
if ! python3 -c "import tkinter" 2>/dev/null; then
    echo "Error: tkinter is required for the GUI but not found."
    echo "Please install tkinter:"
    echo "  Ubuntu/Debian: sudo apt install python3-tk"
    echo "  Fedora: sudo dnf install tkinter"
    echo "  Arch: sudo pacman -S tk"
    exit 1
fi

# Run the GUI
python3 twd_webisodes_gui.py

echo "GUI closed."
