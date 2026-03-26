# PortProton Enhanced - Launchers Documentation

## Overview

PortProton Enhanced includes **25+ custom autoinstall scripts** that provide easy installation and configuration for popular game launchers and direct game shortcuts. All scripts are optimized for RTX 4080 + hybrid graphics with Ubuntu 25.10 and KDE Plasma.

## Game Launcher Installers

### Major Gaming Platforms

#### EA App (`PW_EA_APP`)
- **Description**: EA's successor to Origin launcher
- **Games**: Battlefield, FIFA, The Sims, Mass Effect, Dragon Age
- **Features**: EA Play integration, cloud saves, achievements
- **Wine Compatibility**: Excellent with PROTON_LG

#### Battle.net (Built-in)
- **Description**: Blizzard's official game launcher  
- **Games**: World of Warcraft, Overwatch, Diablo, StarCraft
- **Features**: Social features, tournament integration
- **Note**: Uses existing PortProton Battle.net installer

#### Epic Games Store (Built-in)
- **Description**: Epic's game store and launcher
- **Games**: Fortnite, Rocket League, free weekly games
- **Features**: Epic Games Store integration, Unreal Engine games
- **Note**: Uses existing PortProton Epic installer

#### Steam (Built-in + Enhanced)
- **Description**: Valve's gaming platform
- **Enhanced Features**: Steam library auto-detection and direct game shortcuts
- **Steam Parser**: Reads appmanifest files for accurate game detection
- **Note**: Uses existing PortProton Steam installer + custom library integration

### Secondary Gaming Platforms

#### Riot Games Client (`PW_RIOT_GAMES`)
- **Description**: Riot's unified game launcher
- **Games**: Valorant, League of Legends, Teamfight Tactics, Legends of Runeterra
- **Features**: Riot ID integration, cross-game progression
- **Wine Compatibility**: Good with anti-cheat considerations

#### Amazon Games (`PW_AMAZON_GAMES`)
- **Description**: Amazon's gaming launcher and Prime Gaming
- **Games**: New World, Lost Ark, Prime Gaming titles
- **Features**: Prime Gaming integration, free games for Prime members
- **Wine Compatibility**: Good with PROTON_LG

#### Xbox App (`PW_XBOX_APP`)
- **Description**: Microsoft's PC gaming platform
- **Games**: Game Pass titles, Microsoft exclusives
- **Features**: Xbox Game Pass integration, cloud gaming
- **Wine Compatibility**: Limited due to UWP restrictions

#### GeForce Now (`PW_GEFORCE_NOW`)
- **Description**: NVIDIA's cloud gaming service
- **Games**: Stream games from your existing libraries
- **Features**: RTX cloud gaming, multiple platform integration
- **System Requirements**: Good internet connection required

### Specialty Launchers

#### Unity Hub (`PW_UNITY_HUB`)
- **Description**: Unity game engine and project manager
- **Use Case**: Game development, Unity projects
- **Features**: Project management, Unity version control
- **Wine Compatibility**: Excellent for development work

#### Discord (`PW_DISCORD`)
- **Description**: Popular gaming communication platform
- **Features**: Voice chat, screen sharing, game integration
- **Gaming Integration**: Rich presence, game overlay
- **Wine Compatibility**: Excellent

#### Humble Bundle App (`PW_HUMBLE_APP`)
- **Description**: Humble Bundle's game library manager
- **Games**: Humble Bundle purchases, DRM-free games
- **Features**: Library management, automatic updates
- **Wine Compatibility**: Very good

### Legacy and Niche Launchers

#### Arc (Perfect World) (`PW_ARC_PERFECT_WORLD`)
- **Description**: Perfect World's game launcher
- **Games**: Perfect World, Star Trek Online, Neverwinter
- **Features**: Perfect World account integration
- **Wine Compatibility**: Good

#### Bethesda Launcher (`PW_BETHESDA_LAUNCHER`)
- **Description**: Bethesda's game launcher (being phased out)
- **Games**: Fallout 76, The Elder Scrolls Online, older Bethesda titles
- **Note**: Being migrated to Steam, use for legacy games
- **Wine Compatibility**: Good

#### Activision Launcher (`PW_ACTIVISION_LAUNCHER`)
- **Description**: Activision's dedicated launcher
- **Games**: Call of Duty titles, Activision classics
- **Features**: Call of Duty integration
- **Wine Compatibility**: Good with anti-cheat considerations

### Content Creation and Modding

#### Twitch Desktop (`PW_TWITCH_DESKTOP`)
- **Description**: Twitch's desktop application
- **Features**: Stream management, mod integration
- **Gaming Integration**: Twitch integration, game streaming
- **Wine Compatibility**: Good

#### CurseForge (`PW_CURSEFORGE`)
- **Description**: Mod and addon manager
- **Games**: Minecraft, World of Warcraft, other moddable games
- **Features**: Automatic mod updates, modpack management
- **Wine Compatibility**: Excellent

#### MultiMC (`PW_MULTIMC`)
- **Description**: Advanced Minecraft launcher
- **Features**: Multiple instances, mod management, Java management
- **Gaming Integration**: Minecraft mod support
- **Wine Compatibility**: Excellent

## Direct Game Launchers

### Action/Adventure Games

#### Dead Island 2 Deluxe Edition (`PW_DEAD_ISLAND_2_DIRECT`)
- **Path**: `/media/lou/Games/Games/Dead Island 2 Deluxe Edition/DeadIsland-Win64-Shipping.exe`
- **Genre**: Action/Survival Horror
- **Wine Prefix**: `DEAD_ISLAND_2`
- **RTX Features**: Ray tracing, DLSS support

#### Mafia III Definitive Edition (`PW_MAFIA_III_DIRECT`)
- **Path**: `/media/lou/Games/Games/Mafia III- Definitive Edition/Mafia3DefinitiveEdition.exe`
- **Genre**: Action/Crime Drama
- **Wine Prefix**: `MAFIA_III_DE`
- **RTX Features**: Enhanced lighting, improved textures

#### Battlezone Combat Commander (`PW_BATTLEZONE_CC_DIRECT`)
- **Path**: `/media/lou/Games/Games/Battlezone Combat Commander/bzcc.exe`
- **Genre**: Strategy/Action
- **Wine Prefix**: `BATTLEZONE_CC`
- **Features**: Classic RTS gameplay

### RPG Games

#### Last Epoch (`PW_LAST_EPOCH_DIRECT`)
- **Path**: `/media/lou/Games/Games/Last Epoch/Last Epoch.exe`
- **Genre**: Action RPG
- **Wine Prefix**: `LAST_EPOCH`
- **Features**: Time-travel mechanics, deep character customization

#### Dark Deity Complete Edition (`PW_DARK_DEITY_DIRECT`)
- **Path**: `/media/lou/Games/Games/Dark Deity- Complete Edition/DarkDeity.exe`
- **Genre**: Tactical RPG
- **Wine Prefix**: `DARK_DEITY_CE`
- **Features**: Classic SRPG gameplay

#### Phoenix Point (`PW_PHOENIX_POINT_DIRECT`)
- **Path**: `/media/lou/Games/Games/PhoenixPointEpic/PhoenixPointWin64.exe`
- **Genre**: Turn-Based Strategy
- **Wine Prefix**: `PHOENIX_POINT`
- **Features**: XCOM-style gameplay with destructible environments

### Strategy Games

#### Star Wars Empire at War (`PW_STAR_WARS_EAW_DIRECT`)
- **Path**: `/media/lou/Games/Games/Star Wars Empire at War Gold Pack/corruption/StarWarsG.exe`
- **Genre**: Real-Time Strategy
- **Wine Prefix**: `SW_EMPIRE_AT_WAR`
- **Features**: Space and ground battles, Star Wars universe

## Auto-Detection Scripts

### Regular Games Scanner (`PW_AUTO_DETECT_GAMES`)
- **Purpose**: Scans `/media/lou/Games/Games` for Windows executables
- **Features**: 
  - Automatic executable detection
  - Wine prefix name generation
  - Bulk shortcut creation
- **Usage**: Detects all games in your regular games folder

### Steam Library Parser (`PW_STEAM_GAMES_PARSER`)
- **Purpose**: Reads Steam appmanifest files for proper game identification
- **Features**:
  - Steam appmanifest parsing
  - Proper game name extraction
  - Install directory detection
  - Executable identification
- **Usage**: Creates accurate Steam game shortcuts

### Universal Game Scanner (`PW_AUTO_DETECT_ALL_GAMES`)
- **Purpose**: Combines regular games and Steam library scanning
- **Features**:
  - Scans both game directories
  - Provides comprehensive game count
  - Identifies all Windows games
- **Usage**: One-click detection of all games on your system

## Installation and Usage

### Accessing Autoinstall Scripts

1. **Launch PortProton Enhanced**
2. **Click "AutoInstall"** in the main interface
3. **Select desired launcher** from the list
4. **Follow installation prompts**

### Script Locations

All scripts are installed in:
```bash
~/PortProton-Enhanced/data/scripts/pw_autoinstall/
```

### Manual Script Execution

You can run scripts directly from terminal:
```bash
# Run specific launcher installer
~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_EA_APP

# Run game detection
~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_AUTO_DETECT_ALL_GAMES

# Run Steam parser
~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_STEAM_GAMES_PARSER
```

## Optimization Features

### RTX 4080 Optimizations

All scripts include these optimizations:
```bash
export __GL_SHADER_DISK_CACHE=1
export __GL_THREADED_OPTIMIZATIONS=1
export __NV_PRIME_RENDER_OFFLOAD=1
export __GLX_VENDOR_LIBRARY_NAME=nvidia
export PW_VULKAN_USE="1"
```

### Wine Configuration

- **Default Wine**: PROTON_LG (optimized for gaming)
- **Vulkan**: Enabled by default
- **DXVK/VKD3D**: Automatic installation
- **ESYNC/FSYNC**: Enabled for better performance

### Gaming Enhancements

- **GameMode**: Automatic activation
- **MangoHUD**: Performance overlay available  
- **Hybrid Graphics**: Automatic NVIDIA prime offloading
- **CPU Optimization**: Multi-core topology configuration

## Troubleshooting Launchers

### Common Issues

#### Launcher Won't Install
```bash
# Check script permissions
ls -la ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_*

# Fix permissions if needed
chmod +x ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_*
```

#### Game Won't Launch
```bash
# Check game path exists
ls -la "/media/lou/Games/Games/Your Game/"

# Verify executable exists
find "/media/lou/Games/Games/Your Game/" -name "*.exe"

# Run auto-detection to refresh
~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_AUTO_DETECT_ALL_GAMES
```

#### Steam Games Not Detected
```bash
# Check Steam library path
ls -la /media/lou/Games/SteamLibrary/steamapps/

# Verify manifests exist
ls /media/lou/Games/SteamLibrary/steamapps/appmanifest_*.acf | wc -l

# Run Steam parser manually
~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_STEAM_GAMES_PARSER
```

### Performance Issues

#### Poor Gaming Performance
1. **Verify NVIDIA drivers**: `nvidia-smi`
2. **Check hybrid graphics**: `nvidia-settings`
3. **Enable GameMode**: `gamemoded -s`
4. **Monitor with MangoHUD**: Enable in launcher settings

#### Wine Prefix Issues
```bash
# Recreate Wine prefix
rm -rf ~/.local/share/wineprefixes/LAUNCHER_NAME/

# Reinstall launcher
~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_LAUNCHER_NAME
```

## Customization

### Adding Custom Games

To add your own games to the auto-detection:

1. **Place game** in `/media/lou/Games/Games/`
2. **Run detection**: Use `PW_AUTO_DETECT_ALL_GAMES`
3. **Manual addition**: Create custom script based on existing templates

### Creating Custom Launchers

Use existing scripts as templates:
```bash
# Copy existing script
cp ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_EA_APP \
   ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_MY_LAUNCHER

# Edit for your launcher
nano ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_MY_LAUNCHER

# Make executable
chmod +x ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_MY_LAUNCHER
```

## Future Enhancements

### Planned Features

- **GOG Galaxy integration**
- **More direct game launchers**
- **Automatic game update detection**
- **Enhanced Steam integration**
- **Custom game category management**

### Contributing

To contribute new launchers:

1. **Fork the repository**
2. **Create new launcher script** following existing patterns
3. **Test thoroughly** on Ubuntu 25.10 + RTX 4080
4. **Submit pull request** with documentation

---

**Enjoy gaming with PortProton Enhanced!** 🎮✨