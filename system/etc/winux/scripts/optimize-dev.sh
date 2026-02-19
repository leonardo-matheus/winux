#!/bin/bash
#===============================================================================
# Winux OS - Developer Environment Optimizer
# Optimize system for development workflows
#===============================================================================

set -e

#-------------------------------------------------------------------------------
# Configuration
#-------------------------------------------------------------------------------
VERSION="1.0.0"
SCRIPT_NAME="$(basename "$0")"
BACKUP_DIR="/var/lib/winux/backups/dev-$(date +%Y%m%d_%H%M%S)"
LOG_FILE="/var/log/winux/optimize-dev.log"
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
    echo -e "${BLUE}"
    cat << 'EOF'
 __        __ _
 \ \      / /(_) _ __  _   _ __ __
  \ \ /\ / / | || '_ \| | | |\ \/ /
   \ V  V /  | || | | | |_| | >  <
    \_/\_/   |_||_| |_|\__,_|/_/\_\

     Developer Optimizer v1.0.0
    ============================
          [DEV MODE]
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
    echo -e "${CYAN}[DEV]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [DEV] $1" >> "$LOG_FILE" 2>/dev/null || true
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

    printf "\r${BLUE}["
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
    echo "Developer environment optimizer for Winux OS"
    echo ""
    echo "Options:"
    echo "  -d, --dry-run     Show what would be done without making changes"
    echo "  -r, --restore     Restore from backup"
    echo "  -s, --status      Show current dev optimization status"
    echo "  -h, --help        Show this help message"
    echo "  -v, --version     Show version"
    echo ""
    echo "Features:"
    echo "  - File watchers limit increase"
    echo "  - Git performance configuration"
    echo "  - Docker/Podman optimizations"
    echo "  - Compile cache (ccache, sccache)"
    echo "  - Tmpfs for builds"
    echo ""
    echo "Examples:"
    echo "  $SCRIPT_NAME                  # Apply all dev optimizations"
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

    # Backup current inotify settings
    cat /proc/sys/fs/inotify/max_user_watches > "$BACKUP_DIR/inotify_watches.txt"
    cat /proc/sys/fs/inotify/max_user_instances > "$BACKUP_DIR/inotify_instances.txt"

    # Backup Docker daemon config
    if [ -f /etc/docker/daemon.json ]; then
        cp /etc/docker/daemon.json "$BACKUP_DIR/"
    fi

    # Backup fstab
    cp /etc/fstab "$BACKUP_DIR/"

    log_success "Backup created at $BACKUP_DIR"
}

#-------------------------------------------------------------------------------
# Restore from Backup
#-------------------------------------------------------------------------------
restore_backup() {
    log_step "Restoring from backup..."

    local latest_backup=$(ls -td /var/lib/winux/backups/dev-* 2>/dev/null | head -1)

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

    # Restore Docker config
    if [ -f "$latest_backup/daemon.json" ]; then
        cp "$latest_backup/daemon.json" /etc/docker/daemon.json
        systemctl restart docker 2>/dev/null || true
        log_success "Restored Docker configuration"
    fi

    log_success "Restore complete"
}

#-------------------------------------------------------------------------------
# Configure File Watchers
#-------------------------------------------------------------------------------
configure_file_watchers() {
    log_step "Configuring file watchers limit..."

    local max_watches=524288
    local max_instances=1024
    local max_queued=16384

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would set:"
        log_info "  max_user_watches = $max_watches"
        log_info "  max_user_instances = $max_instances"
        log_info "  max_queued_events = $max_queued"
        return
    fi

    # Apply immediately
    echo $max_watches > /proc/sys/fs/inotify/max_user_watches
    echo $max_instances > /proc/sys/fs/inotify/max_user_instances
    echo $max_queued > /proc/sys/fs/inotify/max_queued_events

    # Make persistent
    cat > /etc/sysctl.d/99-winux-dev-watchers.conf << EOF
# Winux Developer Optimization - File Watchers
# Increased for IDEs (VS Code, IntelliJ, etc.)

fs.inotify.max_user_watches = $max_watches
fs.inotify.max_user_instances = $max_instances
fs.inotify.max_queued_events = $max_queued
EOF

    log_success "File watchers configured: $max_watches watches, $max_instances instances"
}

#-------------------------------------------------------------------------------
# Git Performance Configuration
#-------------------------------------------------------------------------------
configure_git_performance() {
    log_step "Configuring Git for performance..."

    if ! command -v git &>/dev/null; then
        log_warn "Git not installed"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure Git performance settings"
        return
    fi

    # System-wide Git performance settings
    cat > /etc/gitconfig << 'EOF'
# Winux Git Performance Configuration

[core]
    # Use delta for better diffs (if available)
    pager = delta --dark 2>/dev/null || less

    # Preload index for faster status
    preloadindex = true

    # Enable parallel index operations
    fsmonitor = true

    # Untrackedcache for faster status
    untrackedcache = true

    # Multi-pack index
    multiPackIndex = true

[pack]
    # Use all available threads for packing
    threads = 0

    # Larger window for better compression
    window = 250
    depth = 50

[gc]
    # Auto gc less frequently
    auto = 256
    autodetach = true

[fetch]
    # Parallel fetch
    parallel = 0
    prune = true

[protocol]
    # Use protocol v2 for better performance
    version = 2

[feature]
    # Enable new features
    manyFiles = true

[index]
    # Index version 4 for better performance
    version = 4

[status]
    # Show ahead/behind info
    aheadBehind = true
    # Show submodule summary
    submoduleSummary = true

[diff]
    # Use histogram algorithm for better diffs
    algorithm = histogram
    # Detect renames and copies
    renames = copies

[merge]
    # Show common ancestor in conflicts
    conflictstyle = zdiff3

[rebase]
    # Auto stash before rebase
    autoStash = true
    # Update refs automatically
    updateRefs = true

[pull]
    # Rebase by default
    rebase = true

[credential]
    # Cache credentials for 1 hour
    helper = cache --timeout=3600

[http]
    # Increase buffer for large repos
    postBuffer = 524288000
EOF

    # Create Git LFS config for large repos
    mkdir -p /etc/gitconfig.d
    cat > /etc/gitconfig.d/lfs.conf << 'EOF'
[filter "lfs"]
    clean = git-lfs clean -- %f
    smudge = git-lfs smudge -- %f
    process = git-lfs filter-process
    required = true

[lfs]
    # Parallel transfers
    concurrenttransfers = 8
EOF

    log_success "Git configured for performance"
}

#-------------------------------------------------------------------------------
# Docker/Podman Optimizations
#-------------------------------------------------------------------------------
configure_docker() {
    log_step "Configuring Docker optimizations..."

    if ! command -v docker &>/dev/null && ! command -v podman &>/dev/null; then
        log_warn "Neither Docker nor Podman installed"
        return
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure Docker/Podman optimizations"
        return
    fi

    # Docker configuration
    if command -v docker &>/dev/null; then
        mkdir -p /etc/docker
        cat > /etc/docker/daemon.json << 'EOF'
{
    "storage-driver": "overlay2",
    "storage-opts": [
        "overlay2.override_kernel_check=true"
    ],
    "log-driver": "json-file",
    "log-opts": {
        "max-size": "10m",
        "max-file": "3"
    },
    "default-ulimits": {
        "nofile": {
            "Name": "nofile",
            "Hard": 65536,
            "Soft": 65536
        }
    },
    "max-concurrent-downloads": 10,
    "max-concurrent-uploads": 10,
    "features": {
        "buildkit": true
    },
    "builder": {
        "gc": {
            "enabled": true,
            "defaultKeepStorage": "20GB"
        }
    },
    "experimental": true
}
EOF

        # Restart Docker to apply
        systemctl restart docker 2>/dev/null || true
        log_success "Docker configured with BuildKit and optimizations"
    fi

    # Podman configuration
    if command -v podman &>/dev/null; then
        mkdir -p /etc/containers
        cat > /etc/containers/containers.conf << 'EOF'
[containers]
log_driver = "k8s-file"
log_size_max = 10485760

[engine]
cgroup_manager = "systemd"
events_logger = "file"
runtime = "crun"

[network]
network_backend = "netavark"
EOF
        log_success "Podman configured"
    fi

    # Add current user to docker group
    if groups $SUDO_USER 2>/dev/null | grep -qv docker; then
        usermod -aG docker $SUDO_USER 2>/dev/null || true
        log_info "Added $SUDO_USER to docker group (re-login required)"
    fi
}

#-------------------------------------------------------------------------------
# Compile Cache Configuration
#-------------------------------------------------------------------------------
configure_compile_cache() {
    log_step "Configuring compile caches..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure ccache and sccache"
        return
    fi

    # ccache for C/C++
    if command -v ccache &>/dev/null; then
        # System-wide ccache configuration
        mkdir -p /etc/ccache.conf.d
        cat > /etc/ccache.conf << 'EOF'
# Winux ccache Configuration

# Cache size (10GB)
max_size = 10G

# Use compression
compression = true
compression_level = 5

# Enable statistics
stats = true

# Use hard links when possible
hard_link = true

# Sloppiness options for better hit rate
sloppiness = include_file_ctime,include_file_mtime,time_macros,pch_defines,file_stat_matches

# Hash directory
hash_dir = true
EOF

        # Create symlinks for compiler wrappers
        mkdir -p /usr/lib/ccache
        for compiler in gcc g++ cc c++ clang clang++; do
            if command -v $compiler &>/dev/null; then
                ln -sf /usr/bin/ccache /usr/lib/ccache/$compiler 2>/dev/null || true
            fi
        done

        log_success "ccache configured (10GB cache)"
    else
        log_info "ccache not installed. Install with: sudo apt install ccache"
    fi

    # sccache for Rust and more
    if command -v sccache &>/dev/null; then
        cat > /etc/profile.d/sccache.sh << 'EOF'
# Winux sccache Configuration
export SCCACHE_CACHE_SIZE="10G"
export SCCACHE_DIR="$HOME/.cache/sccache"
export RUSTC_WRAPPER="sccache"
EOF
        log_success "sccache configured"
    else
        log_info "sccache not installed. Install with: cargo install sccache"
    fi

    # Create profile.d entry for ccache
    cat > /etc/profile.d/ccache.sh << 'EOF'
# Winux ccache Configuration
export CCACHE_DIR="$HOME/.cache/ccache"
export PATH="/usr/lib/ccache:$PATH"
EOF
}

#-------------------------------------------------------------------------------
# Tmpfs for Builds
#-------------------------------------------------------------------------------
configure_tmpfs_builds() {
    log_step "Configuring tmpfs for builds..."

    local ram_size=$(awk '/MemTotal/ {print int($2/1024/1024)}' /proc/meminfo)
    local tmpfs_size="4G"

    # Adjust based on RAM
    if [ "$ram_size" -ge 32 ]; then
        tmpfs_size="16G"
    elif [ "$ram_size" -ge 16 ]; then
        tmpfs_size="8G"
    elif [ "$ram_size" -ge 8 ]; then
        tmpfs_size="4G"
    else
        tmpfs_size="2G"
    fi

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure tmpfs build directories:"
        log_info "  /tmp/build - ${tmpfs_size}"
        log_info "  /dev/shm size increase"
        return
    fi

    # Create build directory
    mkdir -p /tmp/build

    # Add tmpfs entries to fstab if not present
    if ! grep -q "winux-tmpfs-build" /etc/fstab; then
        cat >> /etc/fstab << EOF

# Winux Developer tmpfs - Build directory
# winux-tmpfs-build
tmpfs   /tmp/build   tmpfs   defaults,noatime,mode=1777,size=${tmpfs_size}   0   0
EOF

        # Mount immediately
        mount /tmp/build 2>/dev/null || true

        log_success "tmpfs build directory configured at /tmp/build (${tmpfs_size})"
    else
        log_info "tmpfs build directory already configured"
    fi

    # Increase /dev/shm size
    if ! grep -q "winux-shm" /etc/fstab; then
        # Remount with larger size
        mount -o remount,size=${tmpfs_size} /dev/shm 2>/dev/null || true
        log_success "/dev/shm increased to ${tmpfs_size}"
    fi

    # Create user build directory symlinks
    if [ -n "$SUDO_USER" ]; then
        local user_home=$(getent passwd $SUDO_USER | cut -d: -f6)
        mkdir -p "$user_home/.cache/builds"
        chown $SUDO_USER:$SUDO_USER "$user_home/.cache/builds"

        # Create link to tmpfs
        ln -sf /tmp/build "$user_home/.cache/tmpfs-build" 2>/dev/null || true
        chown -h $SUDO_USER:$SUDO_USER "$user_home/.cache/tmpfs-build"

        log_success "User build directory linked: ~/.cache/tmpfs-build"
    fi
}

#-------------------------------------------------------------------------------
# Additional Development Optimizations
#-------------------------------------------------------------------------------
apply_dev_tweaks() {
    log_step "Applying additional development tweaks..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would apply development tweaks"
        return
    fi

    # Increase file descriptors limit
    cat > /etc/security/limits.d/99-winux-dev.conf << 'EOF'
# Winux Developer Limits

# Increase open file limits
*               soft    nofile          65536
*               hard    nofile          524288
root            soft    nofile          65536
root            hard    nofile          524288

# Increase process limits
*               soft    nproc           65536
*               hard    nproc           65536

# Increase core dump size (for debugging)
*               soft    core            unlimited
*               hard    core            unlimited

# Increase memlock for JVM and containers
*               soft    memlock         unlimited
*               hard    memlock         unlimited
EOF

    # Sysctl optimizations for development
    cat > /etc/sysctl.d/99-winux-dev.conf << 'EOF'
# Winux Developer Kernel Tuning

# Network performance for local development
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535

# Virtual memory for large builds
vm.max_map_count = 262144
vm.swappiness = 10

# File system
fs.file-max = 2097152
fs.aio-max-nr = 1048576
EOF

    sysctl -p /etc/sysctl.d/99-winux-dev.conf 2>/dev/null || true

    log_success "Development tweaks applied"
}

#-------------------------------------------------------------------------------
# IDE-Specific Optimizations
#-------------------------------------------------------------------------------
configure_ide_optimizations() {
    log_step "Configuring IDE optimizations..."

    if $DRY_RUN; then
        log_info "[DRY-RUN] Would configure IDE optimizations"
        return
    fi

    # VS Code optimizations
    if [ -n "$SUDO_USER" ]; then
        local user_home=$(getent passwd $SUDO_USER | cut -d: -f6)

        # VS Code settings
        local vscode_settings="$user_home/.config/Code/User"
        mkdir -p "$vscode_settings"

        if [ ! -f "$vscode_settings/settings.json" ]; then
            cat > "$vscode_settings/settings.json" << 'EOF'
{
    "files.watcherExclude": {
        "**/.git/objects/**": true,
        "**/.git/subtree-cache/**": true,
        "**/node_modules/**": true,
        "**/tmp/**": true,
        "**/dist/**": true,
        "**/build/**": true,
        "**/.cache/**": true,
        "**/target/**": true
    },
    "search.followSymlinks": false,
    "git.autorefresh": false,
    "npm.autoDetect": "off",
    "files.exclude": {
        "**/.git": true,
        "**/.DS_Store": true,
        "**/node_modules": true,
        "**/bower_components": true
    }
}
EOF
            chown -R $SUDO_USER:$SUDO_USER "$user_home/.config/Code"
            log_success "VS Code settings optimized"
        fi

        # JetBrains IDEs
        for ide_dir in "$user_home/.config/JetBrains"/*; do
            if [ -d "$ide_dir" ]; then
                local idea_properties="$ide_dir/idea.properties"
                if [ ! -f "$idea_properties" ]; then
                    mkdir -p "$ide_dir"
                    cat > "$idea_properties" << 'EOF'
# Winux JetBrains Optimization
idea.max.intellisense.filesize=5000
idea.cycle.buffer.size=2048
idea.no.launcher=false
idea.dynamic.classpath=true
EOF
                    chown $SUDO_USER:$SUDO_USER "$idea_properties"
                fi
            fi
        done

        log_success "IDE optimizations configured"
    fi
}

#-------------------------------------------------------------------------------
# Show Status
#-------------------------------------------------------------------------------
show_status() {
    echo -e "${WHITE}=== Developer Optimization Status ===${NC}"
    echo ""

    # File watchers
    echo -e "${CYAN}File Watchers:${NC}"
    echo "  max_user_watches: $(cat /proc/sys/fs/inotify/max_user_watches)"
    echo "  max_user_instances: $(cat /proc/sys/fs/inotify/max_user_instances)"
    echo ""

    # ccache status
    echo -e "${CYAN}Compile Cache:${NC}"
    if command -v ccache &>/dev/null; then
        echo "  ccache: installed"
        ccache -s 2>/dev/null | grep -E "(cache size|cache hit)" | sed 's/^/    /'
    else
        echo "  ccache: not installed"
    fi

    if command -v sccache &>/dev/null; then
        echo "  sccache: installed"
        sccache --show-stats 2>/dev/null | head -5 | sed 's/^/    /'
    else
        echo "  sccache: not installed"
    fi
    echo ""

    # Docker status
    echo -e "${CYAN}Container Runtime:${NC}"
    if command -v docker &>/dev/null; then
        echo "  Docker: $(docker --version 2>/dev/null | cut -d, -f1)"
        docker info 2>/dev/null | grep -E "(Storage Driver|BuildKit)" | sed 's/^/    /'
    fi
    if command -v podman &>/dev/null; then
        echo "  Podman: $(podman --version 2>/dev/null)"
    fi
    echo ""

    # tmpfs status
    echo -e "${CYAN}Build Tmpfs:${NC}"
    if mountpoint -q /tmp/build 2>/dev/null; then
        echo "  /tmp/build: mounted"
        df -h /tmp/build 2>/dev/null | tail -1 | awk '{print "    Size: "$2", Used: "$3", Available: "$4}'
    else
        echo "  /tmp/build: not mounted"
    fi
    echo ""

    # Git configuration
    echo -e "${CYAN}Git:${NC}"
    if [ -f /etc/gitconfig ]; then
        echo "  System config: present"
        echo "  Protocol version: $(git config --system protocol.version 2>/dev/null || echo 'default')"
        echo "  Fsmonitor: $(git config --system core.fsmonitor 2>/dev/null || echo 'disabled')"
    fi
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
                echo "Winux Developer Optimizer v$VERSION"
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

    # Total steps for progress
    local total_steps=7
    local current_step=0

    # Create backup
    ((current_step++))
    show_progress $current_step $total_steps
    create_backup

    # Configure file watchers
    ((current_step++))
    show_progress $current_step $total_steps
    configure_file_watchers

    # Configure Git
    ((current_step++))
    show_progress $current_step $total_steps
    configure_git_performance

    # Configure Docker/Podman
    ((current_step++))
    show_progress $current_step $total_steps
    configure_docker

    # Configure compile cache
    ((current_step++))
    show_progress $current_step $total_steps
    configure_compile_cache

    # Configure tmpfs for builds
    ((current_step++))
    show_progress $current_step $total_steps
    configure_tmpfs_builds

    # Apply dev tweaks
    ((current_step++))
    show_progress $current_step $total_steps
    apply_dev_tweaks

    # Configure IDE optimizations
    configure_ide_optimizations

    # Done
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Developer optimization complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "Applied optimizations:"
    echo "  - File watchers: 524288 max"
    echo "  - Git: protocol v2, fsmonitor, etc."
    echo "  - Docker: BuildKit, overlay2, logging"
    echo "  - ccache/sccache: configured"
    echo "  - tmpfs: /tmp/build ready"
    echo "  - Limits: increased file descriptors"
    echo ""
    echo "Notes:"
    echo "  - Re-login may be required for some changes"
    echo "  - ccache path added to PATH in /etc/profile.d/"
    echo "  - Use ~/.cache/tmpfs-build for fast builds"
    echo ""
    echo "Backup saved to: $BACKUP_DIR"
    echo "To restore: $SCRIPT_NAME --restore"
}

main "$@"
