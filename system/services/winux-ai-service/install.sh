#!/bin/bash
# Installation script for winux-ai-service
#
# This script installs the winux-ai-service daemon on a Winux system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This script must be run as root"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

info "Installing winux-ai-service..."

# Create service user if it doesn't exist
if ! id "winux-ai" &>/dev/null; then
    info "Creating winux-ai user..."
    useradd --system --no-create-home --shell /usr/sbin/nologin winux-ai
fi

# Create configuration directory
info "Creating configuration directory..."
mkdir -p /etc/winux

# Install configuration file if it doesn't exist
if [[ ! -f /etc/winux/ai-service.toml ]]; then
    info "Installing default configuration..."
    cp "$SCRIPT_DIR/ai-service.toml.example" /etc/winux/ai-service.toml
    chmod 600 /etc/winux/ai-service.toml
    chown winux-ai:winux-ai /etc/winux/ai-service.toml
    warn "Please edit /etc/winux/ai-service.toml and set your API key"
else
    info "Configuration file already exists, skipping..."
fi

# Build the service (if source is present)
if [[ -f "$SCRIPT_DIR/Cargo.toml" ]]; then
    info "Building winux-ai-service..."
    cd "$SCRIPT_DIR"
    cargo build --release

    info "Installing binary..."
    cp target/release/winux-ai-service /usr/bin/
    chmod 755 /usr/bin/winux-ai-service
fi

# Install D-Bus policy
info "Installing D-Bus policy..."
cp "$SCRIPT_DIR/com.winux.AI.conf" /etc/dbus-1/system.d/
chmod 644 /etc/dbus-1/system.d/com.winux.AI.conf

# Install D-Bus interface definition
info "Installing D-Bus interface definition..."
mkdir -p /usr/share/dbus-1/interfaces
cp "$SCRIPT_DIR/com.winux.AI.xml" /usr/share/dbus-1/interfaces/
chmod 644 /usr/share/dbus-1/interfaces/com.winux.AI.xml

# Install systemd service
info "Installing systemd service..."
cp "$SCRIPT_DIR/winux-ai-service.service" /etc/systemd/system/
chmod 644 /etc/systemd/system/winux-ai-service.service

# Reload systemd
info "Reloading systemd daemon..."
systemctl daemon-reload

# Reload D-Bus
info "Reloading D-Bus configuration..."
systemctl reload dbus || true

info "Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Edit /etc/winux/ai-service.toml and configure your Azure OpenAI API key"
echo "  2. Enable the service: sudo systemctl enable winux-ai-service"
echo "  3. Start the service: sudo systemctl start winux-ai-service"
echo "  4. Check status: sudo systemctl status winux-ai-service"
echo ""
echo "To test the service:"
echo "  dbus-send --system --print-reply --dest=com.winux.AI /com/winux/AI com.winux.AI.HealthCheck"
