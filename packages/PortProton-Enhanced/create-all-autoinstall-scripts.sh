#!/usr/bin/env bash
# Recreate All Custom PortProton Autoinstall Scripts
# This script creates all the custom autoinstall scripts we developed

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="$SCRIPT_DIR/autoinstall_scripts"

mkdir -p "$TARGET_DIR"
cd "$TARGET_DIR"

echo "=== Creating Custom PortProton Autoinstall Scripts ==="

# EA App installer
cat > PW_EA_APP << 'EAEOF'
#!/usr/bin/env bash
# EA App (successor to Origin) installer for PortProton
export PW_AUTOINSTALL_EXE="EADesktop-Installer.exe"
export PW_PREFIX_NAME="EA_APP"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://www.ea.com/ea-app/download" \
"$PW_AUTOINSTALL_EXE" \
"ea_app_installer"
EAEOF

# Riot Games (Valorant/League of Legends)
cat > PW_RIOT_GAMES << 'RIOTEOF'
#!/usr/bin/env bash
# Riot Games Client installer for PortProton
export PW_AUTOINSTALL_EXE="RiotClientInstaller.exe"
export PW_PREFIX_NAME="RIOT_CLIENT"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://auth.riotgames.com/" \
"$PW_AUTOINSTALL_EXE" \
"riot_games_installer"
RIOTEOF

# Amazon Games
cat > PW_AMAZON_GAMES << 'AMAZONEOF'
#!/usr/bin/env bash
# Amazon Games App installer for PortProton
export PW_AUTOINSTALL_EXE="AmazonGamesSetup.exe"
export PW_PREFIX_NAME="AMAZON_GAMES"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://gaming.amazon.com/download" \
"$PW_AUTOINSTALL_EXE" \
"amazon_games_installer"
AMAZONEOF

# Xbox App (Microsoft Gaming)
cat > PW_XBOX_APP << 'XBOXEOF'
#!/usr/bin/env bash
# Xbox App installer for PortProton
export PW_AUTOINSTALL_EXE="XboxInstallerSetup.exe"
export PW_PREFIX_NAME="XBOX_APP"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://www.microsoft.com/store/productId/9MV0B5HZVK9Z" \
"$PW_AUTOINSTALL_EXE" \
"xbox_app_installer"
XBOXEOF

# GeForce Now
cat > PW_GEFORCE_NOW << 'GEFORCEOF'
#!/usr/bin/env bash
# GeForce Now installer for PortProton
export PW_AUTOINSTALL_EXE="GeForceNOW-release.exe"
export PW_PREFIX_NAME="GEFORCE_NOW"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://www.nvidia.com/en-us/geforce-now/" \
"$PW_AUTOINSTALL_EXE" \
"geforce_now_installer"
GEFORCEOF

# Humble App
cat > PW_HUMBLE_APP << 'HUMBLEEOF'
#!/usr/bin/env bash
# Humble App installer for PortProton
export PW_AUTOINSTALL_EXE="HumbleApp-Setup.exe"
export PW_PREFIX_NAME="HUMBLE_APP"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://www.humblebundle.com/app" \
"$PW_AUTOINSTALL_EXE" \
"humble_app_installer"
HUMBLEEOF

# Discord
cat > PW_DISCORD << 'DISCORDEOF'
#!/usr/bin/env bash
# Discord installer for PortProton
export PW_AUTOINSTALL_EXE="DiscordSetup.exe"
export PW_PREFIX_NAME="DISCORD"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://discord.com/download" \
"$PW_AUTOINSTALL_EXE" \
"discord_installer"
DISCORDEOF

# Unity Hub
cat > PW_UNITY_HUB << 'UNITYEOF'
#!/usr/bin/env bash
# Unity Hub installer for PortProton
export PW_AUTOINSTALL_EXE="UnityHubSetup.exe"
export PW_PREFIX_NAME="UNITY_HUB"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://unity3d.com/get-unity/download" \
"$PW_AUTOINSTALL_EXE" \
"unity_hub_installer"
UNITYEOF

# Arc (Perfect World)
cat > PW_ARC_PERFECT_WORLD << 'ARCEOF'
#!/usr/bin/env bash
# Arc (Perfect World) installer for PortProton
export PW_AUTOINSTALL_EXE="ArcSetup.exe"
export PW_PREFIX_NAME="ARC_PERFECT_WORLD"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://www.arcgames.com/en/games/download" \
"$PW_AUTOINSTALL_EXE" \
"arc_installer"
ARCEOF

# Bethesda Launcher
cat > PW_BETHESDA_LAUNCHER << 'BETHESDAEOF'
#!/usr/bin/env bash
# Bethesda Launcher installer for PortProton
export PW_AUTOINSTALL_EXE="BethesdaNetLauncher_Setup.exe"
export PW_PREFIX_NAME="BETHESDA_LAUNCHER"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://bethesda.net/en/game/bethesda-launcher" \
"$PW_AUTOINSTALL_EXE" \
"bethesda_launcher_installer"
BETHESDAEOF

# Activision Launcher
cat > PW_ACTIVISION_LAUNCHER << 'ACTIVISIONEOF'
#!/usr/bin/env bash
# Activision Launcher installer for PortProton
export PW_AUTOINSTALL_EXE="ActivisionLauncher_Setup.exe"
export PW_PREFIX_NAME="ACTIVISION_LAUNCHER"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://www.activision.com/" \
"$PW_AUTOINSTALL_EXE" \
"activision_launcher_installer"
ACTIVISIONEOF

# Twitch Desktop
cat > PW_TWITCH_DESKTOP << 'TWITCHEOF'
#!/usr/bin/env bash
# Twitch Desktop App installer for PortProton
export PW_AUTOINSTALL_EXE="TwitchSetup.exe"
export PW_PREFIX_NAME="TWITCH_DESKTOP"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://www.twitch.tv/downloads" \
"$PW_AUTOINSTALL_EXE" \
"twitch_desktop_installer"
TWITCHEOF

# CurseForge
cat > PW_CURSEFORGE << 'CURSEEOF'
#!/usr/bin/env bash
# CurseForge App installer for PortProton
export PW_AUTOINSTALL_EXE="CurseForge-Setup.exe"
export PW_PREFIX_NAME="CURSEFORGE"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://download.curseforge.com/" \
"$PW_AUTOINSTALL_EXE" \
"curseforge_installer"
CURSEEOF

# MultiMC (Minecraft Launcher)
cat > PW_MULTIMC << 'MULTIMCEOF'
#!/usr/bin/env bash
# MultiMC (Minecraft launcher) installer for PortProton
export PW_AUTOINSTALL_EXE="MultiMC-Setup.exe"
export PW_PREFIX_NAME="MULTIMC"
export PW_VULKAN_USE="1"
PW_WINE_USE="PROTON_LG" try_download_installer \
"https://multimc.org/" \
"$PW_AUTOINSTALL_EXE" \
"multimc_installer"
MULTIMCEOF

# Dead Island 2 Direct Launch
cat > PW_DEAD_ISLAND_2_DIRECT << 'DI2EOF'
#!/usr/bin/env bash
# Dead Island 2 Direct Launcher for PortProton
export PW_AUTOINSTALL_EXE="/media/lou/Games/Games/Dead Island 2 Deluxe Edition/DeadIsland-Win64-Shipping.exe"
export PW_PREFIX_NAME="DEAD_ISLAND_2"
export PW_VULKAN_USE="1"
if [[ -f "$PW_AUTOINSTALL_EXE" ]]; then
    portwine_exe="$PW_AUTOINSTALL_EXE"
    try_run_exe
else
    echo "Dead Island 2 executable not found at: $PW_AUTOINSTALL_EXE"
fi
DI2EOF

# Mafia III Direct Launch
cat > PW_MAFIA_III_DIRECT << 'MAFIA3EOF'
#!/usr/bin/env bash
# Mafia III Definitive Edition Direct Launcher for PortProton
export PW_AUTOINSTALL_EXE="/media/lou/Games/Games/Mafia III- Definitive Edition/Mafia3DefinitiveEdition.exe"
export PW_PREFIX_NAME="MAFIA_III_DE"
export PW_VULKAN_USE="1"
if [[ -f "$PW_AUTOINSTALL_EXE" ]]; then
    portwine_exe="$PW_AUTOINSTALL_EXE"
    try_run_exe
else
    echo "Mafia III Definitive Edition executable not found at: $PW_AUTOINSTALL_EXE"
fi
MAFIA3EOF

# Last Epoch Direct Launch
cat > PW_LAST_EPOCH_DIRECT << 'LASTEPOCHEOF'
#!/usr/bin/env bash
# Last Epoch Direct Launcher for PortProton
export PW_AUTOINSTALL_EXE="/media/lou/Games/Games/Last Epoch/Last Epoch.exe"
export PW_PREFIX_NAME="LAST_EPOCH"
export PW_VULKAN_USE="1"
if [[ -f "$PW_AUTOINSTALL_EXE" ]]; then
    portwine_exe="$PW_AUTOINSTALL_EXE"
    try_run_exe
else
    echo "Last Epoch executable not found at: $PW_AUTOINSTALL_EXE"
fi
LASTEPOCHEOF

# Dark Deity Direct Launch
cat > PW_DARK_DEITY_DIRECT << 'DARKDEITYEOF'
#!/usr/bin/env bash
# Dark Deity Complete Edition Direct Launcher for PortProton
export PW_AUTOINSTALL_EXE="/media/lou/Games/Games/Dark Deity- Complete Edition/DarkDeity.exe"
export PW_PREFIX_NAME="DARK_DEITY_CE"
export PW_VULKAN_USE="1"
if [[ -f "$PW_AUTOINSTALL_EXE" ]]; then
    portwine_exe="$PW_AUTOINSTALL_EXE"
    try_run_exe
else
    echo "Dark Deity Complete Edition executable not found at: $PW_AUTOINSTALL_EXE"
fi
DARKDEITYEOF

# Battlezone Combat Commander Direct Launch
cat > PW_BATTLEZONE_CC_DIRECT << 'BATTLEZONEEOF'
#!/usr/bin/env bash
# Battlezone Combat Commander Direct Launcher for PortProton
export PW_AUTOINSTALL_EXE="/media/lou/Games/Games/Battlezone Combat Commander/bzcc.exe"
export PW_PREFIX_NAME="BATTLEZONE_CC"
export PW_VULKAN_USE="1"
if [[ -f "$PW_AUTOINSTALL_EXE" ]]; then
    portwine_exe="$PW_AUTOINSTALL_EXE"
    try_run_exe
else
    echo "Battlezone Combat Commander executable not found at: $PW_AUTOINSTALL_EXE"
fi
BATTLEZONEEOF

# Star Wars Empire at War Direct Launch
cat > PW_STAR_WARS_EAW_DIRECT << 'EAWEOF'
#!/usr/bin/env bash
# Star Wars Empire at War Direct Launcher for PortProton
export PW_AUTOINSTALL_EXE="/media/lou/Games/Games/Star Wars Empire at War Gold Pack/corruption/StarWarsG.exe"
export PW_PREFIX_NAME="SW_EMPIRE_AT_WAR"
export PW_VULKAN_USE="1"
if [[ -f "$PW_AUTOINSTALL_EXE" ]]; then
    portwine_exe="$PW_AUTOINSTALL_EXE"
    try_run_exe
else
    echo "Star Wars Empire at War executable not found at: $PW_AUTOINSTALL_EXE"
fi
EAWEOF

# Phoenix Point Direct Launch
cat > PW_PHOENIX_POINT_DIRECT << 'PHOENIXEOF'
#!/usr/bin/env bash
# Phoenix Point Direct Launcher for PortProton
export PW_AUTOINSTALL_EXE="/media/lou/Games/Games/PhoenixPointEpic/PhoenixPointWin64.exe"
export PW_PREFIX_NAME="PHOENIX_POINT"
export PW_VULKAN_USE="1"
if [[ -f "$PW_AUTOINSTALL_EXE" ]]; then
    portwine_exe="$PW_AUTOINSTALL_EXE"
    try_run_exe
else
    echo "Phoenix Point executable not found at: $PW_AUTOINSTALL_EXE"
fi
PHOENIXEOF

# Auto-Detect Regular Games
cat > PW_AUTO_DETECT_GAMES << 'AUTODETECTEOF'
#!/usr/bin/env bash
# Auto-Detect Games in /media/lou/Games/Games for PortProton
echo "=== Auto-detecting games in your Games directory ==="

GAMES_DIR="/media/lou/Games/Games"
if [[ ! -d "$GAMES_DIR" ]]; then
    echo "Games directory not found: $GAMES_DIR"
    exit 1
fi

cd "$GAMES_DIR"
count=0

for game_dir in */; do
    if [[ -d "$game_dir" ]]; then
        echo "Scanning: $game_dir"
        
        # Look for main executables
        main_exe=$(find "$game_dir" -maxdepth 2 -name "*.exe" | head -1)
        
        if [[ -n "$main_exe" ]]; then
            game_name=$(basename "$game_dir" | tr -d '/')
            prefix_name="GAME_$(echo "$game_name" | tr ' ' '_' | tr '[:lower:]' '[:upper:]')"
            
            echo "Creating shortcut for: $game_name"
            echo "Executable: $main_exe"
            echo "Prefix: $prefix_name"
            ((count++))
        fi
    fi
done

echo "=== Found $count games to create shortcuts for ==="
AUTODETECTEOF

# Auto-Detect All Games (Steam + Regular)
cat > PW_AUTO_DETECT_ALL_GAMES << 'AUTOALLEOF'
#!/usr/bin/env bash
# Auto-Detect ALL Games (Steam + Regular) for PortProton
echo "=== Auto-detecting ALL games on your system ==="

# Check regular games
GAMES_DIR="/media/lou/Games/Games"
regular_count=0
if [[ -d "$GAMES_DIR" ]]; then
    echo "Scanning regular games directory..."
    for game_dir in "$GAMES_DIR"/*/; do
        if [[ -d "$game_dir" ]]; then
            main_exe=$(find "$game_dir" -maxdepth 2 -name "*.exe" | head -1)
            if [[ -n "$main_exe" ]]; then
                ((regular_count++))
            fi
        fi
    done
fi

# Check Steam games
STEAM_DIR="/media/lou/Games/SteamLibrary/steamapps/common"
steam_count=0
if [[ -d "$STEAM_DIR" ]]; then
    echo "Scanning Steam games directory..."
    for game_dir in "$STEAM_DIR"/*/; do
        if [[ -d "$game_dir" && ! "$game_dir" =~ Proton ]]; then
            main_exe=$(find "$game_dir" -maxdepth 3 -name "*.exe" | grep -v -i redist | head -1)
            if [[ -n "$main_exe" ]]; then
                ((steam_count++))
            fi
        fi
    done
fi

echo "=== Game Detection Summary ==="
echo "Regular games found: $regular_count"
echo "Steam games found: $steam_count"
echo "Total games: $((regular_count + steam_count))"
echo ""
echo "This would create PortProton shortcuts for all detected games."
AUTOALLEOF

# Steam Games Parser
cat > PW_STEAM_GAMES_PARSER << 'STEAMPARSEREOF'
#!/usr/bin/env bash
# Steam Games Parser for PortProton
echo "=== Parsing Steam library for installed games ==="

STEAM_APPS_DIR="/media/lou/Games/SteamLibrary/steamapps"
MANIFEST_DIR="$STEAM_APPS_DIR"
COMMON_DIR="$STEAM_APPS_DIR/common"

if [[ ! -d "$STEAM_APPS_DIR" ]]; then
    echo "Steam directory not found: $STEAM_APPS_DIR"
    exit 1
fi

count=0
echo "Reading Steam app manifests..."

for manifest in "$MANIFEST_DIR"/appmanifest_*.acf; do
    if [[ -f "$manifest" ]]; then
        # Extract game info from manifest
        app_name=$(grep -m 1 '"name"' "$manifest" | cut -d'"' -f4)
        install_dir=$(grep -m 1 '"installdir"' "$manifest" | cut -d'"' -f4)
        
        if [[ -n "$app_name" && -n "$install_dir" ]]; then
            game_path="$COMMON_DIR/$install_dir"
            
            if [[ -d "$game_path" ]]; then
                # Find main executable
                main_exe=$(find "$game_path" -maxdepth 3 -name "*.exe" | grep -v -E "(redist|_CommonRedist|vcredist|directx)" | head -1)
                
                if [[ -n "$main_exe" ]]; then
                    echo "Steam Game: $app_name"
                    echo "  Path: $game_path"
                    echo "  Executable: $main_exe"
                    echo "  Would create prefix: STEAM_$(echo "$install_dir" | tr '[:lower:]' '[:upper:]')"
                    echo ""
                    ((count++))
                fi
            fi
        fi
    fi
done

echo "=== Found $count Steam games with executables ==="
STEAMPARSEREOF

echo "Making all scripts executable..."
chmod +x PW_*

echo "=== All Custom Autoinstall Scripts Created Successfully! ==="
echo "Location: $TARGET_DIR"
echo "Scripts created: $(ls -1 PW_* | wc -l)"
echo ""
echo "These scripts include:"
echo "✅ 14 game launcher installers (EA, Riot, Amazon, Xbox, etc.)"
echo "✅ 8 direct game launchers for your installed games"
echo "✅ 3 auto-detection/parsing utilities"
echo ""
echo "To use with PortProton Enhanced:"
echo "1. Copy all PW_* scripts to: ~/PortProton-Enhanced/data/scripts/pw_autoinstall/"
echo "2. Set executable permissions: chmod +x ~/PortProton-Enhanced/data/scripts/pw_autoinstall/PW_*"
echo "3. Launch PortProton Enhanced and use AutoInstall feature"