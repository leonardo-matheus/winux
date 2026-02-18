#!/bin/bash
# =============================================================================
# Winux OS - GameMode Start Script
# Sprint 13-14: Windows Compatibility Layer
# =============================================================================
#
# This script is executed when GameMode starts (game launch detected)
# It applies performance optimizations for gaming sessions
#
# Usage: Called automatically by GameMode or manually via:
#        gamemoded --request-start
# =============================================================================

set -e

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
WINUX_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/winux"
GAMEMODE_LOG="${WINUX_CONFIG_DIR}/gamemode.log"
GAMEMODE_STATE="${WINUX_CONFIG_DIR}/gamemode.state"

# Create config directory if it doesn't exist
mkdir -p "$WINUX_CONFIG_DIR"

# -----------------------------------------------------------------------------
# Logging Function
# -----------------------------------------------------------------------------
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" >> "$GAMEMODE_LOG"

    if [[ "$WINUX_GAMEMODE_DEBUG" == "1" ]]; then
        echo "[$level] $message"
    fi
}

# -----------------------------------------------------------------------------
# Save Current State (for restoration later)
# -----------------------------------------------------------------------------
save_state() {
    log "INFO" "Saving current system state..."

    {
        # Save CPU governor
        echo "CPU_GOVERNOR=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'unknown')"

        # Save GPU power state (AMD)
        if [[ -f /sys/class/drm/card0/device/power_dpm_state ]]; then
            echo "AMD_DPM_STATE=$(cat /sys/class/drm/card0/device/power_dpm_state)"
        fi

        # Save GPU performance level (AMD)
        if [[ -f /sys/class/drm/card0/device/power_dpm_force_performance_level ]]; then
            echo "AMD_PERF_LEVEL=$(cat /sys/class/drm/card0/device/power_dpm_force_performance_level)"
        fi

        # Save NVIDIA settings
        if command -v nvidia-smi &>/dev/null; then
            local nvidia_power
            nvidia_power=$(nvidia-smi --query-gpu=power.limit --format=csv,noheader,nounits 2>/dev/null || echo "")
            if [[ -n "$nvidia_power" ]]; then
                echo "NVIDIA_POWER_LIMIT=$nvidia_power"
            fi
        fi

        # Save compositor state
        if pgrep -x "picom" &>/dev/null; then
            echo "COMPOSITOR=picom"
        elif pgrep -x "compton" &>/dev/null; then
            echo "COMPOSITOR=compton"
        elif pgrep -x "xcompmgr" &>/dev/null; then
            echo "COMPOSITOR=xcompmgr"
        else
            echo "COMPOSITOR=none"
        fi

        # Save screen blanking
        echo "DPMS_ENABLED=$(xset q 2>/dev/null | grep -q 'DPMS is Enabled' && echo 1 || echo 0)"

    } > "$GAMEMODE_STATE"

    log "INFO" "State saved to $GAMEMODE_STATE"
}

# -----------------------------------------------------------------------------
# Apply CPU Optimizations
# -----------------------------------------------------------------------------
optimize_cpu() {
    log "INFO" "Applying CPU optimizations..."

    # Set CPU governor to performance
    if [[ -w /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]]; then
        for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
            if [[ -w "$cpu" ]]; then
                echo "performance" > "$cpu" 2>/dev/null || true
            fi
        done
        log "INFO" "CPU governor set to performance"
    else
        # Try with cpupower if available
        if command -v cpupower &>/dev/null; then
            sudo cpupower frequency-set -g performance 2>/dev/null || true
            log "INFO" "CPU governor set via cpupower"
        fi
    fi

    # Disable CPU frequency boost limits (Intel)
    if [[ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]]; then
        echo "0" > /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null || true
        log "INFO" "Intel Turbo Boost enabled"
    fi

    # Enable AMD boost
    if [[ -f /sys/devices/system/cpu/cpufreq/boost ]]; then
        echo "1" > /sys/devices/system/cpu/cpufreq/boost 2>/dev/null || true
        log "INFO" "AMD CPU Boost enabled"
    fi
}

# -----------------------------------------------------------------------------
# Apply GPU Optimizations
# -----------------------------------------------------------------------------
optimize_gpu() {
    log "INFO" "Applying GPU optimizations..."

    # AMD GPU Optimizations
    if [[ -d /sys/class/drm/card0/device ]]; then
        # Set performance mode
        if [[ -w /sys/class/drm/card0/device/power_dpm_force_performance_level ]]; then
            echo "high" > /sys/class/drm/card0/device/power_dpm_force_performance_level 2>/dev/null || true
            log "INFO" "AMD GPU set to high performance"
        fi

        # Set power state
        if [[ -w /sys/class/drm/card0/device/power_dpm_state ]]; then
            echo "performance" > /sys/class/drm/card0/device/power_dpm_state 2>/dev/null || true
        fi
    fi

    # NVIDIA GPU Optimizations
    if command -v nvidia-smi &>/dev/null; then
        # Set performance mode
        nvidia-settings -a "[gpu:0]/GpuPowerMizerMode=1" 2>/dev/null || true

        # Enable persistence mode
        sudo nvidia-smi -pm 1 2>/dev/null || true

        log "INFO" "NVIDIA GPU optimizations applied"
    fi

    # Intel GPU Optimizations
    if [[ -d /sys/class/drm/card0/gt ]]; then
        # Set minimum frequency to maximum for better sustained performance
        if [[ -w /sys/class/drm/card0/gt_min_freq_mhz ]]; then
            local max_freq
            max_freq=$(cat /sys/class/drm/card0/gt_max_freq_mhz 2>/dev/null || echo "")
            if [[ -n "$max_freq" ]]; then
                echo "$max_freq" > /sys/class/drm/card0/gt_min_freq_mhz 2>/dev/null || true
                log "INFO" "Intel GPU frequency locked to maximum"
            fi
        fi
    fi
}

# -----------------------------------------------------------------------------
# Disable Compositors and Desktop Effects
# -----------------------------------------------------------------------------
disable_compositor() {
    log "INFO" "Disabling compositor for reduced input latency..."

    # Kill common compositors
    pkill -x picom 2>/dev/null || true
    pkill -x compton 2>/dev/null || true
    pkill -x xcompmgr 2>/dev/null || true

    # Disable KWin compositing
    if command -v qdbus &>/dev/null; then
        qdbus org.kde.KWin /Compositor suspend 2>/dev/null || true
    fi

    # Disable Mutter compositing (GNOME) - not recommended but available
    # gsettings set org.gnome.mutter attach-modal-dialogs false 2>/dev/null || true

    log "INFO" "Compositor disabled"
}

# -----------------------------------------------------------------------------
# Apply Memory Optimizations
# -----------------------------------------------------------------------------
optimize_memory() {
    log "INFO" "Applying memory optimizations..."

    # Drop caches to free up memory
    sync
    echo 1 > /proc/sys/vm/drop_caches 2>/dev/null || true

    # Reduce swappiness for gaming (prefer RAM over swap)
    echo 10 > /proc/sys/vm/swappiness 2>/dev/null || true

    # Increase file descriptor limits
    ulimit -n 524288 2>/dev/null || true

    log "INFO" "Memory optimizations applied"
}

# -----------------------------------------------------------------------------
# Apply I/O Optimizations
# -----------------------------------------------------------------------------
optimize_io() {
    log "INFO" "Applying I/O optimizations..."

    # Set I/O scheduler to deadline or mq-deadline for better gaming I/O
    for device in /sys/block/sd*/queue/scheduler /sys/block/nvme*/queue/scheduler; do
        if [[ -w "$device" ]]; then
            if grep -q "mq-deadline" "$device" 2>/dev/null; then
                echo "mq-deadline" > "$device" 2>/dev/null || true
            elif grep -q "deadline" "$device" 2>/dev/null; then
                echo "deadline" > "$device" 2>/dev/null || true
            fi
        fi
    done

    # Increase read-ahead buffer
    for device in /sys/block/sd*/queue/read_ahead_kb /sys/block/nvme*/queue/read_ahead_kb; do
        if [[ -w "$device" ]]; then
            echo 2048 > "$device" 2>/dev/null || true
        fi
    done

    log "INFO" "I/O optimizations applied"
}

# -----------------------------------------------------------------------------
# Disable Screen Blanking and DPMS
# -----------------------------------------------------------------------------
disable_screen_blanking() {
    log "INFO" "Disabling screen blanking and power management..."

    # Disable DPMS
    xset -dpms 2>/dev/null || true

    # Disable screen saver
    xset s off 2>/dev/null || true
    xset s noblank 2>/dev/null || true

    # Disable screen blanking timeout
    xset s 0 0 2>/dev/null || true

    # Inhibit systemd idle
    if command -v systemd-inhibit &>/dev/null; then
        systemd-inhibit --what=idle --who="GameMode" --why="Gaming session active" --mode=block sleep infinity &
        echo $! > "${WINUX_CONFIG_DIR}/inhibit.pid"
    fi

    log "INFO" "Screen blanking disabled"
}

# -----------------------------------------------------------------------------
# Apply Network Optimizations
# -----------------------------------------------------------------------------
optimize_network() {
    log "INFO" "Applying network optimizations..."

    # Disable Nagle's algorithm for lower latency
    for iface in /proc/sys/net/ipv4/tcp_*; do
        case "$(basename "$iface")" in
            tcp_low_latency)
                echo 1 > "$iface" 2>/dev/null || true
                ;;
        esac
    done

    # Optimize network buffers
    echo 4194304 > /proc/sys/net/core/rmem_max 2>/dev/null || true
    echo 4194304 > /proc/sys/net/core/wmem_max 2>/dev/null || true

    log "INFO" "Network optimizations applied"
}

# -----------------------------------------------------------------------------
# Set Process Priority
# -----------------------------------------------------------------------------
set_process_priority() {
    local game_pid="$1"

    if [[ -n "$game_pid" ]] && [[ -d "/proc/$game_pid" ]]; then
        log "INFO" "Setting process priority for PID $game_pid..."

        # Set nice value
        renice -n -10 -p "$game_pid" 2>/dev/null || true

        # Set I/O priority
        ionice -c 1 -n 0 -p "$game_pid" 2>/dev/null || true

        # Set CPU affinity if multiple cores available
        local cpu_count
        cpu_count=$(nproc)
        if [[ "$cpu_count" -gt 4 ]]; then
            # Use all performance cores (assume first N/2 are P-cores on hybrid)
            taskset -cp 0-$((cpu_count-1)) "$game_pid" 2>/dev/null || true
        fi

        log "INFO" "Process priority set"
    fi
}

# -----------------------------------------------------------------------------
# Send Desktop Notification
# -----------------------------------------------------------------------------
notify() {
    local title="$1"
    local message="$2"
    local icon="${3:-applications-games}"

    if command -v notify-send &>/dev/null; then
        notify-send -i "$icon" -a "Winux GameMode" "$title" "$message" 2>/dev/null || true
    fi
}

# -----------------------------------------------------------------------------
# Main Execution
# -----------------------------------------------------------------------------
main() {
    log "INFO" "=========================================="
    log "INFO" "GameMode Starting - $(date)"
    log "INFO" "=========================================="

    # Get game PID if passed as argument
    local game_pid="${1:-}"

    # Save current state for later restoration
    save_state

    # Apply all optimizations
    optimize_cpu
    optimize_gpu
    optimize_memory
    optimize_io
    optimize_network
    disable_screen_blanking

    # Disable compositor if configured
    if [[ "${WINUX_GAMEMODE_DISABLE_COMPOSITOR:-1}" == "1" ]]; then
        disable_compositor
    fi

    # Set game process priority
    if [[ -n "$game_pid" ]]; then
        set_process_priority "$game_pid"
    fi

    # Send notification
    notify "GameMode Activated" "Performance optimizations applied for gaming session"

    log "INFO" "GameMode start completed successfully"

    # Write PID file
    echo "$$" > "${WINUX_CONFIG_DIR}/gamemode-start.pid"
}

# Run main function
main "$@"
