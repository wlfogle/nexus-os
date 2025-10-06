CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -O2 -g
LDFLAGS = -lcurl -ljson-c -lsqlite3 -larchive -lssl -lcrypto

# Target executable
TARGET = nexuspkg
SOURCE = main.c

# Installation paths
PREFIX = /usr/local
BINDIR = $(PREFIX)/bin
CONFDIR = /etc/nexuspkg
CACHEDIR = /var/cache/nexuspkg
DBDIR = /var/lib/nexuspkg
LOGDIR = /var/log/nexuspkg

# Build target
$(TARGET): $(SOURCE)
	$(CC) $(CFLAGS) -o $(TARGET) $(SOURCE) $(LDFLAGS)

# Install dependencies (for development/testing)
deps:
	@echo "Installing NexusPkg dependencies..."
	@if command -v pacman > /dev/null 2>&1; then \
		sudo pacman -S --noconfirm curl json-c sqlite libarchive openssl; \
	elif command -v apt > /dev/null 2>&1; then \
		sudo apt update && sudo apt install -y libcurl4-openssl-dev libjson-c-dev libsqlite3-dev libarchive-dev libssl-dev; \
	elif command -v yum > /dev/null 2>&1; then \
		sudo yum install -y curl-devel json-c-devel sqlite-devel libarchive-devel openssl-devel; \
	elif command -v zypper > /dev/null 2>&1; then \
		sudo zypper install -y libcurl-devel libjson-c-devel sqlite3-devel libarchive-devel libopenssl-devel; \
	else \
		echo "Unsupported package manager. Please install dependencies manually:"; \
		echo "- libcurl development headers"; \
		echo "- json-c development headers"; \
		echo "- sqlite3 development headers"; \
		echo "- libarchive development headers"; \
		echo "- openssl development headers"; \
		exit 1; \
	fi

# Install NexusPkg system-wide
install: $(TARGET)
	@echo "Installing NexusPkg..."
	sudo mkdir -p $(BINDIR)
	sudo mkdir -p $(CONFDIR)
	sudo mkdir -p $(CACHEDIR)
	sudo mkdir -p $(DBDIR)
	sudo mkdir -p $(LOGDIR)
	sudo cp $(TARGET) $(BINDIR)/
	sudo chmod +x $(BINDIR)/$(TARGET)
	@echo "Creating default configuration..."
	@if [ ! -f $(CONFDIR)/nexuspkg.conf ]; then \
		sudo tee $(CONFDIR)/nexuspkg.conf > /dev/null << 'EOF'; \
# NexusPkg Configuration File\n\
# Main repository\n\
repo_url=https://packages.nexusos.org\n\
ai_models_repo=https://models.nexusos.org\n\
\n\
# External repositories\n\
ubuntu_repo=https://archive.ubuntu.com/ubuntu\n\
debian_repo=https://deb.debian.org/debian\n\
arch_repo=https://mirror.rackspace.com/archlinux\n\
fedora_repo=https://download.fedoraproject.org/pub/fedora/linux\n\
opensuse_repo=https://download.opensuse.org\n\
aur_repo=https://aur.archlinux.org\n\
flatpak_repo=https://dl.flathub.org/repo/flathub.flatpakrepo\n\
appimage_repo=https://appimage.github.io\n\
snap_store=https://api.snapcraft.io\n\
\n\
# Settings\n\
verbose=0\n\
auto_resolve_deps=1\n\
enable_ai_features=1\n\
parallel_downloads=4\n\
enable_deb_support=1\n\
enable_rpm_support=1\n\
enable_zst_support=1\n\
enable_appimage_support=1\n\
enable_flatpak_support=1\n\
enable_snap_support=1\n\
sandbox_untrusted=1\n\
auto_update_repos=1\n\
EOF\
	fi
	sudo chmod 644 $(CONFDIR)/nexuspkg.conf
	sudo chown -R root:root $(CONFDIR)
	sudo chmod 755 $(CACHEDIR)
	sudo chmod 755 $(DBDIR)
	sudo chmod 755 $(LOGDIR)
	@echo "NexusPkg installed successfully!"
	@echo ""
	@echo "Usage:"
	@echo "  nexuspkg sync                    # Sync repositories"
	@echo "  nexuspkg search <query>          # Search packages"
	@echo "  nexuspkg install <package>       # Install package"
	@echo "  nexuspkg list                    # List installed packages"
	@echo "  nexuspkg status                  # Show system status"
	@echo ""
	@echo "Format-specific commands:"
	@echo "  nexuspkg deb install <file>      # Install DEB package"
	@echo "  nexuspkg flatpak <package>       # Install Flatpak"
	@echo "  nexuspkg snap <package>          # Install Snap"
	@echo "  nexuspkg appimage <file>         # Install AppImage"

# Uninstall NexusPkg
uninstall:
	@echo "Removing NexusPkg..."
	sudo rm -f $(BINDIR)/$(TARGET)
	sudo rm -rf $(CONFDIR)
	@echo "Note: Cache and database directories preserved:"
	@echo "  $(CACHEDIR)"
	@echo "  $(DBDIR)"
	@echo "  $(LOGDIR)"
	@echo "Run 'sudo rm -rf $(CACHEDIR) $(DBDIR) $(LOGDIR)' to remove all data"

# Clean build artifacts
clean:
	rm -f $(TARGET) *.o

# Test build
test: $(TARGET)
	@echo "Testing NexusPkg build..."
	./$(TARGET)

# Development setup
dev-setup: deps $(TARGET)
	@echo "Setting up development environment..."
	mkdir -p ./test-env/{cache,db,logs,config}
	@echo "# Development configuration" > ./test-env/config/nexuspkg.conf
	@echo "repo_url=https://packages.nexusos.org" >> ./test-env/config/nexuspkg.conf
	@echo "verbose=1" >> ./test-env/config/nexuspkg.conf
	@echo "Development environment ready in ./test-env/"

# Package for distribution
package: clean $(TARGET)
	@echo "Creating distribution package..."
	mkdir -p nexuspkg-dist/bin
	mkdir -p nexuspkg-dist/etc/nexuspkg
	cp $(TARGET) nexuspkg-dist/bin/
	cp nexuspkg.conf.example nexuspkg-dist/etc/nexuspkg/ 2>/dev/null || true
	tar -czf nexuspkg-$(shell date +%Y%m%d).tar.gz nexuspkg-dist/
	rm -rf nexuspkg-dist/
	@echo "Package created: nexuspkg-$(shell date +%Y%m%d).tar.gz"

# Run with debugging
debug: $(TARGET)
	gdb --args ./$(TARGET)

# Install format-specific tools
install-tools:
	@echo "Installing package format tools..."
	@echo "This will install tools for various package formats..."
	@# DEB tools
	@if command -v pacman > /dev/null 2>&1; then \
		sudo pacman -S --noconfirm dpkg alien || true; \
	fi
	@# Flatpak
	@if command -v pacman > /dev/null 2>&1; then \
		sudo pacman -S --noconfirm flatpak || true; \
	fi
	@# Snap
	@if command -v pacman > /dev/null 2>&1; then \
		sudo pacman -S --noconfirm snapd || true; \
	fi
	@# Python pip
	@if command -v pacman > /dev/null 2>&1; then \
		sudo pacman -S --noconfirm python-pip || true; \
	fi
	@# NPM
	@if command -v pacman > /dev/null 2>&1; then \
		sudo pacman -S --noconfirm npm || true; \
	fi
	@# Rust/Cargo
	@if command -v pacman > /dev/null 2>&1; then \
		sudo pacman -S --noconfirm rust || true; \
	fi
	@echo "Format tools installation complete!"

# Show system information
info:
	@echo "NexusPkg Build Information"
	@echo "========================="
	@echo "Compiler: $(CC)"
	@echo "Flags: $(CFLAGS)"
	@echo "Libraries: $(LDFLAGS)"
	@echo "Target: $(TARGET)"
	@echo ""
	@echo "Installation Paths:"
	@echo "  Binary: $(BINDIR)/$(TARGET)"
	@echo "  Config: $(CONFDIR)/"
	@echo "  Cache: $(CACHEDIR)/"
	@echo "  Database: $(DBDIR)/"
	@echo "  Logs: $(LOGDIR)/"
	@echo ""
	@echo "Supported Formats:"
	@echo "  📦 Native NexusOS packages (.npkg)"
	@echo "  🐧 Debian packages (.deb)"
	@echo "  📦 RPM packages (.rpm)"
	@echo "  🗜️  Arch packages (.pkg.tar.zst)"
	@echo "  📱 AppImage packages (.AppImage)"
	@echo "  📦 Flatpak packages"
	@echo "  📦 Snap packages"
	@echo "  🐍 Python wheels (.whl)"
	@echo "  📦 NPM packages"
	@echo "  🦀 Cargo packages"
	@echo "  📁 Archive formats (tar.gz, tar.xz, zip)"
	@echo "  ⚙️  Binary executables"

.PHONY: deps install uninstall clean test dev-setup package debug install-tools info