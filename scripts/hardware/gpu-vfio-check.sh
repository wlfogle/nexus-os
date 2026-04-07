#!/usr/bin/env bash
# VFIO host sanity check: IOMMU support + GPU-containing groups

set -u  # don't use -e so greps that find nothing don't abort

# --- helpers ---------------------------------------------------------------
have() { command -v "$1" >/dev/null 2>&1; }

read_klog() {
  if have journalctl; then journalctl -k -b 0 2>/dev/null
  else dmesg 2>/dev/null
  fi
}

trim() { sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//'; }

# --- 1) CPU vendor + boot flags -------------------------------------------
CPU_VENDOR="$(
  (lscpu 2>/dev/null | awk -F: '/Vendor ID/{print $2}' | trim) ||
  (grep -m1 'vendor_id' /proc/cpuinfo 2>/dev/null | awk '{print $3}')
)"
[ -z "${CPU_VENDOR}" ] && CPU_VENDOR="(unknown)"

CMDLINE="$(cat /proc/cmdline 2>/dev/null || echo '')"
HAS_INTEL_FLAG=$(echo "$CMDLINE" | grep -q 'intel_iommu=on' && echo yes || echo no)
HAS_AMD_FLAG=$(echo "$CMDLINE" | grep -q 'amd_iommu=on' && echo yes || echo no)
HAS_PT_FLAG=$(echo "$CMDLINE" | grep -q 'iommu=pt' && echo yes || echo no)

# --- 2) Kernel log signals ------------------------------------------------
KLOG="$(read_klog)"

DISABLED_MSG=$(echo "$KLOG" | egrep -i 'IOMMU.*disabled by BIOS|DMAR:.*disabled|AMD-Vi:.*disabled' || true)
ENABLED_MSG=$(echo "$KLOG" | egrep -i 'DMAR: IOMMU enabled|AMD-Vi:.*IOMMU.*enabled|IOMMU: .*enabled' || true)
IR_MSG=$(echo "$KLOG" | egrep -i 'Interrupt remapping enabled' || true)

# --- 3) IOMMU groups presence --------------------------------------------
GROUPS_DIR="/sys/kernel/iommu_groups"
GROUP_COUNT=0
if [ -d "$GROUPS_DIR" ]; then
  GROUP_COUNT=$(find "$GROUPS_DIR" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | awk '{print $1}')
fi

# Heuristic: active if groups exist (>0). Logs help explain state.
IOMMU_ACTIVE="no"
[ "$GROUP_COUNT" -gt 0 ] && IOMMU_ACTIVE="yes"

# --- 4) Report summary ----------------------------------------------------
echo "=== IOMMU Summary ==="
echo "CPU vendor           : $CPU_VENDOR"
echo "Kernel cmdline       : $CMDLINE"
echo "Boot flags           : intel_iommu=$HAS_INTEL_FLAG  amd_iommu=$HAS_AMD_FLAG  iommu=pt=$HAS_PT_FLAG"
echo "Groups directory     : $GROUPS_DIR  (exists: $([ -d "$GROUPS_DIR" ] && echo yes || echo no))"
echo "IOMMU group count    : $GROUP_COUNT"
echo "Kernel says enabled  : $([ -n "$ENABLED_MSG" ] && echo yes || echo no)"
echo "Interrupt remapping  : $([ -n "$IR_MSG" ] && echo yes || echo no)"
echo "Kernel says disabled : $([ -n "$DISABLED_MSG" ] && echo yes || echo no)"
echo "IOMMU ACTIVE?        : $IOMMU_ACTIVE"
echo

if [ -n "$ENABLED_MSG" ]; then
  echo "--- Kernel enable lines ---"
  echo "$ENABLED_MSG"
  echo
fi
if [ -n "$DISABLED_MSG" ]; then
  echo "--- Kernel disable lines ---"
  echo "$DISABLED_MSG"
  echo
fi

# --- 5) Original: list only GPU-containing groups -------------------------
echo "=== GPU-Containing IOMMU Groups ==="
if [ ! -d "$GROUPS_DIR" ] || [ "$GROUP_COUNT" -eq 0 ]; then
  echo "(no IOMMU groups found)"
else
  declare -A GPU_COUNT_BY_GROUP=()
  group_warnings=()

  for g in "$GROUPS_DIR"/*; do
    [ -d "$g" ] || continue
    group_num=$(basename "$g")
    gpu_found=false
    device_lines=""
    non_gpu_non_bridge=false
    gpu_count_in_this_group=0

    for d in "$g"/devices/*; do
      [ -e "$d" ] || continue
      pci_addr=$(basename "$d")
      # -nns prints class code [XXXX] and vendor:device [vvvv:dddd]
      line=$(lspci -nns "$pci_addr" 2>/dev/null || echo "$pci_addr (unlisted)")
      device_lines+="$line"$'\n'

      # Extract first [...] which is the class code, e.g. 0300, 0302, 0403, 0604, 0600
      class_code=$(echo "$line" | awk -F'[][]' '{print $2}')

      # Detect GPUs / 3D controllers and their HDA audio functions
      if echo "$line" | grep -qE 'VGA compatible controller|3D controller'; then
        gpu_found=true
        gpu_count_in_this_group=$((gpu_count_in_this_group+1))
      fi

      # Allowlist: 0300(VGA), 0302(3D), 0403(HDA audio), 0600(host bridge), 0604(PCI bridge)
      case "$class_code" in
        0300|0302|0403|0600|0604) : ;;
        *) non_gpu_non_bridge=true ;;
      esac
    done

    if $gpu_found; then
      echo "IOMMU Group $group_num:"
      echo "$device_lines"

      # Track GPUs per group
      GPU_COUNT_BY_GROUP["$group_num"]=$gpu_count_in_this_group

      # Warn if unexpected devices share the group with the GPU
      if $non_gpu_non_bridge; then
        group_warnings+=("WARN: Group $group_num contains non-GPU, non-audio, non-bridge devices (consider different slot/CPU root complex or ACS).")
      fi
    fi
  done

  # Post-checks
  # 1) Each GPU should be alone (one GPU per group)
  shared_groups=()
  for gnum in "${!GPU_COUNT_BY_GROUP[@]}"; do
    if [ "${GPU_COUNT_BY_GROUP[$gnum]}" -gt 1 ]; then
      shared_groups+=("$gnum")
    fi
  done

  if [ "${#shared_groups[@]}" -gt 0 ]; then
    echo
    echo "WARN: Multiple GPUs share these IOMMU groups: ${shared_groups[*]} (prefer one GPU per group for VFIO)."
  fi

  # 2) Any non-bridge co-residents?
  if [ "${#group_warnings[@]}" -gt 0 ]; then
    echo
    printf "%s\n" "${group_warnings[@]}"
  fi
fi
