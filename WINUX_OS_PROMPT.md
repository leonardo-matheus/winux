# 🐧 WINUX OS - Documento de Especificação Técnica Completa

> **Prompt de Instrução para Claude Opus 4.5**
> Versão: 1.0.0 | Data: Fevereiro 2026
> Classificação: Documento Técnico de Desenvolvimento

---

## 📋 Índice

1. [Visão Geral e Filosofia](#1-visão-geral-e-filosofia)
2. [Especificações Técnicas Base](#2-especificações-técnicas-base)
3. [Camada de Compatibilidade Windows](#3-camada-de-compatibilidade-windows)
4. [Sistema de Drivers](#4-sistema-de-drivers)
5. [Winux Shell - Desktop Environment](#5-winux-shell---desktop-environment)
6. [Suite de Aplicações Rust](#6-suite-de-aplicações-rust)
7. [Otimizações de Performance](#7-otimizações-de-performance)
8. [Sistema de Build](#8-sistema-de-build)
9. [Roadmap de Desenvolvimento](#9-roadmap-de-desenvolvimento)
10. [Anexos Técnicos](#10-anexos-técnicos)

---

## 1. Visão Geral e Filosofia

### 1.1 Identidade do Projeto

```
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║   ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗                      ║
║   ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝                      ║
║   ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝                       ║
║   ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗                       ║
║   ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗                      ║
║    ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝                      ║
║                                                                  ║
║   "O Melhor dos Dois Mundos"                                    ║
║   Gaming + Produtividade | Linux + Windows Experience           ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
```

**Nome:** Winux
**Tagline:** "O Melhor dos Dois Mundos"
**Versão Inicial:** 1.0 "Aurora"
**Base:** Ubuntu 24.04 LTS (Noble Numbat)
**Licença:** GPL v3 + MIT (aplicações próprias)

### 1.2 Filosofia de Design

O Winux é construído sobre três pilares fundamentais:

| Pilar | Descrição | Implementação |
|-------|-----------|---------------|
| **Familiaridade** | Interface que usuários Windows reconhecem instantaneamente | Winux Shell com design Fluent |
| **Performance** | Sistema otimizado para gaming e cargas de trabalho intensivas | Kernel zen + tunables agressivos |
| **Liberdade** | Todo o poder do Linux com compatibilidade Windows | Wine/Proton integrados nativamente |

### 1.3 Público-Alvo

```yaml
Perfis de Usuário:
  Gamer:
    - Jogadores de títulos AAA
    - Entusiastas de emulação
    - Streamers e criadores de conteúdo
    - Jogadores competitivos (baixa latência)

  Profissional:
    - Desenvolvedores de software
    - Designers e artistas digitais
    - Editores de vídeo/áudio
    - Profissionais de TI

  Híbrido:
    - Usuários que trabalham e jogam no mesmo sistema
    - Migrantes do Windows buscando alternativa
    - Entusiastas de tecnologia
```

### 1.4 Proposta de Valor

```
┌─────────────────────────────────────────────────────────────────┐
│                    WINUX VALUE PROPOSITION                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│  │   WINDOWS    │    │    WINUX     │    │    LINUX     │     │
│  │              │    │              │    │              │     │
│  │ • Interface  │ ─► │ • Interface  │ ◄─ │ • Kernel     │     │
│  │ • .exe apps  │    │   familiar   │    │ • Segurança  │     │
│  │ • Games      │    │ • Todos apps │    │ • Performance│     │
│  │              │    │ • Gaming     │    │ • Liberdade  │     │
│  │              │    │ • Dev tools  │    │              │     │
│  └──────────────┘    └──────────────┘    └──────────────┘     │
│                                                                 │
│  "Combine a familiaridade do Windows com o poder do Linux"     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Especificações Técnicas Base

### 2.1 Requisitos de Sistema

```yaml
Requisitos Mínimos:
  CPU: x86_64 com suporte SSE4.2
  RAM: 4 GB
  Armazenamento: 30 GB (SSD recomendado)
  GPU: Vulkan 1.1 compatível
  UEFI: Recomendado (Legacy BIOS suportado)

Requisitos Recomendados:
  CPU: AMD Ryzen 5 / Intel Core i5 (6+ cores)
  RAM: 16 GB DDR4/DDR5
  Armazenamento: 100 GB NVMe SSD
  GPU: NVIDIA RTX 3060 / AMD RX 6700 XT ou superior
  UEFI: Secure Boot compatível
```

### 2.2 Kernel Customizado

```bash
# Especificações do Kernel Winux
KERNEL_BASE="linux-zen"
KERNEL_VERSION="6.8.x"
KERNEL_CODENAME="winux-zen"

# Patches aplicados:
patches=(
    "futex-waitv"           # Melhor performance em jogos
    "winesync"              # Sincronização Wine otimizada
    "bore-scheduler"        # Scheduler otimizado para desktop
    "clear-linux-patches"   # Patches de performance Intel
    "graysky-cpu-opts"      # Otimizações por família de CPU
    "acs-override"          # VFIO passthrough
    "bbr3"                  # TCP congestion control
)
```

**Configuração do Kernel (`/etc/winux/kernel.conf`):**

```ini
# Winux Kernel Configuration
[scheduler]
# BORE Scheduler - melhor responsividade
sched_bore=1
sched_burst_penalty_scale=1250

[preemption]
# Preempção completa para baixa latência
preempt=full
hz=1000

[memory]
# Transparent Huge Pages para performance
transparent_hugepage=madvise
compaction_proactiveness=0

[io]
# Multi-queue I/O para NVMe
scsi_mod.use_blk_mq=1
nvme_core.default_ps_max_latency_us=0

[graphics]
# Otimizações gráficas
nvidia-drm.modeset=1
amdgpu.ppfeaturemask=0xffffffff
i915.enable_guc=3

[security]
# Mitigações desabilitadas para performance (gaming)
mitigations=off
```

### 2.3 Sistema de Arquivos

```bash
# Configuração Btrfs Padrão
FILESYSTEM="btrfs"
COMPRESSION="zstd:3"
MOUNT_OPTIONS="noatime,compress=zstd:3,space_cache=v2,discard=async"

# Layout de Subvolumes
subvolumes=(
    "@"           # /
    "@home"       # /home
    "@snapshots"  # /.snapshots
    "@var_log"    # /var/log
    "@var_cache"  # /var/cache
    "@games"      # /games (sem COW para performance)
)
```

**Estrutura de Particionamento:**

```
┌─────────────────────────────────────────────────────────────┐
│                    DISCO (GPT)                              │
├─────────────────────────────────────────────────────────────┤
│ Partição 1: ESP (/boot/efi)                                │
│   - Tamanho: 512 MB                                        │
│   - Filesystem: FAT32                                      │
│   - Flags: boot, esp                                       │
├─────────────────────────────────────────────────────────────┤
│ Partição 2: Root (/)                                       │
│   - Tamanho: Restante do disco                             │
│   - Filesystem: Btrfs                                      │
│   - Subvolumes: @, @home, @snapshots, @var_log, @games     │
├─────────────────────────────────────────────────────────────┤
│ [Opcional] Partição 3: Swap                                │
│   - Tamanho: RAM size ou arquivo swap                      │
│   - Tipo: Linux swap ou swapfile em Btrfs                  │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 Init System e Serviços

```bash
# systemd otimizado para desktop
INIT_SYSTEM="systemd"

# Serviços habilitados por padrão
enabled_services=(
    "NetworkManager"
    "bluetooth"
    "pipewire"
    "pipewire-pulse"
    "winux-session"
    "fstrim.timer"
    "thermald"
)

# Serviços desabilitados para performance
disabled_services=(
    "apt-daily.timer"
    "apt-daily-upgrade.timer"
    "ModemManager"
    "cups-browsed"
)
```

**Configuração systemd (`/etc/systemd/system.conf.d/winux.conf`):**

```ini
[Manager]
# Otimizações de boot
DefaultTimeoutStartSec=15s
DefaultTimeoutStopSec=10s

# Limites de recursos
DefaultLimitNOFILE=1048576
DefaultLimitNPROC=65535

# CPU Affinity para serviços de sistema
CPUAffinity=0-1
```

### 2.5 Bootloader

```bash
# Configuração systemd-boot (UEFI)
BOOTLOADER="systemd-boot"

# /boot/loader/loader.conf
timeout 3
default winux.conf
console-mode max
editor no

# /boot/loader/entries/winux.conf
title   Winux OS
linux   /vmlinuz-winux-zen
initrd  /initramfs-winux-zen.img
options root=PARTUUID=xxx rootflags=subvol=@ rw quiet splash \
        nvidia-drm.modeset=1 mitigations=off
```

---

## 3. Camada de Compatibilidade Windows

### 3.1 Arquitetura de Compatibilidade

```
┌─────────────────────────────────────────────────────────────────┐
│                 WINUX COMPATIBILITY LAYER                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   Native    │  │   Steam     │  │  Standalone │            │
│  │   Linux     │  │   Games     │  │  .exe Apps  │            │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘            │
│         │                │                │                    │
│         ▼                ▼                ▼                    │
│  ┌─────────────────────────────────────────────────┐          │
│  │              WINUX RUNTIME MANAGER              │          │
│  │  (Detecta e roteia para runtime apropriado)     │          │
│  └─────────────────────────────────────────────────┘          │
│         │                │                │                    │
│         ▼                ▼                ▼                    │
│  ┌───────────┐    ┌───────────┐    ┌───────────┐             │
│  │  Native   │    │  Proton   │    │   Wine    │             │
│  │  Runtime  │    │    GE     │    │  Staging  │             │
│  └───────────┘    └───────────┘    └───────────┘             │
│                          │                │                    │
│                          ▼                ▼                    │
│              ┌─────────────────────────────────┐              │
│              │     DXVK / VKD3D-Proton        │              │
│              │   DirectX 9/10/11/12 → Vulkan  │              │
│              └─────────────────────────────────┘              │
│                                                                │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Wine Configuration

```bash
# Versão e configuração Wine
WINE_VERSION="wine-staging-9.x"
WINE_PREFIX_DEFAULT="$HOME/.wine"
WINE_PREFIX_GAMES="$HOME/.wine-games"

# Instalação automatizada
apt install -y \
    wine-staging \
    wine-staging-amd64 \
    wine-staging-i386:amd64 \
    winetricks \
    winbind
```

**Script de Configuração Wine (`/usr/lib/winux/wine-setup.sh`):**

```bash
#!/bin/bash
# Winux Wine Auto-Configuration

setup_wine_prefix() {
    local prefix="${1:-$HOME/.wine}"

    export WINEPREFIX="$prefix"
    export WINEARCH="win64"

    # Criar prefix
    wineboot --init

    # Instalar componentes essenciais
    winetricks -q \
        corefonts \
        vcrun2019 \
        vcrun2022 \
        dotnet48 \
        dxvk \
        vkd3d \
        d3dcompiler_47 \
        win10

    # Configurar DXVK
    setup_dxvk "$prefix"

    # Configurar VKD3D-Proton
    setup_vkd3d "$prefix"
}

setup_dxvk() {
    local prefix="$1"
    local dxvk_version="2.4"

    # Download e extração
    wget "https://github.com/doitsujin/dxvk/releases/download/v${dxvk_version}/dxvk-${dxvk_version}.tar.gz"
    tar xf "dxvk-${dxvk_version}.tar.gz"

    # Instalação
    cd "dxvk-${dxvk_version}"
    WINEPREFIX="$prefix" ./setup_dxvk.sh install
}

setup_vkd3d() {
    local prefix="$1"
    local vkd3d_version="2.12"

    wget "https://github.com/HansKristian-Work/vkd3d-proton/releases/download/v${vkd3d_version}/vkd3d-proton-${vkd3d_version}.tar.zst"
    tar --zstd -xf "vkd3d-proton-${vkd3d_version}.tar.zst"

    cd "vkd3d-proton-${vkd3d_version}"
    WINEPREFIX="$prefix" ./setup_vkd3d_proton.sh install
}
```

### 3.3 Proton-GE Integration

```bash
# Instalação Proton-GE para Steam
PROTON_GE_VERSION="GE-Proton9-7"
PROTON_PATH="$HOME/.steam/root/compatibilitytools.d"

install_proton_ge() {
    mkdir -p "$PROTON_PATH"

    wget "https://github.com/GloriousEggroll/proton-ge-custom/releases/download/${PROTON_GE_VERSION}/${PROTON_GE_VERSION}.tar.gz"
    tar xf "${PROTON_GE_VERSION}.tar.gz" -C "$PROTON_PATH"

    echo "Proton-GE instalado em: $PROTON_PATH/$PROTON_GE_VERSION"
}
```

### 3.4 Associação Automática de .exe

**Desktop Entry (`/usr/share/applications/winux-wine.desktop`):**

```ini
[Desktop Entry]
Name=Winux Wine Launcher
Comment=Execute Windows applications
Exec=/usr/bin/winux-run %f
Icon=wine
Terminal=false
Type=Application
MimeType=application/x-ms-dos-executable;application/x-msdos-program;application/x-msdownload;
Categories=System;
NoDisplay=true
```

**Launcher Script (`/usr/bin/winux-run`):**

```bash
#!/bin/bash
# Winux Universal Windows Application Launcher

APP="$1"
APP_DIR=$(dirname "$APP")
APP_NAME=$(basename "$APP" .exe)

# Detectar tipo de aplicação
detect_app_type() {
    local exe="$1"

    # Verificar se é jogo Steam
    if [[ "$exe" == *"steamapps"* ]]; then
        echo "steam"
        return
    fi

    # Verificar se é jogo conhecido (database)
    if grep -q "$APP_NAME" /usr/share/winux/games-db.txt; then
        echo "game"
        return
    fi

    echo "app"
}

# Selecionar runtime
select_runtime() {
    local type="$1"

    case "$type" in
        steam)
            echo "proton"
            ;;
        game)
            echo "wine-gaming"
            ;;
        *)
            echo "wine-default"
            ;;
    esac
}

# Configurar ambiente
setup_environment() {
    local runtime="$1"

    case "$runtime" in
        proton)
            # Steam cuida do runtime
            ;;
        wine-gaming)
            export WINEPREFIX="$HOME/.wine-games"
            export WINEDLLOVERRIDES="dxgi,d3d9,d3d10core,d3d11=n,b"
            export DXVK_ASYNC=1
            export WINE_FULLSCREEN_FSR=1
            export MANGOHUD=1
            ;;
        wine-default)
            export WINEPREFIX="$HOME/.wine"
            ;;
    esac

    # Comum a todos
    export WINE_LARGE_ADDRESS_AWARE=1
    export __GL_SHADER_DISK_CACHE=1
    export __GL_SHADER_DISK_CACHE_PATH="$HOME/.cache/nvidia/GLCache"
    export mesa_glthread=true
}

# Executar
run_app() {
    local app="$1"
    local runtime="$2"

    cd "$APP_DIR"

    case "$runtime" in
        proton)
            steam steam://run/$(get_steam_appid "$app")
            ;;
        *)
            wine "$app"
            ;;
    esac
}

# Main
TYPE=$(detect_app_type "$APP")
RUNTIME=$(select_runtime "$TYPE")
setup_environment "$RUNTIME"
run_app "$APP" "$RUNTIME"
```

### 3.5 Registro Windows Virtualizado

```bash
# Estrutura do registro Wine
WINE_REGISTRY_FILES=(
    "system.reg"    # HKEY_LOCAL_MACHINE
    "user.reg"      # HKEY_CURRENT_USER
    "userdef.reg"   # HKEY_USERS\.Default
)

# Tweaks de registro para gaming
apply_gaming_registry_tweaks() {
    cat >> "$WINEPREFIX/user.reg" << 'EOF'
[Software\\Wine\\Direct3D]
"csmt"=dword:3
"MaxFrameLatency"=dword:1
"UseGLSL"="enabled"
"DirectDrawRenderer"="opengl"
"OffscreenRenderingMode"="fbo"
"VideoMemorySize"="8192"

[Software\\Wine\\DllOverrides]
"*d3d9"="native,builtin"
"*d3d10core"="native,builtin"
"*d3d11"="native,builtin"
"*dxgi"="native,builtin"

[Software\\Wine\\X11 Driver]
"GrabFullscreen"="Y"
"UseTakeFocus"="N"
EOF
}
```

---

## 4. Sistema de Drivers

### 4.1 Winux Hardware Detection (WHD)

```
┌─────────────────────────────────────────────────────────────────┐
│              WINUX HARDWARE DETECTION (WHD)                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────┐           │
│  │            Hardware Detection Module            │           │
│  │     (lspci, lsusb, udevadm, dmidecode)         │           │
│  └─────────────────────────────────────────────────┘           │
│                          │                                      │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────┐           │
│  │           Driver Database Matcher               │           │
│  │        /usr/share/winux/drivers-db/            │           │
│  └─────────────────────────────────────────────────┘           │
│                          │                                      │
│         ┌────────────────┼────────────────┐                    │
│         ▼                ▼                ▼                    │
│  ┌───────────┐    ┌───────────┐    ┌───────────┐             │
│  │  NVIDIA   │    │    AMD    │    │   Intel   │             │
│  │  Handler  │    │  Handler  │    │  Handler  │             │
│  └───────────┘    └───────────┘    └───────────┘             │
│                                                                │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 NVIDIA Driver Stack

```bash
#!/bin/bash
# /usr/lib/winux/drivers/nvidia-install.sh

install_nvidia() {
    local driver_version="${1:-550}"

    # Adicionar PPA NVIDIA
    add-apt-repository -y ppa:graphics-drivers/ppa
    apt update

    # Instalar driver e componentes
    apt install -y \
        "nvidia-driver-${driver_version}" \
        "nvidia-dkms-${driver_version}" \
        "nvidia-utils-${driver_version}" \
        "libnvidia-gl-${driver_version}:amd64" \
        "libnvidia-gl-${driver_version}:i386" \
        nvidia-settings \
        nvidia-prime \
        libvulkan1 \
        vulkan-tools

    # CUDA (opcional para compute)
    apt install -y \
        "nvidia-cuda-toolkit" \
        "nvidia-cuda-dev"

    # Configurar módulos
    configure_nvidia_modules

    # Configurar Xorg/Wayland
    configure_nvidia_display
}

configure_nvidia_modules() {
    cat > /etc/modprobe.d/nvidia-winux.conf << 'EOF'
# Winux NVIDIA Configuration
options nvidia-drm modeset=1
options nvidia NVreg_UsePageAttributeTable=1
options nvidia NVreg_EnablePCIeGen3=1
options nvidia NVreg_PreserveVideoMemoryAllocations=1
options nvidia NVreg_TemporaryFilePath=/var/tmp
blacklist nouveau
EOF

    # Regenerar initramfs
    update-initramfs -u
}

configure_nvidia_display() {
    # Forçar composição pipeline
    nvidia-settings --assign CurrentMetaMode="nvidia-auto-select +0+0 { ForceCompositionPipeline = On }"

    # Configurar para Wayland (se suportado)
    if [[ -f /etc/gdm3/custom.conf ]]; then
        sed -i 's/#WaylandEnable=false/WaylandEnable=true/' /etc/gdm3/custom.conf
    fi

    # Variáveis de ambiente
    cat > /etc/environment.d/nvidia-winux.conf << 'EOF'
# NVIDIA Wayland
GBM_BACKEND=nvidia-drm
__GLX_VENDOR_LIBRARY_NAME=nvidia
WLR_NO_HARDWARE_CURSORS=1
LIBVA_DRIVER_NAME=nvidia
EOF
}
```

### 4.3 AMD Driver Stack

```bash
#!/bin/bash
# /usr/lib/winux/drivers/amd-install.sh

install_amd() {
    # Mesa (open source - recomendado)
    apt install -y \
        mesa-vulkan-drivers \
        mesa-vulkan-drivers:i386 \
        libgl1-mesa-dri:amd64 \
        libgl1-mesa-dri:i386 \
        mesa-utils \
        vulkan-tools \
        radeontop

    # AMDVLK (alternativo)
    install_amdvlk

    # ROCm (compute - opcional)
    install_rocm
}

install_amdvlk() {
    wget https://github.com/GPUOpen-Drivers/AMDVLK/releases/latest/download/amdvlk_*.deb
    dpkg -i amdvlk_*.deb
    apt install -f -y
}

install_rocm() {
    # Adicionar repositório ROCm
    wget -q -O - https://repo.radeon.com/rocm/rocm.gpg.key | apt-key add -
    echo 'deb [arch=amd64] https://repo.radeon.com/rocm/apt/latest focal main' > /etc/apt/sources.list.d/rocm.list
    apt update

    apt install -y rocm-hip-libraries rocm-opencl-runtime
}

configure_amd() {
    # Configurações de kernel
    cat > /etc/modprobe.d/amdgpu-winux.conf << 'EOF'
# Winux AMD Configuration
options amdgpu ppfeaturemask=0xffffffff
options amdgpu gpu_recovery=1
options amdgpu deep_color=1
options amdgpu dc=1
EOF

    # Variáveis de ambiente
    cat > /etc/environment.d/amd-winux.conf << 'EOF'
# AMD Performance
RADV_PERFTEST=gpl
AMD_VULKAN_ICD=RADV
MESA_DISK_CACHE_SINGLE_FILE=1
mesa_glthread=true
EOF

    update-initramfs -u
}
```

### 4.4 Intel Driver Stack

```bash
#!/bin/bash
# /usr/lib/winux/drivers/intel-install.sh

install_intel() {
    apt install -y \
        mesa-vulkan-drivers \
        mesa-vulkan-drivers:i386 \
        intel-media-va-driver \
        intel-gpu-tools \
        vulkan-tools

    # Arc Graphics (DG2)
    if lspci | grep -q "Arc"; then
        install_intel_arc
    fi
}

install_intel_arc() {
    # Repositório Intel
    wget -qO - https://repositories.intel.com/graphics/intel-graphics.key | apt-key add -
    echo 'deb [arch=amd64] https://repositories.intel.com/graphics/ubuntu jammy main' > /etc/apt/sources.list.d/intel-graphics.list
    apt update

    apt install -y \
        intel-opencl-icd \
        intel-level-zero-gpu \
        level-zero
}

configure_intel() {
    cat > /etc/environment.d/intel-winux.conf << 'EOF'
# Intel Performance
ANV_QUEUE_THREAD_DISABLE=1
INTEL_DEBUG=norbc
mesa_glthread=true
EOF
}
```

### 4.5 WHD Auto-Detection Script

```bash
#!/bin/bash
# /usr/bin/winux-driver-manager

detect_gpu() {
    local gpu_info=$(lspci -nn | grep -E "VGA|3D|Display")

    if echo "$gpu_info" | grep -qi "nvidia"; then
        echo "nvidia"
    elif echo "$gpu_info" | grep -qi "amd\|radeon"; then
        echo "amd"
    elif echo "$gpu_info" | grep -qi "intel"; then
        echo "intel"
    else
        echo "unknown"
    fi
}

get_nvidia_recommended_driver() {
    local gpu_pci_id=$(lspci -nn | grep -i nvidia | grep -oP '\[10de:\K[0-9a-f]+')

    # Database de GPUs
    case "$gpu_pci_id" in
        # RTX 40 series
        2684|2702|2704|2705|2782|2783|2786)
            echo "550"
            ;;
        # RTX 30 series
        2204|2206|2208|220a|2216|2484|2486|2488|2489)
            echo "550"
            ;;
        # GTX 16/RTX 20 series
        1e04|1e07|1e82|1e84|1e87|1e89|1f02|1f06|1f07|1f08)
            echo "535"
            ;;
        # Older cards
        *)
            echo "470"
            ;;
    esac
}

main() {
    echo "╔════════════════════════════════════════════╗"
    echo "║    Winux Hardware Detection (WHD)         ║"
    echo "╚════════════════════════════════════════════╝"
    echo ""

    GPU=$(detect_gpu)
    echo "[INFO] GPU detectada: $GPU"

    case "$GPU" in
        nvidia)
            DRIVER=$(get_nvidia_recommended_driver)
            echo "[INFO] Driver recomendado: nvidia-$DRIVER"
            read -p "Instalar driver NVIDIA $DRIVER? [Y/n] " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
                /usr/lib/winux/drivers/nvidia-install.sh "$DRIVER"
            fi
            ;;
        amd)
            echo "[INFO] Usando drivers Mesa/RADV (open source)"
            /usr/lib/winux/drivers/amd-install.sh
            ;;
        intel)
            echo "[INFO] Usando drivers Mesa/ANV"
            /usr/lib/winux/drivers/intel-install.sh
            ;;
        *)
            echo "[WARN] GPU não reconhecida, usando drivers genéricos"
            ;;
    esac

    echo ""
    echo "[OK] Configuração de drivers concluída!"
    echo "[INFO] Reinicie o sistema para aplicar as mudanças."
}

main "$@"
```

---

## 5. Winux Shell - Desktop Environment

### 5.1 Arquitetura do Desktop

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        WINUX SHELL ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     Winux Shell (Top Level)                      │   │
│  │   • Session Manager   • Settings Daemon   • Notification Daemon │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                  │                                      │
│         ┌────────────────────────┼────────────────────────┐            │
│         ▼                        ▼                        ▼            │
│  ┌─────────────┐    ┌───────────────────────┐    ┌─────────────┐      │
│  │  Panel/Bar  │    │   Window Manager      │    │  Widgets    │      │
│  │  (Taskbar)  │    │   (Tiling + Float)    │    │  (Desktop)  │      │
│  └─────────────┘    └───────────────────────┘    └─────────────┘      │
│                                  │                                      │
│                                  ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Compositor (wlroots)                          │   │
│  │        Wayland Protocol + XWayland Compatibility                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                  │                                      │
│                                  ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Graphics Stack                                │   │
│  │          DRM/KMS → Mesa/NVIDIA → Vulkan/OpenGL                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Compositor Wayland

```rust
// /usr/lib/winux/winux-compositor/src/main.rs
// Baseado em wlroots-rs / smithay

use smithay::{
    backend::renderer::gles2::Gles2Renderer,
    desktop::{Space, Window},
    reexports::wayland_server::Display,
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
    },
};

pub struct WinuxCompositor {
    display: Display<Self>,
    space: Space<Window>,
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
}

impl WinuxCompositor {
    pub fn new() -> Self {
        // Inicialização do compositor
        todo!("Implementar compositor completo")
    }

    pub fn run(&mut self) {
        // Event loop principal
        loop {
            self.display.dispatch_clients().unwrap();
            self.space.refresh();
            self.render();
        }
    }

    fn render(&mut self) {
        // Renderização dos elementos
    }
}

fn main() {
    let mut compositor = WinuxCompositor::new();
    compositor.run();
}
```

**Cargo.toml do Compositor:**

```toml
[package]
name = "winux-compositor"
version = "1.0.0"
edition = "2021"

[dependencies]
smithay = "0.3"
smithay-drm-extras = "0.1"
wayland-server = "0.31"
wayland-protocols = "0.31"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[features]
default = ["wayland", "x11"]
wayland = []
x11 = ["smithay/xwayland"]
```

### 5.3 Tema Visual - Fluent Design

```scss
// /usr/share/winux/themes/winux-fluent/gtk-4.0/gtk.scss

// ========================================
// WINUX FLUENT THEME - GTK4
// Inspirado no Windows 11 Fluent Design
// ========================================

// === Variáveis de Cor ===
$bg-primary: #202020;
$bg-secondary: #2d2d2d;
$bg-tertiary: #383838;
$bg-elevated: #3d3d3d;

$accent-primary: #60cdff;      // Azul Winux
$accent-secondary: #0078d4;
$accent-tertiary: #99ebff;

$text-primary: #ffffff;
$text-secondary: rgba(255, 255, 255, 0.78);
$text-tertiary: rgba(255, 255, 255, 0.54);

$border-color: rgba(255, 255, 255, 0.08);
$border-radius: 8px;

// === Mica Effect ===
@mixin mica-background {
    background-color: rgba($bg-primary, 0.8);
    backdrop-filter: blur(30px) saturate(125%);
}

// === Acrylic Effect ===
@mixin acrylic-background($opacity: 0.7) {
    background-color: rgba($bg-secondary, $opacity);
    backdrop-filter: blur(60px) saturate(150%);
}

// === Janelas ===
window {
    @include mica-background;
    border-radius: $border-radius;
    border: 1px solid $border-color;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

// === Barra de Título ===
headerbar {
    background: transparent;
    border-bottom: 1px solid $border-color;
    min-height: 32px;
    padding: 0 8px;

    .title {
        font-weight: 600;
        font-size: 12px;
        color: $text-primary;
    }

    button {
        min-width: 46px;
        min-height: 32px;
        border: none;
        border-radius: 0;
        background: transparent;

        &:hover {
            background: rgba(255, 255, 255, 0.08);
        }

        // Botão fechar
        &.close:hover {
            background: #c42b1c;
        }
    }
}

// === Botões ===
button {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 4px;
    padding: 5px 12px;
    color: $text-primary;
    transition: all 150ms ease;

    &:hover {
        background: rgba(255, 255, 255, 0.09);
        border-color: rgba(255, 255, 255, 0.12);
    }

    &:active {
        background: rgba(255, 255, 255, 0.04);
    }

    &.suggested-action {
        background: $accent-primary;
        color: #000000;

        &:hover {
            background: lighten($accent-primary, 5%);
        }
    }
}

// === Entradas de Texto ===
entry {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-bottom: 2px solid rgba(255, 255, 255, 0.54);
    border-radius: 4px 4px 0 0;
    padding: 8px 12px;
    color: $text-primary;
    caret-color: $accent-primary;

    &:focus {
        border-bottom-color: $accent-primary;
        background: rgba(255, 255, 255, 0.04);
    }
}
```

### 5.4 Componentes do Shell

#### 5.4.1 Barra de Tarefas (Taskbar)

```rust
// /usr/lib/winux/winux-panel/src/taskbar.rs

use gtk4::{prelude::*, Application, ApplicationWindow, Box, Button, Image};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub struct WinuxTaskbar {
    window: ApplicationWindow,
    app_buttons: Vec<TaskbarButton>,
    system_tray: SystemTray,
    clock: ClockWidget,
}

impl WinuxTaskbar {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Winux Taskbar")
            .default_width(1920)
            .default_height(48)
            .build();

        // Configurar como layer shell
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_exclusive_zone(48);

        Self {
            window,
            app_buttons: Vec::new(),
            system_tray: SystemTray::new(),
            clock: ClockWidget::new(),
        }
    }

    pub fn build_ui(&self) {
        let main_box = Box::new(gtk4::Orientation::Horizontal, 0);
        main_box.add_css_class("taskbar");

        // Menu Iniciar
        let start_button = self.create_start_button();
        main_box.append(&start_button);

        // Apps Pinados
        let pinned_apps = self.create_pinned_apps();
        main_box.append(&pinned_apps);

        // Espaço flexível
        let spacer = Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        main_box.append(&spacer);

        // System Tray
        main_box.append(&self.system_tray.widget());

        // Relógio
        main_box.append(&self.clock.widget());

        // Centro de Ações
        let action_center = self.create_action_center_button();
        main_box.append(&action_center);

        self.window.set_child(Some(&main_box));
    }

    fn create_start_button(&self) -> Button {
        let button = Button::new();
        let icon = Image::from_icon_name("winux-logo");
        icon.set_pixel_size(24);
        button.set_child(Some(&icon));
        button.add_css_class("start-button");

        button.connect_clicked(|_| {
            // Abrir Menu Iniciar
            StartMenu::toggle();
        });

        button
    }

    fn create_pinned_apps(&self) -> Box {
        let apps_box = Box::new(gtk4::Orientation::Horizontal, 4);

        let pinned = vec![
            ("winux-files", "Arquivos"),
            ("winux-terminal", "Terminal"),
            ("firefox", "Firefox"),
            ("steam", "Steam"),
        ];

        for (icon, tooltip) in pinned {
            let btn = Button::new();
            let img = Image::from_icon_name(icon);
            img.set_pixel_size(24);
            btn.set_child(Some(&img));
            btn.set_tooltip_text(Some(tooltip));
            btn.add_css_class("taskbar-button");
            apps_box.append(&btn);
        }

        apps_box
    }
}
```

#### 5.4.2 Menu Iniciar

```rust
// /usr/lib/winux/winux-panel/src/start_menu.rs

use gtk4::{prelude::*, Box, Entry, FlowBox, Label, ScrolledWindow};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub struct StartMenu {
    window: gtk4::Window,
    search_entry: Entry,
    pinned_apps: FlowBox,
    all_apps: FlowBox,
    recent_files: Box,
    matcher: SkimMatcherV2,
}

impl StartMenu {
    pub fn new() -> Self {
        let window = gtk4::Window::builder()
            .title("Start Menu")
            .default_width(600)
            .default_height(700)
            .decorated(false)
            .build();

        window.add_css_class("start-menu");

        Self {
            window,
            search_entry: Entry::new(),
            pinned_apps: FlowBox::new(),
            all_apps: FlowBox::new(),
            recent_files: Box::new(gtk4::Orientation::Vertical, 4),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn build_ui(&self) {
        let main_box = Box::new(gtk4::Orientation::Vertical, 16);
        main_box.set_margin_all(20);

        // Busca
        self.search_entry.set_placeholder_text(Some("Digite para pesquisar"));
        self.search_entry.add_css_class("start-search");
        main_box.append(&self.search_entry);

        // Apps Fixados
        let pinned_label = Label::new(Some("Fixados"));
        pinned_label.add_css_class("section-title");
        main_box.append(&pinned_label);

        self.pinned_apps.set_max_children_per_line(6);
        self.pinned_apps.set_selection_mode(gtk4::SelectionMode::None);
        main_box.append(&self.pinned_apps);

        // Todos os Apps
        let all_label = Label::new(Some("Todos os aplicativos"));
        all_label.add_css_class("section-title");
        main_box.append(&all_label);

        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_child(Some(&self.all_apps));
        main_box.append(&scroll);

        // Área inferior - Usuário e Power
        let bottom_box = self.create_bottom_area();
        main_box.append(&bottom_box);

        self.window.set_child(Some(&main_box));
    }

    fn search_apps(&self, query: &str) {
        let apps = self.get_all_applications();

        let mut results: Vec<_> = apps
            .iter()
            .filter_map(|app| {
                self.matcher
                    .fuzzy_match(&app.name, query)
                    .map(|score| (app, score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));

        // Atualizar UI com resultados
        self.update_search_results(&results);
    }

    pub fn toggle() {
        // Toggle visibilidade
    }
}
```

#### 5.4.3 Snap Layouts

```rust
// /usr/lib/winux/winux-shell/src/snap_layouts.rs

#[derive(Clone, Copy, Debug)]
pub enum SnapZone {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Maximize,
    Center,
}

#[derive(Clone, Debug)]
pub struct SnapLayout {
    pub name: &'static str,
    pub zones: Vec<SnapZoneConfig>,
}

#[derive(Clone, Debug)]
pub struct SnapZoneConfig {
    pub x: f64,      // 0.0 - 1.0
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl SnapLayout {
    pub fn layouts() -> Vec<Self> {
        vec![
            // Layout 50/50 Horizontal
            Self {
                name: "Half Left/Right",
                zones: vec![
                    SnapZoneConfig { x: 0.0, y: 0.0, width: 0.5, height: 1.0 },
                    SnapZoneConfig { x: 0.5, y: 0.0, width: 0.5, height: 1.0 },
                ],
            },
            // Layout 70/30
            Self {
                name: "Primary/Secondary",
                zones: vec![
                    SnapZoneConfig { x: 0.0, y: 0.0, width: 0.7, height: 1.0 },
                    SnapZoneConfig { x: 0.7, y: 0.0, width: 0.3, height: 1.0 },
                ],
            },
            // Layout Quadrantes
            Self {
                name: "Quadrants",
                zones: vec![
                    SnapZoneConfig { x: 0.0, y: 0.0, width: 0.5, height: 0.5 },
                    SnapZoneConfig { x: 0.5, y: 0.0, width: 0.5, height: 0.5 },
                    SnapZoneConfig { x: 0.0, y: 0.5, width: 0.5, height: 0.5 },
                    SnapZoneConfig { x: 0.5, y: 0.5, width: 0.5, height: 0.5 },
                ],
            },
            // Layout 3 Colunas
            Self {
                name: "Three Columns",
                zones: vec![
                    SnapZoneConfig { x: 0.0, y: 0.0, width: 0.33, height: 1.0 },
                    SnapZoneConfig { x: 0.33, y: 0.0, width: 0.34, height: 1.0 },
                    SnapZoneConfig { x: 0.67, y: 0.0, width: 0.33, height: 1.0 },
                ],
            },
        ]
    }
}

pub struct SnapManager {
    current_layout: Option<SnapLayout>,
    preview_window: Option<gtk4::Window>,
}

impl SnapManager {
    pub fn show_snap_assist(&mut self, window_id: u64) {
        // Mostrar overlay com opções de snap
        let overlay = self.create_snap_overlay();
        overlay.show();
    }

    fn create_snap_overlay(&self) -> gtk4::Window {
        let overlay = gtk4::Window::builder()
            .title("Snap Assist")
            .decorated(false)
            .build();

        let layouts_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

        for layout in SnapLayout::layouts() {
            let layout_widget = self.create_layout_preview(&layout);
            layouts_box.append(&layout_widget);
        }

        overlay.set_child(Some(&layouts_box));
        overlay
    }
}
```

---

## 6. Suite de Aplicações Rust

### 6.1 Visão Geral da Suite

```
┌─────────────────────────────────────────────────────────────────────┐
│                    WINUX NATIVE APPS SUITE                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐       │
│  │  Winux    │  │  Winux    │  │  Winux    │  │  Winux    │       │
│  │  Files    │  │ Terminal  │  │ Settings  │  │  Store    │       │
│  │    📁     │  │    💻     │  │    ⚙️     │  │    🛒     │       │
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘       │
│                                                                     │
│  ┌───────────┐  ┌───────────┐                                      │
│  │  Winux    │  │  Winux    │                                      │
│  │ Monitor   │  │   Edit    │                                      │
│  │    📊     │  │    📝     │                                      │
│  └───────────┘  └───────────┘                                      │
│                                                                     │
│  Stack Tecnológico Comum:                                          │
│  • Rust 1.75+                                                      │
│  • GTK4 (gtk4-rs 0.7+)                                            │
│  • Relm4 0.7+ (Elm-like architecture)                             │
│  • Async: Tokio 1.x                                               │
│  • Serialization: Serde + RON                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 Winux Files - Gerenciador de Arquivos

```toml
# /usr/lib/winux/winux-files/Cargo.toml

[package]
name = "winux-files"
version = "1.0.0"
edition = "2021"
authors = ["Winux Team"]
description = "Modern file manager for Winux OS"

[dependencies]
gtk4 = "0.7"
libadwaita = "0.5"
relm4 = "0.7"
relm4-macros = "0.7"
tokio = { version = "1", features = ["full"] }
async-channel = "2.0"
notify = "6.0"           # File system watching
mime_guess = "2.0"
trash = "3.0"            # Trash support
dirs = "5.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ron = "0.8"              # Rusty Object Notation
image = "0.24"           # Thumbnails
rayon = "1.8"            # Parallel iteration
walkdir = "2.4"
chrono = "0.4"
glib = "0.18"
gio = "0.18"

[features]
default = ["cloud", "preview"]
cloud = ["rclone-rs"]
preview = ["poppler-rs", "ffmpeg-next"]
```

```rust
// /usr/lib/winux/winux-files/src/main.rs

mod app;
mod config;
mod file_ops;
mod sidebar;
mod file_view;
mod preview;
mod search;

use relm4::prelude::*;
use gtk4::prelude::*;

fn main() {
    let app = relm4::RelmApp::new("com.winux.files");
    app.run::<app::AppModel>(());
}
```

```rust
// /usr/lib/winux/winux-files/src/app.rs

use relm4::prelude::*;
use gtk4::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AppModel {
    current_path: PathBuf,
    files: Vec<FileEntry>,
    selected: Vec<usize>,
    view_mode: ViewMode,
    show_hidden: bool,
    tabs: Vec<Tab>,
    active_tab: usize,
    sidebar_collapsed: bool,
    preview_panel: Option<PreviewData>,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub file_type: FileType,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub permissions: String,
    pub is_hidden: bool,
    pub thumbnail: Option<gtk4::gdk_pixbuf::Pixbuf>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Icons,
    List,
    Columns,
    Details,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    Navigate(PathBuf),
    NavigateUp,
    NavigateBack,
    NavigateForward,
    Refresh,
    SelectFile(usize),
    SelectMultiple(Vec<usize>),
    OpenFile(PathBuf),
    OpenWith(PathBuf, String),
    CreateFolder(String),
    CreateFile(String),
    Delete(Vec<PathBuf>),
    Rename(PathBuf, String),
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
    Paste,
    Search(String),
    SetViewMode(ViewMode),
    ToggleHidden,
    NewTab(PathBuf),
    CloseTab(usize),
    SwitchTab(usize),
    ToggleSidebar,
    ShowProperties(PathBuf),
    ShowPreview(PathBuf),
    ExtractArchive(PathBuf),
    CompressFiles(Vec<PathBuf>),
}

#[relm4::component(pub)]
impl Component for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("Winux Files"),
            set_default_width: 1200,
            set_default_height: 800,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,

                // Header Bar
                gtk4::HeaderBar {
                    pack_start = &gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 4,

                        gtk4::Button {
                            set_icon_name: "go-previous-symbolic",
                            set_tooltip_text: Some("Voltar"),
                            connect_clicked => AppMsg::NavigateBack,
                        },
                        gtk4::Button {
                            set_icon_name: "go-next-symbolic",
                            set_tooltip_text: Some("Avançar"),
                            connect_clicked => AppMsg::NavigateForward,
                        },
                        gtk4::Button {
                            set_icon_name: "go-up-symbolic",
                            set_tooltip_text: Some("Pasta pai"),
                            connect_clicked => AppMsg::NavigateUp,
                        },
                    },

                    #[wrap(Some)]
                    set_title_widget = &gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,

                        // Breadcrumb / Path Bar
                        gtk4::Entry {
                            set_hexpand: true,
                            set_text: &model.current_path.display().to_string(),
                            set_icon_from_icon_name: (gtk4::EntryIconPosition::Primary, Some("folder-symbolic")),
                        },
                    },

                    pack_end = &gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 4,

                        gtk4::ToggleButton {
                            set_icon_name: "view-grid-symbolic",
                            set_tooltip_text: Some("Modo de visualização"),
                        },
                        gtk4::Button {
                            set_icon_name: "edit-find-symbolic",
                            set_tooltip_text: Some("Pesquisar"),
                        },
                        gtk4::MenuButton {
                            set_icon_name: "open-menu-symbolic",
                        },
                    },
                },

                // Main Content
                gtk4::Paned {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_vexpand: true,

                    // Sidebar
                    #[wrap(Some)]
                    set_start_child = &gtk4::ScrolledWindow {
                        set_width_request: 200,

                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_margin_all: 8,

                            // Favoritos
                            gtk4::Label {
                                set_text: "Favoritos",
                                set_xalign: 0.0,
                                add_css_class: "heading",
                            },

                            // Lista de favoritos
                            gtk4::ListBox {
                                set_selection_mode: gtk4::SelectionMode::Single,
                                add_css_class: "navigation-sidebar",

                                gtk4::ListBoxRow {
                                    gtk4::Box {
                                        gtk4::Image { set_icon_name: Some("user-home-symbolic") },
                                        gtk4::Label { set_text: "Início" },
                                    }
                                },
                                gtk4::ListBoxRow {
                                    gtk4::Box {
                                        gtk4::Image { set_icon_name: Some("folder-documents-symbolic") },
                                        gtk4::Label { set_text: "Documentos" },
                                    }
                                },
                                gtk4::ListBoxRow {
                                    gtk4::Box {
                                        gtk4::Image { set_icon_name: Some("folder-download-symbolic") },
                                        gtk4::Label { set_text: "Downloads" },
                                    }
                                },
                                gtk4::ListBoxRow {
                                    gtk4::Box {
                                        gtk4::Image { set_icon_name: Some("folder-pictures-symbolic") },
                                        gtk4::Label { set_text: "Imagens" },
                                    }
                                },
                                gtk4::ListBoxRow {
                                    gtk4::Box {
                                        gtk4::Image { set_icon_name: Some("folder-videos-symbolic") },
                                        gtk4::Label { set_text: "Vídeos" },
                                    }
                                },
                            },

                            // Dispositivos
                            gtk4::Label {
                                set_text: "Dispositivos",
                                set_xalign: 0.0,
                                add_css_class: "heading",
                                set_margin_top: 16,
                            },
                        },
                    },

                    // File View Area
                    #[wrap(Some)]
                    set_end_child = &gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,

                        // Tabs
                        gtk4::Notebook {
                            set_show_border: false,
                        },

                        // File Grid/List
                        gtk4::ScrolledWindow {
                            set_vexpand: true,

                            gtk4::GridView {
                                // Configurado dinamicamente
                            },
                        },
                    },
                },

                // Status Bar
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 16,
                    set_margin_all: 4,
                    add_css_class: "statusbar",

                    gtk4::Label {
                        #[watch]
                        set_text: &format!("{} itens", model.files.len()),
                    },
                    gtk4::Label {
                        #[watch]
                        set_text: &format!("{} selecionados", model.selected.len()),
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            current_path: dirs::home_dir().unwrap_or_default(),
            files: Vec::new(),
            selected: Vec::new(),
            view_mode: ViewMode::Icons,
            show_hidden: false,
            tabs: vec![Tab::new(dirs::home_dir().unwrap_or_default())],
            active_tab: 0,
            sidebar_collapsed: false,
            preview_panel: None,
        };

        let widgets = view_output!();

        // Carregar arquivos iniciais
        sender.input(AppMsg::Refresh);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AppMsg::Navigate(path) => {
                self.current_path = path;
                self.load_files();
            }
            AppMsg::NavigateUp => {
                if let Some(parent) = self.current_path.parent() {
                    self.current_path = parent.to_path_buf();
                    self.load_files();
                }
            }
            AppMsg::Refresh => {
                self.load_files();
            }
            AppMsg::SelectFile(index) => {
                self.selected = vec![index];
            }
            AppMsg::SetViewMode(mode) => {
                self.view_mode = mode;
            }
            AppMsg::ToggleHidden => {
                self.show_hidden = !self.show_hidden;
                self.load_files();
            }
            // ... outros handlers
            _ => {}
        }
    }
}

impl AppModel {
    fn load_files(&mut self) {
        self.files.clear();

        if let Ok(entries) = std::fs::read_dir(&self.current_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                let metadata = entry.metadata().ok();

                let file_entry = FileEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: path.clone(),
                    file_type: if path.is_dir() {
                        FileType::Directory
                    } else {
                        FileType::from_extension(path.extension())
                    },
                    size: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                    modified: metadata
                        .and_then(|m| m.modified().ok())
                        .map(chrono::DateTime::from)
                        .unwrap_or_default(),
                    permissions: String::new(),
                    is_hidden: entry.file_name().to_string_lossy().starts_with('.'),
                    thumbnail: None,
                };

                if self.show_hidden || !file_entry.is_hidden {
                    self.files.push(file_entry);
                }
            }
        }

        // Ordenar: pastas primeiro, depois por nome
        self.files.sort_by(|a, b| {
            match (&a.file_type, &b.file_type) {
                (FileType::Directory, FileType::Directory) => a.name.cmp(&b.name),
                (FileType::Directory, _) => std::cmp::Ordering::Less,
                (_, FileType::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub path: PathBuf,
    pub title: String,
}

impl Tab {
    pub fn new(path: PathBuf) -> Self {
        let title = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        Self { path, title }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Directory,
    Text,
    Image,
    Video,
    Audio,
    Archive,
    Executable,
    Document,
    Code,
    Other,
}

impl FileType {
    pub fn from_extension(ext: Option<&std::ffi::OsStr>) -> Self {
        match ext.and_then(|e| e.to_str()).map(|s| s.to_lowercase()).as_deref() {
            Some("txt" | "md" | "log") => Self::Text,
            Some("png" | "jpg" | "jpeg" | "gif" | "svg" | "webp") => Self::Image,
            Some("mp4" | "mkv" | "avi" | "webm" | "mov") => Self::Video,
            Some("mp3" | "flac" | "wav" | "ogg" | "m4a") => Self::Audio,
            Some("zip" | "tar" | "gz" | "7z" | "rar" | "xz") => Self::Archive,
            Some("exe" | "AppImage" | "deb" | "rpm") => Self::Executable,
            Some("pdf" | "doc" | "docx" | "odt") => Self::Document,
            Some("rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "go" | "java") => Self::Code,
            _ => Self::Other,
        }
    }
}
```

### 6.3 Winux Terminal - Emulador de Terminal

```toml
# /usr/lib/winux/winux-terminal/Cargo.toml

[package]
name = "winux-terminal"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI
gtk4 = "0.7"
libadwaita = "0.5"

# Terminal
vte4 = "0.7"               # VTE terminal widget para GTK4
alacritty_terminal = "0.21" # Parser de terminal

# GPU Rendering
wgpu = "0.18"
glyphon = "0.4"            # Text rendering

# Config
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dirs = "5.0"

# Async
tokio = { version = "1", features = ["full"] }

[features]
default = ["gpu"]
gpu = ["wgpu", "glyphon"]
```

```rust
// /usr/lib/winux/winux-terminal/src/main.rs

mod app;
mod terminal;
mod config;
mod tabs;
mod themes;

use gtk4::prelude::*;
use libadwaita as adw;

fn main() {
    let app = adw::Application::builder()
        .application_id("com.winux.terminal")
        .build();

    app.connect_activate(|app| {
        let window = build_window(app);
        window.present();
    });

    app.run();
}

fn build_window(app: &adw::Application) -> adw::ApplicationWindow {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Winux Terminal")
        .default_width(900)
        .default_height(600)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // Header Bar
    let header = adw::HeaderBar::new();

    let new_tab_btn = gtk4::Button::from_icon_name("tab-new-symbolic");
    new_tab_btn.set_tooltip_text(Some("Nova aba (Ctrl+Shift+T)"));
    header.pack_start(&new_tab_btn);

    let menu_btn = gtk4::MenuButton::new();
    menu_btn.set_icon_name("open-menu-symbolic");
    header.pack_end(&menu_btn);

    content.append(&header);

    // Tab View
    let tab_view = adw::TabView::new();
    let tab_bar = adw::TabBar::new();
    tab_bar.set_view(Some(&tab_view));
    content.append(&tab_bar);

    // Área do terminal
    let terminal_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    terminal_box.set_vexpand(true);

    // Criar terminal inicial
    let terminal = create_terminal();
    terminal_box.append(&terminal);

    // Adicionar primeira aba
    let page = tab_view.append(&terminal_box);
    page.set_title("Terminal");
    page.set_icon(Some(&gio::ThemedIcon::new("utilities-terminal-symbolic")));

    content.append(&tab_view);

    window.set_content(Some(&content));

    // Atalhos de teclado
    setup_shortcuts(&window, &tab_view);

    window
}

fn create_terminal() -> vte4::Terminal {
    let terminal = vte4::Terminal::new();

    // Configurações visuais
    terminal.set_font_desc(Some(&pango::FontDescription::from_string("JetBrains Mono 11")));
    terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
    terminal.set_cursor_shape(vte4::CursorShape::Block);
    terminal.set_scroll_on_output(false);
    terminal.set_scroll_on_keystroke(true);
    terminal.set_scrollback_lines(10000);

    // Cores - Tema escuro
    let palette: Vec<gdk4::RGBA> = vec![
        gdk4::RGBA::parse("#1e1e2e").unwrap(), // background
        gdk4::RGBA::parse("#f38ba8").unwrap(), // red
        gdk4::RGBA::parse("#a6e3a1").unwrap(), // green
        gdk4::RGBA::parse("#f9e2af").unwrap(), // yellow
        gdk4::RGBA::parse("#89b4fa").unwrap(), // blue
        gdk4::RGBA::parse("#f5c2e7").unwrap(), // magenta
        gdk4::RGBA::parse("#94e2d5").unwrap(), // cyan
        gdk4::RGBA::parse("#cdd6f4").unwrap(), // white
        // Bright variants
        gdk4::RGBA::parse("#45475a").unwrap(),
        gdk4::RGBA::parse("#f38ba8").unwrap(),
        gdk4::RGBA::parse("#a6e3a1").unwrap(),
        gdk4::RGBA::parse("#f9e2af").unwrap(),
        gdk4::RGBA::parse("#89b4fa").unwrap(),
        gdk4::RGBA::parse("#f5c2e7").unwrap(),
        gdk4::RGBA::parse("#94e2d5").unwrap(),
        gdk4::RGBA::parse("#cdd6f4").unwrap(),
    ];

    let bg = gdk4::RGBA::parse("#1e1e2e").unwrap();
    let fg = gdk4::RGBA::parse("#cdd6f4").unwrap();

    terminal.set_colors(Some(&fg), Some(&bg), &palette);

    // Spawnar shell
    terminal.spawn_async(
        vte4::PtyFlags::DEFAULT,
        None,                     // working directory
        &["/bin/bash"],           // command
        &[],                      // environment
        glib::SpawnFlags::DEFAULT,
        || {},                    // child setup
        -1,                       // timeout
        None::<&gio::Cancellable>,
        |_result| {},             // callback
    );

    terminal
}

fn setup_shortcuts(window: &adw::ApplicationWindow, tab_view: &adw::TabView) {
    let controller = gtk4::EventControllerKey::new();

    let tab_view_clone = tab_view.clone();
    controller.connect_key_pressed(move |_, key, _, modifier| {
        let ctrl_shift = gdk4::ModifierType::CONTROL_MASK | gdk4::ModifierType::SHIFT_MASK;

        if modifier.contains(ctrl_shift) {
            match key {
                gdk4::Key::T => {
                    // Nova aba
                    let terminal_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                    let terminal = create_terminal();
                    terminal_box.append(&terminal);

                    let page = tab_view_clone.append(&terminal_box);
                    page.set_title("Terminal");
                    tab_view_clone.set_selected_page(&page);

                    return glib::Propagation::Stop;
                }
                gdk4::Key::W => {
                    // Fechar aba
                    if let Some(page) = tab_view_clone.selected_page() {
                        tab_view_clone.close_page(&page);
                    }
                    return glib::Propagation::Stop;
                }
                _ => {}
            }
        }

        glib::Propagation::Proceed
    });

    window.add_controller(controller);
}
```

### 6.4 Winux Settings - Central de Configurações

```rust
// /usr/lib/winux/winux-settings/src/main.rs

use gtk4::prelude::*;
use libadwaita as adw;

mod pages {
    pub mod display;
    pub mod sound;
    pub mod network;
    pub mod bluetooth;
    pub mod appearance;
    pub mod apps;
    pub mod users;
    pub mod datetime;
    pub mod gaming;
    pub mod about;
}

fn main() {
    let app = adw::Application::builder()
        .application_id("com.winux.settings")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Configurações")
        .default_width(1000)
        .default_height(700)
        .build();

    let leaflet = adw::Leaflet::new();
    leaflet.set_can_unfold(true);

    // Sidebar
    let sidebar = create_sidebar();
    leaflet.append(&sidebar);

    // Separador
    let separator = gtk4::Separator::new(gtk4::Orientation::Vertical);
    leaflet.append(&separator);

    // Área de conteúdo
    let content_stack = gtk4::Stack::new();
    content_stack.set_hexpand(true);

    // Adicionar páginas
    content_stack.add_titled(
        &pages::display::create_page(),
        Some("display"),
        "Tela"
    );
    content_stack.add_titled(
        &pages::sound::create_page(),
        Some("sound"),
        "Som"
    );
    content_stack.add_titled(
        &pages::network::create_page(),
        Some("network"),
        "Rede"
    );
    content_stack.add_titled(
        &pages::appearance::create_page(),
        Some("appearance"),
        "Aparência"
    );
    content_stack.add_titled(
        &pages::gaming::create_page(),
        Some("gaming"),
        "Gaming"
    );
    content_stack.add_titled(
        &pages::about::create_page(),
        Some("about"),
        "Sobre"
    );

    leaflet.append(&content_stack);

    window.set_content(Some(&leaflet));
    window.present();
}

fn create_sidebar() -> gtk4::Box {
    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar.set_width_request(250);
    sidebar.add_css_class("navigation-sidebar");

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_vexpand(true);

    let listbox = gtk4::ListBox::new();
    listbox.set_selection_mode(gtk4::SelectionMode::Single);

    let categories = vec![
        ("Sistema", vec![
            ("display-symbolic", "Tela", "display"),
            ("audio-volume-high-symbolic", "Som", "sound"),
            ("network-wireless-symbolic", "Rede", "network"),
            ("bluetooth-symbolic", "Bluetooth", "bluetooth"),
        ]),
        ("Personalização", vec![
            ("preferences-desktop-wallpaper-symbolic", "Aparência", "appearance"),
            ("application-x-executable-symbolic", "Aplicativos", "apps"),
        ]),
        ("Contas", vec![
            ("system-users-symbolic", "Usuários", "users"),
            ("preferences-system-time-symbolic", "Data e Hora", "datetime"),
        ]),
        ("Gaming", vec![
            ("input-gaming-symbolic", "Gaming", "gaming"),
        ]),
        ("Sobre", vec![
            ("help-about-symbolic", "Sobre o Winux", "about"),
        ]),
    ];

    for (category, items) in categories {
        let label = gtk4::Label::new(Some(category));
        label.set_xalign(0.0);
        label.add_css_class("heading");
        label.set_margin_top(16);
        label.set_margin_start(12);
        listbox.append(&label);

        for (icon, name, id) in items {
            let row = adw::ActionRow::builder()
                .title(name)
                .activatable(true)
                .build();

            let icon_widget = gtk4::Image::from_icon_name(icon);
            row.add_prefix(&icon_widget);

            listbox.append(&row);
        }
    }

    scroll.set_child(Some(&listbox));
    sidebar.append(&scroll);

    sidebar
}
```

```rust
// /usr/lib/winux/winux-settings/src/pages/gaming.rs

use gtk4::prelude::*;
use libadwaita as adw;

pub fn create_page() -> gtk4::Box {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_vexpand(true);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
    content.set_margin_all(24);

    // Header
    let header = gtk4::Label::new(Some("Gaming"));
    header.add_css_class("title-1");
    header.set_xalign(0.0);
    content.append(&header);

    // Game Mode
    let gamemode_group = adw::PreferencesGroup::new();
    gamemode_group.set_title("GameMode");
    gamemode_group.set_description(Some(
        "Otimiza automaticamente o sistema quando jogos são executados"
    ));

    let gamemode_row = adw::SwitchRow::builder()
        .title("Ativar GameMode")
        .subtitle("Ajusta CPU, GPU e I/O automaticamente")
        .build();
    gamemode_group.add(&gamemode_row);

    content.append(&gamemode_group);

    // MangoHud
    let mangohud_group = adw::PreferencesGroup::new();
    mangohud_group.set_title("MangoHud");
    mangohud_group.set_description(Some(
        "Overlay de performance em tempo real durante jogos"
    ));

    let mangohud_row = adw::SwitchRow::builder()
        .title("Ativar MangoHud")
        .subtitle("Exibe FPS, frametime, CPU/GPU usage")
        .build();
    mangohud_group.add(&mangohud_row);

    let mangohud_position = adw::ComboRow::builder()
        .title("Posição do overlay")
        .model(&gtk4::StringList::new(&[
            "Superior esquerdo",
            "Superior direito",
            "Inferior esquerdo",
            "Inferior direito",
        ]))
        .build();
    mangohud_group.add(&mangohud_position);

    content.append(&mangohud_group);

    // Wine/Proton
    let wine_group = adw::PreferencesGroup::new();
    wine_group.set_title("Wine & Proton");

    let wine_version_row = adw::ActionRow::builder()
        .title("Versão do Wine")
        .subtitle("wine-staging-9.0")
        .build();
    let wine_btn = gtk4::Button::with_label("Gerenciar");
    wine_version_row.add_suffix(&wine_btn);
    wine_group.add(&wine_version_row);

    let proton_row = adw::ActionRow::builder()
        .title("Proton-GE")
        .subtitle("GE-Proton9-7")
        .build();
    let proton_btn = gtk4::Button::with_label("Atualizar");
    proton_row.add_suffix(&proton_btn);
    wine_group.add(&proton_row);

    let dxvk_row = adw::SwitchRow::builder()
        .title("DXVK Async")
        .subtitle("Compilação assíncrona de shaders")
        .active(true)
        .build();
    wine_group.add(&dxvk_row);

    content.append(&wine_group);

    // Steam
    let steam_group = adw::PreferencesGroup::new();
    steam_group.set_title("Steam");

    let steam_runtime_row = adw::ComboRow::builder()
        .title("Runtime padrão")
        .model(&gtk4::StringList::new(&[
            "Steam Linux Runtime",
            "Proton Experimental",
            "Proton-GE (mais recente)",
            "Wine nativo",
        ]))
        .build();
    steam_group.add(&steam_runtime_row);

    let shader_cache_row = adw::SwitchRow::builder()
        .title("Pre-cache de shaders")
        .subtitle("Download de shaders compilados da Steam")
        .active(true)
        .build();
    steam_group.add(&shader_cache_row);

    content.append(&steam_group);

    // GPU
    let gpu_group = adw::PreferencesGroup::new();
    gpu_group.set_title("GPU");

    let gpu_info_row = adw::ActionRow::builder()
        .title("GPU Detectada")
        .subtitle("NVIDIA GeForce RTX 4070")
        .build();
    gpu_group.add(&gpu_info_row);

    let vsync_row = adw::ComboRow::builder()
        .title("VSync")
        .model(&gtk4::StringList::new(&[
            "Desativado",
            "Ativado",
            "Adaptativo",
        ]))
        .build();
    gpu_group.add(&vsync_row);

    content.append(&gpu_group);

    scroll.set_child(Some(&content));
    page.append(&scroll);

    page
}
```

### 6.5 Winux Store - Loja de Aplicativos

```rust
// /usr/lib/winux/winux-store/src/main.rs

use gtk4::prelude::*;
use libadwaita as adw;

mod backend;
mod ui;

fn main() {
    let app = adw::Application::builder()
        .application_id("com.winux.store")
        .build();

    app.connect_activate(|app| {
        let window = ui::build_main_window(app);
        window.present();
    });

    app.run();
}
```

```rust
// /usr/lib/winux/winux-store/src/backend/mod.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub developer: String,
    pub icon_url: String,
    pub screenshots: Vec<String>,
    pub version: String,
    pub size: u64,
    pub rating: f32,
    pub reviews_count: u32,
    pub categories: Vec<String>,
    pub source: PackageSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PackageSource {
    Flatpak,
    Snap,
    Apt,
    AppImage,
}

pub struct StoreBackend {
    flatpak: FlatpakBackend,
    snap: SnapBackend,
    apt: AptBackend,
    cache: HashMap<String, AppInfo>,
}

impl StoreBackend {
    pub async fn search(&self, query: &str) -> Vec<AppInfo> {
        let mut results = Vec::new();

        // Busca paralela em todos os backends
        let (flatpak_results, snap_results, apt_results) = tokio::join!(
            self.flatpak.search(query),
            self.snap.search(query),
            self.apt.search(query)
        );

        results.extend(flatpak_results);
        results.extend(snap_results);
        results.extend(apt_results);

        // Ordenar por relevância
        results.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());

        results
    }

    pub async fn install(&self, app: &AppInfo) -> Result<(), InstallError> {
        match app.source {
            PackageSource::Flatpak => self.flatpak.install(&app.id).await,
            PackageSource::Snap => self.snap.install(&app.id).await,
            PackageSource::Apt => self.apt.install(&app.id).await,
            PackageSource::AppImage => todo!(),
        }
    }

    pub async fn get_installed(&self) -> Vec<AppInfo> {
        let mut installed = Vec::new();

        installed.extend(self.flatpak.list_installed().await);
        installed.extend(self.snap.list_installed().await);
        installed.extend(self.apt.list_installed().await);

        installed
    }

    pub async fn get_updates(&self) -> Vec<(AppInfo, String)> {
        let mut updates = Vec::new();

        updates.extend(self.flatpak.check_updates().await);
        updates.extend(self.snap.check_updates().await);
        updates.extend(self.apt.check_updates().await);

        updates
    }
}

pub struct FlatpakBackend;
pub struct SnapBackend;
pub struct AptBackend;

#[derive(Debug)]
pub enum InstallError {
    NotFound,
    PermissionDenied,
    NetworkError,
    DiskFull,
    Other(String),
}

impl FlatpakBackend {
    pub async fn search(&self, query: &str) -> Vec<AppInfo> {
        // flatpak search query
        let output = tokio::process::Command::new("flatpak")
            .args(["search", "--columns=application,name,description", query])
            .output()
            .await
            .ok();

        // Parse output
        Vec::new()
    }

    pub async fn install(&self, app_id: &str) -> Result<(), InstallError> {
        let status = tokio::process::Command::new("flatpak")
            .args(["install", "-y", "flathub", app_id])
            .status()
            .await
            .map_err(|e| InstallError::Other(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(InstallError::Other("Installation failed".into()))
        }
    }

    pub async fn list_installed(&self) -> Vec<AppInfo> {
        Vec::new()
    }

    pub async fn check_updates(&self) -> Vec<(AppInfo, String)> {
        Vec::new()
    }
}
```

### 6.6 Winux Monitor - Gerenciador de Tarefas

```rust
// /usr/lib/winux/winux-monitor/src/main.rs

use gtk4::prelude::*;
use libadwaita as adw;
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, DiskExt};
use std::time::Duration;

fn main() {
    let app = adw::Application::builder()
        .application_id("com.winux.monitor")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Monitor do Sistema")
        .default_width(1000)
        .default_height(700)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // Header
    let header = adw::HeaderBar::new();
    let view_switcher = adw::ViewSwitcher::new();
    header.set_title_widget(Some(&view_switcher));
    content.append(&header);

    // View Stack
    let view_stack = adw::ViewStack::new();

    // Processos
    let processes_page = create_processes_view();
    view_stack.add_titled_with_icon(
        &processes_page,
        Some("processes"),
        "Processos",
        "system-run-symbolic"
    );

    // Performance
    let performance_page = create_performance_view();
    view_stack.add_titled_with_icon(
        &performance_page,
        Some("performance"),
        "Performance",
        "utilities-system-monitor-symbolic"
    );

    // Startup
    let startup_page = create_startup_view();
    view_stack.add_titled_with_icon(
        &startup_page,
        Some("startup"),
        "Inicialização",
        "system-shutdown-symbolic"
    );

    view_switcher.set_stack(Some(&view_stack));
    content.append(&view_stack);

    window.set_content(Some(&content));
    window.present();

    // Atualização periódica
    let view_stack_clone = view_stack.clone();
    glib::timeout_add_local(Duration::from_secs(1), move || {
        update_system_info(&view_stack_clone);
        glib::ControlFlow::Continue
    });
}

fn create_processes_view() -> gtk4::Box {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // Toolbar de busca/filtro
    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    toolbar.set_margin_all(8);

    let search = gtk4::SearchEntry::new();
    search.set_placeholder_text(Some("Buscar processos..."));
    search.set_hexpand(true);
    toolbar.append(&search);

    let end_btn = gtk4::Button::with_label("Encerrar Processo");
    end_btn.add_css_class("destructive-action");
    toolbar.append(&end_btn);

    page.append(&toolbar);

    // Lista de processos
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_vexpand(true);

    let list_store = gio::ListStore::new::<ProcessObject>();
    let selection = gtk4::SingleSelection::new(Some(list_store.clone()));

    let factory = gtk4::SignalListItemFactory::new();
    factory.connect_setup(|_, list_item| {
        let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();

        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        row.set_margin_all(8);

        let name_label = gtk4::Label::new(None);
        name_label.set_xalign(0.0);
        name_label.set_hexpand(true);
        row.append(&name_label);

        let cpu_label = gtk4::Label::new(None);
        cpu_label.set_width_chars(8);
        row.append(&cpu_label);

        let mem_label = gtk4::Label::new(None);
        mem_label.set_width_chars(10);
        row.append(&mem_label);

        let pid_label = gtk4::Label::new(None);
        pid_label.set_width_chars(8);
        row.append(&pid_label);

        list_item.set_child(Some(&row));
    });

    let column_view = gtk4::ColumnView::new(Some(selection));

    // Colunas
    let name_col = gtk4::ColumnViewColumn::new(Some("Nome"), Some(factory.clone()));
    name_col.set_expand(true);
    column_view.append_column(&name_col);

    let cpu_col = gtk4::ColumnViewColumn::new(Some("CPU %"), Some(factory.clone()));
    column_view.append_column(&cpu_col);

    let mem_col = gtk4::ColumnViewColumn::new(Some("Memória"), Some(factory.clone()));
    column_view.append_column(&mem_col);

    let pid_col = gtk4::ColumnViewColumn::new(Some("PID"), Some(factory.clone()));
    column_view.append_column(&pid_col);

    scroll.set_child(Some(&column_view));
    page.append(&scroll);

    page
}

fn create_performance_view() -> gtk4::Box {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    page.set_margin_all(16);

    // CPU
    let cpu_frame = adw::PreferencesGroup::new();
    cpu_frame.set_title("CPU");

    let cpu_usage = adw::ActionRow::builder()
        .title("Utilização")
        .subtitle("0%")
        .build();

    let cpu_graph = gtk4::DrawingArea::new();
    cpu_graph.set_size_request(-1, 100);
    cpu_graph.set_draw_func(|_, cr, width, height| {
        // Desenhar gráfico de CPU
        cr.set_source_rgb(0.2, 0.6, 1.0);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        let _ = cr.fill();
    });

    cpu_frame.add(&cpu_usage);
    page.append(&cpu_frame);
    page.append(&cpu_graph);

    // Memória
    let mem_frame = adw::PreferencesGroup::new();
    mem_frame.set_title("Memória");

    let mem_usage = adw::ActionRow::builder()
        .title("Em uso")
        .subtitle("0 GB / 0 GB")
        .build();

    let mem_graph = gtk4::DrawingArea::new();
    mem_graph.set_size_request(-1, 100);

    mem_frame.add(&mem_usage);
    page.append(&mem_frame);
    page.append(&mem_graph);

    // GPU
    let gpu_frame = adw::PreferencesGroup::new();
    gpu_frame.set_title("GPU");

    let gpu_usage = adw::ActionRow::builder()
        .title("Utilização")
        .subtitle("0%")
        .build();

    let gpu_temp = adw::ActionRow::builder()
        .title("Temperatura")
        .subtitle("0°C")
        .build();

    gpu_frame.add(&gpu_usage);
    gpu_frame.add(&gpu_temp);
    page.append(&gpu_frame);

    page
}

fn create_startup_view() -> gtk4::Box {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_vexpand(true);

    let listbox = gtk4::ListBox::new();
    listbox.set_selection_mode(gtk4::SelectionMode::None);
    listbox.add_css_class("boxed-list");
    listbox.set_margin_all(16);

    // Exemplo de apps de inicialização
    let startup_apps = vec![
        ("Steam", true, "Baixo"),
        ("Discord", true, "Baixo"),
        ("Dropbox", false, "Médio"),
        ("Slack", false, "Baixo"),
    ];

    for (name, enabled, impact) in startup_apps {
        let row = adw::ActionRow::builder()
            .title(name)
            .subtitle(&format!("Impacto: {}", impact))
            .build();

        let switch = gtk4::Switch::new();
        switch.set_active(enabled);
        switch.set_valign(gtk4::Align::Center);
        row.add_suffix(&switch);

        listbox.append(&row);
    }

    scroll.set_child(Some(&listbox));
    page.append(&scroll);

    page
}

fn update_system_info(_stack: &adw::ViewStack) {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Atualizar dados...
}

// GObject wrapper para processos
mod imp {
    use super::*;
    use glib::subclass::prelude::*;

    #[derive(Default)]
    pub struct ProcessObject {
        pub pid: std::cell::Cell<u32>,
        pub name: std::cell::RefCell<String>,
        pub cpu_usage: std::cell::Cell<f32>,
        pub memory: std::cell::Cell<u64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProcessObject {
        const NAME: &'static str = "ProcessObject";
        type Type = super::ProcessObject;
    }

    impl ObjectImpl for ProcessObject {}
}

glib::wrapper! {
    pub struct ProcessObject(ObjectSubclass<imp::ProcessObject>);
}

impl ProcessObject {
    pub fn new(pid: u32, name: &str, cpu: f32, mem: u64) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().pid.set(pid);
        obj.imp().name.replace(name.to_string());
        obj.imp().cpu_usage.set(cpu);
        obj.imp().memory.set(mem);
        obj
    }
}
```

### 6.7 Winux Edit - Editor de Texto

```rust
// /usr/lib/winux/winux-edit/src/main.rs

use gtk4::prelude::*;
use libadwaita as adw;
use sourceview5::prelude::*;

fn main() {
    let app = adw::Application::builder()
        .application_id("com.winux.edit")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Winux Edit")
        .default_width(1200)
        .default_height(800)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // Header Bar
    let header = adw::HeaderBar::new();

    // Botões de arquivo
    let file_buttons = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    let new_btn = gtk4::Button::from_icon_name("document-new-symbolic");
    new_btn.set_tooltip_text(Some("Novo (Ctrl+N)"));
    file_buttons.append(&new_btn);

    let open_btn = gtk4::Button::from_icon_name("document-open-symbolic");
    open_btn.set_tooltip_text(Some("Abrir (Ctrl+O)"));
    file_buttons.append(&open_btn);

    let save_btn = gtk4::Button::from_icon_name("document-save-symbolic");
    save_btn.set_tooltip_text(Some("Salvar (Ctrl+S)"));
    file_buttons.append(&save_btn);

    header.pack_start(&file_buttons);

    // Título do documento
    let title_label = gtk4::Label::new(Some("Sem título"));
    title_label.add_css_class("title");
    header.set_title_widget(Some(&title_label));

    // Menu
    let menu_btn = gtk4::MenuButton::new();
    menu_btn.set_icon_name("open-menu-symbolic");
    header.pack_end(&menu_btn);

    content.append(&header);

    // Main content
    let main_box = gtk4::Paned::new(gtk4::Orientation::Horizontal);

    // Sidebar (explorador de arquivos)
    let sidebar = create_sidebar();
    main_box.set_start_child(Some(&sidebar));

    // Editor area
    let editor_area = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // Tab bar
    let tab_view = adw::TabView::new();
    let tab_bar = adw::TabBar::new();
    tab_bar.set_view(Some(&tab_view));
    editor_area.append(&tab_bar);

    // Criar primeira aba com editor
    let editor = create_editor();
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_child(Some(&editor));
    scroll.set_vexpand(true);

    let page = tab_view.append(&scroll);
    page.set_title("Sem título");

    editor_area.append(&tab_view);

    // Status bar
    let status_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    status_bar.set_margin_all(4);
    status_bar.add_css_class("statusbar");

    let line_col = gtk4::Label::new(Some("Ln 1, Col 1"));
    status_bar.append(&line_col);

    let encoding = gtk4::Label::new(Some("UTF-8"));
    status_bar.append(&encoding);

    let line_ending = gtk4::Label::new(Some("LF"));
    status_bar.append(&line_ending);

    let language = gtk4::Label::new(Some("Plain Text"));
    language.set_hexpand(true);
    language.set_xalign(1.0);
    status_bar.append(&language);

    editor_area.append(&status_bar);

    main_box.set_end_child(Some(&editor_area));

    content.append(&main_box);
    window.set_content(Some(&content));
    window.present();
}

fn create_sidebar() -> gtk4::Box {
    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar.set_width_request(200);

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_vexpand(true);

    // Tree view para arquivos
    let tree = gtk4::TreeView::new();
    tree.set_headers_visible(false);

    let column = gtk4::TreeViewColumn::new();
    let icon_cell = gtk4::CellRendererPixbuf::new();
    let text_cell = gtk4::CellRendererText::new();

    column.pack_start(&icon_cell, false);
    column.pack_start(&text_cell, true);

    tree.append_column(&column);

    scroll.set_child(Some(&tree));
    sidebar.append(&scroll);

    sidebar
}

fn create_editor() -> sourceview5::View {
    let buffer = sourceview5::Buffer::new(None);

    // Configurar syntax highlighting
    let lang_manager = sourceview5::LanguageManager::default();
    if let Some(lang) = lang_manager.language("rust") {
        buffer.set_language(Some(&lang));
    }

    // Configurar tema
    let style_manager = sourceview5::StyleSchemeManager::default();
    if let Some(scheme) = style_manager.scheme("Adwaita-dark") {
        buffer.set_style_scheme(Some(&scheme));
    }

    let view = sourceview5::View::with_buffer(&buffer);

    // Configurações do editor
    view.set_show_line_numbers(true);
    view.set_show_line_marks(true);
    view.set_highlight_current_line(true);
    view.set_auto_indent(true);
    view.set_indent_on_tab(true);
    view.set_indent_width(4);
    view.set_insert_spaces_instead_of_tabs(true);
    view.set_smart_backspace(true);
    view.set_smart_home_end(sourceview5::SmartHomeEndType::Before);

    // Fonte
    let font_desc = pango::FontDescription::from_string("JetBrains Mono 12");
    view.set_monospace(true);

    // Bracket matching
    buffer.set_highlight_matching_brackets(true);

    view
}
```

---

## 7. Otimizações de Performance

### 7.1 Kernel Tunables

```bash
# /etc/sysctl.d/99-winux-performance.conf
# Winux OS - Otimizações de Performance

# ===== MEMÓRIA =====
# Reduzir swappiness para preferir RAM
vm.swappiness=10

# Aumentar cache de inodes/dentries
vm.vfs_cache_pressure=50

# Dirty ratio - quando começar writeback
vm.dirty_ratio=15
vm.dirty_background_ratio=5

# Huge Pages
vm.nr_hugepages=512
vm.hugetlb_shm_group=1000

# Memory overcommit (2 = never overcommit)
vm.overcommit_memory=0
vm.overcommit_ratio=80

# OOM killer menos agressivo
vm.oom_kill_allocating_task=0

# ===== REDE =====
# Aumentar buffers de rede
net.core.rmem_max=16777216
net.core.wmem_max=16777216
net.core.rmem_default=1048576
net.core.wmem_default=1048576
net.core.optmem_max=65536
net.core.netdev_max_backlog=5000

# TCP optimization
net.ipv4.tcp_rmem=4096 1048576 16777216
net.ipv4.tcp_wmem=4096 1048576 16777216
net.ipv4.tcp_fastopen=3
net.ipv4.tcp_tw_reuse=1
net.ipv4.tcp_fin_timeout=10
net.ipv4.tcp_slow_start_after_idle=0
net.ipv4.tcp_keepalive_time=60
net.ipv4.tcp_keepalive_intvl=10
net.ipv4.tcp_keepalive_probes=6
net.ipv4.tcp_mtu_probing=1

# BBR congestion control
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr

# ===== KERNEL =====
# Aumentar limites de arquivos
fs.file-max=2097152
fs.inotify.max_user_watches=524288
fs.inotify.max_user_instances=1024

# Shared memory
kernel.shmmax=68719476736
kernel.shmall=4294967296

# Semáforos
kernel.sem=250 256000 100 1024

# Desabilitar NMI watchdog (economia de CPU)
kernel.nmi_watchdog=0

# Scheduler
kernel.sched_autogroup_enabled=1
kernel.sched_cfs_bandwidth_slice_us=3000

# ===== SEGURANÇA (relaxada para gaming) =====
kernel.unprivileged_bpf_disabled=0
kernel.perf_event_paranoid=1
```

### 7.2 I/O Schedulers

```bash
#!/bin/bash
# /usr/lib/winux/scripts/io-scheduler.sh

set_scheduler() {
    local device="$1"
    local device_name=$(basename "$device")
    local rotational=$(cat "/sys/block/${device_name}/queue/rotational" 2>/dev/null)

    if [[ "$rotational" == "0" ]]; then
        # SSD/NVMe - usar none ou mq-deadline
        echo "none" > "/sys/block/${device_name}/queue/scheduler" 2>/dev/null || \
        echo "mq-deadline" > "/sys/block/${device_name}/queue/scheduler"

        # Otimizações para SSD
        echo "0" > "/sys/block/${device_name}/queue/rotational"
        echo "256" > "/sys/block/${device_name}/queue/nr_requests"
        echo "0" > "/sys/block/${device_name}/queue/add_random"
    else
        # HDD - usar bfq
        echo "bfq" > "/sys/block/${device_name}/queue/scheduler"
        echo "128" > "/sys/block/${device_name}/queue/nr_requests"
    fi
}

# Aplicar para todos os dispositivos de bloco
for device in /sys/block/sd* /sys/block/nvme*; do
    if [[ -e "$device" ]]; then
        set_scheduler "$device"
    fi
done
```

### 7.3 CPU Governor e Power Management

```bash
# /etc/default/cpufrequtils
GOVERNOR="performance"

# /usr/lib/winux/scripts/cpu-setup.sh
#!/bin/bash

set_performance_mode() {
    # Definir governor
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        echo "performance" > "$cpu" 2>/dev/null
    done

    # Desabilitar boost temporariamente para testes
    # echo 0 > /sys/devices/system/cpu/cpufreq/boost

    # Intel P-State
    if [[ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]]; then
        echo 0 > /sys/devices/system/cpu/intel_pstate/no_turbo
        echo 0 > /sys/devices/system/cpu/intel_pstate/hwp_dynamic_boost
    fi

    # AMD P-State
    if [[ -f /sys/devices/system/cpu/amd_pstate/status ]]; then
        echo "active" > /sys/devices/system/cpu/amd_pstate/status
    fi
}

set_balanced_mode() {
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        echo "schedutil" > "$cpu" 2>/dev/null
    done
}

case "$1" in
    performance) set_performance_mode ;;
    balanced) set_balanced_mode ;;
    *) echo "Usage: $0 {performance|balanced}" ;;
esac
```

### 7.4 GameMode Integration

```ini
# /etc/gamemode.ini
# Winux GameMode Configuration

[general]
renice=10
ioprio=0
inhibit_screensaver=1

[gpu]
; Otimizações NVIDIA
apply_gpu_optimisations=accept-responsibility
gpu_device=0
nv_powermizer_mode=1
nv_core_clock_mhz_offset=100
nv_mem_clock_mhz_offset=200

; Otimizações AMD
amd_performance_level=high

[cpu]
; Usar governor performance
desiredgov=performance
park_cores=no

[custom]
; Scripts customizados
start=/usr/lib/winux/scripts/gamemode-start.sh
end=/usr/lib/winux/scripts/gamemode-end.sh
```

```bash
#!/bin/bash
# /usr/lib/winux/scripts/gamemode-start.sh

# Matar compositor se causando overhead
# pkill picom

# Desabilitar power saving
echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Priorizar processo do jogo
renice -n -5 -p $$

# Notificar
notify-send "GameMode" "Modo Gaming ativado" -i input-gaming

exit 0
```

### 7.5 Preload e Boot Optimization

```bash
# /etc/preload.conf
[system]
mapprefix = /usr/;/lib;/var/cache/
exeprefix = /usr/bin;/usr/lib

[model]
cycle = 20
usecorrelation = true
minsize = 2000000

# Systemd boot otimization
# /etc/systemd/system.conf.d/winux-boot.conf
[Manager]
DefaultTimeoutStartSec=10s
DefaultTimeoutStopSec=10s
DefaultDeviceTimeoutSec=10s
```

### 7.6 Performance Benchmarks Esperados

```yaml
Métricas de Performance Winux:

Boot Time:
  Cold Boot: < 5 segundos (NVMe SSD)
  Warm Boot: < 3 segundos
  Resume from Suspend: < 1 segundo

Desktop Responsiveness:
  App Launch (terminal): < 100ms
  App Launch (Files): < 200ms
  Window Switch: < 16ms (60fps)

Gaming Performance vs Windows:
  Native Linux Games: 100-110%
  Proton Games: 95-105%
  Wine Games: 90-100%

Memory Usage:
  Idle Desktop: < 800MB RAM
  With Browser: < 2GB RAM

Disk Performance:
  Sequential Read: Near native
  Random 4K: Near native
  Boot Drive Latency: < 50μs
```

---

## 8. Sistema de Build

### 8.1 Ferramentas de Build

```yaml
Build System Stack:
  Base Tool: Cubic (Custom Ubuntu ISO Creator)
  Alternative: Ubuntu Builder / live-build
  Chroot Management: debootstrap
  Package Building: dpkg-buildpackage
  ISO Creation: xorriso / mkisofs

Repository Structure:
  /winux-build/
  ├── build/                 # Build output
  ├── chroot/                # Chroot environment
  ├── config/                # Configuration files
  ├── packages/              # Custom .deb packages
  ├── scripts/               # Build scripts
  ├── overlay/               # Filesystem overlay
  └── iso/                   # Final ISO output
```

### 8.2 Script de Build Principal

```bash
#!/bin/bash
# /winux-build/scripts/build-winux-iso.sh

set -e

# Configurações
UBUNTU_VERSION="24.04"
UBUNTU_CODENAME="noble"
WINUX_VERSION="1.0"
WINUX_CODENAME="aurora"
BUILD_DIR="/winux-build"
CHROOT_DIR="${BUILD_DIR}/chroot"
ISO_DIR="${BUILD_DIR}/iso"

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# ===== FASE 1: Preparação =====
prepare_environment() {
    log_info "Preparando ambiente de build..."

    apt update
    apt install -y \
        debootstrap \
        squashfs-tools \
        xorriso \
        grub-pc-bin \
        grub-efi-amd64-bin \
        mtools \
        dosfstools

    mkdir -p "${BUILD_DIR}"/{build,chroot,packages,overlay,iso}
}

# ===== FASE 2: Debootstrap =====
create_base_system() {
    log_info "Criando sistema base Ubuntu ${UBUNTU_VERSION}..."

    debootstrap \
        --arch=amd64 \
        --variant=minbase \
        "${UBUNTU_CODENAME}" \
        "${CHROOT_DIR}" \
        http://archive.ubuntu.com/ubuntu

    # Montar filesystems necessários
    mount_chroot_filesystems
}

mount_chroot_filesystems() {
    mount --bind /dev "${CHROOT_DIR}/dev"
    mount --bind /dev/pts "${CHROOT_DIR}/dev/pts"
    mount -t proc none "${CHROOT_DIR}/proc"
    mount -t sysfs none "${CHROOT_DIR}/sys"
    mount -t tmpfs none "${CHROOT_DIR}/run"
}

umount_chroot_filesystems() {
    umount -l "${CHROOT_DIR}/run" 2>/dev/null || true
    umount -l "${CHROOT_DIR}/sys" 2>/dev/null || true
    umount -l "${CHROOT_DIR}/proc" 2>/dev/null || true
    umount -l "${CHROOT_DIR}/dev/pts" 2>/dev/null || true
    umount -l "${CHROOT_DIR}/dev" 2>/dev/null || true
}

# ===== FASE 3: Configurar Sistema =====
configure_base_system() {
    log_info "Configurando sistema base..."

    # Copiar resolv.conf
    cp /etc/resolv.conf "${CHROOT_DIR}/etc/"

    # Configurar sources.list
    cat > "${CHROOT_DIR}/etc/apt/sources.list" << EOF
deb http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME} main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME}-updates main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME}-security main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME}-backports main restricted universe multiverse
EOF

    # Executar configuração no chroot
    chroot "${CHROOT_DIR}" /bin/bash << 'CHROOT_SCRIPT'
        export DEBIAN_FRONTEND=noninteractive

        apt update
        apt upgrade -y

        # Instalar kernel zen e pacotes essenciais
        apt install -y \
            linux-image-generic \
            linux-headers-generic \
            systemd \
            systemd-boot \
            dbus \
            network-manager \
            pipewire \
            pipewire-pulse \
            wireplumber \
            sudo \
            locales \
            console-setup \
            keyboard-configuration

        # Configurar locale
        locale-gen en_US.UTF-8 pt_BR.UTF-8
        update-locale LANG=pt_BR.UTF-8

        # Configurar timezone
        ln -sf /usr/share/zoneinfo/America/Sao_Paulo /etc/localtime
CHROOT_SCRIPT
}

# ===== FASE 4: Instalar Desktop =====
install_desktop_environment() {
    log_info "Instalando Winux Shell..."

    chroot "${CHROOT_DIR}" /bin/bash << 'CHROOT_SCRIPT'
        export DEBIAN_FRONTEND=noninteractive

        # Dependências para Winux Shell
        apt install -y \
            wayland-protocols \
            libwayland-dev \
            libwlroots-dev \
            gtk4 \
            libgtk-4-dev \
            libadwaita-1-dev \
            fonts-jetbrains-mono \
            papirus-icon-theme

        # Instalar pacotes customizados Winux
        dpkg -i /tmp/packages/*.deb || apt install -f -y
CHROOT_SCRIPT
}

# ===== FASE 5: Instalar Drivers =====
install_drivers() {
    log_info "Configurando suporte a drivers..."

    chroot "${CHROOT_DIR}" /bin/bash << 'CHROOT_SCRIPT'
        export DEBIAN_FRONTEND=noninteractive

        # Mesa (AMD/Intel)
        apt install -y \
            mesa-vulkan-drivers \
            mesa-vulkan-drivers:i386 \
            libgl1-mesa-dri \
            libgl1-mesa-dri:i386 \
            vulkan-tools

        # NVIDIA (meta-package para detecção)
        apt install -y ubuntu-drivers-common
CHROOT_SCRIPT
}

# ===== FASE 6: Instalar Wine/Proton =====
install_wine_proton() {
    log_info "Instalando Wine e Proton..."

    chroot "${CHROOT_DIR}" /bin/bash << 'CHROOT_SCRIPT'
        export DEBIAN_FRONTEND=noninteractive

        # Adicionar repositório Wine
        dpkg --add-architecture i386
        mkdir -pm755 /etc/apt/keyrings
        wget -O /etc/apt/keyrings/winehq-archive.key https://dl.winehq.org/wine-builds/winehq.key
        wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/ubuntu/dists/$(lsb_release -cs)/winehq-$(lsb_release -cs).sources
        apt update

        # Instalar Wine Staging
        apt install -y --install-recommends winehq-staging winetricks

        # Instalar Steam
        apt install -y steam-installer
CHROOT_SCRIPT
}

# ===== FASE 7: Aplicar Overlay =====
apply_overlay() {
    log_info "Aplicando overlay de customização..."

    # Copiar arquivos de configuração
    cp -r "${BUILD_DIR}/overlay/"* "${CHROOT_DIR}/"

    # Copiar tema Winux
    cp -r "${BUILD_DIR}/themes/winux-fluent" "${CHROOT_DIR}/usr/share/themes/"

    # Copiar ícones
    cp -r "${BUILD_DIR}/icons/winux-icons" "${CHROOT_DIR}/usr/share/icons/"

    # Copiar wallpapers
    cp -r "${BUILD_DIR}/wallpapers" "${CHROOT_DIR}/usr/share/backgrounds/winux/"
}

# ===== FASE 8: Configurar Instalador =====
configure_installer() {
    log_info "Configurando Calamares..."

    chroot "${CHROOT_DIR}" /bin/bash << 'CHROOT_SCRIPT'
        apt install -y calamares calamares-settings-ubuntu

        # Configuração customizada será aplicada via overlay
CHROOT_SCRIPT
}

# ===== FASE 9: Cleanup =====
cleanup_chroot() {
    log_info "Limpando chroot..."

    chroot "${CHROOT_DIR}" /bin/bash << 'CHROOT_SCRIPT'
        apt clean
        apt autoremove -y
        rm -rf /tmp/* /var/tmp/*
        rm -rf /var/lib/apt/lists/*
        rm -f /etc/resolv.conf
CHROOT_SCRIPT
}

# ===== FASE 10: Criar ISO =====
create_iso() {
    log_info "Criando imagem ISO..."

    # Criar estrutura da ISO
    mkdir -p "${ISO_DIR}"/{casper,boot/grub,EFI/BOOT}

    # Criar squashfs
    log_info "Comprimindo filesystem (pode demorar)..."
    mksquashfs "${CHROOT_DIR}" "${ISO_DIR}/casper/filesystem.squashfs" \
        -comp zstd -Xcompression-level 19 -b 1M

    # Copiar kernel e initrd
    cp "${CHROOT_DIR}/boot/vmlinuz-"* "${ISO_DIR}/casper/vmlinuz"
    cp "${CHROOT_DIR}/boot/initrd.img-"* "${ISO_DIR}/casper/initrd"

    # Criar GRUB config
    cat > "${ISO_DIR}/boot/grub/grub.cfg" << 'EOF'
set timeout=5
set default=0

menuentry "Winux OS - Live Session" {
    linux /casper/vmlinuz boot=casper quiet splash ---
    initrd /casper/initrd
}

menuentry "Winux OS - Install" {
    linux /casper/vmlinuz boot=casper only-ubiquity quiet splash ---
    initrd /casper/initrd
}

menuentry "Winux OS - Safe Mode" {
    linux /casper/vmlinuz boot=casper nomodeset ---
    initrd /casper/initrd
}
EOF

    # Criar EFI boot image
    grub-mkstandalone \
        --format=x86_64-efi \
        --output="${ISO_DIR}/EFI/BOOT/BOOTX64.EFI" \
        --locales="" \
        --fonts="" \
        "boot/grub/grub.cfg=${ISO_DIR}/boot/grub/grub.cfg"

    # Criar ISO
    xorriso -as mkisofs \
        -iso-level 3 \
        -full-iso9660-filenames \
        -volid "WINUX_${WINUX_VERSION}" \
        -eltorito-boot boot/grub/bios.img \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        --eltorito-catalog boot/grub/boot.cat \
        --grub2-boot-info \
        --grub2-mbr /usr/lib/grub/i386-pc/boot_hybrid.img \
        -eltorito-alt-boot \
        -e EFI/efiboot.img \
        -no-emul-boot \
        -append_partition 2 0xef "${ISO_DIR}/EFI/efiboot.img" \
        -output "${BUILD_DIR}/winux-${WINUX_VERSION}-${WINUX_CODENAME}-amd64.iso" \
        -graft-points \
        "${ISO_DIR}"

    log_info "ISO criada: ${BUILD_DIR}/winux-${WINUX_VERSION}-${WINUX_CODENAME}-amd64.iso"
}

# ===== MAIN =====
main() {
    log_info "======================================"
    log_info "   Winux OS Build System v${WINUX_VERSION}"
    log_info "======================================"

    case "${1:-all}" in
        prepare)     prepare_environment ;;
        base)        create_base_system ;;
        configure)   configure_base_system ;;
        desktop)     install_desktop_environment ;;
        drivers)     install_drivers ;;
        wine)        install_wine_proton ;;
        overlay)     apply_overlay ;;
        installer)   configure_installer ;;
        cleanup)     cleanup_chroot ;;
        iso)         create_iso ;;
        all)
            prepare_environment
            create_base_system
            configure_base_system
            install_desktop_environment
            install_drivers
            install_wine_proton
            apply_overlay
            configure_installer
            cleanup_chroot
            umount_chroot_filesystems
            create_iso
            ;;
        *)
            echo "Usage: $0 {prepare|base|configure|desktop|drivers|wine|overlay|installer|cleanup|iso|all}"
            exit 1
            ;;
    esac

    log_info "Build concluído com sucesso!"
}

main "$@"
```

### 8.3 Configuração do Calamares

```yaml
# /winux-build/overlay/etc/calamares/settings.conf

modules-search: [ local ]

sequence:
  - show:
    - welcome
    - locale
    - keyboard
    - partition
    - users
    - summary
  - exec:
    - partition
    - mount
    - unpackfs
    - machineid
    - fstab
    - locale
    - keyboard
    - localecfg
    - users
    - displaymanager
    - networkcfg
    - hwclock
    - services-systemd
    - bootloader
    - umount
  - show:
    - finished

branding: winux

prompt-install: true
dont-chroot: false
```

```yaml
# /winux-build/overlay/etc/calamares/branding/winux/branding.desc

componentName: winux

strings:
    productName:         "Winux OS"
    shortProductName:    "Winux"
    version:             "1.0 Aurora"
    shortVersion:        "1.0"
    versionedName:       "Winux OS 1.0"
    shortVersionedName:  "Winux 1.0"
    bootloaderEntryName: "Winux"
    productUrl:          "https://winux.org"
    supportUrl:          "https://winux.org/support"
    knownIssuesUrl:      "https://winux.org/issues"
    releaseNotesUrl:     "https://winux.org/releases"

images:
    productLogo:         "winux-logo.png"
    productIcon:         "winux-icon.png"
    productWelcome:      "welcome.png"

slideshow:               "show.qml"

style:
   sidebarBackground:    "#1e1e2e"
   sidebarText:          "#cdd6f4"
   sidebarTextHighlight: "#f5e0dc"
```

---

## 9. Roadmap de Desenvolvimento

### 9.1 Visão Geral das Fases

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    WINUX DEVELOPMENT ROADMAP                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  FASE 1          FASE 2          FASE 3          FASE 4          FASE 5│
│  Base System     Desktop Env     Core Apps       Compatibility   Polish│
│                                                                         │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌───────┐│
│  │ Kernel  │───▶│ Winux   │───▶│ Files   │───▶│ Wine    │───▶│ QA    ││
│  │ Drivers │    │ Shell   │    │ Terminal│    │ Proton  │    │ Docs  ││
│  │ Base    │    │ Theme   │    │ Settings│    │ Gaming  │    │ ISO   ││
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘    └───────┘│
│                                                                         │
│  Timeline:                                                              │
│  ════════════════════════════════════════════════════════════════════  │
│  Mês 1-2         Mês 3-4         Mês 5-6         Mês 7-8       Mês 9-10│
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 9.2 Fase 1: Sistema Base

```yaml
Fase 1 - Sistema Base:
  Duração: Mês 1-2
  Objetivo: Sistema bootável funcional

  Tasks:
    Kernel:
      - [ ] Aplicar patches zen/bore
      - [ ] Configurar opções de kernel
      - [ ] Build e teste de kernel
      - [ ] Criar pacote .deb

    Drivers:
      - [ ] Script de detecção de hardware
      - [ ] Integração NVIDIA driver
      - [ ] Integração Mesa/RADV
      - [ ] Teste em múltiplas GPUs

    Sistema Base:
      - [ ] Configurar debootstrap
      - [ ] Pacotes essenciais
      - [ ] systemd otimizado
      - [ ] Network Manager
      - [ ] PipeWire audio

    Filesystem:
      - [ ] Layout Btrfs com subvolumes
      - [ ] Configuração de snapshots
      - [ ] Otimizações de mount

  Entregáveis:
    - Sistema bootável em terminal
    - Drivers funcionando
    - Rede e áudio operacionais
```

### 9.3 Fase 2: Desktop Environment

```yaml
Fase 2 - Desktop Environment:
  Duração: Mês 3-4
  Objetivo: Winux Shell funcional

  Tasks:
    Compositor:
      - [ ] Implementar compositor wlroots
      - [ ] XWayland compatibility
      - [ ] Window management básico
      - [ ] Suporte multi-monitor

    Shell Components:
      - [ ] Taskbar com app launcher
      - [ ] Menu Iniciar
      - [ ] System tray
      - [ ] Notifications daemon
      - [ ] Centro de Ações

    Tema Visual:
      - [ ] GTK4 theme (Fluent)
      - [ ] Icon theme
      - [ ] Cursor theme
      - [ ] Wallpapers padrão
      - [ ] Font configuration

    Window Management:
      - [ ] Snap layouts
      - [ ] Virtual desktops
      - [ ] Alt+Tab switcher
      - [ ] Window animations

  Entregáveis:
    - Desktop usável
    - Aparência Windows 11-like
    - UX fluida e responsiva
```

### 9.4 Fase 3: Aplicações Core

```yaml
Fase 3 - Core Apps:
  Duração: Mês 5-6
  Objetivo: Suite de apps nativa

  Tasks:
    Winux Files:
      - [ ] Navegação básica
      - [ ] Tabs e bookmarks
      - [ ] Preview de arquivos
      - [ ] Operações de arquivo
      - [ ] Drag and drop

    Winux Terminal:
      - [ ] Emulação VTE
      - [ ] GPU rendering
      - [ ] Tabs
      - [ ] Temas e configuração

    Winux Settings:
      - [ ] Módulo Display
      - [ ] Módulo Sound
      - [ ] Módulo Network
      - [ ] Módulo Appearance
      - [ ] Módulo Gaming

    Winux Store:
      - [ ] Backend Flatpak
      - [ ] Backend APT
      - [ ] UI de busca
      - [ ] Instalação de apps

    Winux Monitor:
      - [ ] Lista de processos
      - [ ] Gráficos de performance
      - [ ] Gerenciamento de startup

    Winux Edit:
      - [ ] Editor básico
      - [ ] Syntax highlighting
      - [ ] Múltiplas abas

  Entregáveis:
    - 6 aplicações funcionais
    - Integração com desktop
    - Qualidade production-ready
```

### 9.5 Fase 4: Compatibilidade Windows

```yaml
Fase 4 - Compatibility:
  Duração: Mês 7-8
  Objetivo: Executar software Windows

  Tasks:
    Wine Integration:
      - [ ] Wine Staging instalado
      - [ ] Configuração automática de prefix
      - [ ] winetricks integrado
      - [ ] Associação de .exe

    Proton/Gaming:
      - [ ] Steam instalado
      - [ ] Proton-GE disponível
      - [ ] DXVK/VKD3D configurado
      - [ ] GameMode integrado
      - [ ] MangoHud disponível

    Testing:
      - [ ] Testar 20+ jogos populares
      - [ ] Testar apps de produtividade
      - [ ] Performance benchmarks
      - [ ] Identificar problemas

    Launcher:
      - [ ] winux-run script
      - [ ] Auto-detect app type
      - [ ] Prefix management
      - [ ] Desktop entries

  Entregáveis:
    - Wine/Proton funcionando
    - Jogos AAA jogáveis
    - Performance competitiva
```

### 9.6 Fase 5: Polimento e Release

```yaml
Fase 5 - Polish & Release:
  Duração: Mês 9-10
  Objetivo: ISO release-ready

  Tasks:
    QA:
      - [ ] Testes em hardware real
      - [ ] Bug fixing sprint
      - [ ] Performance tuning
      - [ ] Security audit

    Installer:
      - [ ] Calamares configurado
      - [ ] Branding Winux
      - [ ] Slideshow de instalação
      - [ ] Teste de instalação

    Documentation:
      - [ ] User guide
      - [ ] Installation guide
      - [ ] FAQ
      - [ ] Contributing guide

    Release:
      - [ ] Build final da ISO
      - [ ] Checksums e assinaturas
      - [ ] Mirror setup
      - [ ] Announcement

  Entregáveis:
    - ISO 1.0 Aurora
    - Documentação completa
    - Website com downloads
```

---

## 10. Anexos Técnicos

### 10.1 Estrutura de Diretórios do Projeto

```
/winux-project/
├── README.md
├── LICENSE
├── CONTRIBUTING.md
│
├── build/                          # Sistema de build
│   ├── scripts/
│   │   ├── build-winux-iso.sh
│   │   ├── build-kernel.sh
│   │   └── build-packages.sh
│   ├── config/
│   │   ├── kernel-config
│   │   ├── calamares/
│   │   └── grub/
│   └── Makefile
│
├── kernel/                         # Kernel customizado
│   ├── patches/
│   │   ├── bore-scheduler.patch
│   │   ├── zen-tweaks.patch
│   │   └── winesync.patch
│   └── config/
│       └── winux-kernel-config
│
├── desktop/                        # Desktop Environment
│   ├── winux-compositor/           # Compositor Wayland
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── winux-panel/               # Taskbar + Start Menu
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── winux-shell/               # Session manager
│   │   ├── Cargo.toml
│   │   └── src/
│   └── themes/
│       ├── winux-fluent/          # GTK theme
│       ├── winux-icons/           # Icon theme
│       └── winux-cursors/         # Cursor theme
│
├── apps/                           # Aplicações nativas
│   ├── winux-files/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── winux-terminal/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── winux-settings/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── winux-store/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── winux-monitor/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── winux-edit/
│       ├── Cargo.toml
│       └── src/
│
├── compatibility/                  # Camada Windows
│   ├── wine-config/
│   │   ├── registry-tweaks.reg
│   │   └── dlloverrides.conf
│   ├── scripts/
│   │   ├── winux-run
│   │   ├── wine-setup.sh
│   │   └── proton-install.sh
│   └── games-db/
│       └── games.json
│
├── drivers/                        # Scripts de drivers
│   ├── nvidia-install.sh
│   ├── amd-install.sh
│   ├── intel-install.sh
│   └── winux-driver-manager
│
├── system/                         # Configurações de sistema
│   ├── etc/
│   │   ├── sysctl.d/
│   │   ├── modprobe.d/
│   │   ├── systemd/
│   │   ├── environment.d/
│   │   └── xdg/
│   └── usr/
│       └── share/
│           ├── applications/
│           ├── icons/
│           └── backgrounds/
│
├── docs/                           # Documentação
│   ├── user-guide/
│   ├── developer-guide/
│   ├── api-reference/
│   └── architecture/
│
└── tests/                          # Testes
    ├── unit/
    ├── integration/
    └── e2e/
```

### 10.2 Cargo Workspace Configuration

```toml
# /winux-project/Cargo.toml

[workspace]
resolver = "2"
members = [
    "desktop/winux-compositor",
    "desktop/winux-panel",
    "desktop/winux-shell",
    "apps/winux-files",
    "apps/winux-terminal",
    "apps/winux-settings",
    "apps/winux-store",
    "apps/winux-monitor",
    "apps/winux-edit",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
authors = ["Winux Team <team@winux.org>"]
license = "MIT OR GPL-3.0"
repository = "https://github.com/winux-os/winux"

[workspace.dependencies]
# GTK
gtk4 = "0.7"
libadwaita = "0.5"
relm4 = "0.7"
glib = "0.18"
gio = "0.18"

# Async
tokio = { version = "1", features = ["full"] }
async-channel = "2.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
ron = "0.8"

# Utilities
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = "0.4"
dirs = "5.0"

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
strip = true
```

### 10.3 Exemplo de Configuração Completa

```bash
# /etc/winux/winux.conf
# Configuração global do Winux OS

[system]
# Nome do sistema
name = "Winux OS"
version = "1.0"
codename = "Aurora"

# Atualizações
auto_update = false
update_channel = "stable"

[desktop]
# Compositor
compositor = "winux-compositor"
wayland_enabled = true
xwayland_enabled = true

# Tema
theme = "winux-fluent-dark"
icon_theme = "Papirus-Dark"
cursor_theme = "Adwaita"
font = "Inter"
monospace_font = "JetBrains Mono"

# Animações
animations_enabled = true
animation_speed = 1.0

[gaming]
# GameMode
gamemode_enabled = true
gamemode_auto = true

# MangoHud
mangohud_enabled = false
mangohud_position = "top-left"

# Wine
wine_version = "staging"
wine_prefix = "~/.wine"
dxvk_async = true

# Steam
proton_default = "GE-Proton-latest"
shader_precache = true

[performance]
# CPU
cpu_governor = "schedutil"  # ou "performance" para gaming

# GPU
vsync = "adaptive"

# Memória
swappiness = 10
zram_enabled = true
zram_size = "50%"

[privacy]
# Telemetria
telemetry_enabled = false
crash_reports = true
```

### 10.4 Scripts de Utilidade

```bash
#!/bin/bash
# /usr/bin/winux-setup
# Assistente de configuração inicial do Winux

show_welcome() {
    clear
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║                                                          ║"
    echo "║   ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗              ║"
    echo "║   ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝              ║"
    echo "║   ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝               ║"
    echo "║   ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗               ║"
    echo "║   ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗              ║"
    echo "║    ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝              ║"
    echo "║                                                          ║"
    echo "║            Bem-vindo ao Winux OS 1.0 Aurora              ║"
    echo "║                                                          ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
    echo "Este assistente irá configurar seu sistema."
    echo ""
    read -p "Pressione ENTER para continuar..."
}

configure_drivers() {
    echo ""
    echo "══════════════════════════════════════════════════════════"
    echo "                    CONFIGURAÇÃO DE DRIVERS               "
    echo "══════════════════════════════════════════════════════════"
    echo ""

    /usr/bin/winux-driver-manager
}

configure_gaming() {
    echo ""
    echo "══════════════════════════════════════════════════════════"
    echo "                    CONFIGURAÇÃO DE GAMING                "
    echo "══════════════════════════════════════════════════════════"
    echo ""

    echo "Deseja instalar o Steam? [Y/n]"
    read -r response
    if [[ "$response" != "n" && "$response" != "N" ]]; then
        apt install -y steam-installer
    fi

    echo ""
    echo "Deseja instalar o Lutris? [Y/n]"
    read -r response
    if [[ "$response" != "n" && "$response" != "N" ]]; then
        apt install -y lutris
    fi

    echo ""
    echo "Configurando Wine..."
    /usr/lib/winux/scripts/wine-setup.sh

    echo ""
    echo "Instalando Proton-GE..."
    /usr/lib/winux/scripts/proton-install.sh
}

configure_appearance() {
    echo ""
    echo "══════════════════════════════════════════════════════════"
    echo "                    CONFIGURAÇÃO DE APARÊNCIA             "
    echo "══════════════════════════════════════════════════════════"
    echo ""

    echo "Escolha o tema:"
    echo "1) Escuro (padrão)"
    echo "2) Claro"
    echo "3) Automático (segue horário)"
    read -p "Opção [1]: " theme_choice

    case "$theme_choice" in
        2) gsettings set org.gnome.desktop.interface color-scheme 'prefer-light' ;;
        3) # Configurar automatic theme switching
           ;;
        *) gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark' ;;
    esac
}

show_finish() {
    clear
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║                                                          ║"
    echo "║              CONFIGURAÇÃO CONCLUÍDA!                     ║"
    echo "║                                                          ║"
    echo "║  Seu Winux OS está pronto para uso.                      ║"
    echo "║                                                          ║"
    echo "║  Próximos passos:                                        ║"
    echo "║  • Explore o Menu Iniciar                                ║"
    echo "║  • Configure suas preferências em Configurações          ║"
    echo "║  • Instale seus apps favoritos na Winux Store            ║"
    echo "║  • Divirta-se!                                           ║"
    echo "║                                                          ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
}

main() {
    show_welcome
    configure_drivers
    configure_gaming
    configure_appearance
    show_finish
}

main "$@"
```

### 10.5 Checklist de Verificação Final

```markdown
# Winux OS 1.0 Aurora - Release Checklist

## Sistema Base
- [ ] Boot em < 5 segundos
- [ ] Kernel zen funcionando
- [ ] Drivers NVIDIA/AMD/Intel detectados
- [ ] Áudio PipeWire funcionando
- [ ] Rede conectando automaticamente
- [ ] Bluetooth pareando dispositivos

## Desktop Environment
- [ ] Compositor Wayland estável
- [ ] XWayland para apps legados
- [ ] Taskbar funcional
- [ ] Menu Iniciar abrindo
- [ ] System tray com ícones
- [ ] Notificações aparecendo
- [ ] Snap layouts funcionando
- [ ] Virtual desktops OK

## Aplicações
- [ ] Winux Files navegando
- [ ] Winux Terminal executando comandos
- [ ] Winux Settings salvando preferências
- [ ] Winux Store instalando apps
- [ ] Winux Monitor mostrando processos
- [ ] Winux Edit editando arquivos

## Gaming
- [ ] Steam instalando jogos
- [ ] Proton-GE disponível
- [ ] Wine executando .exe
- [ ] GameMode ativando
- [ ] MangoHud exibindo FPS
- [ ] 95%+ performance vs Windows

## Compatibilidade
- [ ] Flatpak funcionando
- [ ] Snap funcionando
- [ ] APT funcionando
- [ ] .exe associado ao Wine
- [ ] Office apps via Wine

## Instalador
- [ ] Calamares iniciando
- [ ] Particionamento funcionando
- [ ] Instalação completando
- [ ] Bootloader instalado
- [ ] Primeiro boot OK

## Documentação
- [ ] README atualizado
- [ ] User guide completo
- [ ] FAQ respondendo dúvidas
- [ ] Website publicado
```

---

## Conclusão

Este documento fornece uma especificação técnica completa para o desenvolvimento do **Winux OS**, um sistema operacional Linux customizado focado em gaming e produtividade. O projeto combina a estabilidade do Ubuntu com uma interface inspirada no Windows 11 e uma suite completa de aplicações nativas desenvolvidas em Rust.

### Tecnologias Principais:
- **Base:** Ubuntu 24.04 LTS com kernel zen customizado
- **Desktop:** Compositor Wayland próprio com tema Fluent Design
- **Apps:** 6 aplicações nativas em Rust (gtk4-rs + relm4)
- **Gaming:** Wine Staging, Proton-GE, DXVK, GameMode
- **Performance:** Otimizações agressivas de kernel e sistema

### Próximos Passos:
1. Revisar e aprovar especificações
2. Configurar ambiente de desenvolvimento
3. Iniciar Fase 1 (Sistema Base)
4. Iterar baseado em feedback

---

**Documento gerado para Claude Opus 4.5**
**Winux OS Project - 2026**

