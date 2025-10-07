# NexusOS ZFS Layout Design

## Pool Structure
```
nvme0n1p1  -> EFI System Partition (512MB, FAT32)
nvme0n1p2  -> ZFS pool "rpool" (remaining space)
```

## Dataset Layout
```
rpool                           # Root pool
├── ROOT                        # Container for boot environments  
│   ├── nexusos-20250101        # First boot environment
│   ├── nexusos-20250115        # Update boot environment
│   └── nexusos-current         # Current active BE (symlink)
├── home                        # User home directories
│   └── nexus                   # Default user
├── var                         # Variable data
│   ├── log                     # System logs
│   ├── cache                   # Package cache
│   └── tmp                     # Temporary files
└── opt                         # Optional software (AI/ML, games)
    ├── steam                   # Steam games
    ├── ai-models               # AI/ML models
    └── nexuspkg               # Universal package manager data
```

## Boot Environment Features
- Automatic snapshots before system updates
- Easy rollback via ZFSBootMenu
- Kernel/initrd selection per boot environment
- Incremental backups and replication
- Copy-on-write for space efficiency

## ZFSBootMenu Integration
- UEFI boot directly to ZFS
- No separate /boot partition needed
- Boot environment selection menu
- Automatic kernel detection
- Emergency shell access
