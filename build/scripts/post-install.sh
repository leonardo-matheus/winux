#!/bin/bash
# ============================================================================
# Winux OS - Post-Installation Script
# Sprint 15-16: Build System and Installer
# ============================================================================
# This script runs after Calamares completes the base installation.
# It performs final system configuration and cleanup tasks.
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run as root"
    exit 1
fi

# Get target root from Calamares or use /
TARGET_ROOT="${1:-/}"

log_info "Starting Winux OS post-installation..."
log_info "Target root: ${TARGET_ROOT}"

# ============================================================================
# System Configuration
# ============================================================================

configure_system() {
    log_info "Configuring system settings..."

    # Set timezone if not already set
    if [[ ! -L "${TARGET_ROOT}/etc/localtime" ]]; then
        ln -sf /usr/share/zoneinfo/UTC "${TARGET_ROOT}/etc/localtime"
        log_info "Set default timezone to UTC"
    fi

    # Configure locale
    if [[ -f "${TARGET_ROOT}/etc/locale.gen" ]]; then
        sed -i 's/^#en_US.UTF-8/en_US.UTF-8/' "${TARGET_ROOT}/etc/locale.gen"
        chroot "${TARGET_ROOT}" locale-gen 2>/dev/null || true
        echo "LANG=en_US.UTF-8" > "${TARGET_ROOT}/etc/locale.conf"
        log_success "Locale configured"
    fi

    # Configure vconsole
    cat > "${TARGET_ROOT}/etc/vconsole.conf" << EOF
KEYMAP=us
FONT=ter-v16n
EOF
    log_success "Virtual console configured"
}

# ============================================================================
# Enable Services
# ============================================================================

enable_services() {
    log_info "Enabling system services..."

    local services=(
        "NetworkManager"
        "bluetooth"
        "cups"
        "sddm"
        "fstrim.timer"
        "systemd-timesyncd"
        "firewalld"
        "thermald"
        "power-profiles-daemon"
    )

    for service in "${services[@]}"; do
        if [[ -f "${TARGET_ROOT}/usr/lib/systemd/system/${service}.service" ]] || \
           [[ -f "${TARGET_ROOT}/usr/lib/systemd/system/${service}.timer" ]]; then
            chroot "${TARGET_ROOT}" systemctl enable "${service}" 2>/dev/null || true
            log_success "Enabled: ${service}"
        else
            log_warning "Service not found: ${service}"
        fi
    done

    # Enable user services
    log_info "Setting up user service defaults..."
    mkdir -p "${TARGET_ROOT}/etc/systemd/user/default.target.wants"
}

# ============================================================================
# Configure Plymouth
# ============================================================================

configure_plymouth() {
    log_info "Configuring Plymouth boot splash..."

    if [[ -d "${TARGET_ROOT}/usr/share/plymouth/themes/winux" ]]; then
        chroot "${TARGET_ROOT}" plymouth-set-default-theme winux 2>/dev/null || true
        log_success "Plymouth theme set to winux"
    else
        log_warning "Winux Plymouth theme not found"
    fi
}

# ============================================================================
# Configure GRUB
# ============================================================================

configure_grub() {
    log_info "Configuring GRUB bootloader..."

    # Update GRUB configuration
    cat > "${TARGET_ROOT}/etc/default/grub" << 'EOF'
# Winux OS GRUB Configuration
GRUB_DEFAULT=0
GRUB_TIMEOUT=5
GRUB_DISTRIBUTOR="Winux"
GRUB_CMDLINE_LINUX_DEFAULT="quiet splash loglevel=3 rd.systemd.show_status=auto rd.udev.log_level=3"
GRUB_CMDLINE_LINUX=""

# Theme
GRUB_THEME="/boot/grub/themes/winux/theme.txt"
GRUB_BACKGROUND="/boot/grub/themes/winux/background.png"

# Resolution
GRUB_GFXMODE=auto
GRUB_GFXPAYLOAD_LINUX=keep

# OS Prober
GRUB_DISABLE_OS_PROBER=false

# Recovery
GRUB_DISABLE_RECOVERY=false
EOF

    # Regenerate GRUB configuration
    if [[ -f "${TARGET_ROOT}/boot/grub/grub.cfg" ]]; then
        chroot "${TARGET_ROOT}" grub-mkconfig -o /boot/grub/grub.cfg 2>/dev/null || true
        log_success "GRUB configuration updated"
    fi
}

# ============================================================================
# Configure SDDM
# ============================================================================

configure_sddm() {
    log_info "Configuring SDDM display manager..."

    mkdir -p "${TARGET_ROOT}/etc/sddm.conf.d"

    cat > "${TARGET_ROOT}/etc/sddm.conf.d/winux.conf" << 'EOF'
[Theme]
Current=winux
CursorTheme=breeze_cursors
Font=Noto Sans,10,-1,5,50,0,0,0,0,0

[General]
InputMethod=
Numlock=on

[Users]
MaximumUid=60513
MinimumUid=1000
RememberLastSession=true
RememberLastUser=true
EOF

    log_success "SDDM configured"
}

# ============================================================================
# Setup User Environment
# ============================================================================

setup_user_environment() {
    log_info "Setting up user environment defaults..."

    # Create skel directory structure
    local skel_dirs=(
        ".config"
        ".local/share"
        ".local/bin"
        "Documents"
        "Downloads"
        "Music"
        "Pictures"
        "Videos"
        "Projects"
    )

    for dir in "${skel_dirs[@]}"; do
        mkdir -p "${TARGET_ROOT}/etc/skel/${dir}"
    done

    # Default shell configuration
    cat > "${TARGET_ROOT}/etc/skel/.bashrc" << 'EOF'
# Winux OS Bash Configuration

# If not running interactively, don't do anything
[[ $- != *i* ]] && return

# History settings
HISTSIZE=10000
HISTFILESIZE=20000
HISTCONTROL=ignoreboth:erasedups
shopt -s histappend

# Check window size after each command
shopt -s checkwinsize

# Better globbing
shopt -s globstar 2>/dev/null
shopt -s extglob

# Aliases
alias ls='ls --color=auto'
alias ll='ls -la'
alias la='ls -A'
alias l='ls -CF'
alias grep='grep --color=auto'
alias diff='diff --color=auto'

# Safety aliases
alias rm='rm -i'
alias mv='mv -i'
alias cp='cp -i'

# Navigation
alias ..='cd ..'
alias ...='cd ../..'

# Prompt
PS1='\[\033[01;32m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]\$ '

# Environment
export EDITOR=nano
export VISUAL=nano

# Add local bin to PATH
export PATH="$HOME/.local/bin:$PATH"

# Load additional configurations
for config in ~/.bashrc.d/*.bash; do
    [[ -f "$config" ]] && source "$config"
done
EOF

    log_success "User environment configured"
}

# ============================================================================
# Configure Flatpak
# ============================================================================

configure_flatpak() {
    log_info "Configuring Flatpak..."

    if command -v flatpak &>/dev/null; then
        # Add Flathub repository
        chroot "${TARGET_ROOT}" flatpak remote-add --if-not-exists flathub \
            https://dl.flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true
        log_success "Flathub repository added"
    else
        log_warning "Flatpak not installed"
    fi
}

# ============================================================================
# Setup First-Run Experience
# ============================================================================

setup_first_run() {
    log_info "Setting up first-run experience..."

    # Create autostart entry for first-run wizard
    mkdir -p "${TARGET_ROOT}/etc/skel/.config/autostart"

    cat > "${TARGET_ROOT}/etc/skel/.config/autostart/winux-welcome.desktop" << 'EOF'
[Desktop Entry]
Type=Application
Name=Winux Welcome
Comment=Welcome to Winux OS
Exec=/usr/bin/winux-welcome
Icon=winux-welcome
Terminal=false
Categories=System;
X-GNOME-Autostart-enabled=true
X-KDE-autostart-after=panel
OnlyShowIn=KDE;
EOF

    # Create first-run marker check
    cat > "${TARGET_ROOT}/etc/profile.d/winux-first-run.sh" << 'EOF'
# Winux OS First Run Check
if [[ ! -f "$HOME/.config/winux/setup-complete" ]] && [[ -x /usr/bin/winux-first-run ]]; then
    export WINUX_FIRST_RUN=1
fi
EOF

    log_success "First-run experience configured"
}

# ============================================================================
# Cleanup
# ============================================================================

cleanup() {
    log_info "Performing cleanup..."

    # Remove installation artifacts
    rm -f "${TARGET_ROOT}/etc/machine-id"
    rm -rf "${TARGET_ROOT}/var/lib/pacman/sync/"*.db.lck
    rm -rf "${TARGET_ROOT}/var/cache/pacman/pkg/"*

    # Clear temporary files
    rm -rf "${TARGET_ROOT}/tmp/"*
    rm -rf "${TARGET_ROOT}/var/tmp/"*

    # Clear package cache (keep 1 version)
    if command -v paccache &>/dev/null; then
        chroot "${TARGET_ROOT}" paccache -rk1 2>/dev/null || true
    fi

    # Regenerate machine-id on first boot
    touch "${TARGET_ROOT}/etc/machine-id"

    log_success "Cleanup complete"
}

# ============================================================================
# Finalize
# ============================================================================

finalize() {
    log_info "Finalizing installation..."

    # Update font cache
    chroot "${TARGET_ROOT}" fc-cache -f 2>/dev/null || true

    # Update icon cache
    for theme_dir in "${TARGET_ROOT}"/usr/share/icons/*/; do
        if [[ -f "${theme_dir}index.theme" ]]; then
            chroot "${TARGET_ROOT}" gtk-update-icon-cache -f -t "${theme_dir#${TARGET_ROOT}}" 2>/dev/null || true
        fi
    done

    # Update desktop database
    chroot "${TARGET_ROOT}" update-desktop-database 2>/dev/null || true

    # Update mime database
    chroot "${TARGET_ROOT}" update-mime-database /usr/share/mime 2>/dev/null || true

    # Sync filesystem
    sync

    log_success "Installation finalized"
}

# ============================================================================
# Main Execution
# ============================================================================

main() {
    log_info "============================================"
    log_info "Winux OS Post-Installation Script"
    log_info "============================================"

    configure_system
    enable_services
    configure_plymouth
    configure_grub
    configure_sddm
    setup_user_environment
    configure_flatpak
    setup_first_run
    cleanup
    finalize

    log_info "============================================"
    log_success "Post-installation completed successfully!"
    log_info "============================================"
}

# Run main function
main "$@"

exit 0
