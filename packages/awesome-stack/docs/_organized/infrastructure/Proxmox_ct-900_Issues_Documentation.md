### Summary of Proxmox Container 900 Issues and Steps Taken

**Container ID:** 900  
**Hostname:** ai-container

#### **Startup Issues:**
1. **Empty Disk**: Current `vm-900-disk-0` is empty (0.00% usage) - recreated but not restored
2. **I/O Errors**: Buffer I/O errors on the device `dm-6` indicating disk issues
3. **Mount Point Issue**: Reference to `/mnt/nvme-storage` causing confusion, as it exists on Proxmox host
4. **Failed Rollback State**: Stuck in rollback state with filesystem corruption

#### **Actions Taken:**
1. Attempted rollback from multiple snapshots, including `auto-20250730-153317`
2. Created new logical volume for `vm-900-disk-0` manually
3. Cleaned up storage to free up space:
   - Deleted VM 611
   - Deleted snapshots from VMs 612 and 613
   - Renamed VM 613 to VM 611 as `media-bridge`
4. Investigated logs, dmesg, and configuration to identify issues

#### **Options for Resolution:**
1. Restore from working snapshot (preserves changes)
2. Fix mount point and restore
3. Create a fresh container and reinstall LLMs

**Next Steps:** 
Awaiting decision on whether to restore from snapshots, fix mount issue, or start fresh.
