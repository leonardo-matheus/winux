#!/bin/bash
#===============================================================================
# Winux OS - Bluetooth Drivers Installation
#===============================================================================

set -e

echo "Installing Bluetooth drivers and tools..."

# Update package lists
sudo apt update

# Core Bluetooth stack
sudo apt install -y \
    bluez \
    bluez-tools \
    bluez-firmware \
    bluetooth

# GUI tools
sudo apt install -y \
    blueman \
    gnome-bluetooth

# Audio over Bluetooth
sudo apt install -y \
    pulseaudio-module-bluetooth \
    pipewire-audio \
    libspa-0.2-bluetooth \
    || true

# Intel Bluetooth firmware
sudo apt install -y \
    firmware-iwlwifi \
    || true

# Broadcom Bluetooth
sudo apt install -y \
    firmware-brcm80211 \
    || true

# Enable and start Bluetooth service
sudo systemctl enable bluetooth
sudo systemctl start bluetooth

# Fix Bluetooth permissions
sudo usermod -a -G bluetooth $USER 2>/dev/null || true

echo "Bluetooth drivers installed successfully!"
echo "Please logout and login again to apply group changes."
