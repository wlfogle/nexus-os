# 🌟 Hyperion - Linux Power Utilities

**Version 0.1.0** | **Illuminate Your Linux Desktop**

Hyperion is a comprehensive collection of **power utilities** for Linux users, designed to enhance productivity and streamline desktop workflows with native Linux integration.

![Hyperion](docs/hyperion-logo.png)

## ✨ Utility Collection

### 🎯 **Core Utilities** (Planned)

#### **1. 📝 PowerRename** 
- **Bulk File Renaming**: Advanced pattern-based file renaming
- **Regex Support**: Complex renaming rules with regular expressions
- **Preview Mode**: See changes before applying
- **Undo Functionality**: Reverse renaming operations

#### **2. 🪟 FancyZones**
- **Window Snapping**: Create custom window layouts
- **Zone Templates**: Predefined and custom zone layouts
- **Multi-monitor Support**: Zones across multiple displays
- **Hotkey Control**: Keyboard-driven window management

#### **3. 🎨 Color Picker**
- **System-wide Color Picking**: Pick colors from anywhere on screen
- **Multiple Formats**: HEX, RGB, HSL, CMYK support
- **History**: Recently picked colors
- **Magnifier**: Precise pixel-level selection

#### **4. ⌨️ Keyboard Manager**
- **Key Remapping**: Remap individual keys
- **Shortcut Remapping**: Remap key combinations
- **Application-specific**: Different mappings per application
- **Profiles**: Multiple configuration profiles

#### **5. 🔍 PowerLauncher**
- **Quick Application Launcher**: Fast app launching with fuzzy search
- **Plugin System**: Extensible with custom plugins
- **Web Search**: Search the web directly
- **Calculator**: Built-in calculator functionality

#### **6. 📐 Screen Ruler**
- **Pixel-perfect Measurements**: Measure distances and angles on screen
- **Cross-platform Units**: Pixels, inches, centimeters
- **Screenshot Integration**: Measure from screenshots
- **Annotation Tools**: Mark up measurements

## 🏗️ Architecture

### **Modular Design**
PowerToys Linux follows a **modular architecture** where each utility is:
- **Independently Developed**: Each utility is a separate module
- **Shared Framework**: Common UI components and system integration
- **Optional Installation**: Install only the utilities you need
- **Lightweight**: Minimal resource usage

### **Technology Stack**

**Main Application (Tauri + React):**
- **Tauri v2**: Secure, lightweight desktop framework
- **React 18**: Modern UI with TypeScript
- **GTK4 Integration**: Native Linux desktop integration
- **Shared Components**: Consistent UI across all utilities

**Individual Utilities (Rust):**
- **Standalone Binaries**: Each utility can run independently
- **System Integration**: Native Linux desktop hooks
- **IPC Communication**: Communicate with main application
- **Configuration Sharing**: Centralized settings management

## 🚀 Getting Started

### **System Requirements**

**Required:**
- **Linux Distribution**: Any modern distribution
- **Desktop Environment**: GNOME, KDE, XFCE, or others
- **Display Server**: X11 or Wayland support
- **Node.js** >= 18.0.0 (for development)
- **Rust** >= 1.70.0 (for building utilities)

**Recommended:**
- **GTK4**: For best native integration
- **libadwaita**: For GNOME-style applications
- **KDE Integration**: For KDE Plasma users

### **Installation**

#### **Development Build**
```bash
# Clone the repository
git clone https://github.com/yourusername/PowerToysLinux.git
cd PowerToysLinux

# Install dependencies
npm install

# Build standalone utilities
npm run utilities:build

# Run the main application
npm run app:dev
```

#### **Production Build**
```bash
# Build everything
npm run app:build
npm run utilities:build

# Install system-wide
sudo npm run utilities:install
```

## 🛠️ Development

### **Project Structure**
```
PowerToysLinux/
├── src/                        # Main Tauri application (React)
│   ├── components/             # Shared UI components
│   │   ├── common/            # Common components
│   │   └── utilities/         # Utility-specific components
│   ├── hooks/                 # Custom React hooks
│   ├── store/                 # State management (Zustand)
│   ├── types/                 # TypeScript definitions
│   └── utils/                 # Utility functions
├── src-tauri/                  # Tauri backend (Rust)
│   ├── src/
│   │   ├── utilities/         # Utility backend logic
│   │   ├── common/            # Shared backend code
│   │   └── main.rs            # Main application
│   └── Cargo.toml
├── utilities/                  # Standalone utility binaries
│   ├── power-rename/          # PowerRename utility
│   ├── fancy-zones/           # FancyZones utility
│   ├── color-picker/          # Color Picker utility
│   ├── keyboard-manager/      # Keyboard Manager utility
│   ├── power-launcher/        # PowerLauncher utility
│   └── screen-ruler/          # Screen Ruler utility
├── docs/                       # Documentation
├── scripts/                    # Build and install scripts
└── package.json
```

### **Adding New Utilities**

1. **Create Utility Directory**:
   ```bash
   mkdir utilities/my-utility
   cd utilities/my-utility
   cargo init
   ```

2. **Implement Core Logic**:
   ```rust
   // src/main.rs
   use powertoys_common::{UtilityConfig, UtilityTrait};
   
   struct MyUtility {
       config: UtilityConfig,
   }
   
   impl UtilityTrait for MyUtility {
       fn name(&self) -> &str { "My Utility" }
       fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
           // Implementation
           Ok(())
       }
   }
   ```

3. **Add UI Components**:
   ```tsx
   // src/components/utilities/MyUtility.tsx
   import React from 'react';
   import { invoke } from '@tauri-apps/api/core';
   
   export const MyUtility: React.FC = () => {
       return (
           <div>
               {/* Utility UI */}
           </div>
       );
   };
   ```

4. **Register Utility**:
   ```typescript
   // src/types/utilities.ts
   export interface UtilityRegistry {
       'my-utility': MyUtilityConfig;
   }
   ```

## 🎯 Roadmap

### **Phase 1: Foundation (Month 1-2)**
- ✅ Project architecture and tooling
- ✅ Shared component library
- 🔄 PowerRename utility (in progress)
- 🔄 Basic settings system

### **Phase 2: Core Utilities (Month 3-4)**
- 📝 Complete PowerRename
- 🪟 FancyZones window management
- 🎨 Color Picker tool
- 📐 Screen Ruler utility

### **Phase 3: Advanced Features (Month 5-6)**
- ⌨️ Keyboard Manager
- 🔍 PowerLauncher
- 🔧 System integration improvements
- 📦 Distribution packages

### **Phase 4: Polish & Release (Month 7+)**
- 🎨 UI/UX refinements
- 📚 Documentation completion
- 🧪 Testing and bug fixes
- 🚀 Stable release

## 🤝 Contributing

We welcome contributions from the Linux community! Whether you're interested in:
- **Utility Development**: Create new utilities
- **UI/UX Design**: Improve the user experience
- **Documentation**: Help with docs and tutorials
- **Testing**: Test on different Linux distributions
- **Translation**: Localize for different languages

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for detailed guidelines.

### **Development Setup**
```bash
# Fork and clone
git clone https://github.com/yourusername/PowerToysLinux.git
cd PowerToysLinux

# Install dependencies
npm install

# Start development
npm run app:dev

# Make your changes...

# Test and submit PR
npm run type-check
npm run lint
git commit -am "feat: add new feature"
```

## 🔧 Configuration

PowerToys Linux uses a **centralized configuration system**:

### **Configuration Location**
- **User Config**: `~/.config/powertoys-linux/config.toml`
- **System Config**: `/etc/powertoys-linux/config.toml`
- **Runtime Config**: In-memory settings for session

### **Example Configuration**
```toml
[general]
theme = "dark"
startup_with_system = true
show_notifications = true

[power_rename]
enabled = true
show_preview = true
remember_settings = true

[fancy_zones]
enabled = true
zone_border_color = "#0078d4"
zone_opacity = 0.5
multi_monitor = true

[color_picker]
enabled = true
default_format = "hex"
copy_to_clipboard = true

[keyboard_manager]
enabled = false  # Disabled by default for safety
```

## 📊 System Integration

### **Desktop Environment Support**
- **GNOME**: Native GTK4/libadwaita integration
- **KDE Plasma**: Qt-style theming support
- **XFCE**: Lightweight integration
- **Others**: Fallback X11/Wayland support

### **Autostart Integration**
```bash
# Enable autostart
powertoys-linux --enable-autostart

# Disable autostart
powertoys-linux --disable-autostart
```

### **Hotkey Registration**
Uses native Linux hotkey systems:
- **X11**: XGrabKey for global shortcuts
- **Wayland**: Compositor-specific protocols
- **Fallback**: User-space key monitoring

## 📜 Legal

### **License**
PowerToys Linux is released under the **MIT License**, ensuring:
- ✅ **Free and Open Source**: Always free to use and modify
- ✅ **Commercial Use**: Can be used in commercial environments
- ✅ **Distribution**: Can be packaged and distributed
- ✅ **Modification**: Can be forked and customized

### **Microsoft PowerToys**
This project is a **clean room implementation** inspired by Microsoft PowerToys:
- **No Code Copied**: All code written from scratch
- **Independent Development**: No direct dependency on Windows PowerToys
- **Linux-native Design**: Built specifically for Linux desktop environments
- **Respectful Homage**: Acknowledges inspiration while being independent

## 🙏 Acknowledgments

- **Microsoft PowerToys Team**: For the original concept and inspiration
- **Tauri Community**: For the excellent desktop application framework
- **Linux Desktop Developers**: For the foundation we build upon
- **Open Source Community**: For tools, libraries, and inspiration

## 📞 Support & Community

- **Issues**: [GitHub Issues](https://github.com/yourusername/PowerToysLinux/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/PowerToysLinux/discussions)
- **Matrix Chat**: `#powertoys-linux:matrix.org`
- **Reddit**: `/r/PowerToysLinux` (coming soon)

---

**Built with ❤️ by and for the Linux community**

*PowerToys Linux - Because Linux power users deserve powerful tools.*