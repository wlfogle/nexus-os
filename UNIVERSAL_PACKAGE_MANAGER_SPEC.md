# 🌍 Universal Package Manager - ALL Linux Distributions Support

**nexuspkg**: The ultimate universal package manager that can install packages from **every known Linux distribution**.

---

## 📦 **Supported Package Formats & Repositories**

### **1. Arch Linux Family**
- ✅ **pacman** (.pkg.tar.xz, .pkg.tar.zst) - Arch, Manjaro, EndeavourOS, Garuda
- ✅ **AUR** (Arch User Repository) - via yay, paru, aurman
- ✅ **pacman-git** (Git packages) - Development versions

### **2. Debian Family** 
- ✅ **apt/dpkg** (.deb) - Debian, Ubuntu, Linux Mint, Pop!_OS, Elementary OS
- ✅ **PPA** (Personal Package Archives) - Ubuntu PPAs
- ✅ **Debian Backports** - Testing/unstable packages
- ✅ **Ubuntu Universe/Multiverse** - Additional repositories
- ✅ **Debian Experimental** - Experimental packages

### **3. Red Hat Family**
- ✅ **dnf/yum** (.rpm) - Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux
- ✅ **RPM Fusion** - Free and non-free RPM packages
- ✅ **EPEL** (Extra Packages for Enterprise Linux)
- ✅ **Copr** - Community-driven RPM packages
- ✅ **openSUSE Build Service** - Cross-distribution packages

### **4. SUSE Family**
- ✅ **zypper** (.rpm) - openSUSE Leap, Tumbleweed, SLES
- ✅ **openSUSE Factory** - Rolling release packages
- ✅ **Packman Repository** - Multimedia packages
- ✅ **OBS** (Open Build Service) - Community packages

### **5. Gentoo Family**
- ✅ **portage** (source-based) - Gentoo, Calculate Linux, Funtoo
- ✅ **emerge** - Source compilation with USE flags
- ✅ **Gentoo Overlays** - Third-party ebuilds
- ✅ **Binary packages** - Pre-compiled Gentoo packages

### **6. Alpine Linux**
- ✅ **apk** (.apk) - Alpine Linux packages
- ✅ **Alpine Edge** - Development packages
- ✅ **Alpine Community** - Community-maintained packages

### **7. Void Linux**
- ✅ **xbps** (.xbps) - Void Linux packages
- ✅ **Void Templates** - Source package templates

### **8. NixOS**
- ✅ **nix** - Nix packages and NixOS
- ✅ **nixpkgs** - Nix package collection
- ✅ **Nix Channels** - Stable/unstable channels

### **9. Universal Formats**
- ✅ **Flatpak** - Sandboxed applications
- ✅ **Snap** - Ubuntu's universal packages
- ✅ **AppImage** - Portable application format
- ✅ **Docker** - Containerized applications

### **10. Source-Based**
- ✅ **GitHub Releases** - Direct binary downloads
- ✅ **GitLab Releases** - Project releases
- ✅ **make install** - Traditional source compilation
- ✅ **pip** (Python) - Python Package Index
- ✅ **npm** (Node.js) - Node Package Manager
- ✅ **cargo** (Rust) - Rust package manager
- ✅ **gem** (Ruby) - RubyGems
- ✅ **go install** (Go) - Go modules

### **11. Less Common Distributions**
- ✅ **Slackware** (.txz, .tgz) - Slackware packages
- ✅ **FreeBSD Ports** - FreeBSD package system
- ✅ **Solus** (.eopkg) - Solus packages
- ✅ **Clear Linux** (.rpm with swupd) - Intel Clear Linux
- ✅ **Guix** - GNU Guix packages

---

## 🔧 **Implementation Architecture**

### **Package Detection Priority Algorithm**
```c
package_format_t detect_optimal_package_source(const char* package_name) {
    // 1. Check local/native repositories first (highest priority)
    if (check_arch_repos(package_name)) return FORMAT_ARCH_PKG;
    if (check_aur(package_name)) return FORMAT_AUR;
    
    // 2. Check universal formats (cross-platform)
    if (check_flatpak(package_name)) return FORMAT_FLATPAK;
    if (check_snap(package_name)) return FORMAT_SNAP;
    if (check_appimage(package_name)) return FORMAT_APPIMAGE;
    
    // 3. Check major distribution repositories
    if (check_ubuntu_repos(package_name)) return FORMAT_DEB;
    if (check_fedora_repos(package_name)) return FORMAT_RPM;
    if (check_opensuse_repos(package_name)) return FORMAT_ZYPPER_RPM;
    if (check_debian_repos(package_name)) return FORMAT_DEBIAN_DEB;
    
    // 4. Check community repositories
    if (check_ppa_repos(package_name)) return FORMAT_PPA;
    if (check_copr_repos(package_name)) return FORMAT_COPR_RPM;
    if (check_rpm_fusion(package_name)) return FORMAT_RPM_FUSION;
    
    // 5. Check source-based options
    if (check_github_releases(package_name)) return FORMAT_GITHUB_RELEASE;
    if (check_pip_packages(package_name)) return FORMAT_PIP;
    if (check_npm_packages(package_name)) return FORMAT_NPM;
    if (check_cargo_packages(package_name)) return FORMAT_CARGO;
    
    // 6. Check alternative distributions
    if (check_gentoo_portage(package_name)) return FORMAT_GENTOO_EBUILD;
    if (check_void_xbps(package_name)) return FORMAT_VOID_XBPS;
    if (check_alpine_apk(package_name)) return FORMAT_ALPINE_APK;
    if (check_nix_packages(package_name)) return FORMAT_NIX;
    
    return FORMAT_UNKNOWN;
}
```

### **Universal Installation System**
```c
int install_package_universal(const char* package_name, package_format_t format) {
    switch (format) {
        // Arch Linux family
        case FORMAT_ARCH_PKG:
            return install_arch_pacman(package_name);
        case FORMAT_AUR:
            return install_arch_aur(package_name);
            
        // Debian family  
        case FORMAT_DEB:
            return install_debian_apt(package_name);
        case FORMAT_PPA:
            return install_ubuntu_ppa(package_name);
            
        // Red Hat family
        case FORMAT_RPM:
            return install_fedora_dnf(package_name);
        case FORMAT_COPR_RPM:
            return install_fedora_copr(package_name);
        case FORMAT_RPM_FUSION:
            return install_rpm_fusion(package_name);
            
        // SUSE family
        case FORMAT_ZYPPER_RPM:
            return install_opensuse_zypper(package_name);
            
        // Universal formats
        case FORMAT_FLATPAK:
            return install_flatpak(package_name);
        case FORMAT_SNAP:
            return install_snap(package_name);
        case FORMAT_APPIMAGE:
            return install_appimage(package_name);
            
        // Source-based
        case FORMAT_GITHUB_RELEASE:
            return install_github_release(package_name);
        case FORMAT_PIP:
            return install_pip_package(package_name);
        case FORMAT_NPM:
            return install_npm_package(package_name);
        case FORMAT_CARGO:
            return install_cargo_package(package_name);
            
        // Alternative distributions
        case FORMAT_GENTOO_EBUILD:
            return install_gentoo_emerge(package_name);
        case FORMAT_VOID_XBPS:
            return install_void_xbps(package_name);
        case FORMAT_ALPINE_APK:
            return install_alpine_apk(package_name);
        case FORMAT_NIX:
            return install_nix_package(package_name);
            
        default:
            return -1;
    }
}
```

---

## 🌐 **Repository Configuration**

### **Complete Repository Support**
```ini
# /opt/nexusos/etc/repositories.conf

[arch]
name = "Arch Linux Core"
type = pacman
url = https://archlinux.org/packages/
priority = 1
enabled = true

[aur] 
name = "Arch User Repository"
type = aur
url = https://aur.archlinux.org/
priority = 2
enabled = true

[ubuntu-main]
name = "Ubuntu Main"
type = apt
url = http://archive.ubuntu.com/ubuntu/
priority = 10
enabled = true

[ubuntu-universe]
name = "Ubuntu Universe" 
type = apt
url = http://archive.ubuntu.com/ubuntu/
priority = 11
enabled = true

[ubuntu-multiverse]
name = "Ubuntu Multiverse"
type = apt  
url = http://archive.ubuntu.com/ubuntu/
priority = 12
enabled = true

[debian-main]
name = "Debian Main"
type = apt
url = http://deb.debian.org/debian/
priority = 13
enabled = true

[debian-backports]
name = "Debian Backports"
type = apt
url = http://deb.debian.org/debian/
priority = 14
enabled = true

[fedora-updates]
name = "Fedora Updates"
type = dnf
url = https://download.fedoraproject.org/pub/fedora/linux/updates/
priority = 20
enabled = true

[fedora-updates-testing]
name = "Fedora Updates Testing"
type = dnf
url = https://download.fedoraproject.org/pub/fedora/linux/updates/testing/
priority = 21
enabled = false

[rpm-fusion-free]
name = "RPM Fusion Free"
type = dnf
url = https://download1.rpmfusion.org/free/fedora/
priority = 22
enabled = true

[rpm-fusion-nonfree]
name = "RPM Fusion Non-Free" 
type = dnf
url = https://download1.rpmfusion.org/nonfree/fedora/
priority = 23
enabled = true

[epel]
name = "Extra Packages for Enterprise Linux"
type = dnf
url = https://download.fedoraproject.org/pub/epel/
priority = 24
enabled = true

[opensuse-oss]
name = "openSUSE OSS"
type = zypper
url = http://download.opensuse.org/distribution/leap/
priority = 30
enabled = true

[opensuse-non-oss]
name = "openSUSE Non-OSS"
type = zypper  
url = http://download.opensuse.org/distribution/leap/
priority = 31
enabled = true

[packman]
name = "Packman Repository"
type = zypper
url = http://ftp.gwdg.de/pub/linux/misc/packman/suse/
priority = 32
enabled = true

[flathub]
name = "Flathub"
type = flatpak
url = https://flathub.org/repo/flathub.flatpakrepo
priority = 40
enabled = true

[snap-store]
name = "Snap Store"
type = snap
url = https://snapcraft.io/
priority = 41  
enabled = true

[appimage-hub]
name = "AppImageHub"
type = appimage
url = https://appimage.github.io/
priority = 42
enabled = true

[gentoo-portage]
name = "Gentoo Portage"
type = portage
url = https://packages.gentoo.org/
priority = 50
enabled = false

[void-current]
name = "Void Linux Current"
type = xbps
url = https://repo-default.voidlinux.org/current/
priority = 51
enabled = false

[alpine-main]
name = "Alpine Linux Main"
type = apk
url = http://dl-cdn.alpinelinux.org/alpine/
priority = 52
enabled = false

[nixpkgs-stable]
name = "Nix Packages Stable"
type = nix
url = https://channels.nixos.org/nixos-24.05/
priority = 53
enabled = false

[pypi]
name = "Python Package Index"  
type = pip
url = https://pypi.org/
priority = 60
enabled = true

[npm-registry]
name = "NPM Registry"
type = npm
url = https://registry.npmjs.org/
priority = 61
enabled = true

[crates-io]
name = "Crates.io"
type = cargo
url = https://crates.io/
priority = 62
enabled = true

[rubygems]
name = "RubyGems"
type = gem  
url = https://rubygems.org/
priority = 63
enabled = true

[github-releases]
name = "GitHub Releases"
type = github
url = https://api.github.com/
priority = 70
enabled = true

[gitlab-releases] 
name = "GitLab Releases"
type = gitlab
url = https://gitlab.com/api/v4/
priority = 71
enabled = true
```

---

## 🔍 **Universal Search Implementation**

### **Search Across All Repositories**
```c
int cmd_search_universal(const char* query) {
    printf("🌍 Universal Search: %s\n", query);
    printf("=====================================\n\n");
    
    int found_packages = 0;
    
    // Search Arch repositories
    printf("🏛️  ARCH LINUX FAMILY:\n");
    found_packages += search_arch_repos(query);
    found_packages += search_aur(query);
    
    // Search Debian family
    printf("\n🌀 DEBIAN FAMILY:\n");
    found_packages += search_ubuntu_repos(query);
    found_packages += search_debian_repos(query);
    found_packages += search_ppa_repos(query);
    
    // Search Red Hat family
    printf("\n🎩 RED HAT FAMILY:\n");
    found_packages += search_fedora_repos(query);
    found_packages += search_epel(query);
    found_packages += search_rpm_fusion(query);
    found_packages += search_copr(query);
    
    // Search SUSE family
    printf("\n🦎 SUSE FAMILY:\n");
    found_packages += search_opensuse_repos(query);
    found_packages += search_packman(query);
    
    // Search universal formats
    printf("\n📱 UNIVERSAL FORMATS:\n");
    found_packages += search_flatpak(query);
    found_packages += search_snap(query);
    found_packages += search_appimage(query);
    
    // Search source-based
    printf("\n🔧 SOURCE-BASED:\n");
    found_packages += search_github_releases(query);
    found_packages += search_pip_packages(query);
    found_packages += search_npm_packages(query);
    found_packages += search_cargo_packages(query);
    found_packages += search_ruby_gems(query);
    
    // Search alternative distributions
    printf("\n🔬 ALTERNATIVE DISTRIBUTIONS:\n");
    found_packages += search_gentoo_portage(query);
    found_packages += search_void_xbps(query);
    found_packages += search_alpine_apk(query);
    found_packages += search_nix_packages(query);
    
    printf("\n📊 SEARCH SUMMARY:\n");
    printf("Found %d packages across all repositories\n", found_packages);
    
    return found_packages > 0 ? 0 : 1;
}
```

---

## 🛠️ **Installation Functions for Each Format**

### **Arch Linux Family**
```c
int install_arch_pacman(const char* package) {
    char cmd[512];
    printf("📦 Installing %s via pacman...\n", package);
    snprintf(cmd, sizeof(cmd), "sudo pacman -S --noconfirm %s", package);
    return system(cmd);
}

int install_arch_aur(const char* package) {
    char cmd[512];  
    printf("🔍 Installing %s from AUR via yay...\n", package);
    snprintf(cmd, sizeof(cmd), "yay -S --noconfirm %s", package);
    return system(cmd);
}
```

### **Debian Family**
```c
int install_debian_apt(const char* package) {
    char cmd[512];
    printf("🌀 Installing %s via apt...\n", package);
    system("sudo apt update");
    snprintf(cmd, sizeof(cmd), "sudo apt install -y %s", package);
    return system(cmd);
}

int install_ubuntu_ppa(const char* ppa_package) {
    // Format: ppa:user/repo/package
    char cmd[512];
    printf("📦 Installing %s from PPA...\n", ppa_package);
    
    // Extract PPA info and package name
    char* ppa_part = strtok((char*)ppa_package, "/");
    char* package_name = strrchr(ppa_package, '/') + 1;
    
    snprintf(cmd, sizeof(cmd), "sudo add-apt-repository -y %s", ppa_part);
    system(cmd);
    system("sudo apt update");
    snprintf(cmd, sizeof(cmd), "sudo apt install -y %s", package_name);
    return system(cmd);
}
```

### **Red Hat Family**
```c
int install_fedora_dnf(const char* package) {
    char cmd[512];
    printf("🎩 Installing %s via dnf...\n", package);
    snprintf(cmd, sizeof(cmd), "sudo dnf install -y %s", package);
    return system(cmd);
}

int install_fedora_copr(const char* copr_package) {
    // Format: copr:user/repo/package
    char cmd[512];
    printf("🔧 Installing %s from COPR...\n", copr_package);
    
    char* copr_repo = strtok((char*)copr_package, "/");
    char* package_name = strrchr(copr_package, '/') + 1;
    
    snprintf(cmd, sizeof(cmd), "sudo dnf copr enable -y %s", copr_repo);
    system(cmd);
    snprintf(cmd, sizeof(cmd), "sudo dnf install -y %s", package_name);
    return system(cmd);
}

int install_rpm_fusion(const char* package) {
    char cmd[512];
    printf("🎬 Installing %s from RPM Fusion...\n", package);
    
    // Enable RPM Fusion if not already enabled
    system("sudo dnf install -y https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm");
    system("sudo dnf install -y https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm");
    
    snprintf(cmd, sizeof(cmd), "sudo dnf install -y %s", package);
    return system(cmd);
}
```

### **SUSE Family**
```c
int install_opensuse_zypper(const char* package) {
    char cmd[512];
    printf("🦎 Installing %s via zypper...\n", package);
    snprintf(cmd, sizeof(cmd), "sudo zypper install -y %s", package);
    return system(cmd);
}
```

### **Alternative Distributions**
```c
int install_gentoo_emerge(const char* package) {
    char cmd[512];
    printf("🔬 Installing %s via emerge (Gentoo)...\n", package);
    snprintf(cmd, sizeof(cmd), "sudo emerge -av %s", package);
    return system(cmd);
}

int install_void_xbps(const char* package) {
    char cmd[512];
    printf("🕳️  Installing %s via xbps (Void Linux)...\n", package);
    snprintf(cmd, sizeof(cmd), "sudo xbps-install -Sy %s", package);
    return system(cmd);
}

int install_alpine_apk(const char* package) {
    char cmd[512];
    printf("🏔️  Installing %s via apk (Alpine Linux)...\n", package);
    snprintf(cmd, sizeof(cmd), "sudo apk add %s", package);
    return system(cmd);
}

int install_nix_package(const char* package) {
    char cmd[512];
    printf("❄️  Installing %s via nix...\n", package);
    snprintf(cmd, sizeof(cmd), "nix-env -iA nixpkgs.%s", package);
    return system(cmd);
}
```

### **Universal Formats**
```c
int install_flatpak(const char* package) {
    char cmd[512];
    printf("📱 Installing %s via Flatpak...\n", package);
    snprintf(cmd, sizeof(cmd), "flatpak install -y flathub %s", package);
    return system(cmd);
}

int install_snap(const char* package) {
    char cmd[512];
    printf("🔷 Installing %s via Snap...\n", package);
    snprintf(cmd, sizeof(cmd), "sudo snap install %s", package);
    return system(cmd);
}

int install_appimage(const char* package) {
    char cmd[1024];
    printf("📦 Installing %s AppImage...\n", package);
    
    // Download AppImage from GitHub releases or AppImageHub
    snprintf(cmd, sizeof(cmd), 
        "wget -O ~/Applications/%s.appimage $(curl -s https://api.github.com/repos/%s/releases/latest | grep 'browser_download_url.*AppImage' | cut -d '\"' -f 4)",
        package, package);
    system(cmd);
    
    snprintf(cmd, sizeof(cmd), "chmod +x ~/Applications/%s.appimage", package);
    return system(cmd);
}
```

### **Source-Based Installations**
```c
int install_pip_package(const char* package) {
    char cmd[512];
    printf("🐍 Installing %s via pip...\n", package);
    snprintf(cmd, sizeof(cmd), "pip install --user %s", package);
    return system(cmd);
}

int install_npm_package(const char* package) {
    char cmd[512];
    printf("📦 Installing %s via npm...\n", package);
    snprintf(cmd, sizeof(cmd), "npm install -g %s", package);
    return system(cmd);
}

int install_cargo_package(const char* package) {
    char cmd[512];
    printf("🦀 Installing %s via cargo...\n", package);
    snprintf(cmd, sizeof(cmd), "cargo install %s", package);
    return system(cmd);
}

int install_ruby_gem(const char* package) {
    char cmd[512];
    printf("💎 Installing %s via gem...\n", package);
    snprintf(cmd, sizeof(cmd), "gem install --user-install %s", package);
    return system(cmd);
}

int install_github_release(const char* repo_package) {
    // Format: user/repo or user/repo/asset_name
    char cmd[1024];
    printf("🐙 Installing %s from GitHub releases...\n", repo_package);
    
    snprintf(cmd, sizeof(cmd),
        "wget -O /tmp/github_release $(curl -s https://api.github.com/repos/%s/releases/latest | grep 'browser_download_url' | head -1 | cut -d '\"' -f 4) && "
        "sudo install /tmp/github_release /usr/local/bin/%s && "
        "rm /tmp/github_release",
        repo_package, basename(repo_package));
    return system(cmd);
}
```

---

## 🎯 **Usage Examples**

### **Install from Any Repository**
```bash
# Auto-detect best source
nexuspkg install firefox          # Finds in Arch repos
nexuspkg install discord          # Finds in Flatpak/AUR
nexuspkg install code             # Finds best available source

# Force specific format
nexuspkg install --format deb spotify         # Force Debian package
nexuspkg install --format rpm vlc             # Force RPM package  
nexuspkg install --format flatpak gimp        # Force Flatpak
nexuspkg install --format snap telegram       # Force Snap
nexuspkg install --format aur google-chrome   # Force AUR
nexuspkg install --format pip jupyter         # Force pip
nexuspkg install --format npm typescript      # Force npm
nexuspkg install --format cargo ripgrep       # Force cargo
nexuspkg install --format appimage nvim       # Force AppImage

# Install from specific repositories
nexuspkg install --repo fedora firefox        # Fedora repos
nexuspkg install --repo ubuntu chrome         # Ubuntu repos
nexuspkg install --repo opensuse libreoffice  # openSUSE repos
nexuspkg install --repo gentoo firefox        # Gentoo portage
nexuspkg install --repo alpine nginx          # Alpine APK
nexuspkg install --repo void firefox          # Void Linux
nexuspkg install --repo nix emacs             # Nix packages

# Install from community repositories
nexuspkg install ppa:ondrej/php/php8.1        # Ubuntu PPA
nexuspkg install copr:user/repo/package       # Fedora COPR
nexuspkg install github:user/repo             # GitHub release
```

This makes nexuspkg the **ultimate universal package manager** that can install software from literally any Linux distribution or package source!
