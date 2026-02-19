#!/bin/bash
# Winux OS - Publish Remaining Crates to crates.io
# Run this after the rate limit expires (after 13:18 GMT)
# Make sure you're logged in: cargo login <token>

set -e

echo "======================================"
echo "  Winux OS - Publish Remaining Crates"
echo "======================================"

cd "$(dirname "$0")/.."

# Already published:
# - winux-shell-plugins
# - winux-ai
# - winux-cloud
# - winux-connect
# - winux-files

# Remaining crates to publish
CRATES=(
    "winux-terminal"
    "winux-settings"
    "winux-monitor"
    "winux-store"
    "winux-edit"
    "winux-image"
    "winux-player"
    "winux-notes"
    "winux-calendar"
    "winux-contacts"
    "winux-documents"
    "winux-mail"
    "winux-about"
    "winux-personalize"
    "winux-env-manager"
    "winux-welcome"
    "winux-clipboard"
    "winux-bluetooth"
    "winux-network"
    "winux-clock"
    "winux-users"
    "winux-weather"
    "winux-calculator"
    "winux-backup"
    "winux-fonts"
    "winux-updater"
    "winux-screenshot"
    "winux-screencast"
    "winux-power"
    "winux-disks"
    "winux-accessibility"
    "winux-recorder"
    "winux-firewall"
    "winux-printers"
    "winux-camera"
    "winux-logs"
    "winux-builder"
    "winux-dev-hub"
    "winux-mobile-studio"
    "winux-gaming"
    "winux-archive"
    "winux-compositor"
    "winux-panel"
    "winux-shell"
    "winux-launcher"
    "winux-notifications"
    "winux-control-center"
)

echo ""
echo "Publishing ${#CRATES[@]} crates..."
echo ""

for crate in "${CRATES[@]}"; do
    echo "Publishing: $crate"
    if cargo publish -p "$crate" --no-verify --allow-dirty 2>&1; then
        echo "  OK: $crate published"
    else
        echo "  ERROR: $crate failed"
    fi
    # Small delay to avoid rate limiting
    sleep 5
done

echo ""
echo "Done!"
