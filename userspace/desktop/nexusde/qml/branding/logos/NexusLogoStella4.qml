import QtQuick 2.15
import QtQuick.Effects

// Logo 4: Universal Retriever - Stella fetching from universal packages
Item {
    id: universalRetriever
    
    property string title: "Universal Retriever"
    property string description: "Stella fetching from all Linux distros"
    property bool aiActive: true
    
    width: 200
    height: 200
    
    Rectangle {
        anchors.fill: parent
        color: "#1a1a1a"
        radius: 20
        border.color: aiActive ? "#0066cc" : "#555555"
        border.width: 2
        
        // Main logo area
        Item {
            anchors.centerIn: parent
            width: 160
            height: 140
            
            // Stella running with packages
            Canvas {
                id: stellaRunning
                anchors.centerIn: parent
                width: 100
                height: 70
                
                property real runPhase: 0
                
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    
                    // Golden retriever gradient
                    var gradient = ctx.createLinearGradient(0, 0, width, height)
                    gradient.addColorStop(0, "#FFD700")  // Golden
                    gradient.addColorStop(0.5, "#FFA500") // Orange
                    gradient.addColorStop(1, "#CD853F")   // Peru
                    
                    ctx.fillStyle = gradient
                    ctx.strokeStyle = "#ffffff"
                    ctx.lineWidth = 1.5
                    
                    // Dynamic running pose
                    var bounce = Math.sin(runPhase) * 3
                    var stretch = 1 + Math.sin(runPhase * 2) * 0.1
                    
                    ctx.save()
                    ctx.translate(50, 35 + bounce)
                    ctx.scale(stretch, 1)
                    
                    // Draw Stella in running pose
                    ctx.beginPath()
                    
                    // Head (tilted forward while running)
                    ctx.moveTo(-15, -10)              // Nose
                    ctx.quadraticCurveTo(-20, -15, -10, -18) // Forehead
                    ctx.quadraticCurveTo(0, -20, 10, -15) // Top of head
                    ctx.quadraticCurveTo(15, -10, 12, -5) // Ear
                    ctx.quadraticCurveTo(8, 0, 15, 5)   // Back of head to neck
                    
                    // Body (elongated while running)
                    ctx.quadraticCurveTo(25, 10, 30, 15) // Back
                    ctx.quadraticCurveTo(35, 18, 25, 20) // Tail up
                    ctx.quadraticCurveTo(15, 18, 5, 15)  // Belly
                    ctx.quadraticCurveTo(-10, 10, -15, 5) // Chest
                    ctx.lineTo(-15, -10)              // Back to nose
                    
                    ctx.fill()
                    ctx.stroke()
                    
                    // Eye (focused and determined)
                    ctx.fillStyle = "#000000"
                    ctx.beginPath()
                    ctx.arc(-8, -8, 1.5, 0, Math.PI * 2)
                    ctx.fill()
                    
                    // Eye highlight
                    ctx.fillStyle = "#ffffff"
                    ctx.beginPath()
                    ctx.arc(-7, -9, 0.6, 0, Math.PI * 2)
                    ctx.fill()
                    
                    ctx.restore()
                }
                
                // Running animation
                NumberAnimation on runPhase {
                    running: aiActive
                    from: 0
                    to: Math.PI * 2
                    duration: 1000
                    loops: Animation.Infinite
                    onRunningChanged: stellaRunning.requestPaint()
                }
                
                onRunPhaseChanged: requestPaint()
            }
            
            // Package icons floating around Stella (different Linux distros)
            Repeater {
                model: [
                    {name: "DEB", color: "#A80030", icon: "📦", x: 20, y: 20},
                    {name: "RPM", color: "#EE0000", icon: "🎯", x: 120, y: 30},
                    {name: "ARCH", color: "#1793D1", icon: "🏛️", x: 140, y: 80},
                    {name: "FLAT", color: "#4A90E2", icon: "📱", x: 30, y: 100},
                    {name: "SNAP", color: "#E95420", icon: "🫰", x: 100, y: 110},
                    {name: "APP", color: "#4CAF50", icon: "🚀", x: 10, y: 60}
                ]
                
                Rectangle {
                    id: packageIcon
                    x: modelData.x + Math.sin(index * 0.5 + stellaRunning.runPhase) * 10
                    y: modelData.y + Math.cos(index * 0.3 + stellaRunning.runPhase) * 8
                    width: 25
                    height: 25
                    radius: 12
                    color: modelData.color
                    border.color: "#ffffff"
                    border.width: 1
                    
                    // Package icon
                    Text {
                        anchors.centerIn: parent
                        text: modelData.icon
                        font.pixelSize: 12
                    }
                    
                    // Package name
                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.top: parent.bottom
                        anchors.topMargin: 2
                        text: modelData.name
                        font.family: "Inter"
                        font.pixelSize: 6
                        color: "#ffffff"
                        opacity: 0.8
                    }
                    
                    // Floating animation
                    SequentialAnimation on y {
                        running: aiActive
                        loops: Animation.Infinite
                        NumberAnimation { 
                            to: packageIcon.y - 5
                            duration: 1500 + index * 200
                            easing.type: Easing.InOutQuad 
                        }
                        NumberAnimation { 
                            to: packageIcon.y + 5
                            duration: 1500 + index * 200
                            easing.type: Easing.InOutQuad 
                        }
                    }
                    
                    // Pulsing when being "fetched"
                    SequentialAnimation on scale {
                        running: aiActive
                        loops: Animation.Infinite
                        PauseAnimation { duration: index * 800 }
                        NumberAnimation { to: 1.3; duration: 200 }
                        NumberAnimation { to: 1.0; duration: 200 }
                        PauseAnimation { duration: (6 - index) * 800 }
                    }
                }
            }
            
            // Data streams connecting packages to Stella
            Repeater {
                model: 6
                
                Canvas {
                    anchors.fill: parent
                    
                    property real streamPhase: 0
                    
                    onPaint: {
                        var ctx = getContext("2d")
                        ctx.reset()
                        
                        if (!aiActive) return
                        
                        var packages = [
                            {x: 20, y: 20}, {x: 120, y: 30}, {x: 140, y: 80},
                            {x: 30, y: 100}, {x: 100, y: 110}, {x: 10, y: 60}
                        ]
                        
                        var stellaX = 80
                        var stellaY = 70
                        var pkg = packages[index]
                        
                        // Animated data stream
                        ctx.strokeStyle = "#00cc66"
                        ctx.lineWidth = 2
                        ctx.setLineDash([5, 5])
                        ctx.lineDashOffset = streamPhase
                        
                        ctx.beginPath()
                        ctx.moveTo(pkg.x + 12, pkg.y + 12)
                        ctx.quadraticCurveTo(
                            (pkg.x + stellaX) / 2,
                            Math.min(pkg.y, stellaY) - 20,
                            stellaX,
                            stellaY
                        )
                        ctx.stroke()
                    }
                    
                    // Stream animation
                    NumberAnimation on streamPhase {
                        running: aiActive
                        from: 0
                        to: 20
                        duration: 1000 + index * 100
                        loops: Animation.Infinite
                        onRunningChanged: parent.requestPaint()
                    }
                    
                    onStreamPhaseChanged: requestPaint()
                }
            }
            
            // Success indicator when packages are "retrieved"
            Rectangle {
                anchors.centerIn: stellaRunning
                anchors.verticalCenterOffset: -20
                width: 30
                height: 15
                radius: 7
                color: "#00cc66"
                opacity: 0
                
                Text {
                    anchors.centerIn: parent
                    text: "✓"
                    color: "#ffffff"
                    font.bold: true
                    font.pixelSize: 10
                }
                
                // Success animation
                SequentialAnimation on opacity {
                    running: aiActive
                    loops: Animation.Infinite
                    PauseAnimation { duration: 3000 }
                    NumberAnimation { to: 1.0; duration: 300 }
                    PauseAnimation { duration: 500 }
                    NumberAnimation { to: 0; duration: 300 }
                    PauseAnimation { duration: 2000 }
                }
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
}