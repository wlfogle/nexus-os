#!/bin/bash
# =============================================================================
# TiamatsStack APK Builder & Sideloader
# Builds both mobile and Fire TV flavors, optionally installs via ADB
#
# Prerequisites:
#   - Java 17+  (sudo nala install openjdk-17-jdk)
#   - Android SDK command-line tools  (or Android Studio)
#   - ANDROID_HOME set (e.g. ~/Android/Sdk)
#   - adb in PATH for sideloading
#
# Usage:
#   ./build-app.sh                  # build debug APKs
#   ./build-app.sh release          # build release APKs (needs keystore)
#   ./build-app.sh install-firetv   # build + sideload to all 3 Fire TVs
#   ./build-app.sh install-mobile   # build + install to connected phone
# =============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_MODE="${1:-debug}"

# Fire TV device IPs (must be reachable and ADB-enabled)
FIRETV_DEVICES=("192.168.12.51" "192.168.12.52" "192.168.12.53")

APK_MOBILE_DEBUG="$SCRIPT_DIR/app/build/outputs/apk/mobile/debug/app-mobile-debug.apk"
APK_FIRETV_DEBUG="$SCRIPT_DIR/app/build/outputs/apk/firetv/debug/app-firetv-debug.apk"
APK_MOBILE_RELEASE="$SCRIPT_DIR/app/build/outputs/apk/mobile/release/app-mobile-release-unsigned.apk"
APK_FIRETV_RELEASE="$SCRIPT_DIR/app/build/outputs/apk/firetv/release/app-firetv-release-unsigned.apk"

log() { echo "[$(date '+%H:%M:%S')] $*"; }

check_prereqs() {
  if ! command -v java &>/dev/null; then
    echo "ERROR: Java not found. Install with: sudo nala install openjdk-17-jdk"
    exit 1
  fi
  JAVA_VER=$(java -version 2>&1 | awk -F '"' '/version/ {print $2}' | cut -d. -f1)
  if [ "$JAVA_VER" -lt 17 ]; then
    echo "ERROR: Java 17+ required. Found Java $JAVA_VER"
    exit 1
  fi
  if [ -z "$ANDROID_HOME" ]; then
    # Try common locations
    for DIR in ~/Android/Sdk /opt/android-sdk /usr/lib/android-sdk; do
      if [ -d "$DIR" ]; then
        export ANDROID_HOME="$DIR"
        break
      fi
    done
    if [ -z "$ANDROID_HOME" ]; then
      echo "ERROR: ANDROID_HOME not set and Android SDK not found in common locations."
      echo "  Set ANDROID_HOME or install Android Studio."
      exit 1
    fi
  fi
  log "Using Android SDK: $ANDROID_HOME"
  log "Java version: $(java -version 2>&1 | head -1)"
}

build_debug() {
  log "Building debug APKs (mobile + firetv)..."
  "$SCRIPT_DIR/gradlew" assembleDebug
  log "APKs:"
  log "  Mobile:  $APK_MOBILE_DEBUG"
  log "  Fire TV: $APK_FIRETV_DEBUG"
}

build_release() {
  log "Building release APKs..."
  if [ -z "$KEYSTORE_PATH" ] || [ -z "$KEYSTORE_PASS" ] || [ -z "$KEY_ALIAS" ] || [ -z "$KEY_PASS" ]; then
    log "WARNING: KEYSTORE_PATH/KEYSTORE_PASS/KEY_ALIAS/KEY_PASS not set."
    log "         Building unsigned release APKs."
    "$SCRIPT_DIR/gradlew" assembleRelease
  else
    "$SCRIPT_DIR/gradlew" assembleRelease \
      -Pandroid.injected.signing.store.file="$KEYSTORE_PATH" \
      -Pandroid.injected.signing.store.password="$KEYSTORE_PASS" \
      -Pandroid.injected.signing.key.alias="$KEY_ALIAS" \
      -Pandroid.injected.signing.key.password="$KEY_PASS"
  fi
  log "Release APKs built."
}

install_firetv() {
  local apk="$APK_FIRETV_DEBUG"
  [ "$BUILD_MODE" = "release" ] && apk="$APK_FIRETV_RELEASE"

  if [ ! -f "$apk" ]; then
    log "APK not found at $apk — building first..."
    build_debug
  fi

  if ! command -v adb &>/dev/null; then
    echo "ERROR: adb not found. Install android-tools: sudo nala install android-tools-adb"
    exit 1
  fi

  for IP in "${FIRETV_DEVICES[@]}"; do
    log "Connecting to Fire TV at $IP..."
    adb connect "$IP:5555" || { log "WARN: Could not connect to $IP — skipping"; continue; }
    sleep 1
    log "Installing TiamatsStack on $IP..."
    adb -s "$IP:5555" install -r "$apk" && log "  ✓ Installed on $IP" || log "  ✗ Failed on $IP"
  done

  log "Fire TV sideload complete."
  log ""
  log "If ADB is not enabled on your Fire TVs:"
  log "  Settings → My Fire TV → Developer Options → ADB Debugging: ON"
  log "  Then run: adb connect <firetv-ip>:5555"
}

install_mobile() {
  local apk="$APK_MOBILE_DEBUG"
  if ! command -v adb &>/dev/null; then
    echo "ERROR: adb not found."
    exit 1
  fi
  log "Installing mobile APK on connected device..."
  adb install -r "$apk" && log "✓ Installed" || log "✗ Failed"
}

# ── Download Gradle wrapper if missing ──────────────────────────────────────
if [ ! -f "$SCRIPT_DIR/gradlew" ]; then
  log "Downloading Gradle wrapper..."
  curl -fsSL "https://services.gradle.org/distributions/gradle-8.4-bin.zip" -o /tmp/gradle.zip
  mkdir -p /tmp/gradle-extract
  unzip -q /tmp/gradle.zip -d /tmp/gradle-extract
  GRADLE_BIN=$(find /tmp/gradle-extract -name "gradle" -type f | head -1)
  "$GRADLE_BIN" wrapper --gradle-version 8.4 -p "$SCRIPT_DIR"
  chmod +x "$SCRIPT_DIR/gradlew"
fi

# ── Main ────────────────────────────────────────────────────────────────────
check_prereqs

case "$BUILD_MODE" in
  debug)
    build_debug
    ;;
  release)
    build_release
    ;;
  install-firetv)
    build_debug
    install_firetv
    ;;
  install-mobile)
    build_debug
    install_mobile
    ;;
  *)
    echo "Usage: $0 [debug|release|install-firetv|install-mobile]"
    exit 1
    ;;
esac

log "Done."
