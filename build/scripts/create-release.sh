#!/bin/bash
# =============================================================================
# Winux OS - Release Creation Script
# =============================================================================
# Creates a complete release package including ISO, checksums, and signatures
# Usage: ./create-release.sh [VERSION] [CODENAME]
# Example: ./create-release.sh 1.0.0 aurora
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

VERSION="${1:-1.0.0}"
CODENAME="${2:-aurora}"
BUILD_DATE="$(date +%Y%m%d)"
BUILD_ID="${BUILD_DATE}.001"

OUTPUT_DIR="${OUTPUT_DIR:-${PROJECT_ROOT}/release}"
BUILD_DIR="${BUILD_DIR:-/tmp/winux-build}"
SIGNING_KEY="${SIGNING_KEY:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# -----------------------------------------------------------------------------
# Logging Functions
# -----------------------------------------------------------------------------
log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }
log_section() { echo -e "\n${CYAN}=== $1 ===${NC}\n"; }

# -----------------------------------------------------------------------------
# Banner
# -----------------------------------------------------------------------------
show_banner() {
    echo ""
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║                                                                      ║"
    echo "║   ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗    ██████╗ ███████╗      ║"
    echo "║   ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝   ██╔═══██╗██╔════╝      ║"
    echo "║   ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝    ██║   ██║███████╗      ║"
    echo "║   ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗    ██║   ██║╚════██║      ║"
    echo "║   ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗   ╚██████╔╝███████║      ║"
    echo "║    ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝    ╚═════╝ ╚══════╝      ║"
    echo "║                                                                      ║"
    echo "║                    RELEASE CREATION SCRIPT                           ║"
    echo "║                                                                      ║"
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
    log_info "Version: ${VERSION}"
    log_info "Codename: ${CODENAME}"
    log_info "Build ID: ${BUILD_ID}"
    log_info "Output: ${OUTPUT_DIR}"
    echo ""
}

# -----------------------------------------------------------------------------
# Check Prerequisites
# -----------------------------------------------------------------------------
check_prerequisites() {
    log_section "Checking Prerequisites"

    local missing=()

    # Check required tools
    local tools=(cargo rustc git sha256sum md5sum gpg xorriso mksquashfs debootstrap)
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &>/dev/null; then
            missing+=("$tool")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing[*]}"
        log_info "Install them with: sudo apt install ${missing[*]}"
        exit 1
    fi

    # Check if running as root for ISO build
    if [[ $EUID -ne 0 ]]; then
        log_warn "Not running as root. ISO build will require sudo."
    fi

    # Check Rust toolchain
    if ! rustc --version &>/dev/null; then
        log_error "Rust toolchain not found. Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    log_info "All prerequisites satisfied"
}

# -----------------------------------------------------------------------------
# Prepare Output Directory
# -----------------------------------------------------------------------------
prepare_output() {
    log_section "Preparing Output Directory"

    mkdir -p "${OUTPUT_DIR}"
    mkdir -p "${OUTPUT_DIR}/iso"
    mkdir -p "${OUTPUT_DIR}/packages"
    mkdir -p "${OUTPUT_DIR}/checksums"
    mkdir -p "${OUTPUT_DIR}/signatures"

    log_info "Output directories created"
}

# -----------------------------------------------------------------------------
# Update Version Information
# -----------------------------------------------------------------------------
update_version_info() {
    log_section "Updating Version Information"

    local release_info="${PROJECT_ROOT}/system/usr/share/winux/release-info"

    if [[ -f "$release_info" ]]; then
        sed -i "s/WINUX_VERSION=.*/WINUX_VERSION=\"${VERSION}\"/" "$release_info"
        sed -i "s/WINUX_CODENAME=.*/WINUX_CODENAME=\"${CODENAME}\"/" "$release_info"
        sed -i "s/WINUX_RELEASE_DATE=.*/WINUX_RELEASE_DATE=\"$(date +%Y-%m-%d)\"/" "$release_info"
        sed -i "s/WINUX_BUILD_ID=.*/WINUX_BUILD_ID=\"${BUILD_ID}\"/" "$release_info"
        log_info "Updated release-info file"
    fi

    # Update Cargo.toml version in workspace
    if [[ -f "${PROJECT_ROOT}/Cargo.toml" ]]; then
        sed -i "s/^version = .*/version = \"${VERSION}\"/" "${PROJECT_ROOT}/Cargo.toml"
        log_info "Updated Cargo.toml version"
    fi

    log_info "Version information updated"
}

# -----------------------------------------------------------------------------
# Build Rust Applications
# -----------------------------------------------------------------------------
build_applications() {
    log_section "Building Rust Applications"

    cd "${PROJECT_ROOT}"

    log_step "Building workspace in release mode..."

    # Set optimization flags
    export RUSTFLAGS="-C target-cpu=x86-64-v2 -C opt-level=3 -C lto=thin"

    # Build all workspace members
    if cargo build --workspace --release 2>&1; then
        log_info "All applications built successfully"
    else
        log_error "Build failed"
        exit 1
    fi

    # Copy binaries to output
    log_step "Copying binaries to output directory..."

    local binaries=(
        "winux-shell"
        "winux-compositor"
        "winux-panel"
        "winux-files"
        "winux-terminal"
        "winux-settings"
        "winux-store"
        "winux-monitor"
        "winux-edit"
    )

    mkdir -p "${OUTPUT_DIR}/packages/bin"

    for binary in "${binaries[@]}"; do
        if [[ -f "${PROJECT_ROOT}/target/release/${binary}" ]]; then
            cp "${PROJECT_ROOT}/target/release/${binary}" "${OUTPUT_DIR}/packages/bin/"
            log_info "Copied ${binary}"
        else
            log_warn "Binary not found: ${binary}"
        fi
    done

    log_info "Applications build complete"
}

# -----------------------------------------------------------------------------
# Generate .deb Packages
# -----------------------------------------------------------------------------
generate_packages() {
    log_section "Generating .deb Packages"

    cd "${PROJECT_ROOT}"

    # Check if cargo-deb is installed
    if ! command -v cargo-deb &>/dev/null; then
        log_warn "cargo-deb not found. Installing..."
        cargo install cargo-deb
    fi

    local packages=(
        "winux-shell"
        "winux-files"
        "winux-terminal"
        "winux-settings"
        "winux-store"
        "winux-monitor"
        "winux-edit"
    )

    mkdir -p "${OUTPUT_DIR}/packages/deb"

    for package in "${packages[@]}"; do
        if [[ -d "${PROJECT_ROOT}/apps/${package}" ]] || [[ -d "${PROJECT_ROOT}/desktop/${package}" ]]; then
            log_step "Generating package for ${package}..."
            if cargo deb -p "${package}" 2>&1; then
                cp "${PROJECT_ROOT}/target/debian/${package}"*.deb "${OUTPUT_DIR}/packages/deb/" 2>/dev/null || true
                log_info "Package created: ${package}"
            else
                log_warn "Failed to create package: ${package}"
            fi
        fi
    done

    log_info "Package generation complete"
}

# -----------------------------------------------------------------------------
# Build ISO
# -----------------------------------------------------------------------------
build_iso() {
    log_section "Building ISO Image"

    local iso_script="${PROJECT_ROOT}/build/scripts/build-winux-iso.sh"
    local iso_name="winux-${VERSION}-${CODENAME}-amd64.iso"

    if [[ ! -f "$iso_script" ]]; then
        log_error "ISO build script not found: ${iso_script}"
        exit 1
    fi

    # Run ISO build script
    log_step "Running ISO build script..."

    export BUILD_DIR
    export OUTPUT_DIR="${OUTPUT_DIR}/iso"
    export WINUX_VERSION="${VERSION}"
    export WINUX_CODENAME="${CODENAME}"

    if [[ $EUID -eq 0 ]]; then
        bash "$iso_script" all
    else
        sudo -E bash "$iso_script" all
    fi

    if [[ -f "${OUTPUT_DIR}/iso/${iso_name}" ]]; then
        log_info "ISO created successfully: ${iso_name}"
    else
        log_error "ISO creation failed"
        exit 1
    fi
}

# -----------------------------------------------------------------------------
# Generate Checksums
# -----------------------------------------------------------------------------
generate_checksums() {
    log_section "Generating Checksums"

    cd "${OUTPUT_DIR}"

    local iso_name="winux-${VERSION}-${CODENAME}-amd64.iso"
    local iso_path="iso/${iso_name}"

    if [[ ! -f "$iso_path" ]]; then
        log_warn "ISO not found, skipping ISO checksums"
    else
        log_step "Generating SHA256 checksum..."
        sha256sum "$iso_path" > "checksums/${iso_name}.sha256"
        log_info "SHA256: $(cat checksums/${iso_name}.sha256)"

        log_step "Generating SHA512 checksum..."
        sha512sum "$iso_path" > "checksums/${iso_name}.sha512"

        log_step "Generating MD5 checksum..."
        md5sum "$iso_path" > "checksums/${iso_name}.md5"

        log_step "Generating B2 checksum..."
        if command -v b2sum &>/dev/null; then
            b2sum "$iso_path" > "checksums/${iso_name}.b2"
        fi
    fi

    # Generate checksums for packages
    log_step "Generating checksums for packages..."

    if [[ -d "packages/deb" ]]; then
        cd packages/deb
        sha256sum *.deb > SHA256SUMS 2>/dev/null || true
        cd "${OUTPUT_DIR}"
    fi

    log_info "Checksums generated"
}

# -----------------------------------------------------------------------------
# Sign Release
# -----------------------------------------------------------------------------
sign_release() {
    log_section "Signing Release"

    if [[ -z "${SIGNING_KEY}" ]]; then
        log_warn "No signing key specified. Set SIGNING_KEY environment variable."
        log_warn "Example: export SIGNING_KEY=your-gpg-key-id"
        return
    fi

    cd "${OUTPUT_DIR}"

    local iso_name="winux-${VERSION}-${CODENAME}-amd64.iso"

    # Sign ISO checksum
    if [[ -f "checksums/${iso_name}.sha256" ]]; then
        log_step "Signing ISO checksum..."
        gpg --default-key "${SIGNING_KEY}" --armor --detach-sign "checksums/${iso_name}.sha256"
        mv "checksums/${iso_name}.sha256.asc" "signatures/"
        log_info "ISO checksum signed"
    fi

    # Sign ISO directly
    if [[ -f "iso/${iso_name}" ]]; then
        log_step "Signing ISO..."
        gpg --default-key "${SIGNING_KEY}" --armor --detach-sign "iso/${iso_name}"
        mv "iso/${iso_name}.asc" "signatures/"
        log_info "ISO signed"
    fi

    log_info "Release signing complete"
}

# -----------------------------------------------------------------------------
# Generate Release Notes
# -----------------------------------------------------------------------------
generate_release_notes() {
    log_section "Generating Release Notes"

    local notes_file="${OUTPUT_DIR}/RELEASE_NOTES.md"
    local iso_name="winux-${VERSION}-${CODENAME}-amd64.iso"

    cat > "$notes_file" << EOF
# Winux OS ${VERSION} "${CODENAME^}"

**Release Date:** $(date +"%B %d, %Y")
**Build ID:** ${BUILD_ID}

## Downloads

### ISO Image
- **File:** \`${iso_name}\`
- **Size:** $(du -h "${OUTPUT_DIR}/iso/${iso_name}" 2>/dev/null | cut -f1 || echo "N/A")

### Checksums
\`\`\`
$(cat "${OUTPUT_DIR}/checksums/${iso_name}.sha256" 2>/dev/null || echo "SHA256: Not available")
\`\`\`

## System Requirements

### Minimum
- CPU: x86_64 with SSE4.2 support
- RAM: 4 GB
- Storage: 30 GB
- GPU: Vulkan 1.1 compatible

### Recommended
- CPU: AMD Ryzen 5 / Intel Core i5 (6+ cores)
- RAM: 16 GB DDR4/DDR5
- Storage: 100 GB NVMe SSD
- GPU: NVIDIA RTX 3060 / AMD RX 6700 XT or better

## Verification

### Verify SHA256 Checksum
\`\`\`bash
sha256sum -c ${iso_name}.sha256
\`\`\`

### Verify GPG Signature
\`\`\`bash
gpg --verify ${iso_name}.sha256.asc ${iso_name}.sha256
\`\`\`

## Installation

1. Download the ISO image
2. Verify the checksum
3. Write to USB drive:
   \`\`\`bash
   sudo dd if=${iso_name} of=/dev/sdX bs=4M status=progress oflag=sync
   \`\`\`
4. Boot from the USB drive
5. Follow the installation wizard

## What's New

See [CHANGELOG.md](https://github.com/winux-os/winux/blob/main/CHANGELOG.md) for details.

## Support

- **Forum:** https://forum.winux-os.org
- **Discord:** https://discord.gg/winux
- **GitHub Issues:** https://github.com/winux-os/winux/issues

---

**Winux OS Project - $(date +%Y)**
*The Best of Both Worlds*
EOF

    log_info "Release notes generated: ${notes_file}"
}

# -----------------------------------------------------------------------------
# Create Release Archive
# -----------------------------------------------------------------------------
create_release_archive() {
    log_section "Creating Release Archive"

    cd "${OUTPUT_DIR}/.."

    local archive_name="winux-${VERSION}-${CODENAME}-release"

    log_step "Creating tarball..."
    tar -czvf "${archive_name}.tar.gz" -C "${OUTPUT_DIR}/.." "$(basename ${OUTPUT_DIR})"

    log_step "Creating zip archive..."
    if command -v zip &>/dev/null; then
        zip -r "${archive_name}.zip" "$(basename ${OUTPUT_DIR})"
    fi

    log_info "Release archive created"
}

# -----------------------------------------------------------------------------
# Cleanup
# -----------------------------------------------------------------------------
cleanup() {
    log_section "Cleanup"

    if [[ "${CLEANUP:-false}" == "true" ]]; then
        log_step "Removing build directory..."
        rm -rf "${BUILD_DIR}"
        log_info "Build directory removed"
    else
        log_info "Build directory preserved: ${BUILD_DIR}"
    fi
}

# -----------------------------------------------------------------------------
# Summary
# -----------------------------------------------------------------------------
show_summary() {
    log_section "Release Summary"

    echo ""
    echo "============================================================"
    echo "  WINUX OS ${VERSION} ${CODENAME^^} - RELEASE COMPLETE"
    echo "============================================================"
    echo ""
    echo "  Output Directory: ${OUTPUT_DIR}"
    echo ""
    echo "  Contents:"
    ls -la "${OUTPUT_DIR}/" 2>/dev/null || true
    echo ""

    if [[ -d "${OUTPUT_DIR}/iso" ]]; then
        echo "  ISO Images:"
        ls -lh "${OUTPUT_DIR}/iso/"*.iso 2>/dev/null || echo "    None"
        echo ""
    fi

    if [[ -d "${OUTPUT_DIR}/packages/deb" ]]; then
        echo "  Packages:"
        ls -1 "${OUTPUT_DIR}/packages/deb/"*.deb 2>/dev/null | wc -l || echo "0"
        echo " .deb packages"
        echo ""
    fi

    if [[ -d "${OUTPUT_DIR}/checksums" ]]; then
        echo "  Checksums:"
        ls -1 "${OUTPUT_DIR}/checksums/" 2>/dev/null || echo "    None"
        echo ""
    fi

    echo "============================================================"
    echo ""
    log_info "Release creation completed successfully!"
    echo ""
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    local start_time=$(date +%s)

    show_banner
    check_prerequisites
    prepare_output
    update_version_info
    build_applications
    generate_packages

    # ISO build is optional (can be skipped with --no-iso flag)
    if [[ "${NO_ISO:-false}" != "true" ]]; then
        build_iso
    else
        log_warn "ISO build skipped (--no-iso flag)"
    fi

    generate_checksums
    sign_release
    generate_release_notes
    create_release_archive
    cleanup
    show_summary

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    log_info "Total time: $((duration / 60)) minutes $((duration % 60)) seconds"
}

# -----------------------------------------------------------------------------
# Help
# -----------------------------------------------------------------------------
show_help() {
    cat << EOF
Winux OS Release Creation Script

Usage: $(basename "$0") [VERSION] [CODENAME] [OPTIONS]

Arguments:
    VERSION     Version number (e.g., 1.0.0)
    CODENAME    Release codename (e.g., aurora)

Options:
    --no-iso        Skip ISO build
    --no-packages   Skip package generation
    --no-sign       Skip release signing
    --cleanup       Remove build directory after completion
    -h, --help      Show this help message

Environment Variables:
    OUTPUT_DIR      Output directory (default: ./release)
    BUILD_DIR       Build directory (default: /tmp/winux-build)
    SIGNING_KEY     GPG key ID for signing

Examples:
    $(basename "$0") 1.0.0 aurora
    $(basename "$0") 1.1.0 borealis --no-iso
    SIGNING_KEY=ABCD1234 $(basename "$0") 1.0.0 aurora

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-iso)
            NO_ISO=true
            shift
            ;;
        --no-packages)
            NO_PACKAGES=true
            shift
            ;;
        --no-sign)
            SIGNING_KEY=""
            shift
            ;;
        --cleanup)
            CLEANUP=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            if [[ -z "${VERSION_SET:-}" ]]; then
                VERSION="$1"
                VERSION_SET=true
            elif [[ -z "${CODENAME_SET:-}" ]]; then
                CODENAME="$1"
                CODENAME_SET=true
            fi
            shift
            ;;
    esac
done

# Run main
main
