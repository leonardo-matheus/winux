#!/bin/bash
#===============================================================================
# wine-setup.sh - Winux OS Wine Configuration Script
# Configures Wine prefix automatically with gaming optimizations
#===============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
WINEPREFIX="${WINEPREFIX:-$HOME/.wine}"
WINEARCH="${WINEARCH:-win64}"
WINETRICKS_CACHE="${WINETRICKS_CACHE:-$HOME/.cache/winetricks}"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check dependencies
check_dependencies() {
    local missing_deps=()

    for cmd in wine winetricks wget; do
        if ! command -v "$cmd" &>/dev/null; then
            missing_deps+=("$cmd")
        fi
    done

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Install them with: sudo pacman -S wine winetricks wget"
        exit 1
    fi

    log_success "All dependencies found"
}

# Create Wine prefix
create_prefix() {
    log_info "Creating Wine prefix at $WINEPREFIX (arch: $WINEARCH)"

    export WINEPREFIX
    export WINEARCH

    # Initialize prefix silently
    DISPLAY="" wineboot --init 2>/dev/null || true

    # Wait for wineserver to finish
    wineserver --wait 2>/dev/null || true

    log_success "Wine prefix created"
}

# Install Visual C++ Runtimes
install_vcrun() {
    log_info "Installing Visual C++ Runtimes..."

    local vcrun_versions=(
        "vcrun2008"
        "vcrun2010"
        "vcrun2012"
        "vcrun2013"
        "vcrun2015"
        "vcrun2017"
        "vcrun2019"
        "vcrun2022"
    )

    for vcrun in "${vcrun_versions[@]}"; do
        log_info "Installing $vcrun..."
        if winetricks -q "$vcrun" 2>/dev/null; then
            log_success "$vcrun installed"
        else
            log_warn "Failed to install $vcrun (may already be installed or not needed)"
        fi
    done
}

# Install .NET Framework
install_dotnet() {
    log_info "Installing .NET Framework components..."

    local dotnet_versions=(
        "dotnet40"
        "dotnet45"
        "dotnet48"
        "dotnetcoredesktop6"
    )

    for dotnet in "${dotnet_versions[@]}"; do
        log_info "Installing $dotnet..."
        if winetricks -q "$dotnet" 2>/dev/null; then
            log_success "$dotnet installed"
        else
            log_warn "Failed to install $dotnet (may already be installed or not available)"
        fi
    done
}

# Install DXVK (DirectX 9/10/11 to Vulkan)
install_dxvk() {
    log_info "Installing DXVK..."

    # Check for Vulkan support
    if ! command -v vulkaninfo &>/dev/null; then
        log_warn "vulkaninfo not found. Make sure Vulkan drivers are installed."
    fi

    if winetricks -q dxvk 2>/dev/null; then
        log_success "DXVK installed"
    else
        log_warn "DXVK installation via winetricks failed, attempting manual install..."
        install_dxvk_manual
    fi
}

# Manual DXVK installation
install_dxvk_manual() {
    local dxvk_version="2.3.1"
    local dxvk_url="https://github.com/doitsujin/dxvk/releases/download/v${dxvk_version}/dxvk-${dxvk_version}.tar.gz"
    local temp_dir=$(mktemp -d)

    log_info "Downloading DXVK v${dxvk_version}..."

    if wget -q "$dxvk_url" -O "$temp_dir/dxvk.tar.gz"; then
        tar -xzf "$temp_dir/dxvk.tar.gz" -C "$temp_dir"

        if [[ -f "$temp_dir/dxvk-${dxvk_version}/setup_dxvk.sh" ]]; then
            cd "$temp_dir/dxvk-${dxvk_version}"
            WINEPREFIX="$WINEPREFIX" ./setup_dxvk.sh install
            log_success "DXVK installed manually"
        fi
    else
        log_error "Failed to download DXVK"
    fi

    rm -rf "$temp_dir"
}

# Install VKD3D-Proton (DirectX 12 to Vulkan)
install_vkd3d() {
    log_info "Installing VKD3D-Proton..."

    if winetricks -q vkd3d 2>/dev/null; then
        log_success "VKD3D installed"
    else
        log_warn "VKD3D installation via winetricks failed, attempting manual install..."
        install_vkd3d_manual
    fi
}

# Manual VKD3D-Proton installation
install_vkd3d_manual() {
    local vkd3d_version="2.11.1"
    local vkd3d_url="https://github.com/HansKristian-Work/vkd3d-proton/releases/download/v${vkd3d_version}/vkd3d-proton-${vkd3d_version}.tar.zst"
    local temp_dir=$(mktemp -d)

    log_info "Downloading VKD3D-Proton v${vkd3d_version}..."

    if wget -q "$vkd3d_url" -O "$temp_dir/vkd3d.tar.zst"; then
        # Check if zstd is available
        if command -v zstd &>/dev/null; then
            zstd -d "$temp_dir/vkd3d.tar.zst" -o "$temp_dir/vkd3d.tar"
            tar -xf "$temp_dir/vkd3d.tar" -C "$temp_dir"

            if [[ -f "$temp_dir/vkd3d-proton-${vkd3d_version}/setup_vkd3d_proton.sh" ]]; then
                cd "$temp_dir/vkd3d-proton-${vkd3d_version}"
                WINEPREFIX="$WINEPREFIX" ./setup_vkd3d_proton.sh install
                log_success "VKD3D-Proton installed manually"
            fi
        else
            log_warn "zstd not found, skipping VKD3D-Proton manual install"
        fi
    else
        log_error "Failed to download VKD3D-Proton"
    fi

    rm -rf "$temp_dir"
}

# Apply registry tweaks for gaming
apply_registry_tweaks() {
    log_info "Applying registry tweaks for gaming..."

    local reg_file=$(mktemp --suffix=.reg)

    cat > "$reg_file" << 'EOF'
Windows Registry Editor Version 5.00

; Disable Wine debug messages for performance
[HKEY_CURRENT_USER\Software\Wine\Debug]
"RelayExclude"="ntdll.RtlEnterCriticalSection;ntdll.RtlLeaveCriticalSection;kernel32.GetCurrentThreadId"

; Enable CSMT (Command Stream Multi-Threading)
[HKEY_CURRENT_USER\Software\Wine\Direct3D]
"csmt"=dword:00000003
"MaxVersionGL"=dword:00040006
"UseGLSL"="enabled"
"DirectDrawRenderer"="opengl"
"OffscreenRenderingMode"="fbo"
"VideoMemorySize"="8192"

; Disable Window Manager decorations for fullscreen games
[HKEY_CURRENT_USER\Software\Wine\X11 Driver]
"Decorated"="N"
"Managed"="Y"
"GrabFullscreen"="Y"
"UseTakeFocus"="N"

; Disable Gecko and Mono prompts
[HKEY_CURRENT_USER\Software\Wine\DllOverrides]
"winemenubuilder.exe"=""

; Windows version spoofing (Windows 10)
[HKEY_LOCAL_MACHINE\Software\Microsoft\Windows NT\CurrentVersion]
"ProductName"="Windows 10 Pro"
"CSDVersion"=""
"CurrentBuild"="19041"
"CurrentBuildNumber"="19041"
"CurrentVersion"="6.3"

[HKEY_LOCAL_MACHINE\System\CurrentControlSet\Control\Windows]
"CSDVersion"=dword:00000000

; Disable crash dialogs
[HKEY_CURRENT_USER\Software\Wine\WineDbg]
"ShowCrashDialog"=dword:00000000

; Mouse settings for gaming
[HKEY_CURRENT_USER\Software\Wine\DirectInput]
"MouseWarpOverride"="enable"

; Audio settings
[HKEY_CURRENT_USER\Software\Wine\Drivers]
"Audio"="pulse,alsa"

; Font smoothing
[HKEY_CURRENT_USER\Control Panel\Desktop]
"FontSmoothing"="2"
"FontSmoothingType"=dword:00000002
"FontSmoothingGamma"=dword:00000578
"FontSmoothingOrientation"=dword:00000001

; DPI settings
[HKEY_CURRENT_USER\Control Panel\Desktop]
"LogPixels"=dword:00000060

; Performance tweaks
[HKEY_LOCAL_MACHINE\System\CurrentControlSet\Control\Session Manager\Memory Management]
"ClearPageFileAtShutdown"=dword:00000000
"DisablePagingExecutive"=dword:00000001
"LargeSystemCache"=dword:00000001

; Game mode settings
[HKEY_CURRENT_USER\Software\Microsoft\GameBar]
"AutoGameModeEnabled"=dword:00000001
"AllowAutoGameMode"=dword:00000001
EOF

    # Import registry file
    wine regedit "$reg_file" 2>/dev/null
    wineserver --wait 2>/dev/null || true

    rm -f "$reg_file"
    log_success "Registry tweaks applied"
}

# Install additional gaming components
install_gaming_extras() {
    log_info "Installing additional gaming components..."

    local components=(
        "corefonts"
        "tahoma"
        "d3dx9"
        "d3dx10"
        "d3dx11_43"
        "d3dcompiler_43"
        "d3dcompiler_47"
        "xact"
        "xinput"
        "physx"
        "faudio"
    )

    for component in "${components[@]}"; do
        log_info "Installing $component..."
        if winetricks -q "$component" 2>/dev/null; then
            log_success "$component installed"
        else
            log_warn "Failed to install $component"
        fi
    done
}

# Configure DLL overrides
configure_dll_overrides() {
    log_info "Configuring DLL overrides for DXVK/VKD3D..."

    local reg_file=$(mktemp --suffix=.reg)

    cat > "$reg_file" << 'EOF'
Windows Registry Editor Version 5.00

[HKEY_CURRENT_USER\Software\Wine\DllOverrides]
"d3d9"="native,builtin"
"d3d10"="native,builtin"
"d3d10_1"="native,builtin"
"d3d10core"="native,builtin"
"d3d11"="native,builtin"
"d3d12"="native,builtin"
"d3d12core"="native,builtin"
"dxgi"="native,builtin"
"nvapi"="disabled"
"nvapi64"="disabled"
"nvcuda"="disabled"
"nvcuvid"="disabled"
EOF

    wine regedit "$reg_file" 2>/dev/null
    wineserver --wait 2>/dev/null || true

    rm -f "$reg_file"
    log_success "DLL overrides configured"
}

# Show usage
show_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Winux OS Wine Setup Script - Configure Wine prefix for gaming

Options:
    -p, --prefix PATH     Set Wine prefix path (default: ~/.wine)
    -a, --arch ARCH       Set Wine architecture: win32 or win64 (default: win64)
    -c, --components      Install components only (skip prefix creation)
    -r, --registry        Apply registry tweaks only
    -f, --full            Full installation (default)
    -m, --minimal         Minimal installation (vcrun + dxvk only)
    -h, --help            Show this help message

Components installed:
    - Visual C++ Runtimes (2008-2022)
    - .NET Framework (4.0, 4.5, 4.8, Core 6)
    - DXVK (DirectX 9/10/11 to Vulkan)
    - VKD3D-Proton (DirectX 12 to Vulkan)
    - Gaming extras (fonts, DirectX components, etc.)
    - Registry optimizations for gaming

Examples:
    $(basename "$0")                          # Full installation to ~/.wine
    $(basename "$0") -p ~/.wine-gaming        # Custom prefix
    $(basename "$0") -a win32 -m              # 32-bit minimal installation
    $(basename "$0") -r                       # Apply registry tweaks only

Environment variables:
    WINEPREFIX          Wine prefix path
    WINEARCH            Wine architecture (win32/win64)
    WINETRICKS_CACHE    Winetricks download cache path

EOF
}

# Main function
main() {
    local mode="full"
    local components_only=false
    local registry_only=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -p|--prefix)
                WINEPREFIX="$2"
                shift 2
                ;;
            -a|--arch)
                WINEARCH="$2"
                shift 2
                ;;
            -c|--components)
                components_only=true
                shift
                ;;
            -r|--registry)
                registry_only=true
                shift
                ;;
            -f|--full)
                mode="full"
                shift
                ;;
            -m|--minimal)
                mode="minimal"
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    echo "=============================================="
    echo "  Winux OS Wine Setup Script"
    echo "=============================================="
    echo ""
    log_info "Prefix: $WINEPREFIX"
    log_info "Architecture: $WINEARCH"
    log_info "Mode: $mode"
    echo ""

    # Check dependencies
    check_dependencies

    # Registry only mode
    if [[ "$registry_only" == true ]]; then
        apply_registry_tweaks
        log_success "Done!"
        exit 0
    fi

    # Create prefix unless components only
    if [[ "$components_only" == false ]]; then
        create_prefix
    fi

    # Install components based on mode
    case "$mode" in
        minimal)
            install_vcrun
            install_dxvk
            configure_dll_overrides
            apply_registry_tweaks
            ;;
        full)
            install_vcrun
            install_dotnet
            install_dxvk
            install_vkd3d
            install_gaming_extras
            configure_dll_overrides
            apply_registry_tweaks
            ;;
    esac

    echo ""
    log_success "Wine setup complete!"
    log_info "Wine prefix ready at: $WINEPREFIX"
    echo ""
    echo "To use this prefix, set:"
    echo "  export WINEPREFIX=\"$WINEPREFIX\""
    echo ""
}

# Run main function
main "$@"
