#!/bin/bash
#===============================================================================
# Winux OS Kernel Build Script
# Builds a custom Zen kernel with BORE scheduler for optimal gaming performance
#===============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
KERNEL_VERSION="${KERNEL_VERSION:-6.7}"
KERNEL_MAJOR="${KERNEL_VERSION%%.*}"
ZEN_REPO="https://github.com/zen-kernel/zen-kernel"
BORE_REPO="https://github.com/firelzrd/bore-scheduler"
BUILD_DIR="/tmp/winux-kernel-build"
CONFIG_DIR="$(dirname "$(readlink -f "$0")")/config"
PATCHES_DIR="$(dirname "$(readlink -f "$0")")/patches"
INSTALL_MODULES=true
INSTALL_KERNEL=true
JOBS=$(nproc)

#===============================================================================
# Functions
#===============================================================================

print_banner() {
    echo -e "${CYAN}"
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║                    Winux OS Kernel Builder                        ║"
    echo "║               Zen Kernel + BORE Scheduler                         ║"
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

check_dependencies() {
    log_info "Checking build dependencies..."

    local deps=(
        "gcc"
        "make"
        "flex"
        "bison"
        "bc"
        "libncurses-dev"
        "libssl-dev"
        "libelf-dev"
        "dwarves"
        "zstd"
        "git"
        "wget"
        "cpio"
        "xz-utils"
    )

    local missing=()

    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null && ! dpkg -l | grep -q "^ii  $dep"; then
            missing+=("$dep")
        fi
    done

    if [ ${#missing[@]} -ne 0 ]; then
        log_warning "Missing dependencies: ${missing[*]}"
        log_info "Installing dependencies..."

        if command -v apt &> /dev/null; then
            sudo apt update
            sudo apt install -y build-essential bc kmod cpio flex libncurses-dev \
                libelf-dev libssl-dev dwarves bison zstd git wget xz-utils
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --needed base-devel bc kmod cpio flex ncurses libelf \
                openssl pahole bison zstd git wget xz
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y gcc make flex bison bc ncurses-devel openssl-devel \
                elfutils-libelf-devel dwarves zstd git wget xz
        else
            log_error "Could not install dependencies. Please install manually."
            exit 1
        fi
    fi

    log_success "All dependencies satisfied"
}

download_kernel() {
    log_info "Setting up kernel source..."

    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"

    if [ -d "zen-kernel" ]; then
        log_info "Updating existing Zen kernel source..."
        cd zen-kernel
        git fetch --all
        git checkout "v${KERNEL_VERSION}-zen1" 2>/dev/null || git checkout "zen-main"
        cd ..
    else
        log_info "Cloning Zen kernel repository..."
        git clone --depth 1 --branch "v${KERNEL_VERSION}-zen1" "$ZEN_REPO" zen-kernel 2>/dev/null || \
        git clone --depth 1 "$ZEN_REPO" zen-kernel
    fi

    log_success "Kernel source ready"
}

download_bore_patches() {
    log_info "Downloading BORE scheduler patches..."

    cd "$BUILD_DIR"

    if [ -d "bore-scheduler" ]; then
        log_info "Updating existing BORE patches..."
        cd bore-scheduler
        git pull
        cd ..
    else
        log_info "Cloning BORE scheduler repository..."
        git clone --depth 1 "$BORE_REPO" bore-scheduler
    fi

    log_success "BORE patches ready"
}

apply_patches() {
    log_info "Applying patches..."

    cd "$BUILD_DIR/zen-kernel"

    # Apply BORE scheduler patches
    if [ -d "$BUILD_DIR/bore-scheduler/patches" ]; then
        log_info "Applying BORE scheduler patches..."
        for patch in "$BUILD_DIR/bore-scheduler/patches"/*.patch; do
            if [ -f "$patch" ]; then
                log_info "Applying: $(basename "$patch")"
                patch -p1 < "$patch" || log_warning "Patch may have already been applied: $patch"
            fi
        done
    fi

    # Apply Winux custom patches
    if [ -d "$PATCHES_DIR" ]; then
        log_info "Applying Winux custom patches..."
        for patch in "$PATCHES_DIR"/*.patch; do
            if [ -f "$patch" ]; then
                log_info "Applying: $(basename "$patch")"
                # Placeholder patches won't apply, just inform
                if grep -q "placeholder" "$patch" 2>/dev/null; then
                    log_info "Skipping placeholder patch: $(basename "$patch")"
                else
                    patch -p1 < "$patch" || log_warning "Patch may have already been applied: $patch"
                fi
            fi
        done
    fi

    log_success "Patches applied"
}

configure_kernel() {
    log_info "Configuring kernel..."

    cd "$BUILD_DIR/zen-kernel"

    # Start with zen default config or current running config
    if [ -f "$CONFIG_DIR/winux-kernel-config" ]; then
        log_info "Using Winux kernel configuration..."
        cp "$CONFIG_DIR/winux-kernel-config" .config
    elif [ -f /proc/config.gz ]; then
        log_info "Using current running kernel config as base..."
        zcat /proc/config.gz > .config
    else
        log_info "Generating default config..."
        make defconfig
    fi

    # Apply Winux-specific options
    log_info "Applying Winux-specific kernel options..."

    # Enable BORE scheduler
    ./scripts/config --enable CONFIG_SCHED_BORE
    ./scripts/config --enable CONFIG_BORE_DEFAULT_ON

    # Preemption model
    ./scripts/config --disable CONFIG_PREEMPT_NONE
    ./scripts/config --disable CONFIG_PREEMPT_VOLUNTARY
    ./scripts/config --enable CONFIG_PREEMPT
    ./scripts/config --enable CONFIG_PREEMPT_DYNAMIC

    # Timer frequency
    ./scripts/config --disable CONFIG_HZ_100
    ./scripts/config --disable CONFIG_HZ_250
    ./scripts/config --disable CONFIG_HZ_300
    ./scripts/config --enable CONFIG_HZ_1000
    ./scripts/config --set-val CONFIG_HZ 1000

    # Zen interactive
    ./scripts/config --enable CONFIG_ZEN_INTERACTIVE
    ./scripts/config --enable CONFIG_ZEN_TUNABLES

    # GPU drivers
    ./scripts/config --module CONFIG_DRM_NOUVEAU
    ./scripts/config --module CONFIG_DRM_AMDGPU
    ./scripts/config --enable CONFIG_DRM_AMDGPU_SI
    ./scripts/config --enable CONFIG_DRM_AMDGPU_CIK
    ./scripts/config --enable CONFIG_DRM_AMD_DC
    ./scripts/config --module CONFIG_DRM_I915
    ./scripts/config --module CONFIG_DRM_XE

    # Filesystems
    ./scripts/config --enable CONFIG_BTRFS_FS
    ./scripts/config --enable CONFIG_NTFS3_FS
    ./scripts/config --enable CONFIG_EXFAT_FS

    # Gaming optimizations
    ./scripts/config --enable CONFIG_FUTEX
    ./scripts/config --enable CONFIG_FUTEX2

    # Performance
    ./scripts/config --enable CONFIG_CC_OPTIMIZE_FOR_PERFORMANCE
    ./scripts/config --disable CONFIG_CC_OPTIMIZE_FOR_SIZE

    # LTO (if using clang)
    if command -v clang &> /dev/null; then
        ./scripts/config --enable CONFIG_LTO_CLANG_THIN
    fi

    # BBR TCP
    ./scripts/config --enable CONFIG_TCP_CONG_BBR
    ./scripts/config --enable CONFIG_DEFAULT_BBR

    # Disable debug for performance
    ./scripts/config --disable CONFIG_DEBUG_KERNEL
    ./scripts/config --disable CONFIG_DEBUG_INFO
    ./scripts/config --enable CONFIG_DEBUG_INFO_NONE

    # Set local version
    ./scripts/config --set-str CONFIG_LOCALVERSION "-winux-zen"

    # Update config with new options
    make olddefconfig

    # Optional: Interactive config
    if [ "$INTERACTIVE_CONFIG" = "true" ]; then
        log_info "Opening interactive configuration..."
        make menuconfig
    fi

    log_success "Kernel configured"
}

build_kernel() {
    log_info "Building kernel (using $JOBS parallel jobs)..."

    cd "$BUILD_DIR/zen-kernel"

    # Clean if requested
    if [ "$CLEAN_BUILD" = "true" ]; then
        log_info "Cleaning previous build..."
        make clean
    fi

    # Build with timing
    local start_time=$(date +%s)

    # Use clang if available for better optimization
    if command -v clang &> /dev/null && [ "$USE_CLANG" = "true" ]; then
        log_info "Building with Clang/LLVM..."
        make -j"$JOBS" LLVM=1 LLVM_IAS=1
    else
        log_info "Building with GCC..."
        make -j"$JOBS"
    fi

    local end_time=$(date +%s)
    local build_time=$((end_time - start_time))

    log_success "Kernel built successfully in $((build_time / 60))m $((build_time % 60))s"
}

install_kernel() {
    log_info "Installing kernel..."

    cd "$BUILD_DIR/zen-kernel"

    # Get kernel version string
    local kernel_release=$(make kernelrelease)
    log_info "Installing kernel version: $kernel_release"

    # Install modules
    if [ "$INSTALL_MODULES" = "true" ]; then
        log_info "Installing kernel modules..."
        sudo make modules_install
    fi

    # Install kernel
    if [ "$INSTALL_KERNEL" = "true" ]; then
        log_info "Installing kernel image..."
        sudo make install

        # Update bootloader
        if command -v grub-mkconfig &> /dev/null; then
            log_info "Updating GRUB configuration..."
            sudo grub-mkconfig -o /boot/grub/grub.cfg
        elif command -v bootctl &> /dev/null; then
            log_info "systemd-boot detected, please manually add boot entry"
        fi
    fi

    log_success "Kernel installed: $kernel_release"
}

create_dkms_conf() {
    log_info "Setting up DKMS for external modules..."

    local kernel_release=$(cd "$BUILD_DIR/zen-kernel" && make kernelrelease)

    # Create directory for kernel headers
    sudo mkdir -p "/usr/src/linux-headers-$kernel_release"

    # Copy headers needed for DKMS
    cd "$BUILD_DIR/zen-kernel"
    sudo cp -a include scripts arch/x86/include "/usr/src/linux-headers-$kernel_release/"
    sudo cp Module.symvers "/usr/src/linux-headers-$kernel_release/"
    sudo cp .config "/usr/src/linux-headers-$kernel_release/"
    sudo cp Makefile "/usr/src/linux-headers-$kernel_release/"

    # Link headers
    sudo ln -sf "/usr/src/linux-headers-$kernel_release" "/lib/modules/$kernel_release/build"

    log_success "DKMS headers installed"
}

cleanup() {
    if [ "$KEEP_BUILD" != "true" ]; then
        log_info "Cleaning up build directory..."
        rm -rf "$BUILD_DIR"
        log_success "Cleanup complete"
    else
        log_info "Build directory preserved at: $BUILD_DIR"
    fi
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -v, --version VERSION   Kernel version to build (default: $KERNEL_VERSION)"
    echo "  -j, --jobs JOBS         Number of parallel jobs (default: $(nproc))"
    echo "  -c, --clean             Clean build (remove previous artifacts)"
    echo "  -i, --interactive       Open menuconfig for interactive configuration"
    echo "  --clang                 Use Clang/LLVM for compilation"
    echo "  --no-install            Build only, don't install"
    echo "  --keep                  Keep build directory after installation"
    echo "  -h, --help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                      # Build and install with defaults"
    echo "  $0 -v 6.8 -j 8          # Build kernel 6.8 with 8 jobs"
    echo "  $0 --clang --clean      # Clean build with Clang"
}

#===============================================================================
# Main
#===============================================================================

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                KERNEL_VERSION="$2"
                shift 2
                ;;
            -j|--jobs)
                JOBS="$2"
                shift 2
                ;;
            -c|--clean)
                CLEAN_BUILD=true
                shift
                ;;
            -i|--interactive)
                INTERACTIVE_CONFIG=true
                shift
                ;;
            --clang)
                USE_CLANG=true
                shift
                ;;
            --no-install)
                INSTALL_MODULES=false
                INSTALL_KERNEL=false
                shift
                ;;
            --keep)
                KEEP_BUILD=true
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

    log_info "Building Winux OS Kernel"
    log_info "Kernel Version: $KERNEL_VERSION"
    log_info "Parallel Jobs: $JOBS"
    log_info "Build Directory: $BUILD_DIR"
    echo ""

    # Require root for installation
    if [ "$INSTALL_KERNEL" = "true" ] && [ "$EUID" -ne 0 ]; then
        log_warning "Installation requires root privileges. You may be prompted for sudo password."
    fi

    # Execute build steps
    check_dependencies
    download_kernel
    download_bore_patches
    apply_patches
    configure_kernel
    build_kernel

    if [ "$INSTALL_KERNEL" = "true" ] || [ "$INSTALL_MODULES" = "true" ]; then
        install_kernel
        create_dkms_conf
    fi

    cleanup

    echo ""
    log_success "═══════════════════════════════════════════════════════════════"
    log_success "Winux OS Kernel build complete!"
    log_success "Reboot to use the new kernel."
    log_success "═══════════════════════════════════════════════════════════════"
}

# Run main function
main "$@"
