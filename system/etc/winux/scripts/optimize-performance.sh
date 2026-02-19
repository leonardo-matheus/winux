#!/bin/bash
#===============================================================================
# Winux OS - System Performance Optimizer
# Advanced system tuning for maximum performance
#===============================================================================

set -e

#-------------------------------------------------------------------------------
# Configuration
#-------------------------------------------------------------------------------
VERSION="1.0.0"
SCRIPT_NAME="$(basename "$0")"
BACKUP_DIR="/var/lib/winux/backups/performance-$(date +%Y%m%d_%H%M%S)"
LOG_FILE="/var/log/winux/optimize-performance.log"
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
    echo -e "${CYAN}"
    cat << 'EOF'
 __        __ _
 \ \      / /(_) _ __  _   _ __ __
  \ \ /\ / / | || '_ \| | | |\ \/ /
   \ V  V /  | || | | | |_| | >  <
    \_/\_/   |_||_| |_|\__,_|/_/\_\

    Performance Optimizer v1.0.0
    ============================
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
    echo -e "${MAGENTA}[STEP]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [STEP] $1" >> "$LOG_FILE" 2>/dev/null || true
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

    printf "\r${CYAN}["
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
    echo "Advanced system performance optimizer for Winux OS"
    echo ""
    echo "Options:"
    echo "  -d, --dry-run     Show what would be done without making changes"
    echo "  -r, --restore     Restore from backup"
    echo "  -s, --status      Show current performance settings"
    echo "  -h, --help        Show this help message"
    echo "  -v, --version     Show version"
    echo ""
    echo "Features:"
    echo "  - CPU governor optimization"
    echo "  - IO scheduler tuning (mq-deadline for SSD, bfq for HDD)"
    echo "  - Swappiness adjustment"
    echo "  - Transparent Huge Pages configuration"
    echo "  - ZRAM setup"
    echo "  - IRQ balancing"
    echo "  - CPU isolation for gaming"
    echo ""
    echo "Examples:"
    echo "  $SCRIPT_NAME                  # Apply all optimizations"
    echo "  $SCRIPT_NAME --dry-run        # Preview changes"
    echo "  $SCRIPT_NAME --restore        # Restore previous settings"
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
    log_step "Creating backup of current settings..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would create backup in $BACKUP_DIR"
        return
    fi

    mkdir -p "$BACKUP_DIR"

    # Backup sysctl settings
    if [ -f /etc/sysctl.conf ]; then
        cp /etc/sysctl.conf "$BACKUP_DIR/"
    fi

    # Backup CPU governor
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            cat "$cpu" > "$BACKUP_DIR/cpu_governors.txt"
            break
        fi
    done

    # Backup IO schedulers
    for disk in /sys/block/*/queue/scheduler; do
        if [ -f "$disk" ]; then
            disk_name=$(echo "$disk" | cut -d'/' -f4)
            cat "$disk" > "$BACKUP_DIR/io_scheduler_${disk_name}.txt"
        fi
    done

    # Backup swappiness
    cat /proc/sys/vm/swappiness > "$BACKUP_DIR/swappiness.txt"

    # Backup THP settings
    if [ -f /sys/kernel/mm/transparent_hugepage/enabled ]; then
        cat /sys/kernel/mm/transparent_hugepage/enabled > "$BACKUP_DIR/thp_enabled.txt"
    fi

    # Save backup metadata
    cat > "$BACKUP_DIR/metadata.txt" << EOF
Backup created: $(date)
Winux Performance Optimizer v$VERSION
EOF

    log_success "Backup created at $BACKUP_DIR"
}

#-------------------------------------------------------------------------------
# Restore from Backup
#-------------------------------------------------------------------------------
restore_backup() {
    log_step "Restoring from backup..."

    # Find latest backup
    local latest_backup=$(ls -td /var/lib/winux/backups/performance-* 2>/dev/null | head -1)

    if [ -z "$latest_backup" ]; then
        log_error "No backup found to restore"
        exit 1
    fi

    log_info "Restoring from: $latest_backup"

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would restore from $latest_backup"
        return
    fi

    # Restore sysctl
    if [ -f "$latest_backup/sysctl.conf" ]; then
        cp "$latest_backup/sysctl.conf" /etc/sysctl.conf
        sysctl -p
        log_success "Restored sysctl settings"
    fi

    # Restore swappiness
    if [ -f "$latest_backup/swappiness.txt" ]; then
        local swap_val=$(cat "$latest_backup/swappiness.txt")
        echo "$swap_val" > /proc/sys/vm/swappiness
        log_success "Restored swappiness to $swap_val"
    fi

    log_success "Restore complete"
}

#-------------------------------------------------------------------------------
# Show Current Status
#-------------------------------------------------------------------------------
show_status() {
    echo -e "${WHITE}=== Current Performance Settings ===${NC}"
    echo ""

    # CPU Governor
    echo -e "${CYAN}CPU Governor:${NC}"
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            local cpu_num=$(echo "$cpu" | grep -oP 'cpu\d+')
            echo "  $cpu_num: $(cat "$cpu")"
        fi
    done
    echo ""

    # IO Scheduler
    echo -e "${CYAN}IO Schedulers:${NC}"
    for disk in /sys/block/*/queue/scheduler; do
        if [ -f "$disk" ]; then
            local disk_name=$(echo "$disk" | cut -d'/' -f4)
            echo "  $disk_name: $(cat "$disk")"
        fi
    done
    echo ""

    # Swappiness
    echo -e "${CYAN}Swappiness:${NC} $(cat /proc/sys/vm/swappiness)"
    echo ""

    # THP
    if [ -f /sys/kernel/mm/transparent_hugepage/enabled ]; then
        echo -e "${CYAN}Transparent Huge Pages:${NC} $(cat /sys/kernel/mm/transparent_hugepage/enabled)"
    fi
    echo ""

    # ZRAM
    echo -e "${CYAN}ZRAM Status:${NC}"
    if lsmod | grep -q zram; then
        zramctl 2>/dev/null || echo "  ZRAM loaded but not configured"
    else
        echo "  Not loaded"
    fi
    echo ""

    # Memory
    echo -e "${CYAN}Memory Info:${NC}"
    free -h
}

#-------------------------------------------------------------------------------
# Optimize CPU Governor
#-------------------------------------------------------------------------------
optimize_cpu_governor() {
    log_step "Optimizing CPU governor..."

    local governor="performance"

    # Check if performance governor is available
    if [ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors ]; then
        if ! grep -q "performance" /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors; then
            log_warn "Performance governor not available, using schedutil"
            governor="schedutil"
        fi
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would set CPU governor to: $governor"
        return
    fi

    # Set governor for all CPUs
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            echo "$governor" > "$cpu" 2>/dev/null || true
        fi
    done

    # Make persistent via sysctl.d
    cat > /etc/sysctl.d/99-winux-cpu.conf << EOF
# Winux Performance - CPU Settings
# Generated by optimize-performance.sh
EOF

    log_success "CPU governor set to $governor"
}

#-------------------------------------------------------------------------------
# Optimize IO Scheduler
#-------------------------------------------------------------------------------
optimize_io_scheduler() {
    log_step "Optimizing IO schedulers..."

    for disk in /sys/block/*/queue/scheduler; do
        if [ -f "$disk" ]; then
            local disk_name=$(echo "$disk" | cut -d'/' -f4)

            # Skip loop devices and ram disks
            [[ "$disk_name" == loop* ]] && continue
            [[ "$disk_name" == ram* ]] && continue

            # Detect if SSD or HDD
            local rotational="/sys/block/$disk_name/queue/rotational"
            local scheduler="mq-deadline"  # Default for SSD

            if [ -f "$rotational" ] && [ "$(cat "$rotational")" = "1" ]; then
                scheduler="bfq"  # Use BFQ for HDD
            fi

            if $DRY_RUN; then
                log_info "[DRY-RUN] Would set $disk_name scheduler to: $scheduler"
                continue
            fi

            # Check if scheduler is available
            if grep -q "$scheduler" "$disk"; then
                echo "$scheduler" > "$disk" 2>/dev/null || true
                log_success "$disk_name: scheduler set to $scheduler"
            else
                log_warn "$disk_name: $scheduler not available"
            fi
        fi
    done

    # Create udev rule for persistence
    if ! $DRY_RUN; then
        cat > /etc/udev/rules.d/60-winux-io-scheduler.rules << 'EOF'
# Winux Performance - IO Scheduler Rules
# SSD: mq-deadline, HDD: bfq

# SSD - use mq-deadline
ACTION=="add|change", KERNEL=="sd[a-z]", ATTR{queue/rotational}=="0", ATTR{queue/scheduler}="mq-deadline"
ACTION=="add|change", KERNEL=="nvme[0-9]*", ATTR{queue/scheduler}="none"

# HDD - use bfq
ACTION=="add|change", KERNEL=="sd[a-z]", ATTR{queue/rotational}=="1", ATTR{queue/scheduler}="bfq"
EOF
        log_success "IO scheduler udev rules created"
    fi
}

#-------------------------------------------------------------------------------
# Optimize Swappiness
#-------------------------------------------------------------------------------
optimize_swappiness() {
    log_step "Optimizing swappiness..."

    local swappiness=10  # Lower value for better performance

    # Adjust based on RAM
    local total_ram=$(awk '/MemTotal/ {print int($2/1024/1024)}' /proc/meminfo)
    if [ "$total_ram" -ge 16 ]; then
        swappiness=1
    elif [ "$total_ram" -ge 8 ]; then
        swappiness=10
    else
        swappiness=30
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would set swappiness to: $swappiness (RAM: ${total_ram}GB)"
        return
    fi

    echo "$swappiness" > /proc/sys/vm/swappiness

    # Make persistent
    if grep -q "vm.swappiness" /etc/sysctl.conf 2>/dev/null; then
        sed -i "s/vm.swappiness.*/vm.swappiness = $swappiness/" /etc/sysctl.conf
    else
        echo "vm.swappiness = $swappiness" >> /etc/sysctl.conf
    fi

    log_success "Swappiness set to $swappiness (RAM: ${total_ram}GB)"
}

#-------------------------------------------------------------------------------
# Optimize Transparent Huge Pages
#-------------------------------------------------------------------------------
optimize_thp() {
    log_step "Configuring Transparent Huge Pages..."

    if [ ! -f /sys/kernel/mm/transparent_hugepage/enabled ]; then
        log_warn "THP not supported on this system"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would set THP to: madvise"
        return
    fi

    # Use madvise for better application control
    echo "madvise" > /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null || true
    echo "madvise" > /sys/kernel/mm/transparent_hugepage/defrag 2>/dev/null || true

    # Create systemd service for persistence
    cat > /etc/systemd/system/winux-thp.service << 'EOF'
[Unit]
Description=Winux THP Configuration
After=sysinit.target

[Service]
Type=oneshot
ExecStart=/bin/sh -c 'echo madvise > /sys/kernel/mm/transparent_hugepage/enabled'
ExecStart=/bin/sh -c 'echo madvise > /sys/kernel/mm/transparent_hugepage/defrag'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable winux-thp.service 2>/dev/null || true

    log_success "THP configured to madvise"
}

#-------------------------------------------------------------------------------
# Configure ZRAM
#-------------------------------------------------------------------------------
configure_zram() {
    log_step "Configuring ZRAM..."

    # Check if ZRAM is available
    if ! modprobe -n zram 2>/dev/null; then
        log_warn "ZRAM not available on this system"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure ZRAM with lz4 compression"
        return
    fi

    # Load ZRAM module
    modprobe zram num_devices=1

    # Calculate ZRAM size (50% of RAM, max 8GB)
    local total_ram=$(awk '/MemTotal/ {print int($2/1024)}' /proc/meminfo)
    local zram_size=$((total_ram / 2))
    [ "$zram_size" -gt 8192 ] && zram_size=8192

    # Configure ZRAM device
    if [ -b /dev/zram0 ]; then
        # Reset if already configured
        swapoff /dev/zram0 2>/dev/null || true
        echo 1 > /sys/block/zram0/reset 2>/dev/null || true

        # Set compression algorithm and size
        echo lz4 > /sys/block/zram0/comp_algorithm 2>/dev/null || \
        echo lzo > /sys/block/zram0/comp_algorithm
        echo "${zram_size}M" > /sys/block/zram0/disksize

        # Create and enable swap
        mkswap /dev/zram0
        swapon -p 100 /dev/zram0

        log_success "ZRAM configured: ${zram_size}MB with lz4/lzo compression"
    fi

    # Create systemd service for persistence
    cat > /etc/systemd/system/winux-zram.service << EOF
[Unit]
Description=Winux ZRAM Configuration
After=local-fs.target

[Service]
Type=oneshot
ExecStart=/bin/sh -c 'modprobe zram num_devices=1'
ExecStart=/bin/sh -c 'echo lz4 > /sys/block/zram0/comp_algorithm 2>/dev/null || echo lzo > /sys/block/zram0/comp_algorithm'
ExecStart=/bin/sh -c 'echo ${zram_size}M > /sys/block/zram0/disksize'
ExecStart=/bin/sh -c 'mkswap /dev/zram0 && swapon -p 100 /dev/zram0'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable winux-zram.service 2>/dev/null || true
}

#-------------------------------------------------------------------------------
# Optimize IRQ Balancing
#-------------------------------------------------------------------------------
optimize_irq_balance() {
    log_step "Optimizing IRQ balancing..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure IRQ balancing"
        return
    fi

    # Install irqbalance if not present
    if ! command -v irqbalance &>/dev/null; then
        log_warn "irqbalance not installed, skipping"
        return
    fi

    # Configure irqbalance for performance
    if [ -f /etc/default/irqbalance ]; then
        sed -i 's/#IRQBALANCE_ARGS.*/IRQBALANCE_ARGS="--hintpolicy=exact"/' /etc/default/irqbalance
    fi

    # Enable and restart service
    systemctl enable irqbalance 2>/dev/null || true
    systemctl restart irqbalance 2>/dev/null || true

    log_success "IRQ balancing configured"
}

#-------------------------------------------------------------------------------
# CPU Isolation for Gaming
#-------------------------------------------------------------------------------
configure_cpu_isolation() {
    log_step "Configuring CPU isolation for gaming..."

    # Get CPU count
    local cpu_count=$(nproc)

    if [ "$cpu_count" -lt 4 ]; then
        log_warn "CPU isolation requires at least 4 cores, skipping"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure CPU isolation (2 cores reserved for system)"
        return
    fi

    # Create cpuset for gaming
    if [ -d /sys/fs/cgroup/cpuset ]; then
        mkdir -p /sys/fs/cgroup/cpuset/gaming 2>/dev/null || true

        # Reserve CPUs 2 onwards for gaming (leave 0,1 for system)
        local gaming_cpus="2-$((cpu_count - 1))"
        echo "$gaming_cpus" > /sys/fs/cgroup/cpuset/gaming/cpuset.cpus 2>/dev/null || true
        echo 0 > /sys/fs/cgroup/cpuset/gaming/cpuset.mems 2>/dev/null || true

        log_success "CPU isolation configured: gaming CPUs $gaming_cpus"
    else
        log_warn "cpuset cgroup not available"
    fi

    # Create helper script
    cat > /usr/local/bin/winux-game-mode << 'EOF'
#!/bin/bash
# Move process to gaming cpuset
if [ -z "$1" ]; then
    echo "Usage: winux-game-mode <PID>"
    exit 1
fi
echo $1 > /sys/fs/cgroup/cpuset/gaming/tasks 2>/dev/null
echo "Process $1 moved to gaming cpuset"
EOF
    chmod +x /usr/local/bin/winux-game-mode
}

#-------------------------------------------------------------------------------
# Additional Kernel Tuning
#-------------------------------------------------------------------------------
kernel_tuning() {
    log_step "Applying kernel tuning..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would apply kernel performance tuning"
        return
    fi

    # Create sysctl configuration
    cat > /etc/sysctl.d/99-winux-performance.conf << 'EOF'
# Winux Performance Tuning
# Generated by optimize-performance.sh

# Virtual Memory
vm.dirty_ratio = 10
vm.dirty_background_ratio = 5
vm.vfs_cache_pressure = 50

# Network Performance
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_fastopen = 3
net.ipv4.tcp_slow_start_after_idle = 0

# File System
fs.file-max = 2097152
fs.inotify.max_user_watches = 524288
fs.inotify.max_user_instances = 1024

# Kernel
kernel.sched_autogroup_enabled = 1
kernel.sched_child_runs_first = 0
EOF

    # Apply settings
    sysctl -p /etc/sysctl.d/99-winux-performance.conf 2>/dev/null || true

    log_success "Kernel tuning applied"
}

#-------------------------------------------------------------------------------
# Main Function
#-------------------------------------------------------------------------------
main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -d|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -r|--restore)
                check_root
                show_logo
                restore_backup
                exit 0
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
                echo "Winux Performance Optimizer v$VERSION"
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

    # Show logo
    show_logo

    if $DRY_RUN; then
        echo -e "${YELLOW}=== DRY RUN MODE - No changes will be made ===${NC}"
        echo ""
    fi

    # Total steps for progress
    local total_steps=8
    local current_step=0

    # Create backup
    ((current_step++))
    show_progress $current_step $total_steps
    create_backup

    # Optimize CPU governor
    ((current_step++))
    show_progress $current_step $total_steps
    optimize_cpu_governor

    # Optimize IO scheduler
    ((current_step++))
    show_progress $current_step $total_steps
    optimize_io_scheduler

    # Optimize swappiness
    ((current_step++))
    show_progress $current_step $total_steps
    optimize_swappiness

    # Configure THP
    ((current_step++))
    show_progress $current_step $total_steps
    optimize_thp

    # Configure ZRAM
    ((current_step++))
    show_progress $current_step $total_steps
    configure_zram

    # Optimize IRQ balancing
    ((current_step++))
    show_progress $current_step $total_steps
    optimize_irq_balance

    # CPU isolation
    ((current_step++))
    show_progress $current_step $total_steps
    configure_cpu_isolation

    # Kernel tuning
    kernel_tuning

    # Done
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Performance optimization complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "Applied optimizations:"
    echo "  - CPU governor: performance"
    echo "  - IO scheduler: mq-deadline (SSD) / bfq (HDD)"
    echo "  - Swappiness: optimized for RAM size"
    echo "  - THP: madvise mode"
    echo "  - ZRAM: enabled with lz4 compression"
    echo "  - IRQ balancing: enabled"
    echo "  - CPU isolation: configured for gaming"
    echo "  - Kernel: performance tuned"
    echo ""
    echo "Backup saved to: $BACKUP_DIR"
    echo "To restore: $SCRIPT_NAME --restore"
}

main "$@"
