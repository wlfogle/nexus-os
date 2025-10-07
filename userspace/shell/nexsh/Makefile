# NexSh - AI-Powered Shell for NexusOS
# Build system with C++20, AI libraries, and universal package support

CXX = g++
CXXFLAGS = -std=c++20 -Wall -Wextra -O2 -g
TARGET = nexsh
PREFIX = /usr/local
SRCDIR = src
BUILDDIR = build
INSTALLDIR = $(PREFIX)/bin
CONFIGDIR = $(PREFIX)/etc/nexsh
DATADIR = $(PREFIX)/share/nexsh

# Source files
SOURCES = $(wildcard $(SRCDIR)/*.cpp) \
          $(wildcard $(SRCDIR)/ai/*.cpp) \
          $(wildcard $(SRCDIR)/packages/*.cpp) \
          $(wildcard $(SRCDIR)/security/*.cpp) \
          $(wildcard $(SRCDIR)/system/*.cpp) \
          $(wildcard $(SRCDIR)/history/*.cpp)

OBJECTS = $(SOURCES:$(SRCDIR)/%.cpp=$(BUILDDIR)/%.o)

# Dependencies
LIBS = -lreadline -lhistory -lcurl -ljson -ltensorflow -lpthread -ldl
INCLUDES = -I$(SRCDIR) -I/usr/include/tensorflow -I/usr/include/nlohmann

# AI and ML libraries
AI_LIBS = -ltensorflow_cc -ltensorflow_framework -lprotobuf
NLP_LIBS = -lspacy_cpp -ltokenizers -lhuggingface_hub

# Package management libraries
PKG_LIBS = -lpackagekit-glib2 -lflatpak -lsnap -lappimage -lrpm -ldpkg

# Security libraries
SEC_LIBS = -lseccomp -lselinux -lapparmor

# System monitoring libraries
SYS_LIBS = -lsystemd -lprocps -lsensors

ALL_LIBS = $(LIBS) $(AI_LIBS) $(NLP_LIBS) $(PKG_LIBS) $(SEC_LIBS) $(SYS_LIBS)

.PHONY: all clean install uninstall test setup-deps

all: setup-dirs $(TARGET)

setup-dirs:
	@mkdir -p $(BUILDDIR)
	@mkdir -p $(BUILDDIR)/ai
	@mkdir -p $(BUILDDIR)/packages
	@mkdir -p $(BUILDDIR)/security
	@mkdir -p $(BUILDDIR)/system
	@mkdir -p $(BUILDDIR)/history

$(TARGET): $(OBJECTS)
	@echo "🔗 Linking NexSh..."
	$(CXX) $(CXXFLAGS) -o $@ $^ $(ALL_LIBS)
	@echo "✅ NexSh built successfully!"

$(BUILDDIR)/%.o: $(SRCDIR)/%.cpp
	@echo "🔨 Compiling $<..."
	$(CXX) $(CXXFLAGS) $(INCLUDES) -c $< -o $@

# Installation targets
install: $(TARGET)
	@echo "📦 Installing NexSh..."
	sudo mkdir -p $(INSTALLDIR)
	sudo mkdir -p $(CONFIGDIR)
	sudo mkdir -p $(DATADIR)
	sudo cp $(TARGET) $(INSTALLDIR)/
	sudo chmod +x $(INSTALLDIR)/$(TARGET)
	
	# Install configuration files
	sudo cp config/nexsh.conf $(CONFIGDIR)/
	sudo cp config/ai-models.conf $(CONFIGDIR)/
	sudo cp config/package-sources.conf $(CONFIGDIR)/
	
	# Install AI models and data
	sudo cp -r models/ $(DATADIR)/
	sudo cp -r completions/ $(DATADIR)/
	
	# Create symlink for easy access
	sudo ln -sf $(INSTALLDIR)/$(TARGET) /usr/bin/nexsh
	
	# Add to /etc/shells
	echo "$(INSTALLDIR)/$(TARGET)" | sudo tee -a /etc/shells
	
	@echo "✅ NexSh installed successfully!"
	@echo "🚀 Run 'nexsh' to start the AI-powered shell"

uninstall:
	@echo "🗑️  Uninstalling NexSh..."
	sudo rm -f $(INSTALLDIR)/$(TARGET)
	sudo rm -f /usr/bin/nexsh
	sudo rm -rf $(CONFIGDIR)
	sudo rm -rf $(DATADIR)
	sudo sed -i '/nexsh/d' /etc/shells
	@echo "✅ NexSh uninstalled"

# Development and testing
test: $(TARGET)
	@echo "🧪 Running NexSh tests..."
	./$(TARGET) --test-mode
	@echo "✅ Tests completed"

debug: CXXFLAGS += -DDEBUG -g3 -O0
debug: $(TARGET)

profile: CXXFLAGS += -pg
profile: $(TARGET)

clean:
	@echo "🧹 Cleaning build files..."
	rm -rf $(BUILDDIR)
	rm -f $(TARGET)
	rm -f gmon.out
	@echo "✅ Clean complete"

# Dependency setup for different distributions
setup-deps:
	@echo "📚 Setting up dependencies for NexSh..."
	@if command -v apt-get >/dev/null 2>&1; then \
		echo "🐧 Detected Debian/Ubuntu system"; \
		sudo apt-get update; \
		sudo apt-get install -y libreadline-dev libcurl4-openssl-dev \
			libjsoncpp-dev libtensorflow-dev libpackagekit-glib2-dev \
			libflatpak-dev libseccomp-dev libsystemd-dev libprocps-dev \
			libsensors4-dev build-essential cmake pkg-config; \
	elif command -v pacman >/dev/null 2>&1; then \
		echo "🏗️  Detected Arch Linux system"; \
		sudo pacman -Sy --needed readline curl jsoncpp tensorflow \
			packagekit flatpak libseccomp systemd procps-ng \
			lm_sensors base-devel cmake pkgconf; \
	elif command -v dnf >/dev/null 2>&1; then \
		echo "🎩 Detected Fedora/RHEL system"; \
		sudo dnf install -y readline-devel libcurl-devel jsoncpp-devel \
			tensorflow-devel PackageKit-glib-devel flatpak-devel \
			libseccomp-devel systemd-devel procps-ng-devel \
			lm_sensors-devel gcc-c++ cmake pkgconfig; \
	elif command -v zypper >/dev/null 2>&1; then \
		echo "🦎 Detected openSUSE system"; \
		sudo zypper install -y readline-devel libcurl-devel \
			libjsoncpp-devel tensorflow-devel PackageKit-devel \
			flatpak-devel libseccomp-devel systemd-devel \
			procps-devel sensors-devel gcc-c++ cmake pkg-config; \
	else \
		echo "❌ Unsupported distribution. Please install dependencies manually."; \
		echo "Required: readline, curl, jsoncpp, tensorflow, packagekit, flatpak"; \
		echo "         libseccomp, systemd, procps, lm_sensors, build tools"; \
		exit 1; \
	fi
	@echo "✅ Dependencies installed successfully!"

# Create systemd user service for NexSh daemon
install-service:
	@echo "🔧 Installing NexSh systemd service..."
	mkdir -p ~/.config/systemd/user
	cp systemd/nexsh-daemon.service ~/.config/systemd/user/
	systemctl --user daemon-reload
	systemctl --user enable nexsh-daemon.service
	@echo "✅ NexSh daemon service installed"

# Package creation
package:
	@echo "📦 Creating NexSh package..."
	mkdir -p nexsh-1.0.0
	cp -r src/ nexsh-1.0.0/
	cp -r config/ nexsh-1.0.0/
	cp -r models/ nexsh-1.0.0/
	cp -r completions/ nexsh-1.0.0/
	cp Makefile README.md LICENSE nexsh-1.0.0/
	tar czf nexsh-1.0.0.tar.gz nexsh-1.0.0/
	rm -rf nexsh-1.0.0/
	@echo "✅ Package created: nexsh-1.0.0.tar.gz"

# Development helpers
format:
	@echo "🎨 Formatting code..."
	find $(SRCDIR) -name "*.cpp" -o -name "*.h" | xargs clang-format -i
	@echo "✅ Code formatted"

lint:
	@echo "🔍 Running static analysis..."
	cppcheck --enable=all --suppress=missingIncludeSystem $(SRCDIR)/
	@echo "✅ Static analysis complete"

docs:
	@echo "📖 Generating documentation..."
	doxygen Doxyfile
	@echo "✅ Documentation generated in docs/"

# Show build information
info:
	@echo "🔍 NexSh Build Information:"
	@echo "  Target: $(TARGET)"
	@echo "  Compiler: $(CXX)"
	@echo "  Flags: $(CXXFLAGS)"
	@echo "  Libraries: $(ALL_LIBS)"
	@echo "  Install prefix: $(PREFIX)"

# Help target
help:
	@echo "🚀 NexSh Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build NexSh"
	@echo "  install      - Install NexSh system-wide"
	@echo "  uninstall    - Remove NexSh from system"
	@echo "  test         - Run tests"
	@echo "  debug        - Build debug version"
	@echo "  profile      - Build profiling version"
	@echo "  clean        - Clean build files"
	@echo "  setup-deps   - Install build dependencies"
	@echo "  install-service - Install systemd service"
	@echo "  package      - Create distribution package"
	@echo "  format       - Format source code"
	@echo "  lint         - Run static analysis"
	@echo "  docs         - Generate documentation"
	@echo "  info         - Show build information"
	@echo "  help         - Show this help"