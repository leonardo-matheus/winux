#!/bin/bash
#===============================================================================
# Winux OS - System Cleanup Tool
# Clean temporary files, caches, and free up disk space
#===============================================================================

set -e

#-------------------------------------------------------------------------------
# Configuration
#-------------------------------------------------------------------------------
VERSION="1.0.0"
SCRIPT_NAME="$(basename "$0")"
BACKUP_DIR="/var/lib/winux/backups/cleanup-$(date +%Y%m%d_%H%M%S)"
LOG_FILE="/var/log/winux/system-cleanup.log"
DRY_RUN=false
TOTAL_FREED=0

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
    echo -e "${YELLOW}"
    cat << 'EOF'
 __        __ _
 \ \      / /(_) _ __  _   _ __ __
  \ \ /\ / / | || '_ \| | | |\ \/ /
   \ V  V /  | || | | | |_| | >  <
    \_/\_/   |_||_| |_|\__,_|/_/\_\

      System Cleanup v1.0.0
     =========================
          [CLEAN MODE]
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
    echo -e "${YELLOW}[CLEAN]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [CLEAN] $1" >> "$LOG_FILE" 2>/dev/null || true
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

    printf "\r${YELLOW}["
    printf "%${filled}s" | tr ' ' '#'
    printf "%${empty}s" | tr ' ' '-'
    printf "] ${percentage}%%${NC}"

    if [ "$current" -eq "$total" ]; then
        echo ""
    fi
}

#-------------------------------------------------------------------------------
# Format Size
#-------------------------------------------------------------------------------
format_size() {
    local size=$1
    if [ "$size" -ge 1073741824 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $size/1073741824}")GB"
    elif [ "$size" -ge 1048576 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $size/1048576}")MB"
    elif [ "$size" -ge 1024 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $size/1024}")KB"
    else
        echo "${size}B"
    fi
}

#-------------------------------------------------------------------------------
# Get Directory Size
#-------------------------------------------------------------------------------
get_dir_size() {
    local dir="$1"
    if [ -d "$dir" ]; then
        du -sb "$dir" 2>/dev/null | cut -f1
    else
        echo 0
    fi
}

#-------------------------------------------------------------------------------
# Help
#-------------------------------------------------------------------------------
show_help() {
    echo "Usage: $SCRIPT_NAME [OPTIONS]"
    echo ""
    echo "System cleanup tool for Winux OS"
    echo ""
    echo "Options:"
    echo "  -d, --dry-run       Show what would be deleted without removing"
    echo "  -a, --all           Run all cleanup tasks (default)"
    echo "  -p, --packages      Clean package cache only"
    echo "  -l, --logs          Clean old logs only"
    echo "  -t, --thumbnails    Clean thumbnails only"
    echo "  -b, --browser       Clean browser cache only"
    echo "  -k, --kernels       Remove old kernels only"
    echo "  -s, --status        Show disk usage status"
    echo "  -h, --help          Show this help message"
    echo "  -v, --version       Show version"
    echo ""
    echo "Features:"
    echo "  - Clean package manager cache (apt, flatpak, snap)"
    echo "  - Clean old system logs"
    echo "  - Clean thumbnail cache"
    echo "  - Clean browser cache (Firefox, Chrome, Chromium)"
    echo "  - Remove old kernels"
    echo "  - Clean temporary files"
    echo ""
    echo "Examples:"
    echo "  $SCRIPT_NAME                  # Run all cleanup tasks"
    echo "  $SCRIPT_NAME --dry-run        # Preview what would be deleted"
    echo "  $SCRIPT_NAME --packages       # Clean only package cache"
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
# Create Backup Info
#-------------------------------------------------------------------------------
create_backup_info() {
    log_step "Recording cleanup information..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would create backup info"
        return
    fi

    mkdir -p "$BACKUP_DIR"

    # Record what was cleaned
    cat > "$BACKUP_DIR/cleanup-info.txt" << EOF
Winux System Cleanup
Date: $(date)
Version: $VERSION

This file records what was cleaned.
Note: Deleted files cannot be restored.
EOF

    log_success "Cleanup info will be saved to $BACKUP_DIR"
}

#-------------------------------------------------------------------------------
# Clean Package Cache
#-------------------------------------------------------------------------------
clean_package_cache() {
    log_step "Cleaning package cache..."

    local freed=0

    # APT cache
    if command -v apt-get &>/dev/null; then
        local apt_cache="/var/cache/apt/archives"
        local apt_size=$(get_dir_size "$apt_cache")

        if $DRY_RUN; then
            log_info "[DRY-RUN] Would clean APT cache: $(format_size $apt_size)"
        else
            apt-get clean 2>/dev/null || true
            apt-get autoclean 2>/dev/null || true
            log_success "APT cache cleaned: $(format_size $apt_size)"
            freed=$((freed + apt_size))
        fi
    fi

    # APT lists (optional, can be regenerated)
    local apt_lists="/var/lib/apt/lists"
    if [ -d "$apt_lists" ]; then
        local lists_size=$(get_dir_size "$apt_lists")
        if $DRY_RUN; then
            log_info "[DRY-RUN] APT lists size: $(format_size $lists_size) (not removing)"
        fi
    fi

    # Flatpak cache
    if command -v flatpak &>/dev/null; then
        if $DRY_RUN; then
            log_info "[DRY-RUN] Would run flatpak uninstall --unused"
        else
            flatpak uninstall --unused -y 2>/dev/null || true
            flatpak repair 2>/dev/null || true
            log_success "Flatpak unused runtimes removed"
        fi
    fi

    # Snap cache
    if command -v snap &>/dev/null; then
        local snap_cache="/var/lib/snapd/cache"
        if [ -d "$snap_cache" ]; then
            local snap_size=$(get_dir_size "$snap_cache")

            if $DRY_RUN; then
                log_info "[DRY-RUN] Would clean Snap cache: $(format_size $snap_size)"
            else
                rm -rf "$snap_cache"/* 2>/dev/null || true
                log_success "Snap cache cleaned: $(format_size $snap_size)"
                freed=$((freed + snap_size))
            fi
        fi

        # Remove old snap revisions
        if $DRY_RUN; then
            log_info "[DRY-RUN] Would remove old snap revisions"
        else
            snap list --all | awk '/disabled/{print $1, $3}' | while read snapname revision; do
                snap remove "$snapname" --revision="$revision" 2>/dev/null || true
            done
            log_success "Old snap revisions removed"
        fi
    fi

    # pip cache
    local pip_cache="$HOME/.cache/pip"
    if [ -d "$pip_cache" ]; then
        local pip_size=$(get_dir_size "$pip_cache")
        if $DRY_RUN; then
            log_info "[DRY-RUN] Would clean pip cache: $(format_size $pip_size)"
        else
            rm -rf "$pip_cache" 2>/dev/null || true
            log_success "pip cache cleaned: $(format_size $pip_size)"
            freed=$((freed + pip_size))
        fi
    fi

    # npm cache
    if command -v npm &>/dev/null; then
        if $DRY_RUN; then
            log_info "[DRY-RUN] Would clean npm cache"
        else
            npm cache clean --force 2>/dev/null || true
            log_success "npm cache cleaned"
        fi
    fi

    # yarn cache
    if command -v yarn &>/dev/null; then
        if $DRY_RUN; then
            log_info "[DRY-RUN] Would clean yarn cache"
        else
            yarn cache clean 2>/dev/null || true
            log_success "yarn cache cleaned"
        fi
    fi

    TOTAL_FREED=$((TOTAL_FREED + freed))
    echo "$freed" >> "$BACKUP_DIR/package-cache-freed.txt" 2>/dev/null || true
}

#-------------------------------------------------------------------------------
# Clean Old Logs
#-------------------------------------------------------------------------------
clean_old_logs() {
    log_step "Cleaning old logs..."

    local freed=0

    # Journalctl logs
    if command -v journalctl &>/dev/null; then
        local journal_size=$(journalctl --disk-usage 2>/dev/null | grep -oP '\d+\.?\d*[KMGT]' | head -1)

        if $DRY_RUN; then
            log_info "[DRY-RUN] Would vacuum journal logs (current: $journal_size)"
        else
            journalctl --vacuum-time=7d 2>/dev/null || true
            journalctl --vacuum-size=100M 2>/dev/null || true
            log_success "Journal logs vacuumed (kept last 7 days, max 100MB)"
        fi
    fi

    # Old log files in /var/log
    local log_dirs=("/var/log/*.gz" "/var/log/*.old" "/var/log/*/*.gz" "/var/log/*/*.old")

    for pattern in "${log_dirs[@]}"; do
        for logfile in $pattern; do
            if [ -f "$logfile" ]; then
                local file_size=$(stat -c%s "$logfile" 2>/dev/null || echo 0)
                freed=$((freed + file_size))

                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would remove: $logfile ($(format_size $file_size))"
                else
                    rm -f "$logfile" 2>/dev/null || true
                fi
            fi
        done
    done

    # Rotated logs older than 7 days
    if $DRY_RUN; then
        log_info "[DRY-RUN] Would remove logs older than 7 days"
    else
        find /var/log -type f -name "*.log.*" -mtime +7 -delete 2>/dev/null || true
        find /var/log -type f -name "*.[0-9]" -mtime +7 -delete 2>/dev/null || true
    fi

    # Crash reports
    local crash_dir="/var/crash"
    if [ -d "$crash_dir" ]; then
        local crash_size=$(get_dir_size "$crash_dir")
        if $DRY_RUN; then
            log_info "[DRY-RUN] Would clean crash reports: $(format_size $crash_size)"
        else
            rm -rf "$crash_dir"/* 2>/dev/null || true
            log_success "Crash reports cleaned: $(format_size $crash_size)"
            freed=$((freed + crash_size))
        fi
    fi

    log_success "Old logs cleaned: $(format_size $freed)"
    TOTAL_FREED=$((TOTAL_FREED + freed))
}

#-------------------------------------------------------------------------------
# Clean Thumbnails
#-------------------------------------------------------------------------------
clean_thumbnails() {
    log_step "Cleaning thumbnail cache..."

    local freed=0

    # Find all user home directories
    for user_home in /home/*; do
        if [ -d "$user_home" ]; then
            local user=$(basename "$user_home")

            # Standard thumbnail cache
            local thumb_dir="$user_home/.cache/thumbnails"
            if [ -d "$thumb_dir" ]; then
                local thumb_size=$(get_dir_size "$thumb_dir")

                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would clean thumbnails for $user: $(format_size $thumb_size)"
                else
                    rm -rf "$thumb_dir"/* 2>/dev/null || true
                    freed=$((freed + thumb_size))
                    log_success "Thumbnails cleaned for $user: $(format_size $thumb_size)"
                fi
            fi

            # KDE thumbnail cache
            local kde_thumb="$user_home/.cache/kio_http"
            if [ -d "$kde_thumb" ]; then
                local kde_size=$(get_dir_size "$kde_thumb")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would clean KDE cache for $user: $(format_size $kde_size)"
                else
                    rm -rf "$kde_thumb"/* 2>/dev/null || true
                    freed=$((freed + kde_size))
                fi
            fi

            # GNOME thumbnail cache
            local gnome_thumb="$user_home/.cache/gnome-software"
            if [ -d "$gnome_thumb" ]; then
                local gnome_size=$(get_dir_size "$gnome_thumb")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would clean GNOME cache for $user: $(format_size $gnome_size)"
                else
                    rm -rf "$gnome_thumb"/* 2>/dev/null || true
                    freed=$((freed + gnome_size))
                fi
            fi
        fi
    done

    log_success "Thumbnail cache cleaned: $(format_size $freed)"
    TOTAL_FREED=$((TOTAL_FREED + freed))
}

#-------------------------------------------------------------------------------
# Clean Browser Cache
#-------------------------------------------------------------------------------
clean_browser_cache() {
    log_step "Cleaning browser cache..."

    local freed=0

    for user_home in /home/*; do
        if [ -d "$user_home" ]; then
            local user=$(basename "$user_home")

            # Firefox cache
            local firefox_cache="$user_home/.cache/mozilla/firefox"
            if [ -d "$firefox_cache" ]; then
                local ff_size=$(get_dir_size "$firefox_cache")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would clean Firefox cache for $user: $(format_size $ff_size)"
                else
                    find "$firefox_cache" -type f -name "*.sqlite" -exec sqlite3 {} "VACUUM;" 2>/dev/null \; || true
                    find "$firefox_cache" -type d -name "cache2" -exec rm -rf {}/* 2>/dev/null \; || true
                    freed=$((freed + ff_size / 2))  # Estimate
                    log_success "Firefox cache cleaned for $user"
                fi
            fi

            # Chrome/Chromium cache
            for chrome_dir in "$user_home/.cache/google-chrome" "$user_home/.cache/chromium"; do
                if [ -d "$chrome_dir" ]; then
                    local chrome_size=$(get_dir_size "$chrome_dir")
                    if $DRY_RUN; then
                        log_info "[DRY-RUN] Would clean Chrome cache for $user: $(format_size $chrome_size)"
                    else
                        rm -rf "$chrome_dir/Default/Cache"/* 2>/dev/null || true
                        rm -rf "$chrome_dir/Default/Code Cache"/* 2>/dev/null || true
                        rm -rf "$chrome_dir/Default/GPUCache"/* 2>/dev/null || true
                        freed=$((freed + chrome_size / 2))  # Estimate
                        log_success "Chrome/Chromium cache cleaned for $user"
                    fi
                fi
            done

            # Brave cache
            local brave_cache="$user_home/.cache/BraveSoftware"
            if [ -d "$brave_cache" ]; then
                local brave_size=$(get_dir_size "$brave_cache")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would clean Brave cache for $user: $(format_size $brave_size)"
                else
                    rm -rf "$brave_cache"/*/Default/Cache/* 2>/dev/null || true
                    rm -rf "$brave_cache"/*/Default/Code\ Cache/* 2>/dev/null || true
                    freed=$((freed + brave_size / 2))
                    log_success "Brave cache cleaned for $user"
                fi
            fi

            # Edge cache
            local edge_cache="$user_home/.cache/microsoft-edge"
            if [ -d "$edge_cache" ]; then
                local edge_size=$(get_dir_size "$edge_cache")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would clean Edge cache for $user: $(format_size $edge_size)"
                else
                    rm -rf "$edge_cache/Default/Cache"/* 2>/dev/null || true
                    rm -rf "$edge_cache/Default/Code Cache"/* 2>/dev/null || true
                    freed=$((freed + edge_size / 2))
                    log_success "Edge cache cleaned for $user"
                fi
            fi
        fi
    done

    log_success "Browser cache cleaned: $(format_size $freed)"
    TOTAL_FREED=$((TOTAL_FREED + freed))
}

#-------------------------------------------------------------------------------
# Remove Old Kernels
#-------------------------------------------------------------------------------
remove_old_kernels() {
    log_step "Removing old kernels..."

    if ! command -v dpkg &>/dev/null; then
        log_warn "dpkg not available, skipping kernel cleanup"
        return
    fi

    # Get current kernel
    local current_kernel=$(uname -r)
    log_info "Current kernel: $current_kernel"

    # Get list of installed kernels
    local installed_kernels=$(dpkg -l 'linux-image-*' 2>/dev/null | grep '^ii' | awk '{print $2}' | grep -v "$current_kernel" | grep -v "linux-image-generic" | head -n -1)

    if [ -z "$installed_kernels" ]; then
        log_info "No old kernels to remove"
        return
    fi

    local freed=0

    for kernel in $installed_kernels; do
        local kernel_size=$(dpkg-query -W -f='${Installed-Size}' "$kernel" 2>/dev/null || echo 0)
        kernel_size=$((kernel_size * 1024))  # Convert to bytes

        if $DRY_RUN; then
            log_info "[DRY-RUN] Would remove: $kernel ($(format_size $kernel_size))"
        else
            log_info "Removing: $kernel"
            apt-get purge -y "$kernel" 2>/dev/null || true
            freed=$((freed + kernel_size))
        fi
    done

    # Also remove old kernel headers
    local old_headers=$(dpkg -l 'linux-headers-*' 2>/dev/null | grep '^ii' | awk '{print $2}' | grep -v "$current_kernel" | grep -v "linux-headers-generic" | head -n -1)

    for header in $old_headers; do
        if $DRY_RUN; then
            log_info "[DRY-RUN] Would remove: $header"
        else
            apt-get purge -y "$header" 2>/dev/null || true
        fi
    done

    if ! $DRY_RUN; then
        # Clean up
        apt-get autoremove -y 2>/dev/null || true
        update-grub 2>/dev/null || true
    fi

    log_success "Old kernels removed: $(format_size $freed)"
    TOTAL_FREED=$((TOTAL_FREED + freed))
}

#-------------------------------------------------------------------------------
# Clean Temporary Files
#-------------------------------------------------------------------------------
clean_temp_files() {
    log_step "Cleaning temporary files..."

    local freed=0

    # /tmp cleanup (files older than 7 days)
    if $DRY_RUN; then
        local tmp_size=$(find /tmp -type f -mtime +7 -exec du -cb {} + 2>/dev/null | tail -1 | cut -f1)
        log_info "[DRY-RUN] Would clean /tmp files older than 7 days: $(format_size ${tmp_size:-0})"
    else
        find /tmp -type f -mtime +7 -delete 2>/dev/null || true
        find /tmp -type d -empty -mtime +7 -delete 2>/dev/null || true
        log_success "/tmp cleaned (files older than 7 days)"
    fi

    # /var/tmp cleanup
    if $DRY_RUN; then
        local vartmp_size=$(get_dir_size "/var/tmp")
        log_info "[DRY-RUN] Would clean /var/tmp: $(format_size $vartmp_size)"
    else
        find /var/tmp -type f -mtime +30 -delete 2>/dev/null || true
        log_success "/var/tmp cleaned (files older than 30 days)"
    fi

    # Clean trash for all users
    for user_home in /home/*; do
        if [ -d "$user_home" ]; then
            local trash_dir="$user_home/.local/share/Trash"
            if [ -d "$trash_dir" ]; then
                local trash_size=$(get_dir_size "$trash_dir")

                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would empty trash for $(basename $user_home): $(format_size $trash_size)"
                else
                    rm -rf "$trash_dir/files"/* 2>/dev/null || true
                    rm -rf "$trash_dir/info"/* 2>/dev/null || true
                    freed=$((freed + trash_size))
                    log_success "Trash emptied for $(basename $user_home): $(format_size $trash_size)"
                fi
            fi
        fi
    done

    # Root trash
    local root_trash="/root/.local/share/Trash"
    if [ -d "$root_trash" ]; then
        local root_trash_size=$(get_dir_size "$root_trash")
        if $DRY_RUN; then
            log_info "[DRY-RUN] Would empty root trash: $(format_size $root_trash_size)"
        else
            rm -rf "$root_trash/files"/* 2>/dev/null || true
            rm -rf "$root_trash/info"/* 2>/dev/null || true
            freed=$((freed + root_trash_size))
        fi
    fi

    log_success "Temporary files cleaned: $(format_size $freed)"
    TOTAL_FREED=$((TOTAL_FREED + freed))
}

#-------------------------------------------------------------------------------
# Clean Build and Development Caches
#-------------------------------------------------------------------------------
clean_dev_caches() {
    log_step "Cleaning development caches..."

    local freed=0

    for user_home in /home/*; do
        if [ -d "$user_home" ]; then
            # Cargo cache (Rust)
            local cargo_cache="$user_home/.cargo/registry/cache"
            if [ -d "$cargo_cache" ]; then
                local cargo_size=$(get_dir_size "$cargo_cache")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would clean Cargo cache: $(format_size $cargo_size)"
                else
                    rm -rf "$cargo_cache"/* 2>/dev/null || true
                    freed=$((freed + cargo_size))
                fi
            fi

            # Go cache
            local go_cache="$user_home/go/pkg"
            if [ -d "$go_cache" ]; then
                local go_size=$(get_dir_size "$go_cache")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would clean Go cache: $(format_size $go_size)"
                fi
            fi

            # Maven cache
            local maven_cache="$user_home/.m2/repository"
            if [ -d "$maven_cache" ]; then
                local maven_size=$(get_dir_size "$maven_cache")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Maven cache size: $(format_size $maven_size) (not removing)"
                fi
            fi

            # Gradle cache
            local gradle_cache="$user_home/.gradle/caches"
            if [ -d "$gradle_cache" ]; then
                local gradle_size=$(get_dir_size "$gradle_cache")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Gradle cache size: $(format_size $gradle_size) (not removing)"
                fi
            fi

            # ccache
            local ccache_dir="$user_home/.cache/ccache"
            if [ -d "$ccache_dir" ]; then
                local ccache_size=$(get_dir_size "$ccache_dir")
                if $DRY_RUN; then
                    log_info "[DRY-RUN] ccache size: $(format_size $ccache_size) (not removing - use ccache -C)"
                fi
            fi
        fi
    done

    log_success "Development caches cleaned: $(format_size $freed)"
    TOTAL_FREED=$((TOTAL_FREED + freed))
}

#-------------------------------------------------------------------------------
# Show Disk Status
#-------------------------------------------------------------------------------
show_status() {
    echo -e "${WHITE}=== Disk Usage Status ===${NC}"
    echo ""

    # Overall disk usage
    echo -e "${CYAN}Disk Usage:${NC}"
    df -h / | tail -1 | awk '{print "  Root: "$3" used / "$2" total ("$5" used)"}'
    echo ""

    # Package cache sizes
    echo -e "${CYAN}Package Caches:${NC}"
    local apt_size=$(get_dir_size "/var/cache/apt/archives")
    echo "  APT cache: $(format_size $apt_size)"

    if command -v flatpak &>/dev/null; then
        local flatpak_unused=$(flatpak list --unused 2>/dev/null | wc -l)
        echo "  Flatpak unused: $flatpak_unused items"
    fi

    if command -v snap &>/dev/null; then
        local snap_cache=$(get_dir_size "/var/lib/snapd/cache")
        echo "  Snap cache: $(format_size $snap_cache)"
    fi
    echo ""

    # Log sizes
    echo -e "${CYAN}Logs:${NC}"
    local journal_size=$(journalctl --disk-usage 2>/dev/null | grep -oP '\d+\.?\d*[KMGT]' | head -1 || echo "N/A")
    echo "  Journal: $journal_size"
    local log_size=$(du -sh /var/log 2>/dev/null | cut -f1)
    echo "  /var/log: $log_size"
    echo ""

    # Cache sizes for current user
    if [ -n "$SUDO_USER" ]; then
        local user_home=$(getent passwd $SUDO_USER | cut -d: -f6)
        echo -e "${CYAN}User Cache ($SUDO_USER):${NC}"

        local thumb_size=$(get_dir_size "$user_home/.cache/thumbnails")
        echo "  Thumbnails: $(format_size $thumb_size)"

        for browser in "mozilla/firefox" "google-chrome" "chromium" "BraveSoftware"; do
            local browser_cache="$user_home/.cache/$browser"
            if [ -d "$browser_cache" ]; then
                local b_size=$(get_dir_size "$browser_cache")
                echo "  ${browser%%/*}: $(format_size $b_size)"
            fi
        done

        local trash_size=$(get_dir_size "$user_home/.local/share/Trash")
        echo "  Trash: $(format_size $trash_size)"
    fi
    echo ""

    # Kernel count
    echo -e "${CYAN}Kernels:${NC}"
    local kernel_count=$(dpkg -l 'linux-image-*' 2>/dev/null | grep '^ii' | wc -l)
    echo "  Installed: $kernel_count"
    echo "  Current: $(uname -r)"
}

#-------------------------------------------------------------------------------
# Run All Cleanup
#-------------------------------------------------------------------------------
run_all_cleanup() {
    local total_steps=7
    local current_step=0

    echo -e "${YELLOW}=== Running Full System Cleanup ===${NC}"
    echo ""

    ((current_step++))
    show_progress $current_step $total_steps
    create_backup_info

    ((current_step++))
    show_progress $current_step $total_steps
    clean_package_cache

    ((current_step++))
    show_progress $current_step $total_steps
    clean_old_logs

    ((current_step++))
    show_progress $current_step $total_steps
    clean_thumbnails

    ((current_step++))
    show_progress $current_step $total_steps
    clean_browser_cache

    ((current_step++))
    show_progress $current_step $total_steps
    remove_old_kernels

    ((current_step++))
    show_progress $current_step $total_steps
    clean_temp_files

    clean_dev_caches

    # Final summary
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}      System Cleanup Complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "Total space freed: $(format_size $TOTAL_FREED)"
    echo ""

    if $DRY_RUN; then
        echo -e "${YELLOW}This was a dry run - no files were actually deleted.${NC}"
        echo "Run without --dry-run to perform actual cleanup."
    else
        echo "Cleanup log saved to: $LOG_FILE"
    fi

    # Show new disk usage
    echo ""
    echo "Current disk usage:"
    df -h / | tail -1 | awk '{print "  Root: "$3" used / "$2" total ("$5" used)"}'
}

#-------------------------------------------------------------------------------
# Main Function
#-------------------------------------------------------------------------------
main() {
    local mode="all"

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -d|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -a|--all)
                mode="all"
                shift
                ;;
            -p|--packages)
                mode="packages"
                shift
                ;;
            -l|--logs)
                mode="logs"
                shift
                ;;
            -t|--thumbnails)
                mode="thumbnails"
                shift
                ;;
            -b|--browser)
                mode="browser"
                shift
                ;;
            -k|--kernels)
                mode="kernels"
                shift
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
                echo "Winux System Cleanup v$VERSION"
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

    # Create log directory
    mkdir -p /var/log/winux 2>/dev/null || true
    mkdir -p /var/lib/winux/backups 2>/dev/null || true

    # Show logo
    show_logo

    if $DRY_RUN; then
        echo -e "${YELLOW}=== DRY RUN MODE - No files will be deleted ===${NC}"
        echo ""
    fi

    # Execute based on mode
    case $mode in
        all)
            run_all_cleanup
            ;;
        packages)
            create_backup_info
            clean_package_cache
            echo "Space freed: $(format_size $TOTAL_FREED)"
            ;;
        logs)
            create_backup_info
            clean_old_logs
            echo "Space freed: $(format_size $TOTAL_FREED)"
            ;;
        thumbnails)
            create_backup_info
            clean_thumbnails
            echo "Space freed: $(format_size $TOTAL_FREED)"
            ;;
        browser)
            create_backup_info
            clean_browser_cache
            echo "Space freed: $(format_size $TOTAL_FREED)"
            ;;
        kernels)
            create_backup_info
            remove_old_kernels
            echo "Space freed: $(format_size $TOTAL_FREED)"
            ;;
    esac
}

main "$@"
