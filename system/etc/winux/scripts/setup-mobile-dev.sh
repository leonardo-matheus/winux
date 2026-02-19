#!/bin/bash
#===============================================================================
# Winux OS - Complete Mobile Development Environment Setup
# Android, iOS (open-source tools), Cross-Platform (Flutter, React Native, etc)
#===============================================================================

set -e

#-------------------------------------------------------------------------------
# Colors and Logging
#-------------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
NC='\033[0m'
BOLD='\033[1m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${CYAN}[STEP]${NC} $1"; }

#-------------------------------------------------------------------------------
# Winux ASCII Logo
#-------------------------------------------------------------------------------
show_logo() {
    echo -e "${CYAN}"
    cat << 'EOF'

    ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗
    ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝
    ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝
    ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗
    ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗
     ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝

EOF
    echo -e "${NC}"
    echo -e "${WHITE}${BOLD}    Mobile Development Environment Setup${NC}"
    echo -e "${MAGENTA}    Android | iOS Tools | Cross-Platform${NC}"
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo ""
}

#-------------------------------------------------------------------------------
# Check System Requirements
#-------------------------------------------------------------------------------
check_requirements() {
    log_step "Checking system requirements..."

    # Check if running as root
    if [ "$EUID" -eq 0 ]; then
        log_warn "Running as root. Some tools will be installed for root user."
    fi

    # Check available disk space (need at least 20GB)
    AVAILABLE_SPACE=$(df -BG "$HOME" | awk 'NR==2 {print $4}' | tr -d 'G')
    if [ "$AVAILABLE_SPACE" -lt 20 ]; then
        log_warn "Low disk space: ${AVAILABLE_SPACE}GB available. Recommend 20GB+ for mobile dev."
    fi

    # Check RAM (recommend 8GB+)
    TOTAL_RAM=$(free -g | awk '/^Mem:/{print $2}')
    if [ "$TOTAL_RAM" -lt 8 ]; then
        log_warn "Low RAM: ${TOTAL_RAM}GB. Recommend 8GB+ for Android emulator."
    fi

    # Check KVM support
    if [ -e /dev/kvm ]; then
        log_success "KVM acceleration available"
    else
        log_warn "KVM not available. Android emulator will be slower."
    fi

    log_success "System requirements check complete"
}

#-------------------------------------------------------------------------------
# Install System Dependencies
#-------------------------------------------------------------------------------
install_dependencies() {
    log_step "Installing system dependencies..."

    sudo apt-get update

    # Common build tools
    sudo apt-get install -y \
        build-essential \
        git \
        curl \
        wget \
        unzip \
        zip \
        p7zip-full \
        software-properties-common \
        apt-transport-https \
        ca-certificates \
        gnupg \
        lsb-release

    # Java (required for Android)
    sudo apt-get install -y \
        openjdk-17-jdk \
        openjdk-17-jdk-headless

    # Libraries for mobile development
    sudo apt-get install -y \
        libgl1-mesa-dev \
        libpulse0 \
        libnss3 \
        libxcomposite1 \
        libxcursor1 \
        libxi6 \
        libxtst6 \
        libasound2t64 \
        libatk1.0-0 \
        libatk-bridge2.0-0 \
        libgtk-3-0 \
        libgdk-pixbuf2.0-0

    # KVM/QEMU for Android emulator
    sudo apt-get install -y \
        qemu-kvm \
        libvirt-daemon-system \
        libvirt-clients \
        bridge-utils \
        cpu-checker 2>/dev/null || true

    # Add user to kvm group
    sudo usermod -aG kvm "$USER" 2>/dev/null || true

    log_success "System dependencies installed"
}

#===============================================================================
# ANDROID DEVELOPMENT
#===============================================================================

setup_android_sdk() {
    log_step "Setting up Android SDK..."

    export ANDROID_HOME="$HOME/Android/Sdk"
    export ANDROID_SDK_ROOT="$ANDROID_HOME"
    mkdir -p "$ANDROID_HOME"
    mkdir -p "$ANDROID_HOME/cmdline-tools"

    # Download Android command-line tools
    CMDLINE_TOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
    CMDLINE_TOOLS_ZIP="/tmp/cmdline-tools.zip"

    if [ ! -d "$ANDROID_HOME/cmdline-tools/latest" ]; then
        log_info "Downloading Android command-line tools..."
        wget -q --show-progress -O "$CMDLINE_TOOLS_ZIP" "$CMDLINE_TOOLS_URL"

        unzip -q "$CMDLINE_TOOLS_ZIP" -d "$ANDROID_HOME/cmdline-tools"
        mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
        rm "$CMDLINE_TOOLS_ZIP"

        log_success "Command-line tools installed"
    else
        log_warn "Command-line tools already installed"
    fi

    # Add to PATH
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"

    # Accept licenses
    log_info "Accepting Android SDK licenses..."
    yes | sdkmanager --licenses 2>/dev/null || true

    # Install SDK components
    log_info "Installing Android SDK components..."
    sdkmanager --install \
        "platform-tools" \
        "build-tools;34.0.0" \
        "platforms;android-34" \
        "sources;android-34" \
        "emulator" \
        "system-images;android-34;google_apis;x86_64" \
        "extras;android;m2repository" \
        "extras;google;m2repository"

    log_success "Android SDK installed"
}

setup_android_ndk() {
    log_step "Setting up Android NDK..."

    export ANDROID_HOME="$HOME/Android/Sdk"
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"

    # Install NDK
    sdkmanager --install "ndk;26.1.10909125"

    export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"

    log_success "Android NDK installed at $ANDROID_NDK_HOME"
}

setup_android_emulator() {
    log_step "Setting up Android Emulator..."

    export ANDROID_HOME="$HOME/Android/Sdk"
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$PATH"

    # Create AVD (Android Virtual Device)
    if ! avdmanager list avd | grep -q "Winux_Phone"; then
        log_info "Creating Android Virtual Device..."
        echo "no" | avdmanager create avd \
            -n "Winux_Phone" \
            -k "system-images;android-34;google_apis;x86_64" \
            -d "pixel_6" \
            --force

        log_success "AVD 'Winux_Phone' created"
    else
        log_warn "AVD 'Winux_Phone' already exists"
    fi

    # Create AVD for tablet
    if ! avdmanager list avd | grep -q "Winux_Tablet"; then
        echo "no" | avdmanager create avd \
            -n "Winux_Tablet" \
            -k "system-images;android-34;google_apis;x86_64" \
            -d "pixel_tablet" \
            --force

        log_success "AVD 'Winux_Tablet' created"
    fi

    log_success "Android Emulator configured"
}

setup_gradle() {
    log_step "Setting up Gradle..."

    GRADLE_VERSION="8.5"
    GRADLE_HOME="/opt/gradle/gradle-${GRADLE_VERSION}"

    if [ ! -d "$GRADLE_HOME" ]; then
        log_info "Downloading Gradle ${GRADLE_VERSION}..."
        wget -q --show-progress -O /tmp/gradle.zip \
            "https://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip"

        sudo mkdir -p /opt/gradle
        sudo unzip -q /tmp/gradle.zip -d /opt/gradle
        rm /tmp/gradle.zip

        log_success "Gradle installed at $GRADLE_HOME"
    else
        log_warn "Gradle already installed"
    fi

    export GRADLE_HOME
    export PATH="$GRADLE_HOME/bin:$PATH"
}

setup_adb_fastboot() {
    log_step "Configuring ADB and Fastboot..."

    export ANDROID_HOME="$HOME/Android/Sdk"

    # Create udev rules for Android devices
    sudo tee /etc/udev/rules.d/51-android.rules > /dev/null << 'EOF'
# Google
SUBSYSTEM=="usb", ATTR{idVendor}=="18d1", MODE="0666", GROUP="plugdev"
# Samsung
SUBSYSTEM=="usb", ATTR{idVendor}=="04e8", MODE="0666", GROUP="plugdev"
# Xiaomi
SUBSYSTEM=="usb", ATTR{idVendor}=="2717", MODE="0666", GROUP="plugdev"
# OnePlus
SUBSYSTEM=="usb", ATTR{idVendor}=="2a70", MODE="0666", GROUP="plugdev"
# Huawei
SUBSYSTEM=="usb", ATTR{idVendor}=="12d1", MODE="0666", GROUP="plugdev"
# Motorola
SUBSYSTEM=="usb", ATTR{idVendor}=="22b8", MODE="0666", GROUP="plugdev"
# LG
SUBSYSTEM=="usb", ATTR{idVendor}=="1004", MODE="0666", GROUP="plugdev"
# Sony
SUBSYSTEM=="usb", ATTR{idVendor}=="0fce", MODE="0666", GROUP="plugdev"
# ASUS
SUBSYSTEM=="usb", ATTR{idVendor}=="0b05", MODE="0666", GROUP="plugdev"
# HTC
SUBSYSTEM=="usb", ATTR{idVendor}=="0bb4", MODE="0666", GROUP="plugdev"
# OPPO
SUBSYSTEM=="usb", ATTR{idVendor}=="22d9", MODE="0666", GROUP="plugdev"
# Realme
SUBSYSTEM=="usb", ATTR{idVendor}=="2ae5", MODE="0666", GROUP="plugdev"
# Nothing Phone
SUBSYSTEM=="usb", ATTR{idVendor}=="2970", MODE="0666", GROUP="plugdev"
EOF

    sudo chmod a+r /etc/udev/rules.d/51-android.rules
    sudo udevadm control --reload-rules
    sudo udevadm trigger

    # Add user to plugdev group
    sudo usermod -aG plugdev "$USER" 2>/dev/null || true

    log_success "ADB and Fastboot configured with udev rules"
}

#===============================================================================
# IOS DEVELOPMENT (Open-Source Tools)
#===============================================================================

setup_libimobiledevice() {
    log_step "Setting up libimobiledevice (iOS device communication)..."

    sudo apt-get install -y \
        libimobiledevice6 \
        libimobiledevice-utils \
        ideviceinstaller \
        ifuse \
        usbmuxd \
        libplist-utils

    # Start usbmuxd service
    sudo systemctl enable usbmuxd 2>/dev/null || true
    sudo systemctl start usbmuxd 2>/dev/null || true

    log_success "libimobiledevice tools installed"
}

setup_ios_deploy() {
    log_step "Setting up ios-deploy..."

    # Install via npm (requires Node.js)
    if command -v npm &> /dev/null; then
        sudo npm install -g ios-deploy 2>/dev/null || npm install -g ios-deploy
        log_success "ios-deploy installed"
    else
        log_warn "npm not found. Install Node.js first for ios-deploy."
    fi
}

setup_theos() {
    log_step "Setting up Theos (iOS/macOS development toolkit)..."

    export THEOS="$HOME/theos"

    if [ ! -d "$THEOS" ]; then
        # Install dependencies
        sudo apt-get install -y \
            fakeroot \
            dpkg \
            ldid \
            libtool \
            automake \
            perl \
            libxml2-dev \
            libssl-dev \
            libbz2-dev

        # Clone Theos
        git clone --recursive https://github.com/theos/theos.git "$THEOS"

        # Get toolchain
        log_info "Downloading iOS toolchain for Linux..."
        curl -LO https://github.com/sbingner/llvm-project/releases/latest/download/linux-ios-arm64e-clang-toolchain.tar.lzma
        tar -xvf linux-ios-arm64e-clang-toolchain.tar.lzma -C "$THEOS/toolchain"
        rm linux-ios-arm64e-clang-toolchain.tar.lzma

        # Get SDKs
        log_info "Downloading iOS SDKs..."
        curl -LO https://github.com/theos/sdks/archive/master.zip
        unzip -q master.zip -d "$THEOS/sdks"
        mv "$THEOS/sdks/sdks-master"/* "$THEOS/sdks/"
        rm -rf "$THEOS/sdks/sdks-master" master.zip

        log_success "Theos installed at $THEOS"
    else
        log_warn "Theos already installed at $THEOS"
    fi
}

setup_ldid() {
    log_step "Setting up ldid (code signing tool)..."

    if ! command -v ldid &> /dev/null; then
        # Try apt first
        sudo apt-get install -y ldid 2>/dev/null || {
            # Build from source
            log_info "Building ldid from source..."
            git clone https://github.com/ProcursusTeam/ldid.git /tmp/ldid
            cd /tmp/ldid
            make
            sudo make install
            cd -
            rm -rf /tmp/ldid
        }
        log_success "ldid installed"
    else
        log_warn "ldid already installed"
    fi
}

setup_dpkg_deb() {
    log_step "Setting up dpkg for .deb package creation..."

    sudo apt-get install -y \
        dpkg \
        dpkg-dev \
        devscripts \
        debhelper \
        fakeroot

    log_success "dpkg tools installed for iOS .deb creation"
}

#===============================================================================
# SWIFT DEVELOPMENT
#===============================================================================

setup_swift() {
    log_step "Setting up Swift toolchain..."

    # Run dedicated Swift setup script
    if [ -f "/etc/winux/scripts/setup-swift.sh" ]; then
        bash /etc/winux/scripts/setup-swift.sh
    else
        # Inline Swift setup
        SWIFT_VERSION="5.10"
        SWIFT_PLATFORM="ubuntu$(lsb_release -rs)"
        SWIFT_DIR="/opt/swift"

        sudo apt-get install -y \
            binutils \
            libc6-dev \
            libcurl4-openssl-dev \
            libedit2 \
            libgcc-11-dev \
            libpython3-dev \
            libsqlite3-0 \
            libstdc++-11-dev \
            libxml2-dev \
            libz3-dev \
            pkg-config \
            tzdata \
            zlib1g-dev

        if [ ! -d "$SWIFT_DIR" ]; then
            log_info "Downloading Swift ${SWIFT_VERSION}..."
            SWIFT_URL="https://download.swift.org/swift-${SWIFT_VERSION}-release/ubuntu2204/swift-${SWIFT_VERSION}-RELEASE/swift-${SWIFT_VERSION}-RELEASE-ubuntu22.04.tar.gz"

            wget -q --show-progress -O /tmp/swift.tar.gz "$SWIFT_URL"
            sudo mkdir -p "$SWIFT_DIR"
            sudo tar -xzf /tmp/swift.tar.gz -C "$SWIFT_DIR" --strip-components=1
            rm /tmp/swift.tar.gz

            log_success "Swift installed at $SWIFT_DIR"
        fi

        export PATH="$SWIFT_DIR/usr/bin:$PATH"
    fi

    log_success "Swift toolchain configured"
}

setup_sourcekit_lsp() {
    log_step "Setting up SourceKit-LSP..."

    SWIFT_DIR="/opt/swift"

    if [ -f "$SWIFT_DIR/usr/bin/sourcekit-lsp" ]; then
        log_success "SourceKit-LSP is included with Swift toolchain"

        # Create symlink for easy access
        sudo ln -sf "$SWIFT_DIR/usr/bin/sourcekit-lsp" /usr/local/bin/sourcekit-lsp 2>/dev/null || true
    else
        log_warn "SourceKit-LSP not found. It's included in Swift 5.1+"
    fi
}

#===============================================================================
# CROSS-PLATFORM FRAMEWORKS
#===============================================================================

setup_flutter() {
    log_step "Setting up Flutter SDK..."

    FLUTTER_DIR="$HOME/flutter"

    if [ ! -d "$FLUTTER_DIR" ]; then
        log_info "Downloading Flutter SDK..."
        git clone https://github.com/flutter/flutter.git -b stable "$FLUTTER_DIR"

        export PATH="$FLUTTER_DIR/bin:$PATH"

        # Pre-download development binaries
        flutter precache

        # Accept licenses
        yes | flutter doctor --android-licenses 2>/dev/null || true

        log_success "Flutter installed at $FLUTTER_DIR"
    else
        log_warn "Flutter already installed"
        export PATH="$FLUTTER_DIR/bin:$PATH"
    fi

    # Run flutter doctor
    log_info "Running Flutter doctor..."
    flutter doctor
}

setup_dart() {
    log_step "Setting up Dart SDK..."

    # Dart is included with Flutter, but we can also install standalone
    if [ -d "$HOME/flutter" ]; then
        log_success "Dart is included with Flutter"
    else
        # Install standalone Dart
        sudo apt-get install -y apt-transport-https
        wget -qO- https://dl-ssl.google.com/linux/linux_signing_key.pub | sudo gpg --dearmor -o /usr/share/keyrings/dart.gpg
        echo 'deb [signed-by=/usr/share/keyrings/dart.gpg arch=amd64] https://storage.googleapis.com/download.dartlang.org/linux/debian stable main' | sudo tee /etc/apt/sources.list.d/dart_stable.list
        sudo apt-get update
        sudo apt-get install -y dart

        log_success "Dart SDK installed"
    fi
}

setup_react_native() {
    log_step "Setting up React Native..."

    if ! command -v npm &> /dev/null; then
        log_error "npm not found. Please install Node.js first."
        return 1
    fi

    # Install React Native CLI
    sudo npm install -g react-native-cli 2>/dev/null || npm install -g react-native-cli

    # Install Watchman dependencies
    sudo apt-get install -y \
        autoconf \
        automake \
        libtool \
        pkg-config \
        libssl-dev

    # Build and install Watchman
    if ! command -v watchman &> /dev/null; then
        log_info "Building Watchman from source..."
        git clone https://github.com/facebook/watchman.git /tmp/watchman
        cd /tmp/watchman
        git checkout v2024.01.01.00
        ./autogen.sh
        ./configure
        make
        sudo make install
        cd -
        rm -rf /tmp/watchman
    fi

    log_success "React Native CLI installed"
}

setup_expo() {
    log_step "Setting up Expo CLI..."

    if ! command -v npm &> /dev/null; then
        log_error "npm not found. Please install Node.js first."
        return 1
    fi

    # Install Expo CLI
    sudo npm install -g expo-cli eas-cli 2>/dev/null || npm install -g expo-cli eas-cli

    log_success "Expo CLI installed"
}

setup_capacitor() {
    log_step "Setting up Capacitor..."

    if ! command -v npm &> /dev/null; then
        log_error "npm not found. Please install Node.js first."
        return 1
    fi

    # Install Capacitor CLI
    sudo npm install -g @capacitor/cli @capacitor/core 2>/dev/null || \
        npm install -g @capacitor/cli @capacitor/core

    log_success "Capacitor installed"
}

setup_cordova() {
    log_step "Setting up Apache Cordova..."

    if ! command -v npm &> /dev/null; then
        log_error "npm not found. Please install Node.js first."
        return 1
    fi

    # Install Cordova CLI
    sudo npm install -g cordova 2>/dev/null || npm install -g cordova

    log_success "Cordova installed"
}

setup_xcpretty() {
    log_step "Setting up xcpretty..."

    # xcpretty is a Ruby gem
    if command -v gem &> /dev/null; then
        sudo gem install xcpretty 2>/dev/null || gem install xcpretty --user-install
        log_success "xcpretty installed"
    else
        log_warn "Ruby not found. Installing..."
        sudo apt-get install -y ruby ruby-dev
        sudo gem install xcpretty
        log_success "xcpretty installed"
    fi
}

#===============================================================================
# Environment Configuration
#===============================================================================

write_environment_config() {
    log_step "Writing environment configuration..."

    # Create mobile dev profile
    cat > "$HOME/.mobile-dev-env" << 'EOF'
# Winux Mobile Development Environment
# Source this file: source ~/.mobile-dev-env

# Android SDK
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"

# Android PATH
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
export PATH="$ANDROID_HOME/platform-tools:$PATH"
export PATH="$ANDROID_HOME/emulator:$PATH"
export PATH="$ANDROID_HOME/tools:$PATH"
export PATH="$ANDROID_HOME/tools/bin:$PATH"

# Gradle
export GRADLE_HOME="/opt/gradle/gradle-8.5"
export PATH="$GRADLE_HOME/bin:$PATH"

# Java
export JAVA_HOME="/usr/lib/jvm/java-17-openjdk-amd64"
export PATH="$JAVA_HOME/bin:$PATH"

# Flutter
export PATH="$HOME/flutter/bin:$PATH"

# Swift
export PATH="/opt/swift/usr/bin:$PATH"

# Theos (iOS development)
export THEOS="$HOME/theos"
export PATH="$THEOS/bin:$PATH"

# Aliases for mobile development
alias android-emulator='emulator -avd Winux_Phone'
alias android-tablet='emulator -avd Winux_Tablet'
alias adb-devices='adb devices -l'
alias adb-wireless='adb tcpip 5555'
alias flutter-clean='flutter clean && flutter pub get'
alias flutter-run='flutter run'
alias flutter-build-apk='flutter build apk --release'
alias flutter-build-ios='flutter build ios --release'
alias rn-start='npx react-native start'
alias rn-android='npx react-native run-android'
alias rn-ios='npx react-native run-ios'
alias expo-start='expo start'

# iOS device functions
ios-list() {
    idevice_id -l
}

ios-info() {
    ideviceinfo
}

ios-syslog() {
    idevicesyslog
}

ios-screenshot() {
    idevicescreenshot "$1"
}

ios-install() {
    ideviceinstaller -i "$1"
}

# Android helper functions
android-screenshot() {
    adb exec-out screencap -p > "${1:-screenshot.png}"
}

android-record() {
    adb shell screenrecord "/sdcard/${1:-recording.mp4}"
}

android-logcat() {
    adb logcat -v time "$@"
}

android-install() {
    adb install -r "$1"
}

android-uninstall() {
    adb uninstall "$1"
}

# Flutter helper functions
flutter-create-app() {
    flutter create --org com.winux "$1"
}

flutter-analyze() {
    flutter analyze --no-fatal-infos
}

flutter-test() {
    flutter test --coverage
}
EOF

    # Add to bashrc
    if ! grep -q "mobile-dev-env" "$HOME/.bashrc" 2>/dev/null; then
        echo "" >> "$HOME/.bashrc"
        echo "# Winux Mobile Development Environment" >> "$HOME/.bashrc"
        echo '[ -f "$HOME/.mobile-dev-env" ] && source "$HOME/.mobile-dev-env"' >> "$HOME/.bashrc"
    fi

    # Add to zshrc if exists
    if [ -f "$HOME/.zshrc" ]; then
        if ! grep -q "mobile-dev-env" "$HOME/.zshrc" 2>/dev/null; then
            echo "" >> "$HOME/.zshrc"
            echo "# Winux Mobile Development Environment" >> "$HOME/.zshrc"
            echo '[ -f "$HOME/.mobile-dev-env" ] && source "$HOME/.mobile-dev-env"' >> "$HOME/.zshrc"
        fi
    fi

    log_success "Environment configuration written to ~/.mobile-dev-env"
}

#===============================================================================
# Menu and Main
#===============================================================================

show_menu() {
    echo ""
    echo -e "${WHITE}${BOLD}Select components to install:${NC}"
    echo ""
    echo -e "  ${CYAN}1)${NC} Full Mobile Development Stack (recommended)"
    echo -e "  ${CYAN}2)${NC} Android Development Only"
    echo -e "  ${CYAN}3)${NC} iOS Tools Only (libimobiledevice, Theos)"
    echo -e "  ${CYAN}4)${NC} Swift Development Only"
    echo -e "  ${CYAN}5)${NC} Cross-Platform Only (Flutter, React Native, etc)"
    echo -e "  ${CYAN}6)${NC} Custom Selection"
    echo -e "  ${CYAN}0)${NC} Exit"
    echo ""
    read -p "Enter your choice [1-6]: " choice
    echo ""
}

install_android_full() {
    install_dependencies
    setup_android_sdk
    setup_android_ndk
    setup_android_emulator
    setup_gradle
    setup_adb_fastboot
}

install_ios_tools() {
    install_dependencies
    setup_libimobiledevice
    setup_ios_deploy
    setup_theos
    setup_ldid
    setup_dpkg_deb
    setup_xcpretty
}

install_swift_full() {
    install_dependencies
    setup_swift
    setup_sourcekit_lsp
}

install_cross_platform() {
    install_dependencies
    setup_flutter
    setup_dart
    setup_react_native
    setup_expo
    setup_capacitor
    setup_cordova
}

install_full_stack() {
    check_requirements
    install_dependencies

    echo ""
    log_step "Installing Android Development..."
    setup_android_sdk
    setup_android_ndk
    setup_android_emulator
    setup_gradle
    setup_adb_fastboot

    echo ""
    log_step "Installing iOS Tools..."
    setup_libimobiledevice
    setup_ios_deploy
    setup_theos
    setup_ldid
    setup_dpkg_deb

    echo ""
    log_step "Installing Swift..."
    setup_swift
    setup_sourcekit_lsp

    echo ""
    log_step "Installing Cross-Platform Frameworks..."
    setup_flutter
    setup_dart
    setup_react_native
    setup_expo
    setup_capacitor
    setup_cordova
    setup_xcpretty

    write_environment_config
}

custom_selection() {
    echo -e "${WHITE}Custom Installation${NC}"
    echo ""

    read -p "Install Android SDK? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_android_sdk

    read -p "Install Android NDK? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_android_ndk

    read -p "Install Android Emulator? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_android_emulator

    read -p "Install Gradle? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_gradle

    read -p "Install libimobiledevice (iOS tools)? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_libimobiledevice

    read -p "Install Theos (iOS development)? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_theos

    read -p "Install Swift? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_swift

    read -p "Install Flutter? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_flutter

    read -p "Install React Native? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_react_native

    read -p "Install Expo? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_expo

    read -p "Install Capacitor? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_capacitor

    read -p "Install Cordova? [Y/n]: " ans
    [[ ! "$ans" =~ ^[Nn]$ ]] && setup_cordova

    write_environment_config
}

show_summary() {
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}${BOLD}    Mobile Development Environment Setup Complete!${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${WHITE}Installed Components:${NC}"

    command -v sdkmanager &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Android SDK"
    [ -d "$HOME/Android/Sdk/ndk" ] && echo -e "  ${GREEN}[OK]${NC} Android NDK"
    command -v emulator &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Android Emulator"
    command -v gradle &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Gradle"
    command -v adb &>/dev/null && echo -e "  ${GREEN}[OK]${NC} ADB & Fastboot"
    command -v ideviceinfo &>/dev/null && echo -e "  ${GREEN}[OK]${NC} libimobiledevice"
    [ -d "$HOME/theos" ] && echo -e "  ${GREEN}[OK]${NC} Theos"
    command -v swift &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Swift"
    command -v flutter &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Flutter"
    command -v dart &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Dart"
    command -v react-native &>/dev/null && echo -e "  ${GREEN}[OK]${NC} React Native"
    command -v expo &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Expo CLI"
    command -v cap &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Capacitor"
    command -v cordova &>/dev/null && echo -e "  ${GREEN}[OK]${NC} Cordova"

    echo ""
    echo -e "${WHITE}Quick Start Commands:${NC}"
    echo -e "  ${CYAN}flutter create myapp${NC}     - Create new Flutter app"
    echo -e "  ${CYAN}npx react-native init${NC}   - Create new React Native app"
    echo -e "  ${CYAN}expo init myapp${NC}          - Create new Expo app"
    echo -e "  ${CYAN}android-emulator${NC}         - Start Android emulator"
    echo -e "  ${CYAN}flutter doctor${NC}           - Check Flutter setup"
    echo ""
    echo -e "${YELLOW}Please restart your terminal or run:${NC}"
    echo -e "  ${CYAN}source ~/.mobile-dev-env${NC}"
    echo ""
}

main() {
    show_logo

    # Check for command line arguments
    case "${1:-}" in
        --full)
            install_full_stack
            show_summary
            exit 0
            ;;
        --android)
            install_android_full
            write_environment_config
            show_summary
            exit 0
            ;;
        --ios)
            install_ios_tools
            write_environment_config
            show_summary
            exit 0
            ;;
        --swift)
            install_swift_full
            write_environment_config
            show_summary
            exit 0
            ;;
        --cross-platform)
            install_cross_platform
            write_environment_config
            show_summary
            exit 0
            ;;
        --help|-h)
            echo "Usage: $0 [OPTION]"
            echo ""
            echo "Options:"
            echo "  --full            Install full mobile development stack"
            echo "  --android         Install Android development only"
            echo "  --ios             Install iOS tools only"
            echo "  --swift           Install Swift only"
            echo "  --cross-platform  Install cross-platform frameworks only"
            echo "  --help            Show this help message"
            echo ""
            echo "Without options, an interactive menu will be shown."
            exit 0
            ;;
    esac

    # Interactive mode
    show_menu

    case $choice in
        1)
            install_full_stack
            ;;
        2)
            install_android_full
            write_environment_config
            ;;
        3)
            install_ios_tools
            write_environment_config
            ;;
        4)
            install_swift_full
            write_environment_config
            ;;
        5)
            install_cross_platform
            write_environment_config
            ;;
        6)
            install_dependencies
            custom_selection
            ;;
        0)
            echo "Exiting..."
            exit 0
            ;;
        *)
            log_error "Invalid choice"
            exit 1
            ;;
    esac

    show_summary
}

main "$@"
