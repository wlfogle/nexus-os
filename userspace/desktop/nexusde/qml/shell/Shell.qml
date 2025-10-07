import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Window 2.15
import QtQuick.Effects
import NexusDE.Shell 1.0
import "components"
import "panels"
import "widgets"
import "../themes"

ApplicationWindow {
    id: shellWindow
    
    // Window properties
    title: "NexusDE - AI-Powered Desktop Environment"
    visible: true
    flags: Qt.FramelessWindowHint | Qt.WindowStaysOnBottomHint
    color: "transparent"
    
    // Full screen coverage
    width: Screen.width
    height: Screen.height
    x: Screen.virtualX
    y: Screen.virtualY
    
    // NexusOS Branding Colors
    readonly property color nexusPrimary: "#0066cc"      // Deep blue
    readonly property color nexusSecondary: "#00cc66"    // Nexus green
    readonly property color nexusAccent: "#cc6600"       // Warm orange
    readonly property color nexusDark: "#1a1a1a"         // Dark background
    readonly property color nexusLight: "#f0f0f0"        // Light text
    readonly property color nexusGlass: "#2a2a2a80"      // Glass effect
    
    // Garuda-inspired glass morphism
    readonly property color garudaGlass: "#1e1e2e80"
    readonly property color garudaAccent: "#89b4fa"
    readonly property color garudaRed: "#f38ba8"
    readonly property color garudaGreen: "#a6e3a1"
    readonly property color garudaYellow: "#f9e2af"
    
    // AI status indicators
    property bool aiActive: nexusShell.aiEnabled
    property bool digitalFortressActive: securityStatus.digitalFortressStatus
    property bool vaultwardenActive: securityStatus.vaultwardenStatus
    property bool wireguardActive: false
    property real aiConfidence: nexusShell.aiConfidence
    
    // Background with dynamic effects
    Rectangle {
        anchors.fill: parent
        
        // Dynamic gradient background
        gradient: Gradient {
            GradientStop { 
                position: 0.0; 
                color: aiActive ? nexusPrimary + "40" : nexusDark + "60"
            }
            GradientStop { 
                position: 0.5; 
                color: digitalFortressActive ? nexusSecondary + "20" : nexusDark + "80"
            }
            GradientStop { 
                position: 1.0; 
                color: nexusDark 
            }
            
            Behavior on color { ColorAnimation { duration: 500 } }
        }
        
        // Animated particle system for AI activity
        ParticleSystem {
            id: aiParticles
            anchors.fill: parent
            running: aiActive
            
            ImageParticle {
                source: "qrc:/images/particle.png"
                color: nexusSecondary
                alpha: 0.1
                entryEffect: ImageParticle.Scale
            }
            
            Emitter {
                id: particleEmitter
                anchors.centerIn: parent
                emitRate: aiActive ? 50 : 0
                lifeSpan: 3000
                size: 8
                sizeVariation: 4
                
                velocity: AngleDirection {
                    angle: 0
                    angleVariation: 360
                    magnitude: 50
                    magnitudeVariation: 30
                }
                
                Behavior on emitRate { NumberAnimation { duration: 300 } }
            }
        }
    }
    
    // Top Toolbar - Garuda Style
    TopToolbar {
        id: topToolbar
        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
        }
        height: 45
        
        // Glassmorphism background
        background: Rectangle {
            color: garudaGlass
            border.color: nexusSecondary + "30"
            border.width: 1
            radius: 0
            
            // Blur effect
            layer.enabled: true
            layer.effect: MultiEffect {
                blurEnabled: true
                blur: 0.4
                saturation: 1.2
                brightness: 0.1
            }
        }
        
        RowLayout {
            anchors.fill: parent
            anchors.margins: 8
            spacing: 12
            
            // NexusOS Logo and Branding
            Item {
                Layout.preferredWidth: 180
                Layout.fillHeight: true
                
                RowLayout {
                    anchors.fill: parent
                    spacing: 8
                    
                    // Stella & Max Jr. Duo Mascot
                    NexusLogoStellaMacJr {
                        id: duoMascot
                        Layout.preferredWidth: 45
                        Layout.preferredHeight: 32
                        
                        aiActive: shellWindow.aiActive
                        stellaActive: packageManager.isActive || securityStatus.digitalFortressActive
                        maxActive: systemMonitor.isOptimizing || systemMonitor.cpuUsage > 0
                        
                        // Show different states based on system activity
                        property bool installing: packageManager.isInstalling
                        property bool securing: securityStatus.digitalFortressActive
                        property bool monitoring: systemMonitor.isMonitoring
                        
                        // Dynamic scaling based on activity
                        scale: {
                            if (installing || securing) return 1.1
                            if (monitoring) return 1.05
                            return 1.0
                        }
                        
                        Behavior on scale { ScaleAnimator { duration: 300 } }
                    }
                    
                    Column {
                        Layout.fillWidth: true
                        
                        Text {
                            text: "NexusOS"
                            font.family: "Inter"
                            font.weight: Font.Bold
                            font.pixelSize: 14
                            color: nexusLight
                        }
                        
                        Text {
                            text: aiActive ? "🤖 AI Enhanced" : "Standard Mode"
                            font.family: "Inter"
                            font.pixelSize: 10
                            color: aiActive ? nexusSecondary : nexusLight + "80"
                            opacity: 0.8
                        }
                    }
                }
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: universalLauncher.toggle()
                    cursorShape: Qt.PointingHandCursor
                    
                    hoverEnabled: true
                    onEntered: parent.scale = 1.05
                    onExited: parent.scale = 1.0
                    
                    Behavior on scale { ScaleAnimator { duration: 200 } }
                }
            }
            
            // System Status Indicators
            RowLayout {
                Layout.fillWidth: true
                spacing: 16
                
                // AI Confidence Indicator
                AIConfidenceWidget {
                    confidence: aiConfidence
                    visible: aiActive
                }
                
                // Digital Fortress Status
                SecurityStatusWidget {
                    id: fortressStatus
                    title: "Digital Fortress"
                    active: digitalFortressActive
                    statusColor: digitalFortressActive ? garudaGreen : garudaRed
                    icon: "🛡️"
                    
                    MouseArea {
                        anchors.fill: parent
                        onClicked: securityStatus.toggleDigitalFortress()
                        cursorShape: Qt.PointingHandCursor
                    }
                }
                
                // Vaultwarden Status
                SecurityStatusWidget {
                    title: "Vaultwarden"
                    active: vaultwardenActive
                    statusColor: vaultwardenActive ? garudaGreen : garudaYellow
                    icon: "🔐"
                    
                    MouseArea {
                        anchors.fill: parent
                        onClicked: securityStatus.toggleVaultwarden()
                        cursorShape: Qt.PointingHandCursor
                    }
                }
                
                // WireGuard Killswitch
                WireGuardKillswitch {
                    id: wireguardKillswitch
                    active: wireguardActive
                    
                    onToggleRequested: {
                        wireguardActive = !wireguardActive
                        securityStatus.setWireGuardKillswitch(wireguardActive)
                    }
                }
            }
            
            // System Metrics
            SystemMetricsWidget {
                Layout.preferredWidth: 200
                
                cpuUsage: systemMonitor.cpuUsage
                memoryUsage: systemMonitor.memoryUsage
                gpuUsage: systemMonitor.gpuUsage
                networkActivity: systemMonitor.networkActivity
            }
            
            // System Tray and Controls
            RowLayout {
                Layout.preferredWidth: 120
                spacing: 8
                
                // Workspace Switcher
                WorkspaceSwitcher {
                    currentWorkspace: nexusShell.currentWorkspace
                    onWorkspaceChanged: nexusShell.setCurrentWorkspace(workspace)
                }
                
                // System tray
                SystemTray {
                    Layout.preferredWidth: 24
                    Layout.preferredHeight: 24
                }
                
                // User menu
                UserMenu {
                    Layout.preferredWidth: 32
                    Layout.preferredHeight: 32
                    
                    userAvatar: "qrc:/images/user-avatar.png"
                    userName: systemMonitor.currentUser
                }
            }
        }
    }
    
    // Main Desktop Area
    Rectangle {
        id: desktopArea
        anchors {
            top: topToolbar.bottom
            bottom: bottomTaskbar.top
            left: parent.left
            right: parent.right
        }
        color: "transparent"
        
        // Desktop Icons and Widgets
        GridLayout {
            anchors {
                top: parent.top
                left: parent.left
                margins: 20
            }
            columns: 1
            rowSpacing: 20
            
            // AI Assistant Widget (floating)
            AIAssistantWidget {
                id: aiWidget
                visible: aiActive
                Layout.preferredWidth: 300
                Layout.preferredHeight: 150
                
                onCommandRequested: nexusShell.handleAICommand(command)
            }
            
            // Quick Launch Icons
            QuickLaunchGrid {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 160
                
                apps: [
                    "firefox", "konsole", "dolphin", "code", 
                    "discord", "steam", "gimp", "obs"
                ]
                
                onAppLaunched: nexusShell.launchApplication(appId)
            }
        }
        
        // Wallpaper and desktop effects
        DesktopWallpaper {
            anchors.fill: parent
            z: -1
            source: "qrc:/images/nexusos-wallpaper.jpg"
            aiActive: shellWindow.aiActive
        }
    }
    
    // Bottom Taskbar - Garuda Style
    BottomTaskbar {
        id: bottomTaskbar
        anchors {
            bottom: parent.bottom
            left: parent.left
            right: parent.right
        }
        height: 50
        
        // Glassmorphism background
        background: Rectangle {
            color: garudaGlass
            border.color: nexusSecondary + "30"
            border.width: 1
            radius: 8
            
            // Blur effect
            layer.enabled: true
            layer.effect: MultiEffect {
                blurEnabled: true
                blur: 0.4
                saturation: 1.2
                brightness: 0.1
            }
        }
        
        RowLayout {
            anchors.fill: parent
            anchors.margins: 8
            spacing: 8
            
            // Application Launcher Button
            LauncherButton {
                Layout.preferredWidth: 40
                Layout.preferredHeight: 40
                
                icon: "qrc:/images/nexusos-icon.svg"
                tooltip: "Universal Application Launcher (AI-Enhanced)"
                
                onClicked: universalLauncher.toggle()
            }
            
            // Active Applications
            ActiveApplications {
                Layout.fillWidth: true
                Layout.fillHeight: true
                
                applications: systemMonitor.activeApplications
                currentWorkspace: nexusShell.currentWorkspace
                
                onApplicationClicked: systemMonitor.focusApplication(appId)
                onApplicationClosed: systemMonitor.closeApplication(appId)
            }
            
            // System Tray
            SystemTrayArea {
                Layout.preferredWidth: 150
                Layout.fillHeight: true
                
                // Package Manager Status
                PackageStatusIndicator {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.rightMargin: 8
                    
                    updateCount: packageManager.pendingUpdates
                    installing: packageManager.isInstalling
                    
                    onClicked: packageManager.showUpdateDialog()
                }
            }
            
            // Digital Clock
            DigitalClock {
                Layout.preferredWidth: 120
                Layout.fillHeight: true
                
                timeFormat: "hh:mm:ss"
                dateFormat: "MMM dd"
                showSeconds: true
            }
        }
    }
    
    // Universal Launcher Overlay
    UniversalLauncherOverlay {
        id: launcherOverlay
        anchors.fill: parent
        visible: universalLauncher.visible
        
        onSearchQuery: nexusShell.handleSearchQuery(query)
        onAppSelected: {
            nexusShell.launchApplication(appId)
            universalLauncher.hide()
        }
        onInstallRequested: {
            nexusShell.installPackage(packageName, format)
        }
    }
    
    // Context Menu
    ContextMenuOverlay {
        id: contextMenu
        anchors.fill: parent
        
        onAiToggleRequested: nexusShell.setAIEnabled(!nexusShell.aiEnabled)
        onSettingsRequested: systemSettings.show()
        onTerminalRequested: nexusShell.launchApplication("konsole")
        onFileManagerRequested: nexusShell.launchApplication("dolphin")
    }
    
    // Notification System
    NotificationArea {
        id: notifications
        anchors {
            top: topToolbar.bottom
            right: parent.right
            margins: 10
        }
        width: 350
        
        onNotificationClicked: handleNotification(notificationId)
    }
    
    // Performance Monitoring Overlay
    PerformanceOverlay {
        id: perfOverlay
        visible: systemMonitor.showPerformanceOverlay
        anchors {
            top: topToolbar.bottom
            left: parent.left
            margins: 10
        }
        
        metrics: systemMonitor.detailedMetrics
        onOptimizationRequested: nexusShell.optimizeSystemResources()
    }
    
    // Connections for shell integration
    Connections {
        target: nexusShell
        
        function onAiEnabledChanged() {
            // Update UI when AI state changes
            aiParticles.restart()
        }
        
        function onWorkspaceChanged(workspace) {
            // Animate workspace transition
            workspaceTransition.start()
        }
        
        function onApplicationLaunched(appId) {
            // Show launch feedback
            showLaunchFeedback(appId)
        }
    }
    
    // Workspace transition animation
    SequentialAnimation {
        id: workspaceTransition
        
        ParallelAnimation {
            NumberAnimation {
                target: desktopArea
                property: "opacity"
                to: 0.3
                duration: 150
            }
            NumberAnimation {
                target: desktopArea
                property: "scale"
                to: 0.95
                duration: 150
            }
        }
        
        ParallelAnimation {
            NumberAnimation {
                target: desktopArea
                property: "opacity"
                to: 1.0
                duration: 150
            }
            NumberAnimation {
                target: desktopArea
                property: "scale"
                to: 1.0
                duration: 150
            }
        }
    }
    
    // Global keyboard shortcuts
    Shortcut {
        sequence: "Meta+Space"
        onActivated: universalLauncher.toggle()
    }
    
    Shortcut {
        sequence: "Meta+T"
        onActivated: nexusShell.launchApplication("konsole")
    }
    
    Shortcut {
        sequence: "Meta+E"
        onActivated: nexusShell.launchApplication("dolphin")
    }
    
    Shortcut {
        sequence: "Meta+A"
        onActivated: aiWidget.focus()
    }
    
    Shortcut {
        sequence: "Meta+S"
        onActivated: securityStatus.showSecurityCenter()
    }
    
    // Functions
    function showLaunchFeedback(appId) {
        // Show brief launch animation/feedback
        var component = Qt.createComponent("components/LaunchFeedback.qml")
        if (component.status === Component.Ready) {
            var feedback = component.createObject(desktopArea, {"appId": appId})
            feedback.show()
        }
    }
    
    function handleNotification(notificationId) {
        // Handle notification interactions
        systemMonitor.handleNotification(notificationId)
    }
}