# PortProton Enhanced - Installation Guide

## System Requirements

### Minimum Requirements
- **OS**: Ubuntu 24.04+ (25.10 recommended)
- **GPU**: NVIDIA GTX 1060+ with proprietary drivers
- **CPU**: 4+ cores, 2.5GHz+
- **RAM**: 8GB+ 
- **Storage**: 4GB free space

### Recommended Configuration (Tested)
- **OS**: Ubuntu 25.10 + KDE Plasma
- **GPU**: NVIDIA RTX 4080 (driver 580.82.07)
- **CPU**: AMD/Intel 8+ cores, 3.0GHz+
- **RAM**: 16GB+
- **Storage**: 8GB+ free space (SSD recommended)

## Quick Installation

### 1. Clone the Repository
```bash
git clone https://github.com/wlfogle/PortProton-Enhanced.git
cd PortProton-Enhanced
```

### 2. Run the Enhanced Installer
```bash
chmod +x install-enhanced-portproton.sh
./install-enhanced-portproton.sh
```

### 3. Launch PortProton Enhanced
```bash
# From desktop menu
# Search for "PortProton Enhanced"

# Or from terminal
portproton-enhanced

# Or direct path
~/PortProton-Enhanced/bin/portproton
```

## What the Installer Does

The enhanced installer performs these steps automatically:

### 1. System Dependencies
Installs required packages:
```bash
sudo apt install -y \
    yad jq cabextract xterm desktop-file-utils vulkan-tools mesa-utils \
    xrandr setxkbmap perl curl wget git gamemode mangohud goverlay \
    wine winetricks zenity unzip p7zip-full
```

### 2. PortProton Core Setup
- Downloads PortWINE core components
- Creates installation directory: `~/PortProton-Enhanced`
- Sets up directory structure: `bin/`, `data/`, `scripts/`, etc.

### 3. Custom Autoinstall Scripts
Installs 25+ custom scripts:
- **Game Launchers**: EA App, Riot Games, Amazon Games, Xbox App, etc.
- **Direct Game Launchers**: For your installed games
- **Auto-Detection Scripts**: Steam library parser and game scanner

### 4. System Optimizations
Applies RTX 4080 + Ubuntu 25.10 specific optimizations:
- NVIDIA shader caching
- Hybrid graphics prime offloading
- Multi-core CPU topology
- KDE Plasma integration

### 5. Desktop Integration
- Creates desktop shortcut
- Installs to application menu
- Creates command-line launcher: `portproton-enhanced`

### 6. Game Library Configuration
Auto-configures paths for:
- Regular games: `/media/lou/Games/Games`
- Steam library: `/media/lou/Games/SteamLibrary`

## Manual Installation Steps

If you prefer manual installation or need to troubleshoot:

### Step 1: Install Dependencies
```bash
sudo apt update
sudo apt install -y yad jq cabextract xterm desktop-file-utils vulkan-tools mesa-utils
sudo apt install -y xrandr setxkbmap perl curl wget git gamemode mangohud goverlay
sudo apt install -y wine winetricks zenity unzip p7zip-full
```

### Step 2: Create Directory Structure
```bash
mkdir -p ~/PortProton-Enhanced/{bin,data,scripts,share,themes,img}
```

### Step 3: Download PortProton Core
```bash
curl -L "https://github.com/Castro-Fidel/PortWINE/archive/refs/heads/master.tar.gz" -o /tmp/PortWINE-master.tar.gz
tar -xf /tmp/PortWINE-master.tar.gz -C /tmp/
cp -r /tmp/PortWINE-master/data_from_portwine/* ~/PortProton-Enhanced/data/
```

### Step 4: Generate Autoinstall Scripts
```bash
./create-all-autoinstall-scripts.sh
cp autoinstall_scripts/PW_* ~/PortProton-Enhanced/data/scripts/pw_autoinstall/
chmod +x ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_*
```

### Step 5: Create Launcher Script
```bash
cat > ~/PortProton-Enhanced/bin/portproton << 'EOF'
#!/usr/bin/env bash
unset PW_AUTOPLAY PW_SET_LANG PW_SILENT_INSTALL
export port_ver=1.7.1-Enhanced

def_path="${HOME}/PortProton-Enhanced"

# System optimizations for Ubuntu 25.10 + RTX 4080
export __GL_SHADER_DISK_CACHE=1
export __GL_THREADED_OPTIMIZATIONS=1
export __NV_PRIME_RENDER_OFFLOAD=1
export __GLX_VENDOR_LIBRARY_NAME=nvidia
export WINE_CPU_TOPOLOGY=8:4
export MANGOHUD=1

# Enhanced PortProton paths
export PORT_WINE_PATH="$def_path"
export PORT_WINE_DATA_PATH="$def_path/data"
export INSTALLING_PORT=1

cd "$def_path/data"
if [[ -f "$def_path/data/scripts/start.sh" ]]; then
    echo "$port_ver" > "$def_path/data/tmp/PortProton_ver"
    /usr/bin/env bash "$def_path/data/scripts/start.sh" "$@"
else
    echo "Please run the installation first!"
fi
EOF

chmod +x ~/PortProton-Enhanced/bin/portproton
```

### Step 6: Create Desktop Integration
```bash
mkdir -p ~/.local/share/applications
cat > ~/.local/share/applications/portproton-enhanced.desktop << EOF
[Desktop Entry]
Name=PortProton Enhanced
Comment=Enhanced PortProton with custom autoinstall scripts and RTX 4080 optimizations
Exec=$HOME/PortProton-Enhanced/bin/portproton
Icon=$HOME/PortProton-Enhanced/data/img/gui/pp.png
Terminal=false
Type=Application
Categories=Game;
StartupNotify=true
StartupWMClass=portproton
EOF

update-desktop-database ~/.local/share/applications/
```

### Step 7: Create System Launcher
```bash
sudo ln -sf ~/PortProton-Enhanced/bin/portproton /usr/local/bin/portproton-enhanced
```

## Post-Installation Configuration

### Verify Installation
```bash
# Check if PortProton Enhanced is installed
ls -la ~/PortProton-Enhanced/

# Check desktop integration
ls -la ~/.local/share/applications/portproton-enhanced.desktop

# Check system launcher
which portproton-enhanced

# Count autoinstall scripts
ls ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_* | wc -l
```

### Configure Game Paths
Edit the configuration file:
```bash
nano ~/PortProton-Enhanced/data/user.conf
```

Add or modify these paths:
```bash
export GAMES_BASE_PATH="/your/games/path"
export STEAM_LIBRARY_PATH="/your/steam/library/path"
```

### Test NVIDIA Optimizations
Verify hybrid graphics are working:
```bash
# Check NVIDIA driver
nvidia-smi

# Test Vulkan
vulkaninfo | grep -i "device name"

# Test GameMode
gamemoded -s
```

## Troubleshooting Installation

### Permission Issues
```bash
# Fix script permissions
chmod +x ~/PortProton-Enhanced/bin/portproton
chmod +x ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_*

# Fix ownership
sudo chown -R $USER:$USER ~/PortProton-Enhanced/
```

### Missing Dependencies
```bash
# Install missing packages
sudo apt install --fix-missing
sudo apt install -f

# Verify Wine installation
wine --version

# Check Vulkan support
vulkaninfo --summary
```

### Desktop Integration Issues
```bash
# Refresh desktop database
update-desktop-database ~/.local/share/applications/

# Clear icon cache
gtk-update-icon-cache ~/.local/share/icons/

# Restart desktop environment (KDE Plasma)
kquitapp5 plasmashell && kstart5 plasmashell
```

### Game Detection Issues
```bash
# Test game path access
ls -la "/media/lou/Games/Games"
ls -la "/media/lou/Games/SteamLibrary"

# Run auto-detection manually
~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_AUTO_DETECT_ALL_GAMES

# Check Steam manifests
ls /media/lou/Games/SteamLibrary/steamapps/appmanifest_*.acf | wc -l
```

## Uninstallation

To completely remove PortProton Enhanced:

```bash
# Remove installation directory
rm -rf ~/PortProton-Enhanced/

# Remove desktop integration
rm -f ~/.local/share/applications/portproton-enhanced.desktop

# Remove system launcher
sudo rm -f /usr/local/bin/portproton-enhanced

# Update desktop database
update-desktop-database ~/.local/share/applications/
```

## Next Steps

After successful installation:

1. **Launch PortProton Enhanced** from your application menu
2. **Explore AutoInstall** options to see all 25+ custom launchers
3. **Use Auto-Detect** features to scan your game libraries
4. **Install game launchers** like EA App, Riot Games, etc.
5. **Create direct shortcuts** for your existing games
6. **Enjoy optimized gaming** with RTX 4080 + hybrid graphics!

## Getting Help

- Check the main [README.md](README.md) for overview
- Review [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues
- See [OPTIMIZATION.md](OPTIMIZATION.md) for performance tuning
- Visit the [GitHub Issues](https://github.com/wlfogle/PortProton-Enhanced/issues) page