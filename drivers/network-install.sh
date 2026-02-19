#!/bin/bash
#===============================================================================
# Winux OS - Network Drivers Installation
#===============================================================================

set -e

echo "Installing network drivers..."

# Update package lists
sudo apt update

# Intel WiFi drivers
sudo apt install -y \
    firmware-iwlwifi \
    linux-firmware \
    || true

# Broadcom WiFi drivers
sudo apt install -y \
    broadcom-sta-dkms \
    bcmwl-kernel-source \
    firmware-b43-installer \
    || true

# Realtek drivers
sudo apt install -y \
    firmware-realtek \
    r8168-dkms \
    || true

# Atheros drivers
sudo apt install -y \
    firmware-atheros \
    || true

# Mediatek/Ralink drivers
sudo apt install -y \
    firmware-misc-nonfree \
    || true

# NetworkManager and tools
sudo apt install -y \
    network-manager \
    network-manager-gnome \
    wpasupplicant \
    wireless-tools \
    iw \
    rfkill \
    net-tools \
    ethtool

# Restart NetworkManager
sudo systemctl restart NetworkManager

echo "Network drivers installed successfully!"
