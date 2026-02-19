#!/bin/bash
#===============================================================================
# Winux OS - Swift Toolchain Setup
# Installs official Swift toolchain from swift.org for Linux
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
    echo -e "${WHITE}${BOLD}         Swift Toolchain Setup${NC}"
    echo -e "${MAGENTA}    Official Swift for Linux from swift.org${NC}"
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo ""
}

#-------------------------------------------------------------------------------
# Configuration
#-------------------------------------------------------------------------------
SWIFT_VERSION="${SWIFT_VERSION:-5.10}"
SWIFT_RELEASE="swift-${SWIFT_VERSION}-RELEASE"
SWIFT_DIR="/opt/swift"
SWIFT_BIN="$SWIFT_DIR/usr/bin"

# Detect Ubuntu version
detect_ubuntu_version() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        UBUNTU_VERSION="${VERSION_ID}"
        UBUNTU_CODENAME="${VERSION_CODENAME:-$(lsb_release -cs 2>/dev/null || echo 'jammy')}"
    else
        UBUNTU_VERSION="22.04"
        UBUNTU_CODENAME="jammy"
    fi

    log_info "Detected Ubuntu ${UBUNTU_VERSION} (${UBUNTU_CODENAME})"
}

# Get Swift download URL based on Ubuntu version
get_swift_url() {
    local platform=""

    case "$UBUNTU_VERSION" in
        24.04)
            platform="ubuntu2404"
            ;;
        23.10)
            platform="ubuntu2310"
            ;;
        22.04)
            platform="ubuntu2204"
            ;;
        20.04)
            platform="ubuntu2004"
            ;;
        *)
            # Default to Ubuntu 22.04
            platform="ubuntu2204"
            log_warn "Ubuntu ${UBUNTU_VERSION} not officially supported, using Ubuntu 22.04 build"
            ;;
    esac

    SWIFT_PLATFORM="$platform"
    SWIFT_URL="https://download.swift.org/swift-${SWIFT_VERSION}-release/${platform}/${SWIFT_RELEASE}/${SWIFT_RELEASE}-${platform}.tar.gz"
    SWIFT_SIG_URL="${SWIFT_URL}.sig"
}

#-------------------------------------------------------------------------------
# Install Dependencies
#-------------------------------------------------------------------------------
install_dependencies() {
    log_step "Installing Swift dependencies..."

    sudo apt-get update

    # Core dependencies for Swift
    sudo apt-get install -y \
        binutils \
        git \
        gnupg2 \
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
        unzip \
        zlib1g-dev

    # Additional dependencies for SourceKit-LSP and development
    sudo apt-get install -y \
        libncurses5 \
        libcurl4 \
        libpython3.10 \
        libtinfo5 \
        libatomic1 2>/dev/null || true

    # Development tools
    sudo apt-get install -y \
        clang \
        lldb \
        cmake \
        ninja-build

    log_success "Dependencies installed"
}

#-------------------------------------------------------------------------------
# Download and Install Swift
#-------------------------------------------------------------------------------
download_swift() {
    log_step "Downloading Swift ${SWIFT_VERSION}..."

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Download Swift
    log_info "Downloading from: $SWIFT_URL"
    wget -q --show-progress -O swift.tar.gz "$SWIFT_URL"

    # Download and verify signature (optional but recommended)
    log_info "Verifying signature..."
    wget -q -O swift.tar.gz.sig "$SWIFT_SIG_URL" 2>/dev/null || {
        log_warn "Could not download signature, skipping verification"
    }

    # Import Swift PGP keys
    if [ -f swift.tar.gz.sig ]; then
        wget -q https://swift.org/keys/all-keys.asc -O swift-keys.asc
        gpg --import swift-keys.asc 2>/dev/null || true
        gpg --verify swift.tar.gz.sig swift.tar.gz 2>/dev/null && \
            log_success "Signature verified" || \
            log_warn "Could not verify signature, continuing anyway"
    fi

    log_success "Swift downloaded"
}

install_swift() {
    log_step "Installing Swift to ${SWIFT_DIR}..."

    # Remove existing installation
    if [ -d "$SWIFT_DIR" ]; then
        log_warn "Removing existing Swift installation..."
        sudo rm -rf "$SWIFT_DIR"
    fi

    # Create directory and extract
    sudo mkdir -p "$SWIFT_DIR"
    sudo tar -xzf swift.tar.gz -C "$SWIFT_DIR" --strip-components=1

    # Clean up
    cd -
    rm -rf "$TEMP_DIR"

    # Create symlinks for easy access
    sudo ln -sf "$SWIFT_BIN/swift" /usr/local/bin/swift 2>/dev/null || true
    sudo ln -sf "$SWIFT_BIN/swiftc" /usr/local/bin/swiftc 2>/dev/null || true
    sudo ln -sf "$SWIFT_BIN/swift-build" /usr/local/bin/swift-build 2>/dev/null || true
    sudo ln -sf "$SWIFT_BIN/swift-run" /usr/local/bin/swift-run 2>/dev/null || true
    sudo ln -sf "$SWIFT_BIN/swift-test" /usr/local/bin/swift-test 2>/dev/null || true
    sudo ln -sf "$SWIFT_BIN/swift-package" /usr/local/bin/swift-package 2>/dev/null || true
    sudo ln -sf "$SWIFT_BIN/sourcekit-lsp" /usr/local/bin/sourcekit-lsp 2>/dev/null || true
    sudo ln -sf "$SWIFT_BIN/lldb" /usr/local/bin/swift-lldb 2>/dev/null || true

    log_success "Swift installed to ${SWIFT_DIR}"
}

#-------------------------------------------------------------------------------
# Configure PATH and Environment
#-------------------------------------------------------------------------------
configure_environment() {
    log_step "Configuring environment..."

    # Create Swift environment file
    cat > "$HOME/.swift-env" << EOF
# Swift Development Environment
# Generated by Winux OS setup-swift.sh

# Swift Toolchain Path
export SWIFT_HOME="${SWIFT_DIR}"
export PATH="${SWIFT_BIN}:\$PATH"

# Swift aliases
alias swift-repl='swift repl'
alias spm='swift package'
alias spm-init='swift package init --type executable'
alias spm-lib='swift package init --type library'
alias spm-build='swift build'
alias spm-test='swift test'
alias spm-run='swift run'
alias spm-clean='swift package clean'
alias spm-update='swift package update'
alias spm-resolve='swift package resolve'

# SourceKit-LSP
alias sourcekit='sourcekit-lsp'

# Swift helper functions
swift-new() {
    local name="\${1:-MyApp}"
    local type="\${2:-executable}"
    mkdir -p "\$name"
    cd "\$name"
    swift package init --type "\$type"
    echo "Created Swift \$type package: \$name"
}

swift-playground() {
    local name="\${1:-playground}"
    mkdir -p "\$name"
    cat > "\$name/main.swift" << 'SWIFT'
// Swift Playground
// Run with: swift main.swift

import Foundation

print("Hello, Swift!")

// Example: Working with arrays
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map { \$0 * 2 }
print("Doubled: \(doubled)")

// Example: Working with optionals
let optionalString: String? = "Hello"
if let unwrapped = optionalString {
    print("Unwrapped: \(unwrapped)")
}

// Example: Working with structs
struct Person {
    let name: String
    var age: Int

    func greet() -> String {
        return "Hello, my name is \(name)"
    }
}

let person = Person(name: "Swift Developer", age: 25)
print(person.greet())
SWIFT
    echo "Created Swift playground at: \$name/"
    echo "Run with: cd \$name && swift main.swift"
}

# Version info
swift-info() {
    echo "Swift Environment Info"
    echo "====================="
    echo "Swift Home: \$SWIFT_HOME"
    swift --version
    echo ""
    echo "Installed components:"
    ls -la "${SWIFT_BIN}/" | grep -E "^-.*x" | awk '{print "  - " \$NF}'
}
EOF

    # Add to bashrc
    if ! grep -q ".swift-env" "$HOME/.bashrc" 2>/dev/null; then
        echo "" >> "$HOME/.bashrc"
        echo "# Swift Development Environment" >> "$HOME/.bashrc"
        echo '[ -f "$HOME/.swift-env" ] && source "$HOME/.swift-env"' >> "$HOME/.bashrc"
    fi

    # Add to zshrc if exists
    if [ -f "$HOME/.zshrc" ]; then
        if ! grep -q ".swift-env" "$HOME/.zshrc" 2>/dev/null; then
            echo "" >> "$HOME/.zshrc"
            echo "# Swift Development Environment" >> "$HOME/.zshrc"
            echo '[ -f "$HOME/.swift-env" ] && source "$HOME/.swift-env"' >> "$HOME/.zshrc"
        fi
    fi

    # Source the environment
    export PATH="${SWIFT_BIN}:$PATH"

    log_success "Environment configured"
}

#-------------------------------------------------------------------------------
# Configure SourceKit-LSP for IDEs
#-------------------------------------------------------------------------------
configure_sourcekit_lsp() {
    log_step "Configuring SourceKit-LSP for IDEs..."

    # Check if SourceKit-LSP exists
    if [ ! -f "$SWIFT_BIN/sourcekit-lsp" ]; then
        log_warn "SourceKit-LSP not found in Swift installation"
        return
    fi

    # VS Code configuration
    VSCODE_DIR="$HOME/.config/Code/User"
    if [ -d "$HOME/.config/Code" ] || [ -d "$HOME/.vscode" ]; then
        mkdir -p "$VSCODE_DIR"

        # Create or update settings.json
        SETTINGS_FILE="$VSCODE_DIR/settings.json"

        if [ -f "$SETTINGS_FILE" ]; then
            # Backup existing settings
            cp "$SETTINGS_FILE" "${SETTINGS_FILE}.backup"
        fi

        # Create SourceKit-LSP config
        cat > "$HOME/.sourcekit-lsp/config.json" 2>/dev/null << EOF || true
{
    "swiftPM": {
        "configuration": "debug"
    },
    "backgroundIndexing": true,
    "logging": {
        "level": "info"
    }
}
EOF
        mkdir -p "$HOME/.sourcekit-lsp"

        log_info "VS Code: Install 'Swift' extension from marketplace"
        log_info "SourceKit-LSP path: $SWIFT_BIN/sourcekit-lsp"
    fi

    # Neovim configuration hint
    cat > "$HOME/.config/nvim/lua/swift-lsp.lua" 2>/dev/null << EOF || true
-- Swift LSP configuration for Neovim
-- Add this to your init.lua or lspconfig setup

local lspconfig = require('lspconfig')

lspconfig.sourcekit.setup{
    cmd = { '${SWIFT_BIN}/sourcekit-lsp' },
    filetypes = { 'swift', 'objective-c', 'objective-cpp' },
    root_dir = lspconfig.util.root_pattern('Package.swift', '.git', 'compile_commands.json'),
}
EOF

    log_success "SourceKit-LSP configured"
}

#-------------------------------------------------------------------------------
# Test Installation
#-------------------------------------------------------------------------------
test_installation() {
    log_step "Testing Swift installation..."

    export PATH="${SWIFT_BIN}:$PATH"

    # Test swift version
    echo ""
    log_info "Swift version:"
    swift --version

    # Test swift compiler
    echo ""
    log_info "Swift compiler test:"

    # Create test file
    TEST_DIR=$(mktemp -d)
    cat > "$TEST_DIR/test.swift" << 'EOF'
import Foundation

print("Swift is working correctly!")

// Test basic features
let greeting = "Hello from Swift on Linux!"
print(greeting)

// Test Foundation
let date = Date()
let formatter = DateFormatter()
formatter.dateStyle = .full
print("Current date: \(formatter.string(from: date))")

// Test optionals
let numbers: [Int] = [1, 2, 3, 4, 5]
let sum = numbers.reduce(0, +)
print("Sum of \(numbers) = \(sum)")

print("\nAll tests passed!")
EOF

    # Compile and run
    log_info "Compiling test program..."
    swiftc "$TEST_DIR/test.swift" -o "$TEST_DIR/test"

    log_info "Running test program..."
    "$TEST_DIR/test"

    # Clean up
    rm -rf "$TEST_DIR"

    echo ""
    log_success "Swift installation test passed!"
}

#-------------------------------------------------------------------------------
# Test Swift Package Manager
#-------------------------------------------------------------------------------
test_spm() {
    log_step "Testing Swift Package Manager..."

    export PATH="${SWIFT_BIN}:$PATH"

    # Create test package
    TEST_DIR=$(mktemp -d)
    cd "$TEST_DIR"

    log_info "Creating test package..."
    swift package init --type executable --name TestPackage

    log_info "Building test package..."
    swift build

    log_info "Running test package..."
    swift run

    # Clean up
    cd -
    rm -rf "$TEST_DIR"

    log_success "Swift Package Manager test passed!"
}

#-------------------------------------------------------------------------------
# Show Summary
#-------------------------------------------------------------------------------
show_summary() {
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}${BOLD}         Swift Installation Complete!${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${WHITE}Installation Details:${NC}"
    echo -e "  Swift Version:    ${CYAN}${SWIFT_VERSION}${NC}"
    echo -e "  Install Path:     ${CYAN}${SWIFT_DIR}${NC}"
    echo -e "  Platform:         ${CYAN}${SWIFT_PLATFORM}${NC}"
    echo ""
    echo -e "${WHITE}Installed Tools:${NC}"
    echo -e "  ${GREEN}[OK]${NC} swift          - Swift REPL and compiler driver"
    echo -e "  ${GREEN}[OK]${NC} swiftc         - Swift compiler"
    echo -e "  ${GREEN}[OK]${NC} swift-build    - Swift Package Manager build"
    echo -e "  ${GREEN}[OK]${NC} swift-run      - Swift Package Manager run"
    echo -e "  ${GREEN}[OK]${NC} swift-test     - Swift Package Manager test"
    echo -e "  ${GREEN}[OK]${NC} swift-package  - Swift Package Manager"
    [ -f "$SWIFT_BIN/sourcekit-lsp" ] && \
        echo -e "  ${GREEN}[OK]${NC} sourcekit-lsp  - Language Server Protocol"
    [ -f "$SWIFT_BIN/lldb" ] && \
        echo -e "  ${GREEN}[OK]${NC} lldb           - LLVM Debugger"
    echo ""
    echo -e "${WHITE}Quick Start:${NC}"
    echo -e "  ${CYAN}swift --version${NC}           Check Swift version"
    echo -e "  ${CYAN}swift repl${NC}                Start Swift REPL"
    echo -e "  ${CYAN}swift package init${NC}        Create new package"
    echo -e "  ${CYAN}swift build${NC}               Build current package"
    echo -e "  ${CYAN}swift run${NC}                 Build and run"
    echo -e "  ${CYAN}swift test${NC}                Run tests"
    echo ""
    echo -e "${WHITE}Custom Aliases (after sourcing):${NC}"
    echo -e "  ${CYAN}swift-new myapp${NC}           Create new Swift app"
    echo -e "  ${CYAN}swift-playground${NC}          Create playground"
    echo -e "  ${CYAN}swift-info${NC}                Show Swift info"
    echo -e "  ${CYAN}spm-build${NC}                 Build package"
    echo -e "  ${CYAN}spm-test${NC}                  Run tests"
    echo ""
    echo -e "${WHITE}IDE Integration:${NC}"
    echo -e "  VS Code: Install 'Swift' extension"
    echo -e "  Neovim:  Use sourcekit-lsp with nvim-lspconfig"
    echo -e "  Emacs:   Use lsp-mode with sourcekit-lsp"
    echo ""
    echo -e "${YELLOW}Apply environment changes:${NC}"
    echo -e "  ${CYAN}source ~/.swift-env${NC}"
    echo ""
}

#-------------------------------------------------------------------------------
# Uninstall Swift
#-------------------------------------------------------------------------------
uninstall_swift() {
    log_step "Uninstalling Swift..."

    # Remove installation directory
    if [ -d "$SWIFT_DIR" ]; then
        sudo rm -rf "$SWIFT_DIR"
        log_success "Removed $SWIFT_DIR"
    fi

    # Remove symlinks
    sudo rm -f /usr/local/bin/swift 2>/dev/null || true
    sudo rm -f /usr/local/bin/swiftc 2>/dev/null || true
    sudo rm -f /usr/local/bin/swift-build 2>/dev/null || true
    sudo rm -f /usr/local/bin/swift-run 2>/dev/null || true
    sudo rm -f /usr/local/bin/swift-test 2>/dev/null || true
    sudo rm -f /usr/local/bin/swift-package 2>/dev/null || true
    sudo rm -f /usr/local/bin/sourcekit-lsp 2>/dev/null || true
    sudo rm -f /usr/local/bin/swift-lldb 2>/dev/null || true

    # Remove environment file
    rm -f "$HOME/.swift-env" 2>/dev/null || true

    log_success "Swift uninstalled"
    echo "Note: You may want to remove the Swift entries from ~/.bashrc and ~/.zshrc"
}

#-------------------------------------------------------------------------------
# Main
#-------------------------------------------------------------------------------
main() {
    show_logo

    case "${1:-}" in
        --uninstall)
            uninstall_swift
            exit 0
            ;;
        --test)
            test_installation
            test_spm
            exit 0
            ;;
        --version)
            echo "Setup script for Swift ${SWIFT_VERSION}"
            exit 0
            ;;
        --help|-h)
            echo "Usage: $0 [OPTION]"
            echo ""
            echo "Options:"
            echo "  --uninstall    Remove Swift installation"
            echo "  --test         Test existing installation"
            echo "  --version      Show script version"
            echo "  --help         Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  SWIFT_VERSION  Swift version to install (default: 5.10)"
            echo ""
            echo "Examples:"
            echo "  $0                         Install Swift with defaults"
            echo "  SWIFT_VERSION=5.9 $0       Install Swift 5.9"
            echo "  $0 --uninstall             Remove Swift"
            exit 0
            ;;
    esac

    # Detect system
    detect_ubuntu_version
    get_swift_url

    echo -e "${WHITE}This script will install:${NC}"
    echo -e "  - Swift ${SWIFT_VERSION} for Ubuntu ${UBUNTU_VERSION}"
    echo -e "  - Swift compiler (swiftc)"
    echo -e "  - Swift Package Manager"
    echo -e "  - SourceKit-LSP"
    echo -e "  - LLDB Debugger"
    echo ""
    read -p "Continue? [Y/n]: " confirm
    [[ "$confirm" =~ ^[Nn]$ ]] && exit 0

    echo ""

    # Install
    install_dependencies
    download_swift
    install_swift
    configure_environment
    configure_sourcekit_lsp

    echo ""
    read -p "Run installation tests? [Y/n]: " run_tests
    if [[ ! "$run_tests" =~ ^[Nn]$ ]]; then
        test_installation
        test_spm
    fi

    show_summary
}

main "$@"
