#!/bin/bash
# Winux OS - Publish Crates to crates.io
# Run this script on Linux after logging in with: cargo login <token>

set -e

echo "======================================"
echo "  Winux OS - Publish to crates.io"
echo "======================================"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if logged in
if ! cargo owner --list 2>/dev/null; then
    echo -e "${RED}Error: Not logged in to crates.io${NC}"
    echo "Run: cargo login <your-token>"
    exit 1
fi

# Version
VERSION="1.2.0"

# List of publishable library crates (not binary-only apps)
LIBRARY_CRATES=(
    "desktop/winux-shell-plugins"
)

# Function to publish a crate
publish_crate() {
    local crate_path=$1
    local crate_name=$(basename "$crate_path")

    echo -e "${YELLOW}Publishing: $crate_name${NC}"

    cd "$crate_path"

    # Check if already published with this version
    if cargo search "$crate_name" 2>/dev/null | grep -q "^$crate_name = \"$VERSION\""; then
        echo -e "${GREEN}  Already published: $crate_name v$VERSION${NC}"
        cd - > /dev/null
        return 0
    fi

    # Package to verify
    if cargo package --allow-dirty 2>&1; then
        # Publish
        if cargo publish --allow-dirty 2>&1; then
            echo -e "${GREEN}  Published: $crate_name v$VERSION${NC}"
        else
            echo -e "${RED}  Failed to publish: $crate_name${NC}"
        fi
    else
        echo -e "${RED}  Failed to package: $crate_name${NC}"
    fi

    cd - > /dev/null
}

# Main
cd "$(dirname "$0")/.."

echo ""
echo "Note: Most Winux apps are OS-specific binaries and are distributed"
echo "via the ISO image rather than crates.io."
echo ""
echo "Library crates available for publishing:"
for crate in "${LIBRARY_CRATES[@]}"; do
    echo "  - $(basename $crate)"
done
echo ""

# Publish library crates
for crate in "${LIBRARY_CRATES[@]}"; do
    if [ -d "$crate" ]; then
        publish_crate "$crate"
    else
        echo -e "${RED}Crate not found: $crate${NC}"
    fi
done

echo ""
echo -e "${GREEN}Done!${NC}"
echo ""
echo "Note: Binary applications (winux-files, winux-terminal, etc.) are"
echo "distributed as part of the Winux OS ISO, not via crates.io."
