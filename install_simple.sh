#!/bin/bash

# AI Coding Assistant Installation Script
# This script installs the AI Coding Assistant and removes the old open interpreter launcher

set -e

APP_NAME="AI Coding Assistant"
BINARY_NAME="ai-coding-assistant"
INSTALL_DIR="/opt/ai-coding-assistant"
DESKTOP_FILE="/usr/share/applications/${BINARY_NAME}.desktop"
BIN_LINK="/usr/local/bin/${BINARY_NAME}"
SOURCE_DIR="/home/lou/awesome_stack/open-interpreter-tauri"

echo "🚀 Installing $APP_NAME..."

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (use sudo)" 
   exit 1
fi

# Check if binary exists
if [ ! -f "$SOURCE_DIR/src-tauri/target/release/app" ]; then
    echo "❌ Binary not found. Building first..."
    echo "Please run: cd $SOURCE_DIR && npm run build && cd src-tauri && cargo build --release"
    exit 1
fi

# Create installation directory
echo "📁 Creating installation directory..."
mkdir -p "$INSTALL_DIR"

# Copy the binary
echo "📋 Installing binary..."
cp "$SOURCE_DIR/src-tauri/target/release/app" "$INSTALL_DIR/${BINARY_NAME}"
chmod +x "$INSTALL_DIR/${BINARY_NAME}"

# Create symlink for command line access
echo "🔗 Creating command line shortcut..."
ln -sf "$INSTALL_DIR/${BINARY_NAME}" "$BIN_LINK"

# Create desktop entry
echo "🖥️  Creating desktop entry..."
cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Name=AI Coding Assistant
Comment=AI-powered coding assistant with local LLM support
Exec=${INSTALL_DIR}/${BINARY_NAME}
Icon=applications-development
Terminal=false
Type=Application
Categories=Development;TextEditor;
Keywords=AI;Code;Assistant;Programming;LLM;
StartupNotify=true
EOF

# Set permissions
chmod 644 "$DESKTOP_FILE"

# Remove old open interpreter desktop files
echo "🗑️  Removing old open interpreter launcher..."
OLD_DESKTOP_FILES=(
    "/usr/share/applications/open-interpreter.desktop"
    "/usr/share/applications/interpreter.desktop"
    "/home/$SUDO_USER/.local/share/applications/open-interpreter.desktop"
    "/home/$SUDO_USER/.local/share/applications/interpreter.desktop"
)

for file in "${OLD_DESKTOP_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   Removing $file"
        rm -f "$file"
    fi
done

# Update desktop database
echo "🔄 Updating desktop database..."
update-desktop-database /usr/share/applications/ 2>/dev/null || true
if [ -d "/home/$SUDO_USER/.local/share/applications" ]; then
    update-desktop-database "/home/$SUDO_USER/.local/share/applications/" 2>/dev/null || true
fi

# Create uninstall script
echo "📝 Creating uninstall script..."
cat > "$INSTALL_DIR/uninstall.sh" << EOF
#!/bin/bash
# Uninstall AI Coding Assistant

echo "Removing AI Coding Assistant..."
rm -rf "$INSTALL_DIR"
rm -f "$BIN_LINK"
rm -f "$DESKTOP_FILE"
update-desktop-database /usr/share/applications/ 2>/dev/null || true
echo "AI Coding Assistant uninstalled successfully!"
EOF

chmod +x "$INSTALL_DIR/uninstall.sh"

echo "✅ Installation completed successfully!"
echo ""
echo "🎉 $APP_NAME has been installed!"
echo ""
echo "📍 Installation location: $INSTALL_DIR"
echo "💻 Command line: $BINARY_NAME"
echo "🖥️  Desktop launcher: Available in your applications menu"
echo "🗑️  To uninstall: sudo $INSTALL_DIR/uninstall.sh"
echo ""
echo "🔧 The application connects to your Ollama instance at http://192.168.122.172:11434"
echo "🤖 Make sure Ollama is running with your preferred models"
echo ""
echo "🚀 You can now launch the AI Coding Assistant from:"
echo "   • Applications menu"
echo "   • Command line: $BINARY_NAME"
echo "   • Direct path: $INSTALL_DIR/$BINARY_NAME"
