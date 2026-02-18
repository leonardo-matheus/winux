#!/bin/bash
# =============================================================================
# Winux OS - Build System v2.0
# =============================================================================
# Script principal completo para construcao da ISO do Winux OS
# Requer: Ubuntu 22.04+ com privilegios de root
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuracoes
# -----------------------------------------------------------------------------
WINUX_VERSION="${WINUX_VERSION:-1.0}"
WINUX_CODENAME="${WINUX_CODENAME:-aurora}"
UBUNTU_VERSION="24.04"
UBUNTU_CODENAME="noble"

# Diretorios
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_DIR="${BUILD_DIR:-/tmp/winux-build}"
CHROOT_DIR="${BUILD_DIR}/chroot"
ISO_DIR="${BUILD_DIR}/iso"
OUTPUT_DIR="${OUTPUT_DIR:-${PROJECT_ROOT}/output}"
CACHE_DIR="${CACHE_DIR:-${BUILD_DIR}/cache}"

# Opcoes de build
COMPRESSION="${COMPRESSION:-zstd}"
COMPRESSION_LEVEL="${COMPRESSION_LEVEL:-19}"
PARALLEL_JOBS="${PARALLEL_JOBS:-$(nproc)}"
QUICK_BUILD="${QUICK_BUILD:-false}"

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'
BOLD='\033[1m'

# Timestamp
BUILD_TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="${BUILD_DIR}/build-${BUILD_TIMESTAMP}.log"

# -----------------------------------------------------------------------------
# Funcoes de Log
# -----------------------------------------------------------------------------
log_info() { echo -e "${GREEN}[INFO]${NC} $1" | tee -a "${LOG_FILE}" 2>/dev/null || echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "${LOG_FILE}" 2>/dev/null || echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1" | tee -a "${LOG_FILE}" 2>/dev/null || echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} ${BOLD}$1${NC}" | tee -a "${LOG_FILE}" 2>/dev/null || echo -e "${BLUE}[STEP]${NC} ${BOLD}$1${NC}"; }
log_phase() { echo -e "\n${MAGENTA}═══════════════════════════════════════════════════════════════${NC}"; echo -e "${MAGENTA}  FASE: $1${NC}"; echo -e "${MAGENTA}═══════════════════════════════════════════════════════════════${NC}\n"; }
log_debug() { [[ "${DEBUG:-false}" == "true" ]] && echo -e "${CYAN}[DEBUG]${NC} $1" | tee -a "${LOG_FILE}" 2>/dev/null; }

# -----------------------------------------------------------------------------
# Verificacoes
# -----------------------------------------------------------------------------
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "Este script deve ser executado como root"
        log_info "Use: sudo $0 $*"
        exit 1
    fi
}

check_system() {
    log_step "Verificando sistema..."

    # Verificar distribuicao
    if [[ ! -f /etc/os-release ]]; then
        log_error "Sistema nao suportado (sem /etc/os-release)"
        exit 1
    fi

    source /etc/os-release
    if [[ "${ID}" != "ubuntu" && "${ID_LIKE}" != *"ubuntu"* ]]; then
        log_warn "Sistema nao e Ubuntu, pode haver incompatibilidades"
    fi

    # Verificar espaco em disco
    local available_space=$(df -BG "${BUILD_DIR%/*}" 2>/dev/null | awk 'NR==2 {print $4}' | tr -d 'G')
    if [[ -n "${available_space}" && "${available_space}" -lt 50 ]]; then
        log_warn "Espaco em disco baixo: ${available_space}GB disponivel (recomendado: 50GB+)"
    fi

    # Verificar memoria
    local total_mem=$(free -g | awk '/^Mem:/ {print $2}')
    if [[ "${total_mem}" -lt 4 ]]; then
        log_warn "Pouca memoria RAM: ${total_mem}GB (recomendado: 8GB+)"
    fi

    log_info "Sistema verificado: ${PRETTY_NAME:-Unknown}"
}

check_deps() {
    log_step "Verificando dependencias..."

    local deps=(
        debootstrap
        squashfs-tools
        xorriso
        grub-pc-bin
        grub-efi-amd64-bin
        grub-common
        mtools
        dosfstools
        isolinux
        syslinux-common
        rsync
        wget
        curl
        git
        build-essential
        libgtk-4-dev
        libadwaita-1-dev
        pkg-config
    )

    local missing=()

    for dep in "${deps[@]}"; do
        if ! dpkg -l 2>/dev/null | grep -q "^ii  $dep"; then
            missing+=("$dep")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_warn "Instalando dependencias faltantes: ${missing[*]}"
        apt update
        apt install -y "${missing[@]}"
    fi

    # Verificar Rust
    if ! command -v cargo &> /dev/null; then
        log_warn "Rust nao encontrado, instalando..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    log_info "Todas as dependencias estao instaladas"
}

# -----------------------------------------------------------------------------
# Fase 1: Preparacao
# -----------------------------------------------------------------------------
phase_prepare() {
    log_phase "1 - PREPARACAO DO AMBIENTE"

    log_step "Criando estrutura de diretorios..."

    mkdir -p "${BUILD_DIR}"
    mkdir -p "${CHROOT_DIR}"
    mkdir -p "${ISO_DIR}"/{casper,boot/grub,EFI/BOOT,isolinux,.disk}
    mkdir -p "${OUTPUT_DIR}"
    mkdir -p "${CACHE_DIR}"/{debootstrap,packages,rust}

    # Iniciar log
    echo "=== Winux OS Build Log ===" > "${LOG_FILE}"
    echo "Data: $(date)" >> "${LOG_FILE}"
    echo "Versao: ${WINUX_VERSION}" >> "${LOG_FILE}"
    echo "Codename: ${WINUX_CODENAME}" >> "${LOG_FILE}"
    echo "=========================" >> "${LOG_FILE}"

    log_info "Diretorios criados"
    log_info "Log: ${LOG_FILE}"
}

# -----------------------------------------------------------------------------
# Fase 2: Debootstrap
# -----------------------------------------------------------------------------
phase_debootstrap() {
    log_phase "2 - DEBOOTSTRAP (Sistema Base)"

    if [[ -d "${CHROOT_DIR}/bin" && -f "${CHROOT_DIR}/etc/os-release" ]]; then
        log_warn "Sistema base ja existe, pulando debootstrap"
        log_info "Use 'clean' para reconstruir do zero"
        return
    fi

    log_step "Criando sistema base Ubuntu ${UBUNTU_VERSION} (${UBUNTU_CODENAME})..."

    # Usar cache se disponivel
    local cache_opt=""
    if [[ -d "${CACHE_DIR}/debootstrap" ]]; then
        cache_opt="--cache-dir=${CACHE_DIR}/debootstrap"
    fi

    debootstrap \
        --arch=amd64 \
        --variant=minbase \
        --include=apt-utils,locales,sudo \
        ${cache_opt} \
        "${UBUNTU_CODENAME}" \
        "${CHROOT_DIR}" \
        http://archive.ubuntu.com/ubuntu

    log_info "Sistema base criado com sucesso"
}

# -----------------------------------------------------------------------------
# Fase 3: Montar/Desmontar Chroot
# -----------------------------------------------------------------------------
mount_chroot() {
    log_step "Montando filesystems no chroot..."

    # Evitar montar duplicado
    if mountpoint -q "${CHROOT_DIR}/dev"; then
        log_warn "Chroot ja montado"
        return
    fi

    mount --bind /dev "${CHROOT_DIR}/dev"
    mount --bind /dev/pts "${CHROOT_DIR}/dev/pts"
    mount -t proc none "${CHROOT_DIR}/proc"
    mount -t sysfs none "${CHROOT_DIR}/sys"
    mount -t tmpfs none "${CHROOT_DIR}/run"

    # DNS
    mkdir -p "${CHROOT_DIR}/run/systemd/resolve"
    cp /etc/resolv.conf "${CHROOT_DIR}/etc/" 2>/dev/null || true

    log_info "Filesystems montados"
}

umount_chroot() {
    log_step "Desmontando filesystems do chroot..."

    local mounts=("run" "sys" "proc" "dev/pts" "dev")

    for mnt in "${mounts[@]}"; do
        if mountpoint -q "${CHROOT_DIR}/${mnt}" 2>/dev/null; then
            umount -l "${CHROOT_DIR}/${mnt}" 2>/dev/null || true
        fi
    done

    log_info "Filesystems desmontados"
}

# Funcao auxiliar para executar comandos no chroot
run_in_chroot() {
    chroot "${CHROOT_DIR}" /bin/bash -c "$1"
}

# -----------------------------------------------------------------------------
# Fase 4: Configurar Sistema Base
# -----------------------------------------------------------------------------
phase_configure_base() {
    log_phase "3 - CONFIGURACAO DO SISTEMA BASE"

    mount_chroot

    log_step "Configurando repositorios APT..."

    cat > "${CHROOT_DIR}/etc/apt/sources.list" << EOF
# Ubuntu ${UBUNTU_VERSION} (${UBUNTU_CODENAME})
deb http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME} main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME}-updates main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME}-security main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME}-backports main restricted universe multiverse
EOF

    log_step "Instalando pacotes essenciais..."

    run_in_chroot "
export DEBIAN_FRONTEND=noninteractive
apt update
apt upgrade -y

# Kernel e boot
apt install -y \
    linux-image-generic \
    linux-headers-generic \
    linux-firmware \
    systemd \
    systemd-boot \
    dbus \
    dbus-x11

# Rede e conectividade
apt install -y \
    network-manager \
    network-manager-gnome \
    wpasupplicant \
    wireless-tools \
    bluez \
    blueman

# Audio (PipeWire)
apt install -y \
    pipewire \
    pipewire-pulse \
    pipewire-alsa \
    wireplumber \
    libspa-0.2-bluetooth

# Display e GPU
apt install -y \
    mesa-utils \
    mesa-vulkan-drivers \
    libgl1-mesa-dri \
    vainfo \
    vdpauinfo \
    xorg \
    xserver-xorg-video-all

# Desktop essenciais (Wayland + X11)
apt install -y \
    wayland-protocols \
    libwayland-dev \
    weston \
    xwayland \
    libgtk-4-1 \
    libadwaita-1-0 \
    gnome-themes-extra \
    adwaita-icon-theme \
    papirus-icon-theme \
    fonts-noto \
    fonts-noto-color-emoji \
    fonts-liberation

# Utilitarios
apt install -y \
    sudo \
    locales \
    console-setup \
    keyboard-configuration \
    tzdata \
    curl \
    wget \
    git \
    nano \
    vim \
    htop \
    neofetch \
    bash-completion \
    usbutils \
    pciutils \
    lshw \
    dmidecode

# Live session
apt install -y \
    casper \
    lupin-casper \
    discover \
    laptop-detect \
    os-prober \
    efibootmgr \
    grub-efi-amd64-signed \
    shim-signed

# Instalador Calamares
apt install -y \
    calamares \
    calamares-settings-ubuntu \
    libkf5parts5 \
    qml-module-qtquick2 \
    qml-module-qtquick-window2 \
    qml-module-qtquick-layouts \
    qml-module-qtquick-controls \
    qml-module-qtquick-controls2

# Apps basicos
apt install -y \
    firefox \
    thunderbird \
    libreoffice \
    gnome-calculator \
    gnome-system-monitor \
    gnome-disk-utility \
    gparted \
    evince \
    eog \
    file-roller \
    gnome-screenshot

# Gaming e compatibilidade
apt install -y \
    steam-installer \
    gamemode \
    mangohud \
    lutris \
    wine64 \
    winetricks

# Limpar
apt clean
rm -rf /tmp/* /var/tmp/*
"

    log_step "Configurando locale e timezone..."

    run_in_chroot "
locale-gen en_US.UTF-8 pt_BR.UTF-8
update-locale LANG=pt_BR.UTF-8 LC_ALL=pt_BR.UTF-8

# Timezone (sera configurado pelo instalador)
ln -sf /usr/share/zoneinfo/America/Sao_Paulo /etc/localtime
echo 'America/Sao_Paulo' > /etc/timezone
dpkg-reconfigure -f noninteractive tzdata
"

    log_step "Configurando hostname e usuarios..."

    echo "winux" > "${CHROOT_DIR}/etc/hostname"

    cat > "${CHROOT_DIR}/etc/hosts" << EOF
127.0.0.1   localhost
127.0.1.1   winux

# IPv6
::1         localhost ip6-localhost ip6-loopback
ff02::1     ip6-allnodes
ff02::2     ip6-allrouters
EOF

    # Usuario live
    run_in_chroot "
useradd -m -s /bin/bash -G sudo,audio,video,cdrom,plugdev,netdev winux 2>/dev/null || true
echo 'winux:winux' | chpasswd
echo 'winux ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/winux
chmod 440 /etc/sudoers.d/winux
"

    log_info "Sistema base configurado"
}

# -----------------------------------------------------------------------------
# Fase 5: Compilar e Instalar Apps Rust
# -----------------------------------------------------------------------------
phase_install_rust_apps() {
    log_phase "4 - COMPILACAO E INSTALACAO DOS APPS RUST"

    log_step "Compilando aplicativos Winux..."

    # Lista de apps para compilar
    local apps=(
        "winux-files"
        "winux-terminal"
        "winux-settings"
        "winux-monitor"
        "winux-edit"
        "winux-image"
        "winux-player"
        "winux-store"
    )

    local desktop_components=(
        "winux-compositor"
        "winux-shell"
        "winux-panel"
    )

    # Criar diretorio de binarios
    mkdir -p "${CHROOT_DIR}/usr/bin"
    mkdir -p "${CHROOT_DIR}/usr/share/applications"
    mkdir -p "${CHROOT_DIR}/usr/share/icons/hicolor/scalable/apps"

    # Compilar apps
    for app in "${apps[@]}"; do
        local app_dir="${PROJECT_ROOT}/apps/${app}"
        if [[ -d "${app_dir}" ]]; then
            log_info "Compilando ${app}..."

            (
                cd "${app_dir}"
                CARGO_TARGET_DIR="${CACHE_DIR}/rust" cargo build --release -j "${PARALLEL_JOBS}" 2>&1 | tee -a "${LOG_FILE}"
            )

            # Copiar binario
            local binary="${CACHE_DIR}/rust/release/${app}"
            if [[ -f "${binary}" ]]; then
                cp "${binary}" "${CHROOT_DIR}/usr/bin/"
                chmod +x "${CHROOT_DIR}/usr/bin/${app}"
                log_info "  -> ${app} instalado"
            else
                log_warn "  -> ${app} nao encontrado em ${binary}"
            fi

            # Criar .desktop file
            create_desktop_entry "${app}"
        else
            log_warn "App nao encontrado: ${app_dir}"
        fi
    done

    # Compilar componentes desktop
    for component in "${desktop_components[@]}"; do
        local comp_dir="${PROJECT_ROOT}/desktop/${component}"
        if [[ -d "${comp_dir}" ]]; then
            log_info "Compilando ${component}..."

            (
                cd "${comp_dir}"
                CARGO_TARGET_DIR="${CACHE_DIR}/rust" cargo build --release -j "${PARALLEL_JOBS}" 2>&1 | tee -a "${LOG_FILE}"
            )

            local binary="${CACHE_DIR}/rust/release/${component}"
            if [[ -f "${binary}" ]]; then
                cp "${binary}" "${CHROOT_DIR}/usr/bin/"
                chmod +x "${CHROOT_DIR}/usr/bin/${component}"
                log_info "  -> ${component} instalado"
            fi
        fi
    done

    log_info "Aplicativos Rust compilados e instalados"
}

create_desktop_entry() {
    local app="$1"
    local name="${app/winux-/Winux }"
    name="${name^}"  # Capitalize

    local categories="Utility;"
    local icon="utilities-terminal"

    case "${app}" in
        winux-files)
            categories="System;FileManager;"
            icon="system-file-manager"
            name="Winux Files"
            ;;
        winux-terminal)
            categories="System;TerminalEmulator;"
            icon="utilities-terminal"
            name="Winux Terminal"
            ;;
        winux-settings)
            categories="Settings;System;"
            icon="preferences-system"
            name="Winux Settings"
            ;;
        winux-monitor)
            categories="System;Monitor;"
            icon="utilities-system-monitor"
            name="Winux Monitor"
            ;;
        winux-edit)
            categories="Utility;TextEditor;"
            icon="accessories-text-editor"
            name="Winux Edit"
            ;;
        winux-image)
            categories="Graphics;Viewer;"
            icon="image-x-generic"
            name="Winux Image"
            ;;
        winux-player)
            categories="AudioVideo;Player;"
            icon="multimedia-video-player"
            name="Winux Player"
            ;;
        winux-store)
            categories="System;PackageManager;"
            icon="system-software-install"
            name="Winux Store"
            ;;
    esac

    cat > "${CHROOT_DIR}/usr/share/applications/${app}.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=${name}
Comment=Winux OS Native Application
Exec=${app}
Icon=${icon}
Terminal=false
Categories=${categories}
StartupNotify=true
EOF
}

# -----------------------------------------------------------------------------
# Fase 6: Instalar Tema Winux Fluent
# -----------------------------------------------------------------------------
phase_install_theme() {
    log_phase "5 - INSTALACAO DO TEMA WINUX FLUENT"

    log_step "Copiando tema Winux Fluent..."

    local theme_dir="${PROJECT_ROOT}/themes/winux-fluent"
    local dest_dir="${CHROOT_DIR}/usr/share/themes/Winux-Fluent"

    if [[ -d "${theme_dir}" ]]; then
        mkdir -p "${dest_dir}"
        cp -r "${theme_dir}"/* "${dest_dir}/"

        # Configurar como tema padrao
        mkdir -p "${CHROOT_DIR}/etc/gtk-4.0"
        cat > "${CHROOT_DIR}/etc/gtk-4.0/settings.ini" << EOF
[Settings]
gtk-theme-name=Winux-Fluent
gtk-icon-theme-name=Papirus
gtk-font-name=Noto Sans 11
gtk-cursor-theme-name=Adwaita
gtk-application-prefer-dark-theme=1
EOF

        # GTK3 tambem
        mkdir -p "${CHROOT_DIR}/etc/gtk-3.0"
        cat > "${CHROOT_DIR}/etc/gtk-3.0/settings.ini" << EOF
[Settings]
gtk-theme-name=Winux-Fluent
gtk-icon-theme-name=Papirus
gtk-font-name=Noto Sans 11
gtk-cursor-theme-name=Adwaita
gtk-application-prefer-dark-theme=1
EOF

        log_info "Tema instalado: ${dest_dir}"
    else
        log_warn "Tema nao encontrado: ${theme_dir}"
    fi

    # Copiar configuracoes de sistema
    log_step "Copiando configuracoes de sistema..."

    local system_dir="${PROJECT_ROOT}/system"
    if [[ -d "${system_dir}" ]]; then
        cp -r "${system_dir}"/etc/* "${CHROOT_DIR}/etc/" 2>/dev/null || true
        log_info "Configuracoes de sistema copiadas"
    fi
}

# -----------------------------------------------------------------------------
# Fase 7: Configurar Calamares
# -----------------------------------------------------------------------------
phase_configure_calamares() {
    log_phase "6 - CONFIGURACAO DO CALAMARES"

    log_step "Configurando instalador Calamares..."

    local calamares_config="${PROJECT_ROOT}/build/config/calamares"
    local dest="${CHROOT_DIR}/etc/calamares"

    if [[ -d "${calamares_config}" ]]; then
        mkdir -p "${dest}"
        cp -r "${calamares_config}"/* "${dest}/"

        # Copiar branding
        mkdir -p "${dest}/branding/winux"
        if [[ -d "${calamares_config}/branding/winux" ]]; then
            cp -r "${calamares_config}/branding/winux"/* "${dest}/branding/winux/"
        fi

        log_info "Calamares configurado"
    else
        log_warn "Configuracao do Calamares nao encontrada"
    fi

    # Criar entrada desktop para o instalador
    cat > "${CHROOT_DIR}/usr/share/applications/winux-install.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Install Winux OS
Comment=Install Winux OS on your computer
Exec=pkexec calamares
Icon=calamares
Terminal=false
Categories=System;
StartupNotify=true
EOF

    # Autostart do instalador no desktop da sessao live
    mkdir -p "${CHROOT_DIR}/etc/skel/Desktop"
    cp "${CHROOT_DIR}/usr/share/applications/winux-install.desktop" \
       "${CHROOT_DIR}/etc/skel/Desktop/"
    chmod +x "${CHROOT_DIR}/etc/skel/Desktop/winux-install.desktop"
}

# -----------------------------------------------------------------------------
# Fase 8: Configuracoes Finais do Sistema
# -----------------------------------------------------------------------------
phase_final_config() {
    log_phase "7 - CONFIGURACOES FINAIS"

    mount_chroot

    log_step "Configurando servicos systemd..."

    run_in_chroot "
# Habilitar servicos essenciais
systemctl enable NetworkManager
systemctl enable bluetooth
systemctl enable cups 2>/dev/null || true

# Desabilitar servicos desnecessarios para live
systemctl disable apt-daily.timer 2>/dev/null || true
systemctl disable apt-daily-upgrade.timer 2>/dev/null || true

# Configurar PipeWire
systemctl --global enable pipewire.socket 2>/dev/null || true
systemctl --global enable pipewire-pulse.socket 2>/dev/null || true
systemctl --global enable wireplumber.service 2>/dev/null || true
"

    log_step "Configurando autologin para sessao live..."

    # GDM autologin (se usar GDM)
    mkdir -p "${CHROOT_DIR}/etc/gdm3"
    cat > "${CHROOT_DIR}/etc/gdm3/custom.conf" << EOF
[daemon]
AutomaticLoginEnable=true
AutomaticLogin=winux
TimedLoginEnable=true
TimedLogin=winux
TimedLoginDelay=5

[security]

[xdmcp]

[chooser]

[debug]
EOF

    log_step "Criando arquivo de release..."

    cat > "${CHROOT_DIR}/etc/winux-release" << EOF
DISTRIB_ID=Winux
DISTRIB_RELEASE=${WINUX_VERSION}
DISTRIB_CODENAME=${WINUX_CODENAME}
DISTRIB_DESCRIPTION="Winux OS ${WINUX_VERSION} (${WINUX_CODENAME})"
BUILD_DATE=$(date +%Y-%m-%d)
BUILD_TIMESTAMP=${BUILD_TIMESTAMP}
EOF

    cat > "${CHROOT_DIR}/etc/lsb-release" << EOF
DISTRIB_ID=Winux
DISTRIB_RELEASE=${WINUX_VERSION}
DISTRIB_CODENAME=${WINUX_CODENAME}
DISTRIB_DESCRIPTION="Winux OS ${WINUX_VERSION} (${WINUX_CODENAME})"
EOF

    log_step "Executando scripts post-install..."

    # Copiar e executar script post-install
    if [[ -f "${PROJECT_ROOT}/build/scripts/post-install.sh" ]]; then
        cp "${PROJECT_ROOT}/build/scripts/post-install.sh" "${CHROOT_DIR}/tmp/"
        run_in_chroot "chmod +x /tmp/post-install.sh && /tmp/post-install.sh" || true
        rm -f "${CHROOT_DIR}/tmp/post-install.sh"
    fi

    log_step "Limpando sistema..."

    run_in_chroot "
apt autoremove -y
apt autoclean
apt clean

# Limpar caches
rm -rf /tmp/*
rm -rf /var/tmp/*
rm -rf /var/cache/apt/archives/*.deb
rm -rf /var/lib/apt/lists/*
rm -f /var/log/*.log
rm -f /var/log/**/*.log
truncate -s 0 /var/log/wtmp
truncate -s 0 /var/log/lastlog

# Remover arquivos de configuracao orfaos
rm -f /etc/resolv.conf
rm -rf /root/.bash_history
rm -rf /home/*/.bash_history
"

    umount_chroot

    log_info "Configuracoes finais concluidas"
}

# -----------------------------------------------------------------------------
# Fase 9: Criar Squashfs
# -----------------------------------------------------------------------------
phase_create_squashfs() {
    log_phase "8 - CRIACAO DO SQUASHFS"

    umount_chroot

    local squashfs_file="${ISO_DIR}/casper/filesystem.squashfs"

    if [[ -f "${squashfs_file}" && "${FORCE_REBUILD:-false}" != "true" ]]; then
        log_warn "Squashfs ja existe. Use FORCE_REBUILD=true para recriar"
        return
    fi

    rm -f "${squashfs_file}"

    log_step "Criando filesystem.squashfs..."
    log_info "Compressao: ${COMPRESSION} (nivel ${COMPRESSION_LEVEL})"
    log_info "Isso pode demorar varios minutos..."

    local comp_opts=""
    case "${COMPRESSION}" in
        zstd)
            comp_opts="-comp zstd -Xcompression-level ${COMPRESSION_LEVEL}"
            ;;
        xz)
            comp_opts="-comp xz -Xbcj x86"
            ;;
        lz4)
            comp_opts="-comp lz4 -Xhc"
            ;;
        gzip)
            comp_opts="-comp gzip"
            ;;
    esac

    mksquashfs "${CHROOT_DIR}" "${squashfs_file}" \
        ${comp_opts} \
        -b 1M \
        -no-duplicates \
        -no-recovery \
        -processors "${PARALLEL_JOBS}" \
        -wildcards \
        -e 'var/cache/apt/archives/*.deb' \
        -e 'var/lib/apt/lists/*' \
        -e 'tmp/*' \
        -e 'var/tmp/*'

    # Criar manifest e tamanho
    log_step "Criando arquivos auxiliares..."

    # filesystem.manifest
    chroot "${CHROOT_DIR}" dpkg-query -W --showformat='${Package} ${Version}\n' \
        > "${ISO_DIR}/casper/filesystem.manifest" 2>/dev/null || true

    # filesystem.size
    du -sx --block-size=1 "${CHROOT_DIR}" | cut -f1 > "${ISO_DIR}/casper/filesystem.size"

    local size=$(ls -lh "${squashfs_file}" | awk '{print $5}')
    log_info "Squashfs criado: ${squashfs_file} (${size})"
}

# -----------------------------------------------------------------------------
# Fase 10: Preparar Boot
# -----------------------------------------------------------------------------
phase_prepare_boot() {
    log_phase "9 - PREPARACAO DO BOOT"

    log_step "Copiando kernel e initrd..."

    # Encontrar kernel mais recente
    local kernel=$(ls -t "${CHROOT_DIR}"/boot/vmlinuz-* 2>/dev/null | head -1)
    local initrd=$(ls -t "${CHROOT_DIR}"/boot/initrd.img-* 2>/dev/null | head -1)

    if [[ -z "${kernel}" || -z "${initrd}" ]]; then
        log_error "Kernel ou initrd nao encontrado"
        exit 1
    fi

    cp "${kernel}" "${ISO_DIR}/casper/vmlinuz"
    cp "${initrd}" "${ISO_DIR}/casper/initrd"

    log_info "Kernel: $(basename ${kernel})"
    log_info "Initrd: $(basename ${initrd})"

    log_step "Configurando GRUB..."

    # GRUB config
    cat > "${ISO_DIR}/boot/grub/grub.cfg" << 'EOF'
# Winux OS GRUB Configuration

set timeout=10
set default=0

# Carrega modulos necessarios
insmod all_video
insmod gfxterm
insmod png

# Configuracao de video
if loadfont /boot/grub/fonts/unicode.pf2; then
    set gfxmode=auto
    set gfxpayload=keep
    terminal_output gfxterm
fi

# Menu de boot
menuentry "Winux OS - Iniciar Sessao Live" --class winux --class gnu-linux --class os {
    set gfxpayload=keep
    linux /casper/vmlinuz boot=casper quiet splash ---
    initrd /casper/initrd
}

menuentry "Winux OS - Instalar no Disco" --class winux --class gnu-linux --class os {
    set gfxpayload=keep
    linux /casper/vmlinuz boot=casper only-ubiquity quiet splash ---
    initrd /casper/initrd
}

menuentry "Winux OS - Modo Seguro (nomodeset)" --class winux --class gnu-linux --class os {
    linux /casper/vmlinuz boot=casper nomodeset ---
    initrd /casper/initrd
}

menuentry "Winux OS - Modo de Compatibilidade" --class winux --class gnu-linux --class os {
    linux /casper/vmlinuz boot=casper acpi=off noapic nolapic ---
    initrd /casper/initrd
}

menuentry "Verificar Integridade do Disco" --class winux {
    linux /casper/vmlinuz boot=casper integrity-check ---
    initrd /casper/initrd
}

menuentry "Teste de Memoria (memtest86+)" --class memtest {
    linux16 /boot/memtest86+.bin
}

menuentry "Boot do Disco Rigido" --class hd {
    chainloader +1
}
EOF

    log_step "Configurando ISOLINUX (BIOS)..."

    # ISOLINUX para BIOS
    cp /usr/lib/ISOLINUX/isolinux.bin "${ISO_DIR}/isolinux/" 2>/dev/null || true
    cp /usr/lib/syslinux/modules/bios/*.c32 "${ISO_DIR}/isolinux/" 2>/dev/null || true

    cat > "${ISO_DIR}/isolinux/isolinux.cfg" << EOF
UI vesamenu.c32
TIMEOUT 100
PROMPT 0
DEFAULT live

MENU TITLE Winux OS ${WINUX_VERSION}
MENU BACKGROUND splash.png
MENU COLOR border       30;44   #40ffffff #a0000000 std
MENU COLOR title        1;36;44 #9033ccff #a0000000 std
MENU COLOR sel          7;37;40 #e0ffffff #20ffffff all
MENU COLOR unsel        37;44   #50ffffff #a0000000 std
MENU COLOR help         37;40   #c0ffffff #a0000000 std

LABEL live
    MENU LABEL ^Iniciar Winux OS - Sessao Live
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper quiet splash ---

LABEL install
    MENU LABEL ^Instalar Winux OS
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper only-ubiquity quiet splash ---

LABEL safe
    MENU LABEL Modo ^Seguro (nomodeset)
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper nomodeset ---

LABEL check
    MENU LABEL ^Verificar disco
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper integrity-check ---
EOF

    log_step "Criando arquivos de informacao..."

    # .disk/info
    echo "Winux OS ${WINUX_VERSION} \"${WINUX_CODENAME}\" - $(date +%Y-%m-%d)" > "${ISO_DIR}/.disk/info"
    echo "live" > "${ISO_DIR}/.disk/cd_type"
    touch "${ISO_DIR}/.disk/base_installable"
    echo "https://github.com/winux-os" > "${ISO_DIR}/.disk/release_notes_url"

    log_info "Boot configurado"
}

# -----------------------------------------------------------------------------
# Fase 11: Gerar ISO
# -----------------------------------------------------------------------------
phase_create_iso() {
    log_phase "10 - GERACAO DA ISO"

    local iso_name="winux-${WINUX_VERSION}-${WINUX_CODENAME}-amd64.iso"
    local iso_path="${OUTPUT_DIR}/${iso_name}"

    log_step "Criando imagem EFI..."

    # Criar EFI boot image
    grub-mkstandalone \
        --format=x86_64-efi \
        --output="${ISO_DIR}/EFI/BOOT/BOOTx64.EFI" \
        --locales="" \
        --fonts="" \
        "boot/grub/grub.cfg=${ISO_DIR}/boot/grub/grub.cfg"

    # Criar imagem FAT para EFI
    local efi_img="${ISO_DIR}/boot/grub/efi.img"
    dd if=/dev/zero of="${efi_img}" bs=1M count=10
    mkfs.vfat "${efi_img}"
    mmd -i "${efi_img}" efi efi/boot
    mcopy -i "${efi_img}" "${ISO_DIR}/EFI/BOOT/BOOTx64.EFI" ::efi/boot/

    log_step "Criando imagem BIOS..."

    # BIOS boot
    grub-mkstandalone \
        --format=i386-pc \
        --output="${ISO_DIR}/boot/grub/bios.img" \
        --install-modules="linux normal iso9660 biosdisk memdisk search tar ls" \
        --modules="linux normal iso9660 biosdisk search" \
        --locales="" \
        --fonts="" \
        "boot/grub/grub.cfg=${ISO_DIR}/boot/grub/grub.cfg"

    cat /usr/lib/grub/i386-pc/cdboot.img "${ISO_DIR}/boot/grub/bios.img" \
        > "${ISO_DIR}/boot/grub/bios_boot.img"

    log_step "Gerando ISO hibrida (UEFI + BIOS)..."
    log_info "Output: ${iso_path}"

    xorriso -as mkisofs \
        -iso-level 3 \
        -full-iso9660-filenames \
        -volid "WINUX_${WINUX_VERSION//./_}" \
        -J -joliet-long \
        -rational-rock \
        -eltorito-boot boot/grub/bios_boot.img \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        --eltorito-catalog boot/grub/boot.cat \
        --grub2-boot-info \
        --grub2-mbr /usr/lib/grub/i386-pc/boot_hybrid.img \
        -eltorito-alt-boot \
        -e boot/grub/efi.img \
        -no-emul-boot \
        -append_partition 2 0xef "${efi_img}" \
        -isohybrid-gpt-basdat \
        -output "${iso_path}" \
        "${ISO_DIR}"

    log_info "ISO criada com sucesso!"
}

# -----------------------------------------------------------------------------
# Fase 12: Checksums
# -----------------------------------------------------------------------------
phase_checksums() {
    log_phase "11 - CHECKSUMS"

    local iso_name="winux-${WINUX_VERSION}-${WINUX_CODENAME}-amd64.iso"
    local iso_path="${OUTPUT_DIR}/${iso_name}"

    if [[ ! -f "${iso_path}" ]]; then
        log_error "ISO nao encontrada: ${iso_path}"
        return 1
    fi

    log_step "Calculando checksums..."

    cd "${OUTPUT_DIR}"

    # MD5
    md5sum "${iso_name}" > "${iso_name}.md5"
    log_info "MD5: $(cat ${iso_name}.md5)"

    # SHA256
    sha256sum "${iso_name}" > "${iso_name}.sha256"
    log_info "SHA256: $(cat ${iso_name}.sha256)"

    # SHA512
    sha512sum "${iso_name}" > "${iso_name}.sha512"

    # Criar arquivo de release
    local size=$(ls -lh "${iso_path}" | awk '{print $5}')
    local size_bytes=$(stat -c%s "${iso_path}")

    cat > "${OUTPUT_DIR}/${iso_name}.info" << EOF
Winux OS ${WINUX_VERSION} (${WINUX_CODENAME})
==========================================

Arquivo: ${iso_name}
Tamanho: ${size} (${size_bytes} bytes)
Data: $(date +"%Y-%m-%d %H:%M:%S")

Checksums:
----------
MD5:    $(cat ${iso_name}.md5 | awk '{print $1}')
SHA256: $(cat ${iso_name}.sha256 | awk '{print $1}')
SHA512: $(cat ${iso_name}.sha512 | awk '{print $1}')

Como verificar:
---------------
md5sum -c ${iso_name}.md5
sha256sum -c ${iso_name}.sha256

Gravar em USB:
--------------
sudo dd if=${iso_name} of=/dev/sdX bs=4M status=progress oflag=sync

Ou use ferramentas como:
- balenaEtcher
- Ventoy
- Rufus (Windows)
EOF

    log_info "Checksums gerados"
    log_info "Arquivos em: ${OUTPUT_DIR}"
}

# -----------------------------------------------------------------------------
# Cleanup
# -----------------------------------------------------------------------------
cleanup() {
    log_step "Executando limpeza..."

    umount_chroot 2>/dev/null || true

    if [[ "${CLEANUP:-false}" == "true" ]]; then
        log_info "Removendo arquivos temporarios..."
        rm -rf "${BUILD_DIR}"
    fi

    log_info "Cleanup concluido"
}

# -----------------------------------------------------------------------------
# Banner
# -----------------------------------------------------------------------------
show_banner() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                              ║${NC}"
    echo -e "${CYAN}║   ${BOLD}██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗${NC}${CYAN}                ║${NC}"
    echo -e "${CYAN}║   ${BOLD}██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝${NC}${CYAN}                ║${NC}"
    echo -e "${CYAN}║   ${BOLD}██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝${NC}${CYAN}                 ║${NC}"
    echo -e "${CYAN}║   ${BOLD}██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗${NC}${CYAN}                 ║${NC}"
    echo -e "${CYAN}║   ${BOLD}╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗${NC}${CYAN}               ║${NC}"
    echo -e "${CYAN}║   ${BOLD} ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝${NC}${CYAN}               ║${NC}"
    echo -e "${CYAN}║                                                              ║${NC}"
    echo -e "${CYAN}║              ${BOLD}ISO BUILD SYSTEM v2.0${NC}${CYAN}                        ║${NC}"
    echo -e "${CYAN}║              Versao: ${WINUX_VERSION} (${WINUX_CODENAME})${CYAN}                          ║${NC}"
    echo -e "${CYAN}║                                                              ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

show_help() {
    echo "Uso: $0 [COMANDO] [OPCOES]"
    echo ""
    echo "Comandos:"
    echo "  all           Executar todas as fases (padrao)"
    echo "  prepare       Fase 1: Preparar ambiente"
    echo "  debootstrap   Fase 2: Criar sistema base"
    echo "  configure     Fase 3: Configurar sistema base"
    echo "  rust-apps     Fase 4: Compilar apps Rust"
    echo "  theme         Fase 5: Instalar tema"
    echo "  calamares     Fase 6: Configurar Calamares"
    echo "  finalize      Fase 7: Configuracoes finais"
    echo "  squashfs      Fase 8: Criar squashfs"
    echo "  boot          Fase 9: Preparar boot"
    echo "  iso           Fase 10: Gerar ISO"
    echo "  checksums     Fase 11: Gerar checksums"
    echo "  clean         Limpar arquivos de build"
    echo ""
    echo "Opcoes (variaveis de ambiente):"
    echo "  BUILD_DIR          Diretorio de build (default: /tmp/winux-build)"
    echo "  OUTPUT_DIR         Diretorio de saida (default: ./output)"
    echo "  COMPRESSION        Tipo de compressao: zstd, xz, lz4, gzip (default: zstd)"
    echo "  COMPRESSION_LEVEL  Nivel de compressao (default: 19)"
    echo "  PARALLEL_JOBS      Numero de jobs paralelos (default: nproc)"
    echo "  QUICK_BUILD        Build rapido para testes (default: false)"
    echo "  FORCE_REBUILD      Forcar reconstrucao (default: false)"
    echo "  CLEANUP            Limpar apos build (default: false)"
    echo "  DEBUG              Modo debug verbose (default: false)"
    echo ""
    echo "Exemplos:"
    echo "  sudo $0 all"
    echo "  sudo QUICK_BUILD=true $0 all"
    echo "  sudo COMPRESSION=lz4 $0 squashfs"
    echo "  sudo OUTPUT_DIR=/mnt/iso $0 iso"
    echo ""
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    show_banner

    case "${1:-all}" in
        -h|--help|help)
            show_help
            exit 0
            ;;
        prepare)
            check_root
            check_system
            check_deps
            phase_prepare
            ;;
        debootstrap)
            check_root
            phase_debootstrap
            ;;
        configure)
            check_root
            phase_configure_base
            ;;
        rust-apps)
            check_root
            phase_install_rust_apps
            ;;
        theme)
            check_root
            phase_install_theme
            ;;
        calamares)
            check_root
            phase_configure_calamares
            ;;
        finalize)
            check_root
            phase_final_config
            ;;
        squashfs)
            check_root
            phase_create_squashfs
            ;;
        boot)
            check_root
            phase_prepare_boot
            ;;
        iso)
            check_root
            phase_create_iso
            ;;
        checksums)
            check_root
            phase_checksums
            ;;
        all)
            check_root
            check_system
            check_deps
            phase_prepare
            phase_debootstrap
            phase_configure_base
            phase_install_rust_apps
            phase_install_theme
            phase_configure_calamares
            phase_final_config
            phase_create_squashfs
            phase_prepare_boot
            phase_create_iso
            phase_checksums
            cleanup
            ;;
        clean)
            check_root
            CLEANUP=true cleanup
            ;;
        *)
            log_error "Comando desconhecido: $1"
            show_help
            exit 1
            ;;
    esac

    echo ""
    log_info "============================================"
    log_info "Build concluido com sucesso!"
    log_info "============================================"
    echo ""
}

trap cleanup EXIT
main "$@"
