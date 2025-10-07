import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Effects

// WireGuard Killswitch Widget - Network security control
Rectangle {
    id: killswitchWidget
    
    property bool active: false
    property bool connected: false
    property string connectionStatus: "Disconnected"
    property string serverLocation: "Unknown"
    property real latency: 0.0
    property bool networkBlocked: false
    
    signal toggleRequested()
    signal configurationRequested()
    signal statusRequested()
    
    width: 120
    height: 32
    radius: 16
    
    // Dynamic styling based on state
    color: {
        if (!active) return "#3a3a3a"           // Inactive: Dark gray
        if (networkBlocked) return "#f38ba8"    // Blocked: Red
        if (connected) return "#a6e3a1"         // Connected: Green  
        return "#f9e2af"                        // Connecting: Yellow
    }
    
    border.width: 2
    border.color: {
        if (!active) return "#555555"
        if (networkBlocked) return "#f38ba8"
        if (connected) return "#a6e3a1"
        return "#f9e2af"
    }
    
    // Pulsing animation when connecting or blocked
    SequentialAnimation on opacity {
        running: (active && !connected) || networkBlocked
        loops: Animation.Infinite
        NumberAnimation { to: 0.6; duration: 800; easing.type: Easing.InOutQuad }
        NumberAnimation { to: 1.0; duration: 800; easing.type: Easing.InOutQuad }
    }
    
    // Glassmorphism effect
    layer.enabled: true
    layer.effect: MultiEffect {
        blurEnabled: true
        blur: 0.2
        brightness: 0.1
        saturation: 1.1
    }
    
    RowLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6
        
        // WireGuard Icon
        Text {
            text: "🔒"
            font.pixelSize: 14
            color: "#ffffff"
            Layout.preferredWidth: 16
        }
        
        // Status Text
        Column {
            Layout.fillWidth: true
            spacing: 0
            
            Text {
                text: "WireGuard"
                font.family: "Inter"
                font.pixelSize: 9
                font.weight: Font.Medium
                color: "#ffffff"
                opacity: 0.9
            }
            
            Text {
                text: {
                    if (!active) return "Disabled"
                    if (networkBlocked) return "Blocked"
                    if (connected) return "Secured"
                    return "Connecting"
                }
                font.family: "Inter"
                font.pixelSize: 8
                color: "#ffffff"
                opacity: 0.7
            }
        }
        
        // Toggle indicator
        Rectangle {
            Layout.preferredWidth: 12
            Layout.preferredHeight: 12
            radius: 6
            
            color: {
                if (!active) return "#666666"
                if (networkBlocked) return "#ff4444"
                if (connected) return "#44ff44"
                return "#ffaa00"
            }
            
            border.width: 1
            border.color: "#ffffff"
            opacity: 0.9
            
            // Blinking animation for critical states
            SequentialAnimation on opacity {
                running: networkBlocked
                loops: Animation.Infinite
                NumberAnimation { to: 0.3; duration: 300 }
                NumberAnimation { to: 1.0; duration: 300 }
            }
        }
    }
    
    // Click area
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        
        onClicked: toggleRequested()
        onPressAndHold: configurationRequested()
        
        onEntered: {
            killswitchWidget.scale = 1.05
            tooltip.show()
        }
        onExited: {
            killswitchWidget.scale = 1.0
            tooltip.hide()
        }
        
        Behavior on scale { ScaleAnimator { duration: 150 } }
    }
    
    // Enhanced Tooltip
    Item {
        id: tooltip
        visible: false
        
        function show() {
            if (!visible) {
                visible = true
                showAnimation.start()
            }
        }
        
        function hide() {
            if (visible) {
                hideAnimation.start()
            }
        }
        
        Rectangle {
            id: tooltipBackground
            x: killswitchWidget.x + (killswitchWidget.width - width) / 2
            y: killswitchWidget.y - height - 10
            width: tooltipContent.width + 16
            height: tooltipContent.height + 12
            radius: 8
            color: "#2a2a2a"
            border.color: "#555555"
            border.width: 1
            opacity: 0
            
            // Glass effect
            layer.enabled: true
            layer.effect: MultiEffect {
                blurEnabled: true
                blur: 0.3
                brightness: 0.2
            }
            
            Column {
                id: tooltipContent
                anchors.centerIn: parent
                spacing: 2
                
                Text {
                    text: "WireGuard Killswitch"
                    font.family: "Inter"
                    font.weight: Font.Bold
                    font.pixelSize: 11
                    color: "#ffffff"
                }
                
                Text {
                    text: `Status: ${connectionStatus}`
                    font.family: "Inter"
                    font.pixelSize: 9
                    color: "#cccccc"
                }
                
                Text {
                    text: connected ? `Server: ${serverLocation}` : "No active connection"
                    font.family: "Inter"
                    font.pixelSize: 9
                    color: "#cccccc"
                    visible: active
                }
                
                Text {
                    text: connected ? `Latency: ${latency.toFixed(0)}ms` : ""
                    font.family: "Inter"
                    font.pixelSize: 9
                    color: "#cccccc"
                    visible: connected && latency > 0
                }
                
                Rectangle {
                    width: parent.width
                    height: 1
                    color: "#555555"
                    visible: active
                }
                
                Text {
                    text: active ? 
                        (networkBlocked ? 
                            "⚠️ Network traffic blocked" : 
                            (connected ? "✅ Traffic encrypted & secured" : "🔄 Establishing connection")
                        ) : "❌ Killswitch disabled"
                    font.family: "Inter"
                    font.pixelSize: 8
                    color: {
                        if (!active) return "#ff8888"
                        if (networkBlocked) return "#ffaa00"
                        if (connected) return "#88ff88"
                        return "#aaaaaa"
                    }
                    wrapMode: Text.WordWrap
                    width: 160
                }
                
                Text {
                    text: "Click: Toggle | Hold: Configure"
                    font.family: "Inter"
                    font.pixelSize: 8
                    color: "#888888"
                    opacity: 0.8
                }
            }
            
            // Tooltip arrow
            Canvas {
                id: arrow
                width: 10
                height: 5
                x: parent.width / 2 - width / 2
                y: parent.height
                
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    ctx.fillStyle = "#2a2a2a"
                    ctx.beginPath()
                    ctx.moveTo(0, 0)
                    ctx.lineTo(width, 0)
                    ctx.lineTo(width / 2, height)
                    ctx.closePath()
                    ctx.fill()
                }
            }
        }
        
        NumberAnimation {
            id: showAnimation
            target: tooltipBackground
            property: "opacity"
            to: 0.95
            duration: 200
            easing.type: Easing.OutQuad
        }
        
        SequentialAnimation {
            id: hideAnimation
            NumberAnimation {
                target: tooltipBackground
                property: "opacity"
                to: 0
                duration: 150
                easing.type: Easing.InQuad
            }
            ScriptAction {
                script: tooltip.visible = false
            }
        }
    }
    
    // Connection status monitoring
    Timer {
        id: statusTimer
        interval: 2000
        running: active
        repeat: true
        
        onTriggered: {
            // Update connection status
            updateConnectionStatus()
        }
    }
    
    // Network monitoring
    Timer {
        id: networkTimer
        interval: 5000
        running: active && connected
        repeat: true
        
        onTriggered: {
            // Check network latency
            measureLatency()
        }
    }
    
    // Functions
    function updateConnectionStatus() {
        // This would interface with actual WireGuard status
        // For now, simulate status updates
        if (active) {
            // Check WireGuard interface status
            checkWireGuardStatus()
        }
    }
    
    function measureLatency() {
        // Measure connection latency
        if (connected) {
            // This would ping the WireGuard server
            // Simulated for now
            latency = Math.random() * 100 + 20 // 20-120ms
        }
    }
    
    function checkWireGuardStatus() {
        // Interface with system WireGuard
        // This would run: wg show
        // For demonstration, simulate states
        
        if (active && Math.random() > 0.1) {
            connected = true
            connectionStatus = "Connected"
            serverLocation = "Netherlands"
            networkBlocked = false
        } else if (active) {
            connected = false
            connectionStatus = "Connecting..."
            networkBlocked = true  // Block traffic when not connected
        }
    }
    
    function toggleKillswitch() {
        active = !active
        
        if (active) {
            // Enable killswitch - block all traffic until VPN connects
            enableKillswitch()
        } else {
            // Disable killswitch - allow normal traffic
            disableKillswitch()
        }
        
        toggleRequested()
    }
    
    function enableKillswitch() {
        // This would configure iptables/firewall rules
        console.log("WireGuard killswitch enabled - blocking non-VPN traffic")
        
        // Example iptables rules (would be executed via system interface):
        // iptables -I OUTPUT ! -o wg0 -m owner --uid-owner $(id -u) -j DROP
        // iptables -I OUTPUT -d 192.168.1.0/24 -j ACCEPT  // Local network
        
        networkBlocked = true
        connectionStatus = "Killswitch Active"
    }
    
    function disableKillswitch() {
        // Remove firewall rules
        console.log("WireGuard killswitch disabled")
        
        networkBlocked = false
        connected = false
        connectionStatus = "Disabled"
    }
    
    // Component initialization
    Component.onCompleted: {
        if (active) {
            updateConnectionStatus()
        }
    }
}