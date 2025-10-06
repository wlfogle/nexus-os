QT += core widgets network
CONFIG += c++17

TARGET = nexus-setup-assistant
TEMPLATE = app

# Source files
SOURCES += main.cpp

# Headers (for MOC)
HEADERS += 

# Resources
RESOURCES += resources.qrc

# Installation paths
PREFIX = /usr/local
BINDIR = $(PREFIX)/bin
DESKTOPDIR = /usr/share/applications
ICONDIR = /usr/share/icons/hicolor/128x128/apps

# Build configuration
CONFIG(debug, debug|release) {
    DESTDIR = debug
    OBJECTS_DIR = debug/.obj
    MOC_DIR = debug/.moc
    RCC_DIR = debug/.rcc
} else {
    DESTDIR = release
    OBJECTS_DIR = release/.obj
    MOC_DIR = release/.moc
    RCC_DIR = release/.rcc
}

# Compiler flags
QMAKE_CXXFLAGS += -Wall -Wextra -std=c++17

# Libraries
LIBS += -lQt5Core -lQt5Widgets -lQt5Network

# Include paths
INCLUDEPATH += .

# Qt modules
QT += widgets network

# Platform specific
linux {
    DEFINES += PLATFORM_LINUX
}

# Install rules
target.path = $(BINDIR)
INSTALLS += target

# Desktop file
desktop.files = nexus-setup-assistant.desktop
desktop.path = $(DESKTOPDIR)
INSTALLS += desktop

# Icon
icon.files = icons/nexusos-logo.png
icon.path = $(ICONDIR)
INSTALLS += icon

# Build using qmake + make
all: $(TARGET)

$(TARGET):
	qmake -o Makefile.qt
	make -f Makefile.qt

# Clean
clean:
	rm -rf debug release
	rm -f Makefile.qt
	rm -f *.o moc_*

# Install system dependencies
deps:
	@echo "Installing Qt development dependencies..."
	@if command -v pacman > /dev/null 2>&1; then \
		sudo pacman -S --noconfirm qt5-base qt5-tools qt5-networkauth; \
	elif command -v apt > /dev/null 2>&1; then \
		sudo apt update && sudo apt install -y qtbase5-dev qttools5-dev qtnetworkauth5-dev; \
	elif command -v yum > /dev/null 2>&1; then \
		sudo yum install -y qt5-qtbase-devel qt5-qttools-devel; \
	elif command -v zypper > /dev/null 2>&1; then \
		sudo zypper install -y libqt5-qtbase-devel qt5-tools; \
	else \
		echo "Unsupported package manager. Please install Qt5 development packages manually."; \
		exit 1; \
	fi

# Install the application system-wide
install: $(TARGET)
	@echo "Installing NexusOS Setup Assistant..."
	sudo mkdir -p $(BINDIR)
	sudo mkdir -p $(DESKTOPDIR)
	sudo mkdir -p $(ICONDIR)
	sudo cp $(DESTDIR)/$(TARGET) $(BINDIR)/
	sudo chmod +x $(BINDIR)/$(TARGET)
	sudo cp nexus-setup-assistant.desktop $(DESKTOPDIR)/
	sudo cp icons/nexusos-logo.png $(ICONDIR)/
	@echo "Creating autostart entry..."
	mkdir -p ~/.config/autostart
	cp nexus-setup-assistant-autostart.desktop ~/.config/autostart/
	@echo "Installation complete!"
	@echo "Run: nexus-setup-assistant"

# Uninstall
uninstall:
	@echo "Uninstalling NexusOS Setup Assistant..."
	sudo rm -f $(BINDIR)/$(TARGET)
	sudo rm -f $(DESKTOPDIR)/nexus-setup-assistant.desktop
	sudo rm -f $(ICONDIR)/nexusos-logo.png
	rm -f ~/.config/autostart/nexus-setup-assistant-autostart.desktop
	@echo "Uninstall complete!"

# Package for distribution
package: clean $(TARGET)
	@echo "Creating distribution package..."
	mkdir -p nexus-setup-assistant-dist/bin
	mkdir -p nexus-setup-assistant-dist/share/applications
	mkdir -p nexus-setup-assistant-dist/share/icons
	cp $(DESTDIR)/$(TARGET) nexus-setup-assistant-dist/bin/
	cp nexus-setup-assistant.desktop nexus-setup-assistant-dist/share/applications/
	cp icons/* nexus-setup-assistant-dist/share/icons/
	tar -czf nexus-setup-assistant-$(shell date +%Y%m%d).tar.gz nexus-setup-assistant-dist/
	rm -rf nexus-setup-assistant-dist/
	@echo "Package created: nexus-setup-assistant-$(shell date +%Y%m%d).tar.gz"

# Development run
run: $(TARGET)
	./$(DESTDIR)/$(TARGET)

# Debug run
debug-run: $(TARGET)
	gdb ./$(DESTDIR)/$(TARGET)

# Show build info
info:
	@echo "NexusOS Setup Assistant Build Information"
	@echo "========================================"
	@echo "Target: $(TARGET)"
	@echo "Qt Version: $(shell qmake -version)"
	@echo "Installation paths:"
	@echo "  Binary: $(BINDIR)/$(TARGET)"
	@echo "  Desktop: $(DESKTOPDIR)/nexus-setup-assistant.desktop"
	@echo "  Icon: $(ICONDIR)/nexusos-logo.png"

.PHONY: all clean deps install uninstall package run debug-run info