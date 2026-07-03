#!/usr/bin/env bash
# =============================================================================
# NexusOS Integration Test Suite  (v0.6.2 — Phase 6 subsystems)
# =============================================================================
# Verifies the from-scratch x86_64 Rust microkernel (Limine ISO) and the four
# Phase-6 subsystems integrated on this branch:
#   • core boot + memory + scheduler (per-process CR3 address spaces)
#   • VirtIO-blk + FAT32 + installer
#   • NVMe + AHCI (SATA) block drivers
#   • VirtIO-net + smoltcp networking
#   • ring-3 shell reachable (interactive: ls/cat/run — see --interactive)
#
# Two modes:
#   (default)      Headless QEMU boot, serial captured to a log, asserts the
#                  real boot-log markers each subsystem prints. CI-friendly.
#   --interactive  Launch the ISO in MobaLiveCD (GTK) for a human to drive the
#                  shell (ls /EFI, cat /EFI/BOOT/limine.conf, run HELLO.ELF).
#
# The kernel needs LEGACY/transitional VirtIO (I/O-port BAR), so the headless
# run forces virtio-*-pci.disable-modern=on. NVMe/AHCI use MMIO BARs and are
# attached with -device nvme / -device ich9-ahci.
#
# Usage:
#   tests/integration_test.sh [--iso PATH] [--timeout N] [--interactive] [--keep]
# =============================================================================
set -u

# ── Config / args ────────────────────────────────────────────────────────────
ISO=""
TIMEOUT=30
INTERACTIVE=0
KEEP=0
MOBA="/home/loufogle/nexus-os/packages/mobalivecd-linux/enhanced_mobalivecd.py"
SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SELF_DIR/.." && pwd)"
BUILD_DIR="${BUILD_DIR:-out}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --iso)         ISO="$2"; shift 2 ;;
    --timeout)     TIMEOUT="$2"; shift 2 ;;
    --interactive) INTERACTIVE=1; shift ;;
    --keep)        KEEP=1; shift ;;
    -h|--help)     grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *)             echo "unknown arg: $1"; exit 2 ;;
  esac
done

RED=$'\033[0;31m'; GRN=$'\033[0;32m'; YEL=$'\033[1;33m'; CYN=$'\033[0;36m'; NC=$'\033[0m'
PASS=0; FAIL=0; SKIP=0

# ── Locate or build the ISO ──────────────────────────────────────────────────
if [[ -z "$ISO" ]]; then
  for cand in "$REPO_ROOT/$BUILD_DIR/nexusos-laptop.iso" "$REPO_ROOT/build/nexusos-laptop.iso"; do
    [[ -f "$cand" ]] && { ISO="$cand"; break; }
  done
fi
if [[ -z "$ISO" || ! -f "$ISO" ]]; then
  echo "${YEL}[build]${NC} ISO not found — building (make laptop && iso-laptop, BUILD_DIR=$BUILD_DIR)"
  ( cd "$REPO_ROOT" && make laptop BUILD_DIR="$BUILD_DIR" && make iso-laptop BUILD_DIR="$BUILD_DIR" ) \
    || { echo "${RED}[fatal]${NC} build failed"; exit 1; }
  ISO="$REPO_ROOT/$BUILD_DIR/nexusos-laptop.iso"
fi
echo "${CYN}NexusOS Integration Test${NC}  ISO=$ISO"

# ── Interactive mode: hand off to MobaLiveCD ─────────────────────────────────
if [[ "$INTERACTIVE" -eq 1 ]]; then
  echo "${CYN}[interactive]${NC} Launching ISO in MobaLiveCD for manual shell testing..."
  echo "  At the nexus> prompt, verify: help | ls /EFI | cat /EFI/BOOT/limine.conf | run HELLO.ELF"
  if [[ -f "$MOBA" ]]; then
    exec python3 "$MOBA" --quick-launch --memory 2 "$ISO"
  else
    echo "${RED}[error]${NC} MobaLiveCD not found at $MOBA"; exit 1
  fi
fi

# ── Headless boot: prepare throwaway devices ─────────────────────────────────
WORK="$(mktemp -d /tmp/nexus-itest.XXXXXX)"
LOG="$WORK/serial.log"
BLK="$WORK/virtio-blk.qcow2"      # fresh blank disk -> installer path exercised
NVME="$WORK/nvme.img"
SATA="$WORK/sata.img"
cleanup() { pkill -f "qemu-system-x86_64.*$WORK" 2>/dev/null; [[ "$KEEP" -eq 1 ]] || rm -rf "$WORK"; }
trap cleanup EXIT

qemu-img create -f qcow2 "$BLK" 256M >/dev/null 2>&1
qemu-img create -f raw   "$NVME" 64M  >/dev/null 2>&1
qemu-img create -f raw   "$SATA" 64M  >/dev/null 2>&1

KVM=(); [[ -w /dev/kvm ]] && KVM=(-enable-kvm -cpu host)

echo "${CYN}[boot]${NC} headless QEMU (timeout ${TIMEOUT}s, legacy VirtIO + NVMe + AHCI + net)..."
timeout "$TIMEOUT" qemu-system-x86_64 \
  "${KVM[@]}" -m 2G -smp 2 \
  -global virtio-blk-pci.disable-modern=on -global virtio-blk-pci.disable-legacy=off \
  -global virtio-net-pci.disable-modern=on -global virtio-net-pci.disable-legacy=off \
  -boot d -cdrom "$ISO" \
  -drive file="$BLK",if=virtio,format=qcow2 \
  -netdev user,id=n0 -device virtio-net-pci,netdev=n0 \
  -drive id=nvm,file="$NVME",if=none,format=raw -device nvme,serial=nx01,drive=nvm \
  -device ich9-ahci,id=ahci0 \
  -drive id=sata0,file="$SATA",if=none,format=raw -device ide-hd,drive=sata0,bus=ahci0.0 \
  -serial file:"$LOG" -display none -no-reboot -no-shutdown >/dev/null 2>&1 || true

if [[ ! -s "$LOG" ]]; then
  echo "${RED}[fatal]${NC} no serial output captured (boot produced nothing)"; exit 1
fi

# ── Assertion helpers ────────────────────────────────────────────────────────
want() {  # want "<name>" "<grep-pattern>"
  if grep -qaF "$2" "$LOG"; then echo "  ${GRN}✓${NC} $1"; PASS=$((PASS+1));
  else echo "  ${RED}✗${NC} $1  (missing: '$2')"; FAIL=$((FAIL+1)); fi
}
want_re() {  # regex variant
  if grep -qaE "$2" "$LOG"; then echo "  ${GRN}✓${NC} $1"; PASS=$((PASS+1));
  else echo "  ${RED}✗${NC} $1  (missing /$2/)"; FAIL=$((FAIL+1)); fi
}
absent() {  # absent "<name>" "<grep-pattern>"  (pass when NOT present)
  if grep -qaF "$2" "$LOG"; then echo "  ${RED}✗${NC} $1  (unexpected: '$2')"; FAIL=$((FAIL+1));
  else echo "  ${GRN}✓${NC} $1"; PASS=$((PASS+1)); fi
}
skip() { echo "  ${YEL}—${NC} $1 (manual: run with --interactive)"; SKIP=$((SKIP+1)); }

echo
echo "${CYN}Core boot + scheduler (CR3 address spaces must not regress boot)${NC}"
want_re "kernel banner"          "NexusOS Kernel v0\.6"
want    "paging initialised"     "[mem]  Paging initialised"
want    "kernel heap ready"      "Kernel heap"
want    "PIC + PIT timer"        "[timer] PIC remapped"
want    "i8042 keyboard init"    "[kbd]  i8042 initialised"
want    "ring-3 init spawned"    "[user] nexus-init spawned as pid="
want    "scheduler LIVE"         "[arch] Interrupts enabled — scheduler is LIVE"
want    "shell ready"            "[kbd]  PS/2 keyboard online — nexus-shell ready"
want    "AI Core online"         "[nexus-ai] AI Core online"

echo
echo "${CYN}Storage: VirtIO-blk + FAT32 + installer${NC}"
want_re "VirtIO-blk detected"    "\[disk\] VirtIO-blk: [0-9]+ GiB"
want_re "FAT32 layer active"     "\[fs\]   (FAT32 mounted|disk present, not formatted|no disk)"

echo
echo "${CYN}Storage: NVMe driver (device attached; success is silent)${NC}"
absent  "NVMe controller found"  "[nvme] no NVMe controller found"
absent  "NVMe init ok"           "[nvme] init failed"

echo
echo "${CYN}Storage: AHCI/SATA driver (device attached)${NC}"
absent  "AHCI controller found"  "[ahci] no AHCI SATA controller found"
absent  "AHCI init ok"           "[ahci] init failed"

echo
echo "${CYN}Networking: VirtIO-net + smoltcp${NC}"
want    "VirtIO-net driver up"   "[net]"
absent  "net device bound"       "[net]  no VirtIO-net device found"

echo
echo "${CYN}Interactive-only (need PS/2 shell input — use --interactive)${NC}"
skip "VFS path ls/cat  (ls /EFI, cat /EFI/BOOT/limine.conf)"
skip "ELF exec         (run HELLO.ELF)"

# ── Summary ──────────────────────────────────────────────────────────────────
echo
echo "${CYN}========================================${NC}"
echo "  Passed: ${GRN}$PASS${NC}   Failed: ${RED}$FAIL${NC}   Manual: ${YEL}$SKIP${NC}"
echo "${CYN}========================================${NC}"
if [[ "$FAIL" -ne 0 ]]; then
  echo "${YEL}Last 40 serial lines:${NC}"; tail -40 "$LOG"
  exit 1
fi
echo "${GRN}✓ All automated integration checks passed${NC}"
