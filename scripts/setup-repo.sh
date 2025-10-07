#!/bin/bash
#
# NexusOS Repository Infrastructure Setup
# Creates custom NexusOS package repositories and signing keys
#
# Usage: sudo ./setup-repo.sh [repo-directory]
#

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
REPO_DIR="${1:-${PROJECT_ROOT}/repository}"
REPO_NAME="nexusos"
GPG_KEY_NAME="NexusOS Package Signing Key"
GPG_KEY_EMAIL="packages@nexusos.org"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

print_banner() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════╗"
    echo "║           NexusOS Repository Setup               ║"
    echo "║        Package Repository Infrastructure         ║"
    echo "╚══════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

create_directory_structure() {
    print_status "Creating repository directory structure..."
    
    mkdir -p "$REPO_DIR"
    mkdir -p "$REPO_DIR/x86_64"
    mkdir -p "$REPO_DIR/sources"
    mkdir -p "$REPO_DIR/keys"
    mkdir -p "$REPO_DIR/cache"
    
    print_status "Directory structure created at: $REPO_DIR"
}

generate_signing_key() {
    print_status "Generating GPG signing key for NexusOS packages..."
    
    # Check if key already exists
    if gpg --list-secret-keys --keyid-format LONG | grep -q "$GPG_KEY_EMAIL"; then
        print_warning "GPG key for $GPG_KEY_EMAIL already exists"
        return 0
    fi
    
    # Generate GPG key
    cat > /tmp/gpg-batch << EOF
%echo Generating NexusOS package signing key
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: $GPG_KEY_NAME
Name-Email: $GPG_KEY_EMAIL
Expire-Date: 2y
Passphrase: 
%commit
%echo done
EOF
    
    gpg --batch --generate-key /tmp/gpg-batch
    rm /tmp/gpg-batch
    
    # Export public key
    gpg --armor --export "$GPG_KEY_EMAIL" > "$REPO_DIR/keys/nexusos.gpg"
    gpg --armor --export-secret-keys "$GPG_KEY_EMAIL" > "$REPO_DIR/keys/nexusos-secret.gpg"
    
    print_status "GPG signing key generated and exported"
}

create_pacman_config() {
    print_status "Creating pacman repository configuration..."
    
    cat > "$REPO_DIR/nexusos.conf" << EOF
#
# NexusOS Repository Configuration
# Add this to /etc/pacman.conf to enable NexusOS repositories
#

[nexusos-core]
SigLevel = Required DatabaseOptional
Server = https://repo.nexusos.org/\$arch/core

[nexusos-extra]
SigLevel = Required DatabaseOptional  
Server = https://repo.nexusos.org/\$arch/extra

[nexusos-community]
SigLevel = Required DatabaseOptional
Server = https://repo.nexusos.org/\$arch/community

# Universal package compatibility layer
[nexusos-universal]
SigLevel = Optional TrustAll
Server = https://repo.nexusos.org/\$arch/universal
EOF

    print_status "Pacman configuration created"
}

create_repository_database() {
    print_status "Creating repository database..."
    
    cd "$REPO_DIR/x86_64"
    
    # Create empty database
    repo-add --sign --key "$GPG_KEY_EMAIL" nexusos-core.db.tar.xz
    repo-add --sign --key "$GPG_KEY_EMAIL" nexusos-extra.db.tar.xz
    repo-add --sign --key "$GPG_KEY_EMAIL" nexusos-community.db.tar.xz
    repo-add --sign --key "$GPG_KEY_EMAIL" nexusos-universal.db.tar.xz
    
    print_status "Repository databases created"
}

build_nexusos_packages() {
    print_status "Building core NexusOS packages..."
    
    # Create nexuspkg PKGBUILD
    mkdir -p "$REPO_DIR/sources/nexuspkg"
    cat > "$REPO_DIR/sources/nexuspkg/PKGBUILD" << 'EOF'
# Maintainer: NexusOS Team <packages@nexusos.org>
pkgname=nexuspkg
pkgver=1.0.0
pkgrel=1
epoch=1
pkgdesc="NexusOS Universal Package Manager"
arch=('x86_64')
url="https://nexusos.org"
license=('GPL3')
depends=('curl' 'json-c' 'pacman' 'flatpak' 'snapd' 'docker')
makedepends=('cmake' 'ninja' 'gcc')
source=("nexuspkg-${pkgver}.tar.gz::https://github.com/nexusos/nexus-os/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/nexus-os-${pkgver}"
    
    # Build nexuspkg
    cd src/nexuspkg
    mkdir -p build
    cd build
    cmake .. -GNinja -DCMAKE_BUILD_TYPE=Release
    ninja
}

package() {
    cd "$srcdir/nexus-os-${pkgver}/src/nexuspkg/build"
    
    # Install binary
    install -Dm755 nexuspkg "$pkgdir/usr/bin/nexuspkg"
    
    # Install configuration
    install -Dm644 ../nexuspkg.conf "$pkgdir/etc/nexuspkg/nexuspkg.conf"
    
    # Install systemd service
    install -Dm644 ../nexuspkg.service "$pkgdir/usr/lib/systemd/system/nexuspkg.service"
    
    # Install man page
    install -Dm644 ../docs/nexuspkg.1 "$pkgdir/usr/share/man/man1/nexuspkg.1"
}
EOF

    # Create stella-ai PKGBUILD
    mkdir -p "$REPO_DIR/sources/stella-ai"
    cat > "$REPO_DIR/sources/stella-ai/PKGBUILD" << 'EOF'
# Maintainer: NexusOS Team <packages@nexusos.org>
pkgname=stella-ai
pkgver=1.0.0
pkgrel=1
pkgdesc="Stella AI - NexusOS Security Guardian (Golden Retriever mascot)"
arch=('any')
url="https://nexusos.org"
license=('GPL3')
depends=('python' 'python-fastapi' 'python-requests' 'python-psutil')
source=("stella-ai-${pkgver}.tar.gz::https://github.com/nexusos/nexus-os/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')

package() {
    cd "$srcdir/nexus-os-${pkgver}"
    
    # Install Python module
    install -Dm644 core/ai/stella/* "$pkgdir/usr/lib/python3.11/site-packages/stella/"
    
    # Install executable
    install -Dm755 core/ai/stella/stella.py "$pkgdir/usr/bin/stella"
    
    # Install systemd service
    install -Dm644 core/ai/stella/stella.service "$pkgdir/usr/lib/systemd/system/stella.service"
    
    # Install configuration
    install -Dm644 core/ai/stella/config.yaml "$pkgdir/etc/stella/config.yaml"
}
EOF

    # Create maxjr-ai PKGBUILD
    mkdir -p "$REPO_DIR/sources/maxjr-ai"
    cat > "$REPO_DIR/sources/maxjr-ai/PKGBUILD" << 'EOF'
# Maintainer: NexusOS Team <packages@nexusos.org>
pkgname=maxjr-ai
pkgver=1.0.0
pkgrel=1
pkgdesc="Max Jr. AI - NexusOS Performance Optimizer (Cat mascot)"
arch=('any')
url="https://nexusos.org"
license=('GPL3')
depends=('python' 'python-fastapi' 'python-requests' 'python-psutil')
source=("maxjr-ai-${pkgver}.tar.gz::https://github.com/nexusos/nexus-os/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')

package() {
    cd "$srcdir/nexus-os-${pkgver}"
    
    # Install Python module
    install -Dm644 core/ai/maxjr/* "$pkgdir/usr/lib/python3.11/site-packages/maxjr/"
    
    # Install executable
    install -Dm755 core/ai/maxjr/maxjr.py "$pkgdir/usr/bin/maxjr"
    
    # Install systemd service
    install -Dm644 core/ai/maxjr/maxjr.service "$pkgdir/usr/lib/systemd/system/maxjr.service"
    
    # Install configuration
    install -Dm644 core/ai/maxjr/config.yaml "$pkgdir/etc/maxjr/config.yaml"
}
EOF

    # Create nexusos-branding PKGBUILD
    mkdir -p "$REPO_DIR/sources/nexusos-branding"
    cat > "$REPO_DIR/sources/nexusos-branding/PKGBUILD" << 'EOF'
# Maintainer: NexusOS Team <packages@nexusos.org>
pkgname=nexusos-branding
pkgver=1.0.0
pkgrel=1
pkgdesc="NexusOS branding and visual identity"
arch=('any')
url="https://nexusos.org"
license=('GPL3')
depends=()
source=("nexusos-branding-${pkgver}.tar.gz::https://github.com/nexusos/nexus-os/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')

package() {
    cd "$srcdir/nexus-os-${pkgver}"
    
    # Install wallpapers
    install -Dm644 assets/wallpapers/* "$pkgdir/usr/share/pixmaps/nexusos/"
    
    # Install plymouth theme
    install -Dm644 assets/plymouth/* "$pkgdir/usr/share/plymouth/themes/nexusos/"
    
    # Install GRUB theme
    install -Dm644 assets/grub/* "$pkgdir/usr/share/grub/themes/nexusos/"
    
    # Install SDDM theme
    install -Dm644 assets/sddm/* "$pkgdir/usr/share/sddm/themes/nexusos/"
}
EOF

    print_status "NexusOS package sources created"
}

create_update_script() {
    print_status "Creating repository update script..."
    
    cat > "$REPO_DIR/update-repo.sh" << 'EOF'
#!/bin/bash
#
# NexusOS Repository Update Script
# Updates repository databases after adding packages
#

set -e

REPO_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
GPG_KEY_EMAIL="packages@nexusos.org"

echo "Updating NexusOS repository databases..."

cd "$REPO_DIR/x86_64"

# Update each repository
for repo in nexusos-core nexusos-extra nexusos-community nexusos-universal; do
    echo "Updating $repo..."
    
    if [[ -f "$repo.db.tar.xz" ]]; then
        # Update existing database
        repo-add --sign --key "$GPG_KEY_EMAIL" "$repo.db.tar.xz" *.pkg.tar.xz 2>/dev/null || true
    else
        # Create new database
        repo-add --sign --key "$GPG_KEY_EMAIL" "$repo.db.tar.xz"
    fi
done

echo "Repository update completed!"
echo
echo "To sync with server:"
echo "  rsync -av --delete $REPO_DIR/ nexusos@repo.nexusos.org:/var/www/repo/"
echo
echo "To add package to repository:"
echo "  cp package.pkg.tar.xz $REPO_DIR/x86_64/"
echo "  cd $REPO_DIR/x86_64/"
echo "  repo-add --sign --key $GPG_KEY_EMAIL nexusos-extra.db.tar.xz package.pkg.tar.xz"
EOF

    chmod +x "$REPO_DIR/update-repo.sh"
    
    print_status "Repository update script created"
}

create_install_instructions() {
    print_status "Creating installation instructions..."
    
    cat > "$REPO_DIR/INSTALL.md" << EOF
# NexusOS Repository Installation

## Adding NexusOS Repository to Existing System

### 1. Import GPG Key

\`\`\`bash
# Download and import the NexusOS signing key
sudo curl -o /etc/pacman.d/nexusos.gpg https://repo.nexusos.org/keys/nexusos.gpg
sudo pacman-key --add /etc/pacman.d/nexusos.gpg
sudo pacman-key --lsign-key packages@nexusos.org
\`\`\`

### 2. Add Repository Configuration

Add the following to your \`/etc/pacman.conf\`:

\`\`\`ini
# NexusOS Universal Package Repository
[nexusos-core]
SigLevel = Required DatabaseOptional
Server = https://repo.nexusos.org/\$arch/core

[nexusos-extra]
SigLevel = Required DatabaseOptional  
Server = https://repo.nexusos.org/\$arch/extra

[nexusos-community]
SigLevel = Required DatabaseOptional
Server = https://repo.nexusos.org/\$arch/community

[nexusos-universal]
SigLevel = Optional TrustAll
Server = https://repo.nexusos.org/\$arch/universal
\`\`\`

### 3. Update Package Database

\`\`\`bash
sudo pacman -Sy
\`\`\`

### 4. Install NexusOS Components

\`\`\`bash
# Install universal package manager
sudo pacman -S nexuspkg

# Install AI companions
sudo pacman -S stella-ai maxjr-ai

# Install branding (optional)
sudo pacman -S nexusos-branding

# Enable AI services
sudo systemctl enable --now stella maxjr
\`\`\`

### 5. Test Installation

\`\`\`bash
# Test nexuspkg
nexuspkg --version
nexuspkg status

# Test AI companions  
stella --help
maxjr --help

# Test universal package installation
nexuspkg install firefox
nexuspkg search "video editor"
\`\`\`

## Repository Contents

- **nexusos-core**: Essential NexusOS components
- **nexusos-extra**: Additional tools and applications
- **nexusos-community**: Community-contributed packages
- **nexusos-universal**: Cross-distribution compatibility packages

## Package Signing

All packages are signed with the NexusOS signing key:
- **Key ID**: packages@nexusos.org
- **Fingerprint**: Available at https://repo.nexusos.org/keys/
- **Public Key**: https://repo.nexusos.org/keys/nexusos.gpg

## Support

- GitHub Issues: https://github.com/nexusos/nexus-os/issues
- Documentation: https://docs.nexusos.org
- Repository Status: https://status.nexusos.org
EOF

    print_status "Installation instructions created"
}

main() {
    print_banner
    
    # Check if running as root
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
    
    print_status "Setting up NexusOS repository infrastructure..."
    print_status "Repository directory: $REPO_DIR"
    
    create_directory_structure
    generate_signing_key
    create_pacman_config
    create_repository_database
    build_nexusos_packages
    create_update_script
    create_install_instructions
    
    print_status "Repository setup completed successfully!"
    
    echo
    echo -e "${GREEN}╔══════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║            REPOSITORY SETUP COMPLETE!           ║${NC}"
    echo -e "${GREEN}╠══════════════════════════════════════════════════╣${NC}"
    echo -e "${GREEN}║${NC} Repository: $REPO_DIR"
    echo -e "${GREEN}║${NC} GPG Key: $GPG_KEY_EMAIL"
    echo -e "${GREEN}║${NC} Configuration: $REPO_DIR/nexusos.conf"
    echo -e "${GREEN}║${NC} Update Script: $REPO_DIR/update-repo.sh"
    echo -e "${GREEN}║${NC}"
    echo -e "${GREEN}║${NC} Next Steps:"
    echo -e "${GREEN}║${NC}   1. Build packages: cd sources/package && makepkg"
    echo -e "${GREEN}║${NC}   2. Add to repo: ./update-repo.sh"
    echo -e "${GREEN}║${NC}   3. Deploy to server: rsync to repo.nexusos.org"
    echo -e "${GREEN}║${NC}   4. Test installation: See INSTALL.md"
    echo -e "${GREEN}╚══════════════════════════════════════════════════╝${NC}"
}

main "$@"
EOF