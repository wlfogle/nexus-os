import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Effects
import "logos"

// NexusOS Logo Gallery - 10 Dynamic Concepts with Stella the Golden Retriever
Rectangle {
    id: logoGallery
    width: 1200
    height: 800
    color: "#1a1a1a"
    
    property int currentLogo: 0
    property bool aiActive: true
    
    ScrollView {
        anchors.fill: parent
        anchors.margins: 20
        
        GridLayout {
            columns: 5
            rowSpacing: 40
            columnSpacing: 40
            
            // Logo 1: Stella as AI Guardian
            NexusLogoStella1 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Stella AI Guardian"
                description: "Stella as the protective AI companion"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 1
                }
            }
            
            // Logo 2: Cyber-Stella Network
            NexusLogoStella2 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Cyber-Stella Network"
                description: "Digital neural network with Stella"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 2
                }
            }
            
            // Logo 3: Holographic Companion
            NexusLogoStella3 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Holographic Companion"
                description: "Stella as holographic OS mascot"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 3
                }
            }
            
            // Logo 4: Universal Retriever
            NexusLogoStella4 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Universal Retriever"
                description: "Stella fetching from universal packages"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 4
                }
            }
            
            // Logo 5: Constellation Stella
            NexusLogoStella5 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Constellation Stella"
                description: "Stella formed by star constellation"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 5
                }
            }
            
            // Logo 6: Data Stream Guardian
            NexusLogoStella6 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Data Stream Guardian"
                description: "Stella protecting data streams"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 6
                }
            }
            
            // Logo 7: Quantum Stella
            NexusLogoStella7 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Quantum Stella"
                description: "Quantum computing with Stella"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 7
                }
            }
            
            // Logo 8: AI Code Companion
            NexusLogoStella8 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "AI Code Companion"
                description: "Stella helping with code"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 8
                }
            }
            
            // Logo 9: Security Watchdog
            NexusLogoStella9 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Security Watchdog"
                description: "Stella as system security guardian"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 9
                }
            }
            
            // Logo 10: Golden Innovation
            NexusLogoStella10 {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                title: "Golden Innovation"
                description: "Classic Stella with tech elements"
                aiActive: logoGallery.aiActive
                
                MouseArea {
                    anchors.fill: parent
                    onClicked: currentLogo = 10
                }
            }
        }
    }
    
    // Logo selection indicator
    Text {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 20
        text: `Selected Logo: ${currentLogo === 0 ? "None" : currentLogo}`
        font.family: "Inter"
        font.pixelSize: 16
        color: "#ffffff"
    }
}