# Proxmox VE VM Profile

This directory contains the VM profile configuration for running Proxmox VE in a KVM virtual machine with direct disk passthrough.

## Files

- `proxmox-ve.json` - JSON configuration profile for the KVM Manager application
- `proxmox-ve.xml` - Libvirt XML domain definition
- `README.md` - This documentation file

## Configuration Details

### VM Specifications
- **Name**: Proxmox VE Server
- **Memory**: 8GB RAM (8192 MB)
- **CPUs**: 4 virtual CPUs
- **OS Type**: Linux (Debian 12 base)
- **Machine Type**: Q35 with UEFI firmware

### Storage Configuration
1. **Primary Disk (vda)**:
   - Source: `/run/media/lou/Data/proxmox-ve.qcow2`
   - Format: qcow2
   - Bus: VirtIO
   - Cache: writeback

2. **Passthrough Disk (vdb)**:
   - Source: `/dev/nvme1n1p2`
   - Format: raw
   - Bus: VirtIO
   - Cache: none (for best performance)

### Network Configuration
- **Interface**: VirtIO network adapter
- **MAC Address**: 52:54:00:12:34:56
- **Network**: Default libvirt network (NAT)
- **Bridge**: virbr0

### Display Configuration
- **Graphics**: SPICE
- **Video**: QXL with 64MB VRAM
- **Listen Address**: 127.0.0.1 (local only)
- **Auto Port**: Enabled

## Prerequisites

1. **Libvirt and KVM**: Ensure libvirt and KVM are installed and running
2. **UEFI Firmware**: OVMF package must be installed for UEFI boot support
3. **Permissions**: User must have access to `/dev/nvme1n1p2`
4. **Storage**: Verify the Proxmox VE qcow2 file exists at the specified path

## Usage

### Using the KVM Manager Application
1. Load the profile from `profiles/proxmox-ve.json`
2. Review and adjust settings as needed
3. Create and start the VM

### Using virsh directly
```bash
# Define the VM from XML
virsh define profiles/proxmox-ve.xml

# Start the VM
virsh start proxmox-ve-server

# Connect to console
virsh console proxmox-ve-server

# Connect via SPICE (requires spice client)
spicy localhost:5900  # Port will vary based on autoport assignment
```

## Security Considerations

1. **Disk Passthrough**: The NVMe partition passthrough provides direct hardware access
2. **Host Passthrough CPU**: Uses host CPU features directly for better performance
3. **Local Graphics**: SPICE is bound to localhost only for security

## Performance Optimizations

- **VirtIO**: All devices use VirtIO for optimal performance
- **Host CPU Passthrough**: Maximum CPU performance
- **Direct Disk Access**: Raw disk passthrough eliminates virtualization overhead
- **Native I/O**: Uses native I/O for the passthrough disk

## Troubleshooting

### Common Issues
1. **Permission Denied on /dev/nvme1n1p2**:
   ```bash
   sudo chown libvirt-qemu:libvirt-qemu /dev/nvme1n1p2
   # Or add user to appropriate groups
   sudo usermod -a -G disk,libvirt $USER
   ```

2. **UEFI Boot Issues**:
   ```bash
   # Ensure OVMF is installed
   sudo pacman -S edk2-ovmf  # Arch/Garuda
   # Verify OVMF paths exist
   ls -la /usr/share/edk2/ovmf/
   ```

3. **Network Connectivity**:
   ```bash
   # Ensure default network is active
   virsh net-start default
   virsh net-autostart default
   ```

### Monitoring
- Use the KVM Manager application's system monitoring features
- Monitor disk I/O on both virtual disks
- Check Proxmox VE logs within the VM once running

## Customization

To modify the configuration:
1. Edit `proxmox-ve.json` for application-level settings
2. Edit `proxmox-ve.xml` for low-level libvirt configuration
3. Restart/recreate the VM to apply changes

## Notes

- This configuration assumes Proxmox VE is already installed on the qcow2 disk
- The passthrough NVMe partition can be used for Proxmox storage (VMs, containers, backups)
- Consider adjusting memory allocation based on your intended Proxmox workload
- The MAC address may need to be changed if you have existing VMs with the same address
