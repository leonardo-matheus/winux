#!/bin/bash
#===============================================================================
# Winux OS - Android Studio Installation
# Downloads and configures Android Studio with SDK and Emulator
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
    echo -e "${WHITE}${BOLD}       Android Studio Installation${NC}"
    echo -e "${MAGENTA}     Official IDE for Android Development${NC}"
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo ""
}

#-------------------------------------------------------------------------------
# Configuration
#-------------------------------------------------------------------------------
ANDROID_STUDIO_VERSION="2024.2.1.11"  # Ladybug
ANDROID_STUDIO_DIR="/opt/android-studio"
ANDROID_HOME="$HOME/Android/Sdk"
DESKTOP_FILE="/usr/share/applications/android-studio.desktop"

# Installation method
INSTALL_METHOD="${INSTALL_METHOD:-tar}"  # tar or snap

#-------------------------------------------------------------------------------
# Check System Requirements
#-------------------------------------------------------------------------------
check_requirements() {
    log_step "Checking system requirements..."

    # Check architecture
    ARCH=$(uname -m)
    if [ "$ARCH" != "x86_64" ]; then
        log_error "Android Studio requires x86_64 architecture. Detected: $ARCH"
        exit 1
    fi

    # Check available disk space (need at least 8GB)
    AVAILABLE_SPACE=$(df -BG "$HOME" | awk 'NR==2 {print $4}' | tr -d 'G')
    if [ "$AVAILABLE_SPACE" -lt 8 ]; then
        log_error "Insufficient disk space: ${AVAILABLE_SPACE}GB. Need at least 8GB."
        exit 1
    fi
    log_success "Disk space: ${AVAILABLE_SPACE}GB available"

    # Check RAM (recommend 8GB+)
    TOTAL_RAM=$(free -g | awk '/^Mem:/{print $2}')
    if [ "$TOTAL_RAM" -lt 8 ]; then
        log_warn "Low RAM: ${TOTAL_RAM}GB. Android Studio recommends 8GB+."
    else
        log_success "RAM: ${TOTAL_RAM}GB"
    fi

    # Check KVM support
    if [ -e /dev/kvm ] && [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
        log_success "KVM acceleration available"
        KVM_AVAILABLE=true
    else
        log_warn "KVM not available or not accessible. Emulator will be slower."
        log_info "To enable KVM: sudo usermod -aG kvm \$USER && logout"
        KVM_AVAILABLE=false
    fi

    log_success "System requirements check complete"
}

#-------------------------------------------------------------------------------
# Install Dependencies
#-------------------------------------------------------------------------------
install_dependencies() {
    log_step "Installing dependencies..."

    sudo apt-get update

    # Core dependencies
    sudo apt-get install -y \
        openjdk-17-jdk \
        openjdk-17-jdk-headless \
        wget \
        unzip \
        git \
        lib32z1 \
        lib32ncurses6 \
        lib32stdc++6 \
        libbz2-1.0

    # 32-bit libraries for Android tools
    sudo dpkg --add-architecture i386 2>/dev/null || true
    sudo apt-get update
    sudo apt-get install -y \
        libc6:i386 \
        libncurses5:i386 \
        libstdc++6:i386 \
        zlib1g:i386 2>/dev/null || true

    # KVM/QEMU for emulator
    sudo apt-get install -y \
        qemu-kvm \
        libvirt-daemon-system \
        libvirt-clients \
        bridge-utils \
        cpu-checker 2>/dev/null || true

    # Graphics libraries
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
        libgdk-pixbuf2.0-0 \
        libxrandr2 \
        libxss1 \
        libgconf-2-4 2>/dev/null || true

    log_success "Dependencies installed"
}

#-------------------------------------------------------------------------------
# Setup KVM Acceleration
#-------------------------------------------------------------------------------
setup_kvm() {
    log_step "Setting up KVM acceleration..."

    # Check if KVM module is loaded
    if ! lsmod | grep -q kvm; then
        log_info "Loading KVM module..."
        sudo modprobe kvm 2>/dev/null || true
        sudo modprobe kvm_intel 2>/dev/null || sudo modprobe kvm_amd 2>/dev/null || true
    fi

    # Add user to kvm group
    if ! groups "$USER" | grep -q kvm; then
        log_info "Adding user to kvm group..."
        sudo usermod -aG kvm "$USER"
        log_warn "You need to logout and login again for KVM group membership"
    fi

    # Set permissions on /dev/kvm
    if [ -e /dev/kvm ]; then
        sudo chmod 666 /dev/kvm 2>/dev/null || true

        # Create udev rule for persistent permissions
        sudo tee /etc/udev/rules.d/60-kvm.rules > /dev/null << 'EOF'
KERNEL=="kvm", GROUP="kvm", MODE="0666"
EOF
        sudo udevadm control --reload-rules 2>/dev/null || true
        sudo udevadm trigger 2>/dev/null || true
    fi

    # Verify KVM
    if command -v kvm-ok &> /dev/null; then
        kvm-ok && log_success "KVM acceleration is working" || \
            log_warn "KVM acceleration may not work properly"
    fi

    log_success "KVM setup complete"
}

#-------------------------------------------------------------------------------
# Install via Snap
#-------------------------------------------------------------------------------
install_via_snap() {
    log_step "Installing Android Studio via Snap..."

    # Check if snap is available
    if ! command -v snap &> /dev/null; then
        log_error "Snap is not installed. Installing..."
        sudo apt-get install -y snapd
        sudo systemctl enable --now snapd.socket
        sudo ln -s /var/lib/snapd/snap /snap 2>/dev/null || true
    fi

    # Install Android Studio
    sudo snap install android-studio --classic

    log_success "Android Studio installed via Snap"

    # Note: Snap handles desktop entry automatically
    ANDROID_STUDIO_DIR="/snap/android-studio/current/android-studio"
}

#-------------------------------------------------------------------------------
# Install via Tar (Manual)
#-------------------------------------------------------------------------------
install_via_tar() {
    log_step "Installing Android Studio from tarball..."

    # Get latest download URL
    # Note: This URL pattern may need updating
    DOWNLOAD_URL="https://redirector.gvt1.com/edgedl/android/studio/ide-zips/${ANDROID_STUDIO_VERSION}/android-studio-${ANDROID_STUDIO_VERSION}-linux.tar.gz"

    # Alternative: Use a fixed URL that redirects to latest
    DOWNLOAD_PAGE="https://developer.android.com/studio"

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Download Android Studio
    log_info "Downloading Android Studio..."
    log_info "URL: $DOWNLOAD_URL"

    # Try direct download first
    if ! wget -q --show-progress -O android-studio.tar.gz "$DOWNLOAD_URL" 2>/dev/null; then
        # Fallback: Try to get the latest URL from the download page
        log_warn "Direct download failed. Trying alternative method..."

        # Get download URL from Google's API
        LATEST_URL=$(curl -sL "https://dl.google.com/android/studio/patches/updates.xml" 2>/dev/null | \
            grep -oP 'https://[^"]+android-studio-[^"]+linux\.tar\.gz' | head -1) || true

        if [ -n "$LATEST_URL" ]; then
            wget -q --show-progress -O android-studio.tar.gz "$LATEST_URL"
        else
            log_error "Could not download Android Studio automatically."
            echo ""
            echo "Please download manually from: $DOWNLOAD_PAGE"
            echo "Then run: $0 --install-from /path/to/android-studio-*-linux.tar.gz"
            exit 1
        fi
    fi

    log_success "Download complete"

    # Remove existing installation
    if [ -d "$ANDROID_STUDIO_DIR" ]; then
        log_warn "Removing existing installation..."
        sudo rm -rf "$ANDROID_STUDIO_DIR"
    fi

    # Extract
    log_info "Extracting Android Studio..."
    sudo mkdir -p /opt
    sudo tar -xzf android-studio.tar.gz -C /opt

    # Rename if needed
    if [ -d "/opt/android-studio-*" ]; then
        sudo mv /opt/android-studio-* "$ANDROID_STUDIO_DIR"
    fi

    # Clean up
    cd -
    rm -rf "$TEMP_DIR"

    log_success "Android Studio installed to $ANDROID_STUDIO_DIR"
}

#-------------------------------------------------------------------------------
# Install from Local File
#-------------------------------------------------------------------------------
install_from_file() {
    local file="$1"

    log_step "Installing Android Studio from local file..."

    if [ ! -f "$file" ]; then
        log_error "File not found: $file"
        exit 1
    fi

    # Remove existing installation
    if [ -d "$ANDROID_STUDIO_DIR" ]; then
        log_warn "Removing existing installation..."
        sudo rm -rf "$ANDROID_STUDIO_DIR"
    fi

    # Extract
    log_info "Extracting Android Studio..."
    sudo mkdir -p /opt
    sudo tar -xzf "$file" -C /opt

    # Rename if needed
    for dir in /opt/android-studio-*/; do
        if [ -d "$dir" ] && [ "$dir" != "$ANDROID_STUDIO_DIR/" ]; then
            sudo mv "$dir" "$ANDROID_STUDIO_DIR"
            break
        fi
    done

    log_success "Android Studio installed to $ANDROID_STUDIO_DIR"
}

#-------------------------------------------------------------------------------
# Create Desktop Entry
#-------------------------------------------------------------------------------
create_desktop_entry() {
    log_step "Creating desktop entry..."

    # Determine installation path
    local studio_bin=""
    local studio_icon=""

    if [ -d "/snap/android-studio" ]; then
        studio_bin="/snap/bin/android-studio"
        studio_icon="/snap/android-studio/current/android-studio/bin/studio.svg"
    elif [ -d "$ANDROID_STUDIO_DIR" ]; then
        studio_bin="$ANDROID_STUDIO_DIR/bin/studio.sh"
        studio_icon="$ANDROID_STUDIO_DIR/bin/studio.svg"
    else
        log_error "Android Studio installation not found"
        return 1
    fi

    # Create desktop entry
    sudo tee "$DESKTOP_FILE" > /dev/null << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Android Studio
GenericName=Android IDE
Comment=The Official IDE for Android Development
Icon=$studio_icon
Exec="$studio_bin" %f
Terminal=false
Categories=Development;IDE;
StartupNotify=true
StartupWMClass=jetbrains-studio
MimeType=application/x-android-studio-project;
Keywords=android;studio;ide;development;mobile;
Actions=NewProject;OpenProject;

[Desktop Action NewProject]
Name=New Project
Exec="$studio_bin" --new-project

[Desktop Action OpenProject]
Name=Open Project
Exec="$studio_bin"
EOF

    # Update desktop database
    sudo update-desktop-database 2>/dev/null || true

    log_success "Desktop entry created"
}

#-------------------------------------------------------------------------------
# Configure Android SDK
#-------------------------------------------------------------------------------
configure_sdk() {
    log_step "Configuring Android SDK..."

    # Create SDK directory
    mkdir -p "$ANDROID_HOME"

    # Set environment variables
    export ANDROID_HOME
    export ANDROID_SDK_ROOT="$ANDROID_HOME"

    # Check if command-line tools are installed
    if [ ! -d "$ANDROID_HOME/cmdline-tools/latest" ]; then
        log_info "Downloading Android command-line tools..."

        CMDLINE_TOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
        wget -q --show-progress -O /tmp/cmdline-tools.zip "$CMDLINE_TOOLS_URL"

        mkdir -p "$ANDROID_HOME/cmdline-tools"
        unzip -q /tmp/cmdline-tools.zip -d "$ANDROID_HOME/cmdline-tools"
        mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
        rm /tmp/cmdline-tools.zip

        log_success "Command-line tools installed"
    fi

    # Add to PATH
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"

    # Accept licenses
    log_info "Accepting Android SDK licenses..."
    yes | sdkmanager --licenses 2>/dev/null || true

    # Install essential SDK components
    log_info "Installing SDK components (this may take a while)..."

    sdkmanager --install \
        "platform-tools" \
        "build-tools;34.0.0" \
        "platforms;android-34" \
        "emulator" \
        "system-images;android-34;google_apis;x86_64" 2>/dev/null || {
            log_warn "Some SDK components may need to be installed from Android Studio"
        }

    log_success "Android SDK configured at $ANDROID_HOME"
}

#-------------------------------------------------------------------------------
# Configure Emulator
#-------------------------------------------------------------------------------
configure_emulator() {
    log_step "Configuring Android Emulator..."

    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/emulator:$PATH"

    # Check if avdmanager exists
    if ! command -v avdmanager &> /dev/null; then
        log_warn "avdmanager not found. Configure emulator from Android Studio."
        return
    fi

    # Create default AVD
    if ! avdmanager list avd 2>/dev/null | grep -q "Pixel_6_API_34"; then
        log_info "Creating default AVD..."

        echo "no" | avdmanager create avd \
            -n "Pixel_6_API_34" \
            -k "system-images;android-34;google_apis;x86_64" \
            -d "pixel_6" \
            --force 2>/dev/null || {
                log_warn "Could not create AVD. Create it from Android Studio."
                return
            }

        log_success "Default AVD 'Pixel_6_API_34' created"
    else
        log_warn "Default AVD already exists"
    fi

    # Configure emulator settings for better performance
    mkdir -p "$HOME/.android"
    cat > "$HOME/.android/advancedFeatures.ini" << 'EOF'
# Android Emulator Advanced Features
Vulkan = on
GLDirectMem = on
GrallocSync = on
EOF

    log_success "Emulator configured"
}

#-------------------------------------------------------------------------------
# Configure ADB Rules
#-------------------------------------------------------------------------------
configure_adb() {
    log_step "Configuring ADB udev rules..."

    # Create comprehensive udev rules for Android devices
    sudo tee /etc/udev/rules.d/51-android.rules > /dev/null << 'EOF'
# Android device udev rules for adb/fastboot
# Google
SUBSYSTEM=="usb", ATTR{idVendor}=="18d1", MODE="0666", GROUP="plugdev", SYMLINK+="android%n"
# Samsung
SUBSYSTEM=="usb", ATTR{idVendor}=="04e8", MODE="0666", GROUP="plugdev"
# Xiaomi
SUBSYSTEM=="usb", ATTR{idVendor}=="2717", MODE="0666", GROUP="plugdev"
SUBSYSTEM=="usb", ATTR{idVendor}=="05c6", MODE="0666", GROUP="plugdev"
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
# OPPO/Realme
SUBSYSTEM=="usb", ATTR{idVendor}=="22d9", MODE="0666", GROUP="plugdev"
SUBSYSTEM=="usb", ATTR{idVendor}=="2ae5", MODE="0666", GROUP="plugdev"
# Vivo
SUBSYSTEM=="usb", ATTR{idVendor}=="2d95", MODE="0666", GROUP="plugdev"
# Nothing Phone
SUBSYSTEM=="usb", ATTR{idVendor}=="2970", MODE="0666", GROUP="plugdev"
# Generic Qualcomm
SUBSYSTEM=="usb", ATTR{idVendor}=="05c6", MODE="0666", GROUP="plugdev"
# Generic MediaTek
SUBSYSTEM=="usb", ATTR{idVendor}=="0e8d", MODE="0666", GROUP="plugdev"
EOF

    # Set permissions and reload rules
    sudo chmod a+r /etc/udev/rules.d/51-android.rules
    sudo udevadm control --reload-rules
    sudo udevadm trigger

    # Add user to plugdev group
    sudo usermod -aG plugdev "$USER" 2>/dev/null || true

    log_success "ADB udev rules configured"
}

#-------------------------------------------------------------------------------
# Write Environment Configuration
#-------------------------------------------------------------------------------
write_environment() {
    log_step "Writing environment configuration..."

    # Create Android environment file
    cat > "$HOME/.android-env" << EOF
# Android Development Environment
# Generated by Winux OS setup-android-studio.sh

# Android SDK
export ANDROID_HOME="\$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="\$ANDROID_HOME"

# Java
export JAVA_HOME="/usr/lib/jvm/java-17-openjdk-amd64"

# PATH additions
export PATH="\$ANDROID_HOME/cmdline-tools/latest/bin:\$PATH"
export PATH="\$ANDROID_HOME/platform-tools:\$PATH"
export PATH="\$ANDROID_HOME/emulator:\$PATH"
export PATH="\$ANDROID_HOME/tools:\$PATH"
export PATH="\$ANDROID_HOME/tools/bin:\$PATH"
export PATH="\$JAVA_HOME/bin:\$PATH"

# Android aliases
alias android-studio="$ANDROID_STUDIO_DIR/bin/studio.sh"
alias adb-devices='adb devices -l'
alias adb-wireless='adb tcpip 5555'
alias adb-connect='adb connect'
alias adb-screenshot='adb exec-out screencap -p'
alias adb-record='adb shell screenrecord'
alias adb-logcat='adb logcat -v time'
alias adb-install='adb install -r'
alias avd-list='avdmanager list avd'
alias avd-create='avdmanager create avd'
alias emulator-list='emulator -list-avds'
alias emulator-start='emulator -avd'

# Emulator function
android-emulator() {
    local avd="\${1:-Pixel_6_API_34}"
    emulator -avd "\$avd" -gpu auto &
}

# Quick connect to device over WiFi
adb-wifi() {
    local ip="\$1"
    if [ -z "\$ip" ]; then
        echo "Usage: adb-wifi <device-ip>"
        echo "First connect via USB and run: adb tcpip 5555"
        return 1
    fi
    adb connect "\$ip:5555"
}

# Take screenshot and save to current directory
android-screenshot() {
    local filename="\${1:-screenshot-\$(date +%Y%m%d-%H%M%S).png}"
    adb exec-out screencap -p > "\$filename"
    echo "Screenshot saved: \$filename"
}

# Record screen
android-record() {
    local filename="\${1:-recording-\$(date +%Y%m%d-%H%M%S).mp4}"
    echo "Recording... Press Ctrl+C to stop"
    adb shell screenrecord "/sdcard/\$filename"
    adb pull "/sdcard/\$filename" .
    adb shell rm "/sdcard/\$filename"
    echo "Recording saved: \$filename"
}

# Install APK and keep data
android-reinstall() {
    adb install -r -d "\$1"
}

# Clear app data
android-clear() {
    adb shell pm clear "\$1"
}

# Uninstall but keep data
android-uninstall() {
    adb uninstall -k "\$1"
}

# Show device info
android-info() {
    echo "Device Information:"
    echo "  Model:    \$(adb shell getprop ro.product.model)"
    echo "  Android:  \$(adb shell getprop ro.build.version.release)"
    echo "  SDK:      \$(adb shell getprop ro.build.version.sdk)"
    echo "  Serial:   \$(adb get-serialno)"
}
EOF

    # Add to bashrc
    if ! grep -q ".android-env" "$HOME/.bashrc" 2>/dev/null; then
        echo "" >> "$HOME/.bashrc"
        echo "# Android Development Environment" >> "$HOME/.bashrc"
        echo '[ -f "$HOME/.android-env" ] && source "$HOME/.android-env"' >> "$HOME/.bashrc"
    fi

    # Add to zshrc if exists
    if [ -f "$HOME/.zshrc" ]; then
        if ! grep -q ".android-env" "$HOME/.zshrc" 2>/dev/null; then
            echo "" >> "$HOME/.zshrc"
            echo "# Android Development Environment" >> "$HOME/.zshrc"
            echo '[ -f "$HOME/.android-env" ] && source "$HOME/.android-env"' >> "$HOME/.zshrc"
        fi
    fi

    log_success "Environment configuration written"
}

#-------------------------------------------------------------------------------
# Show Summary
#-------------------------------------------------------------------------------
show_summary() {
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}${BOLD}      Android Studio Installation Complete!${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${WHITE}Installation Details:${NC}"
    echo -e "  Android Studio:   ${CYAN}$ANDROID_STUDIO_DIR${NC}"
    echo -e "  Android SDK:      ${CYAN}$ANDROID_HOME${NC}"
    echo -e "  Java Home:        ${CYAN}/usr/lib/jvm/java-17-openjdk-amd64${NC}"
    echo ""
    echo -e "${WHITE}Installed Components:${NC}"
    [ -d "$ANDROID_STUDIO_DIR" ] && echo -e "  ${GREEN}[OK]${NC} Android Studio IDE"
    [ -d "$ANDROID_HOME/platform-tools" ] && echo -e "  ${GREEN}[OK]${NC} Platform Tools (adb, fastboot)"
    [ -d "$ANDROID_HOME/build-tools" ] && echo -e "  ${GREEN}[OK]${NC} Build Tools"
    [ -d "$ANDROID_HOME/emulator" ] && echo -e "  ${GREEN}[OK]${NC} Android Emulator"
    [ "$KVM_AVAILABLE" = true ] && echo -e "  ${GREEN}[OK]${NC} KVM Acceleration"
    echo ""
    echo -e "${WHITE}Quick Start:${NC}"
    echo -e "  ${CYAN}Launch:${NC}              Click 'Android Studio' in applications menu"
    echo -e "  ${CYAN}Command line:${NC}        android-studio"
    echo -e "  ${CYAN}List devices:${NC}        adb devices"
    echo -e "  ${CYAN}Start emulator:${NC}      android-emulator"
    echo ""
    echo -e "${WHITE}First Launch:${NC}"
    echo "  1. Android Studio will download additional components"
    echo "  2. Complete the setup wizard"
    echo "  3. Create your first project or import existing one"
    echo ""
    if [ "$KVM_AVAILABLE" != true ]; then
        echo -e "${YELLOW}Note: KVM acceleration is not enabled.${NC}"
        echo "  To enable faster emulation:"
        echo "  1. sudo usermod -aG kvm \$USER"
        echo "  2. Logout and login again"
        echo ""
    fi
    echo -e "${YELLOW}Apply environment changes:${NC}"
    echo -e "  ${CYAN}source ~/.android-env${NC}"
    echo ""
}

#-------------------------------------------------------------------------------
# Uninstall
#-------------------------------------------------------------------------------
uninstall() {
    log_step "Uninstalling Android Studio..."

    # Remove via snap if installed that way
    if snap list android-studio &>/dev/null; then
        sudo snap remove android-studio
        log_success "Removed Android Studio snap"
    fi

    # Remove tar installation
    if [ -d "$ANDROID_STUDIO_DIR" ]; then
        sudo rm -rf "$ANDROID_STUDIO_DIR"
        log_success "Removed $ANDROID_STUDIO_DIR"
    fi

    # Remove desktop entry
    sudo rm -f "$DESKTOP_FILE" 2>/dev/null || true
    sudo update-desktop-database 2>/dev/null || true

    # Ask about SDK removal
    if [ -d "$ANDROID_HOME" ]; then
        read -p "Remove Android SDK ($ANDROID_HOME)? [y/N]: " remove_sdk
        if [[ "$remove_sdk" =~ ^[Yy]$ ]]; then
            rm -rf "$ANDROID_HOME"
            log_success "Removed Android SDK"
        fi
    fi

    # Remove Android Studio config
    read -p "Remove Android Studio configuration (~/.config/Google)? [y/N]: " remove_config
    if [[ "$remove_config" =~ ^[Yy]$ ]]; then
        rm -rf "$HOME/.config/Google/AndroidStudio*" 2>/dev/null || true
        rm -rf "$HOME/.local/share/Google/AndroidStudio*" 2>/dev/null || true
        rm -rf "$HOME/.cache/Google/AndroidStudio*" 2>/dev/null || true
        log_success "Removed Android Studio configuration"
    fi

    log_success "Android Studio uninstalled"
}

#-------------------------------------------------------------------------------
# Main
#-------------------------------------------------------------------------------
main() {
    show_logo

    case "${1:-}" in
        --snap)
            INSTALL_METHOD="snap"
            ;;
        --tar)
            INSTALL_METHOD="tar"
            ;;
        --install-from)
            if [ -z "${2:-}" ]; then
                log_error "Please provide path to Android Studio archive"
                exit 1
            fi
            check_requirements
            install_dependencies
            setup_kvm
            install_from_file "$2"
            create_desktop_entry
            configure_sdk
            configure_emulator
            configure_adb
            write_environment
            show_summary
            exit 0
            ;;
        --uninstall)
            uninstall
            exit 0
            ;;
        --help|-h)
            echo "Usage: $0 [OPTION]"
            echo ""
            echo "Options:"
            echo "  --snap              Install via Snap (recommended)"
            echo "  --tar               Install via tarball"
            echo "  --install-from FILE Install from local archive"
            echo "  --uninstall         Remove Android Studio"
            echo "  --help              Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                  Interactive installation"
            echo "  $0 --snap           Install via Snap"
            echo "  $0 --install-from /path/to/android-studio.tar.gz"
            exit 0
            ;;
    esac

    # Check requirements
    check_requirements

    # Choose installation method
    if [ -z "${1:-}" ]; then
        echo -e "${WHITE}Select installation method:${NC}"
        echo ""
        echo -e "  ${CYAN}1)${NC} Snap (recommended, auto-updates)"
        echo -e "  ${CYAN}2)${NC} Tarball (manual updates)"
        echo ""
        read -p "Enter choice [1-2]: " method_choice

        case $method_choice in
            1) INSTALL_METHOD="snap" ;;
            2) INSTALL_METHOD="tar" ;;
            *) INSTALL_METHOD="snap" ;;
        esac
    fi

    echo ""

    # Install dependencies
    install_dependencies

    # Setup KVM
    setup_kvm

    # Install Android Studio
    if [ "$INSTALL_METHOD" = "snap" ]; then
        install_via_snap
    else
        install_via_tar
    fi

    # Create desktop entry (for tar installation)
    if [ "$INSTALL_METHOD" = "tar" ]; then
        create_desktop_entry
    fi

    # Configure SDK, emulator, and ADB
    configure_sdk
    configure_emulator
    configure_adb

    # Write environment configuration
    write_environment

    # Show summary
    show_summary
}

main "$@"
