import QtQuick 2.15
import QtQuick.Effects

// Logo 1: Stella AI Guardian - Golden retriever with AI neural network halo
Item {
    id: stellaGuardian
    
    property string title: "Stella AI Guardian"
    property string description: "Protective AI companion"
    property bool aiActive: true
    
    width: 200
    height: 200
    
    Rectangle {
        anchors.fill: parent
        color: "#1a1a1a"
        radius: 20
        border.color: aiActive ? "#00cc66" : "#555555"
        border.width: 2
        
        // Background glow when AI active
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 1.2
            height: parent.height * 1.2
            color: "transparent"
            border.color: "#00cc66"
            border.width: 1
            radius: 25
            opacity: aiActive ? 0.3 : 0
            
            Behavior on opacity { NumberAnimation { duration: 500 } }
        }
        
        // Main logo area
        Item {
            anchors.centerIn: parent
            width: 120
            height: 120
            
            // Stella's silhouette (golden retriever)
            Canvas {
                id: stellaSilhouette
                anchors.centerIn: parent
                width: 80
                height: 60
                
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    
                    // Golden retriever gradient
                    var gradient = ctx.createLinearGradient(0, 0, width, height)
                    gradient.addColorStop(0, "#FFD700")  // Golden
                    gradient.addColorStop(0.5, "#FFA500") // Orange
                    gradient.addColorStop(1, "#CD853F")   // Peru
                    
                    ctx.fillStyle = gradient
                    ctx.strokeStyle = aiActive ? "#00cc66" : "#ffffff"
                    ctx.lineWidth = 1.5
                    
                    // Draw Stella's profile
                    ctx.beginPath()
                    
                    // Head and ears
                    ctx.moveTo(20, 25)              // Start at nose
                    ctx.quadraticCurveTo(15, 20, 25, 15) // Nose to forehead
                    ctx.quadraticCurveTo(35, 10, 45, 18) // Forehead to ear
                    ctx.lineTo(42, 25)              // Ear tip
                    ctx.quadraticCurveTo(40, 30, 50, 28) // Back of head
                    
                    // Neck and body
                    ctx.quadraticCurveTo(55, 35, 50, 45) // Neck
                    ctx.quadraticCurveTo(65, 50, 60, 55) // Back
                    ctx.lineTo(45, 58)              // Tail area
                    ctx.quadraticCurveTo(35, 55, 30, 50) // Belly
                    ctx.quadraticCurveTo(25, 45, 28, 35) // Chest
                    ctx.lineTo(20, 25)              // Back to nose
                    
                    ctx.fill()
                    ctx.stroke()
                    
                    // Eye
                    ctx.fillStyle = "#000000"
                    ctx.beginPath()
                    ctx.arc(30, 23, 2, 0, Math.PI * 2)
                    ctx.fill()
                    
                    // Eye highlight
                    ctx.fillStyle = "#ffffff"
                    ctx.beginPath()
                    ctx.arc(31, 22, 0.8, 0, Math.PI * 2)
                    ctx.fill()
                }
                
                // Breathing animation
                SequentialAnimation on scale {
                    running: aiActive
                    loops: Animation.Infinite
                    NumberAnimation { to: 1.05; duration: 2000; easing.type: Easing.InOutQuad }
                    NumberAnimation { to: 1.0; duration: 2000; easing.type: Easing.InOutQuad }
                }
            }
            
            // AI Neural Network Halo
            Repeater {
                model: 12
                
                Rectangle {
                    property real angle: index * 30 * Math.PI / 180
                    property real radius: 55
                    
                    x: parent.width/2 + Math.cos(angle) * radius - width/2
                    y: parent.height/2 + Math.sin(angle) * radius - height/2
                    
                    width: 4
                    height: 4
                    radius: 2
                    color: aiActive ? "#00cc66" : "#666666"
                    
                    // Pulsing animation
                    SequentialAnimation on opacity {
                        running: aiActive
                        loops: Animation.Infinite
                        PauseAnimation { duration: index * 100 }
                        NumberAnimation { to: 0.3; duration: 300 }
                        NumberAnimation { to: 1.0; duration: 300 }
                        PauseAnimation { duration: (12 - index) * 100 }
                    }
                    
                    // Scale animation
                    SequentialAnimation on scale {
                        running: aiActive
                        loops: Animation.Infinite
                        PauseAnimation { duration: index * 100 }
                        NumberAnimation { to: 1.5; duration: 300 }
                        NumberAnimation { to: 1.0; duration: 300 }
                        PauseAnimation { duration: (12 - index) * 100 }
                    }
                }
            }
            
            // Neural connections between nodes
            Repeater {
                model: 12
                
                Canvas {
                    anchors.fill: parent
                    
                    property real angle1: index * 30 * Math.PI / 180
                    property real angle2: ((index + 1) % 12) * 30 * Math.PI / 180
                    property real radius: 55
                    
                    onPaint: {
                        var ctx = getContext("2d")
                        ctx.reset()
                        
                        if (!aiActive) return
                        
                        var x1 = width/2 + Math.cos(angle1) * radius
                        var y1 = height/2 + Math.sin(angle1) * radius
                        var x2 = width/2 + Math.cos(angle2) * radius
                        var y2 = height/2 + Math.sin(angle2) * radius
                        
                        ctx.strokeStyle = "#00cc66"
                        ctx.lineWidth = 0.5
                        ctx.globalAlpha = 0.3
                        
                        ctx.beginPath()
                        ctx.moveTo(x1, y1)
                        ctx.lineTo(x2, y2)
                        ctx.stroke()
                    }
                    
                    // Redraw when AI state changes
                    onPaint: {
                        requestPaint()
                    }
                }
            }
            
            // Central AI core glow
            Rectangle {
                anchors.centerIn: stellaSilhouette
                width: 100
                height: 80
                radius: 40
                color: "transparent"
                border.color: aiActive ? "#00cc66" : "transparent"
                border.width: 1
                opacity: aiActive ? 0.2 : 0
                
                // Pulsing glow
                SequentialAnimation on scale {
                    running: aiActive
                    loops: Animation.Infinite
                    NumberAnimation { to: 1.1; duration: 1500; easing.type: Easing.InOutQuad }
                    NumberAnimation { to: 1.0; duration: 1500; easing.type: Easing.InOutQuad }
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