#!/usr/bin/env bash
# =============================================================================
# NexusOS ISO Patcher — Apply fixes to existing ISO without full rebuild
# Extracts squashfs, applies fixes in chroot, repacks ISO
# =============================================================================
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
readonly BUILD_DIR="${PROJECT_ROOT}/build"
readonly WORK="/tmp/nexusos-patch-$$"
readonly UNSQUASH="${WORK}/rootfs"
readonly ISO_MNT="${WORK}/iso-mnt"
readonly ISO_NEW="${WORK}/iso-new"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
log()  { echo -e "${GREEN}[patch]${NC} $*"; }
err()  { echo -e "${RED}[error]${NC} $*"; }
die()  { err "$*"; cleanup; exit 1; }

cleanup() {
    log "Cleaning up..."
    umount -lf "${UNSQUASH}/dev/pts" 2>/dev/null || true
    umount -lf "${UNSQUASH}/dev"     2>/dev/null || true
    umount -lf "${UNSQUASH}/proc"    2>/dev/null || true
    umount -lf "${UNSQUASH}/sys"     2>/dev/null || true
    umount -lf "${UNSQUASH}/run"     2>/dev/null || true
    umount -lf "${UNSQUASH}/tmp"     2>/dev/null || true
    umount -lf "${ISO_MNT}"          2>/dev/null || true
    rm -rf "$WORK"
}
trap cleanup EXIT

mount_chroot() {
    mount --bind /dev     "${UNSQUASH}/dev"
    mount --bind /dev/pts "${UNSQUASH}/dev/pts"
    mount -t proc proc    "${UNSQUASH}/proc"
    mount -t sysfs sys    "${UNSQUASH}/sys"
    mount -t tmpfs tmpfs  "${UNSQUASH}/run"
    mount -t tmpfs tmpfs  "${UNSQUASH}/tmp"
    local resolv="${UNSQUASH}/etc/resolv.conf"
    [[ -L "$resolv" ]] && rm -f "$resolv"
    cp /etc/resolv.conf "$resolv"
}

umount_chroot() {
    umount -lf "${UNSQUASH}/dev/pts" 2>/dev/null || true
    umount -lf "${UNSQUASH}/dev"     2>/dev/null || true
    umount -lf "${UNSQUASH}/proc"    2>/dev/null || true
    umount -lf "${UNSQUASH}/sys"     2>/dev/null || true
    umount -lf "${UNSQUASH}/run"     2>/dev/null || true
    umount -lf "${UNSQUASH}/tmp"     2>/dev/null || true
}

run_in_chroot() {
    chroot "$UNSQUASH" /bin/bash -c "$1"
}

# ── Find existing ISO ────────────────────────────────────────────────────
ISO_PATH="${1:-}"
if [[ -z "$ISO_PATH" ]]; then
    ISO_PATH="$(ls -t "${BUILD_DIR}"/nexusos-*.iso 2>/dev/null | head -1)"
fi
[[ -f "$ISO_PATH" ]] || die "No ISO found. Usage: sudo $0 [path/to/nexusos.iso]"

log "Patching: ${ISO_PATH}"

# ── Step 1: Extract ISO contents ─────────────────────────────────────────
mkdir -p "$ISO_MNT" "$ISO_NEW" "$UNSQUASH"

log "Mounting ISO..."
mount -o loop,ro "$ISO_PATH" "$ISO_MNT"

log "Copying ISO structure..."
rsync -a --exclude='casper/filesystem.squashfs' "$ISO_MNT/" "$ISO_NEW/"

log "Extracting squashfs (this takes a few minutes)..."
unsquashfs -f -d "$UNSQUASH" "${ISO_MNT}/casper/filesystem.squashfs"

umount "$ISO_MNT"

# ── Step 2: Apply fixes in chroot ────────────────────────────────────────
mount_chroot

log "Fixing portage (typing_extensions)..."
run_in_chroot "
    set -e

    # Remove stale system typing_extensions so pip's build isolation
    # doesn't pick up the old jammy version missing 'Required'
    rm -f /usr/lib/python3/dist-packages/typing_extensions.py
    rm -rf /usr/lib/python3/dist-packages/typing_extensions-*.egg-info
    pip3 install --upgrade --force-reinstall typing_extensions

    if ! command -v emerge &>/dev/null; then
        export DEBIAN_FRONTEND=noninteractive
        pip3 install 'meson>=1.0,<1.4' ninja 'meson-python<0.16' 'packaging<24' 'pyproject-metadata<0.8'
        cd /tmp
        rm -rf portage-src
        git clone --depth 1 https://github.com/gentoo/portage.git portage-src
        cd portage-src
        pip3 install --no-build-isolation .
        command -v emerge >/dev/null && echo '[OK] portage/emerge installed'
        cd / && rm -rf /tmp/portage-src
    else
        echo '[OK] portage/emerge already present'
    fi
"

# Replace SDDM with LightDM — SDDM has xauth/flock issues on casper overlayfs
log "Switching to LightDM..."
run_in_chroot "
    set -e
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq lightdm lightdm-gtk-greeter
    systemctl disable sddm 2>/dev/null || true
    systemctl enable lightdm
    echo '/usr/sbin/lightdm' > /etc/X11/default-display-manager
    systemctl set-default graphical.target
    groupadd -f autologin
    usermod -aG autologin nexus 2>/dev/null || true
    echo '[OK] LightDM installed and enabled'
"

# LightDM autologin for live session
mkdir -p "${UNSQUASH}/etc/lightdm/lightdm.conf.d"
cat > "${UNSQUASH}/etc/lightdm/lightdm.conf.d/50-nexus-autologin.conf" << 'LDMEOF'
[LightDM]
user-authority-in-system-dir=true

[Seat:*]
autologin-user=nexus
autologin-user-timeout=0
user-session=plasma
greeter-session=lightdm-gtk-greeter
LDMEOF

# PAM autologin config for LightDM
cat > "${UNSQUASH}/etc/pam.d/lightdm-autologin" << 'PAMEOF'
auth required pam_succeed_if.so user != root quiet_success
auth required pam_permit.so
@include common-account
session optional pam_loginuid.so
session required pam_limits.so
@include common-session
@include common-password
PAMEOF

# Remove any XAUTHORITY override from /etc/environment — LightDM manages this
# itself and any override breaks the X authority cookie handoff
log "Removing stale XAUTHORITY/LIBGL overrides from /etc/environment..."
sed -i '/^XAUTHORITY=/d' "${UNSQUASH}/etc/environment" 2>/dev/null || true
sed -i '/^LIBGL_ALWAYS_SOFTWARE=/d' "${UNSQUASH}/etc/environment" 2>/dev/null || true

# KWin software rendering fallback for VMs without GPU acceleration
log "Adding KWin VM rendering fallback..."
mkdir -p "${UNSQUASH}/etc/xdg"
cat > "${UNSQUASH}/etc/xdg/kwinrc" << 'KWINEOF'
[Compositing]
Backend=XRender
GLCore=false
OpenGLIsUnsafe=false
KWINEOF

# Fix home directory ownership and create Xorg log directory
# /home/nexus is created by debootstrap as root — must be owned by nexus (uid 1000)
log "Fixing nexus home directory ownership and Xorg log dir..."
chown 1000:1000 "${UNSQUASH}/home/nexus"
chmod 755 "${UNSQUASH}/home/nexus"
mkdir -p "${UNSQUASH}/home/nexus/.local/share/xorg"
chown -R 1000:1000 "${UNSQUASH}/home/nexus/.local"
chmod 755 "${UNSQUASH}/home/nexus/.local/share/xorg"

# Ensure no conflicting main lightdm.conf overrides the drop-in
rm -f "${UNSQUASH}/etc/lightdm/lightdm.conf"

# Install overlay-fallback init-premount hook (checks both /lib and /usr/lib)
log "Installing overlay-fallback hook..."
mkdir -p "${UNSQUASH}/usr/share/initramfs-tools/scripts/init-premount"
cat > "${UNSQUASH}/usr/share/initramfs-tools/scripts/init-premount/overlay-fallback" << 'OVHOOK'
#!/bin/sh
PREREQ=""
prereqs() { echo "$PREREQ"; }
case $1 in prereqs) prereqs; exit 0;; esac

if ! grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems 2>/dev/null; then
    modprobe overlay 2>/dev/null || true
fi
if ! grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems 2>/dev/null; then
    KVER="$(uname -r)"
    for ko in "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko" \
              "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.zst" \
              "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.xz" \
              "/usr/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko" \
              "/usr/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.zst" \
              "/usr/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.xz"; do
        [ -f "$ko" ] && insmod "$ko" 2>/dev/null && break
    done
fi
OVHOOK
chmod +x "${UNSQUASH}/usr/share/initramfs-tools/scripts/init-premount/overlay-fallback"

# Ensure overlay module listed in initramfs modules conf
grep -qxF 'overlay' "${UNSQUASH}/etc/initramfs-tools/modules" 2>/dev/null || \
    echo 'overlay' >> "${UNSQUASH}/etc/initramfs-tools/modules"

# Fix casper overlay panic — write Python patcher to file, then run it
log "Patching casper overlay check..."
cat > "${UNSQUASH}/tmp/patch-casper-overlay.py" << 'PYCASPER'
from pathlib import Path

casper = Path('/usr/share/initramfs-tools/scripts/casper')
if not casper.exists():
    raise SystemExit(0)

text = casper.read_text()
needle = '    modprobe "${MP_QUIET}" -b overlay || panic "/cow format specified as \'overlay\' and no support found"'
replacement = """    if ! grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems; then
        modprobe "${MP_QUIET}" -b overlay 2>/dev/null || true
    fi
    if ! grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems; then
        KVER="$(uname -r)"
        for ko in "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko" \
                  "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.zst" \
                  "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.xz" \
                  "/usr/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko" \
                  "/usr/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.zst" \
                  "/usr/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.xz"; do
            [ -f "$ko" ] && insmod "$ko" 2>/dev/null && break
        done
    fi
    grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems || panic \"/cow format specified as 'overlay' and no support found\""""
if needle in text:
    casper.write_text(text.replace(needle, replacement, 1))
    print('[OK] casper overlay panic patched')
else:
    # Check if already patched
    if 'insmod "$ko"' in text and 'overlay 2>/dev/null || true' in text:
        print('[INFO] casper already patched')
    else:
        print('[WARN] casper overlay pattern not found — manual review needed')
PYCASPER
run_in_chroot "python3 /tmp/patch-casper-overlay.py && rm -f /tmp/patch-casper-overlay.py"

# Rebuild initramfs — stash NVIDIA .ko to keep initrd small
log "Rebuilding initramfs..."
run_in_chroot "
    KVER=\$(ls /lib/modules/ 2>/dev/null | sort -V | tail -1)
    [ -z \"\${KVER}\" ] && KVER=\$(ls /usr/lib/modules/ 2>/dev/null | sort -V | tail -1)
    echo \"[patch] Target kernel: \${KVER}\"

    # Stash nvidia .ko so they don't bloat initrd
    mkdir -p /tmp/nvidia-stash
    find \"/lib/modules/\${KVER}\" -name 'nvidia*.ko*' \
        -exec mv -t /tmp/nvidia-stash {} + 2>/dev/null || true
    find \"/usr/lib/modules/\${KVER}\" -name 'nvidia*.ko*' \
        -exec mv -t /tmp/nvidia-stash {} + 2>/dev/null || true

    depmod -a \"\${KVER}\" 2>/dev/null || true
    update-initramfs -u -k \"\${KVER}\" || { echo '[FATAL] initramfs rebuild failed'; exit 1; }

    # Restore nvidia modules
    if ls /tmp/nvidia-stash/nvidia*.ko* 1>/dev/null 2>&1; then
        nvidia_dir=\"/lib/modules/\${KVER}/updates/dkms\"
        mkdir -p \"\${nvidia_dir}\"
        mv /tmp/nvidia-stash/nvidia*.ko* \"\${nvidia_dir}/\"
        depmod -a \"\${KVER}\"
    fi
    rm -rf /tmp/nvidia-stash
    echo \"[OK] initramfs rebuilt for \${KVER}\"
"

# Verify casper + overlay in initramfs
log "Verifying initramfs contents..."
run_in_chroot "
    KVER=\$(ls /boot/vmlinuz-* | sort -V | tail -1 | sed 's|.*/vmlinuz-||')
    INITRD=/boot/initrd.img-\${KVER}

    lsinitramfs \${INITRD} 2>/dev/null | grep -q '^scripts/casper$' || {
        echo '[FATAL] casper scripts missing from initramfs'
        exit 1
    }
    lsinitramfs \${INITRD} 2>/dev/null | grep -q '^scripts/init-premount/overlay-fallback$' || {
        echo '[FATAL] overlay-fallback script missing from initramfs'
        exit 1
    }
    lsinitramfs \${INITRD} 2>/dev/null | grep -Eq 'kernel/fs/overlayfs/overlay\.ko(\.zst|\.xz)?$' || {
        echo '[FATAL] overlay module missing from initramfs'
        exit 1
    }
    echo '[OK] casper + overlay assets verified in initramfs'
"

# Copy updated kernel+initrd to ISO structure
log "Copying kernel and initrd to ISO..."
KVER_BOOT=$(ls "${UNSQUASH}"/boot/vmlinuz-* | sort -V | tail -1 | sed 's|.*/vmlinuz-||')
cp "${UNSQUASH}/boot/vmlinuz-${KVER_BOOT}" "${ISO_NEW}/casper/vmlinuz"
cp "${UNSQUASH}/boot/initrd.img-${KVER_BOOT}" "${ISO_NEW}/casper/initrd"
log "Copied vmlinuz + initrd for kernel ${KVER_BOOT}"

# Clean caches
run_in_chroot "
    apt-get clean 2>/dev/null || true
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
"

umount_chroot

# ── Step 3: Repack squashfs ──────────────────────────────────────────────
log "Repacking squashfs (this takes several minutes)..."
mksquashfs "$UNSQUASH" "${ISO_NEW}/casper/filesystem.squashfs" \
    -comp xz -Xbcj x86 -b 1M -Xdict-size 1M -noappend

du -sx --block-size=1 "$UNSQUASH" | awk '{print $1}' > "${ISO_NEW}/casper/filesystem.size"

# ── Step 4: Rebuild ISO ──────────────────────────────────────────────────
ISO_LABEL="NEXUSOS_1_0"
BUILD_DATE="$(date +%Y.%m.%d)"
ISO_NAME="nexusos-1.0-${BUILD_DATE}-x86_64.iso"
OUTPUT="${BUILD_DIR}/${ISO_NAME}"

log "Building patched ISO..."
mkdir -p "$BUILD_DIR"

if [[ -f "${ISO_NEW}/isolinux/isolinux.bin" ]] && [[ -f "${ISO_NEW}/boot/grub/efi.img" ]]; then
    isohdpfx=""
    for p in /usr/lib/ISOLINUX/isohdpfx.bin /usr/lib/syslinux/mbr/isohdpfx.bin; do
        [[ -f "$p" ]] && { isohdpfx="$p"; break; }
    done
    xorriso -as mkisofs \
        -volid "$ISO_LABEL" \
        -J -R -l -iso-level 3 \
        -b isolinux/isolinux.bin \
        -c isolinux/boot.cat \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        ${isohdpfx:+-isohybrid-mbr "$isohdpfx"} \
        -eltorito-alt-boot \
        -e boot/grub/efi.img \
        -no-emul-boot \
        -isohybrid-gpt-basdat \
        -append_partition 2 0xEF "${ISO_NEW}/boot/grub/efi.img" \
        -o "$OUTPUT" \
        "$ISO_NEW"
elif [[ -f "${ISO_NEW}/boot/grub/efi.img" ]]; then
    xorriso -as mkisofs \
        -volid "$ISO_LABEL" \
        -J -R -l -iso-level 3 \
        -eltorito-boot boot/grub/efi.img \
        -no-emul-boot \
        -eltorito-catalog boot/grub/boot.cat \
        -append_partition 2 0xEF "${ISO_NEW}/boot/grub/efi.img" \
        -o "$OUTPUT" \
        "$ISO_NEW"
else
    die "No bootloader images found in extracted ISO"
fi

sha256sum "$OUTPUT" > "${OUTPUT}.sha256"

log "Done: ${OUTPUT} ($(du -h "$OUTPUT" | awk '{print $1}'))"
