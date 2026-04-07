#!/bin/bash

# Ollama Code Checker Installation Script
set -e

echo "🚀 Installing Ollama Code Checker..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo -e "${YELLOW}Installing Ollama...${NC}"
    curl -fsSL https://ollama.ai/install.sh | sh
else
    echo -e "${GREEN}Ollama is already installed${NC}"
fi

# Create installation directory
# Allow customization via environment variable
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

# Copy main script
echo -e "${BLUE}Installing main script...${NC}"
cp ollama-code-checker.sh "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/ollama-code-checker.sh"

# Create symlink for easy access
ln -sf "$INSTALL_DIR/ollama-code-checker.sh" "$INSTALL_DIR/codecheck"

# Copy GUI
echo -e "${BLUE}Installing GUI...${NC}"
cp ollama-gui.py "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/ollama-gui.py"

# Check if Python tkinter is available
if ! python3 -c "import tkinter" 2>/dev/null; then
    echo -e "${YELLOW}Warning: Python tkinter not found. GUI may not work.${NC}"
    echo "Install with: sudo pacman -S tk (Arch) or sudo apt install python3-tkinter (Debian/Ubuntu)"
fi

# Add to PATH if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}Adding $INSTALL_DIR to PATH...${NC}"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.bashrc"
    echo "Please run: source ~/.bashrc or start a new terminal"
fi

echo -e "${GREEN}✅ Installation complete!${NC}"
echo
echo "Usage:"
echo "  codecheck                    # Interactive mode"
echo "  codecheck /path/to/code      # Analyze directory"
echo "  codecheck -f file.rs         # Analyze single file"
echo "  python3 $INSTALL_DIR/ollama-gui.py  # Launch GUI"
echo
echo "For more options: codecheck --help"