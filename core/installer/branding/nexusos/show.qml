import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Rectangle {
    id: slideshow
    width: 800
    height: 440
    color: "#0d1117"

    property int currentSlide: 0
    property int slideCount: 5

    Timer {
        interval: 6000
        running: true
        repeat: true
        onTriggered: currentSlide = (currentSlide + 1) % slideCount
    }

    // Slide content
    Item {
        anchors.fill: parent
        anchors.margins: 40

        // Slide 0: Welcome
        Column {
            visible: currentSlide === 0
            anchors.centerIn: parent
            spacing: 16

            Text {
                text: "Welcome to NexusOS"
                font.pixelSize: 32
                font.weight: Font.Light
                color: "#4a9eff"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "The AI-Native Operating System"
                font.pixelSize: 16
                color: "#8b949e"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "KDE Plasma X11 • NVIDIA Optimized • AI-Powered"
                font.pixelSize: 13
                color: "#6e7681"
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }

        // Slide 1: Stella
        Column {
            visible: currentSlide === 1
            anchors.centerIn: parent
            spacing: 12

            Text {
                text: "🐕 Meet Stella"
                font.pixelSize: 28
                color: "#c084fc"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "Your Security Guardian"
                font.pixelSize: 16
                color: "#8b949e"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "Stella monitors your firewall, scans for threats,\nvalidates packages, and hardens your system.\nShe wags her tail when everything is secure!"
                font.pixelSize: 13
                color: "#e6edf3"
                horizontalAlignment: Text.AlignHCenter
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }

        // Slide 2: Max Jr.
        Column {
            visible: currentSlide === 2
            anchors.centerIn: parent
            spacing: 12

            Text {
                text: "🐱 Meet Max Jr."
                font.pixelSize: 28
                color: "#fbbf24"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "Your Performance Optimizer"
                font.pixelSize: 16
                color: "#8b949e"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "Max Jr. monitors CPU, GPU, and memory in real-time.\nHe activates gaming mode and optimizes your system.\nHe purrs when performance is at its peak!"
                font.pixelSize: 13
                color: "#e6edf3"
                horizontalAlignment: Text.AlignHCenter
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }

        // Slide 3: Package Manager
        Column {
            visible: currentSlide === 3
            anchors.centerIn: parent
            spacing: 12

            Text {
                text: "📦 nexuspkg"
                font.pixelSize: 28
                color: "#4a9eff"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "Universal Package Manager"
                font.pixelSize: 16
                color: "#8b949e"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "Install software from 20+ sources with one command.\nnala, apt, flatpak, snap, pip, cargo, npm,\nAppImage, GitHub releases, and more."
                font.pixelSize: 13
                color: "#e6edf3"
                horizontalAlignment: Text.AlignHCenter
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }

        // Slide 4: AI Ready
        Column {
            visible: currentSlide === 4
            anchors.centerIn: parent
            spacing: 12

            Text {
                text: "🧠 AI-Native"
                font.pixelSize: 28
                color: "#4a9eff"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "Built for AI Workloads"
                font.pixelSize: 16
                color: "#8b949e"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            Text {
                text: "NVIDIA CUDA out of the box. Ollama for local LLMs.\nDocker for containerized AI services.\nOptimized kernel for GPU compute and gaming."
                font.pixelSize: 13
                color: "#e6edf3"
                horizontalAlignment: Text.AlignHCenter
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }
    }

    // Slide indicators
    Row {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 16
        spacing: 8

        Repeater {
            model: slideCount
            Rectangle {
                width: currentSlide === index ? 24 : 8
                height: 8
                radius: 4
                color: currentSlide === index ? "#4a9eff" : "#30363d"
                Behavior on width { NumberAnimation { duration: 200 } }
            }
        }
    }
}
