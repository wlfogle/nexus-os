/**
 * NexusDE Glow Effect Component
 * Animated glow effect for highlighting active states and mascot presence
 */

import QtQuick 2.15
import QtGraphicalEffects 1.15

Item {
    id: glowEffect
    
    property color glowColor: "#3B82F6"
    property real intensity: 0.8
    property int animationDuration: 1000
    property bool pulsing: true
    property bool active: true
    property real glowRadius: 20
    property real spread: 0.3
    
    anchors.fill: parent
    
    Rectangle {
        id: glowSource
        anchors.centerIn: parent
        width: parent.width + glowRadius * 2
        height: parent.height + glowRadius * 2
        color: "transparent"
        border.color: glowColor
        border.width: 2
        radius: parent.radius || 8
        opacity: 0
    }
    
    Glow {
        anchors.fill: glowSource
        source: glowSource
        radius: glowRadius
        samples: 32
        color: glowColor
        spread: spread
        opacity: active ? intensity : 0
        
        Behavior on opacity {
            NumberAnimation {
                duration: animationDuration / 4
                easing.type: Easing.InOutQuad
            }
        }
    }
    
    // Pulsing animation
    SequentialAnimation {
        running: pulsing && active
        loops: Animation.Infinite
        
        ParallelAnimation {
            NumberAnimation {
                target: glowEffect
                property: "intensity"
                from: 0.3
                to: 1.0
                duration: animationDuration / 2
                easing.type: Easing.InOutSine
            }
            
            NumberAnimation {
                target: glowEffect
                property: "glowRadius"
                from: 15
                to: 25
                duration: animationDuration / 2
                easing.type: Easing.InOutSine
            }
        }
        
        ParallelAnimation {
            NumberAnimation {
                target: glowEffect
                property: "intensity"
                from: 1.0
                to: 0.3
                duration: animationDuration / 2
                easing.type: Easing.InOutSine
            }
            
            NumberAnimation {
                target: glowEffect
                property: "glowRadius"
                from: 25
                to: 15
                duration: animationDuration / 2
                easing.type: Easing.InOutSine
            }
        }
    }
    
    // Activation animation
    NumberAnimation {
        id: activationGlow
        target: glowEffect
        property: "intensity"
        from: 0
        to: 1.2
        duration: 300
        easing.type: Easing.OutBack
        
        onFinished: {
            // Return to normal intensity
            var returnAnimation = Qt.createQmlObject('
                import QtQuick 2.15
                NumberAnimation {
                    target: glowEffect
                    property: "intensity"
                    to: 0.8
                    duration: 200
                    easing.type: Easing.InOutQuad
                }
            ', glowEffect)
            returnAnimation.start()
        }
    }
    
    function trigger() {
        if (!active) return
        activationGlow.start()
    }
    
    function setColor(color) {
        glowColor = color
    }
    
    function setIntensity(newIntensity) {
        intensity = Math.max(0, Math.min(1, newIntensity))
    }
}