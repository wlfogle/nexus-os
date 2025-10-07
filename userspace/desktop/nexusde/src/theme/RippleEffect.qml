/**
 * NexusDE Ripple Effect Component
 * Material Design-style ripple animation for button interactions
 */

import QtQuick 2.15

Item {
    id: rippleEffect
    
    property color rippleColor: "#3B82F6"
    property int duration: 400
    property real maxRadius: Math.max(width, height)
    property bool clipToParent: true
    
    anchors.fill: parent
    clip: clipToParent
    
    Rectangle {
        id: ripple
        width: 0
        height: width
        radius: width / 2
        color: Qt.rgba(rippleColor.r, rippleColor.g, rippleColor.b, 0.3)
        opacity: 0
        
        property real targetRadius: maxRadius * 1.2
        
        ParallelAnimation {
            id: rippleAnimation
            
            NumberAnimation {
                target: ripple
                property: "width"
                from: 0
                to: ripple.targetRadius * 2
                duration: rippleEffect.duration
                easing.type: Easing.OutQuad
            }
            
            SequentialAnimation {
                NumberAnimation {
                    target: ripple
                    property: "opacity"
                    from: 0
                    to: 1
                    duration: rippleEffect.duration * 0.1
                    easing.type: Easing.OutQuad
                }
                
                NumberAnimation {
                    target: ripple
                    property: "opacity"
                    from: 1
                    to: 0
                    duration: rippleEffect.duration * 0.9
                    easing.type: Easing.OutQuad
                }
            }
        }
        
        onOpacityChanged: {
            if (opacity === 0 && width > 0) {
                // Reset for next animation
                width = 0
            }
        }
    }
    
    function trigger(centerX, centerY) {
        // Position ripple at touch point
        if (centerX !== undefined && centerY !== undefined) {
            ripple.x = centerX - ripple.targetRadius
            ripple.y = centerY - ripple.targetRadius
        } else {
            // Default to center
            ripple.x = (width - ripple.targetRadius * 2) / 2
            ripple.y = (height - ripple.targetRadius * 2) / 2
        }
        
        // Calculate appropriate radius based on distance from corners
        var corners = [
            Math.sqrt(Math.pow(ripple.x, 2) + Math.pow(ripple.y, 2)),
            Math.sqrt(Math.pow(width - ripple.x, 2) + Math.pow(ripple.y, 2)),
            Math.sqrt(Math.pow(ripple.x, 2) + Math.pow(height - ripple.y, 2)),
            Math.sqrt(Math.pow(width - ripple.x, 2) + Math.pow(height - ripple.y, 2))
        ]
        
        ripple.targetRadius = Math.max.apply(null, corners) + 10
        
        // Start animation
        rippleAnimation.start()
    }
    
    function setColor(color) {
        rippleColor = color
        ripple.color = Qt.rgba(color.r, color.g, color.b, 0.3)
    }
    
    Component.onCompleted: {
        // Auto-trigger if parent has mouse area
        if (parent && parent.hasOwnProperty('clicked')) {
            parent.clicked.connect(function() {
                trigger()
            })
        }
    }
}