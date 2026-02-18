#!/bin/bash
#===============================================================================
# proton-install.sh - Winux OS Proton-GE Installer
# Downloads and installs the latest Proton-GE for Steam
#===============================================================================

set -euo pipefail

# Script version
VERSION="1.0.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
GITHUB_API="https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases"
STEAM_COMPAT_DIR="$HOME/.steam/root/compatibilitytools.d"
STEAM_COMPAT_DIR_ALT="$HOME/.steam/steam/compatibilitytools.d"
TEMP_DIR="${TMPDIR:-/tmp}/proton-ge-install"

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_progress() { echo -e "${CYAN}[...]${NC} $1"; }

# Check dependencies
check_dependencies() {
    local missing_deps=()

    for cmd in curl jq tar; do
        if ! command -v "$cmd" &>/dev/null; then
            missing_deps+=("$cmd")
        fi
    done

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Install them with: sudo pacman -S curl jq tar"
        exit 1
    fi

    log_success "All dependencies found"
}

# Detect Steam compatibility directory
detect_steam_dir() {
    # Check common Steam installation paths
    local possible_dirs=(
        "$HOME/.steam/root/compatibilitytools.d"
        "$HOME/.steam/steam/compatibilitytools.d"
        "$HOME/.local/share/Steam/compatibilitytools.d"
        "$HOME/.var/app/com.valvesoftware.Steam/.local/share/Steam/compatibilitytools.d"
    )

    for dir in "${possible_dirs[@]}"; do
        # Check if parent Steam directory exists
        local parent_dir=$(dirname "$dir")
        if [[ -d "$parent_dir" ]]; then
            mkdir -p "$dir"
            echo "$dir"
            return
        fi
    done

    # Fallback to default
    mkdir -p "$STEAM_COMPAT_DIR"
    echo "$STEAM_COMPAT_DIR"
}

# Get latest Proton-GE release info
get_latest_release() {
    log_progress "Fetching latest Proton-GE release info..."

    local release_info
    release_info=$(curl -sL "$GITHUB_API/latest")

    if [[ -z "$release_info" ]] || [[ "$release_info" == "null" ]]; then
        log_error "Failed to fetch release information from GitHub"
        exit 1
    fi

    echo "$release_info"
}

# Get specific release by tag
get_release_by_tag() {
    local tag="$1"
    log_progress "Fetching Proton-GE release $tag..."

    local release_info
    release_info=$(curl -sL "$GITHUB_API/tags/$tag")

    if [[ -z "$release_info" ]] || [[ "$release_info" == "null" ]]; then
        log_error "Failed to fetch release $tag from GitHub"
        exit 1
    fi

    echo "$release_info"
}

# List available releases
list_releases() {
    local count="${1:-10}"
    log_info "Fetching available Proton-GE releases..."

    local releases
    releases=$(curl -sL "$GITHUB_API?per_page=$count")

    if [[ -z "$releases" ]] || [[ "$releases" == "null" ]]; then
        log_error "Failed to fetch releases from GitHub"
        exit 1
    fi

    echo ""
    echo "Available Proton-GE Releases:"
    echo "=============================="

    echo "$releases" | jq -r '.[] | "\(.tag_name)\t\(.published_at | split("T")[0])\t\(.assets[0].size / 1048576 | floor)MB"' | \
        while IFS=$'\t' read -r tag date size; do
            printf "  %-20s  %s  %s\n" "$tag" "$date" "$size"
        done

    echo ""
}

# List installed versions
list_installed() {
    local install_dir=$(detect_steam_dir)

    echo ""
    echo "Installed Proton-GE Versions:"
    echo "=============================="

    if [[ -d "$install_dir" ]]; then
        local found=false
        for version_dir in "$install_dir"/GE-Proton*; do
            if [[ -d "$version_dir" ]]; then
                found=true
                local version_name=$(basename "$version_dir")
                local install_date=$(stat -c %y "$version_dir" 2>/dev/null | cut -d' ' -f1 || echo "unknown")
                printf "  %-20s  (installed: %s)\n" "$version_name" "$install_date"
            fi
        done

        if [[ "$found" == false ]]; then
            echo "  No Proton-GE versions installed"
        fi
    else
        echo "  Steam compatibility directory not found"
    fi

    echo ""
    echo "Install directory: $install_dir"
    echo ""
}

# Parse release info
parse_release_info() {
    local release_info="$1"

    local tag_name=$(echo "$release_info" | jq -r '.tag_name')
    local download_url=$(echo "$release_info" | jq -r '.assets[] | select(.name | endswith(".tar.gz")) | .browser_download_url')
    local checksum_url=$(echo "$release_info" | jq -r '.assets[] | select(.name | endswith(".sha512sum")) | .browser_download_url')
    local file_size=$(echo "$release_info" | jq -r '.assets[] | select(.name | endswith(".tar.gz")) | .size')

    if [[ -z "$download_url" ]] || [[ "$download_url" == "null" ]]; then
        log_error "Could not find download URL in release info"
        exit 1
    fi

    echo "$tag_name|$download_url|$checksum_url|$file_size"
}

# Download Proton-GE
download_proton() {
    local url="$1"
    local output="$2"
    local file_size="$3"

    local file_size_mb=$((file_size / 1048576))
    log_progress "Downloading Proton-GE (${file_size_mb}MB)..."

    # Download with progress
    if curl -L --progress-bar "$url" -o "$output"; then
        log_success "Download complete"
        return 0
    else
        log_error "Download failed"
        return 1
    fi
}

# Verify checksum
verify_checksum() {
    local archive="$1"
    local checksum_url="$2"

    if [[ -z "$checksum_url" ]] || [[ "$checksum_url" == "null" ]]; then
        log_warn "No checksum file available, skipping verification"
        return 0
    fi

    log_progress "Verifying checksum..."

    local checksum_file="$TEMP_DIR/checksum.sha512sum"

    if curl -sL "$checksum_url" -o "$checksum_file"; then
        cd "$TEMP_DIR"
        if sha512sum -c "$checksum_file" &>/dev/null; then
            log_success "Checksum verified"
            return 0
        else
            log_error "Checksum verification failed"
            return 1
        fi
    else
        log_warn "Failed to download checksum file, skipping verification"
        return 0
    fi
}

# Extract Proton-GE
extract_proton() {
    local archive="$1"
    local install_dir="$2"

    log_progress "Extracting Proton-GE..."

    if tar -xzf "$archive" -C "$install_dir"; then
        log_success "Extraction complete"
        return 0
    else
        log_error "Extraction failed"
        return 1
    fi
}

# Install Proton-GE
install_proton() {
    local release_info="$1"
    local install_dir="$2"

    # Parse release info
    local parsed
    parsed=$(parse_release_info "$release_info")

    local tag_name=$(echo "$parsed" | cut -d'|' -f1)
    local download_url=$(echo "$parsed" | cut -d'|' -f2)
    local checksum_url=$(echo "$parsed" | cut -d'|' -f3)
    local file_size=$(echo "$parsed" | cut -d'|' -f4)

    log_info "Installing $tag_name"

    # Check if already installed
    if [[ -d "$install_dir/$tag_name" ]]; then
        log_warn "$tag_name is already installed"
        read -p "Reinstall? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Installation cancelled"
            return 0
        fi
        rm -rf "$install_dir/$tag_name"
    fi

    # Create temp directory
    mkdir -p "$TEMP_DIR"

    local archive_name=$(basename "$download_url")
    local archive_path="$TEMP_DIR/$archive_name"

    # Download
    if ! download_proton "$download_url" "$archive_path" "$file_size"; then
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    # Verify checksum
    if ! verify_checksum "$archive_path" "$checksum_url"; then
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    # Extract
    if ! extract_proton "$archive_path" "$install_dir"; then
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    # Cleanup
    rm -rf "$TEMP_DIR"

    log_success "$tag_name installed successfully"
    log_info "Location: $install_dir/$tag_name"
    echo ""
    log_info "Restart Steam to use the new Proton version"
    log_info "Select it in Steam: Game Properties > Compatibility > Force specific compatibility tool"
}

# Uninstall Proton-GE version
uninstall_proton() {
    local version="$1"
    local install_dir=$(detect_steam_dir)

    local version_dir="$install_dir/$version"

    if [[ ! -d "$version_dir" ]]; then
        # Try with GE-Proton prefix
        version_dir="$install_dir/GE-Proton$version"
        if [[ ! -d "$version_dir" ]]; then
            log_error "Version not found: $version"
            list_installed
            exit 1
        fi
    fi

    log_warn "This will remove: $version_dir"
    read -p "Continue? [y/N] " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$version_dir"
        log_success "Removed $(basename "$version_dir")"
    else
        log_info "Uninstall cancelled"
    fi
}

# Update to latest version
update_proton() {
    local install_dir=$(detect_steam_dir)

    # Get latest release
    local release_info
    release_info=$(get_latest_release)

    local latest_tag=$(echo "$release_info" | jq -r '.tag_name')

    # Check if already installed
    if [[ -d "$install_dir/$latest_tag" ]]; then
        log_success "Already running the latest version: $latest_tag"
        return 0
    fi

    log_info "New version available: $latest_tag"
    install_proton "$release_info" "$install_dir"
}

# Clean old versions
clean_old_versions() {
    local keep_count="${1:-2}"
    local install_dir=$(detect_steam_dir)

    log_info "Cleaning old Proton-GE versions (keeping $keep_count newest)..."

    # Get all versions sorted by modification time (newest first)
    local versions=()
    while IFS= read -r -d '' dir; do
        versions+=("$dir")
    done < <(find "$install_dir" -maxdepth 1 -name "GE-Proton*" -type d -printf '%T@ %p\0' | \
             sort -rn | cut -d' ' -f2-)

    local total=${#versions[@]}

    if [[ $total -le $keep_count ]]; then
        log_info "Only $total version(s) installed, nothing to clean"
        return 0
    fi

    local to_remove=$((total - keep_count))
    log_warn "Will remove $to_remove old version(s)"

    for ((i=keep_count; i<total; i++)); do
        local version_dir="${versions[$i]}"
        log_info "Removing: $(basename "$version_dir")"
        rm -rf "$version_dir"
    done

    log_success "Cleanup complete"
}

# Show usage
show_usage() {
    cat << EOF
Usage: $(basename "$0") [COMMAND] [OPTIONS]

Winux OS Proton-GE Installer - Install and manage Proton-GE for Steam

Commands:
    install [VERSION]   Install latest or specific Proton-GE version
    update              Update to the latest Proton-GE version
    uninstall VERSION   Remove a specific Proton-GE version
    list                List available releases on GitHub
    installed           List locally installed versions
    clean [COUNT]       Remove old versions, keeping COUNT newest (default: 2)

Options:
    -d, --dir PATH      Custom installation directory
    -f, --force         Force installation without prompts
    -h, --help          Show this help message
    -v, --version       Show script version

Examples:
    $(basename "$0") install                    # Install latest version
    $(basename "$0") install GE-Proton9-4      # Install specific version
    $(basename "$0") update                     # Update to latest
    $(basename "$0") list                       # Show available versions
    $(basename "$0") installed                  # Show installed versions
    $(basename "$0") uninstall GE-Proton8-25   # Remove specific version
    $(basename "$0") clean 3                    # Keep only 3 newest versions

Notes:
    - Restart Steam after installation to see new Proton versions
    - Select Proton-GE in Steam: Right-click game > Properties >
      Compatibility > Force specific Steam Play compatibility tool

EOF
}

# Show version
show_version() {
    echo "proton-install.sh version $VERSION"
    echo "Winux OS Proton-GE Installer"
}

# Main function
main() {
    local command=""
    local custom_dir=""
    local force=false
    local args=()

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -d|--dir)
                custom_dir="$2"
                shift 2
                ;;
            -f|--force)
                force=true
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            -v|--version)
                show_version
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
            *)
                if [[ -z "$command" ]]; then
                    command="$1"
                else
                    args+=("$1")
                fi
                shift
                ;;
        esac
    done

    # Default command
    command="${command:-install}"

    echo "=============================================="
    echo "  Winux OS Proton-GE Installer"
    echo "=============================================="
    echo ""

    # Check dependencies (except for list commands)
    if [[ "$command" != "installed" ]]; then
        check_dependencies
    fi

    # Detect installation directory
    local install_dir
    if [[ -n "$custom_dir" ]]; then
        install_dir="$custom_dir"
        mkdir -p "$install_dir"
    else
        install_dir=$(detect_steam_dir)
    fi

    log_info "Installation directory: $install_dir"
    echo ""

    # Execute command
    case "$command" in
        install)
            if [[ ${#args[@]} -gt 0 ]]; then
                local release_info
                release_info=$(get_release_by_tag "${args[0]}")
                install_proton "$release_info" "$install_dir"
            else
                local release_info
                release_info=$(get_latest_release)
                install_proton "$release_info" "$install_dir"
            fi
            ;;
        update)
            update_proton
            ;;
        uninstall|remove)
            if [[ ${#args[@]} -eq 0 ]]; then
                log_error "Please specify a version to uninstall"
                list_installed
                exit 1
            fi
            uninstall_proton "${args[0]}"
            ;;
        list|available)
            local count="${args[0]:-10}"
            list_releases "$count"
            ;;
        installed|local)
            list_installed
            ;;
        clean|cleanup)
            local keep="${args[0]:-2}"
            clean_old_versions "$keep"
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
