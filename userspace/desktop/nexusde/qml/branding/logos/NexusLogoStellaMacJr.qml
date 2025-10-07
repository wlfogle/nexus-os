import QtQuick 2.15
import QtQuick.Effects

// NexusOS Duo Mascot - Stella & Max Jr. as AI Companions
Item {
    id: duoMascot
    
    property string title: "Stella & Max Jr."
    property string description: "AI Companions guarding NexusOS"
    property bool aiActive: true
    property bool stellaActive: true  // Stella handles packages/security
    property bool maxActive: true    // Max Jr. handles system monitoring/optimization
    
    width: 200
    height: 200
    
    Rectangle {
        anchors.fill: parent
        color: "#1a1a1a"
        radius: 20
        border.color: aiActive ? "#00cc66" : "#555555"
        border.width: 2
        
        // Duo harmony aura
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 1.2
            height: parent.height * 1.2
            color: "transparent"
            border.color: "#FFD700"
            border.width: 1
            radius: 25
            opacity: (stellaActive && maxActive) ? 0.4 : 0.2
            
            // Harmony pulse
            SequentialAnimation on opacity {
                running: aiActive && stellaActive && maxActive
                loops: Animation.Infinite
                NumberAnimation { to: 0.6; duration: 2000; easing.type: Easing.InOutQuad }
                NumberAnimation { to: 0.2; duration: 2000; easing.type: Easing.InOutQuad }
            }
            
            Behavior on opacity { NumberAnimation { duration: 500 } }
        }
        
        // Main mascot area
        Item {
            anchors.centerIn: parent
            width: 160
            height: 140
            
            // Stella (Golden Retriever) - Left side, handling security/packages
            Canvas {
                id: stellaCanvas
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                anchors.verticalCenterOffset: 10
                width: 70
                height: 55
                
                property real tailWag: 0
                
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    
                    // Stella's beautiful golden coat gradient
                    var gradient = ctx.createLinearGradient(0, 0, width, height)
                    gradient.addColorStop(0, "#FFD700")   // Golden
                    gradient.addColorStop(0.3, "#FFA500") // Rich golden orange
                    gradient.addColorStop(0.7, "#CD853F") // Darker golden brown
                    gradient.addColorStop(1, "#DEB887")   // Light golden
                    
                    ctx.fillStyle = gradient
                    ctx.strokeStyle = stellaActive ? "#00cc66" : "#ffffff"
                    ctx.lineWidth = 1.2
                    
                    // Stella's profile (relaxed, sitting pose)
                    ctx.beginPath()
                    
                    // Head and distinctive golden retriever features
                    ctx.moveTo(15, 30)                    // Nose
                    ctx.quadraticCurveTo(10, 25, 20, 20)  // Gentle forehead
                    ctx.quadraticCurveTo(30, 15, 40, 20)  // Top of head
                    ctx.quadraticCurveTo(45, 25, 42, 32)  // Soft ear
                    ctx.quadraticCurveTo(40, 35, 45, 38)  // Back of head
                    
                    // Body (sitting peacefully)
                    ctx.quadraticCurveTo(50, 42, 45, 48)  // Neck to back
                    ctx.quadraticCurveTo(40, 52, 35, 50)  // Back
                    
                    // Tail (wagging based on animation)
                    var tailX = 35 + Math.sin(tailWag) * 8
                    var tailY = 48 + Math.cos(tailWag) * 4
                    ctx.quadraticCurveTo(tailX, tailY, 30, 48)
                    
                    ctx.quadraticCurveTo(20, 46, 18, 40)  // Belly
                    ctx.quadraticCurveTo(12, 35, 15, 30)  // Chest back to nose
                    
                    ctx.fill()
                    ctx.stroke()
                    
                    // Stella's kind, intelligent eye
                    ctx.fillStyle = "#2F4F2F"  // Dark forest green
                    ctx.beginPath()
                    ctx.arc(25, 27, 2.5, 0, Math.PI * 2)
                    ctx.fill()
                    
                    // Eye sparkle (shows alertness)
                    ctx.fillStyle = "#ffffff"
                    ctx.beginPath()
                    ctx.arc(26, 26, 0.8, 0, Math.PI * 2)
                    ctx.fill()
                    
                    // Nose detail
                    ctx.fillStyle = "#000000"
                    ctx.beginPath()
                    ctx.arc(15, 30, 1.5, 0, Math.PI * 2)
                    ctx.fill()
                }
                
                // Tail wagging animation
                NumberAnimation on tailWag {
                    running: aiActive && stellaActive
                    from: -0.5
                    to: 0.5
                    duration: 800
                    loops: Animation.Infinite
                    easing.type: Easing.InOutQuad
                    onRunningChanged: stellaCanvas.requestPaint()
                }
                
                onTailWagChanged: requestPaint()
                
                // Gentle breathing
                SequentialAnimation on scale {
                    running: aiActive
                    loops: Animation.Infinite
                    NumberAnimation { to: 1.03; duration: 3000; easing.type: Easing.InOutQuad }
                    NumberAnimation { to: 1.0; duration: 3000; easing.type: Easing.InOutQuad }
                }
            }
            
            // Max Jr. (Cat) - Right side, handling system optimization
            Canvas {
                id: maxCanvas
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.verticalCenterOffset: 5
                width: 60
                height: 45
                
                property real purr: 0
                
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    
                    // Max Jr.'s beautiful coat (appears to be cream/light colored from photo)
                    var gradient = ctx.createLinearGradient(0, 0, width, height)
                    gradient.addColorStop(0, "#F5F5DC")   // Beige
                    gradient.addColorStop(0.3, "#FAEBD7") // Antique white
                    gradient.addColorStop(0.7, "#DDD")    // Light gray
                    gradient.addColorStop(1, "#E6E6FA")   // Lavender tint
                    
                    ctx.fillStyle = gradient
                    ctx.strokeStyle = maxActive ? "#0066cc" : "#ffffff"
                    ctx.lineWidth = 1.2
                    
                    // Max Jr.'s feline silhouette (alert, monitoring pose)
                    ctx.beginPath()
                    
                    // Cat head (alert ears up)
                    ctx.moveTo(12, 25)                    // Nose
                    ctx.quadraticCurveTo(8, 20, 15, 15)   // Forehead
                    ctx.quadraticCurveTo(18, 12, 22, 15)  // Left ear
                    ctx.lineTo(20, 20)                    // Ear tip
                    ctx.quadraticCurveTo(25, 12, 30, 15)  // Right ear  
                    ctx.lineTo(28, 20)                    // Right ear tip
                    ctx.quadraticCurveTo(35, 18, 38, 22)  // Back of head
                    
                    // Body (sitting attentively)
                    ctx.quadraticCurveTo(42, 28, 38, 35)  // Neck to back
                    ctx.quadraticCurveTo(35, 38, 30, 36)  // Back
                    
                    // Tail (curved, shows contentment)
                    ctx.quadraticCurveTo(25, 35, 22, 32)  // Tail curve
                    ctx.quadraticCurveTo(18, 30, 15, 32)  // Tail tip
                    ctx.quadraticCurveTo(10, 30, 12, 25)  // Back to nose
                    
                    ctx.fill()
                    ctx.stroke()
                    
                    // Max Jr.'s alert eyes (cats have excellent vision for monitoring!)
                    ctx.fillStyle = "#4169E1"  // Royal blue
                    ctx.beginPath()
                    ctx.arc(20, 22, 2, 0, Math.PI * 2)
                    ctx.fill()
                    
                    ctx.beginPath()
                    ctx.arc(28, 22, 2, 0, Math.PI * 2)
                    ctx.fill()
                    
                    // Eye highlights
                    ctx.fillStyle = "#ffffff"
                    ctx.beginPath()
                    ctx.arc(21, 21, 0.6, 0, Math.PI * 2)
                    ctx.fill()
                    ctx.beginPath()
                    ctx.arc(29, 21, 0.6, 0, Math.PI * 2)
                    ctx.fill()
                    
                    // Nose (pink)
                    ctx.fillStyle = "#FFB6C1"
                    ctx.beginPath()
                    ctx.arc(12, 25, 1, 0, Math.PI * 2)
                    ctx.fill()
                }
                
                // Subtle purring animation (slight vibration when content)
                SequentialAnimation on x {
                    running: aiActive && maxActive
                    loops: Animation.Infinite
                    NumberAnimation { to: maxCanvas.x + 0.5; duration: 100 }
                    NumberAnimation { to: maxCanvas.x - 0.5; duration: 100 }
                    PauseAnimation { duration: 2000 }
                }
                
                // Alert ear twitching
                NumberAnimation on purr {
                    running: aiActive && maxActive
                    from: 0
                    to: Math.PI * 2
                    duration: 3000
                    loops: Animation.Infinite
                    onRunningChanged: maxCanvas.requestPaint()
                }
                
                onPurrChanged: requestPaint()
            }
            
            // Connection between Stella & Max Jr. (showing teamwork)
            Canvas {
                anchors.fill: parent
                
                property real connectionPhase: 0
                
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.reset()
                    
                    if (!aiActive || !stellaActive || !maxActive) return
                    
                    // Heart connection between the companions
                    ctx.strokeStyle = "#FFD700"
                    ctx.lineWidth = 2
                    ctx.globalAlpha = 0.6
                    
                    // Animated data flow between them
                    ctx.setLineDash([3, 3])
                    ctx.lineDashOffset = connectionPhase
                    
                    ctx.beginPath()
                    ctx.moveTo(70, 85)  // From Stella
                    ctx.quadraticCurveTo(80, 65, 90, 85)  // To Max Jr.
                    ctx.stroke()
                }
                
                // Connection animation
                NumberAnimation on connectionPhase {
                    running: aiActive && stellaActive && maxActive
                    from: 0
                    to: 12
                    duration: 2000
                    loops: Animation.Infinite
                    onRunningChanged: parent.requestPaint()
                }
                
                onConnectionPhaseChanged: requestPaint()
            }
            
            // Role indicators
            Text {
                anchors.horizontalCenter: stellaCanvas.horizontalCenter
                anchors.top: stellaCanvas.bottom
                anchors.topMargin: 5
                text: "🛡️📦"
                font.pixelSize: 12
                opacity: stellaActive ? 0.8 : 0.4
            }
            
            Text {
                anchors.horizontalCenter: maxCanvas.horizontalCenter
                anchors.top: maxCanvas.bottom
                anchors.topMargin: 5
                text: "📊⚡"
                font.pixelSize: 12
                opacity: maxActive ? 0.8 : 0.4
            }
            
            // System status indicator (shows when both are working together)
            Rectangle {
                anchors.centerIn: parent
                anchors.verticalCenterOffset: -45
                width: 70
                height: 16
                radius: 8
                color: (stellaActive && maxActive) ? "#00cc66" : "#f9e2af"
                opacity: aiActive ? 0.9 : 0
                
                Text {
                    anchors.centerIn: parent
                    text: (stellaActive && maxActive) ? "🤝 TEAM ACTIVE" : "⚠️ PARTIAL"
                    color: "#ffffff"
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
                // Toggle individual mascot states for demo
                if (Math.random() > 0.5) {
                    stellaActive = !stellaActive
                } else {
                    maxActive = !maxActive
                }
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
}