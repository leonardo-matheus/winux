#!/bin/bash
#===============================================================================
# Winux OS AMD Driver Installation Script
# Complete installation for AMD GPUs (AMDGPU/Radeon) with gaming optimizations
#===============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
AMDGPU_PRO_VERSION="${AMDGPU_PRO_VERSION:-latest}"
AMD_DOWNLOAD_URL="https://repo.radeon.com"
VULKAN_ICD_DIR="/usr/share/vulkan/icd.d"
XORG_CONF_DIR="/etc/X11/xorg.conf.d"
MESA_VERSION="${MESA_VERSION:-latest}"

#===============================================================================
# Functions
#===============================================================================

print_banner() {
    echo -e "${RED}"
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║                Winux OS AMD Driver Installer                      ║"
    echo "║                  Gaming Optimized Setup                           ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "This script must be run as root"
        exit 1
    fi
}

detect_gpu() {
    log_info "Detecting AMD GPU..."

    if ! lspci | grep -iE "AMD|ATI|Radeon" | grep -iE "VGA|3D|Display" &> /dev/null; then
        log_error "No AMD GPU detected"
        exit 1
    fi

    GPU_INFO=$(lspci | grep -iE "AMD|ATI|Radeon" | grep -iE "VGA|3D|Display" | head -1)
    log_success "Found: $GPU_INFO"

    # Detect GPU generation
    if echo "$GPU_INFO" | grep -qiE "RX 7[0-9]{3}|Radeon 7[0-9]{3}|Navi 3"; then
        GPU_GEN="rdna3"
        DRIVER_TYPE="amdgpu"
    elif echo "$GPU_INFO" | grep -qiE "RX 6[0-9]{3}|Radeon 6[0-9]{3}|Navi 2"; then
        GPU_GEN="rdna2"
        DRIVER_TYPE="amdgpu"
    elif echo "$GPU_INFO" | grep -qiE "RX 5[0-9]{3}|Navi"; then
        GPU_GEN="rdna1"
        DRIVER_TYPE="amdgpu"
    elif echo "$GPU_INFO" | grep -qiE "Vega|RX 5[0-8][0-9]|RX 4[0-9]{2}|RX 3[0-9]{2}"; then
        GPU_GEN="gcn5"
        DRIVER_TYPE="amdgpu"
    elif echo "$GPU_INFO" | grep -qiE "Polaris|RX 5[0-9][0-9]|RX 4[0-9][0-9]"; then
        GPU_GEN="gcn4"
        DRIVER_TYPE="amdgpu"
    elif echo "$GPU_INFO" | grep -qiE "Fury|R9 [23][0-9][0-9]|R7 [23][0-9][0-9]"; then
        GPU_GEN="gcn3"
        DRIVER_TYPE="amdgpu"
    else
        GPU_GEN="legacy"
        DRIVER_TYPE="radeon"
    fi

    log_info "GPU Generation: $GPU_GEN (driver: $DRIVER_TYPE)"
}

detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$ID
        DISTRO_VERSION=$VERSION_ID
        DISTRO_CODENAME=$VERSION_CODENAME
    else
        DISTRO="unknown"
    fi
    log_info "Detected distribution: $DISTRO $DISTRO_VERSION"
}

install_dependencies() {
    log_info "Installing dependencies..."

    case $DISTRO in
        ubuntu|debian|linuxmint)
            apt-get update
            apt-get install -y \
                linux-headers-$(uname -r) \
                build-essential \
                dkms \
                libdrm-dev \
                libdrm-amdgpu1 \
                xserver-xorg-video-amdgpu \
                mesa-vulkan-drivers \
                vulkan-tools \
                libvulkan1 \
                mesa-utils \
                vainfo \
                vdpauinfo \
                radeontop
            ;;
        fedora|rhel|centos)
            dnf install -y \
                kernel-devel \
                kernel-headers \
                dkms \
                mesa-dri-drivers \
                mesa-vulkan-drivers \
                vulkan-loader \
                vulkan-tools \
                xorg-x11-drv-amdgpu \
                mesa-libGL \
                mesa-libEGL \
                libva-utils \
                vdpauinfo \
                radeontop
            ;;
        arch|manjaro|endeavouros)
            pacman -S --needed --noconfirm \
                linux-headers \
                mesa \
                lib32-mesa \
                xf86-video-amdgpu \
                vulkan-radeon \
                lib32-vulkan-radeon \
                vulkan-icd-loader \
                lib32-vulkan-icd-loader \
                vulkan-tools \
                libva-mesa-driver \
                lib32-libva-mesa-driver \
                mesa-vdpau \
                lib32-mesa-vdpau \
                radeontop
            ;;
    esac

    log_success "Dependencies installed"
}

install_mesa_git() {
    log_info "Installing latest Mesa drivers..."

    case $DISTRO in
        ubuntu|debian|linuxmint)
            # Add Kisak Mesa PPA for latest drivers
            add-apt-repository -y ppa:kisak/kisak-mesa 2>/dev/null || true
            apt-get update
            apt-get upgrade -y
            ;;
        fedora)
            # Fedora usually has recent Mesa
            dnf upgrade -y mesa*
            ;;
        arch|manjaro|endeavouros)
            # Arch has latest Mesa in repos
            pacman -Syu --noconfirm mesa lib32-mesa
            ;;
    esac

    log_success "Mesa drivers updated"
}

install_amdgpu_pro() {
    log_info "Installing AMDGPU-PRO components..."

    # Note: AMDGPU-PRO is mainly useful for OpenCL/ROCm workloads
    # For gaming, open-source Mesa drivers are recommended

    case $DISTRO in
        ubuntu)
            log_info "Adding AMD ROCm repository..."

            # Add AMD GPG key
            wget -q -O - https://repo.radeon.com/rocm/rocm.gpg.key | apt-key add -

            # Add repository
            echo "deb [arch=amd64] https://repo.radeon.com/amdgpu/${AMDGPU_PRO_VERSION}/ubuntu ${DISTRO_CODENAME} main" > /etc/apt/sources.list.d/amdgpu.list
            echo "deb [arch=amd64] https://repo.radeon.com/rocm/apt/${AMDGPU_PRO_VERSION} ${DISTRO_CODENAME} main" > /etc/apt/sources.list.d/rocm.list

            apt-get update

            # Install only OpenCL components (keep Mesa for gaming)
            apt-get install -y rocm-opencl-runtime 2>/dev/null || log_warning "ROCm OpenCL not available"
            ;;
        fedora|rhel|centos)
            log_info "Adding AMD ROCm repository..."

            cat > /etc/yum.repos.d/amdgpu.repo << EOF
[amdgpu]
name=amdgpu
baseurl=https://repo.radeon.com/amdgpu/${AMDGPU_PRO_VERSION}/rhel/\$releasever/main/x86_64
enabled=1
gpgcheck=1
gpgkey=https://repo.radeon.com/rocm/rocm.gpg.key
EOF

            dnf install -y rocm-opencl-runtime 2>/dev/null || log_warning "ROCm OpenCL not available"
            ;;
    esac

    log_success "AMDGPU-PRO components installed"
}

configure_kernel_module() {
    log_info "Configuring AMDGPU kernel module..."

    # Create module configuration
    cat > /etc/modprobe.d/amdgpu.conf << 'EOF'
# Winux OS AMDGPU Configuration

# Enable display code (DC) - required for modern GPUs
options amdgpu dc=1

# Enable FreeSync/Adaptive Sync
options amdgpu freesync_video=1

# GPU reset on hang
options amdgpu gpu_recovery=1

# Audio support
options amdgpu audio=1

# Deep color (10/12-bit) support
options amdgpu deep_color=1

# Enable power play table modification (for overclocking)
options amdgpu ppfeaturemask=0xffffffff

# Virtual display (useful for VMs)
# options amdgpu virtual_display=0000:XX:00.0,1

# HDMI 2.1 support (RDNA2+)
# options amdgpu hdmi21=1
EOF

    # For legacy Radeon cards
    if [ "$DRIVER_TYPE" = "radeon" ]; then
        cat > /etc/modprobe.d/radeon.conf << 'EOF'
# Winux OS Radeon Configuration

# Enable DPM (Dynamic Power Management)
options radeon dpm=1

# Audio support
options radeon audio=1

# Deep color support
options radeon deep_color=1
EOF
    fi

    log_success "Kernel module configured"
}

configure_xorg() {
    log_info "Configuring Xorg..."

    mkdir -p "$XORG_CONF_DIR"

    cat > "$XORG_CONF_DIR/20-amdgpu.conf" << 'EOF'
# Winux OS AMD GPU Configuration

Section "OutputClass"
    Identifier "AMD"
    MatchDriver "amdgpu"
    Driver "amdgpu"
    Option "DRI" "3"
    Option "TearFree" "true"
    Option "VariableRefresh" "true"
EndSection

Section "Device"
    Identifier "AMD Graphics"
    Driver "amdgpu"

    # Enable TearFree
    Option "TearFree" "true"

    # Enable Variable Refresh Rate (FreeSync)
    Option "VariableRefresh" "true"

    # DRI version
    Option "DRI" "3"

    # Acceleration method
    Option "AccelMethod" "glamor"
EndSection
EOF

    log_success "Xorg configured"
}

configure_vulkan() {
    log_info "Configuring Vulkan..."

    mkdir -p "$VULKAN_ICD_DIR"

    # RADV (Mesa) Vulkan driver - recommended for gaming
    cat > "$VULKAN_ICD_DIR/radeon_icd.x86_64.json" << 'EOF'
{
    "file_format_version" : "1.0.0",
    "ICD": {
        "library_path": "libvulkan_radeon.so",
        "api_version" : "1.3"
    }
}
EOF

    # 32-bit support
    cat > "$VULKAN_ICD_DIR/radeon_icd.i686.json" << 'EOF'
{
    "file_format_version" : "1.0.0",
    "ICD": {
        "library_path": "libvulkan_radeon.so",
        "api_version" : "1.3"
    }
}
EOF

    log_success "Vulkan configured"
}

configure_video_acceleration() {
    log_info "Configuring video acceleration..."

    # VA-API configuration
    cat > /etc/profile.d/amd-vaapi.sh << 'EOF'
# AMD VA-API Configuration
export LIBVA_DRIVER_NAME=radeonsi
export VDPAU_DRIVER=radeonsi
EOF

    chmod +x /etc/profile.d/amd-vaapi.sh

    log_success "Video acceleration configured"
}

configure_gaming_optimizations() {
    log_info "Applying gaming optimizations..."

    # Environment variables for gaming
    cat > /etc/profile.d/amd-gaming.sh << 'EOF'
# Winux OS AMD Gaming Optimizations

# Use RADV (Mesa) Vulkan driver - best for gaming
export AMD_VULKAN_ICD=RADV

# Enable ACO shader compiler (faster compilation)
export RADV_PERFTEST=aco

# Mesa shader cache
export MESA_SHADER_CACHE_DIR="$HOME/.cache/mesa_shader_cache"
export MESA_SHADER_CACHE_MAX_SIZE=10G

# Enable GL threading
export mesa_glthread=true

# VKD3D shader cache for Proton
export VKD3D_SHADER_CACHE_PATH="$HOME/.cache/vkd3d-proton"

# DXVK state cache
export DXVK_STATE_CACHE_PATH="$HOME/.cache/dxvk"

# Force composition bypass for lower latency
# export KWIN_DRM_NO_AMS=1

# FreeSync - enabled via kernel module
# Can also be set per-application via environment

# Debug/Performance options (uncomment if needed):
# export R600_DEBUG=nohyperz
# export RADV_DEBUG=info
# export AMD_DEBUG=info
EOF

    chmod +x /etc/profile.d/amd-gaming.sh

    # Create shader cache directory
    mkdir -p /tmp/mesa_shader_cache
    chmod 1777 /tmp/mesa_shader_cache

    log_success "Gaming optimizations applied"
}

configure_freesync() {
    log_info "Configuring FreeSync/Variable Refresh Rate..."

    # Enable FreeSync at boot
    cat > /etc/udev/rules.d/99-freesync.rules << 'EOF'
# Enable FreeSync for all AMD GPUs
ACTION=="add", SUBSYSTEM=="drm", DRIVERS=="amdgpu", ATTR{device/power_dpm_force_performance_level}="auto"
EOF

    # Create helper script
    cat > /usr/local/bin/freesync-toggle << 'EOF'
#!/bin/bash
# Toggle FreeSync on AMD GPUs

CARD=${1:-card0}
VRR_PATH="/sys/class/drm/${CARD}/device/freesync_video"

if [ ! -f "$VRR_PATH" ]; then
    echo "FreeSync control not found for $CARD"
    echo "Make sure amdgpu.freesync_video=1 is set"
    exit 1
fi

CURRENT=$(cat "$VRR_PATH")

if [ "$2" = "on" ]; then
    echo 1 > "$VRR_PATH"
    echo "FreeSync enabled"
elif [ "$2" = "off" ]; then
    echo 0 > "$VRR_PATH"
    echo "FreeSync disabled"
else
    if [ "$CURRENT" = "1" ]; then
        echo 0 > "$VRR_PATH"
        echo "FreeSync disabled"
    else
        echo 1 > "$VRR_PATH"
        echo "FreeSync enabled"
    fi
fi
EOF

    chmod +x /usr/local/bin/freesync-toggle

    log_success "FreeSync configured"
}

configure_power_management() {
    log_info "Configuring power management..."

    # Create power profile scripts
    cat > /usr/local/bin/amd-gpu-profile << 'EOF'
#!/bin/bash
# AMD GPU Power Profile Switcher

CARD=${2:-card0}
DPM_PATH="/sys/class/drm/${CARD}/device/power_dpm_force_performance_level"
PROFILE_PATH="/sys/class/drm/${CARD}/device/pp_power_profile_mode"

case "$1" in
    performance|gaming)
        echo "high" > "$DPM_PATH"
        # Profile 4 is typically "VR" or high performance
        echo "5" > "$PROFILE_PATH" 2>/dev/null || true
        echo "Set to performance/gaming mode"
        ;;
    balanced)
        echo "auto" > "$DPM_PATH"
        echo "0" > "$PROFILE_PATH" 2>/dev/null || true
        echo "Set to balanced mode"
        ;;
    powersave)
        echo "low" > "$DPM_PATH"
        echo "1" > "$PROFILE_PATH" 2>/dev/null || true
        echo "Set to power saving mode"
        ;;
    status)
        echo "Current performance level: $(cat "$DPM_PATH")"
        if [ -f "$PROFILE_PATH" ]; then
            echo "Available power profiles:"
            cat "$PROFILE_PATH"
        fi
        ;;
    *)
        echo "Usage: $0 {performance|balanced|powersave|status} [card]"
        exit 1
        ;;
esac
EOF

    chmod +x /usr/local/bin/amd-gpu-profile

    # Create udev rule for automatic power management
    cat > /etc/udev/rules.d/80-amdgpu-pm.rules << 'EOF'
# AMD GPU Power Management
ACTION=="add", SUBSYSTEM=="pci", ATTR{vendor}=="0x1002", ATTR{class}=="0x030000", TAG+="systemd", ENV{SYSTEMD_WANTS}="amdgpu-pm.service"
EOF

    # Create systemd service for power management
    cat > /etc/systemd/system/amdgpu-pm.service << 'EOF'
[Unit]
Description=AMD GPU Power Management
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/bin/bash -c 'for card in /sys/class/drm/card*/device/power_dpm_force_performance_level; do echo "auto" > "$card" 2>/dev/null || true; done'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable amdgpu-pm.service

    log_success "Power management configured"
}

configure_overclocking() {
    log_info "Setting up overclocking support..."

    # Ensure ppfeaturemask is set for overclocking
    if ! grep -q "amdgpu.ppfeaturemask" /etc/default/grub 2>/dev/null; then
        sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT="\(.*\)"/GRUB_CMDLINE_LINUX_DEFAULT="\1 amdgpu.ppfeaturemask=0xffffffff"/' /etc/default/grub

        if command -v update-grub &> /dev/null; then
            update-grub
        elif command -v grub2-mkconfig &> /dev/null; then
            grub2-mkconfig -o /boot/grub2/grub.cfg
        fi
    fi

    # Install CoreCtrl if available (GUI overclocking tool)
    case $DISTRO in
        ubuntu|debian|linuxmint)
            apt-get install -y corectrl 2>/dev/null || log_info "CoreCtrl not in repos, install from PPA or Flatpak"
            ;;
        arch|manjaro|endeavouros)
            pacman -S --noconfirm corectrl 2>/dev/null || log_info "CoreCtrl available in AUR"
            ;;
    esac

    # Create helper script for command-line overclocking
    cat > /usr/local/bin/amd-oc << 'EOF'
#!/bin/bash
# AMD GPU Overclocking Helper

CARD=${CARD:-card0}
DEVICE="/sys/class/drm/${CARD}/device"

show_info() {
    echo "=== AMD GPU Info ==="
    echo "GPU Clock: $(cat ${DEVICE}/pp_dpm_sclk | grep '\*' | awk '{print $2}')"
    echo "Memory Clock: $(cat ${DEVICE}/pp_dpm_mclk | grep '\*' | awk '{print $2}')"
    echo "Temperature: $(cat ${DEVICE}/hwmon/hwmon*/temp1_input 2>/dev/null | head -1 | awk '{print $1/1000}')°C"
    echo "Fan Speed: $(cat ${DEVICE}/hwmon/hwmon*/fan1_input 2>/dev/null | head -1) RPM"
    echo "Power: $(cat ${DEVICE}/hwmon/hwmon*/power1_average 2>/dev/null | head -1 | awk '{print $1/1000000}')W"

    if [ -f "${DEVICE}/pp_od_clk_voltage" ]; then
        echo ""
        echo "=== Overdrive Table ==="
        cat "${DEVICE}/pp_od_clk_voltage"
    fi
}

set_sclk() {
    echo "s 1 $1" > "${DEVICE}/pp_od_clk_voltage"
    echo "c" > "${DEVICE}/pp_od_clk_voltage"
    echo "GPU clock set to $1 MHz"
}

set_mclk() {
    echo "m 1 $1" > "${DEVICE}/pp_od_clk_voltage"
    echo "c" > "${DEVICE}/pp_od_clk_voltage"
    echo "Memory clock set to $1 MHz"
}

set_voltage() {
    echo "vo $1" > "${DEVICE}/pp_od_clk_voltage"
    echo "c" > "${DEVICE}/pp_od_clk_voltage"
    echo "Voltage offset set to $1 mV"
}

reset() {
    echo "r" > "${DEVICE}/pp_od_clk_voltage"
    echo "c" > "${DEVICE}/pp_od_clk_voltage"
    echo "Settings reset to default"
}

case "$1" in
    info)
        show_info
        ;;
    sclk)
        set_sclk "$2"
        ;;
    mclk)
        set_mclk "$2"
        ;;
    voltage)
        set_voltage "$2"
        ;;
    reset)
        reset
        ;;
    *)
        echo "Usage: $0 {info|sclk <MHz>|mclk <MHz>|voltage <mV>|reset}"
        echo ""
        echo "Environment: CARD=cardX (default: card0)"
        exit 1
        ;;
esac
EOF

    chmod +x /usr/local/bin/amd-oc

    log_success "Overclocking support configured"
}

update_initramfs() {
    log_info "Updating initramfs..."

    if command -v update-initramfs &> /dev/null; then
        update-initramfs -u
    elif command -v dracut &> /dev/null; then
        dracut --force
    elif command -v mkinitcpio &> /dev/null; then
        mkinitcpio -P
    fi

    log_success "Initramfs updated"
}

verify_installation() {
    log_info "Verifying installation..."

    echo ""

    # Check kernel module
    if lsmod | grep -q amdgpu; then
        log_success "AMDGPU kernel module loaded"
    elif lsmod | grep -q radeon; then
        log_success "Radeon kernel module loaded"
    else
        log_warning "AMD kernel module not loaded (may require reboot)"
    fi

    # Check Vulkan
    if command -v vulkaninfo &> /dev/null; then
        if vulkaninfo 2>/dev/null | grep -qiE "RADV|AMD|radeon"; then
            log_success "Vulkan AMD driver detected"
            echo ""
            vulkaninfo 2>/dev/null | grep -A2 "deviceName"
        else
            log_warning "Vulkan driver may not be properly configured"
        fi
    fi

    # Check VA-API
    if command -v vainfo &> /dev/null; then
        if vainfo 2>/dev/null | grep -q "radeonsi"; then
            log_success "VA-API video acceleration working"
        fi
    fi

    # Check OpenGL
    if command -v glxinfo &> /dev/null; then
        echo ""
        log_info "OpenGL Info:"
        glxinfo 2>/dev/null | grep -E "OpenGL renderer|OpenGL version" || true
    fi
}

print_post_install() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}AMD Driver Installation Complete!${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Post-installation steps:"
    echo "  1. Reboot your system"
    echo "  2. Run 'vulkaninfo' to verify Vulkan"
    echo "  3. Run 'vainfo' to verify video acceleration"
    echo ""
    echo "Gaming optimizations enabled:"
    echo "  - RADV Vulkan driver (recommended for gaming)"
    echo "  - ACO shader compiler"
    echo "  - Mesa shader caching"
    echo "  - FreeSync/VRR support"
    echo ""
    echo "Useful commands:"
    echo "  - amd-gpu-profile performance   # Enable gaming mode"
    echo "  - amd-gpu-profile balanced      # Return to balanced"
    echo "  - freesync-toggle               # Toggle FreeSync"
    echo "  - amd-oc info                   # Show GPU info"
    echo "  - radeontop                     # GPU monitoring"
    echo ""
    echo "For GUI overclocking, install CoreCtrl:"
    echo "  flatpak install flathub org.corectrl.CoreCtrl"
    echo ""
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --mesa-git          Install latest Mesa from PPA/AUR"
    echo "  --amdgpu-pro        Install AMDGPU-PRO OpenCL components"
    echo "  --no-reboot         Don't prompt for reboot"
    echo "  -h, --help          Show this help message"
}

#===============================================================================
# Main
#===============================================================================

main() {
    INSTALL_MESA_GIT=false
    INSTALL_PRO=false
    NO_REBOOT=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --mesa-git)
                INSTALL_MESA_GIT=true
                shift
                ;;
            --amdgpu-pro)
                INSTALL_PRO=true
                shift
                ;;
            --no-reboot)
                NO_REBOOT=true
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done

    print_banner
    check_root
    detect_gpu
    detect_distro
    install_dependencies

    if [ "$INSTALL_MESA_GIT" = "true" ]; then
        install_mesa_git
    fi

    if [ "$INSTALL_PRO" = "true" ]; then
        install_amdgpu_pro
    fi

    configure_kernel_module
    configure_xorg
    configure_vulkan
    configure_video_acceleration
    configure_gaming_optimizations
    configure_freesync
    configure_power_management
    configure_overclocking
    update_initramfs
    verify_installation
    print_post_install

    if [ "$NO_REBOOT" != "true" ]; then
        echo ""
        read -p "Reboot now? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            reboot
        fi
    fi
}

main "$@"
