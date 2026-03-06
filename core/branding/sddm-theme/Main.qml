import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import SddmComponents 2.0

Rectangle {
    id: root
    width: Screen.width
    height: Screen.height

    gradient: Gradient {
        GradientStop { position: 0.0; color: "#0d1117" }
        GradientStop { position: 1.0; color: "#06090f" }
    }

    property color accentColor: "#4a9eff"
    property color textColor: "#e6edf3"
    property color dimColor: "#8b949e"
    property color inputBg: "#161b22"
    property color inputBorder: "#30363d"

    // NexusOS Title
    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: parent.height * 0.15
        spacing: 8

        Text {
            text: "N E X U S O S"
            font.pixelSize: 42
            font.weight: Font.Light
            font.letterSpacing: 4
            color: root.accentColor
            anchors.horizontalCenter: parent.horizontalCenter
        }

        Text {
            text: "AI-Native Operating System"
            font.pixelSize: 14
            color: root.dimColor
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }

    // Login form
    Column {
        anchors.centerIn: parent
        spacing: 16
        width: 320

        // Username
        TextField {
            id: userField
            width: parent.width
            height: 44
            placeholderText: "Username"
            text: userModel.lastUser
            font.pixelSize: 14
            color: root.textColor
            placeholderTextColor: root.dimColor
            background: Rectangle {
                color: root.inputBg
                border.color: userField.activeFocus ? root.accentColor : root.inputBorder
                border.width: 1
                radius: 6
            }
            Keys.onReturnPressed: passwordField.focus = true
        }

        // Password
        TextField {
            id: passwordField
            width: parent.width
            height: 44
            placeholderText: "Password"
            echoMode: TextInput.Password
            font.pixelSize: 14
            color: root.textColor
            placeholderTextColor: root.dimColor
            background: Rectangle {
                color: root.inputBg
                border.color: passwordField.activeFocus ? root.accentColor : root.inputBorder
                border.width: 1
                radius: 6
            }
            Keys.onReturnPressed: sddm.login(userField.text, passwordField.text, sessionModel.lastIndex)
        }

        // Login button
        Button {
            width: parent.width
            height: 44
            text: "Log In"
            font.pixelSize: 14
            font.weight: Font.Medium
            onClicked: sddm.login(userField.text, passwordField.text, sessionModel.lastIndex)
            contentItem: Text {
                text: parent.text
                font: parent.font
                color: "#ffffff"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
            background: Rectangle {
                color: parent.pressed ? Qt.darker(root.accentColor, 1.2) :
                       parent.hovered ? Qt.lighter(root.accentColor, 1.1) : root.accentColor
                radius: 6
            }
        }

        // Error message
        Text {
            id: errorMessage
            width: parent.width
            text: ""
            color: "#f85149"
            font.pixelSize: 12
            horizontalAlignment: Text.AlignHCenter
            visible: text !== ""
        }
    }

    // Session selector (bottom left)
    ComboBox {
        id: sessionSelect
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.margins: 20
        width: 200
        model: sessionModel
        currentIndex: sessionModel.lastIndex
        textRole: "name"
        font.pixelSize: 12
    }

    // Clock (bottom right)
    Text {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 20
        font.pixelSize: 14
        color: root.dimColor
        text: Qt.formatDateTime(new Date(), "dddd, MMMM d  h:mm AP")

        Timer {
            interval: 30000
            running: true
            repeat: true
            onTriggered: parent.text = Qt.formatDateTime(new Date(), "dddd, MMMM d  h:mm AP")
        }
    }

    // Power buttons (top right)
    Row {
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: 20
        spacing: 12

        Button {
            text: "⏻"
            font.pixelSize: 18
            onClicked: sddm.powerOff()
            flat: true
            contentItem: Text { text: parent.text; font: parent.font; color: root.dimColor }
            background: Rectangle { color: "transparent" }
        }

        Button {
            text: "⟳"
            font.pixelSize: 18
            onClicked: sddm.reboot()
            flat: true
            contentItem: Text { text: parent.text; font: parent.font; color: root.dimColor }
            background: Rectangle { color: "transparent" }
        }
    }

    Connections {
        target: sddm
        function onLoginFailed() {
            errorMessage.text = "Login failed. Check your credentials."
            passwordField.text = ""
            passwordField.focus = true
        }
        function onLoginSucceeded() {
            errorMessage.text = ""
        }
    }

    Component.onCompleted: {
        if (userField.text !== "") {
            passwordField.focus = true
        } else {
            userField.focus = true
        }
    }
}
