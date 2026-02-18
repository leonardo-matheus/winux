#!/bin/bash
#===============================================================================
# Winux OS NVIDIA Driver Installation Script
# Complete installation for NVIDIA GPUs with gaming optimizations
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
DRIVER_VERSION="${DRIVER_VERSION:-latest}"
NVIDIA_SITE="https://download.nvidia.com/XFree86/Linux-x86_64"
INSTALL_DIR="/opt/nvidia"
VULKAN_ICD_DIR="/usr/share/vulkan/icd.d"
XORG_CONF_DIR="/etc/X11/xorg.conf.d"

#===============================================================================
# Functions
#===============================================================================

print_banner() {
    echo -e "${GREEN}"
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║              Winux OS NVIDIA Driver Installer                     ║"
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
    log_info "Detecting NVIDIA GPU..."

    if ! lspci | grep -i nvidia &> /dev/null; then
        log_error "No NVIDIA GPU detected"
        exit 1
    fi

    GPU_INFO=$(lspci | grep -i nvidia | head -1)
    log_success "Found: $GPU_INFO"

    # Determine GPU generation for driver compatibility
    if echo "$GPU_INFO" | grep -qi "RTX 4"; then
        GPU_GEN="ada"
        MIN_DRIVER="525"
    elif echo "$GPU_INFO" | grep -qi "RTX 3"; then
        GPU_GEN="ampere"
        MIN_DRIVER="455"
    elif echo "$GPU_INFO" | grep -qi "RTX 2\|GTX 16"; then
        GPU_GEN="turing"
        MIN_DRIVER="418"
    elif echo "$GPU_INFO" | grep -qi "GTX 10"; then
        GPU_GEN="pascal"
        MIN_DRIVER="390"
    else
        GPU_GEN="legacy"
        MIN_DRIVER="470"
    fi

    log_info "GPU Generation: $GPU_GEN (minimum driver: $MIN_DRIVER)"
}

detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$ID
        DISTRO_VERSION=$VERSION_ID
    else
        DISTRO="unknown"
    fi
    log_info "Detected distribution: $DISTRO $DISTRO_VERSION"
}

remove_existing_drivers() {
    log_info "Removing existing NVIDIA drivers..."

    # Remove distro packages
    case $DISTRO in
        ubuntu|debian|linuxmint)
            apt-get remove --purge -y nvidia-* libnvidia-* 2>/dev/null || true
            apt-get autoremove -y 2>/dev/null || true
            ;;
        fedora|rhel|centos)
            dnf remove -y nvidia-* akmod-nvidia* 2>/dev/null || true
            ;;
        arch|manjaro|endeavouros)
            pacman -Rns --noconfirm nvidia nvidia-utils nvidia-settings 2>/dev/null || true
            ;;
    esac

    # Remove manual installations
    if [ -f /usr/bin/nvidia-uninstall ]; then
        log_info "Running NVIDIA uninstaller..."
        /usr/bin/nvidia-uninstall -s 2>/dev/null || true
    fi

    # Blacklist nouveau
    log_info "Blacklisting nouveau driver..."
    cat > /etc/modprobe.d/blacklist-nouveau.conf << 'EOF'
blacklist nouveau
blacklist lbm-nouveau
options nouveau modeset=0
alias nouveau off
alias lbm-nouveau off
EOF

    # Update initramfs
    if command -v update-initramfs &> /dev/null; then
        update-initramfs -u
    elif command -v dracut &> /dev/null; then
        dracut --force
    elif command -v mkinitcpio &> /dev/null; then
        mkinitcpio -P
    fi

    log_success "Existing drivers removed"
}

install_dependencies() {
    log_info "Installing dependencies..."

    case $DISTRO in
        ubuntu|debian|linuxmint)
            apt-get update
            apt-get install -y \
                build-essential \
                dkms \
                linux-headers-$(uname -r) \
                pkg-config \
                libglvnd-dev \
                libvulkan1 \
                vulkan-tools \
                mesa-vulkan-drivers
            ;;
        fedora|rhel|centos)
            dnf install -y \
                kernel-devel \
                kernel-headers \
                dkms \
                gcc \
                make \
                vulkan-loader \
                vulkan-tools
            ;;
        arch|manjaro|endeavouros)
            pacman -S --needed --noconfirm \
                linux-headers \
                dkms \
                base-devel \
                vulkan-icd-loader \
                vulkan-tools
            ;;
    esac

    log_success "Dependencies installed"
}

install_nvidia_package_manager() {
    log_info "Installing NVIDIA drivers via package manager..."

    case $DISTRO in
        ubuntu|debian|linuxmint)
            # Add NVIDIA PPA if available
            if command -v add-apt-repository &> /dev/null; then
                add-apt-repository -y ppa:graphics-drivers/ppa 2>/dev/null || true
                apt-get update
            fi

            # Install recommended driver
            if [ "$DRIVER_VERSION" = "latest" ]; then
                ubuntu-drivers install nvidia 2>/dev/null || \
                apt-get install -y nvidia-driver-550 nvidia-dkms-550
            else
                apt-get install -y "nvidia-driver-${DRIVER_VERSION}" "nvidia-dkms-${DRIVER_VERSION}"
            fi
            ;;

        fedora|rhel|centos)
            # Enable RPM Fusion
            dnf install -y \
                "https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm" \
                "https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm" \
                2>/dev/null || true

            dnf install -y akmod-nvidia xorg-x11-drv-nvidia-cuda
            ;;

        arch|manjaro|endeavouros)
            pacman -S --noconfirm nvidia nvidia-utils nvidia-settings lib32-nvidia-utils
            ;;
    esac

    log_success "NVIDIA drivers installed via package manager"
}

install_nvidia_manual() {
    log_info "Installing NVIDIA drivers manually..."

    mkdir -p "$INSTALL_DIR"
    cd "$INSTALL_DIR"

    # Get latest driver version
    if [ "$DRIVER_VERSION" = "latest" ]; then
        log_info "Fetching latest driver version..."
        DRIVER_VERSION=$(curl -s "$NVIDIA_SITE/latest.txt" | head -1 | awk '{print $1}')
    fi

    DRIVER_FILE="NVIDIA-Linux-x86_64-${DRIVER_VERSION}.run"
    DRIVER_URL="${NVIDIA_SITE}/${DRIVER_VERSION}/${DRIVER_FILE}"

    log_info "Downloading NVIDIA driver $DRIVER_VERSION..."

    if [ ! -f "$DRIVER_FILE" ]; then
        wget -q --show-progress "$DRIVER_URL" -O "$DRIVER_FILE"
    fi

    chmod +x "$DRIVER_FILE"

    log_info "Installing driver..."

    # Stop display manager
    systemctl stop display-manager 2>/dev/null || true
    systemctl stop gdm 2>/dev/null || true
    systemctl stop sddm 2>/dev/null || true
    systemctl stop lightdm 2>/dev/null || true

    # Install driver
    ./"$DRIVER_FILE" \
        --silent \
        --dkms \
        --install-libglvnd \
        --no-questions \
        --ui=none

    log_success "NVIDIA driver $DRIVER_VERSION installed"
}

configure_nvidia() {
    log_info "Configuring NVIDIA driver..."

    # Create Xorg configuration
    mkdir -p "$XORG_CONF_DIR"

    cat > "$XORG_CONF_DIR/20-nvidia.conf" << 'EOF'
# Winux OS NVIDIA Configuration
# Optimized for gaming performance

Section "OutputClass"
    Identifier "nvidia"
    MatchDriver "nvidia-drm"
    Driver "nvidia"
    Option "AllowEmptyInitialConfiguration"
    Option "PrimaryGPU" "yes"
    ModulePath "/usr/lib/x86_64-linux-gnu/nvidia/xorg"
EndSection

Section "Device"
    Identifier "NVIDIA Card"
    Driver "nvidia"
    VendorName "NVIDIA Corporation"

    # Performance options
    Option "Coolbits" "28"
    Option "TripleBuffer" "True"
    Option "AllowIndirectGLXProtocol" "off"
    Option "RegistryDwords" "PerfLevelSrc=0x2222"
EndSection

Section "Screen"
    Identifier "Default Screen"
    Device "NVIDIA Card"

    # Allow mode setting
    Option "metamodes" "nvidia-auto-select +0+0 {ForceCompositionPipeline=On, ForceFullCompositionPipeline=On}"
EndSection
EOF

    log_success "Xorg configuration created"
}

configure_vulkan() {
    log_info "Configuring Vulkan..."

    mkdir -p "$VULKAN_ICD_DIR"

    # NVIDIA Vulkan ICD
    cat > "$VULKAN_ICD_DIR/nvidia_icd.json" << 'EOF'
{
    "file_format_version" : "1.0.0",
    "ICD": {
        "library_path": "libGLX_nvidia.so.0",
        "api_version" : "1.3"
    }
}
EOF

    log_success "Vulkan configured"
}

configure_dkms() {
    log_info "Configuring DKMS..."

    # Ensure NVIDIA module rebuilds on kernel update
    if dkms status | grep -q nvidia; then
        log_success "NVIDIA DKMS module registered"
    else
        log_warning "DKMS may not be properly configured"
    fi
}

configure_power_management() {
    log_info "Configuring power management..."

    # Enable runtime power management for laptops
    cat > /etc/udev/rules.d/80-nvidia-pm.rules << 'EOF'
# Enable runtime PM for NVIDIA VGA/3D controller devices
ACTION=="bind", SUBSYSTEM=="pci", ATTR{vendor}=="0x10de", ATTR{class}=="0x030000", TEST=="power/control", ATTR{power/control}="auto"
ACTION=="bind", SUBSYSTEM=="pci", ATTR{vendor}=="0x10de", ATTR{class}=="0x030200", TEST=="power/control", ATTR{power/control}="auto"

# Disable runtime PM for NVIDIA VGA/3D controller devices during gaming
ACTION=="unbind", SUBSYSTEM=="pci", ATTR{vendor}=="0x10de", ATTR{class}=="0x030000", TEST=="power/control", ATTR{power/control}="on"
ACTION=="unbind", SUBSYSTEM=="pci", ATTR{vendor}=="0x10de", ATTR{class}=="0x030200", TEST=="power/control", ATTR{power/control}="on"
EOF

    # Create nvidia-persistenced service if not exists
    if [ ! -f /etc/systemd/system/nvidia-persistenced.service ]; then
        cat > /etc/systemd/system/nvidia-persistenced.service << 'EOF'
[Unit]
Description=NVIDIA Persistence Daemon
Wants=syslog.target

[Service]
Type=forking
ExecStart=/usr/bin/nvidia-persistenced --user nvidia-persistenced --persistence-mode --verbose
ExecStopPost=/bin/rm -rf /var/run/nvidia-persistenced

[Install]
WantedBy=multi-user.target
EOF
    fi

    # Create user for persistence daemon
    useradd -r -M -d /var/run/nvidia-persistenced nvidia-persistenced 2>/dev/null || true

    log_success "Power management configured"
}

configure_gaming_optimizations() {
    log_info "Applying gaming optimizations..."

    # Create NVIDIA settings profile for gaming
    mkdir -p /etc/nvidia

    cat > /etc/nvidia/nvidia-application-profiles-rc << 'EOF'
{
    "profiles": [
        {
            "name": "WinuxGaming",
            "settings": [
                {
                    "key": "GLThreadedOptimizations",
                    "value": true
                },
                {
                    "key": "GLSLCaching",
                    "value": true
                },
                {
                    "key": "GLShaderDiskCache",
                    "value": true
                },
                {
                    "key": "GLShaderDiskCachePath",
                    "value": "$HOME/.cache/nvidia/GLCache"
                }
            ]
        }
    ],
    "rules": [
        {
            "pattern": {
                "feature": "procname",
                "matches": ".*"
            },
            "profile": "WinuxGaming"
        }
    ]
}
EOF

    # Environment variables for gaming
    cat > /etc/profile.d/nvidia-gaming.sh << 'EOF'
# Winux OS NVIDIA Gaming Optimizations

# Enable threaded optimizations
export __GL_THREADED_OPTIMIZATIONS=1

# Shader cache
export __GL_SHADER_DISK_CACHE=1
export __GL_SHADER_DISK_CACHE_PATH="$HOME/.cache/nvidia/GLCache"
export __GL_SHADER_DISK_CACHE_SKIP_CLEANUP=1

# VKD3D shader cache for Proton
export VKD3D_SHADER_CACHE_PATH="$HOME/.cache/vkd3d-proton"

# DXVK state cache
export DXVK_STATE_CACHE_PATH="$HOME/.cache/dxvk"

# Enable GPU video acceleration
export LIBVA_DRIVER_NAME=nvidia
export VDPAU_DRIVER=nvidia

# Force composition pipeline (reduces tearing)
# Uncomment if experiencing tearing:
# export __GL_SYNC_DISPLAY_DEVICE=DP-0
# export __GL_SYNC_TO_VBLANK=1
EOF

    chmod +x /etc/profile.d/nvidia-gaming.sh

    log_success "Gaming optimizations applied"
}

configure_wayland() {
    log_info "Configuring Wayland support..."

    # Enable DRM modeset for Wayland
    if ! grep -q "nvidia-drm.modeset=1" /etc/default/grub 2>/dev/null; then
        sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT="\(.*\)"/GRUB_CMDLINE_LINUX_DEFAULT="\1 nvidia-drm.modeset=1"/' /etc/default/grub

        if command -v update-grub &> /dev/null; then
            update-grub
        elif command -v grub2-mkconfig &> /dev/null; then
            grub2-mkconfig -o /boot/grub2/grub.cfg
        fi
    fi

    # Module options
    cat > /etc/modprobe.d/nvidia.conf << 'EOF'
# Enable modesetting for Wayland
options nvidia-drm modeset=1

# Preserve video memory on suspend
options nvidia NVreg_PreserveVideoMemoryAllocations=1

# Enable power management
options nvidia NVreg_DynamicPowerManagement=0x02
EOF

    log_success "Wayland support configured"
}

install_cuda_toolkit() {
    log_info "Installing CUDA toolkit..."

    case $DISTRO in
        ubuntu|debian|linuxmint)
            # Install CUDA from NVIDIA repo
            apt-get install -y nvidia-cuda-toolkit 2>/dev/null || {
                log_warning "CUDA toolkit not available in repos"
            }
            ;;
        fedora|rhel|centos)
            dnf install -y cuda 2>/dev/null || {
                log_warning "CUDA toolkit not available in repos"
            }
            ;;
        arch|manjaro|endeavouros)
            pacman -S --noconfirm cuda 2>/dev/null || {
                log_warning "CUDA toolkit not available in repos"
            }
            ;;
    esac
}

verify_installation() {
    log_info "Verifying installation..."

    # Load nvidia module
    modprobe nvidia 2>/dev/null || true

    # Check nvidia-smi
    if command -v nvidia-smi &> /dev/null; then
        echo ""
        nvidia-smi
        echo ""
        log_success "nvidia-smi working"
    else
        log_warning "nvidia-smi not found"
    fi

    # Check Vulkan
    if command -v vulkaninfo &> /dev/null; then
        if vulkaninfo 2>/dev/null | grep -q "NVIDIA"; then
            log_success "Vulkan NVIDIA driver detected"
        else
            log_warning "Vulkan may not be properly configured"
        fi
    fi

    # Check kernel module
    if lsmod | grep -q nvidia; then
        log_success "NVIDIA kernel module loaded"
    else
        log_warning "NVIDIA kernel module not loaded (may require reboot)"
    fi
}

print_post_install() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}NVIDIA Driver Installation Complete!${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Post-installation steps:"
    echo "  1. Reboot your system"
    echo "  2. Run 'nvidia-smi' to verify the driver"
    echo "  3. Run 'nvidia-settings' to configure display settings"
    echo ""
    echo "Gaming optimizations enabled:"
    echo "  - Threaded OpenGL optimizations"
    echo "  - Shader disk caching"
    echo "  - Vulkan ICD configured"
    echo "  - DXVK/VKD3D cache paths set"
    echo ""
    echo "For Wayland users:"
    echo "  - DRM modesetting enabled"
    echo "  - GBM backend available"
    echo ""
    echo "Performance tuning (requires nvidia-settings):"
    echo "  - Run: nvidia-settings -a '[gpu:0]/GPUPowerMizerMode=1'"
    echo "  - For overclocking: nvidia-settings (Coolbits enabled)"
    echo ""
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --version VERSION    Install specific driver version (default: latest)"
    echo "  --package-manager    Use package manager instead of manual install"
    echo "  --cuda               Also install CUDA toolkit"
    echo "  --no-reboot          Don't prompt for reboot"
    echo "  -h, --help           Show this help message"
}

#===============================================================================
# Main
#===============================================================================

main() {
    USE_PACKAGE_MANAGER=false
    INSTALL_CUDA=false
    NO_REBOOT=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                DRIVER_VERSION="$2"
                shift 2
                ;;
            --package-manager)
                USE_PACKAGE_MANAGER=true
                shift
                ;;
            --cuda)
                INSTALL_CUDA=true
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
    remove_existing_drivers

    if [ "$USE_PACKAGE_MANAGER" = "true" ]; then
        install_nvidia_package_manager
    else
        install_nvidia_manual
    fi

    configure_nvidia
    configure_vulkan
    configure_dkms
    configure_power_management
    configure_gaming_optimizations
    configure_wayland

    if [ "$INSTALL_CUDA" = "true" ]; then
        install_cuda_toolkit
    fi

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
