#!/bin/bash
#
# Winux Cursor Theme Build Script
# Generates modern minimalist cursors with xcursorgen
#
# Usage: ./build-cursors.sh [--generate-images] [--clean]
#
# Dependencies:
#   - xcursorgen (from x11-apps package)
#   - python3 with Pillow (only for --generate-images)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC_DIR="$SCRIPT_DIR/src"
CONFIG_DIR="$SRC_DIR/config"
OUTPUT_DIR="$SCRIPT_DIR/cursors"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Winux accent color info
ACCENT_COLOR="#00d4ff"

echo -e "${CYAN}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║           Winux Cursor Theme Builder                       ║"
echo "║           Modern Minimalist Cursors                        ║"
echo "║           Accent Color: ${ACCENT_COLOR}                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Parse arguments
GENERATE_IMAGES=false
CLEAN=false

for arg in "$@"; do
    case $arg in
        --generate-images)
            GENERATE_IMAGES=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --generate-images  Generate PNG images using Python script"
            echo "  --clean           Remove generated files before building"
            echo "  -h, --help        Show this help message"
            echo ""
            echo "Cursor sizes generated: 24, 32, 48, 64 pixels"
            exit 0
            ;;
    esac
done

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}Cleaning previous build...${NC}"
    rm -rf "$OUTPUT_DIR"
    rm -rf "$SRC_DIR/x1" "$SRC_DIR/x1.25" "$SRC_DIR/x1.5" "$SRC_DIR/x2"
    echo -e "${GREEN}Clean complete.${NC}"
fi

# Check dependencies
check_dependency() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed.${NC}"
        echo "Please install it using:"
        echo "  sudo apt install $2"
        exit 1
    fi
}

echo -e "${CYAN}Checking dependencies...${NC}"
check_dependency "xcursorgen" "x11-apps"

# Generate images if requested or if they don't exist
if [ "$GENERATE_IMAGES" = true ] || [ ! -d "$SRC_DIR/x1" ]; then
    echo -e "${CYAN}Generating cursor images with Python...${NC}"

    # Check for Python and Pillow
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}Error: python3 is not installed.${NC}"
        exit 1
    fi

    if ! python3 -c "import PIL" 2>/dev/null; then
        echo -e "${YELLOW}Installing Pillow...${NC}"
        pip3 install --user Pillow
    fi

    # Generate the cursor images
    python3 "$SRC_DIR/generate-cursors.py"
    echo -e "${GREEN}Cursor images generated.${NC}"
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Cursor name mappings (standard X11 cursor names and their aliases)
declare -A CURSOR_ALIASES

# Default arrow
CURSOR_ALIASES["default"]="arrow left_ptr top_left_arrow"

# Pointer/Hand
CURSOR_ALIASES["pointer"]="hand hand1 hand2 pointing_hand openhand"

# Text/I-beam
CURSOR_ALIASES["text"]="xterm ibeam text"

# Wait/Loading
CURSOR_ALIASES["wait"]="watch wait left_ptr_watch"

# Progress
CURSOR_ALIASES["progress"]="progress left_ptr_watch half-busy"

# Help
CURSOR_ALIASES["help"]="question_arrow help whats_this dnd-ask"

# Crosshair
CURSOR_ALIASES["crosshair"]="cross crosshair cross_reverse tcross"

# Move
CURSOR_ALIASES["move"]="fleur move size_all"

# Not allowed
CURSOR_ALIASES["not-allowed"]="crossed_circle not-allowed no-drop forbidden dnd-no-drop circle"

# Grab
CURSOR_ALIASES["grab"]="openhand grab dnd-none"

# Grabbing
CURSOR_ALIASES["grabbing"]="closedhand grabbing dnd-move dnd-copy dnd-link"

# Zoom in/out
CURSOR_ALIASES["zoom-in"]="zoom-in zoom_in"
CURSOR_ALIASES["zoom-out"]="zoom-out zoom_out"

# Resize horizontal/vertical
CURSOR_ALIASES["col-resize"]="col-resize sb_h_double_arrow h_double_arrow split_h ew-resize"
CURSOR_ALIASES["row-resize"]="row-resize sb_v_double_arrow v_double_arrow split_v ns-resize"

# Directional resize
CURSOR_ALIASES["n-resize"]="n-resize top_side"
CURSOR_ALIASES["s-resize"]="s-resize bottom_side"
CURSOR_ALIASES["e-resize"]="e-resize right_side"
CURSOR_ALIASES["w-resize"]="w-resize left_side"
CURSOR_ALIASES["ne-resize"]="ne-resize top_right_corner nesw-resize fd_double_arrow"
CURSOR_ALIASES["nw-resize"]="nw-resize top_left_corner nwse-resize bd_double_arrow size_fdiag"
CURSOR_ALIASES["se-resize"]="se-resize bottom_right_corner nwse-resize bd_double_arrow size_bdiag"
CURSOR_ALIASES["sw-resize"]="sw-resize bottom_left_corner nesw-resize fd_double_arrow"

# All scroll
CURSOR_ALIASES["all-scroll"]="all-scroll fleur size_all"

# Build cursors
echo -e "${CYAN}Building cursors with xcursorgen...${NC}"
echo ""

build_cursor() {
    local cursor_name="$1"
    local config_file="$CONFIG_DIR/${cursor_name}.cursor"

    if [ ! -f "$config_file" ]; then
        echo -e "${RED}  Warning: Config file not found: $config_file${NC}"
        return 1
    fi

    echo -e "  Building ${YELLOW}${cursor_name}${NC}..."

    # Generate the cursor
    xcursorgen -p "$SRC_DIR" "$config_file" "$OUTPUT_DIR/$cursor_name"

    if [ $? -eq 0 ]; then
        echo -e "    ${GREEN}✓${NC} Created: $cursor_name"

        # Create symlinks for aliases
        local aliases="${CURSOR_ALIASES[$cursor_name]}"
        if [ -n "$aliases" ]; then
            for alias in $aliases; do
                if [ "$alias" != "$cursor_name" ] && [ ! -e "$OUTPUT_DIR/$alias" ]; then
                    ln -sf "$cursor_name" "$OUTPUT_DIR/$alias"
                    echo -e "    ${GREEN}→${NC} Linked: $alias -> $cursor_name"
                fi
            done
        fi
    else
        echo -e "    ${RED}✗${NC} Failed: $cursor_name"
        return 1
    fi
}

# Build all cursors
CURSOR_NAMES=(
    "default"
    "pointer"
    "text"
    "wait"
    "progress"
    "help"
    "crosshair"
    "move"
    "not-allowed"
    "grab"
    "grabbing"
    "zoom-in"
    "zoom-out"
    "col-resize"
    "row-resize"
    "n-resize"
    "s-resize"
    "e-resize"
    "w-resize"
    "ne-resize"
    "nw-resize"
    "se-resize"
    "sw-resize"
    "all-scroll"
)

built_count=0
failed_count=0

for cursor in "${CURSOR_NAMES[@]}"; do
    if build_cursor "$cursor"; then
        ((built_count++))
    else
        ((failed_count++))
    fi
done

echo ""
echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Build complete!${NC}"
echo -e "  Cursors built: ${GREEN}${built_count}${NC}"
if [ $failed_count -gt 0 ]; then
    echo -e "  Failed: ${RED}${failed_count}${NC}"
fi
echo ""
echo -e "Cursor theme location: ${YELLOW}$SCRIPT_DIR${NC}"
echo ""
echo -e "To install system-wide:"
echo -e "  ${CYAN}sudo cp -r $SCRIPT_DIR /usr/share/icons/winux-cursors${NC}"
echo ""
echo -e "To install for current user:"
echo -e "  ${CYAN}mkdir -p ~/.local/share/icons${NC}"
echo -e "  ${CYAN}cp -r $SCRIPT_DIR ~/.local/share/icons/winux-cursors${NC}"
echo ""
echo -e "To activate, use your desktop environment's settings or run:"
echo -e "  ${CYAN}gsettings set org.gnome.desktop.interface cursor-theme 'Winux Cursors'${NC}"
echo ""
