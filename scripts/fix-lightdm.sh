#!/usr/bin/env bash
# =============================================================================
# NexusOS LightDM Fix — minimal, no chroot, no apt, no initramfs rebuild
# Extracts squashfs, patches LightDM config + permissions, repacks ISO
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$(dirname "$SCRIPT_DIR")/build"
WORK="/tmp/nexusos-lightdm-fix-$$"
ROOTFS="$WORK/rootfs"
ISO_MNT="$WORK/mnt"
ISO_NEW="$WORK/iso-new"

RED='\033[0;31m'; GREEN='\033[0;32m'; NC='\033[0m'
log() { echo -e "${GREEN}[fix]${NC} $*"; }
die() { echo -e "${RED}[error]${NC} $*"; rm -rf "$WORK"; exit 1; }

ISO="${1:-$(ls -t "$BUILD_DIR"/nexusos-*.iso 2>/dev/null | head -1)}"
[[ -f "$ISO" ]] || die "No ISO found. Usage: sudo $0 [path/to/nexusos.iso]"
log "Fixing: $ISO"

mkdir -p "$ISO_MNT" "$ROOTFS" "$ISO_NEW"

# ── Step 1: Extract ISO + squashfs ───────────────────────────────────────────
log "Mounting ISO..."
mount -o loop,ro "$ISO" "$ISO_MNT"

log "Copying ISO structure..."
rsync -a --exclude='casper/filesystem.squashfs' "$ISO_MNT/" "$ISO_NEW/"

log "Extracting squashfs (few minutes)..."
unsquashfs -f -d "$ROOTFS" "$ISO_MNT/casper/filesystem.squashfs"
umount "$ISO_MNT"

# ── Step 2: Apply fixes (no chroot needed) ───────────────────────────────────
log "Fix 1: /home/nexus ownership..."
chown 1000:1000 "$ROOTFS/home/nexus"
chmod 755 "$ROOTFS/home/nexus"

log "Fix 2: Remove XAUTHORITY + LIBGL_ALWAYS_SOFTWARE from /etc/environment..."
sed -i '/^XAUTHORITY=/d'          "$ROOTFS/etc/environment" 2>/dev/null || true
sed -i '/^LIBGL_ALWAYS_SOFTWARE=/d' "$ROOTFS/etc/environment" 2>/dev/null || true

log "Fix 3: LightDM autologin + user-authority-in-system-dir..."
rm -f "$ROOTFS/etc/lightdm/lightdm.conf"
mkdir -p "$ROOTFS/etc/lightdm/lightdm.conf.d"
cat > "$ROOTFS/etc/lightdm/lightdm.conf.d/50-nexus-autologin.conf" << 'EOF'
[LightDM]
user-authority-in-system-dir=true

[Seat:*]
autologin-user=nexus
autologin-user-timeout=0
user-session=plasma
greeter-session=lightdm-gtk-greeter
EOF

log "Fix 4: PAM lightdm-autologin (pam_loginuid optional)..."
cat > "$ROOTFS/etc/pam.d/lightdm-autologin" << 'EOF'
auth required pam_succeed_if.so user != root quiet_success
auth required pam_permit.so
@include common-account
session optional pam_loginuid.so
session required pam_limits.so
@include common-session
@include common-password
EOF

log "Fix 5: Xorg log directory..."
mkdir -p "$ROOTFS/home/nexus/.local/share/xorg"
chown -R 1000:1000 "$ROOTFS/home/nexus/.local"
chmod 755 "$ROOTFS/home/nexus/.local/share/xorg"

# ── Step 3: Repack squashfs ───────────────────────────────────────────────────
log "Repacking squashfs..."
mksquashfs "$ROOTFS" "$ISO_NEW/casper/filesystem.squashfs" \
    -comp xz -Xbcj x86 -b 1M -Xdict-size 1M -noappend
du -sx --block-size=1 "$ROOTFS" | awk '{print $1}' > "$ISO_NEW/casper/filesystem.size"

# ── Step 4: Rebuild ISO ───────────────────────────────────────────────────────
OUTPUT="$ISO"  # overwrite in-place

log "Rebuilding ISO..."
if [[ -f "$ISO_NEW/isolinux/isolinux.bin" ]] && [[ -f "$ISO_NEW/boot/grub/efi.img" ]]; then
    isohdpfx=""
    for p in /usr/lib/ISOLINUX/isohdpfx.bin /usr/lib/syslinux/mbr/isohdpfx.bin; do
        [[ -f "$p" ]] && { isohdpfx="$p"; break; }
    done
    xorriso -as mkisofs \
        -volid "NEXUSOS_1_0" -J -R -l -iso-level 3 \
        -b isolinux/isolinux.bin -c isolinux/boot.cat \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        ${isohdpfx:+-isohybrid-mbr "$isohdpfx"} \
        -eltorito-alt-boot -e boot/grub/efi.img -no-emul-boot \
        -isohybrid-gpt-basdat \
        -append_partition 2 0xEF "$ISO_NEW/boot/grub/efi.img" \
        -o "$OUTPUT" "$ISO_NEW"
elif [[ -f "$ISO_NEW/boot/grub/efi.img" ]]; then
    xorriso -as mkisofs \
        -volid "NEXUSOS_1_0" -J -R -l -iso-level 3 \
        -eltorito-boot boot/grub/efi.img -no-emul-boot \
        -eltorito-catalog boot/grub/boot.cat \
        -append_partition 2 0xEF "$ISO_NEW/boot/grub/efi.img" \
        -o "$OUTPUT" "$ISO_NEW"
else
    die "No bootloader images found"
fi

rm -rf "$WORK"
log "Done: $OUTPUT ($(du -h "$OUTPUT" | awk '{print $1}'))"
