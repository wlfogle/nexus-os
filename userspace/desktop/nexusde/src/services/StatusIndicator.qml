/**
 * StatusIndicator Component
 * Service health and status display for NexusDE System Services
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import NexusDE.Theme 1.0

Item {
    id: statusIndicator
    
    property string service: "Service"
    property string status: "unknown"
    property string details: ""
    property color color: NexusTheme.colors.primary
    property bool active: false
    
    width: 120
    height: 60
    
    Rectangle {
        anchors.fill: parent
        color: active ? Qt.rgba(color.r, color.g, color.b, 0.1) : "transparent"
        border.color: active ? color : NexusTheme.colors.outline
        border.width: 1
        radius: NexusTheme.radius.sm
        
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: NexusTheme.spacing.xs
            spacing: 2
            
            Row {
                spacing: 4
                Layout.alignment: Qt.AlignHCenter
                
                Rectangle {
                    width: 8
                    height: 8
                    radius: 4
                    color: getStatusColor()
                    
                    SequentialAnimation {
                        running: status === "running" || status === "active"
                        loops: Animation.Infinite
                        
                        PropertyAnimation {
                            target: parent
                            property: "opacity"
                            from: 1.0
                            to: 0.3
                            duration: 1000
                        }
                        PropertyAnimation {
                            target: parent
                            property: "opacity"
                            from: 0.3
                            to: 1.0
                            duration: 1000
                        }
                    }
                }
                
                Text {
                    text: service
                    font.family: NexusTheme.typography.primaryFont
                    font.pixelSize: 11
                    font.weight: Font.Medium
                    color: active ? color : NexusTheme.colors.onSurfaceVariant
                }
            }
            
            Text {
                text: status.toUpperCase()
                font.family: NexusTheme.typography.primaryFont
                font.pixelSize: 10
                font.weight: Font.Bold
                color: getStatusColor()
                Layout.alignment: Qt.AlignHCenter
            }
            
            Text {
                text: details
                font.family: NexusTheme.typography.primaryFont
                font.pixelSize: 9
                color: NexusTheme.colors.onSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
                elide: Text.ElideRight
                Layout.fillWidth: true
                horizontalAlignment: Text.AlignHCenter
            }
        }
        
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            onEntered: parent.scale = 1.02
            onExited: parent.scale = 1.0
            
            Behavior on scale {
                NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
            }
        }
    }
    
    function getStatusColor() {
        switch (status.toLowerCase()) {
            case "running":
            case "active":
            case "ready":
                return NexusTheme.colors.success
            case "idle":
            case "paused":
                return NexusTheme.colors.warning
            case "stopped":
            case "offline":
            case "error":
                return NexusTheme.colors.error
            default:
                return NexusTheme.colors.onSurfaceVariant
        }
    }
}