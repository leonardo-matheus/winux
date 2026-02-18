/* Winux OS - Calamares Installer Slideshow
 * Sprint 15-16: Build System and Installer
 *
 * A beautiful QML slideshow shown during system installation
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtGraphicalEffects 1.15

Presentation {
    id: presentation

    // Timer interval for auto-advance (10 seconds)
    Timer {
        id: advanceTimer
        interval: 10000
        running: presentation.activatedInCalamares
        repeat: true
        onTriggered: presentation.goToNextSlide()
    }

    // Slide 1: Welcome
    Slide {
        id: welcomeSlide

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#1a1a2e" }
                GradientStop { position: 1.0; color: "#16213e" }
            }

            ColumnLayout {
                anchors.centerIn: parent
                spacing: 30

                Image {
                    source: "winux-logo.svg"
                    sourceSize.width: 200
                    sourceSize.height: 200
                    Layout.alignment: Qt.AlignHCenter

                    SequentialAnimation on scale {
                        loops: Animation.Infinite
                        NumberAnimation { to: 1.05; duration: 2000; easing.type: Easing.InOutQuad }
                        NumberAnimation { to: 1.0; duration: 2000; easing.type: Easing.InOutQuad }
                    }
                }

                Text {
                    text: "Welcome to Winux OS"
                    color: "#e94560"
                    font.pixelSize: 42
                    font.bold: true
                    font.family: "Segoe UI"
                    Layout.alignment: Qt.AlignHCenter

                    layer.enabled: true
                    layer.effect: Glow {
                        radius: 8
                        samples: 17
                        color: "#e94560"
                        spread: 0.3
                    }
                }

                Text {
                    text: "The Modern Linux Experience"
                    color: "#ffffff"
                    font.pixelSize: 24
                    font.family: "Segoe UI"
                    opacity: 0.9
                    Layout.alignment: Qt.AlignHCenter
                }

                Text {
                    text: "Installing your new operating system..."
                    color: "#0f3460"
                    font.pixelSize: 16
                    font.family: "Segoe UI"
                    opacity: 0.7
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: 20
                }
            }
        }
    }

    // Slide 2: Modern Desktop
    Slide {
        id: desktopSlide

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#0f3460" }
                GradientStop { position: 1.0; color: "#16213e" }
            }

            RowLayout {
                anchors.centerIn: parent
                anchors.margins: 50
                spacing: 60

                // Icon representation
                Rectangle {
                    width: 300
                    height: 300
                    color: "transparent"

                    Image {
                        anchors.centerIn: parent
                        source: "slide-desktop.svg"
                        sourceSize.width: 250
                        sourceSize.height: 250
                    }
                }

                ColumnLayout {
                    spacing: 20
                    Layout.maximumWidth: 400

                    Text {
                        text: "Beautiful Modern Desktop"
                        color: "#e94560"
                        font.pixelSize: 32
                        font.bold: true
                        font.family: "Segoe UI"
                    }

                    Text {
                        text: "Winux OS features a stunning KDE Plasma desktop with a familiar yet innovative design. Enjoy smooth animations, customizable panels, and a consistent visual experience across all applications."
                        color: "#ffffff"
                        font.pixelSize: 16
                        font.family: "Segoe UI"
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                        lineHeight: 1.4
                        opacity: 0.9
                    }

                    RowLayout {
                        spacing: 15
                        Layout.topMargin: 10

                        FeatureBadge { text: "KDE Plasma 6" }
                        FeatureBadge { text: "Wayland" }
                        FeatureBadge { text: "HDR Support" }
                    }
                }
            }
        }
    }

    // Slide 3: Software Center
    Slide {
        id: softwareSlide

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#16213e" }
                GradientStop { position: 1.0; color: "#1a1a2e" }
            }

            RowLayout {
                anchors.centerIn: parent
                anchors.margins: 50
                spacing: 60
                layoutDirection: Qt.RightToLeft

                Rectangle {
                    width: 300
                    height: 300
                    color: "transparent"

                    Image {
                        anchors.centerIn: parent
                        source: "slide-software.svg"
                        sourceSize.width: 250
                        sourceSize.height: 250
                    }
                }

                ColumnLayout {
                    spacing: 20
                    Layout.maximumWidth: 400

                    Text {
                        text: "Thousands of Applications"
                        color: "#e94560"
                        font.pixelSize: 32
                        font.bold: true
                        font.family: "Segoe UI"
                    }

                    Text {
                        text: "Access a vast library of free and open-source software through our Software Center. Install popular applications like Firefox, LibreOffice, GIMP, and more with just one click. Flatpak and native packages are fully supported."
                        color: "#ffffff"
                        font.pixelSize: 16
                        font.family: "Segoe UI"
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                        lineHeight: 1.4
                        opacity: 0.9
                    }

                    RowLayout {
                        spacing: 15
                        Layout.topMargin: 10

                        FeatureBadge { text: "Flatpak" }
                        FeatureBadge { text: "Pacman" }
                        FeatureBadge { text: "AUR" }
                    }
                }
            }
        }
    }

    // Slide 4: Gaming
    Slide {
        id: gamingSlide

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#1a1a2e" }
                GradientStop { position: 1.0; color: "#0f3460" }
            }

            RowLayout {
                anchors.centerIn: parent
                anchors.margins: 50
                spacing: 60

                Rectangle {
                    width: 300
                    height: 300
                    color: "transparent"

                    Image {
                        anchors.centerIn: parent
                        source: "slide-gaming.svg"
                        sourceSize.width: 250
                        sourceSize.height: 250
                    }
                }

                ColumnLayout {
                    spacing: 20
                    Layout.maximumWidth: 400

                    Text {
                        text: "Ready for Gaming"
                        color: "#e94560"
                        font.pixelSize: 32
                        font.bold: true
                        font.family: "Segoe UI"
                    }

                    Text {
                        text: "Play your favorite games with Steam, Proton, and our optimized gaming tools. Winux includes pre-configured drivers, Vulkan support, and GameMode for the best gaming experience on Linux."
                        color: "#ffffff"
                        font.pixelSize: 16
                        font.family: "Segoe UI"
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                        lineHeight: 1.4
                        opacity: 0.9
                    }

                    RowLayout {
                        spacing: 15
                        Layout.topMargin: 10

                        FeatureBadge { text: "Steam" }
                        FeatureBadge { text: "Proton" }
                        FeatureBadge { text: "Vulkan" }
                    }
                }
            }
        }
    }

    // Slide 5: Privacy & Security
    Slide {
        id: securitySlide

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#0f3460" }
                GradientStop { position: 1.0; color: "#16213e" }
            }

            RowLayout {
                anchors.centerIn: parent
                anchors.margins: 50
                spacing: 60
                layoutDirection: Qt.RightToLeft

                Rectangle {
                    width: 300
                    height: 300
                    color: "transparent"

                    Image {
                        anchors.centerIn: parent
                        source: "slide-security.svg"
                        sourceSize.width: 250
                        sourceSize.height: 250
                    }
                }

                ColumnLayout {
                    spacing: 20
                    Layout.maximumWidth: 400

                    Text {
                        text: "Privacy & Security First"
                        color: "#e94560"
                        font.pixelSize: 32
                        font.bold: true
                        font.family: "Segoe UI"
                    }

                    Text {
                        text: "Your data belongs to you. Winux OS includes built-in encryption, firewall protection, and no telemetry. Regular security updates keep your system safe and your privacy protected."
                        color: "#ffffff"
                        font.pixelSize: 16
                        font.family: "Segoe UI"
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                        lineHeight: 1.4
                        opacity: 0.9
                    }

                    RowLayout {
                        spacing: 15
                        Layout.topMargin: 10

                        FeatureBadge { text: "LUKS" }
                        FeatureBadge { text: "Firewall" }
                        FeatureBadge { text: "No Telemetry" }
                    }
                }
            }
        }
    }

    // Slide 6: Thank You
    Slide {
        id: thanksSlide

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#16213e" }
                GradientStop { position: 1.0; color: "#1a1a2e" }
            }

            ColumnLayout {
                anchors.centerIn: parent
                spacing: 30

                Image {
                    source: "winux-logo.svg"
                    sourceSize.width: 150
                    sourceSize.height: 150
                    Layout.alignment: Qt.AlignHCenter
                }

                Text {
                    text: "Thank You for Choosing Winux!"
                    color: "#e94560"
                    font.pixelSize: 36
                    font.bold: true
                    font.family: "Segoe UI"
                    Layout.alignment: Qt.AlignHCenter
                }

                Text {
                    text: "Installation is in progress. Your system will be ready soon."
                    color: "#ffffff"
                    font.pixelSize: 18
                    font.family: "Segoe UI"
                    opacity: 0.9
                    Layout.alignment: Qt.AlignHCenter
                }

                RowLayout {
                    spacing: 40
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: 30

                    SocialLink { icon: "web"; text: "winux.io" }
                    SocialLink { icon: "github"; text: "GitHub" }
                    SocialLink { icon: "discord"; text: "Discord" }
                    SocialLink { icon: "forum"; text: "Forum" }
                }
            }
        }
    }

    // Progress indicator at bottom
    Rectangle {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 30
        width: slideIndicator.width + 40
        height: 40
        radius: 20
        color: "#000000"
        opacity: 0.3

        Row {
            id: slideIndicator
            anchors.centerIn: parent
            spacing: 12

            Repeater {
                model: 6

                Rectangle {
                    width: presentation.currentSlide === index ? 24 : 10
                    height: 10
                    radius: 5
                    color: presentation.currentSlide === index ? "#e94560" : "#ffffff"
                    opacity: presentation.currentSlide === index ? 1.0 : 0.5

                    Behavior on width { NumberAnimation { duration: 200 } }
                    Behavior on opacity { NumberAnimation { duration: 200 } }
                }
            }
        }
    }

    // Helper components
    component FeatureBadge: Rectangle {
        property string text: ""

        width: badgeText.width + 20
        height: 30
        radius: 15
        color: "#e94560"
        opacity: 0.9

        Text {
            id: badgeText
            anchors.centerIn: parent
            text: parent.text
            color: "#ffffff"
            font.pixelSize: 12
            font.bold: true
            font.family: "Segoe UI"
        }
    }

    component SocialLink: ColumnLayout {
        property string icon: ""
        property string text: ""

        spacing: 8

        Rectangle {
            width: 50
            height: 50
            radius: 25
            color: "#e94560"
            opacity: 0.8
            Layout.alignment: Qt.AlignHCenter

            Image {
                anchors.centerIn: parent
                source: parent.parent.icon + ".svg"
                sourceSize.width: 24
                sourceSize.height: 24
            }
        }

        Text {
            text: parent.text
            color: "#ffffff"
            font.pixelSize: 12
            font.family: "Segoe UI"
            opacity: 0.8
            Layout.alignment: Qt.AlignHCenter
        }
    }
}
