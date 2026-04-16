# NexusOS 1.0.1-dev Release Checklist

Scoped to the active Ubuntu Jammy distro track. Vision-track milestones
live in `docs/vision/ROADMAP-KERNEL.md` and are not part of this
checklist.

## 🎯 Pre-Release Checklist

### ✅ Documentation Complete
- [x] **README.md** — Project overview with features, install, usage
- [x] **CHANGELOG.md** — 1.0.0-dev and 1.0.1-dev entries
- [x] **DISTROWATCH_SUBMISSION.md** — refreshed for 1.0.1-dev
- [x] **ROADMAP.md** — active-track milestones, vision reference
- [x] **WARP.md** — reflects current Ubuntu Jammy debootstrap build
- [x] **CONTRIBUTING.md** — contributor guidelines
- [x] **LICENSE** — GPL v3.0

### ✅ Core Components Ready
- [x] **nexuspkg** — universal package manager, 15+ backends compiled
- [x] **OmnioSearch** — cross-repository search
- [x] **AI Companions** — Stella (8601) & Max Jr. (8602) FastAPI services
- [x] **Orchestrator** — central coordinator on 8600
- [x] **Package Format Support** — 15+ formats end-to-end
- [x] **Repository Detection** — auto-detect best source per package
- [x] **Media stack integration** — `core/media-stack/homelab/` synced with `homelab-media-stack`

### ✅ Distribution Infrastructure
- [x] **ISO Builder** — `scripts/build-iso.sh` builds 4.8 GB hybrid ISO
- [x] **Delta patcher** — `scripts/patch-iso.sh` applies fixes without full rebuild
- [x] **Installation System** — `installer/nexus-install.sh` (overlay + fresh ZFS-on-root)
- [x] **Calamares modules** — live installer modules in `core/installer/`
- [x] **Distribution Metadata** — `distro/os-release`
- [ ] **Package Signing** — GPG infrastructure planned for 1.0.2-dev

### 🔄 Testing & Validation (in progress)
- [x] **VM Testing** — ISO boots in QEMU
- [ ] **Hardware Testing** — reference i9-13900HX + RTX 4080 + second no-NVIDIA box
- [ ] **Fresh-install test** — ZFS-on-root on a spare disk
- [ ] **nexuspkg end-to-end** — 15+ backends verified search→install→upgrade→remove
- [ ] **AI Services** — systemd timers + self-recovery validated

### 🌐 Infrastructure Setup (planned)
- [ ] **Domain registration** — nexusos.org
- [ ] **Repository hosting** — signed APT repo on Bahamut or dedicated CT
- [ ] **Documentation Site** — docs.nexusos.org
- [ ] **Community Platforms** — Discord/forum

---

## 🚀 Release Process

### Step 1: Final Preparation
```bash
git status                            # clean tree
git add .
git commit -m "Release v1.0.1-dev"
git tag -a v1.0.1-dev -m "NexusOS 1.0.1-dev"
git push origin main --tags
```

### Step 2: Build Release ISO
```bash
sudo ./scripts/build-iso.sh           # default output dir: build/
ls -la build/nexusos-1.0-*.iso*
```

### Step 3: Create GitHub Release
1. Go to https://github.com/wlfogle/nexus-os/releases/new
2. Tag: `v1.0.1-dev`
3. Title: `NexusOS 1.0.1-dev`
4. Description: extract from `CHANGELOG.md`
5. Upload ISO + SHA256 + signature
6. Mark as "Pre-release"
7. Publish

### Step 4: DistroWatch Submission
1. Visit https://distrowatch.com/dwres.php?resource=submit
2. Use information from `DISTROWATCH_SUBMISSION.md`
3. Provide screenshots (to be taken from live ISO)
4. Submit download links from GitHub release

### Step 5: Community Announcement
- [ ] Post on Reddit (r/linux, r/linuxdistros, r/pop_os)
- [ ] Announce on Twitter/X (@NexusOS_Linux)
- [ ] Create blog post announcement
- [ ] Notify Linux news sites (Phoronix, OMG Ubuntu, etc.)

---

## 📋 DistroWatch Submission Requirements

### Required Information
- [x] **Distribution Name**: NexusOS
- [x] **Version**: 1.0.1-dev
- [x] **Release Date**: 2026-04-07 (1.0.1-dev); 2026-03-09 (1.0.0-dev first boot)
- [x] **Architecture**: x86_64
- [x] **Base Distribution**: Ubuntu Jammy 22.04 LTS (debootstrap)
- [x] **Desktop Environment**: KDE Plasma X11 + SDDM
- [x] **Package Manager**: nexuspkg (universal) + nala (native)
- [x] **Category**: Desktop, Gaming, Media Center

### Required Materials
- [x] **Detailed Description** - In DISTROWATCH_SUBMISSION.md
- [x] **Feature List** - Universal package management, AI companions
- [x] **Technical Specifications** - Complete system requirements
- [x] **Installation Instructions** - Step-by-step guide
- [x] **Download Links** - Will be provided from GitHub release
- [ ] **Screenshots** - To be taken from live ISO
- [ ] **Torrent Links** - Optional, can be added later

### Submission Categories
- **Primary**: Desktop
- **Secondary**: Gaming, Live Medium, Media Center
- **Keywords**: universal, packages, AI, gaming, ubuntu-based, LTS-based

---

## 📊 Release Statistics

### Code Metrics
- **Lines of Code**: 25,000+ (core system)
- **Supported Formats**: 15+ package formats
- **Repository Coverage**: 25+ major repositories  
- **Available Packages**: 80,000+ (via universal access)
- **Media Services**: 65+ ready-to-deploy services

### Platform Support
- **Architecture**: x86_64 (ARM64 planned)
- **Boot Systems**: BIOS, UEFI
- **Display Servers**: X11, Wayland
- **Init System**: systemd
- **File Systems**: ext4, btrfs, ZFS, NTFS, exFAT

### Performance Targets
- **RAM Usage**: <2GB idle (without media stack)
- **Storage Usage**: ~40GB base installation
- **Boot Time**: <30 seconds on SSD
- **Package Install**: <5 minutes for major applications

---

## 🎬 Post-Release Roadmap

### Immediate (1-2 weeks)
- [ ] Monitor community feedback
- [ ] Fix critical bugs and issues
- [ ] Create installation tutorials and videos
- [ ] Set up community support channels

### Short-term (1-3 months) 
- [ ] GUI Package Manager interface
- [ ] Enhanced AI recommendation engine
- [ ] ARM64 architecture support
- [ ] Package format conversion tools

### Medium-term (3-6 months)
- [ ] Enterprise security features
- [ ] Custom repository hosting service
- [ ] Mobile device integration
- [ ] Cloud deployment tools

### Long-term (6-12 months)
- [ ] Hardware certification program
- [ ] OEM partnerships
- [ ] Educational institution adoption
- [ ] Commercial support options

---

## 📞 Release Support Plan

### Community Support
- **GitHub Issues**: Primary support channel
- **Discord Server**: Real-time community chat (planned)
- **Forum**: nexusos.org/forum (planned)
- **Documentation**: Comprehensive wiki and guides

### Development Support
- **Bug Reports**: GitHub issue templates
- **Feature Requests**: Community voting system
- **Contributions**: Contributor guidelines and recognition
- **Code Review**: Maintainer review process

### User Support
- **Installation Guide**: Step-by-step documentation
- **Troubleshooting**: Common issues and solutions
- **FAQ**: Frequently asked questions
- **Video Tutorials**: YouTube channel (planned)

---

## ✅ Final Pre-Release Validation

### Core Functionality Tests
- [ ] Boot ISO in VM successfully
- [ ] Complete installation process
- [ ] nexuspkg install from different sources
- [ ] AI companions respond to commands
- [ ] Desktop environment loads properly
- [ ] Network connectivity works
- [ ] Graphics drivers load correctly
- [ ] Audio system functions

### Package Management Tests  
- [ ] Install from nala/apt repositories
- [ ] Install from Flatpak
- [ ] Install Flatpak applications
- [ ] Install Snap packages
- [ ] Install pip packages
- [ ] Install npm packages
- [ ] Search across all repositories
- [ ] Package dependency resolution

### System Integration Tests
- [ ] System updates work properly
- [ ] Service management functional
- [ ] User account creation
- [ ] Sudo privileges working
- [ ] File system permissions correct
- [ ] Network configuration
- [ ] Hardware detection

---

**Release Status**: 🔄 1.0.1-dev (active Ubuntu-track)  
**Next Milestone**: 1.0.2-dev — hardware validation + installer polish (see `ROADMAP.md`)  
**Target Audience**: Linux enthusiasts, developers, early adopters  

---

*"One distribution to rule them all, one package manager to find them, one system to bring them all, and in the compatibility bind them."*  
**– The NexusOS Project**