#!/bin/bash
#===============================================================================
# Winux OS - Gaming Optimizer
# Optimize system for gaming performance
#===============================================================================

set -e

#-------------------------------------------------------------------------------
# Configuration
#-------------------------------------------------------------------------------
VERSION="1.0.0"
SCRIPT_NAME="$(basename "$0")"
BACKUP_DIR="/var/lib/winux/backups/gaming-$(date +%Y%m%d_%H%M%S)"
LOG_FILE="/var/log/winux/optimize-gaming.log"
DRY_RUN=false
GAME_PID=""

#-------------------------------------------------------------------------------
# Colors
#-------------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
NC='\033[0m'

#-------------------------------------------------------------------------------
# ASCII Art Logo
#-------------------------------------------------------------------------------
show_logo() {
    echo -e "${MAGENTA}"
    cat << 'EOF'
 __        __ _
 \ \      / /(_) _ __  _   _ __ __
  \ \ /\ / / | || '_ \| | | |\ \/ /
   \ V  V /  | || | | | |_| | >  <
    \_/\_/   |_||_| |_|\__,_|/_/\_\

       Gaming Optimizer v1.0.0
      ==========================
          [GAME MODE]
EOF
    echo -e "${NC}"
}

#-------------------------------------------------------------------------------
# Logging Functions
#-------------------------------------------------------------------------------
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $1" >> "$LOG_FILE" 2>/dev/null || true
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [OK] $1" >> "$LOG_FILE" 2>/dev/null || true
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [WARN] $1" >> "$LOG_FILE" 2>/dev/null || true
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $1" >> "$LOG_FILE" 2>/dev/null || true
}

log_step() {
    echo -e "${MAGENTA}[GAME]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [GAME] $1" >> "$LOG_FILE" 2>/dev/null || true
}

#-------------------------------------------------------------------------------
# Progress Bar
#-------------------------------------------------------------------------------
show_progress() {
    local current=$1
    local total=$2
    local width=50
    local percentage=$((current * 100 / total))
    local filled=$((width * current / total))
    local empty=$((width - filled))

    printf "\r${MAGENTA}["
    printf "%${filled}s" | tr ' ' '#'
    printf "%${empty}s" | tr ' ' '-'
    printf "] ${percentage}%%${NC}"

    if [ "$current" -eq "$total" ]; then
        echo ""
    fi
}

#-------------------------------------------------------------------------------
# Help
#-------------------------------------------------------------------------------
show_help() {
    echo "Usage: $SCRIPT_NAME [OPTIONS] [COMMAND]"
    echo ""
    echo "Gaming optimizer for Winux OS"
    echo ""
    echo "Options:"
    echo "  -d, --dry-run     Show what would be done without making changes"
    echo "  -e, --enable      Enable gaming mode (default)"
    echo "  -x, --disable     Disable gaming mode and restore normal settings"
    echo "  -p, --pid PID     Optimize for specific game PID"
    echo "  -s, --status      Show current gaming mode status"
    echo "  -h, --help        Show this help message"
    echo "  -v, --version     Show version"
    echo ""
    echo "Features:"
    echo "  - Disable compositor during gaming"
    echo "  - GameMode integration"
    echo "  - Nice priority optimization"
    echo "  - Temporary ASLR disable"
    echo "  - GPU power management"
    echo "  - Shader cache warmup"
    echo ""
    echo "Examples:"
    echo "  $SCRIPT_NAME                  # Enable gaming mode"
    echo "  $SCRIPT_NAME --disable        # Disable gaming mode"
    echo "  $SCRIPT_NAME --pid 12345      # Optimize for specific game"
}

#-------------------------------------------------------------------------------
# Check Root
#-------------------------------------------------------------------------------
check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "This script must be run as root"
        echo "Please run: sudo $SCRIPT_NAME"
        exit 1
    fi
}

#-------------------------------------------------------------------------------
# Create Backup
#-------------------------------------------------------------------------------
create_backup() {
    log_step "Saving current settings..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would create backup in $BACKUP_DIR"
        return
    fi

    mkdir -p "$BACKUP_DIR"

    # Save ASLR state
    cat /proc/sys/kernel/randomize_va_space > "$BACKUP_DIR/aslr.txt" 2>/dev/null || true

    # Save compositor state
    if pgrep -x "kwin_x11" > /dev/null; then
        echo "kwin" > "$BACKUP_DIR/compositor.txt"
    elif pgrep -x "picom" > /dev/null; then
        echo "picom" > "$BACKUP_DIR/compositor.txt"
    elif pgrep -x "compton" > /dev/null; then
        echo "compton" > "$BACKUP_DIR/compositor.txt"
    fi

    # Save GPU power state
    if [ -f /sys/class/drm/card0/device/power_dpm_force_performance_level ]; then
        cat /sys/class/drm/card0/device/power_dpm_force_performance_level > "$BACKUP_DIR/gpu_power.txt"
    fi

    log_success "Settings backed up to $BACKUP_DIR"
}

#-------------------------------------------------------------------------------
# Disable Compositor
#-------------------------------------------------------------------------------
disable_compositor() {
    log_step "Disabling compositor for gaming..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would disable compositor"
        return
    fi

    # KDE Plasma (kwin)
    if command -v kwriteconfig5 &>/dev/null; then
        # Disable compositor via DBus
        qdbus org.kde.KWin /Compositor suspend 2>/dev/null || true
        log_success "KWin compositor suspended"
    fi

    # Picom
    if pgrep -x "picom" > /dev/null; then
        pkill picom 2>/dev/null || true
        log_success "Picom compositor stopped"
    fi

    # Compton
    if pgrep -x "compton" > /dev/null; then
        pkill compton 2>/dev/null || true
        log_success "Compton compositor stopped"
    fi

    # XFCE
    if command -v xfconf-query &>/dev/null; then
        xfconf-query -c xfwm4 -p /general/use_compositing -s false 2>/dev/null || true
    fi

    log_success "Compositor disabled"
}

#-------------------------------------------------------------------------------
# Enable Compositor
#-------------------------------------------------------------------------------
enable_compositor() {
    log_step "Re-enabling compositor..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would enable compositor"
        return
    fi

    # KDE Plasma (kwin)
    if command -v kwriteconfig5 &>/dev/null; then
        qdbus org.kde.KWin /Compositor resume 2>/dev/null || true
        log_success "KWin compositor resumed"
    fi

    # Check backup for which compositor was running
    if [ -f "$BACKUP_DIR/compositor.txt" ]; then
        local compositor=$(cat "$BACKUP_DIR/compositor.txt")
        case "$compositor" in
            picom)
                picom --daemon 2>/dev/null &
                log_success "Picom compositor restarted"
                ;;
            compton)
                compton --daemon 2>/dev/null &
                log_success "Compton compositor restarted"
                ;;
        esac
    fi

    # XFCE
    if command -v xfconf-query &>/dev/null; then
        xfconf-query -c xfwm4 -p /general/use_compositing -s true 2>/dev/null || true
    fi
}

#-------------------------------------------------------------------------------
# GameMode Integration
#-------------------------------------------------------------------------------
setup_gamemode() {
    log_step "Configuring GameMode integration..."

    if ! command -v gamemoded &>/dev/null; then
        log_warn "GameMode not installed. Install with: sudo apt install gamemode"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure GameMode"
        return
    fi

    # Create GameMode configuration
    mkdir -p /etc/gamemode.ini.d
    cat > /etc/gamemode.ini.d/winux-gaming.ini << 'EOF'
[general]
renice=10
softrealtime=auto
ioprio=0
inhibit_screensaver=1

[gpu]
apply_gpu_optimisations=accept-responsibility
gpu_device=0
nv_powermizer_mode=1
amd_performance_level=high

[cpu]
pin_cores=1

[custom]
start=notify-send "Winux Gaming" "Game Mode Enabled"
end=notify-send "Winux Gaming" "Game Mode Disabled"
EOF

    # Start GameMode daemon
    systemctl --user start gamemoded 2>/dev/null || \
    gamemoded -d 2>/dev/null &

    log_success "GameMode configured and started"
}

#-------------------------------------------------------------------------------
# Set Process Priority
#-------------------------------------------------------------------------------
set_game_priority() {
    log_step "Setting game process priority..."

    if [ -z "$GAME_PID" ]; then
        log_info "No specific game PID provided"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would set nice priority for PID $GAME_PID"
        return
    fi

    # Verify PID exists
    if ! kill -0 "$GAME_PID" 2>/dev/null; then
        log_error "PID $GAME_PID does not exist"
        return
    fi

    # Set nice priority (lower = higher priority, -20 to 19)
    renice -n -10 -p "$GAME_PID" 2>/dev/null || true

    # Set IO priority (0 = real-time)
    ionice -c 1 -n 0 -p "$GAME_PID" 2>/dev/null || true

    # Set CPU affinity to performance cores (skip core 0,1)
    local cpu_count=$(nproc)
    if [ "$cpu_count" -ge 4 ]; then
        taskset -cp 2-$((cpu_count - 1)) "$GAME_PID" 2>/dev/null || true
    fi

    log_success "Game PID $GAME_PID priority optimized"
}

#-------------------------------------------------------------------------------
# Disable ASLR Temporarily
#-------------------------------------------------------------------------------
disable_aslr() {
    log_step "Temporarily disabling ASLR for gaming..."

    echo -e "${YELLOW}WARNING: Disabling ASLR reduces security.${NC}"
    echo -e "${YELLOW}This will be re-enabled when gaming mode is disabled.${NC}"

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would disable ASLR"
        return
    fi

    # Save current state
    cat /proc/sys/kernel/randomize_va_space > "$BACKUP_DIR/aslr.txt" 2>/dev/null || true

    # Disable ASLR
    echo 0 > /proc/sys/kernel/randomize_va_space

    log_success "ASLR temporarily disabled"
}

#-------------------------------------------------------------------------------
# Enable ASLR
#-------------------------------------------------------------------------------
enable_aslr() {
    log_step "Re-enabling ASLR..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would enable ASLR"
        return
    fi

    # Restore from backup or use default (2)
    local aslr_val=2
    if [ -f "$BACKUP_DIR/aslr.txt" ]; then
        aslr_val=$(cat "$BACKUP_DIR/aslr.txt")
    fi

    echo "$aslr_val" > /proc/sys/kernel/randomize_va_space

    log_success "ASLR re-enabled (value: $aslr_val)"
}

#-------------------------------------------------------------------------------
# GPU Power Management
#-------------------------------------------------------------------------------
optimize_gpu() {
    log_step "Optimizing GPU power management..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would optimize GPU settings"
        return
    fi

    # AMD GPU
    if [ -f /sys/class/drm/card0/device/power_dpm_force_performance_level ]; then
        # Save current state
        cat /sys/class/drm/card0/device/power_dpm_force_performance_level > "$BACKUP_DIR/gpu_power.txt" 2>/dev/null || true

        # Set to high performance
        echo "high" > /sys/class/drm/card0/device/power_dpm_force_performance_level 2>/dev/null || true
        log_success "AMD GPU set to high performance"
    fi

    # NVIDIA GPU
    if command -v nvidia-settings &>/dev/null; then
        # Set performance mode
        nvidia-settings -a "[gpu:0]/GpuPowerMizerMode=1" 2>/dev/null || true

        # Disable power management
        nvidia-smi -pm 1 2>/dev/null || true
        nvidia-smi --auto-boost-default=0 2>/dev/null || true
        nvidia-smi -pl 999 2>/dev/null || true  # Max power limit

        log_success "NVIDIA GPU optimized for performance"
    fi

    # Intel GPU
    if [ -d /sys/class/drm/card0/gt ]; then
        # Set min frequency to max
        local max_freq=$(cat /sys/class/drm/card0/gt_max_freq_mhz 2>/dev/null)
        if [ -n "$max_freq" ]; then
            echo "$max_freq" > /sys/class/drm/card0/gt_min_freq_mhz 2>/dev/null || true
            log_success "Intel GPU frequency locked to max"
        fi
    fi
}

#-------------------------------------------------------------------------------
# Restore GPU Settings
#-------------------------------------------------------------------------------
restore_gpu() {
    log_step "Restoring GPU settings..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would restore GPU settings"
        return
    fi

    # AMD GPU
    if [ -f /sys/class/drm/card0/device/power_dpm_force_performance_level ]; then
        local gpu_state="auto"
        if [ -f "$BACKUP_DIR/gpu_power.txt" ]; then
            gpu_state=$(cat "$BACKUP_DIR/gpu_power.txt")
        fi
        echo "$gpu_state" > /sys/class/drm/card0/device/power_dpm_force_performance_level 2>/dev/null || true
        log_success "AMD GPU power restored to $gpu_state"
    fi

    # NVIDIA GPU
    if command -v nvidia-settings &>/dev/null; then
        nvidia-settings -a "[gpu:0]/GpuPowerMizerMode=0" 2>/dev/null || true  # Auto mode
        log_success "NVIDIA GPU restored to auto mode"
    fi

    # Intel GPU
    if [ -d /sys/class/drm/card0/gt ]; then
        local min_freq=$(cat /sys/class/drm/card0/gt_RPn_freq_mhz 2>/dev/null)
        if [ -n "$min_freq" ]; then
            echo "$min_freq" > /sys/class/drm/card0/gt_min_freq_mhz 2>/dev/null || true
            log_success "Intel GPU frequency unlocked"
        fi
    fi
}

#-------------------------------------------------------------------------------
# Shader Cache Warmup
#-------------------------------------------------------------------------------
warmup_shader_cache() {
    log_step "Warming up shader cache..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would warmup shader cache"
        return
    fi

    # Mesa shader cache
    local mesa_cache="$HOME/.cache/mesa_shader_cache"
    if [ -d "$mesa_cache" ]; then
        # Touch files to keep them in memory
        find "$mesa_cache" -type f -exec cat {} > /dev/null 2>&1 \;
        log_success "Mesa shader cache warmed up"
    fi

    # Steam shader cache
    local steam_cache="$HOME/.steam/steam/steamapps/shadercache"
    if [ -d "$steam_cache" ]; then
        find "$steam_cache" -type f -name "*.foz" -exec cat {} > /dev/null 2>&1 \;
        log_success "Steam shader cache warmed up"
    fi

    # DXVK cache
    local dxvk_cache="$HOME/.cache/dxvk"
    if [ -d "$dxvk_cache" ]; then
        find "$dxvk_cache" -type f -exec cat {} > /dev/null 2>&1 \;
        log_success "DXVK cache warmed up"
    fi

    # vkBasalt cache
    local vkbasalt_cache="$HOME/.cache/vkBasalt"
    if [ -d "$vkbasalt_cache" ]; then
        find "$vkbasalt_cache" -type f -exec cat {} > /dev/null 2>&1 \;
        log_success "vkBasalt cache warmed up"
    fi

    log_success "Shader caches warmed up"
}

#-------------------------------------------------------------------------------
# Additional Gaming Optimizations
#-------------------------------------------------------------------------------
apply_gaming_tweaks() {
    log_step "Applying additional gaming tweaks..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would apply gaming tweaks"
        return
    fi

    # Disable kernel watchdog
    echo 0 > /proc/sys/kernel/nmi_watchdog 2>/dev/null || true

    # Disable kernel split lock detection (helps some games)
    echo 2 > /proc/sys/kernel/split_lock_mitigate 2>/dev/null || true

    # Increase file descriptors limit
    ulimit -n 1048576 2>/dev/null || true

    # Disable mouse acceleration (X11)
    if command -v xinput &>/dev/null; then
        for id in $(xinput list --id-only 2>/dev/null); do
            xinput set-prop "$id" "libinput Accel Profile Enabled" 0 1 2>/dev/null || true
        done
    fi

    # Stop unnecessary services
    local services_to_stop=(
        "cups"
        "bluetooth"
        "ModemManager"
        "avahi-daemon"
        "packagekit"
    )

    for service in "${services_to_stop[@]}"; do
        if systemctl is-active --quiet "$service" 2>/dev/null; then
            systemctl stop "$service" 2>/dev/null || true
            echo "$service" >> "$BACKUP_DIR/stopped_services.txt"
        fi
    done

    log_success "Gaming tweaks applied"
}

#-------------------------------------------------------------------------------
# Restore Gaming Tweaks
#-------------------------------------------------------------------------------
restore_gaming_tweaks() {
    log_step "Restoring normal settings..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would restore normal settings"
        return
    fi

    # Re-enable kernel watchdog
    echo 1 > /proc/sys/kernel/nmi_watchdog 2>/dev/null || true

    # Restore split lock detection
    echo 1 > /proc/sys/kernel/split_lock_mitigate 2>/dev/null || true

    # Restart stopped services
    if [ -f "$BACKUP_DIR/stopped_services.txt" ]; then
        while read -r service; do
            systemctl start "$service" 2>/dev/null || true
        done < "$BACKUP_DIR/stopped_services.txt"
    fi

    log_success "Normal settings restored"
}

#-------------------------------------------------------------------------------
# Show Status
#-------------------------------------------------------------------------------
show_status() {
    echo -e "${WHITE}=== Gaming Mode Status ===${NC}"
    echo ""

    # Check if gaming mode is active
    local gaming_active=false
    if [ -f /var/lib/winux/.gaming-mode-active ]; then
        gaming_active=true
        echo -e "${GREEN}Gaming Mode: ACTIVE${NC}"
    else
        echo -e "${YELLOW}Gaming Mode: INACTIVE${NC}"
    fi
    echo ""

    # Compositor status
    echo -e "${CYAN}Compositor:${NC}"
    if pgrep -x "kwin_x11" > /dev/null; then
        echo "  KWin: Running"
    elif pgrep -x "picom" > /dev/null; then
        echo "  Picom: Running"
    else
        echo "  No compositor detected"
    fi
    echo ""

    # GameMode status
    echo -e "${CYAN}GameMode:${NC}"
    if pgrep -x "gamemoded" > /dev/null; then
        echo "  Status: Running"
        gamemoded -s 2>/dev/null || true
    else
        echo "  Status: Not running"
    fi
    echo ""

    # ASLR status
    echo -e "${CYAN}ASLR:${NC}"
    local aslr=$(cat /proc/sys/kernel/randomize_va_space)
    case $aslr in
        0) echo "  Status: Disabled (security risk)" ;;
        1) echo "  Status: Partial" ;;
        2) echo "  Status: Full (secure)" ;;
    esac
    echo ""

    # GPU status
    echo -e "${CYAN}GPU Power:${NC}"
    if [ -f /sys/class/drm/card0/device/power_dpm_force_performance_level ]; then
        echo "  AMD: $(cat /sys/class/drm/card0/device/power_dpm_force_performance_level)"
    fi
    if command -v nvidia-smi &>/dev/null; then
        nvidia-smi --query-gpu=power.management --format=csv,noheader 2>/dev/null || true
    fi
}

#-------------------------------------------------------------------------------
# Enable Gaming Mode
#-------------------------------------------------------------------------------
enable_gaming_mode() {
    local total_steps=7
    local current_step=0

    echo -e "${GREEN}=== Enabling Gaming Mode ===${NC}"
    echo ""

    # Create backup
    ((current_step++))
    show_progress $current_step $total_steps
    create_backup

    # Disable compositor
    ((current_step++))
    show_progress $current_step $total_steps
    disable_compositor

    # Setup GameMode
    ((current_step++))
    show_progress $current_step $total_steps
    setup_gamemode

    # Set game priority if PID provided
    ((current_step++))
    show_progress $current_step $total_steps
    set_game_priority

    # Disable ASLR
    ((current_step++))
    show_progress $current_step $total_steps
    disable_aslr

    # Optimize GPU
    ((current_step++))
    show_progress $current_step $total_steps
    optimize_gpu

    # Warmup shader cache
    ((current_step++))
    show_progress $current_step $total_steps
    warmup_shader_cache

    # Apply gaming tweaks
    apply_gaming_tweaks

    # Mark gaming mode as active
    if ! $DRY_RUN; then
        mkdir -p /var/lib/winux
        echo "$(date)" > /var/lib/winux/.gaming-mode-active
    fi

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}      Gaming Mode ENABLED!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "Optimizations applied:"
    echo "  - Compositor disabled"
    echo "  - GameMode active"
    echo "  - ASLR temporarily disabled"
    echo "  - GPU set to high performance"
    echo "  - Shader caches warmed up"
    echo "  - Background services paused"
    echo ""
    echo "To disable: $SCRIPT_NAME --disable"
}

#-------------------------------------------------------------------------------
# Disable Gaming Mode
#-------------------------------------------------------------------------------
disable_gaming_mode() {
    echo -e "${YELLOW}=== Disabling Gaming Mode ===${NC}"
    echo ""

    # Find latest backup
    local latest_backup=$(ls -td /var/lib/winux/backups/gaming-* 2>/dev/null | head -1)
    if [ -n "$latest_backup" ]; then
        BACKUP_DIR="$latest_backup"
    fi

    enable_compositor
    enable_aslr
    restore_gpu
    restore_gaming_tweaks

    # Remove active marker
    if ! $DRY_RUN; then
        rm -f /var/lib/winux/.gaming-mode-active
    fi

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}      Gaming Mode DISABLED${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "Normal settings restored."
}

#-------------------------------------------------------------------------------
# Main Function
#-------------------------------------------------------------------------------
main() {
    local mode="enable"

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -d|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -e|--enable)
                mode="enable"
                shift
                ;;
            -x|--disable)
                mode="disable"
                shift
                ;;
            -p|--pid)
                GAME_PID="$2"
                shift 2
                ;;
            -s|--status)
                show_logo
                show_status
                exit 0
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--version)
                echo "Winux Gaming Optimizer v$VERSION"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # Check root unless dry-run
    if ! $DRY_RUN; then
        check_root
    fi

    # Create log directory
    mkdir -p /var/log/winux 2>/dev/null || true
    mkdir -p /var/lib/winux/backups 2>/dev/null || true

    # Show logo
    show_logo

    if $DRY_RUN; then
        echo -e "${YELLOW}=== DRY RUN MODE - No changes will be made ===${NC}"
        echo ""
    fi

    # Execute based on mode
    case $mode in
        enable)
            enable_gaming_mode
            ;;
        disable)
            disable_gaming_mode
            ;;
    esac
}

main "$@"
