import QtQuick 2.15
import QtQuick.Layouts 1.15
import QtQuick.Controls 2.15 as Controls
import org.kde.plasma.core 2.0 as PlasmaCore
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.components 3.0 as PlasmaComponents3
import org.kde.plasma.extras 2.0 as PlasmaExtras

Item {
    id: root
    
    Plasmoid.preferredRepresentation: Plasmoid.compactRepresentation
    Plasmoid.compactRepresentation: CompactRepresentation {}
    Plasmoid.fullRepresentation: FullRepresentation {}
    
    property var services: [
        {name: "Proxmox", icon: "computer", url: "https://192.168.122.9:8006", status: "unknown"},
        {name: "Jellyfin", icon: "media-playback-start", url: "http://192.168.122.9:8096", status: "unknown"},
        {name: "Grafana", icon: "office-chart-line", url: "http://192.168.122.9:3000", status: "unknown"},
        {name: "Pi-hole", icon: "security-high", url: "http://192.168.122.9:80/admin", status: "unknown"}
    ]
    
    Timer {
        id: statusTimer
        interval: 30000 // 30 seconds
        running: true
        repeat: true
        onTriggered: checkServices()
    }
    
    function checkServices() {
        // This would call your API to check service status
        // For now, simulate status updates
        for (var i = 0; i < services.length; i++) {
            services[i].status = Math.random() > 0.3 ? "online" : "offline"
        }
    }
    
    Component.onCompleted: {
        checkServices()
    }
    
    // Compact representation (what shows in the panel)
    Component {
        id: CompactRepresentation
        
        MouseArea {
            id: compactMouse
            
            Layout.minimumWidth: PlasmaCore.Units.iconSizes.small
            Layout.minimumHeight: PlasmaCore.Units.iconSizes.small
            Layout.preferredWidth: Layout.minimumWidth
            Layout.preferredHeight: Layout.minimumHeight
            
            property bool wasExpanded: false
            
            onPressed: wasExpanded = root.expanded
            onClicked: root.expanded = !wasExpanded
            
            PlasmaCore.IconItem {
                id: homeIcon
                anchors.fill: parent
                source: "network-server"
                colorGroup: PlasmaCore.ColorScope.colorGroup
                
                // Status indicator overlay
                Rectangle {
                    anchors.bottom: parent.bottom
                    anchors.right: parent.right
                    width: PlasmaCore.Units.smallSpacing
                    height: PlasmaCore.Units.smallSpacing
                    radius: width / 2
                    color: getOverallStatus() === "online" ? "#27ae60" : "#e74c3c"
                    border.color: PlasmaCore.Theme.backgroundColor
                    border.width: 1
                }
            }
            
            PlasmaCore.ToolTipArea {
                anchors.fill: parent
                mainText: i18n("Homelab Manager")
                subText: i18n("Manage Proxmox VMs and Media Stack")
            }
        }
    }
    
    // Full representation (popup/expanded view)
    Component {
        id: FullRepresentation
        
        PlasmaExtras.Representation {
            Layout.minimumWidth: PlasmaCore.Units.gridUnit * 20
            Layout.minimumHeight: PlasmaCore.Units.gridUnit * 25
            Layout.preferredWidth: Layout.minimumWidth
            Layout.preferredHeight: Layout.minimumHeight
            
            collapseMarginsHint: true
            
            header: PlasmaExtras.PlasmoidHeading {
                RowLayout {
                    anchors.fill: parent
                    
                    PlasmaCore.IconItem {
                        source: "network-server"
                        Layout.preferredWidth: PlasmaCore.Units.iconSizes.small
                        Layout.preferredHeight: PlasmaCore.Units.iconSizes.small
                    }
                    
                    Controls.Label {
                        text: i18n("Homelab Dashboard")
                        Layout.fillWidth: true
                        font.bold: true
                    }
                    
                    Controls.Button {
                        icon.name: "view-refresh"
                        onClicked: checkServices()
                        Controls.ToolTip.text: i18n("Refresh Status")
                        Controls.ToolTip.visible: hovered
                    }
                }
            }
            
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: PlasmaCore.Units.largeSpacing
                
                // Quick Stats
                PlasmaExtras.Heading {
                    text: i18n("Quick Stats")
                    level: 4
                }
                
                GridLayout {
                    columns: 2
                    Layout.fillWidth: true
                    
                    Controls.Label { text: i18n("CPU:") }
                    Controls.Label { 
                        text: "45%" // Would be dynamically updated
                        color: PlasmaCore.Theme.positiveTextColor
                    }
                    
                    Controls.Label { text: i18n("RAM:") }
                    Controls.Label { 
                        text: "68%" 
                        color: PlasmaCore.Theme.neutralTextColor
                    }
                    
                    Controls.Label { text: i18n("Storage:") }
                    Controls.Label { 
                        text: "34%" 
                        color: PlasmaCore.Theme.positiveTextColor
                    }
                    
                    Controls.Label { text: i18n("Temp:") }
                    Controls.Label { 
                        text: "52°C" 
                        color: PlasmaCore.Theme.positiveTextColor
                    }
                }
                
                PlasmaComponents3.Separator {
                    Layout.fillWidth: true
                }
                
                // Services
                PlasmaExtras.Heading {
                    text: i18n("Services")
                    level: 4
                }
                
                Repeater {
                    model: services
                    delegate: ServiceItem {
                        Layout.fillWidth: true
                        serviceName: modelData.name
                        serviceIcon: modelData.icon
                        serviceStatus: modelData.status
                        serviceUrl: modelData.url
                    }
                }
                
                PlasmaComponents3.Separator {
                    Layout.fillWidth: true
                }
                
                // Quick Actions
                PlasmaExtras.Heading {
                    text: i18n("Quick Actions")
                    level: 4
                }
                
                GridLayout {
                    columns: 2
                    Layout.fillWidth: true
                    
                    Controls.Button {
                        text: i18n("Open Toolbar")
                        icon.name: "view-fullscreen"
                        Layout.fillWidth: true
                        onClicked: openHomelabToolbar()
                    }
                    
                    Controls.Button {
                        text: i18n("SSH Proxmox")
                        icon.name: "utilities-terminal"
                        Layout.fillWidth: true
                        onClicked: openSSH()
                    }
                    
                    Controls.Button {
                        text: i18n("Quick Backup")
                        icon.name: "document-save"
                        Layout.fillWidth: true
                        onClicked: performBackup()
                    }
                    
                    Controls.Button {
                        text: i18n("Emergency Stop")
                        icon.name: "process-stop"
                        Layout.fillWidth: true
                        onClicked: emergencyStop()
                    }
                }
                
                Item {
                    Layout.fillHeight: true
                }
            }
        }
    }
    
    // Service Item Component
    Component {
        id: ServiceItem
        
        Rectangle {
            property string serviceName
            property string serviceIcon
            property string serviceStatus
            property string serviceUrl
            
            height: PlasmaCore.Units.gridUnit * 2.5
            color: mouseArea.containsMouse ? PlasmaCore.Theme.highlightColor : "transparent"
            radius: PlasmaCore.Units.smallSpacing
            
            MouseArea {
                id: mouseArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: Qt.openUrlExternally(serviceUrl)
            }
            
            RowLayout {
                anchors.fill: parent
                anchors.margins: PlasmaCore.Units.smallSpacing
                
                PlasmaCore.IconItem {
                    source: serviceIcon
                    Layout.preferredWidth: PlasmaCore.Units.iconSizes.medium
                    Layout.preferredHeight: PlasmaCore.Units.iconSizes.medium
                }
                
                ColumnLayout {
                    Layout.fillWidth: true
                    
                    Controls.Label {
                        text: serviceName
                        font.bold: true
                        Layout.fillWidth: true
                    }
                    
                    Controls.Label {
                        text: serviceUrl
                        opacity: 0.7
                        font.pointSize: PlasmaCore.Theme.smallestFont.pointSize
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                    }
                }
                
                Rectangle {
                    width: PlasmaCore.Units.largeSpacing
                    height: PlasmaCore.Units.largeSpacing
                    radius: width / 2
                    color: serviceStatus === "online" ? "#27ae60" : 
                           serviceStatus === "offline" ? "#e74c3c" : "#f39c12"
                    
                    SequentialAnimation on opacity {
                        running: serviceStatus === "unknown"
                        loops: Animation.Infinite
                        NumberAnimation { from: 1; to: 0.3; duration: 1000 }
                        NumberAnimation { from: 0.3; to: 1; duration: 1000 }
                    }
                }
            }
        }
    }
    
    function getOverallStatus() {
        var onlineCount = 0
        var totalCount = services.length
        
        for (var i = 0; i < services.length; i++) {
            if (services[i].status === "online") {
                onlineCount++
            }
        }
        
        return onlineCount > totalCount / 2 ? "online" : "offline"
    }
    
    function openHomelabToolbar() {
        PlasmaCore.ApplicationLauncher.open("firefox", ["file:///root/awesome-stack/garuda-homelab-toolbar/homelab-toolbar.html"])
    }
    
    function openSSH() {
        PlasmaCore.ApplicationLauncher.open("konsole", ["-e", "ssh root@192.168.122.9"])
    }
    
    function performBackup() {
        PlasmaCore.ApplicationLauncher.open("konsole", ["-e", "ssh root@192.168.122.9 'vzdump --all --compress gzip --storage local'"])
    }
    
    function emergencyStop() {
        // Show confirmation dialog first
        PlasmaCore.ApplicationLauncher.open("kdialog", ["--yesno", "Emergency stop all VMs?"])
    }
}
