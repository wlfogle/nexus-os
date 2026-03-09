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
