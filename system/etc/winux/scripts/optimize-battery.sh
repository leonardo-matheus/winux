#!/bin/bash
#===============================================================================
# Winux OS - Battery Optimizer
# Power saving and battery life optimization
#===============================================================================

set -e

#-------------------------------------------------------------------------------
# Configuration
#-------------------------------------------------------------------------------
VERSION="1.0.0"
SCRIPT_NAME="$(basename "$0")"
BACKUP_DIR="/var/lib/winux/backups/battery-$(date +%Y%m%d_%H%M%S)"
LOG_FILE="/var/log/winux/optimize-battery.log"
DRY_RUN=false

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
    echo -e "${GREEN}"
    cat << 'EOF'
 __        __ _
 \ \      / /(_) _ __  _   _ __ __
  \ \ /\ / / | || '_ \| | | |\ \/ /
   \ V  V /  | || | | | |_| | >  <
    \_/\_/   |_||_| |_|\__,_|/_/\_\

      Battery Optimizer v1.0.0
     ===========================
         [POWER SAVE MODE]
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
    echo -e "${GREEN}[POWER]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [POWER] $1" >> "$LOG_FILE" 2>/dev/null || true
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

    printf "\r${GREEN}["
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
    echo "Usage: $SCRIPT_NAME [OPTIONS]"
    echo ""
    echo "Battery optimizer for Winux OS"
    echo ""
    echo "Options:"
    echo "  -d, --dry-run       Show what would be done without making changes"
    echo "  -e, --enable        Enable power save mode (default)"
    echo "  -x, --disable       Disable power save mode"
    echo "  -a, --auto          Enable auto mode (switch based on power source)"
    echo "  -s, --status        Show current power status"
    echo "  -r, --restore       Restore from backup"
    echo "  -h, --help          Show this help message"
    echo "  -v, --version       Show version"
    echo ""
    echo "Features:"
    echo "  - TLP configuration"
    echo "  - PowerTop auto-tune"
    echo "  - Display brightness control"
    echo "  - CPU frequency scaling"
    echo "  - USB autosuspend"
    echo "  - WiFi power save"
    echo ""
    echo "Examples:"
    echo "  $SCRIPT_NAME                  # Enable power save"
    echo "  $SCRIPT_NAME --auto           # Auto-switch mode"
    echo "  $SCRIPT_NAME --status         # Show battery info"
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
# Check Battery Presence
#-------------------------------------------------------------------------------
check_battery() {
    if [ ! -d /sys/class/power_supply/BAT0 ] && [ ! -d /sys/class/power_supply/BAT1 ]; then
        log_warn "No battery detected. This appears to be a desktop system."
        echo "Continue anyway? (y/N)"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            exit 0
        fi
    fi
}

#-------------------------------------------------------------------------------
# Create Backup
#-------------------------------------------------------------------------------
create_backup() {
    log_step "Creating backup of current settings..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would create backup in $BACKUP_DIR"
        return
    fi

    mkdir -p "$BACKUP_DIR"

    # Backup TLP config
    if [ -f /etc/tlp.conf ]; then
        cp /etc/tlp.conf "$BACKUP_DIR/"
    fi

    # Backup CPU governor
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            cat "$cpu" > "$BACKUP_DIR/cpu_governors.txt"
            break
        fi
    done

    # Backup brightness
    if [ -f /sys/class/backlight/*/brightness ]; then
        cat /sys/class/backlight/*/brightness > "$BACKUP_DIR/brightness.txt" 2>/dev/null || true
    fi

    log_success "Backup created at $BACKUP_DIR"
}

#-------------------------------------------------------------------------------
# Restore from Backup
#-------------------------------------------------------------------------------
restore_backup() {
    log_step "Restoring from backup..."

    local latest_backup=$(ls -td /var/lib/winux/backups/battery-* 2>/dev/null | head -1)

    if [ -z "$latest_backup" ]; then
        log_error "No backup found to restore"
        exit 1
    fi

    log_info "Restoring from: $latest_backup"

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would restore from $latest_backup"
        return
    fi

    # Restore TLP config
    if [ -f "$latest_backup/tlp.conf" ]; then
        cp "$latest_backup/tlp.conf" /etc/tlp.conf
        systemctl restart tlp 2>/dev/null || true
        log_success "TLP configuration restored"
    fi

    # Restore brightness
    if [ -f "$latest_backup/brightness.txt" ]; then
        local brightness=$(cat "$latest_backup/brightness.txt")
        for bl in /sys/class/backlight/*/brightness; do
            echo "$brightness" > "$bl" 2>/dev/null || true
        done
        log_success "Brightness restored"
    fi

    log_success "Restore complete"
}

#-------------------------------------------------------------------------------
# Configure TLP
#-------------------------------------------------------------------------------
configure_tlp() {
    log_step "Configuring TLP power management..."

    if ! command -v tlp &>/dev/null; then
        log_warn "TLP not installed. Install with: sudo apt install tlp tlp-rdw"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure TLP"
        return
    fi

    # Create optimized TLP configuration
    cat > /etc/tlp.conf << 'EOF'
# Winux OS - TLP Configuration
# Optimized for maximum battery life

#-------------------------------------------------------------------------------
# Operation Mode
#-------------------------------------------------------------------------------
TLP_ENABLE=1
TLP_DEFAULT_MODE=BAT
TLP_PERSISTENT_DEFAULT=0

#-------------------------------------------------------------------------------
# CPU Settings
#-------------------------------------------------------------------------------
CPU_SCALING_GOVERNOR_ON_AC=performance
CPU_SCALING_GOVERNOR_ON_BAT=powersave

CPU_ENERGY_PERF_POLICY_ON_AC=balance_performance
CPU_ENERGY_PERF_POLICY_ON_BAT=power

CPU_MIN_PERF_ON_AC=0
CPU_MAX_PERF_ON_AC=100
CPU_MIN_PERF_ON_BAT=0
CPU_MAX_PERF_ON_BAT=50

CPU_BOOST_ON_AC=1
CPU_BOOST_ON_BAT=0

CPU_HWP_DYN_BOOST_ON_AC=1
CPU_HWP_DYN_BOOST_ON_BAT=0

#-------------------------------------------------------------------------------
# Platform Profile
#-------------------------------------------------------------------------------
PLATFORM_PROFILE_ON_AC=balanced
PLATFORM_PROFILE_ON_BAT=low-power

#-------------------------------------------------------------------------------
# SATA Power Management
#-------------------------------------------------------------------------------
SATA_LINKPWR_ON_AC="med_power_with_dipm max_performance"
SATA_LINKPWR_ON_BAT="min_power med_power_with_dipm"

AHCI_RUNTIME_PM_ON_AC=on
AHCI_RUNTIME_PM_ON_BAT=auto

#-------------------------------------------------------------------------------
# PCIe Power Management
#-------------------------------------------------------------------------------
PCIE_ASPM_ON_AC=default
PCIE_ASPM_ON_BAT=powersupersave

RUNTIME_PM_ON_AC=on
RUNTIME_PM_ON_BAT=auto

#-------------------------------------------------------------------------------
# Graphics
#-------------------------------------------------------------------------------
INTEL_GPU_MIN_FREQ_ON_AC=0
INTEL_GPU_MIN_FREQ_ON_BAT=0
INTEL_GPU_MAX_FREQ_ON_AC=0
INTEL_GPU_MAX_FREQ_ON_BAT=800
INTEL_GPU_BOOST_FREQ_ON_AC=0
INTEL_GPU_BOOST_FREQ_ON_BAT=0

RADEON_DPM_STATE_ON_AC=performance
RADEON_DPM_STATE_ON_BAT=battery
RADEON_DPM_PERF_LEVEL_ON_AC=auto
RADEON_DPM_PERF_LEVEL_ON_BAT=low

#-------------------------------------------------------------------------------
# WiFi Power Management
#-------------------------------------------------------------------------------
WIFI_PWR_ON_AC=off
WIFI_PWR_ON_BAT=on

#-------------------------------------------------------------------------------
# Audio Power Management
#-------------------------------------------------------------------------------
SOUND_POWER_SAVE_ON_AC=0
SOUND_POWER_SAVE_ON_BAT=1
SOUND_POWER_SAVE_CONTROLLER=Y

#-------------------------------------------------------------------------------
# USB Settings
#-------------------------------------------------------------------------------
USB_AUTOSUSPEND=1
USB_EXCLUDE_AUDIO=1
USB_EXCLUDE_BTUSB=0
USB_EXCLUDE_PHONE=1
USB_EXCLUDE_PRINTER=1
USB_EXCLUDE_WWAN=0

#-------------------------------------------------------------------------------
# Battery Care
#-------------------------------------------------------------------------------
# Charge thresholds (if supported)
START_CHARGE_THRESH_BAT0=75
STOP_CHARGE_THRESH_BAT0=80

# ThinkPad specific
TPACPI_ENABLE=1
TPSMAPI_ENABLE=1

#-------------------------------------------------------------------------------
# Disk Settings
#-------------------------------------------------------------------------------
DISK_DEVICES="sda sdb nvme0n1"
DISK_APM_LEVEL_ON_AC="254 254"
DISK_APM_LEVEL_ON_BAT="128 128"
DISK_SPINDOWN_TIMEOUT_ON_AC="0 0"
DISK_SPINDOWN_TIMEOUT_ON_BAT="0 0"
DISK_IOSCHED="mq-deadline mq-deadline"

#-------------------------------------------------------------------------------
# Network Devices
#-------------------------------------------------------------------------------
DEVICES_TO_DISABLE_ON_STARTUP=""
DEVICES_TO_ENABLE_ON_STARTUP=""
DEVICES_TO_DISABLE_ON_SHUTDOWN="bluetooth wifi wwan"
DEVICES_TO_ENABLE_ON_SHUTDOWN=""
DEVICES_TO_DISABLE_ON_BAT_NOT_IN_USE="bluetooth"
DEVICES_TO_ENABLE_ON_AC=""

#-------------------------------------------------------------------------------
# Restore on LAN
#-------------------------------------------------------------------------------
RESTORE_DEVICE_STATE_ON_STARTUP=0
EOF

    # Enable and start TLP
    systemctl enable tlp 2>/dev/null || true
    systemctl start tlp 2>/dev/null || true

    # Apply configuration
    tlp start 2>/dev/null || true

    log_success "TLP configured and started"
}

#-------------------------------------------------------------------------------
# PowerTop Auto-Tune
#-------------------------------------------------------------------------------
run_powertop() {
    log_step "Running PowerTop auto-tune..."

    if ! command -v powertop &>/dev/null; then
        log_warn "PowerTop not installed. Install with: sudo apt install powertop"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would run PowerTop auto-tune"
        return
    fi

    # Run auto-tune
    powertop --auto-tune 2>/dev/null || true

    # Create systemd service for persistence
    cat > /etc/systemd/system/winux-powertop.service << 'EOF'
[Unit]
Description=Winux PowerTop Auto-Tune
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/usr/sbin/powertop --auto-tune
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable winux-powertop.service 2>/dev/null || true

    log_success "PowerTop auto-tune applied"
}

#-------------------------------------------------------------------------------
# Display Brightness Control
#-------------------------------------------------------------------------------
control_brightness() {
    log_step "Configuring display brightness..."

    local backlight_path=""
    for path in /sys/class/backlight/*/; do
        if [ -d "$path" ]; then
            backlight_path="$path"
            break
        fi
    done

    if [ -z "$backlight_path" ]; then
        log_warn "No backlight control found"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure brightness"
        return
    fi

    local max_brightness=$(cat "${backlight_path}max_brightness")
    local target_brightness=$((max_brightness * 50 / 100))  # 50% brightness

    # Save current brightness
    cat "${backlight_path}brightness" > "$BACKUP_DIR/brightness.txt"

    # Set new brightness
    echo "$target_brightness" > "${backlight_path}brightness"

    log_success "Brightness set to 50% ($(cat ${backlight_path}brightness)/${max_brightness})"

    # Create brightness control script
    cat > /usr/local/bin/winux-brightness << 'BSCRIPT'
#!/bin/bash
# Winux Brightness Control

BACKLIGHT=$(ls -d /sys/class/backlight/*/ 2>/dev/null | head -1)
if [ -z "$BACKLIGHT" ]; then
    echo "No backlight found"
    exit 1
fi

MAX=$(cat "${BACKLIGHT}max_brightness")
CURRENT=$(cat "${BACKLIGHT}brightness")

case "$1" in
    up)
        NEW=$((CURRENT + MAX/20))
        [ $NEW -gt $MAX ] && NEW=$MAX
        ;;
    down)
        NEW=$((CURRENT - MAX/20))
        [ $NEW -lt 0 ] && NEW=0
        ;;
    set)
        NEW=$((MAX * $2 / 100))
        ;;
    *)
        echo "Usage: $0 {up|down|set PERCENT}"
        exit 1
        ;;
esac

echo $NEW > "${BACKLIGHT}brightness"
echo "Brightness: $((NEW * 100 / MAX))%"
BSCRIPT
    chmod +x /usr/local/bin/winux-brightness
}

#-------------------------------------------------------------------------------
# CPU Frequency Scaling
#-------------------------------------------------------------------------------
configure_cpu_scaling() {
    log_step "Configuring CPU frequency scaling..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would set CPU governor to powersave"
        return
    fi

    # Set powersave governor
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            echo "powersave" > "$cpu" 2>/dev/null || true
        fi
    done

    # Disable turbo boost
    if [ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]; then
        echo 1 > /sys/devices/system/cpu/intel_pstate/no_turbo
        log_success "Intel Turbo Boost disabled"
    fi

    if [ -f /sys/devices/system/cpu/cpufreq/boost ]; then
        echo 0 > /sys/devices/system/cpu/cpufreq/boost
        log_success "CPU Boost disabled"
    fi

    # Set energy performance preference
    for epp in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do
        if [ -f "$epp" ]; then
            echo "power" > "$epp" 2>/dev/null || true
        fi
    done

    log_success "CPU set to powersave mode"
}

#-------------------------------------------------------------------------------
# USB Autosuspend
#-------------------------------------------------------------------------------
configure_usb_autosuspend() {
    log_step "Configuring USB autosuspend..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would enable USB autosuspend"
        return
    fi

    # Enable autosuspend for all USB devices
    for device in /sys/bus/usb/devices/*/power/autosuspend; do
        if [ -f "$device" ]; then
            echo 1 > "$device" 2>/dev/null || true
        fi
    done

    for device in /sys/bus/usb/devices/*/power/control; do
        if [ -f "$device" ]; then
            echo "auto" > "$device" 2>/dev/null || true
        fi
    done

    # Create udev rule for persistence
    cat > /etc/udev/rules.d/60-winux-usb-power.rules << 'EOF'
# Winux USB Power Management
# Enable autosuspend for USB devices

ACTION=="add", SUBSYSTEM=="usb", ATTR{power/autosuspend}="1"
ACTION=="add", SUBSYSTEM=="usb", ATTR{power/control}="auto"

# Exceptions for input devices (optional)
# ACTION=="add", SUBSYSTEM=="usb", ATTR{bInterfaceClass}=="03", ATTR{power/control}="on"
EOF

    udevadm control --reload-rules 2>/dev/null || true

    log_success "USB autosuspend enabled"
}

#-------------------------------------------------------------------------------
# WiFi Power Save
#-------------------------------------------------------------------------------
configure_wifi_powersave() {
    log_step "Configuring WiFi power save..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would enable WiFi power save"
        return
    fi

    # Enable power save via iw
    for iface in /sys/class/net/wl*; do
        if [ -d "$iface" ]; then
            local name=$(basename "$iface")
            iw dev "$name" set power_save on 2>/dev/null || true
            log_success "WiFi power save enabled on $name"
        fi
    done

    # Create NetworkManager configuration
    mkdir -p /etc/NetworkManager/conf.d
    cat > /etc/NetworkManager/conf.d/wifi-powersave.conf << 'EOF'
[connection]
wifi.powersave = 3
EOF

    # Reload NetworkManager
    systemctl reload NetworkManager 2>/dev/null || true

    log_success "WiFi power save configured"
}

#-------------------------------------------------------------------------------
# Additional Power Optimizations
#-------------------------------------------------------------------------------
apply_power_tweaks() {
    log_step "Applying additional power optimizations..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would apply power tweaks"
        return
    fi

    # Enable laptop mode
    if [ -f /proc/sys/vm/laptop_mode ]; then
        echo 5 > /proc/sys/vm/laptop_mode
    fi

    # Reduce disk I/O
    echo 1500 > /proc/sys/vm/dirty_writeback_centisecs 2>/dev/null || true

    # Enable audio codec power save
    echo 1 > /sys/module/snd_hda_intel/parameters/power_save 2>/dev/null || true
    echo "Y" > /sys/module/snd_hda_intel/parameters/power_save_controller 2>/dev/null || true

    # Disable NMI watchdog
    echo 0 > /proc/sys/kernel/nmi_watchdog 2>/dev/null || true

    # PCI runtime power management
    for dev in /sys/bus/pci/devices/*/power/control; do
        echo "auto" > "$dev" 2>/dev/null || true
    done

    # SATA link power management
    for sata in /sys/class/scsi_host/*/link_power_management_policy; do
        echo "med_power_with_dipm" > "$sata" 2>/dev/null || true
    done

    log_success "Power tweaks applied"
}

#-------------------------------------------------------------------------------
# Show Battery Status
#-------------------------------------------------------------------------------
show_status() {
    echo -e "${WHITE}=== Battery Status ===${NC}"
    echo ""

    # Battery info
    for bat in /sys/class/power_supply/BAT*; do
        if [ -d "$bat" ]; then
            local name=$(basename "$bat")
            local status=$(cat "$bat/status" 2>/dev/null || echo "Unknown")
            local capacity=$(cat "$bat/capacity" 2>/dev/null || echo "Unknown")
            local energy_now=$(cat "$bat/energy_now" 2>/dev/null || cat "$bat/charge_now" 2>/dev/null || echo "0")
            local energy_full=$(cat "$bat/energy_full" 2>/dev/null || cat "$bat/charge_full" 2>/dev/null || echo "0")

            echo -e "${CYAN}$name:${NC}"
            echo "  Status: $status"
            echo "  Capacity: ${capacity}%"

            # Calculate remaining time (rough estimate)
            if [ "$status" = "Discharging" ] && [ -f "$bat/power_now" ]; then
                local power_now=$(cat "$bat/power_now")
                if [ "$power_now" -gt 0 ]; then
                    local hours=$((energy_now / power_now))
                    local mins=$(((energy_now * 60 / power_now) % 60))
                    echo "  Estimated time: ${hours}h ${mins}m"
                fi
            fi
            echo ""
        fi
    done

    # Power source
    local ac_online=0
    for ac in /sys/class/power_supply/AC* /sys/class/power_supply/ADP*; do
        if [ -f "$ac/online" ]; then
            ac_online=$(cat "$ac/online")
            break
        fi
    done

    if [ "$ac_online" = "1" ]; then
        echo -e "${GREEN}Power Source: AC (plugged in)${NC}"
    else
        echo -e "${YELLOW}Power Source: Battery${NC}"
    fi
    echo ""

    # CPU Governor
    echo -e "${CYAN}CPU Governor:${NC}"
    for cpu in /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            echo "  $(cat "$cpu")"
            break
        fi
    done
    echo ""

    # TLP Status
    if command -v tlp-stat &>/dev/null; then
        echo -e "${CYAN}TLP Status:${NC}"
        tlp-stat -s 2>/dev/null | head -5 | sed 's/^/  /'
    fi
    echo ""

    # WiFi Power Save
    echo -e "${CYAN}WiFi Power Save:${NC}"
    for iface in /sys/class/net/wl*; do
        if [ -d "$iface" ]; then
            local name=$(basename "$iface")
            local ps=$(iw dev "$name" get power_save 2>/dev/null | awk '{print $NF}')
            echo "  $name: $ps"
        fi
    done
}

#-------------------------------------------------------------------------------
# Enable Power Save Mode
#-------------------------------------------------------------------------------
enable_powersave() {
    local total_steps=7
    local current_step=0

    echo -e "${GREEN}=== Enabling Power Save Mode ===${NC}"
    echo ""

    ((current_step++))
    show_progress $current_step $total_steps
    create_backup

    ((current_step++))
    show_progress $current_step $total_steps
    configure_tlp

    ((current_step++))
    show_progress $current_step $total_steps
    run_powertop

    ((current_step++))
    show_progress $current_step $total_steps
    control_brightness

    ((current_step++))
    show_progress $current_step $total_steps
    configure_cpu_scaling

    ((current_step++))
    show_progress $current_step $total_steps
    configure_usb_autosuspend

    ((current_step++))
    show_progress $current_step $total_steps
    configure_wifi_powersave

    apply_power_tweaks

    # Mark power save as active
    if ! $DRY_RUN; then
        mkdir -p /var/lib/winux
        echo "$(date)" > /var/lib/winux/.powersave-active
    fi

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}    Power Save Mode ENABLED!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "Optimizations applied:"
    echo "  - TLP power management configured"
    echo "  - PowerTop auto-tune applied"
    echo "  - Display brightness reduced to 50%"
    echo "  - CPU set to powersave mode"
    echo "  - USB autosuspend enabled"
    echo "  - WiFi power save enabled"
    echo ""
    echo "Backup saved to: $BACKUP_DIR"
    echo "To restore: $SCRIPT_NAME --restore"
}

#-------------------------------------------------------------------------------
# Disable Power Save Mode
#-------------------------------------------------------------------------------
disable_powersave() {
    log_step "Disabling power save mode..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would disable power save mode"
        return
    fi

    # Set balanced CPU governor
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            echo "schedutil" > "$cpu" 2>/dev/null || \
            echo "ondemand" > "$cpu" 2>/dev/null || true
        fi
    done

    # Enable turbo boost
    if [ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]; then
        echo 0 > /sys/devices/system/cpu/intel_pstate/no_turbo
    fi

    if [ -f /sys/devices/system/cpu/cpufreq/boost ]; then
        echo 1 > /sys/devices/system/cpu/cpufreq/boost
    fi

    # Disable WiFi power save
    for iface in /sys/class/net/wl*; do
        if [ -d "$iface" ]; then
            local name=$(basename "$iface")
            iw dev "$name" set power_save off 2>/dev/null || true
        fi
    done

    # Remove marker
    rm -f /var/lib/winux/.powersave-active

    echo -e "${GREEN}Power save mode disabled${NC}"
}

#-------------------------------------------------------------------------------
# Auto Mode (switch based on power source)
#-------------------------------------------------------------------------------
enable_auto_mode() {
    log_step "Enabling auto power mode..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would enable auto mode"
        return
    fi

    # Create udev rules for power source changes
    cat > /etc/udev/rules.d/61-winux-power-auto.rules << 'EOF'
# Winux Auto Power Mode
# Switch power profile based on power source

# On AC power
SUBSYSTEM=="power_supply", ATTR{online}=="1", RUN+="/etc/winux/scripts/optimize-battery.sh --disable"

# On battery
SUBSYSTEM=="power_supply", ATTR{online}=="0", RUN+="/etc/winux/scripts/optimize-battery.sh --enable"
EOF

    udevadm control --reload-rules 2>/dev/null || true

    log_success "Auto power mode enabled"
    echo "System will automatically switch between power save and balanced mode"
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
            -a|--auto)
                mode="auto"
                shift
                ;;
            -s|--status)
                show_logo
                show_status
                exit 0
                ;;
            -r|--restore)
                check_root
                show_logo
                restore_backup
                exit 0
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--version)
                echo "Winux Battery Optimizer v$VERSION"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # Check root unless dry-run or status
    if ! $DRY_RUN; then
        check_root
    fi

    # Create directories
    mkdir -p /var/log/winux 2>/dev/null || true
    mkdir -p /var/lib/winux/backups 2>/dev/null || true

    # Show logo
    show_logo

    if $DRY_RUN; then
        echo -e "${YELLOW}=== DRY RUN MODE - No changes will be made ===${NC}"
        echo ""
    fi

    # Check for battery (unless disabling)
    if [ "$mode" != "disable" ]; then
        check_battery
    fi

    # Execute based on mode
    case $mode in
        enable)
            enable_powersave
            ;;
        disable)
            disable_powersave
            ;;
        auto)
            enable_auto_mode
            ;;
    esac
}

main "$@"
