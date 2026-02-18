#!/bin/bash
# =============================================================================
# Winux OS - MangoHud Configuration Generator
# Sprint 13-14: Windows Compatibility Layer
# =============================================================================
#
# This script generates MangoHud configuration files based on presets
# and user preferences for gaming overlay display.
#
# Usage: mangohud-config.sh [preset] [options]
#        mangohud-config.sh gaming
#        mangohud-config.sh minimal --fps-limit 60
#        mangohud-config.sh streaming --position top-right
# =============================================================================

set -e

# -----------------------------------------------------------------------------
# Configuration Paths
# -----------------------------------------------------------------------------
MANGOHUD_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/MangoHud"
MANGOHUD_CONFIG_FILE="${MANGOHUD_CONFIG_DIR}/MangoHud.conf"
WINUX_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/winux"

# Ensure directories exist
mkdir -p "$MANGOHUD_CONFIG_DIR"
mkdir -p "$WINUX_CONFIG_DIR"

# -----------------------------------------------------------------------------
# Default Values
# -----------------------------------------------------------------------------
PRESET="gaming"
POSITION="top-left"
FPS_LIMIT=0
CUSTOM_TEXT=""
OUTPUT_FILE="$MANGOHUD_CONFIG_FILE"
GAME_PROFILE=""

# -----------------------------------------------------------------------------
# Help Function
# -----------------------------------------------------------------------------
show_help() {
    cat << 'EOF'
Winux OS MangoHud Configuration Generator

Usage: mangohud-config.sh [PRESET] [OPTIONS]

Presets:
  gaming      Full gaming overlay with FPS, CPU, GPU, and frame timing
  minimal     Minimal overlay with just FPS counter
  streaming   Streaming-optimized overlay for OBS/content creation
  benchmark   Full metrics for benchmarking and performance analysis
  battery     Battery-focused overlay for laptop gaming
  vr          VR-optimized minimal overlay
  custom      Start with empty config for manual customization

Options:
  -p, --position POS    Set overlay position (top-left, top-right,
                        bottom-left, bottom-right, top-center, bottom-center)
  -f, --fps-limit N     Set FPS limit (0 = unlimited)
  -t, --text TEXT       Set custom text to display
  -o, --output FILE     Output to specific file instead of default
  -g, --game GAME       Generate game-specific config
  --no-gpu              Hide GPU metrics
  --no-cpu              Hide CPU metrics
  --no-frametime        Hide frame time graph
  --compact             Use compact/condensed layout
  --horizontal          Use horizontal layout
  -h, --help            Show this help message

Examples:
  mangohud-config.sh gaming
  mangohud-config.sh minimal --fps-limit 60
  mangohud-config.sh streaming --position top-right
  mangohud-config.sh benchmark --output ~/benchmark.conf
  mangohud-config.sh gaming --game "Cyberpunk 2077"

Environment Variables:
  MANGOHUD_CONFIG       Override default config file location
  WINUX_MANGOHUD_PRESET Default preset to use

EOF
}

# -----------------------------------------------------------------------------
# Parse Arguments
# -----------------------------------------------------------------------------
SHOW_GPU=1
SHOW_CPU=1
SHOW_FRAMETIME=1
COMPACT=0
HORIZONTAL=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        gaming|minimal|streaming|benchmark|battery|vr|custom)
            PRESET="$1"
            shift
            ;;
        -p|--position)
            POSITION="$2"
            shift 2
            ;;
        -f|--fps-limit)
            FPS_LIMIT="$2"
            shift 2
            ;;
        -t|--text)
            CUSTOM_TEXT="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -g|--game)
            GAME_PROFILE="$2"
            shift 2
            ;;
        --no-gpu)
            SHOW_GPU=0
            shift
            ;;
        --no-cpu)
            SHOW_CPU=0
            shift
            ;;
        --no-frametime)
            SHOW_FRAMETIME=0
            shift
            ;;
        --compact)
            COMPACT=1
            shift
            ;;
        --horizontal)
            HORIZONTAL=1
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# -----------------------------------------------------------------------------
# Position Mapping
# -----------------------------------------------------------------------------
get_position_value() {
    case "$1" in
        top-left)      echo "0" ;;
        top-right)     echo "1" ;;
        bottom-left)   echo "2" ;;
        bottom-right)  echo "3" ;;
        top-center)    echo "4" ;;
        bottom-center) echo "5" ;;
        *)             echo "0" ;;
    esac
}

# -----------------------------------------------------------------------------
# Generate Common Header
# -----------------------------------------------------------------------------
generate_header() {
    cat << EOF
### MangoHud Configuration
### Generated by Winux OS mangohud-config.sh
### Preset: $PRESET
### Generated: $(date '+%Y-%m-%d %H:%M:%S')
###
### Documentation: https://github.com/flightlessmango/MangoHud

EOF
}

# -----------------------------------------------------------------------------
# Generate Gaming Preset
# -----------------------------------------------------------------------------
generate_gaming() {
    cat << EOF
################### GAMING PRESET ###################
# Full gaming overlay with comprehensive metrics

### Display Settings
position=$(get_position_value "$POSITION")
round_corners=8
background_alpha=0.5
font_size=20

### Performance Metrics
fps
fps_color_change
fps_value=30,60
fps_color=FF0000,FFFF00,00FF00

$(if [[ $SHOW_GPU -eq 1 ]]; then echo "gpu_stats
gpu_temp
gpu_core_clock
gpu_mem_clock
gpu_power
gpu_load_change
gpu_load_value=50,90
gpu_load_color=39F900,FDFD09,B22222
vram
vulkan_driver"; fi)

$(if [[ $SHOW_CPU -eq 1 ]]; then echo "cpu_stats
cpu_temp
cpu_mhz
cpu_load_change
cpu_load_value=50,90
cpu_load_color=39F900,FDFD09,B22222
core_load"; fi)

### Memory
ram
procmem

### Frame Timing
$(if [[ $SHOW_FRAMETIME -eq 1 ]]; then echo "frame_timing
frametime_color=00FF00
histogram"; fi)

### Additional Info
engine_version
gamemode
wine

### FPS Limit
$(if [[ $FPS_LIMIT -gt 0 ]]; then echo "fps_limit=$FPS_LIMIT"; else echo "# fps_limit=0"; fi)

### Logging (F2 to start/stop)
output_folder=/tmp/mangohud_logs
log_duration=30
autostart_log=0

### Toggle Key
toggle_hud=Shift_R+F12
toggle_fps_limit=Shift_L+F1

$(if [[ -n "$CUSTOM_TEXT" ]]; then echo "custom_text=$CUSTOM_TEXT"; fi)

EOF
}

# -----------------------------------------------------------------------------
# Generate Minimal Preset
# -----------------------------------------------------------------------------
generate_minimal() {
    cat << EOF
################### MINIMAL PRESET ###################
# Simple FPS counter only

### Display Settings
position=$(get_position_value "$POSITION")
round_corners=4
background_alpha=0.3
font_size=24

### Only show FPS
fps
fps_only
fps_color_change
fps_value=30,60
fps_color=FF0000,FFFF00,00FF00

### FPS Limit
$(if [[ $FPS_LIMIT -gt 0 ]]; then echo "fps_limit=$FPS_LIMIT"; else echo "# fps_limit=0"; fi)

### Toggle Key
toggle_hud=Shift_R+F12

$(if [[ -n "$CUSTOM_TEXT" ]]; then echo "custom_text=$CUSTOM_TEXT"; fi)

EOF
}

# -----------------------------------------------------------------------------
# Generate Streaming Preset
# -----------------------------------------------------------------------------
generate_streaming() {
    cat << EOF
################### STREAMING PRESET ###################
# Optimized for content creation and streaming

### Display Settings
position=$(get_position_value "$POSITION")
round_corners=6
background_alpha=0.4
font_size=18

### Performance Metrics (Condensed)
fps
fps_color_change
fps_value=30,60
fps_color=FF0000,FFFF00,00FF00

$(if [[ $SHOW_GPU -eq 1 ]]; then echo "gpu_stats
gpu_temp"; fi)

$(if [[ $SHOW_CPU -eq 1 ]]; then echo "cpu_stats
cpu_temp"; fi)

### Frame Timing (Important for stream smoothness)
$(if [[ $SHOW_FRAMETIME -eq 1 ]]; then echo "frame_timing"; fi)

### Streaming indicators
gamemode

### FPS Limit (Important for consistent streaming)
$(if [[ $FPS_LIMIT -gt 0 ]]; then echo "fps_limit=$FPS_LIMIT"; else echo "fps_limit=60"; fi)

### Toggle Key
toggle_hud=Shift_R+F12

### Keep it clean for viewers
no_display
# Toggle with keybind when needed

$(if [[ -n "$CUSTOM_TEXT" ]]; then echo "custom_text=$CUSTOM_TEXT"; fi)

EOF
}

# -----------------------------------------------------------------------------
# Generate Benchmark Preset
# -----------------------------------------------------------------------------
generate_benchmark() {
    cat << EOF
################### BENCHMARK PRESET ###################
# Full metrics for performance analysis

### Display Settings
position=$(get_position_value "$POSITION")
round_corners=8
background_alpha=0.6
font_size=18
table_columns=4

### All Performance Metrics
fps
frametime

### GPU Metrics
gpu_stats
gpu_temp
gpu_junction_temp
gpu_core_clock
gpu_mem_clock
gpu_power
gpu_voltage
gpu_fan
vram
vulkan_driver
gpu_name

### CPU Metrics
cpu_stats
cpu_temp
cpu_mhz
cpu_power
core_load
core_bars

### Memory
ram
swap
procmem
procmem_shared
procmem_virt

### Frame Timing Analysis
frame_timing
frametime_color=00FF00
histogram
frame_count

### Advanced
engine_version
arch
wine
gamemode
fsr
hdr

### Benchmark Logging
output_folder=/tmp/mangohud_benchmarks
log_duration=60
autostart_log=0
log_interval=100

### FPS Limit
$(if [[ $FPS_LIMIT -gt 0 ]]; then echo "fps_limit=$FPS_LIMIT"; else echo "# fps_limit=0"; fi)

### Toggle Keys
toggle_hud=Shift_R+F12
toggle_logging=F2

$(if [[ -n "$CUSTOM_TEXT" ]]; then echo "custom_text=$CUSTOM_TEXT"; fi)

EOF
}

# -----------------------------------------------------------------------------
# Generate Battery Preset
# -----------------------------------------------------------------------------
generate_battery() {
    cat << EOF
################### BATTERY PRESET ###################
# Battery-focused for laptop gaming

### Display Settings
position=$(get_position_value "$POSITION")
round_corners=6
background_alpha=0.4
font_size=16

### Essential Metrics
fps
fps_color_change
fps_value=30,60
fps_color=FF0000,FFFF00,00FF00

### Power Monitoring
battery
battery_icon
gpu_power
cpu_power

### Temperatures
gpu_temp
cpu_temp

### Keep it minimal for power savings
$(if [[ $SHOW_FRAMETIME -eq 1 ]]; then echo "frame_timing"; fi)

### FPS Limit (Important for battery life!)
$(if [[ $FPS_LIMIT -gt 0 ]]; then echo "fps_limit=$FPS_LIMIT"; else echo "fps_limit=60"; fi)

### Toggle Key
toggle_hud=Shift_R+F12

$(if [[ -n "$CUSTOM_TEXT" ]]; then echo "custom_text=$CUSTOM_TEXT"; fi)

EOF
}

# -----------------------------------------------------------------------------
# Generate VR Preset
# -----------------------------------------------------------------------------
generate_vr() {
    cat << EOF
################### VR PRESET ###################
# VR-optimized minimal overlay

### Display Settings
position=$(get_position_value "$POSITION")
round_corners=4
background_alpha=0.25
font_size=14
# Keep small to not obstruct VR view

### Essential VR Metrics
fps
fps_color_change
fps_value=72,90
fps_color=FF0000,FFFF00,00FF00

### Frame Timing (Critical for VR)
frame_timing
frametime_color=00FF00

### Temperature (VR headsets can cause high GPU temps)
gpu_temp

### FPS Limit (Usually locked to HMD refresh)
$(if [[ $FPS_LIMIT -gt 0 ]]; then echo "fps_limit=$FPS_LIMIT"; else echo "# fps_limit=90"; fi)

### Toggle Key
toggle_hud=Shift_R+F12

### Keep minimal
$(if [[ -n "$CUSTOM_TEXT" ]]; then echo "custom_text=$CUSTOM_TEXT"; fi)

EOF
}

# -----------------------------------------------------------------------------
# Generate Custom (Empty) Preset
# -----------------------------------------------------------------------------
generate_custom() {
    cat << EOF
################### CUSTOM PRESET ###################
# Empty template for manual configuration

### Display Settings
position=$(get_position_value "$POSITION")
round_corners=8
background_alpha=0.5
font_size=20

### Add your metrics below
# fps
# gpu_stats
# gpu_temp
# cpu_stats
# cpu_temp
# ram
# frame_timing

### FPS Limit
$(if [[ $FPS_LIMIT -gt 0 ]]; then echo "fps_limit=$FPS_LIMIT"; else echo "# fps_limit=0"; fi)

### Toggle Key
toggle_hud=Shift_R+F12

$(if [[ -n "$CUSTOM_TEXT" ]]; then echo "custom_text=$CUSTOM_TEXT"; fi)

EOF
}

# -----------------------------------------------------------------------------
# Apply Layout Modifiers
# -----------------------------------------------------------------------------
apply_modifiers() {
    local config="$1"

    if [[ $COMPACT -eq 1 ]]; then
        config=$(echo "$config" | sed 's/font_size=.*/font_size=14/')
        echo "$config"
        echo "# Compact mode enabled"
        echo "cellpadding_y=-0.085"
    else
        echo "$config"
    fi
}

# -----------------------------------------------------------------------------
# Generate Game-Specific Config
# -----------------------------------------------------------------------------
generate_game_config() {
    local game="$1"
    local game_config_dir="${MANGOHUD_CONFIG_DIR}/game_profiles"
    mkdir -p "$game_config_dir"

    # Sanitize game name for filename
    local safe_name
    safe_name=$(echo "$game" | tr '[:upper:]' '[:lower:]' | tr ' ' '_' | tr -cd '[:alnum:]_-')
    OUTPUT_FILE="${game_config_dir}/${safe_name}.conf"

    echo "### Game-Specific Configuration for: $game" > "$OUTPUT_FILE"
    echo "### Copy relevant settings to MangoHud.conf or use MANGOHUD_CONFIGFILE" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
}

# -----------------------------------------------------------------------------
# Main Function
# -----------------------------------------------------------------------------
main() {
    echo "Generating MangoHud configuration..."
    echo "  Preset: $PRESET"
    echo "  Position: $POSITION"
    echo "  FPS Limit: $FPS_LIMIT"
    echo "  Output: $OUTPUT_FILE"

    # Handle game-specific config
    if [[ -n "$GAME_PROFILE" ]]; then
        generate_game_config "$GAME_PROFILE"
    fi

    # Generate configuration based on preset
    local config
    config=$(generate_header)

    case "$PRESET" in
        gaming)
            config+=$(generate_gaming)
            ;;
        minimal)
            config+=$(generate_minimal)
            ;;
        streaming)
            config+=$(generate_streaming)
            ;;
        benchmark)
            config+=$(generate_benchmark)
            ;;
        battery)
            config+=$(generate_battery)
            ;;
        vr)
            config+=$(generate_vr)
            ;;
        custom)
            config+=$(generate_custom)
            ;;
        *)
            echo "Unknown preset: $PRESET"
            exit 1
            ;;
    esac

    # Apply modifiers
    config=$(apply_modifiers "$config")

    # Write configuration
    echo "$config" > "$OUTPUT_FILE"

    echo ""
    echo "MangoHud configuration generated successfully!"
    echo "Config file: $OUTPUT_FILE"
    echo ""
    echo "To use with a game:"
    echo "  mangohud <game_command>"
    echo "  MANGOHUD=1 <game_command>"
    echo ""
    echo "To toggle HUD in-game: Shift+F12"
}

# Run main function
main
