#!/bin/bash
#===============================================================================
# Winux OS - First Boot Setup
# Runs once on first boot to configure the system
#===============================================================================

FIRST_BOOT_MARKER="/var/lib/winux/.first-boot-complete"

# Check if already ran
if [ -f "$FIRST_BOOT_MARKER" ]; then
    exit 0
fi

# Log file
LOG="/var/log/winux-first-boot.log"
exec 1>> "$LOG" 2>&1

echo "=========================================="
echo "Winux OS First Boot Setup"
echo "Date: $(date)"
echo "=========================================="

# Update font cache
echo "Updating font cache..."
fc-cache -f

# Update icon cache
echo "Updating icon cache..."
gtk-update-icon-cache /usr/share/icons/hicolor 2>/dev/null || true
gtk-update-icon-cache /usr/share/icons/Papirus 2>/dev/null || true

# Configure default Plymouth theme
echo "Configuring Plymouth..."
plymouth-set-default-theme winux 2>/dev/null || true
update-initramfs -u 2>/dev/null || true

# Enable Flatpak repositories
echo "Configuring Flatpak..."
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true

# Set default applications
echo "Setting default applications..."
xdg-mime default winux-files.desktop inode/directory
xdg-mime default winux-image.desktop image/png
xdg-mime default winux-image.desktop image/jpeg
xdg-mime default winux-edit.desktop text/plain

# Configure SDDM for autologin (live session only)
if grep -q "boot=casper" /proc/cmdline 2>/dev/null; then
    echo "Live session detected, enabling autologin..."
    mkdir -p /etc/sddm.conf.d
    cat > /etc/sddm.conf.d/autologin.conf << SDDMEOF
[Autologin]
User=winux
Session=winux.desktop
SDDMEOF
fi

# Create first boot marker
mkdir -p "$(dirname "$FIRST_BOOT_MARKER")"
touch "$FIRST_BOOT_MARKER"

echo "First boot setup complete!"
