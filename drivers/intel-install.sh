#!/bin/bash
#===============================================================================
# Winux OS Intel Driver Installation Script
# Complete installation for Intel GPUs (integrated and discrete Arc)
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
VULKAN_ICD_DIR="/usr/share/vulkan/icd.d"
XORG_CONF_DIR="/etc/X11/xorg.conf.d"
INTEL_COMPUTE_RUNTIME_VERSION="latest"

#===============================================================================
# Functions
#===============================================================================

print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║               Winux OS Intel Driver Installer                     ║"
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
    log_info "Detecting Intel GPU..."

    if ! lspci | grep -iE "Intel.*(VGA|3D|Display|Graphics)" &> /dev/null; then
        log_error "No Intel GPU detected"
        exit 1
    fi

    GPU_INFO=$(lspci | grep -iE "Intel.*(VGA|3D|Display|Graphics)" | head -1)
    log_success "Found: $GPU_INFO"

    # Detect GPU generation
    if echo "$GPU_INFO" | grep -qiE "Arc A[0-9]{3}|DG2|Alchemist"; then
        GPU_GEN="arc"
        DRIVER="xe"
        log_info "Intel Arc discrete GPU detected"
    elif echo "$GPU_INFO" | grep -qiE "Meteor|MTL|Lunar|LNL"; then
        GPU_GEN="meteorlake"
        DRIVER="i915"
    elif echo "$GPU_INFO" | grep -qiE "Raptor|RPL|Alder|ADL"; then
        GPU_GEN="alderlake"
        DRIVER="i915"
    elif echo "$GPU_INFO" | grep -qiE "Tiger|TGL|Rocket|RKL"; then
        GPU_GEN="tigerlake"
        DRIVER="i915"
    elif echo "$GPU_INFO" | grep -qiE "Ice|ICL|Jasper|JSL|Elkhart|EHL"; then
        GPU_GEN="icelake"
        DRIVER="i915"
    elif echo "$GPU_INFO" | grep -qiE "Coffee|CFL|Comet|CML|Whiskey|WHL"; then
        GPU_GEN="coffeelake"
        DRIVER="i915"
    elif echo "$GPU_INFO" | grep -qiE "Kaby|KBL|Amber|AML"; then
        GPU_GEN="kabylake"
        DRIVER="i915"
    elif echo "$GPU_INFO" | grep -qiE "Skylake|SKL"; then
        GPU_GEN="skylake"
        DRIVER="i915"
    elif echo "$GPU_INFO" | grep -qiE "Broadwell|BDW"; then
        GPU_GEN="broadwell"
        DRIVER="i915"
    else
        GPU_GEN="legacy"
        DRIVER="i915"
    fi

    log_info "GPU Generation: $GPU_GEN (driver: $DRIVER)"
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
                mesa-utils \
                mesa-vulkan-drivers \
                intel-media-va-driver-non-free \
                vainfo \
                vulkan-tools \
                libvulkan1 \
                intel-gpu-tools \
                libdrm-intel1
            ;;
        fedora|rhel|centos)
            dnf install -y \
                kernel-devel \
                kernel-headers \
                mesa-dri-drivers \
                mesa-vulkan-drivers \
                intel-media-driver \
                libva-utils \
                vulkan-loader \
                vulkan-tools \
                intel-gpu-tools \
                libdrm
            ;;
        arch|manjaro|endeavouros)
            pacman -S --needed --noconfirm \
                linux-headers \
                mesa \
                lib32-mesa \
                vulkan-intel \
                lib32-vulkan-intel \
                vulkan-icd-loader \
                lib32-vulkan-icd-loader \
                vulkan-tools \
                intel-media-driver \
                libva-intel-driver \
                libva-utils \
                intel-gpu-tools
            ;;
    esac

    log_success "Dependencies installed"
}

install_intel_graphics_compiler() {
    log_info "Installing Intel Graphics Compiler..."

    case $DISTRO in
        ubuntu|debian|linuxmint)
            # Add Intel graphics repository
            wget -qO - https://repositories.intel.com/graphics/intel-graphics.key | gpg --dearmor -o /usr/share/keyrings/intel-graphics.gpg

            echo "deb [arch=amd64 signed-by=/usr/share/keyrings/intel-graphics.gpg] https://repositories.intel.com/graphics/ubuntu ${DISTRO_CODENAME} main" > /etc/apt/sources.list.d/intel-graphics.list

            apt-get update
            apt-get install -y \
                intel-opencl-icd \
                intel-level-zero-gpu \
                level-zero \
                intel-media-va-driver-non-free \
                libmfx1 \
                libmfxgen1 \
                libvpl2 \
                libegl-mesa0 \
                libegl1-mesa \
                libegl1-mesa-dev \
                libgbm1 \
                libgl1-mesa-dev \
                libgl1-mesa-dri \
                libglapi-mesa \
                libgles2-mesa-dev \
                libglx-mesa0 \
                libigdgmm12 \
                libxatracker2 \
                mesa-va-drivers \
                mesa-vdpau-drivers \
                mesa-vulkan-drivers \
                va-driver-all \
                2>/dev/null || log_warning "Some Intel packages not available"
            ;;
        fedora|rhel|centos)
            # Intel compute runtime
            dnf install -y \
                intel-compute-runtime \
                level-zero \
                intel-media-driver \
                2>/dev/null || log_warning "Some Intel packages not available"
            ;;
        arch|manjaro|endeavouros)
            # Install from AUR or repos
            pacman -S --needed --noconfirm \
                intel-compute-runtime \
                level-zero-loader \
                intel-media-driver \
                2>/dev/null || log_warning "Some Intel packages not available"
            ;;
    esac

    log_success "Intel Graphics Compiler installed"
}

install_arc_support() {
    log_info "Installing Intel Arc GPU support..."

    # Arc requires newer kernel and Mesa
    case $DISTRO in
        ubuntu|debian|linuxmint)
            # Ensure we have the latest Mesa
            add-apt-repository -y ppa:kisak/kisak-mesa 2>/dev/null || true
            apt-get update
            apt-get upgrade -y mesa-vulkan-drivers

            # Install Arc-specific packages
            apt-get install -y \
                intel-media-va-driver-non-free \
                2>/dev/null || true
            ;;
        arch|manjaro|endeavouros)
            # Arch usually has recent enough packages
            pacman -Syu --noconfirm
            ;;
    esac

    # Configure kernel for Arc
    configure_arc_kernel

    log_success "Intel Arc support configured"
}

configure_arc_kernel() {
    log_info "Configuring kernel for Intel Arc..."

    # Add kernel parameters for Arc
    local grub_params="i915.force_probe=* intel_iommu=on iommu=pt"

    if ! grep -q "i915.force_probe" /etc/default/grub 2>/dev/null; then
        sed -i "s/GRUB_CMDLINE_LINUX_DEFAULT=\"\(.*\)\"/GRUB_CMDLINE_LINUX_DEFAULT=\"\1 ${grub_params}\"/" /etc/default/grub

        if command -v update-grub &> /dev/null; then
            update-grub
        elif command -v grub2-mkconfig &> /dev/null; then
            grub2-mkconfig -o /boot/grub2/grub.cfg
        fi
    fi

    log_success "Arc kernel parameters configured"
}

configure_kernel_module() {
    log_info "Configuring i915 kernel module..."

    cat > /etc/modprobe.d/i915.conf << 'EOF'
# Winux OS Intel GPU Configuration

# Enable GuC/HuC firmware loading (improves performance)
options i915 enable_guc=3

# Enable frame buffer compression (power saving)
options i915 enable_fbc=1

# Enable Panel Self Refresh (PSR) for laptops
# May cause flickering on some systems - disable if issues
options i915 enable_psr=1

# Enable deep sleep for power management
options i915 enable_dc=2

# Enable DPCD backlight control (for eDP)
options i915 enable_dpcd_backlight=1

# Force probe for newer GPUs
# Uncomment for Arc GPUs or new integrated graphics:
# options i915 force_probe=*

# Error capture for debugging (can be disabled for slight performance gain)
# options i915 error_capture=0

# Fastboot for faster boot (may cause issues on some systems)
options i915 fastboot=1
EOF

    # For Intel Arc with Xe driver (kernel 6.8+)
    if [ "$GPU_GEN" = "arc" ] || [ "$DRIVER" = "xe" ]; then
        cat > /etc/modprobe.d/xe.conf << 'EOF'
# Winux OS Intel Xe (Arc) GPU Configuration

# Enable performance mode
options xe force_probe=*
EOF
    fi

    log_success "Kernel module configured"
}

configure_xorg() {
    log_info "Configuring Xorg..."

    mkdir -p "$XORG_CONF_DIR"

    cat > "$XORG_CONF_DIR/20-intel.conf" << 'EOF'
# Winux OS Intel GPU Configuration

Section "OutputClass"
    Identifier "Intel Graphics"
    MatchDriver "i915"
    Driver "modesetting"
    Option "DRI" "3"
EndSection

Section "Device"
    Identifier "Intel Graphics"
    Driver "modesetting"

    # Use DRI3 (modern, better performance)
    Option "DRI" "3"

    # Acceleration method (glamor is default and recommended)
    Option "AccelMethod" "glamor"

    # TearFree prevents screen tearing
    # May add some latency - disable for competitive gaming
    Option "TearFree" "true"

    # Enable triple buffering for smoother vsync
    Option "TripleBuffer" "true"
EndSection
EOF

    # Alternative Intel DDX driver config (if using xf86-video-intel)
    cat > "$XORG_CONF_DIR/20-intel-ddx.conf.disabled" << 'EOF'
# Intel DDX Driver Configuration (disabled by default)
# Rename to 20-intel.conf to use instead of modesetting
# Note: modesetting is recommended for modern Intel GPUs

Section "Device"
    Identifier "Intel Graphics"
    Driver "intel"

    # SNA acceleration (faster but may have bugs)
    Option "AccelMethod" "sna"

    # TearFree
    Option "TearFree" "true"

    # Triple buffering
    Option "TripleBuffer" "true"

    # Virtual pages (performance improvement)
    Option "VirtualHeads" "1"
EndSection
EOF

    log_success "Xorg configured"
}

configure_vulkan() {
    log_info "Configuring Vulkan..."

    mkdir -p "$VULKAN_ICD_DIR"

    # Intel ANV (Mesa) Vulkan driver
    cat > "$VULKAN_ICD_DIR/intel_icd.x86_64.json" << 'EOF'
{
    "file_format_version" : "1.0.0",
    "ICD": {
        "library_path": "libvulkan_intel.so",
        "api_version" : "1.3"
    }
}
EOF

    # 32-bit support
    cat > "$VULKAN_ICD_DIR/intel_icd.i686.json" << 'EOF'
{
    "file_format_version" : "1.0.0",
    "ICD": {
        "library_path": "libvulkan_intel.so",
        "api_version" : "1.3"
    }
}
EOF

    # Intel ANV Hasvk driver (for Haswell and older)
    cat > "$VULKAN_ICD_DIR/intel_hasvk_icd.x86_64.json" << 'EOF'
{
    "file_format_version" : "1.0.0",
    "ICD": {
        "library_path": "libvulkan_intel_hasvk.so",
        "api_version" : "1.3"
    }
}
EOF

    log_success "Vulkan configured"
}

configure_video_acceleration() {
    log_info "Configuring video acceleration..."

    # VA-API configuration
    cat > /etc/profile.d/intel-vaapi.sh << 'EOF'
# Intel VA-API Configuration

# Modern Intel GPUs (Gen8+)
export LIBVA_DRIVER_NAME=iHD

# For older Intel GPUs (Gen7 and earlier), use:
# export LIBVA_DRIVER_NAME=i965

# Enable hybrid codec support for newer GPUs
export LIBVA_DRIVERS_PATH=/usr/lib/x86_64-linux-gnu/dri

# For Intel Quick Sync Video
export MFX_HOME=/opt/intel/mediasdk
EOF

    chmod +x /etc/profile.d/intel-vaapi.sh

    # For older GPUs
    cat > /etc/profile.d/intel-vaapi-legacy.sh.disabled << 'EOF'
# Intel VA-API Configuration (Legacy)
# Rename to intel-vaapi.sh for Gen7 and older

export LIBVA_DRIVER_NAME=i965
EOF

    log_success "Video acceleration configured"
}

configure_gaming_optimizations() {
    log_info "Applying gaming optimizations..."

    # Environment variables for gaming
    cat > /etc/profile.d/intel-gaming.sh << 'EOF'
# Winux OS Intel Gaming Optimizations

# Enable ANV threading for better performance
export ANV_QUEUE_THREAD_DISABLE=0

# Mesa shader cache
export MESA_SHADER_CACHE_DIR="$HOME/.cache/mesa_shader_cache"
export MESA_SHADER_CACHE_MAX_SIZE=10G

# Enable GL threading
export mesa_glthread=true

# VKD3D shader cache for Proton
export VKD3D_SHADER_CACHE_PATH="$HOME/.cache/vkd3d-proton"

# DXVK state cache
export DXVK_STATE_CACHE_PATH="$HOME/.cache/dxvk"

# Intel-specific performance tuning
export INTEL_DEBUG=""

# Enable VAAPI video acceleration in browsers
export MOZ_DISABLE_RDD_SANDBOX=1
export MOZ_X11_EGL=1

# Force VAAPI backend
export LIBVA_DRIVER_NAME=iHD

# For Intel Arc GPUs, enable ray tracing support in Vulkan
# export VKD3D_FEATURE_LEVEL=12_1
EOF

    chmod +x /etc/profile.d/intel-gaming.sh

    # Create shader cache directory
    mkdir -p /tmp/mesa_shader_cache
    chmod 1777 /tmp/mesa_shader_cache

    log_success "Gaming optimizations applied"
}

configure_power_management() {
    log_info "Configuring power management..."

    # Create power profile scripts
    cat > /usr/local/bin/intel-gpu-profile << 'EOF'
#!/bin/bash
# Intel GPU Power Profile Switcher

# Find the Intel GPU
GPU_PATH=""
for card in /sys/class/drm/card*/device/vendor; do
    if [ -f "$card" ] && grep -q "0x8086" "$card"; then
        GPU_PATH=$(dirname "$card")
        break
    fi
done

if [ -z "$GPU_PATH" ]; then
    echo "Intel GPU not found"
    exit 1
fi

# Check for RC6 power state
RC6_PATH="${GPU_PATH}/power/rc6_enable"

case "$1" in
    performance)
        # Disable power saving for maximum performance
        echo "performance" > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true
        echo "Set to performance mode"
        ;;
    balanced)
        echo "schedutil" > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true
        echo "Set to balanced mode"
        ;;
    powersave)
        echo "powersave" > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true
        echo "Set to power saving mode"
        ;;
    status)
        echo "=== Intel GPU Info ==="
        intel_gpu_top -l1 2>/dev/null || echo "Run intel_gpu_top for monitoring"
        echo ""
        echo "CPU Governor: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'N/A')"
        ;;
    *)
        echo "Usage: $0 {performance|balanced|powersave|status}"
        exit 1
        ;;
esac
EOF

    chmod +x /usr/local/bin/intel-gpu-profile

    # TLP configuration for Intel GPUs (if TLP is installed)
    if [ -d /etc/tlp.d ]; then
        cat > /etc/tlp.d/50-intel-gpu.conf << 'EOF'
# Intel GPU Power Management for TLP

# Enable Intel GPU RC6 deep power states
INTEL_GPU_MIN_FREQ_ON_AC=300
INTEL_GPU_MIN_FREQ_ON_BAT=300
INTEL_GPU_MAX_FREQ_ON_AC=1300
INTEL_GPU_MAX_FREQ_ON_BAT=800
INTEL_GPU_BOOST_FREQ_ON_AC=1300
INTEL_GPU_BOOST_FREQ_ON_BAT=900
EOF
    fi

    log_success "Power management configured"
}

configure_hybrid_graphics() {
    log_info "Checking for hybrid graphics setup..."

    # Check if there's also an NVIDIA or AMD GPU
    if lspci | grep -iE "NVIDIA|AMD|ATI" | grep -iE "VGA|3D|Display" &> /dev/null; then
        log_info "Hybrid graphics detected!"

        # Install PRIME utilities
        case $DISTRO in
            ubuntu|debian|linuxmint)
                apt-get install -y nvidia-prime 2>/dev/null || true
                apt-get install -y switcheroo-control 2>/dev/null || true
                ;;
            arch|manjaro|endeavouros)
                pacman -S --needed --noconfirm switcheroo-control 2>/dev/null || true
                ;;
        esac

        # Create PRIME render offload script
        cat > /usr/local/bin/prime-run << 'EOF'
#!/bin/bash
# Run application with discrete GPU (NVIDIA/AMD)

# Detect discrete GPU
if lspci | grep -qi nvidia; then
    # NVIDIA PRIME offload
    export __NV_PRIME_RENDER_OFFLOAD=1
    export __NV_PRIME_RENDER_OFFLOAD_PROVIDER=NVIDIA-G0
    export __GLX_VENDOR_LIBRARY_NAME=nvidia
    export __VK_LAYER_NV_optimus=NVIDIA_only
    exec "$@"
elif lspci | grep -qiE "AMD|ATI"; then
    # AMD PRIME offload
    export DRI_PRIME=1
    exec "$@"
else
    exec "$@"
fi
EOF
        chmod +x /usr/local/bin/prime-run

        # Create script to run on Intel
        cat > /usr/local/bin/intel-run << 'EOF'
#!/bin/bash
# Run application with Intel GPU

export DRI_PRIME=0
export __NV_PRIME_RENDER_OFFLOAD=0
exec "$@"
EOF
        chmod +x /usr/local/bin/intel-run

        # Enable switcheroo service if available
        systemctl enable switcheroo-control 2>/dev/null || true

        log_success "Hybrid graphics configured"
        log_info "Use 'prime-run <app>' to run on discrete GPU"
        log_info "Use 'intel-run <app>' to force Intel GPU"
    else
        log_info "Single GPU system, no hybrid configuration needed"
    fi
}

update_initramfs() {
    log_info "Updating initramfs..."

    # Add i915 to early loading
    case $DISTRO in
        ubuntu|debian|linuxmint)
            if ! grep -q "i915" /etc/initramfs-tools/modules 2>/dev/null; then
                echo "i915" >> /etc/initramfs-tools/modules
            fi
            update-initramfs -u
            ;;
        fedora|rhel|centos)
            dracut --force
            ;;
        arch|manjaro|endeavouros)
            # Add to mkinitcpio.conf
            if ! grep -q "i915" /etc/mkinitcpio.conf 2>/dev/null; then
                sed -i 's/MODULES=(\(.*\))/MODULES=(\1 i915)/' /etc/mkinitcpio.conf
            fi
            mkinitcpio -P
            ;;
    esac

    log_success "Initramfs updated"
}

verify_installation() {
    log_info "Verifying installation..."

    echo ""

    # Check kernel module
    if lsmod | grep -q i915; then
        log_success "i915 kernel module loaded"
    elif lsmod | grep -q xe; then
        log_success "xe (Arc) kernel module loaded"
    else
        log_warning "Intel kernel module not loaded (may require reboot)"
    fi

    # Check GuC/HuC
    if dmesg | grep -qi "guc.*loaded\|huc.*loaded"; then
        log_success "GuC/HuC firmware loaded"
    else
        log_info "GuC/HuC status unknown (check after reboot)"
    fi

    # Check Vulkan
    if command -v vulkaninfo &> /dev/null; then
        if vulkaninfo 2>/dev/null | grep -qi "Intel"; then
            log_success "Vulkan Intel driver detected"
            echo ""
            vulkaninfo 2>/dev/null | grep -A2 "deviceName" | head -5
        else
            log_warning "Vulkan driver may not be properly configured"
        fi
    fi

    # Check VA-API
    if command -v vainfo &> /dev/null; then
        if vainfo 2>/dev/null | grep -qiE "iHD|i965"; then
            log_success "VA-API video acceleration working"
        fi
    fi

    # Check OpenGL
    if command -v glxinfo &> /dev/null; then
        echo ""
        log_info "OpenGL Info:"
        glxinfo 2>/dev/null | grep -E "OpenGL renderer|OpenGL version" | head -2 || true
    fi
}

print_post_install() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}Intel Driver Installation Complete!${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Post-installation steps:"
    echo "  1. Reboot your system"
    echo "  2. Run 'vulkaninfo' to verify Vulkan"
    echo "  3. Run 'vainfo' to verify video acceleration"
    echo ""
    echo "Gaming optimizations enabled:"
    echo "  - ANV Vulkan driver"
    echo "  - Mesa shader caching"
    echo "  - GuC/HuC firmware enabled"
    echo ""
    echo "Useful commands:"
    echo "  - intel-gpu-profile performance  # Enable performance mode"
    echo "  - intel-gpu-profile balanced     # Return to balanced"
    echo "  - intel_gpu_top                  # GPU monitoring"
    echo "  - vainfo                         # Check video acceleration"
    echo ""

    if [ "$GPU_GEN" = "arc" ]; then
        echo "Intel Arc specific notes:"
        echo "  - Ensure kernel 6.2+ for best support"
        echo "  - Ray tracing supported in Vulkan applications"
        echo "  - Use 'DRI_PRIME=1' if in hybrid setup"
        echo ""
    fi

    if lspci | grep -iE "NVIDIA|AMD" | grep -iE "VGA|3D|Display" &> /dev/null; then
        echo "Hybrid graphics detected:"
        echo "  - Use 'prime-run <app>' to run on discrete GPU"
        echo "  - Use 'intel-run <app>' to force Intel GPU"
        echo ""
    fi
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --arc               Install Intel Arc discrete GPU support"
    echo "  --compute           Install OpenCL/compute runtime"
    echo "  --no-reboot         Don't prompt for reboot"
    echo "  -h, --help          Show this help message"
}

#===============================================================================
# Main
#===============================================================================

main() {
    INSTALL_ARC=false
    INSTALL_COMPUTE=false
    NO_REBOOT=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --arc)
                INSTALL_ARC=true
                shift
                ;;
            --compute)
                INSTALL_COMPUTE=true
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

    if [ "$GPU_GEN" = "arc" ] || [ "$INSTALL_ARC" = "true" ]; then
        install_arc_support
    fi

    if [ "$INSTALL_COMPUTE" = "true" ]; then
        install_intel_graphics_compiler
    fi

    configure_kernel_module
    configure_xorg
    configure_vulkan
    configure_video_acceleration
    configure_gaming_optimizations
    configure_power_management
    configure_hybrid_graphics
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
