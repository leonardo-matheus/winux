#!/bin/bash
# =============================================================================
# Winux OS - GameMode End Script
# Sprint 13-14: Windows Compatibility Layer
# =============================================================================
#
# This script is executed when GameMode ends (game exit detected)
# It restores system settings to their pre-gaming state
#
# Usage: Called automatically by GameMode or manually via:
#        gamemoded --request-end
# =============================================================================

set -e

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
WINUX_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/winux"
GAMEMODE_LOG="${WINUX_CONFIG_DIR}/gamemode.log"
GAMEMODE_STATE="${WINUX_CONFIG_DIR}/gamemode.state"

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
# Load Saved State
# -----------------------------------------------------------------------------
load_state() {
    if [[ -f "$GAMEMODE_STATE" ]]; then
        log "INFO" "Loading saved state from $GAMEMODE_STATE"
        # shellcheck source=/dev/null
        source "$GAMEMODE_STATE"
        return 0
    else
        log "WARN" "No saved state file found, using defaults"
        return 1
    fi
}

# -----------------------------------------------------------------------------
# Restore CPU Settings
# -----------------------------------------------------------------------------
restore_cpu() {
    log "INFO" "Restoring CPU settings..."

    local target_governor="${CPU_GOVERNOR:-ondemand}"

    # Restore CPU governor
    if [[ -w /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]]; then
        for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
            if [[ -w "$cpu" ]]; then
                # Check if governor is available
                local available
                available=$(cat "${cpu%/*}/scaling_available_governors" 2>/dev/null || echo "")
                if echo "$available" | grep -q "$target_governor"; then
                    echo "$target_governor" > "$cpu" 2>/dev/null || true
                else
                    # Fallback to schedutil or ondemand
                    if echo "$available" | grep -q "schedutil"; then
                        echo "schedutil" > "$cpu" 2>/dev/null || true
                    elif echo "$available" | grep -q "ondemand"; then
                        echo "ondemand" > "$cpu" 2>/dev/null || true
                    fi
                fi
            fi
        done
        log "INFO" "CPU governor restored to $target_governor"
    else
        if command -v cpupower &>/dev/null; then
            sudo cpupower frequency-set -g "$target_governor" 2>/dev/null || true
        fi
    fi
}

# -----------------------------------------------------------------------------
# Restore GPU Settings
# -----------------------------------------------------------------------------
restore_gpu() {
    log "INFO" "Restoring GPU settings..."

    # Restore AMD GPU settings
    if [[ -d /sys/class/drm/card0/device ]]; then
        # Restore performance level
        local amd_perf="${AMD_PERF_LEVEL:-auto}"
        if [[ -w /sys/class/drm/card0/device/power_dpm_force_performance_level ]]; then
            echo "$amd_perf" > /sys/class/drm/card0/device/power_dpm_force_performance_level 2>/dev/null || true
            log "INFO" "AMD GPU performance level restored to $amd_perf"
        fi

        # Restore power state
        local amd_dpm="${AMD_DPM_STATE:-balanced}"
        if [[ -w /sys/class/drm/card0/device/power_dpm_state ]]; then
            echo "$amd_dpm" > /sys/class/drm/card0/device/power_dpm_state 2>/dev/null || true
        fi
    fi

    # Restore NVIDIA GPU settings
    if command -v nvidia-smi &>/dev/null; then
        # Set adaptive power mode
        nvidia-settings -a "[gpu:0]/GpuPowerMizerMode=0" 2>/dev/null || true

        # Restore power limit if saved
        if [[ -n "${NVIDIA_POWER_LIMIT:-}" ]]; then
            sudo nvidia-smi -pl "$NVIDIA_POWER_LIMIT" 2>/dev/null || true
        fi

        log "INFO" "NVIDIA GPU settings restored"
    fi

    # Restore Intel GPU settings
    if [[ -d /sys/class/drm/card0/gt ]]; then
        # Reset to auto frequency scaling
        if [[ -w /sys/class/drm/card0/gt_min_freq_mhz ]]; then
            local min_freq
            min_freq=$(cat /sys/class/drm/card0/gt_RPn_freq_mhz 2>/dev/null || echo "")
            if [[ -n "$min_freq" ]]; then
                echo "$min_freq" > /sys/class/drm/card0/gt_min_freq_mhz 2>/dev/null || true
            fi
        fi
    fi
}

# -----------------------------------------------------------------------------
# Restore Compositor
# -----------------------------------------------------------------------------
restore_compositor() {
    log "INFO" "Restoring compositor..."

    local compositor="${COMPOSITOR:-none}"

    case "$compositor" in
        picom)
            if ! pgrep -x "picom" &>/dev/null; then
                picom --daemon 2>/dev/null || picom -b 2>/dev/null || true
                log "INFO" "Picom compositor restarted"
            fi
            ;;
        compton)
            if ! pgrep -x "compton" &>/dev/null; then
                compton --daemon 2>/dev/null || compton -b 2>/dev/null || true
                log "INFO" "Compton compositor restarted"
            fi
            ;;
        xcompmgr)
            if ! pgrep -x "xcompmgr" &>/dev/null; then
                xcompmgr & 2>/dev/null
                log "INFO" "Xcompmgr compositor restarted"
            fi
            ;;
        none)
            log "INFO" "No compositor to restore"
            ;;
        *)
            log "WARN" "Unknown compositor: $compositor"
            ;;
    esac

    # Resume KWin compositing
    if command -v qdbus &>/dev/null; then
        qdbus org.kde.KWin /Compositor resume 2>/dev/null || true
    fi
}

# -----------------------------------------------------------------------------
# Restore Memory Settings
# -----------------------------------------------------------------------------
restore_memory() {
    log "INFO" "Restoring memory settings..."

    # Restore default swappiness
    echo 60 > /proc/sys/vm/swappiness 2>/dev/null || true

    log "INFO" "Memory settings restored"
}

# -----------------------------------------------------------------------------
# Restore I/O Settings
# -----------------------------------------------------------------------------
restore_io() {
    log "INFO" "Restoring I/O settings..."

    # Restore default I/O scheduler (usually already set correctly)
    # Most systems use mq-deadline by default anyway

    # Restore default read-ahead
    for device in /sys/block/sd*/queue/read_ahead_kb /sys/block/nvme*/queue/read_ahead_kb; do
        if [[ -w "$device" ]]; then
            echo 128 > "$device" 2>/dev/null || true
        fi
    done

    log "INFO" "I/O settings restored"
}

# -----------------------------------------------------------------------------
# Restore Screen Blanking and DPMS
# -----------------------------------------------------------------------------
restore_screen_blanking() {
    log "INFO" "Restoring screen blanking and power management..."

    local dpms_enabled="${DPMS_ENABLED:-1}"

    if [[ "$dpms_enabled" == "1" ]]; then
        # Re-enable DPMS
        xset +dpms 2>/dev/null || true

        # Restore default timeouts (standby: 10min, suspend: 15min, off: 20min)
        xset dpms 600 900 1200 2>/dev/null || true
    fi

    # Re-enable screen saver
    xset s on 2>/dev/null || true
    xset s default 2>/dev/null || true

    # Kill systemd-inhibit if we started one
    if [[ -f "${WINUX_CONFIG_DIR}/inhibit.pid" ]]; then
        local inhibit_pid
        inhibit_pid=$(cat "${WINUX_CONFIG_DIR}/inhibit.pid" 2>/dev/null || echo "")
        if [[ -n "$inhibit_pid" ]] && [[ -d "/proc/$inhibit_pid" ]]; then
            kill "$inhibit_pid" 2>/dev/null || true
        fi
        rm -f "${WINUX_CONFIG_DIR}/inhibit.pid"
    fi

    log "INFO" "Screen blanking restored"
}

# -----------------------------------------------------------------------------
# Restore Network Settings
# -----------------------------------------------------------------------------
restore_network() {
    log "INFO" "Restoring network settings..."

    # Restore default TCP settings
    echo 0 > /proc/sys/net/ipv4/tcp_low_latency 2>/dev/null || true

    log "INFO" "Network settings restored"
}

# -----------------------------------------------------------------------------
# Clean Up Temporary Files
# -----------------------------------------------------------------------------
cleanup() {
    log "INFO" "Cleaning up temporary files..."

    # Remove PID files
    rm -f "${WINUX_CONFIG_DIR}/gamemode-start.pid" 2>/dev/null || true
    rm -f "${WINUX_CONFIG_DIR}/inhibit.pid" 2>/dev/null || true

    # Keep state file for debugging but rotate if too large
    if [[ -f "$GAMEMODE_STATE" ]]; then
        local size
        size=$(stat -f%z "$GAMEMODE_STATE" 2>/dev/null || stat -c%s "$GAMEMODE_STATE" 2>/dev/null || echo 0)
        if [[ "$size" -gt 10240 ]]; then
            mv "$GAMEMODE_STATE" "${GAMEMODE_STATE}.old" 2>/dev/null || true
        fi
    fi

    # Rotate log file if too large (>10MB)
    if [[ -f "$GAMEMODE_LOG" ]]; then
        local size
        size=$(stat -f%z "$GAMEMODE_LOG" 2>/dev/null || stat -c%s "$GAMEMODE_LOG" 2>/dev/null || echo 0)
        if [[ "$size" -gt 10485760 ]]; then
            mv "$GAMEMODE_LOG" "${GAMEMODE_LOG}.old" 2>/dev/null || true
            gzip "${GAMEMODE_LOG}.old" 2>/dev/null || true
        fi
    fi

    log "INFO" "Cleanup completed"
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
# Generate Session Summary
# -----------------------------------------------------------------------------
generate_summary() {
    local start_time
    local end_time
    local duration

    end_time=$(date +%s)

    if [[ -f "${WINUX_CONFIG_DIR}/gamemode-start.pid" ]]; then
        start_time=$(stat -c %Y "${WINUX_CONFIG_DIR}/gamemode-start.pid" 2>/dev/null || echo "$end_time")
        duration=$((end_time - start_time))

        local hours=$((duration / 3600))
        local minutes=$(((duration % 3600) / 60))
        local seconds=$((duration % 60))

        log "INFO" "Gaming session duration: ${hours}h ${minutes}m ${seconds}s"

        # Return formatted duration for notification
        if [[ $hours -gt 0 ]]; then
            echo "${hours}h ${minutes}m"
        elif [[ $minutes -gt 0 ]]; then
            echo "${minutes}m ${seconds}s"
        else
            echo "${seconds}s"
        fi
    else
        echo "Unknown"
    fi
}

# -----------------------------------------------------------------------------
# Main Execution
# -----------------------------------------------------------------------------
main() {
    log "INFO" "=========================================="
    log "INFO" "GameMode Ending - $(date)"
    log "INFO" "=========================================="

    # Generate session summary before cleanup
    local session_duration
    session_duration=$(generate_summary)

    # Load saved state
    load_state || true

    # Restore all settings
    restore_cpu
    restore_gpu
    restore_memory
    restore_io
    restore_network
    restore_screen_blanking

    # Restore compositor if configured
    if [[ "${WINUX_GAMEMODE_RESTORE_COMPOSITOR:-1}" == "1" ]]; then
        restore_compositor
    fi

    # Clean up
    cleanup

    # Send notification
    notify "GameMode Deactivated" "Gaming session ended. Duration: $session_duration" "dialog-information"

    log "INFO" "GameMode end completed successfully"
    log "INFO" "=========================================="
}

# Run main function
main "$@"
