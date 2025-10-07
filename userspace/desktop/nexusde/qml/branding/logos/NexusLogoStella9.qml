import QtQuick 2.15
import QtQuick.Effects

// Logo 9: Security Watchdog - Stella as system security guardian
Item {
    id: securityWatchdog
    
    property string title: "Security Watchdog"
    property string description: "Stella protecting your digital fortress"
    property bool aiActive: true
    property bool threatDetected: false
    
    width: 200
    height: 200
    
    Rectangle {
        anchors.fill: parent
        color: "#1a1a1a"
        radius: 20
        border.color: threatDetected ? "#f38ba8" : (aiActive ? "#a6e3a1" : "#555555")
        border.width: 2
        
        // Threat detection glow
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 1.3
            height: parent.height * 1.3
            color: "transparent"
            border.color: threatDetected ? "#f38ba8" : "#a6e3a1"
            border.width: 1
            radius: 30
            opacity: (aiActive && (threatDetected ? 0.6 : 0.2)) ? 1 : 0
            
            // Alert pulsing
            SequentialAnimation on opacity {
                running: threatDetected
                loops: Animation.Infinite
                NumberAnimation { to: 0.8; duration: 300 }
                NumberAnimation { to: 0.3; duration: 300 }
            }
            
            Behavior on opacity { NumberAnimation { duration: 500 } }
        }
        
        // Main logo area
        Item {
            anchors.centerIn: parent
            width: 150
            height: 130
            
            // Digital Fortress Shield Background
            Canvas {
                id: fortressShield
                anchors.centerIn: parent
                width: 120
                height: 100
                
                property real shieldPhase: 0
                
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    
                    if (!aiActive) return
                    
                    // Shield gradient
                    var gradient = ctx.createRadialGradient(60, 50, 10, 60, 50, 50)
                    gradient.addColorStop(0, threatDetected ? "#f38ba844" : "#a6e3a144")
                    gradient.addColorStop(1, threatDetected ? "#f38ba822" : "#a6e3a122")
                    
                    ctx.fillStyle = gradient
                    ctx.strokeStyle = threatDetected ? "#f38ba8" : "#a6e3a1"
                    ctx.lineWidth = 2
                    
                    // Draw hexagonal shield
                    ctx.beginPath()
                    var centerX = 60
                    var centerY = 50
                    var radius = 45
                    
                    for (var i = 0; i < 6; i++) {
                        var angle = (i * Math.PI) / 3 + shieldPhase
                        var x = centerX + radius * Math.cos(angle)
                        var y = centerY + radius * Math.sin(angle)
                        
                        if (i === 0) {
                            ctx.moveTo(x, y)
                        } else {
                            ctx.lineTo(x, y)
                        }
                    }
                    ctx.closePath()
                    ctx.fill()
                    ctx.stroke()
                    
                    // Digital grid pattern inside shield
                    ctx.strokeStyle = threatDetected ? "#f38ba850" : "#a6e3a150"
                    ctx.lineWidth = 0.5
                    
                    for (var j = 0; j < 8; j++) {
                        var lineY = 20 + j * 10
                        ctx.beginPath()
                        ctx.moveTo(25, lineY)
                        ctx.lineTo(95, lineY)
                        ctx.stroke()
                    }
                    
                    for (var k = 0; k < 8; k++) {
                        var lineX = 25 + k * 10
                        ctx.beginPath()
                        ctx.moveTo(lineX, 20)
                        ctx.lineTo(lineX, 80)
                        ctx.stroke()
                    }
                }
                
                // Shield rotation animation
                NumberAnimation on shieldPhase {
                    running: aiActive
                    from: 0
                    to: Math.PI / 3
                    duration: 8000
                    loops: Animation.Infinite
                    onRunningChanged: fortressShield.requestPaint()
                }
                
                onShieldPhaseChanged: requestPaint()
            }
            
            // Stella as Security Guardian
            Canvas {
                id: stellaGuardian
                anchors.centerIn: parent
                width: 90
                height: 65
                
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    
                    // Stella gradient (alert colors when threat detected)
                    var gradient = ctx.createLinearGradient(0, 0, width, height)
                    if (threatDetected) {
                        gradient.addColorStop(0, "#FFB366")  // Alert golden
                        gradient.addColorStop(0.5, "#FF8C42") // Alert orange
                        gradient.addColorStop(1, "#D2691E")   // Alert brown
                    } else {
                        gradient.addColorStop(0, "#FFD700")  // Golden
                        gradient.addColorStop(0.5, "#FFA500") // Orange
                        gradient.addColorStop(1, "#CD853F")   // Peru
                    }
                    
                    ctx.fillStyle = gradient
                    ctx.strokeStyle = threatDetected ? "#f38ba8" : "#a6e3a1"
                    ctx.lineWidth = 1.5
                    
                    // Draw Stella in alert/guard pose
                    ctx.beginPath()
                    
                    // Head (alert, ears up)
                    ctx.moveTo(25, 30)              // Nose
                    ctx.quadraticCurveTo(20, 25, 30, 20) // Forehead
                    ctx.quadraticCurveTo(40, 15, 50, 22) // Perked ear
                    ctx.lineTo(47, 28)              // Ear tip
                    ctx.quadraticCurveTo(45, 35, 55, 33) // Back of head
                    
                    // Body (sitting alert)
                    ctx.quadraticCurveTo(60, 40, 55, 50) // Neck to back
                    ctx.quadraticCurveTo(50, 55, 40, 52) // Back to tail
                    ctx.quadraticCurveTo(30, 50, 25, 45) // Belly
                    ctx.quadraticCurveTo(20, 40, 25, 30) // Chest to nose
                    
                    ctx.fill()
                    ctx.stroke()
                    
                    // Alert/focused eye
                    ctx.fillStyle = "#000000"
                    ctx.beginPath()
                    ctx.arc(35, 28, 2.5, 0, Math.PI * 2)
                    ctx.fill()
                    
                    // Eye highlight (intense when threat detected)
                    ctx.fillStyle = threatDetected ? "#ff4444" : "#ffffff"
                    ctx.beginPath()
                    ctx.arc(36, 27, 1, 0, Math.PI * 2)
                    ctx.fill()
                }
                
                // Alert breathing when threat detected
                SequentialAnimation on scale {
                    running: threatDetected
                    loops: Animation.Infinite
                    NumberAnimation { to: 1.1; duration: 600; easing.type: Easing.InOutQuad }
                    NumberAnimation { to: 1.0; duration: 600; easing.type: Easing.InOutQuad }
                }
                
                // Gentle breathing when secure
                SequentialAnimation on scale {
                    running: aiActive && !threatDetected
                    loops: Animation.Infinite
                    NumberAnimation { to: 1.03; duration: 3000; easing.type: Easing.InOutQuad }
                    NumberAnimation { to: 1.0; duration: 3000; easing.type: Easing.InOutQuad }
                }
            }
            
            // Security scan beams from Stella's eyes
            Repeater {
                model: 3
                
                Rectangle {
                    id: scanBeam
                    x: 80 + index * 15
                    y: 63
                    width: 2
                    height: 20 + index * 10
                    color: threatDetected ? "#f38ba8" : "#a6e3a1"
                    opacity: 0
                    
                    // Scanning animation
                    SequentialAnimation on opacity {
                        running: aiActive
                        loops: Animation.Infinite
                        PauseAnimation { duration: index * 200 }
                        NumberAnimation { to: 0.8; duration: 150 }
                        PauseAnimation { duration: 100 }
                        NumberAnimation { to: 0; duration: 150 }
                        PauseAnimation { duration: (3 - index) * 200 + 1000 }
                    }
                    
                    // Sweep animation
                    SequentialAnimation on rotation {
                        running: aiActive
                        loops: Animation.Infinite
                        NumberAnimation { from: -15; to: 15; duration: 2000 + index * 300 }
                        NumberAnimation { from: 15; to: -15; duration: 2000 + index * 300 }
                    }
                }
            }
            
            // Security status indicators around the shield
            Repeater {
                model: [
                    {name: "VPN", icon: "🔒", angle: 0, color: "#4CAF50"},
                    {name: "FW", icon: "🛡️", angle: 60, color: "#2196F3"},
                    {name: "AV", icon: "🦠", angle: 120, color: "#FF9800"},
                    {name: "LOG", icon: "📊", angle: 180, color: "#9C27B0"},
                    {name: "NET", icon: "🌐", angle: 240, color: "#607D8B"},
                    {name: "KEY", icon: "🔐", angle: 300, color: "#795548"}
                ]
                
                Rectangle {
                    property real angle: modelData.angle * Math.PI / 180
                    property real radius: 70
                    
                    x: parent.width/2 + Math.cos(angle) * radius - width/2
                    y: parent.height/2 + Math.sin(angle) * radius - height/2
                    
                    width: 18
                    height: 18
                    radius: 9
                    color: modelData.color
                    border.color: "#ffffff"
                    border.width: 1
                    
                    Text {
                        anchors.centerIn: parent
                        text: modelData.icon
                        font.pixelSize: 8
                    }
                    
                    // Status indicator pulsing
                    SequentialAnimation on opacity {
                        running: aiActive
                        loops: Animation.Infinite
                        PauseAnimation { duration: index * 500 }
                        NumberAnimation { to: 0.5; duration: 200 }
                        NumberAnimation { to: 1.0; duration: 200 }
                        PauseAnimation { duration: (6 - index) * 500 }
                    }
                    
                    // Threat alert
                    SequentialAnimation on scale {
                        running: threatDetected
                        loops: Animation.Infinite
                        NumberAnimation { to: 1.3; duration: 300 }
                        NumberAnimation { to: 1.0; duration: 300 }
                    }
                }
            }
            
            // Threat detection alert
            Rectangle {
                anchors.centerIn: parent
                anchors.verticalCenterOffset: -50
                width: 80
                height: 20
                radius: 10
                color: "#f38ba8"
                opacity: threatDetected ? 0.9 : 0
                
                Text {
                    anchors.centerIn: parent
                    text: "⚠️ THREAT DETECTED"
                    color: "#ffffff"
                    font.family: "Inter"
                    font.weight: Font.Bold
                    font.pixelSize: 8
                }
                
                // Alert flashing
                SequentialAnimation on opacity {
                    running: threatDetected
                    loops: Animation.Infinite
                    NumberAnimation { to: 1.0; duration: 400 }
                    NumberAnimation { to: 0.4; duration: 400 }
                }
                
                Behavior on opacity { NumberAnimation { duration: 300 } }
            }
            
            // Secure status
            Rectangle {
                anchors.centerIn: parent
                anchors.verticalCenterOffset: -50
                width: 60
                height: 18
                radius: 9
                color: "#a6e3a1"
                opacity: (aiActive && !threatDetected) ? 0.8 : 0
                
                Text {
                    anchors.centerIn: parent
                    text: "✅ SECURE"
                    color: "#1a1a1a"
                    font.family: "Inter"
                    font.weight: Font.Bold
                    font.pixelSize: 8
                }
                
                Behavior on opacity { NumberAnimation { duration: 500 } }
            }
        }
        
        // Title and description
        Column {
            anchors.bottom: parent.bottom
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottomMargin: 10
            
            Text {
                text: title
                font.family: "Inter"
                font.weight: Font.Bold
                font.pixelSize: 12
                color: "#ffffff"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            
            Text {
                text: description
                font.family: "Inter"
                font.pixelSize: 9
                color: "#cccccc"
                opacity: 0.8
                anchors.horizontalCenter: parent.horizontalCenter
                wrapMode: Text.WordWrap
                width: parent.parent.width - 20
                horizontalAlignment: Text.AlignHCenter
            }
        }
        
        // Interactive hover effect
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            
            onClicked: {
                // Simulate threat detection toggle for demo
                threatDetected = !threatDetected
            }
            
            onEntered: {
                parent.scale = 1.05
                hoverGlow.opacity = 0.5
            }
            onExited: {
                parent.scale = 1.0
                hoverGlow.opacity = 0
            }
            
            Rectangle {
                id: hoverGlow
                anchors.fill: parent
                color: "transparent"
                border.color: "#FFD700"
                border.width: 2
                radius: 20
                opacity: 0
                
                Behavior on opacity { NumberAnimation { duration: 200 } }
            }
            
            Behavior on scale { ScaleAnimator { duration: 200 } }
        }
    }
    
    // Simulate threat detection for demo
    Timer {
        interval: 8000
        running: aiActive
        repeat: true
        onTriggered: {
            threatDetected = Math.random() > 0.7  // 30% chance of threat
        }
    }
}