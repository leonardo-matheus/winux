import QtQuick 2.0;
import calamares.slideshow 1.0;

Presentation {
    id: presentation

    Timer {
        interval: 8000
        running: true
        repeat: true
        onTriggered: presentation.goToNextSlide()
    }

    Slide {
        Rectangle {
            anchors.fill: parent
            color: "#0d1117"

            Column {
                anchors.centerIn: parent
                spacing: 30

                Text {
                    text: "W"
                    font.pixelSize: 80
                    font.weight: Font.Light
                    color: "#58a6ff"
                    anchors.horizontalCenter: parent.horizontalCenter
                }

                Text {
                    text: "Bem-vindo ao Winux OS"
                    font.pixelSize: 32
                    font.weight: Font.Light
                    color: "#c9d1d9"
                    anchors.horizontalCenter: parent.horizontalCenter
                }

                Text {
                    text: "O melhor dos dois mundos"
                    font.pixelSize: 16
                    color: "#8b949e"
                    anchors.horizontalCenter: parent.horizontalCenter
                }
            }
        }
    }

    Slide {
        Rectangle {
            anchors.fill: parent
            color: "#0d1117"

            Column {
                anchors.centerIn: parent
                spacing: 25

                Text {
                    text: "Developer Edition"
                    font.pixelSize: 28
                    color: "#58a6ff"
                    anchors.horizontalCenter: parent.horizontalCenter
                }

                Text {
                    text: "VS Code, IntelliJ, Python, Node.js,\nPHP, Java, Rust, Docker, Git"
                    font.pixelSize: 16
                    color: "#8b949e"
                    horizontalAlignment: Text.AlignHCenter
                    anchors.horizontalCenter: parent.horizontalCenter
                }
            }
        }
    }

    Slide {
        Rectangle {
            anchors.fill: parent
            color: "#0d1117"

            Column {
                anchors.centerIn: parent
                spacing: 25

                Text {
                    text: "Personalize"
                    font.pixelSize: 28
                    color: "#58a6ff"
                    anchors.horizontalCenter: parent.horizontalCenter
                }

                Text {
                    text: "Windows Like | Linux Like | Mac Like"
                    font.pixelSize: 16
                    color: "#c9d1d9"
                    anchors.horizontalCenter: parent.horizontalCenter
                }
            }
        }
    }

    Slide {
        Rectangle {
            anchors.fill: parent
            color: "#0d1117"

            Column {
                anchors.centerIn: parent
                spacing: 25

                Text {
                    text: "Idiomas"
                    font.pixelSize: 28
                    color: "#58a6ff"
                    anchors.horizontalCenter: parent.horizontalCenter
                }

                Text {
                    text: "Portugues (Brasil) | English (USA)"
                    font.pixelSize: 16
                    color: "#c9d1d9"
                    anchors.horizontalCenter: parent.horizontalCenter
                }
            }
        }
    }
}
