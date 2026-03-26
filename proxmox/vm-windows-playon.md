# VM-901: Windows 11 Gaming + PlayOn (GPU + Disk Passthrough)
## Purpose
VM-901 is the Windows gaming + PlayOn VM on Tiamat.
It uses RX 580 passthrough for gaming and passes through the 240GB SSD (`/dev/sdb`) for game storage.
PlayOn saves recordings to `/mnt/hdd/media/playon`, then Plex/Jellyfin ingest them.
## Current State
- Host IOMMU is enabled: `amd_iommu=on iommu=pt`
- GPU passthrough is already configured:
  - `09:00.0` RX 580 VGA → `vfio-pci`
  - `09:00.1` RX 580 HDMI audio → `vfio-pci`
- VM-901 storage volumes already exist on `local-lvm`:
  - `vm-901-disk-1` (300GB)
  - `vm-901-disk-efivars`
  - `vm-901-tpmstate`
- Windows 11 26H1 ISO is already on host:
  - `/var/lib/vz/template/iso/28000.1_MULTI_X64_EN-US.ISO`
- VirtIO ISO is already on host:
  - `/var/lib/vz/template/iso/virtio-win.iso`
## Recreate VM-901 Config
If `/etc/pve/qemu-server/901.conf` is missing, recreate it:
```bash
cat > /etc/pve/qemu-server/901.conf <<'EOF'
agent: 1
balloon: 0
bios: ovmf
boot: order=scsi0;ide2
cores: 4
cpu: host
efidisk0: local-lvm:vm-901-disk-efivars,efitype=4m,pre-enrolled-keys=1,size=4M
hostpci0: 09:00,pcie=1,x-vga=1
machine: q35
memory: 4096
meta: creation-qemu=9.2.0,ctime=0
name: windows-gaming
net0: virtio=DE:AD:BE:EF:90:01,bridge=vmbr0,firewall=1
numa: 0
onboot: 1
ostype: win11
scsi0: local-lvm:vm-901-disk-1,cache=writeback,discard=on,iothread=1,size=300G,ssd=1
scsi1: /dev/sdb,cache=none,discard=on,iothread=1,ssd=1
scsihw: virtio-scsi-single
smbios1: uuid=auto
sockets: 1
tpmstate0: local-lvm:vm-901-tpmstate,size=4M,version=v2.0
vga: none
ide2: local:iso/28000.1_MULTI_X64_EN-US.ISO,media=cdrom
ide3: local:iso/virtio-win.iso,media=cdrom
EOF
```
## Start + Verify
```bash
qm start 901
qm status 901
qm config 901
```
## Windows Install Notes
1. Boot VM-901 from Win11 ISO.
2. If no disk appears, load VirtIO driver from `ide3` (`viostor` or `vioscsi`).
3. Install Windows to `scsi0` (300GB local-lvm disk).
4. Keep `/dev/sdb` passthrough disk as game/data volume inside Windows.
## PlayOn Setup
1. Install PlayOn Home.
2. Sign in with your lifetime account.
3. Set recording path to shared media folder that lands on `/mnt/hdd/media/playon`.
4. In Plex/Jellyfin, add a library path for PlayOn output.
## RAM Constraint Workaround (Current 8GB Host RAM)
Until RAM upgrade:
- Stop non-essential CTs before gaming VM sessions.
- Keep VM at 4GB RAM for now.
- Prefer running PlayOn jobs in off-hours.
Recommended upgrade: 2×32GB DDR4-3200 CL16.
## Management
```bash
# Start/stop VM
qm start 901
qm stop 901
qm shutdown 901

# Check status
qm status 901

# Open console
# Proxmox UI -> VM 901 -> Console
```
## Recovery Notes
- If VM boots to black screen after driver changes, remove hostpci temporarily in VM config and boot with standard VGA.
- If AMD reset issues appear after repeated VM restarts, do a full host reboot.
