#!/bin/bash
# =============================================================================
# Winux OS - Complete Development Environment Setup
# =============================================================================
# Script completo para configurar ambiente de desenvolvimento profissional
# Inclui: .NET, C/C++, Rust, Apple Build Tools, Cross-Platform, Node.js,
#         Python, Java, Go, Zig, Containers e muito mais
# =============================================================================

set -e

# =============================================================================
# Cores e formatacao
# =============================================================================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
NC='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'

# =============================================================================
# Funcoes de log
# =============================================================================
log_info() { echo -e "${GREEN}[+]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[x]${NC} $1"; }
log_step() { echo -e "${BLUE}[>]${NC} ${BOLD}$1${NC}"; }
log_section() {
    echo ""
    echo -e "${MAGENTA}==============================================================================${NC}"
    echo -e "${MAGENTA}  $1${NC}"
    echo -e "${MAGENTA}==============================================================================${NC}"
}
log_skip() { echo -e "${DIM}[-] $1 (ja instalado)${NC}"; }

# =============================================================================
# Logo Winux
# =============================================================================
show_logo() {
    echo -e "${CYAN}"
    cat << 'EOF'

    ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗     ██████╗ ███████╗
    ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝    ██╔═══██╗██╔════╝
    ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝     ██║   ██║███████╗
    ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗     ██║   ██║╚════██║
    ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗    ╚██████╔╝███████║
     ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝     ╚═════╝ ╚══════╝

    ╔══════════════════════════════════════════════════════════════╗
    ║      Complete Development Environment Setup                  ║
    ║      Version 2.0 - Professional Developer Edition            ║
    ╚══════════════════════════════════════════════════════════════╝

EOF
    echo -e "${NC}"
}

# =============================================================================
# Verificacoes iniciais
# =============================================================================
check_requirements() {
    log_section "Verificando requisitos do sistema"

    # Verificar se nao e root
    if [[ $EUID -eq 0 ]]; then
        log_error "Nao execute como root. Use seu usuario normal."
        exit 1
    fi

    # Verificar distribuicao
    if [[ ! -f /etc/os-release ]]; then
        log_error "Sistema operacional nao suportado"
        exit 1
    fi

    source /etc/os-release
    log_info "Sistema detectado: $PRETTY_NAME"

    # Verificar arquitetura
    ARCH=$(uname -m)
    log_info "Arquitetura: $ARCH"

    # Verificar espaco em disco (minimo 20GB)
    AVAILABLE_SPACE=$(df -BG / | awk 'NR==2 {print $4}' | sed 's/G//')
    if [[ $AVAILABLE_SPACE -lt 20 ]]; then
        log_warn "Espaco em disco baixo: ${AVAILABLE_SPACE}GB disponivel (recomendado: 20GB+)"
    else
        log_info "Espaco em disco: ${AVAILABLE_SPACE}GB disponivel"
    fi

    # Verificar memoria
    TOTAL_MEM=$(free -g | awk '/^Mem:/{print $2}')
    log_info "Memoria RAM: ${TOTAL_MEM}GB"

    # Verificar conexao com internet
    if ping -c 1 google.com &> /dev/null; then
        log_info "Conexao com internet: OK"
    else
        log_error "Sem conexao com internet"
        exit 1
    fi
}

# =============================================================================
# Funcao auxiliar para verificar se comando existe
# =============================================================================
command_exists() {
    command -v "$1" &> /dev/null
}

# =============================================================================
# Atualizar sistema e instalar dependencias basicas
# =============================================================================
setup_base() {
    log_section "Atualizando sistema e instalando dependencias base"

    sudo apt update
    sudo apt upgrade -y

    # Dependencias essenciais para compilacao
    sudo apt install -y \
        apt-transport-https \
        ca-certificates \
        curl \
        wget \
        gnupg \
        lsb-release \
        software-properties-common \
        build-essential \
        pkg-config \
        libssl-dev \
        libffi-dev \
        zlib1g-dev \
        libbz2-dev \
        libreadline-dev \
        libsqlite3-dev \
        libncurses5-dev \
        libncursesw5-dev \
        xz-utils \
        tk-dev \
        libxml2-dev \
        libxmlsec1-dev \
        liblzma-dev \
        libgdbm-dev \
        libnss3-dev \
        uuid-dev

    log_info "Sistema base atualizado"
}

# =============================================================================
# 1. ECOSSISTEMA .NET
# =============================================================================
setup_dotnet() {
    log_section "1. Configurando Ecossistema .NET"

    # Verificar se .NET ja esta instalado
    if command_exists dotnet && dotnet --list-sdks | grep -q "8.0"; then
        log_skip ".NET SDK 8.0 ja instalado"
        dotnet --list-sdks
    else
        log_step "Instalando .NET SDK 8.0 (LTS)..."

        # Adicionar repositorio Microsoft
        wget -q https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb -O /tmp/packages-microsoft-prod.deb
        sudo dpkg -i /tmp/packages-microsoft-prod.deb
        rm /tmp/packages-microsoft-prod.deb

        sudo apt update

        # Instalar .NET SDK 8.0
        sudo apt install -y dotnet-sdk-8.0

        log_info ".NET SDK 8.0 instalado"
    fi

    # ASP.NET Core Runtime
    if ! dpkg -l | grep -q "aspnetcore-runtime-8.0"; then
        log_step "Instalando ASP.NET Core Runtime..."
        sudo apt install -y aspnetcore-runtime-8.0
        log_info "ASP.NET Core Runtime instalado"
    else
        log_skip "ASP.NET Core Runtime ja instalado"
    fi

    # dotnet-ef tools
    if ! dotnet tool list -g | grep -q "dotnet-ef"; then
        log_step "Instalando dotnet-ef tools..."
        dotnet tool install --global dotnet-ef
        log_info "dotnet-ef instalado"
    else
        log_skip "dotnet-ef ja instalado"
    fi

    # Outras ferramentas .NET uteis
    log_step "Instalando ferramentas .NET adicionais..."
    dotnet tool install --global dotnet-outdated-tool 2>/dev/null || true
    dotnet tool install --global dotnet-format 2>/dev/null || true
    dotnet tool install --global dotnet-reportgenerator-globaltool 2>/dev/null || true

    # Configurar PATH para tools .NET
    if ! grep -q ".dotnet/tools" "$HOME/.bashrc" 2>/dev/null; then
        echo 'export PATH="$PATH:$HOME/.dotnet/tools"' >> "$HOME/.bashrc"
    fi

    # OmniSharp para LSP (usado por VS Code e outros editores)
    log_step "Configurando OmniSharp LSP..."
    OMNISHARP_DIR="$HOME/.omnisharp"
    if [[ ! -d "$OMNISHARP_DIR" ]]; then
        mkdir -p "$OMNISHARP_DIR"
        OMNISHARP_VERSION="v1.39.11"
        wget -q "https://github.com/OmniSharp/omnisharp-roslyn/releases/download/${OMNISHARP_VERSION}/omnisharp-linux-x64-net6.0.tar.gz" -O /tmp/omnisharp.tar.gz
        tar -xzf /tmp/omnisharp.tar.gz -C "$OMNISHARP_DIR"
        rm /tmp/omnisharp.tar.gz
        log_info "OmniSharp instalado em $OMNISHARP_DIR"
    else
        log_skip "OmniSharp ja instalado"
    fi

    echo ""
    log_info ".NET Ecosystem: $(dotnet --version)"
}

# =============================================================================
# 2. ECOSSISTEMA C/C++
# =============================================================================
setup_cpp() {
    log_section "2. Configurando Ecossistema C/C++"

    # GCC 13+
    if command_exists gcc && gcc --version | head -1 | grep -qE "1[3-9]\.|[2-9][0-9]\."; then
        log_skip "GCC 13+ ja instalado"
    else
        log_step "Instalando GCC 13..."
        sudo add-apt-repository -y ppa:ubuntu-toolchain-r/test 2>/dev/null || true
        sudo apt update
        sudo apt install -y gcc-13 g++-13

        # Configurar como padrao
        sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-13 130 --slave /usr/bin/g++ g++ /usr/bin/g++-13
        log_info "GCC 13 instalado e configurado como padrao"
    fi

    # Clang 17+
    if command_exists clang && clang --version | head -1 | grep -qE "1[7-9]\.|[2-9][0-9]\."; then
        log_skip "Clang 17+ ja instalado"
    else
        log_step "Instalando Clang 17..."
        wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | sudo tee /etc/apt/trusted.gpg.d/apt.llvm.org.asc

        CODENAME=$(lsb_release -cs)
        echo "deb http://apt.llvm.org/${CODENAME}/ llvm-toolchain-${CODENAME}-17 main" | sudo tee /etc/apt/sources.list.d/llvm-17.list

        sudo apt update
        sudo apt install -y clang-17 lldb-17 lld-17 clangd-17

        # Criar links simbolicos
        sudo update-alternatives --install /usr/bin/clang clang /usr/bin/clang-17 170
        sudo update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-17 170
        sudo update-alternatives --install /usr/bin/clangd clangd /usr/bin/clangd-17 170

        log_info "Clang 17 instalado"
    fi

    # CMake
    if command_exists cmake; then
        log_skip "CMake ja instalado"
    else
        log_step "Instalando CMake..."
        sudo apt install -y cmake
        log_info "CMake instalado"
    fi

    # Ninja
    if command_exists ninja; then
        log_skip "Ninja ja instalado"
    else
        log_step "Instalando Ninja..."
        sudo apt install -y ninja-build
        log_info "Ninja instalado"
    fi

    # ccache
    if command_exists ccache; then
        log_skip "ccache ja instalado"
    else
        log_step "Instalando ccache..."
        sudo apt install -y ccache

        # Configurar ccache
        mkdir -p "$HOME/.ccache"
        cat > "$HOME/.ccache/ccache.conf" << 'CCACHE'
max_size = 10G
compression = true
compression_level = 6
CCACHE

        log_info "ccache instalado e configurado"
    fi

    # GDB
    if command_exists gdb; then
        log_skip "GDB ja instalado"
    else
        log_step "Instalando GDB..."
        sudo apt install -y gdb
        log_info "GDB instalado"
    fi

    # LLDB
    if command_exists lldb; then
        log_skip "LLDB ja instalado"
    else
        log_step "Instalando LLDB..."
        sudo apt install -y lldb-17 || sudo apt install -y lldb
        sudo update-alternatives --install /usr/bin/lldb lldb /usr/bin/lldb-17 170 2>/dev/null || true
        log_info "LLDB instalado"
    fi

    # Valgrind
    if command_exists valgrind; then
        log_skip "Valgrind ja instalado"
    else
        log_step "Instalando Valgrind..."
        sudo apt install -y valgrind
        log_info "Valgrind instalado"
    fi

    # clang-format e clang-tidy
    log_step "Instalando clang-format e clang-tidy..."
    sudo apt install -y clang-format-17 clang-tidy-17 2>/dev/null || sudo apt install -y clang-format clang-tidy
    sudo update-alternatives --install /usr/bin/clang-format clang-format /usr/bin/clang-format-17 170 2>/dev/null || true
    sudo update-alternatives --install /usr/bin/clang-tidy clang-tidy /usr/bin/clang-tidy-17 170 2>/dev/null || true

    # Ferramentas adicionais de C/C++
    log_step "Instalando ferramentas adicionais C/C++..."
    sudo apt install -y \
        cppcheck \
        include-what-you-use \
        libboost-all-dev \
        libeigen3-dev \
        libgtest-dev \
        libgmock-dev \
        doxygen \
        graphviz

    echo ""
    log_info "GCC: $(gcc --version | head -1)"
    log_info "Clang: $(clang --version | head -1)"
    log_info "CMake: $(cmake --version | head -1)"
}

# =============================================================================
# 3. RUST TOOLCHAIN
# =============================================================================
setup_rust() {
    log_section "3. Configurando Rust Toolchain"

    # Rustup
    if command_exists rustup; then
        log_skip "Rustup ja instalado"
        rustup update
    else
        log_step "Instalando Rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        source "$HOME/.cargo/env"
        log_info "Rustup instalado"
    fi

    # Garantir que cargo esta no PATH
    export PATH="$HOME/.cargo/bin:$PATH"
    source "$HOME/.cargo/env" 2>/dev/null || true

    # Instalar toolchains
    log_step "Configurando toolchains Rust..."
    rustup toolchain install stable
    rustup toolchain install nightly
    rustup default stable

    # Componentes essenciais
    log_step "Instalando componentes Rust..."
    rustup component add rustfmt clippy rust-src rust-analyzer

    # Targets para cross-compilation
    log_step "Adicionando targets de cross-compilation..."
    rustup target add x86_64-unknown-linux-gnu
    rustup target add x86_64-unknown-linux-musl
    rustup target add x86_64-pc-windows-gnu
    rustup target add x86_64-pc-windows-msvc
    rustup target add wasm32-unknown-unknown
    rustup target add wasm32-wasi
    rustup target add aarch64-unknown-linux-gnu

    # Ferramentas Cargo
    log_step "Instalando ferramentas Cargo..."

    # cargo-watch
    if ! cargo install --list | grep -q "cargo-watch"; then
        cargo install cargo-watch
    else
        log_skip "cargo-watch ja instalado"
    fi

    # cargo-edit (add, rm, upgrade)
    if ! cargo install --list | grep -q "cargo-edit"; then
        cargo install cargo-edit
    else
        log_skip "cargo-edit ja instalado"
    fi

    # cargo-audit (security)
    if ! cargo install --list | grep -q "cargo-audit"; then
        cargo install cargo-audit
    else
        log_skip "cargo-audit ja instalado"
    fi

    # Outras ferramentas uteis
    log_step "Instalando ferramentas Cargo adicionais..."
    cargo install cargo-outdated 2>/dev/null || true
    cargo install cargo-expand 2>/dev/null || true
    cargo install cargo-make 2>/dev/null || true
    cargo install cargo-deny 2>/dev/null || true
    cargo install cargo-nextest 2>/dev/null || true
    cargo install sccache 2>/dev/null || true
    cargo install tokei 2>/dev/null || true
    cargo install just 2>/dev/null || true

    # Configurar sccache para Rust
    if command_exists sccache; then
        mkdir -p "$HOME/.cargo"
        if ! grep -q "RUSTC_WRAPPER" "$HOME/.cargo/env" 2>/dev/null; then
            echo 'export RUSTC_WRAPPER=sccache' >> "$HOME/.cargo/env"
        fi
    fi

    echo ""
    log_info "Rust: $(rustc --version)"
    log_info "Cargo: $(cargo --version)"
}

# =============================================================================
# 4. BUILD TOOLS PARA APPLE
# =============================================================================
setup_apple_tools() {
    log_section "4. Configurando Build Tools para Apple"

    # Darling (macOS compatibility layer)
    log_step "Verificando Darling..."
    if command_exists darling; then
        log_skip "Darling ja instalado"
    else
        log_step "Instalando dependencias para Darling..."
        sudo apt install -y \
            cmake \
            clang \
            bison \
            flex \
            xz-utils \
            libfuse-dev \
            libudev-dev \
            pkg-config \
            libc6-dev-i386 \
            libcap2-bin \
            git \
            libglu1-mesa-dev \
            libcairo2-dev \
            libgl1-mesa-dev \
            libtiff5-dev \
            libfreetype6-dev \
            libelf-dev \
            libxml2-dev \
            libegl1-mesa-dev \
            libfontconfig1-dev \
            libbsd-dev \
            libxrandr-dev \
            libxcursor-dev \
            libgif-dev \
            libpulse-dev \
            libavformat-dev \
            libavcodec-dev \
            libswresample-dev \
            libdbus-1-dev \
            libxkbfile-dev \
            libssl-dev \
            llvm-dev 2>/dev/null || true

        log_warn "Darling requer compilacao manual. Clone de: https://github.com/darlinghq/darling"
        log_info "Instrucoes: git clone --recursive https://github.com/darlinghq/darling.git && cd darling && mkdir build && cd build && cmake .. && make && sudo make install"
    fi

    # create-dmg
    if command_exists create-dmg; then
        log_skip "create-dmg ja instalado"
    else
        log_step "Instalando create-dmg..."
        sudo apt install -y genisoimage

        if [[ ! -d "/opt/create-dmg" ]]; then
            sudo git clone https://github.com/create-dmg/create-dmg.git /opt/create-dmg
            sudo ln -sf /opt/create-dmg/create-dmg /usr/local/bin/create-dmg
        fi
        log_info "create-dmg instalado"
    fi

    # appimagetool
    if command_exists appimagetool; then
        log_skip "appimagetool ja instalado"
    else
        log_step "Instalando appimagetool..."
        wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" -O /tmp/appimagetool
        chmod +x /tmp/appimagetool
        sudo mv /tmp/appimagetool /usr/local/bin/appimagetool
        log_info "appimagetool instalado"
    fi

    # bomutils (para criar arquivos .pkg do macOS)
    if command_exists mkbom; then
        log_skip "bomutils ja instalado"
    else
        log_step "Instalando bomutils..."
        if [[ ! -d "/tmp/bomutils" ]]; then
            git clone https://github.com/hogliux/bomutils.git /tmp/bomutils
            cd /tmp/bomutils
            make
            sudo make install
            cd -
            rm -rf /tmp/bomutils
        fi
        log_info "bomutils instalado"
    fi

    # xar (para manipular arquivos .pkg)
    if command_exists xar; then
        log_skip "xar ja instalado"
    else
        log_step "Instalando xar..."
        sudo apt install -y libxml2-dev libssl-dev zlib1g-dev libbz2-dev

        if [[ ! -d "/tmp/xar" ]]; then
            git clone https://github.com/mackyle/xar.git /tmp/xar
            cd /tmp/xar/xar
            ./autogen.sh --noconfigure
            ./configure
            make
            sudo make install
            cd -
            rm -rf /tmp/xar
        fi
        log_info "xar instalado"
    fi

    # libimobiledevice (para desenvolvimento iOS)
    if command_exists ideviceinfo; then
        log_skip "libimobiledevice ja instalado"
    else
        log_step "Instalando libimobiledevice..."
        sudo apt install -y \
            libimobiledevice6 \
            libimobiledevice-dev \
            libimobiledevice-utils \
            ideviceinstaller \
            libplist-dev \
            libplist-utils \
            libusbmuxd-dev \
            libusbmuxd-tools \
            usbmuxd
        log_info "libimobiledevice instalado"
    fi

    echo ""
    log_info "Apple Build Tools configurados"
}

# =============================================================================
# 5. FERRAMENTAS DE BUILD CROSS-PLATFORM
# =============================================================================
setup_crossplatform() {
    log_section "5. Configurando Ferramentas Cross-Platform"

    # osxcross (cross-compile para macOS)
    log_step "Verificando osxcross..."
    if [[ -d "/opt/osxcross" ]] || [[ -d "$HOME/osxcross" ]]; then
        log_skip "osxcross ja instalado"
    else
        log_step "Instalando dependencias para osxcross..."
        sudo apt install -y \
            clang \
            cmake \
            git \
            patch \
            python3 \
            libssl-dev \
            lzma-dev \
            libxml2-dev \
            xz-utils \
            bzip2 \
            cpio \
            zlib1g-dev

        log_warn "osxcross requer SDK do macOS (XCode). Clone de: https://github.com/tpoechtrager/osxcross"
        log_info "Instrucoes:"
        log_info "  1. git clone https://github.com/tpoechtrager/osxcross.git"
        log_info "  2. Obter MacOSX SDK (.sdk.tar.xz) e colocar em osxcross/tarballs/"
        log_info "  3. cd osxcross && ./build.sh"
    fi

    # mingw-w64 (cross-compile para Windows)
    if command_exists x86_64-w64-mingw32-gcc; then
        log_skip "mingw-w64 ja instalado"
    else
        log_step "Instalando mingw-w64..."
        sudo apt install -y \
            mingw-w64 \
            mingw-w64-tools \
            mingw-w64-common \
            mingw-w64-x86-64-dev \
            gcc-mingw-w64 \
            g++-mingw-w64 \
            wine64
        log_info "mingw-w64 instalado"
    fi

    # flatpak-builder
    if command_exists flatpak-builder; then
        log_skip "flatpak-builder ja instalado"
    else
        log_step "Instalando flatpak-builder..."
        sudo apt install -y flatpak flatpak-builder

        # Adicionar repositorio Flathub
        sudo flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

        # Instalar runtime base
        flatpak install -y flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08 2>/dev/null || true

        log_info "flatpak-builder instalado"
    fi

    # snapcraft
    if command_exists snapcraft; then
        log_skip "snapcraft ja instalado"
    else
        log_step "Instalando snapcraft..."
        sudo snap install snapcraft --classic
        log_info "snapcraft instalado"
    fi

    # Ferramentas adicionais de packaging
    log_step "Instalando ferramentas adicionais de packaging..."
    sudo apt install -y \
        dpkg-dev \
        debhelper \
        devscripts \
        fakeroot \
        lintian \
        rpm \
        alien \
        nsis 2>/dev/null || true

    echo ""
    log_info "Cross-Platform Tools configurados"
}

# =============================================================================
# 6. NODE.JS / WEB
# =============================================================================
setup_nodejs() {
    log_section "6. Configurando Node.js / Web Development"

    # NVM
    export NVM_DIR="$HOME/.nvm"

    if [[ -d "$NVM_DIR" ]]; then
        log_skip "NVM ja instalado"
        [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    else
        log_step "Instalando NVM..."
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash

        [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
        log_info "NVM instalado"
    fi

    # Node.js LTS
    log_step "Instalando Node.js LTS..."
    nvm install --lts
    nvm use --lts
    nvm alias default node

    # pnpm
    if command_exists pnpm; then
        log_skip "pnpm ja instalado"
    else
        log_step "Instalando pnpm..."
        npm install -g pnpm
        log_info "pnpm instalado"
    fi

    # yarn
    if command_exists yarn; then
        log_skip "yarn ja instalado"
    else
        log_step "Instalando yarn..."
        npm install -g yarn
        log_info "yarn instalado"
    fi

    # Outras ferramentas Node.js
    log_step "Instalando ferramentas Node.js..."
    npm install -g \
        typescript \
        ts-node \
        tsx \
        nodemon \
        pm2 \
        eslint \
        prettier \
        @angular/cli \
        @vue/cli \
        create-react-app \
        create-next-app \
        vite \
        turbo \
        nx \
        serve \
        http-server \
        json-server \
        vercel \
        netlify-cli 2>/dev/null || true

    # Electron
    if npm list -g electron &>/dev/null; then
        log_skip "Electron ja instalado"
    else
        log_step "Instalando Electron..."
        npm install -g electron
        npm install -g @electron-forge/cli
        log_info "Electron instalado"
    fi

    # Tauri CLI
    log_step "Instalando Tauri CLI..."
    npm install -g @tauri-apps/cli 2>/dev/null || true

    # Bun (runtime alternativo)
    if command_exists bun; then
        log_skip "Bun ja instalado"
    else
        log_step "Instalando Bun..."
        curl -fsSL https://bun.sh/install | bash
        log_info "Bun instalado"
    fi

    # Deno
    if command_exists deno; then
        log_skip "Deno ja instalado"
    else
        log_step "Instalando Deno..."
        curl -fsSL https://deno.land/install.sh | sh
        log_info "Deno instalado"
    fi

    echo ""
    log_info "Node.js: $(node --version)"
    log_info "npm: $(npm --version)"
    log_info "pnpm: $(pnpm --version 2>/dev/null || echo 'instalado')"
}

# =============================================================================
# 7. PYTHON
# =============================================================================
setup_python() {
    log_section "7. Configurando Python"

    # Python 3.12
    if python3.12 --version &>/dev/null; then
        log_skip "Python 3.12 ja instalado"
    else
        log_step "Instalando Python 3.12..."
        sudo add-apt-repository -y ppa:deadsnakes/ppa 2>/dev/null || true
        sudo apt update
        sudo apt install -y \
            python3.12 \
            python3.12-venv \
            python3.12-dev \
            python3.12-distutils

        # Configurar como padrao
        sudo update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.12 312 2>/dev/null || true

        log_info "Python 3.12 instalado"
    fi

    # pip
    log_step "Atualizando pip..."
    python3 -m ensurepip --upgrade 2>/dev/null || true
    python3 -m pip install --upgrade pip

    # pipx
    if command_exists pipx; then
        log_skip "pipx ja instalado"
    else
        log_step "Instalando pipx..."
        sudo apt install -y pipx
        pipx ensurepath
        log_info "pipx instalado"
    fi

    # poetry
    if command_exists poetry; then
        log_skip "poetry ja instalado"
    else
        log_step "Instalando poetry..."
        pipx install poetry
        log_info "poetry instalado"
    fi

    # pyenv
    if [[ -d "$HOME/.pyenv" ]]; then
        log_skip "pyenv ja instalado"
    else
        log_step "Instalando pyenv..."
        curl https://pyenv.run | bash

        # Adicionar ao bashrc
        if ! grep -q "PYENV_ROOT" "$HOME/.bashrc" 2>/dev/null; then
            cat >> "$HOME/.bashrc" << 'PYENV'

# Pyenv
export PYENV_ROOT="$HOME/.pyenv"
[[ -d $PYENV_ROOT/bin ]] && export PATH="$PYENV_ROOT/bin:$PATH"
eval "$(pyenv init -)"
eval "$(pyenv virtualenv-init -)"
PYENV
        fi

        log_info "pyenv instalado"
    fi

    # Ferramentas Python adicionais
    log_step "Instalando ferramentas Python..."
    pipx install black 2>/dev/null || true
    pipx install flake8 2>/dev/null || true
    pipx install mypy 2>/dev/null || true
    pipx install pylint 2>/dev/null || true
    pipx install isort 2>/dev/null || true
    pipx install httpie 2>/dev/null || true
    pipx install ipython 2>/dev/null || true
    pipx install ruff 2>/dev/null || true
    pipx install pdm 2>/dev/null || true
    pipx install hatch 2>/dev/null || true
    pipx install uv 2>/dev/null || true

    echo ""
    log_info "Python: $(python3 --version)"
}

# =============================================================================
# 8. JAVA / JVM
# =============================================================================
setup_java() {
    log_section "8. Configurando Java / JVM"

    # OpenJDK 21
    if java --version 2>/dev/null | grep -q "21"; then
        log_skip "OpenJDK 21 ja instalado"
    else
        log_step "Instalando OpenJDK 21..."
        sudo apt install -y openjdk-21-jdk openjdk-21-source openjdk-21-doc
        log_info "OpenJDK 21 instalado"
    fi

    # Configurar JAVA_HOME
    JAVA_HOME_PATH="/usr/lib/jvm/java-21-openjdk-amd64"
    if [[ -d "$JAVA_HOME_PATH" ]]; then
        if ! grep -q "JAVA_HOME" "$HOME/.bashrc" 2>/dev/null; then
            cat >> "$HOME/.bashrc" << JAVA

# Java
export JAVA_HOME="$JAVA_HOME_PATH"
export PATH="\$JAVA_HOME/bin:\$PATH"
JAVA
        fi
    fi

    # Maven
    if command_exists mvn; then
        log_skip "Maven ja instalado"
    else
        log_step "Instalando Maven..."
        sudo apt install -y maven
        log_info "Maven instalado"
    fi

    # Gradle
    if command_exists gradle; then
        log_skip "Gradle ja instalado"
    else
        log_step "Instalando Gradle..."

        GRADLE_VERSION="8.6"
        wget -q "https://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip" -O /tmp/gradle.zip
        sudo unzip -qo /tmp/gradle.zip -d /opt/
        sudo ln -sf "/opt/gradle-${GRADLE_VERSION}" /opt/gradle

        if ! grep -q "GRADLE_HOME" "$HOME/.bashrc" 2>/dev/null; then
            echo 'export GRADLE_HOME=/opt/gradle' >> "$HOME/.bashrc"
            echo 'export PATH=$GRADLE_HOME/bin:$PATH' >> "$HOME/.bashrc"
        fi

        rm /tmp/gradle.zip
        log_info "Gradle ${GRADLE_VERSION} instalado"
    fi

    # SDKMAN (gerenciador de SDKs Java)
    if [[ -d "$HOME/.sdkman" ]]; then
        log_skip "SDKMAN ja instalado"
    else
        log_step "Instalando SDKMAN..."
        curl -s "https://get.sdkman.io" | bash
        log_info "SDKMAN instalado"
    fi

    echo ""
    log_info "Java: $(java --version 2>/dev/null | head -1)"
    log_info "Maven: $(mvn --version 2>/dev/null | head -1)"
}

# =============================================================================
# 9. GO E ZIG
# =============================================================================
setup_go_zig() {
    log_section "9. Configurando Go e Zig"

    # Go 1.22+
    GO_VERSION="1.22.0"
    if command_exists go && go version | grep -qE "go1\.2[2-9]|go1\.[3-9]"; then
        log_skip "Go 1.22+ ja instalado"
    else
        log_step "Instalando Go ${GO_VERSION}..."

        wget -q "https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz" -O /tmp/go.tar.gz
        sudo rm -rf /usr/local/go
        sudo tar -C /usr/local -xzf /tmp/go.tar.gz
        rm /tmp/go.tar.gz

        # Configurar PATH
        if ! grep -q "/usr/local/go/bin" "$HOME/.bashrc" 2>/dev/null; then
            cat >> "$HOME/.bashrc" << 'GO'

# Go
export GOPATH="$HOME/go"
export PATH="$PATH:/usr/local/go/bin:$GOPATH/bin"
GO
        fi

        export PATH="$PATH:/usr/local/go/bin:$HOME/go/bin"
        log_info "Go ${GO_VERSION} instalado"
    fi

    # Ferramentas Go
    log_step "Instalando ferramentas Go..."
    export PATH="$PATH:/usr/local/go/bin:$HOME/go/bin"

    go install golang.org/x/tools/gopls@latest 2>/dev/null || true
    go install github.com/go-delve/delve/cmd/dlv@latest 2>/dev/null || true
    go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest 2>/dev/null || true
    go install github.com/cosmtrek/air@latest 2>/dev/null || true
    go install github.com/swaggo/swag/cmd/swag@latest 2>/dev/null || true

    # Zig 0.12+
    ZIG_VERSION="0.12.0"
    if command_exists zig && zig version | grep -qE "0\.1[2-9]|0\.[2-9]|1\."; then
        log_skip "Zig 0.12+ ja instalado"
    else
        log_step "Instalando Zig ${ZIG_VERSION}..."

        wget -q "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-x86_64-${ZIG_VERSION}.tar.xz" -O /tmp/zig.tar.xz
        sudo rm -rf /opt/zig
        sudo mkdir -p /opt/zig
        sudo tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1
        rm /tmp/zig.tar.xz

        # Criar link simbolico
        sudo ln -sf /opt/zig/zig /usr/local/bin/zig

        log_info "Zig ${ZIG_VERSION} instalado"
    fi

    # zls (Zig Language Server)
    if command_exists zls; then
        log_skip "zls ja instalado"
    else
        log_step "Instalando Zig Language Server..."

        ZLS_VERSION="0.12.0"
        wget -q "https://github.com/zigtools/zls/releases/download/${ZLS_VERSION}/zls-x86_64-linux.tar.xz" -O /tmp/zls.tar.xz 2>/dev/null || {
            log_warn "zls nao disponivel para download. Compile manualmente."
        }

        if [[ -f /tmp/zls.tar.xz ]]; then
            sudo tar -xJf /tmp/zls.tar.xz -C /usr/local/bin
            rm /tmp/zls.tar.xz
            log_info "zls instalado"
        fi
    fi

    echo ""
    log_info "Go: $(go version 2>/dev/null || echo 'instalado')"
    log_info "Zig: $(zig version 2>/dev/null || echo 'instalado')"
}

# =============================================================================
# 10. CONTAINERS
# =============================================================================
setup_containers() {
    log_section "10. Configurando Containers"

    # Docker
    if command_exists docker; then
        log_skip "Docker ja instalado"
    else
        log_step "Instalando Docker..."

        # Remover versoes antigas
        sudo apt remove -y docker docker-engine docker.io containerd runc 2>/dev/null || true

        # Instalar Docker
        curl -fsSL https://get.docker.com | sudo sh

        # Adicionar usuario ao grupo docker
        sudo usermod -aG docker "$USER"

        log_info "Docker instalado"
    fi

    # Docker Compose
    if docker compose version &>/dev/null; then
        log_skip "Docker Compose ja instalado"
    else
        log_step "Instalando Docker Compose..."
        sudo apt install -y docker-compose-plugin
        log_info "Docker Compose instalado"
    fi

    # Docker Buildx
    log_step "Configurando Docker Buildx..."
    docker buildx create --name multiarch --driver docker-container --use 2>/dev/null || true

    # Podman
    if command_exists podman; then
        log_skip "Podman ja instalado"
    else
        log_step "Instalando Podman..."
        sudo apt install -y podman podman-compose
        log_info "Podman instalado"
    fi

    # kubectl
    if command_exists kubectl; then
        log_skip "kubectl ja instalado"
    else
        log_step "Instalando kubectl..."

        curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.29/deb/Release.key | sudo gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg
        echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.29/deb/ /' | sudo tee /etc/apt/sources.list.d/kubernetes.list

        sudo apt update
        sudo apt install -y kubectl

        log_info "kubectl instalado"
    fi

    # Ferramentas adicionais de Kubernetes
    log_step "Instalando ferramentas Kubernetes..."

    # Helm
    if ! command_exists helm; then
        curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
    fi

    # k9s
    if ! command_exists k9s; then
        go install github.com/derailed/k9s@latest 2>/dev/null || true
    fi

    # kind (Kubernetes in Docker)
    if ! command_exists kind; then
        go install sigs.k8s.io/kind@latest 2>/dev/null || true
    fi

    # minikube
    if ! command_exists minikube; then
        curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
        sudo install minikube-linux-amd64 /usr/local/bin/minikube
        rm minikube-linux-amd64
    fi

    # Lazydocker (TUI para Docker)
    if ! command_exists lazydocker; then
        go install github.com/jesseduffield/lazydocker@latest 2>/dev/null || true
    fi

    echo ""
    log_info "Docker: $(docker --version 2>/dev/null || echo 'instalado')"
    log_info "Podman: $(podman --version 2>/dev/null || echo 'instalado')"
    log_info "kubectl: $(kubectl version --client --short 2>/dev/null || echo 'instalado')"
}

# =============================================================================
# Configurar shell
# =============================================================================
setup_shell_config() {
    log_section "Configurando Shell"

    # Criar configuracao unificada
    cat >> "$HOME/.bashrc" << 'BASHCONFIG'

# =============================================================================
# Winux OS Developer Environment Configuration
# =============================================================================

# Cores
export CLICOLOR=1
export LSCOLORS=GxFxCxDxBxegedabagaced

# Editor padrao
export EDITOR=nano
export VISUAL=code

# Historico
export HISTSIZE=10000
export HISTFILESIZE=20000
export HISTCONTROL=ignoreboth:erasedups

# PATH personalizado
export PATH="$HOME/.local/bin:$PATH"

# Aliases uteis
alias ll='ls -alF'
alias la='ls -A'
alias l='ls -CF'
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias g='git'
alias dc='docker compose'
alias k='kubectl'
alias tf='terraform'
alias py='python3'
alias serve='python3 -m http.server'

# Git aliases
alias gs='git status'
alias ga='git add'
alias gc='git commit'
alias gp='git push'
alias gl='git log --oneline --graph'
alias gd='git diff'
alias gb='git branch'
alias gco='git checkout'

# Docker aliases
alias dps='docker ps'
alias dpsa='docker ps -a'
alias di='docker images'
alias drm='docker rm'
alias drmi='docker rmi'
alias dprune='docker system prune -af'

# Funcoes uteis
mkcd() { mkdir -p "$1" && cd "$1"; }
extract() {
    if [ -f "$1" ]; then
        case "$1" in
            *.tar.bz2)   tar xjf "$1"     ;;
            *.tar.gz)    tar xzf "$1"     ;;
            *.tar.xz)    tar xJf "$1"     ;;
            *.bz2)       bunzip2 "$1"     ;;
            *.rar)       unrar x "$1"     ;;
            *.gz)        gunzip "$1"      ;;
            *.tar)       tar xf "$1"      ;;
            *.tbz2)      tar xjf "$1"     ;;
            *.tgz)       tar xzf "$1"     ;;
            *.zip)       unzip "$1"       ;;
            *.Z)         uncompress "$1"  ;;
            *.7z)        7z x "$1"        ;;
            *)           echo "'$1' cannot be extracted via extract()" ;;
        esac
    else
        echo "'$1' is not a valid file"
    fi
}

BASHCONFIG

    log_info "Shell configurado"
}

# =============================================================================
# Menu de selecao
# =============================================================================
show_menu() {
    echo -e "${CYAN}"
    cat << 'MENU'

    Selecione o que deseja instalar:

    [1]  .NET Ecosystem
    [2]  C/C++ Ecosystem
    [3]  Rust Toolchain
    [4]  Apple Build Tools
    [5]  Cross-Platform Tools
    [6]  Node.js / Web
    [7]  Python
    [8]  Java / JVM
    [9]  Go & Zig
    [10] Containers (Docker, Podman, K8s)

    [A]  INSTALAR TUDO (recomendado)
    [Q]  Sair

MENU
    echo -e "${NC}"
}

# =============================================================================
# Sumario final
# =============================================================================
show_summary() {
    echo -e "${CYAN}"
    cat << 'EOF'

    ╔══════════════════════════════════════════════════════════════╗
    ║                   INSTALACAO CONCLUIDA!                      ║
    ╚══════════════════════════════════════════════════════════════╝

EOF
    echo -e "${NC}"

    echo -e "${GREEN}Ferramentas instaladas:${NC}"
    echo ""

    echo -e "${BOLD}.NET:${NC}"
    echo "  - .NET SDK: $(dotnet --version 2>/dev/null || echo 'N/A')"
    echo "  - dotnet-ef, OmniSharp"
    echo ""

    echo -e "${BOLD}C/C++:${NC}"
    echo "  - GCC: $(gcc --version 2>/dev/null | head -1 || echo 'N/A')"
    echo "  - Clang: $(clang --version 2>/dev/null | head -1 || echo 'N/A')"
    echo "  - CMake, Ninja, ccache, GDB, LLDB, Valgrind"
    echo ""

    echo -e "${BOLD}Rust:${NC}"
    echo "  - Rust: $(rustc --version 2>/dev/null || echo 'N/A')"
    echo "  - Cargo: $(cargo --version 2>/dev/null || echo 'N/A')"
    echo "  - rust-analyzer, cargo-watch, cargo-edit, cargo-audit"
    echo ""

    echo -e "${BOLD}Apple Tools:${NC}"
    echo "  - create-dmg, appimagetool, bomutils, xar, libimobiledevice"
    echo ""

    echo -e "${BOLD}Cross-Platform:${NC}"
    echo "  - mingw-w64, flatpak-builder, snapcraft"
    echo ""

    echo -e "${BOLD}Node.js:${NC}"
    echo "  - Node.js: $(node --version 2>/dev/null || echo 'N/A')"
    echo "  - npm: $(npm --version 2>/dev/null || echo 'N/A')"
    echo "  - pnpm, yarn, Electron, Bun, Deno"
    echo ""

    echo -e "${BOLD}Python:${NC}"
    echo "  - Python: $(python3 --version 2>/dev/null || echo 'N/A')"
    echo "  - pip, pipx, poetry, pyenv"
    echo ""

    echo -e "${BOLD}Java:${NC}"
    echo "  - Java: $(java --version 2>/dev/null | head -1 || echo 'N/A')"
    echo "  - Maven: $(mvn --version 2>/dev/null | head -1 || echo 'N/A')"
    echo "  - Gradle, SDKMAN"
    echo ""

    echo -e "${BOLD}Go & Zig:${NC}"
    echo "  - Go: $(go version 2>/dev/null || echo 'N/A')"
    echo "  - Zig: $(zig version 2>/dev/null || echo 'N/A')"
    echo ""

    echo -e "${BOLD}Containers:${NC}"
    echo "  - Docker: $(docker --version 2>/dev/null || echo 'N/A')"
    echo "  - Podman: $(podman --version 2>/dev/null || echo 'N/A')"
    echo "  - kubectl, Helm, k9s, kind, minikube"
    echo ""

    echo -e "${YELLOW}==============================================================================${NC}"
    echo -e "${YELLOW}  IMPORTANTE: Execute os seguintes comandos para aplicar as configuracoes:${NC}"
    echo -e "${YELLOW}==============================================================================${NC}"
    echo ""
    echo -e "  ${WHITE}source ~/.bashrc${NC}          # Recarregar configuracoes do shell"
    echo -e "  ${WHITE}newgrp docker${NC}             # Ativar grupo Docker (ou logout/login)"
    echo ""
    echo -e "${CYAN}Winux OS - The Future of Desktop Linux${NC}"
    echo ""
}

# =============================================================================
# Main
# =============================================================================
main() {
    show_logo

    # Verificar argumentos
    if [[ "$1" == "--all" ]] || [[ "$1" == "-a" ]]; then
        INSTALL_ALL=true
    elif [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
        echo "Uso: $0 [opcao]"
        echo ""
        echo "Opcoes:"
        echo "  --all, -a    Instalar todos os componentes"
        echo "  --help, -h   Mostrar esta ajuda"
        echo "  (sem args)   Menu interativo"
        exit 0
    else
        INSTALL_ALL=false
    fi

    check_requirements
    setup_base

    if [[ "$INSTALL_ALL" == true ]]; then
        setup_dotnet
        setup_cpp
        setup_rust
        setup_apple_tools
        setup_crossplatform
        setup_nodejs
        setup_python
        setup_java
        setup_go_zig
        setup_containers
        setup_shell_config
        show_summary
    else
        while true; do
            show_menu
            read -p "Escolha uma opcao: " choice

            case $choice in
                1)  setup_dotnet ;;
                2)  setup_cpp ;;
                3)  setup_rust ;;
                4)  setup_apple_tools ;;
                5)  setup_crossplatform ;;
                6)  setup_nodejs ;;
                7)  setup_python ;;
                8)  setup_java ;;
                9)  setup_go_zig ;;
                10) setup_containers ;;
                [Aa])
                    setup_dotnet
                    setup_cpp
                    setup_rust
                    setup_apple_tools
                    setup_crossplatform
                    setup_nodejs
                    setup_python
                    setup_java
                    setup_go_zig
                    setup_containers
                    setup_shell_config
                    show_summary
                    break
                    ;;
                [Qq])
                    log_info "Saindo..."
                    exit 0
                    ;;
                *)
                    log_error "Opcao invalida"
                    ;;
            esac

            echo ""
            read -p "Pressione Enter para continuar..."
        done
    fi
}

# Executar
main "$@"
