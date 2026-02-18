/*
 * Winux OS Installer Slideshow
 * Modern Apple-Style Design with Fluid Animations
 *
 * Design System:
 * - Colors: #1c1c1e (bg), #2c2c2e (surface), #0a84ff (accent)
 * - Font: SF Pro Display / Inter
 * - Transitions: 500ms ease-in-out
 * - Border radius: 12px standard
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Rectangle {
    id: root

    width: 800
    height: 450

    // Apple-style dark background
    gradient: Gradient {
        GradientStop { position: 0.0; color: "#1c1c1e" }
        GradientStop { position: 1.0; color: "#0d0d0f" }
    }

    // Properties
    property int currentSlide: 0
    property int totalSlides: 8
    property bool activatedInCalamares: true
    property int transitionDuration: 500

    // Auto-advance timer
    Timer {
        id: slideTimer
        interval: 8000
        running: root.activatedInCalamares
        repeat: true
        onTriggered: nextSlide()
    }

    // Navigation functions
    function nextSlide() {
        if (currentSlide < totalSlides - 1) {
            currentSlide++
        } else {
            currentSlide = 0
        }
    }

    function previousSlide() {
        if (currentSlide > 0) {
            currentSlide--
        } else {
            currentSlide = totalSlides - 1
        }
    }

    // Main content container
    Item {
        id: slideContainer
        anchors.fill: parent
        anchors.margins: 40

        // Slide stack
        Item {
            id: slides
            anchors.fill: parent

            // Slide 1: Welcome
            SlideItem {
                visible: opacity > 0
                opacity: currentSlide === 0 ? 1 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: transitionDuration
                        easing.type: Easing.InOutCubic
                    }
                }

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 24

                    // Logo with pulse animation
                    Rectangle {
                        Layout.alignment: Qt.AlignHCenter
                        width: 120
                        height: 120
                        radius: 28
                        color: "transparent"
                        border.color: "#0a84ff"
                        border.width: 3

                        Rectangle {
                            anchors.centerIn: parent
                            width: 100
                            height: 100
                            radius: 24
                            gradient: Gradient {
                                GradientStop { position: 0.0; color: "#0a84ff" }
                                GradientStop { position: 1.0; color: "#5856d6" }
                            }

                            Text {
                                anchors.centerIn: parent
                                text: "W"
                                font.pixelSize: 48
                                font.weight: Font.Bold
                                font.family: "SF Pro Display, Inter, system-ui"
                                color: "#ffffff"
                            }
                        }

                        SequentialAnimation on scale {
                            loops: Animation.Infinite
                            NumberAnimation {
                                to: 1.05
                                duration: 1500
                                easing.type: Easing.InOutSine
                            }
                            NumberAnimation {
                                to: 1.0
                                duration: 1500
                                easing.type: Easing.InOutSine
                            }
                        }
                    }

                    Text {
                        Layout.alignment: Qt.AlignHCenter
                        text: "Welcome to Winux OS"
                        font.pixelSize: 36
                        font.weight: Font.Bold
                        font.family: "SF Pro Display, Inter, system-ui"
                        color: "#ffffff"
                    }

                    Text {
                        Layout.alignment: Qt.AlignHCenter
                        text: "A modern Linux experience"
                        font.pixelSize: 18
                        font.weight: Font.Normal
                        font.family: "SF Pro Text, Inter, system-ui"
                        color: "#8e8e93"
                    }

                    Text {
                        Layout.alignment: Qt.AlignHCenter
                        Layout.topMargin: 24
                        text: "Installation is starting..."
                        font.pixelSize: 14
                        font.family: "SF Pro Text, Inter, system-ui"
                        color: "#48484a"
                    }
                }
            }

            // Slide 2: Modern Desktop
            SlideItem {
                visible: opacity > 0
                opacity: currentSlide === 1 ? 1 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: transitionDuration
                        easing.type: Easing.InOutCubic
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    spacing: 48

                    // Icon
                    FeatureIcon {
                        Layout.preferredWidth: 160
                        Layout.preferredHeight: 160
                        iconText: "\u2328" // Keyboard icon
                        gradientStart: "#5856d6"
                        gradientEnd: "#af52de"
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 16

                        Text {
                            text: "Beautiful Desktop"
                            font.pixelSize: 32
                            font.weight: Font.Bold
                            font.family: "SF Pro Display, Inter, system-ui"
                            color: "#ffffff"
                        }

                        Text {
                            Layout.fillWidth: true
                            text: "Experience a stunning KDE Plasma desktop with a familiar yet innovative design. Smooth animations, customizable workspaces, and a consistent visual experience."
                            font.pixelSize: 16
                            font.family: "SF Pro Text, Inter, system-ui"
                            color: "#8e8e93"
                            wrapMode: Text.WordWrap
                            lineHeight: 1.5
                        }

                        Row {
                            spacing: 12
                            Layout.topMargin: 16

                            FeatureBadge { text: "KDE Plasma 6" }
                            FeatureBadge { text: "Wayland" }
                            FeatureBadge { text: "HDR" }
                        }
                    }
                }
            }

            // Slide 3: Software
            SlideItem {
                visible: opacity > 0
                opacity: currentSlide === 2 ? 1 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: transitionDuration
                        easing.type: Easing.InOutCubic
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    spacing: 48
                    layoutDirection: Qt.RightToLeft

                    FeatureIcon {
                        Layout.preferredWidth: 160
                        Layout.preferredHeight: 160
                        iconText: "\u2b50" // Star
                        gradientStart: "#ff9500"
                        gradientEnd: "#ff3b30"
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 16

                        Text {
                            text: "Thousands of Apps"
                            font.pixelSize: 32
                            font.weight: Font.Bold
                            font.family: "SF Pro Display, Inter, system-ui"
                            color: "#ffffff"
                        }

                        Text {
                            Layout.fillWidth: true
                            text: "Access a vast library of applications through our Software Center. Install Firefox, LibreOffice, GIMP, and more with just one click. Native and Flatpak packages fully supported."
                            font.pixelSize: 16
                            font.family: "SF Pro Text, Inter, system-ui"
                            color: "#8e8e93"
                            wrapMode: Text.WordWrap
                            lineHeight: 1.5
                        }

                        Row {
                            spacing: 12
                            Layout.topMargin: 16

                            FeatureBadge { text: "Flatpak"; accent: "#ff9500" }
                            FeatureBadge { text: "Pacman"; accent: "#ff9500" }
                            FeatureBadge { text: "AUR"; accent: "#ff9500" }
                        }
                    }
                }
            }

            // Slide 4: Gaming
            SlideItem {
                visible: opacity > 0
                opacity: currentSlide === 3 ? 1 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: transitionDuration
                        easing.type: Easing.InOutCubic
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    spacing: 48

                    FeatureIcon {
                        Layout.preferredWidth: 160
                        Layout.preferredHeight: 160
                        iconText: "\u265e" // Chess knight for gaming
                        gradientStart: "#30d158"
                        gradientEnd: "#34c759"
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 16

                        Text {
                            text: "Ready for Gaming"
                            font.pixelSize: 32
                            font.weight: Font.Bold
                            font.family: "SF Pro Display, Inter, system-ui"
                            color: "#ffffff"
                        }

                        Text {
                            Layout.fillWidth: true
                            text: "Play your favorite games with Steam, Proton, and optimized gaming tools. Pre-configured drivers, Vulkan support, and GameMode for the best gaming experience on Linux."
                            font.pixelSize: 16
                            font.family: "SF Pro Text, Inter, system-ui"
                            color: "#8e8e93"
                            wrapMode: Text.WordWrap
                            lineHeight: 1.5
                        }

                        Row {
                            spacing: 12
                            Layout.topMargin: 16

                            FeatureBadge { text: "Steam"; accent: "#30d158" }
                            FeatureBadge { text: "Proton"; accent: "#30d158" }
                            FeatureBadge { text: "Vulkan"; accent: "#30d158" }
                        }
                    }
                }
            }

            // Slide 5: Security
            SlideItem {
                visible: opacity > 0
                opacity: currentSlide === 4 ? 1 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: transitionDuration
                        easing.type: Easing.InOutCubic
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    spacing: 48
                    layoutDirection: Qt.RightToLeft

                    FeatureIcon {
                        Layout.preferredWidth: 160
                        Layout.preferredHeight: 160
                        iconText: "\u26e8" // Shield
                        gradientStart: "#0a84ff"
                        gradientEnd: "#5ac8fa"
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 16

                        Text {
                            text: "Privacy & Security"
                            font.pixelSize: 32
                            font.weight: Font.Bold
                            font.family: "SF Pro Display, Inter, system-ui"
                            color: "#ffffff"
                        }

                        Text {
                            Layout.fillWidth: true
                            text: "Your data belongs to you. Built-in encryption, firewall protection, and zero telemetry. Regular security updates keep your system safe and your privacy protected."
                            font.pixelSize: 16
                            font.family: "SF Pro Text, Inter, system-ui"
                            color: "#8e8e93"
                            wrapMode: Text.WordWrap
                            lineHeight: 1.5
                        }

                        Row {
                            spacing: 12
                            Layout.topMargin: 16

                            FeatureBadge { text: "LUKS" }
                            FeatureBadge { text: "Firewall" }
                            FeatureBadge { text: "No Telemetry" }
                        }
                    }
                }
            }

            // Slide 6: Performance
            SlideItem {
                visible: opacity > 0
                opacity: currentSlide === 5 ? 1 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: transitionDuration
                        easing.type: Easing.InOutCubic
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    spacing: 48

                    FeatureIcon {
                        Layout.preferredWidth: 160
                        Layout.preferredHeight: 160
                        iconText: "\u26a1" // Lightning
                        gradientStart: "#ff2d55"
                        gradientEnd: "#ff6482"
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 16

                        Text {
                            text: "Blazing Fast"
                            font.pixelSize: 32
                            font.weight: Font.Bold
                            font.family: "SF Pro Display, Inter, system-ui"
                            color: "#ffffff"
                        }

                        Text {
                            Layout.fillWidth: true
                            text: "Optimized for performance from the ground up. Fast boot times, efficient memory usage, and responsive UI. Your hardware will feel faster than ever."
                            font.pixelSize: 16
                            font.family: "SF Pro Text, Inter, system-ui"
                            color: "#8e8e93"
                            wrapMode: Text.WordWrap
                            lineHeight: 1.5
                        }

                        Row {
                            spacing: 12
                            Layout.topMargin: 16

                            FeatureBadge { text: "Systemd"; accent: "#ff2d55" }
                            FeatureBadge { text: "Zram"; accent: "#ff2d55" }
                            FeatureBadge { text: "Btrfs"; accent: "#ff2d55" }
                        }
                    }
                }
            }

            // Slide 7: Community
            SlideItem {
                visible: opacity > 0
                opacity: currentSlide === 6 ? 1 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: transitionDuration
                        easing.type: Easing.InOutCubic
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    spacing: 48
                    layoutDirection: Qt.RightToLeft

                    FeatureIcon {
                        Layout.preferredWidth: 160
                        Layout.preferredHeight: 160
                        iconText: "\u2665" // Heart
                        gradientStart: "#af52de"
                        gradientEnd: "#bf5af2"
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 16

                        Text {
                            text: "Community Driven"
                            font.pixelSize: 32
                            font.weight: Font.Bold
                            font.family: "SF Pro Display, Inter, system-ui"
                            color: "#ffffff"
                        }

                        Text {
                            Layout.fillWidth: true
                            text: "Join a welcoming community of users and developers. Get help on our forums, contribute to development, or just share your experience with fellow Winux users."
                            font.pixelSize: 16
                            font.family: "SF Pro Text, Inter, system-ui"
                            color: "#8e8e93"
                            wrapMode: Text.WordWrap
                            lineHeight: 1.5
                        }

                        Row {
                            spacing: 12
                            Layout.topMargin: 16

                            FeatureBadge { text: "Discord"; accent: "#af52de" }
                            FeatureBadge { text: "Forum"; accent: "#af52de" }
                            FeatureBadge { text: "GitHub"; accent: "#af52de" }
                        }
                    }
                }
            }

            // Slide 8: Thank You
            SlideItem {
                visible: opacity > 0
                opacity: currentSlide === 7 ? 1 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: transitionDuration
                        easing.type: Easing.InOutCubic
                    }
                }

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 24

                    Rectangle {
                        Layout.alignment: Qt.AlignHCenter
                        width: 80
                        height: 80
                        radius: 40
                        gradient: Gradient {
                            GradientStop { position: 0.0; color: "#30d158" }
                            GradientStop { position: 1.0; color: "#34c759" }
                        }

                        Text {
                            anchors.centerIn: parent
                            text: "\u2713"
                            font.pixelSize: 40
                            font.weight: Font.Bold
                            color: "#ffffff"
                        }
                    }

                    Text {
                        Layout.alignment: Qt.AlignHCenter
                        text: "Thank You"
                        font.pixelSize: 36
                        font.weight: Font.Bold
                        font.family: "SF Pro Display, Inter, system-ui"
                        color: "#ffffff"
                    }

                    Text {
                        Layout.alignment: Qt.AlignHCenter
                        text: "for choosing Winux OS"
                        font.pixelSize: 18
                        font.family: "SF Pro Text, Inter, system-ui"
                        color: "#8e8e93"
                    }

                    Text {
                        Layout.alignment: Qt.AlignHCenter
                        Layout.topMargin: 16
                        text: "Your new system will be ready soon"
                        font.pixelSize: 14
                        font.family: "SF Pro Text, Inter, system-ui"
                        color: "#48484a"
                    }

                    // Social links
                    Row {
                        Layout.alignment: Qt.AlignHCenter
                        Layout.topMargin: 32
                        spacing: 24

                        SocialButton { label: "winux.io" }
                        SocialButton { label: "GitHub" }
                        SocialButton { label: "Discord" }
                    }
                }
            }
        }
    }

    // Progress indicator at bottom
    Rectangle {
        id: progressContainer
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 24
        width: progressRow.width + 32
        height: 40
        radius: 20
        color: "#2c2c2e"
        border.color: "#3a3a3c"
        border.width: 1

        Row {
            id: progressRow
            anchors.centerIn: parent
            spacing: 8

            Repeater {
                model: totalSlides

                Rectangle {
                    width: currentSlide === index ? 24 : 8
                    height: 8
                    radius: 4
                    color: currentSlide === index ? "#0a84ff" : "#48484a"

                    Behavior on width {
                        NumberAnimation {
                            duration: 300
                            easing.type: Easing.InOutCubic
                        }
                    }

                    Behavior on color {
                        ColorAnimation {
                            duration: 300
                        }
                    }
                }
            }
        }
    }

    // Navigation arrows (optional, for manual control)
    MouseArea {
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: parent.width * 0.2
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            slideTimer.restart()
            previousSlide()
        }
    }

    MouseArea {
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: parent.width * 0.2
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            slideTimer.restart()
            nextSlide()
        }
    }

    // Component definitions
    component SlideItem: Item {
        anchors.fill: parent

        transform: Translate {
            x: opacity < 1 ? 30 : 0

            Behavior on x {
                NumberAnimation {
                    duration: transitionDuration
                    easing.type: Easing.OutCubic
                }
            }
        }
    }

    component FeatureIcon: Rectangle {
        property string iconText: ""
        property color gradientStart: "#0a84ff"
        property color gradientEnd: "#5ac8fa"

        radius: 32
        gradient: Gradient {
            GradientStop { position: 0.0; color: gradientStart }
            GradientStop { position: 1.0; color: gradientEnd }
        }

        // Subtle glow effect
        Rectangle {
            anchors.fill: parent
            anchors.margins: -4
            radius: parent.radius + 4
            color: "transparent"
            border.color: gradientStart
            border.width: 2
            opacity: 0.3
        }

        Text {
            anchors.centerIn: parent
            text: iconText
            font.pixelSize: 64
            color: "#ffffff"
        }

        // Hover animation
        scale: iconHover.containsMouse ? 1.05 : 1.0

        Behavior on scale {
            NumberAnimation {
                duration: 200
                easing.type: Easing.InOutCubic
            }
        }

        MouseArea {
            id: iconHover
            anchors.fill: parent
            hoverEnabled: true
        }
    }

    component FeatureBadge: Rectangle {
        property string text: ""
        property color accent: "#0a84ff"

        width: badgeText.width + 20
        height: 28
        radius: 14
        color: Qt.rgba(accent.r, accent.g, accent.b, 0.2)
        border.color: Qt.rgba(accent.r, accent.g, accent.b, 0.4)
        border.width: 1

        Text {
            id: badgeText
            anchors.centerIn: parent
            text: parent.text
            font.pixelSize: 12
            font.weight: Font.Medium
            font.family: "SF Pro Text, Inter, system-ui"
            color: accent
        }
    }

    component SocialButton: Rectangle {
        property string label: ""

        width: socialText.width + 32
        height: 36
        radius: 18
        color: socialHover.containsMouse ? "#3a3a3c" : "#2c2c2e"
        border.color: "#48484a"
        border.width: 1

        Behavior on color {
            ColorAnimation { duration: 150 }
        }

        Text {
            id: socialText
            anchors.centerIn: parent
            text: label
            font.pixelSize: 13
            font.weight: Font.Medium
            font.family: "SF Pro Text, Inter, system-ui"
            color: "#ffffff"
        }

        MouseArea {
            id: socialHover
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
        }
    }
}
