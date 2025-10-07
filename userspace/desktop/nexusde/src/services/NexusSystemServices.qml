/**
 * NexusDE System Services Manager
 * Unified interface for KVM management, backup/restore, biometric auth, and system monitoring
 * Integrates with Stella & Max Jr. for intelligent system orchestration
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtGraphicalEffects 1.15
import NexusDE.Theme 1.0

ApplicationWindow {
    id: systemServices
    title: "NexusDE System Services"
    width: 1400
    height: 900
    minimumWidth: 1200
    minimumHeight: 700
    
    color: NexusTheme.colors.background
    
    // System state monitoring
    property bool kvmManagerActive: false
    property bool backupSystemActive: false
    property bool biometricSystemActive: false
    property bool stellaMonitoring: true
    property bool maxJrMonitoring: true
    
    // Service health status
    property var serviceStates: ({
        "kvm": { "status": "running", "vms": 3, "load": 0.4 },
        "backup": { "status": "idle", "lastBackup": "2h ago", "nextBackup": "22h" },
        "biometric": { "status": "ready", "enrolled": 2, "lastAuth": "5m ago" },
        "security": { "status": "active", "threats": 0, "scans": 1247 }
    })
    
    header: Rectangle {
        height: 80
        color: NexusTheme.colors.panelBackground
        border.color: NexusTheme.colors.panelBorder
        border.width: 1
        
        RowLayout {
            anchors.fill: parent
            anchors.margins: NexusTheme.spacing.md
            
            // Mascot indicators
            Row {
                spacing: NexusTheme.spacing.sm
                
                // Stella - Security & Backup Guardian
                Rectangle {
                    width: 64
                    height: 64
                    radius: 32
                    color: stellaMonitoring ? NexusTheme.brandColors.stellaBlue : NexusTheme.colors.surfaceVariant
                    border.color: stellaMonitoring ? NexusTheme.brandColors.stellaGlow : "transparent"
                    border.width: stellaMonitoring ? 2 : 0
                    
                    Image {
                        anchors.centerIn: parent
                        width: 48
                        height: 48
                        source: NexusTheme.icons.stellaIcon
                        fillMode: Image.PreserveAspectFit
                    }
                    
                    GlowEffect {
                        anchors.fill: parent
                        glowColor: NexusTheme.brandColors.stellaGlow
                        intensity: stellaMonitoring ? 0.8 : 0.3
                        active: stellaMonitoring
                    }
                    
                    Text {
                        anchors.top: parent.bottom
                        anchors.topMargin: 4
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: "Stella"
                        font.family: NexusTheme.typography.primaryFont
                        font.pixelSize: 12
                        font.weight: Font.Medium
                        color: stellaMonitoring ? NexusTheme.brandColors.stellaBlue : NexusTheme.colors.onSurfaceVariant
                    }
                    
                    MouseArea {
                        anchors.fill: parent
                        onClicked: stellaMonitoring = !stellaMonitoring
                        hoverEnabled: true
                        onEntered: parent.scale = 1.05
                        onExited: parent.scale = 1.0
                        
                        Behavior on scale {
                            NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                        }
                    }
                }
                
                // Max Jr. - System & VM Monitor
                Rectangle {
                    width: 64
                    height: 64
                    radius: 32
                    color: maxJrMonitoring ? NexusTheme.brandColors.maxJrOrange : NexusTheme.colors.surfaceVariant
                    border.color: maxJrMonitoring ? NexusTheme.brandColors.maxJrGlow : "transparent"
                    border.width: maxJrMonitoring ? 2 : 0
                    
                    Image {
                        anchors.centerIn: parent
                        width: 48
                        height: 48
                        source: NexusTheme.icons.maxJrIcon
                        fillMode: Image.PreserveAspectFit
                    }
                    
                    GlowEffect {
                        anchors.fill: parent
                        glowColor: NexusTheme.brandColors.maxJrGlow
                        intensity: maxJrMonitoring ? 0.8 : 0.3
                        active: maxJrMonitoring
                    }
                    
                    Text {
                        anchors.top: parent.bottom
                        anchors.topMargin: 4
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: "Max Jr."
                        font.family: NexusTheme.typography.primaryFont
                        font.pixelSize: 12
                        font.weight: Font.Medium
                        color: maxJrMonitoring ? NexusTheme.brandColors.maxJrOrange : NexusTheme.colors.onSurfaceVariant
                    }
                    
                    MouseArea {
                        anchors.fill: parent
                        onClicked: maxJrMonitoring = !maxJrMonitoring
                        hoverEnabled: true
                        onEntered: parent.scale = 1.05
                        onExited: parent.scale = 1.0
                        
                        Behavior on scale {
                            NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                        }
                    }
                }
            }
            
            Item { Layout.fillWidth: true }
            
            // System status overview
            Row {
                spacing: NexusTheme.spacing.lg
                
                // KVM Status
                StatusIndicator {
                    service: "Virtualization"
                    status: serviceStates.kvm.status
                    details: serviceStates.kvm.vms + " VMs running"
                    color: NexusTheme.colors.info
                    active: kvmManagerActive
                }
                
                // Backup Status
                StatusIndicator {
                    service: "Backup System"
                    status: serviceStates.backup.status
                    details: "Last: " + serviceStates.backup.lastBackup
                    color: NexusTheme.colors.success
                    active: backupSystemActive
                }
                
                // Biometric Status
                StatusIndicator {
                    service: "Biometric Auth"
                    status: serviceStates.biometric.status
                    details: serviceStates.biometric.enrolled + " devices enrolled"
                    color: NexusTheme.colors.secondary
                    active: biometricSystemActive
                }
                
                // Security Status
                StatusIndicator {
                    service: "Security"
                    status: serviceStates.security.status
                    details: serviceStates.security.threats + " threats detected"
                    color: serviceStates.security.threats > 0 ? NexusTheme.colors.error : NexusTheme.colors.success
                    active: true
                }
            }
        }
    }
    
    // Main content area
    RowLayout {
        anchors.fill: parent
        anchors.margins: NexusTheme.spacing.md
        spacing: NexusTheme.spacing.md
        
        // Service navigation sidebar
        Rectangle {
            Layout.preferredWidth: 280
            Layout.fillHeight: true
            color: NexusTheme.colors.surface
            radius: NexusTheme.radius.card
            border.color: NexusTheme.colors.outline
            border.width: 1
            
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: NexusTheme.spacing.md
                spacing: NexusTheme.spacing.sm
                
                Text {
                    text: "System Services"
                    font.family: NexusTheme.typography.primaryFont
                    font.pixelSize: NexusTheme.typography.titleLarge
                    font.weight: Font.SemiBold
                    color: NexusTheme.colors.onSurface
                    Layout.bottomMargin: NexusTheme.spacing.sm
                }
                
                ServiceNavigationItem {
                    text: "KVM Manager"
                    icon: "virtualization"
                    description: "Manage virtual machines"
                    active: kvmManagerActive
                    mascot: "maxjr"
                    onClicked: {
                        kvmManagerActive = true
                        backupSystemActive = false
                        biometricSystemActive = false
                        loadKVMManager()
                    }
                }
                
                ServiceNavigationItem {
                    text: "Backup & Restore"
                    icon: "backup"
                    description: "System backup management"
                    active: backupSystemActive
                    mascot: "stella"
                    onClicked: {
                        kvmManagerActive = false
                        backupSystemActive = true
                        biometricSystemActive = false
                        loadBackupSystem()
                    }
                }
                
                ServiceNavigationItem {
                    text: "Biometric Security"
                    icon: "fingerprint"
                    description: "Garuda Hello authentication"
                    active: biometricSystemActive
                    mascot: "stella"
                    onClicked: {
                        kvmManagerActive = false
                        backupSystemActive = false
                        biometricSystemActive = true
                        loadBiometricSystem()
                    }
                }
                
                Rectangle {
                    Layout.fillWidth: true
                    height: 1
                    color: NexusTheme.colors.outline
                    Layout.topMargin: NexusTheme.spacing.sm
                    Layout.bottomMargin: NexusTheme.spacing.sm
                }
                
                ServiceNavigationItem {
                    text: "System Dashboard"
                    icon: "dashboard"
                    description: "Overall system health"
                    active: false
                    onClicked: loadSystemDashboard()
                }
                
                ServiceNavigationItem {
                    text: "AI Optimization"
                    icon: "ai"
                    description: "Stella & Max Jr. insights"
                    active: false
                    onClicked: loadAIOptimization()
                }
                
                Item { Layout.fillHeight: true }
                
                // Mascot status panel
                Rectangle {
                    Layout.fillWidth: true
                    height: 120
                    color: NexusTheme.colors.surfaceVariant
                    radius: NexusTheme.radius.sm
                    
                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: NexusTheme.spacing.sm
                        
                        Text {
                            text: "AI Assistants"
                            font.family: NexusTheme.typography.primaryFont
                            font.pixelSize: NexusTheme.typography.labelLarge
                            font.weight: Font.Medium
                            color: NexusTheme.colors.onSurfaceVariant
                        }
                        
                        Row {
                            spacing: NexusTheme.spacing.sm
                            
                            Column {
                                Text {
                                    text: "🛡️ Stella"
                                    font.pixelSize: 14
                                    color: stellaMonitoring ? NexusTheme.brandColors.stellaBlue : NexusTheme.colors.onSurfaceVariant
                                }
                                Text {
                                    text: stellaMonitoring ? "Monitoring Security" : "Offline"
                                    font.pixelSize: 10
                                    color: NexusTheme.colors.onSurfaceVariant
                                }
                            }
                            
                            Column {
                                Text {
                                    text: "⚡ Max Jr."
                                    font.pixelSize: 14
                                    color: maxJrMonitoring ? NexusTheme.brandColors.maxJrOrange : NexusTheme.colors.onSurfaceVariant
                                }
                                Text {
                                    text: maxJrMonitoring ? "Optimizing System" : "Offline"
                                    font.pixelSize: 10
                                    color: NexusTheme.colors.onSurfaceVariant
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Main content area
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: NexusTheme.colors.surface
            radius: NexusTheme.radius.card
            border.color: NexusTheme.colors.outline
            border.width: 1
            
            StackLayout {
                id: contentStack
                anchors.fill: parent
                anchors.margins: NexusTheme.spacing.lg
                currentIndex: getCurrentServiceIndex()
                
                // Welcome/Dashboard view
                WelcomeDashboard {
                    stellaActive: stellaMonitoring
                    maxJrActive: maxJrMonitoring
                }
                
                // KVM Manager integration
                KVMManagerView {
                    visible: kvmManagerActive
                    stellaActive: stellaMonitoring
                    maxJrActive: maxJrMonitoring
                }
                
                // Backup System integration
                BackupSystemView {
                    visible: backupSystemActive
                    stellaActive: stellaMonitoring
                }
                
                // Biometric System integration
                BiometricSystemView {
                    visible: biometricSystemActive
                    stellaActive: stellaMonitoring
                }
            }
        }
    }
    
    // Functions
    function getCurrentServiceIndex() {
        if (kvmManagerActive) return 1
        if (backupSystemActive) return 2
        if (biometricSystemActive) return 3
        return 0
    }
    
    function loadKVMManager() {
        console.log("Loading KVM Manager integration...")
        // Launch KVM Manager or load embedded view
    }
    
    function loadBackupSystem() {
        console.log("Loading Garuda Ultimate Restore System...")
        // Launch backup system interface
    }
    
    function loadBiometricSystem() {
        console.log("Loading Garuda Hello biometric system...")
        // Launch biometric configuration
    }
    
    function loadSystemDashboard() {
        console.log("Loading system dashboard...")
        kvmManagerActive = false
        backupSystemActive = false
        biometricSystemActive = false
    }
    
    function loadAIOptimization() {
        console.log("Loading AI optimization interface...")
        // Show Stella & Max Jr. recommendations
    }
    
    // Mascot integration updates
    Timer {
        interval: 5000
        running: true
        repeat: true
        onTriggered: {
            // Update service states
            serviceStates.kvm.load = Math.random() * 0.8
            serviceStates.backup.lastBackup = Math.floor(Math.random() * 24) + "h ago"
            
            // Trigger mascot reactions based on system state
            if (serviceStates.kvm.load > 0.7 && maxJrMonitoring) {
                console.log("Max Jr. detected high VM load - suggesting optimization")
            }
            
            if (serviceStates.security.threats > 0 && stellaMonitoring) {
                console.log("Stella detected security threat - taking action")
            }
        }
    }
}