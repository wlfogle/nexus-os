#!/bin/bash

# Wrapper script to run Warp auto-login with proper virtual environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$SCRIPT_DIR/../venv"
PYTHON_SCRIPT="$SCRIPT_DIR/warp_auto_login.py"

# Check if virtual environment exists
if [ ! -f "$VENV_DIR/bin/python" ]; then
    echo "❌ Virtual environment not found at $VENV_DIR"
    echo "Creating virtual environment..."
    python -m venv "$VENV_DIR"
    
    echo "Installing dependencies..."
    "$VENV_DIR/bin/pip" install selenium==4.15.0 pyperclip webdriver-manager
fi

# Run the script with the virtual environment's Python
echo "🚀 Starting Warp Auto-Login with virtual environment..."
"$VENV_DIR/bin/python" "$PYTHON_SCRIPT" "$@"
