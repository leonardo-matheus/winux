#!/bin/bash
# =============================================================================
# Winux OS - Developer Environment Setup
# =============================================================================
# Configura ambiente completo de desenvolvimento
# =============================================================================

set -e

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

log_info() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }
log_step() { echo -e "${BLUE}[→]${NC} ${BOLD}$1${NC}"; }

echo -e "${CYAN}"
cat << 'EOF'
╔══════════════════════════════════════════════════════════════╗
║     WINUX OS - Developer Environment Setup                  ║
║     Configurando ambiente profissional de desenvolvimento   ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Verificar se é root
if [[ $EUID -eq 0 ]]; then
    log_error "Não execute como root. Use seu usuário normal."
    exit 1
fi

# =============================================================================
# Atualizar sistema
# =============================================================================
log_step "Atualizando sistema..."
sudo apt update && sudo apt upgrade -y

# =============================================================================
# Ferramentas essenciais de build
# =============================================================================
log_step "Instalando ferramentas de build..."
sudo apt install -y \
    build-essential \
    cmake \
    ninja-build \
    pkg-config \
    autoconf \
    automake \
    libtool \
    make \
    gcc \
    g++ \
    clang \
    llvm \
    gdb \
    valgrind

# =============================================================================
# Git e controle de versão
# =============================================================================
log_step "Configurando Git..."
sudo apt install -y git git-lfs git-flow gitk

# Configurações úteis do Git
git config --global init.defaultBranch main
git config --global pull.rebase false
git config --global core.autocrlf input
git config --global color.ui auto

log_info "Git configurado"

# =============================================================================
# Rust + Cargo
# =============================================================================
log_step "Instalando Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Componentes úteis
rustup component add rustfmt clippy rust-analyzer
rustup target add wasm32-unknown-unknown

# Ferramentas Cargo úteis
cargo install cargo-watch cargo-edit cargo-audit cargo-outdated sccache

log_info "Rust $(rustc --version) instalado"

# =============================================================================
# Node.js + NVM
# =============================================================================
log_step "Instalando NVM e Node.js..."
if [[ ! -d "$HOME/.nvm" ]]; then
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
fi

export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# Instalar Node LTS
nvm install --lts
nvm use --lts
nvm alias default node

# Pacotes globais úteis
npm install -g yarn pnpm typescript ts-node nodemon pm2 eslint prettier

log_info "Node.js $(node --version) instalado"

# =============================================================================
# Python + pip
# =============================================================================
log_step "Instalando Python..."
sudo apt install -y \
    python3 \
    python3-pip \
    python3-venv \
    python3-dev \
    python3-setuptools \
    python3-wheel \
    pipx

# Ferramentas Python
pipx install poetry
pipx install black
pipx install flake8
pipx install mypy
pipx install httpie
pipx install ipython

log_info "Python $(python3 --version) instalado"

# =============================================================================
# PHP + Composer
# =============================================================================
log_step "Instalando PHP..."
sudo apt install -y \
    php \
    php-cli \
    php-fpm \
    php-mysql \
    php-pgsql \
    php-sqlite3 \
    php-curl \
    php-gd \
    php-mbstring \
    php-xml \
    php-zip \
    php-bcmath \
    php-json \
    php-redis

# Composer
if ! command -v composer &> /dev/null; then
    curl -sS https://getcomposer.org/installer | php
    sudo mv composer.phar /usr/local/bin/composer
fi

log_info "PHP $(php --version | head -1) instalado"

# =============================================================================
# Go
# =============================================================================
log_step "Instalando Go..."
GO_VERSION="1.22.0"
if ! command -v go &> /dev/null; then
    wget -q "https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz" -O /tmp/go.tar.gz
    sudo rm -rf /usr/local/go
    sudo tar -C /usr/local -xzf /tmp/go.tar.gz
    rm /tmp/go.tar.gz
fi

echo 'export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin' >> ~/.bashrc
export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin

log_info "Go $(go version) instalado"

# =============================================================================
# Java (OpenJDK)
# =============================================================================
log_step "Instalando Java..."
sudo apt install -y \
    openjdk-21-jdk \
    openjdk-21-source \
    maven \
    gradle

log_info "Java $(java --version | head -1) instalado"

# =============================================================================
# Servidores Web
# =============================================================================
log_step "Instalando servidores web..."
sudo apt install -y \
    apache2 \
    nginx \
    libapache2-mod-php

# Desabilitar por padrão (iniciar manualmente quando necessário)
sudo systemctl disable apache2 nginx
sudo systemctl stop apache2 nginx

log_info "Apache e Nginx instalados (desabilitados por padrão)"

# =============================================================================
# Databases
# =============================================================================
log_step "Instalando databases..."
sudo apt install -y \
    postgresql \
    postgresql-contrib \
    mysql-server \
    redis-server \
    sqlite3

# Desabilitar por padrão
sudo systemctl disable postgresql mysql redis-server
sudo systemctl stop postgresql mysql redis-server

log_info "PostgreSQL, MySQL, Redis e SQLite instalados"

# =============================================================================
# Docker
# =============================================================================
log_step "Instalando Docker..."
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com | sudo sh
    sudo usermod -aG docker $USER
fi

# Docker Compose
sudo apt install -y docker-compose-plugin

log_info "Docker instalado (requer logout/login para grupo docker)"

# =============================================================================
# Terminal Tools
# =============================================================================
log_step "Instalando ferramentas de terminal..."
sudo apt install -y \
    zsh \
    tmux \
    fzf \
    ripgrep \
    fd-find \
    bat \
    exa \
    jq \
    yq \
    tree \
    ncdu \
    htop \
    btop \
    tldr \
    neofetch \
    figlet \
    lolcat

# Oh My Zsh
if [[ ! -d "$HOME/.oh-my-zsh" ]]; then
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended
fi

# Plugins do Zsh
git clone https://github.com/zsh-users/zsh-autosuggestions ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-autosuggestions 2>/dev/null || true
git clone https://github.com/zsh-users/zsh-syntax-highlighting ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-syntax-highlighting 2>/dev/null || true

# Starship prompt
curl -sS https://starship.rs/install.sh | sh -s -- -y

log_info "Ferramentas de terminal instaladas"

# =============================================================================
# IDEs e Editores
# =============================================================================
log_step "Instalando IDEs..."

# VS Code
if ! command -v code &> /dev/null; then
    wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > /tmp/packages.microsoft.gpg
    sudo install -D -o root -g root -m 644 /tmp/packages.microsoft.gpg /etc/apt/keyrings/packages.microsoft.gpg
    echo "deb [arch=amd64,arm64,armhf signed-by=/etc/apt/keyrings/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" | sudo tee /etc/apt/sources.list.d/vscode.list
    sudo apt update
    sudo apt install -y code
fi

# Extensões úteis do VS Code
code --install-extension ms-python.python || true
code --install-extension rust-lang.rust-analyzer || true
code --install-extension ms-vscode.cpptools || true
code --install-extension golang.go || true
code --install-extension redhat.java || true
code --install-extension esbenp.prettier-vscode || true
code --install-extension dbaeumer.vscode-eslint || true
code --install-extension ms-azuretools.vscode-docker || true
code --install-extension eamodio.gitlens || true
code --install-extension pkief.material-icon-theme || true
code --install-extension dracula-theme.theme-dracula || true

log_info "VS Code instalado com extensões"

# IntelliJ IDEA Community
log_step "Instalando IntelliJ IDEA..."
if ! command -v idea &> /dev/null; then
    sudo snap install intellij-idea-community --classic
fi
log_info "IntelliJ IDEA Community instalado"

# =============================================================================
# Ferramentas de API
# =============================================================================
log_step "Instalando ferramentas de API..."

# Postman
sudo snap install postman

# Insomnia (alternativa leve)
sudo snap install insomnia

log_info "Postman e Insomnia instalados"

# =============================================================================
# Ferramentas de rede e debug
# =============================================================================
log_step "Instalando ferramentas de rede..."
sudo apt install -y \
    curl \
    wget \
    net-tools \
    dnsutils \
    nmap \
    tcpdump \
    wireshark \
    mtr-tiny \
    iperf3

log_info "Ferramentas de rede instaladas"

# =============================================================================
# Configurar ZSH como shell padrão
# =============================================================================
log_step "Configurando ZSH..."

cat > ~/.zshrc << 'ZSHRC'
# Winux OS Developer ZSH Configuration

# Oh My Zsh
export ZSH="$HOME/.oh-my-zsh"
ZSH_THEME="robbyrussell"
plugins=(git docker docker-compose node npm rust cargo python pip golang kubectl helm fzf zsh-autosuggestions zsh-syntax-highlighting)
source $ZSH/oh-my-zsh.sh

# NVM
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"

# Rust
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

# Go
export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin

# Aliases úteis
alias ll='exa -la --icons'
alias ls='exa --icons'
alias cat='bat --style=plain'
alias grep='rg'
alias find='fd'
alias top='btop'
alias vim='nvim'
alias g='git'
alias dc='docker compose'
alias k='kubectl'

# Starship prompt
eval "$(starship init zsh)"

# FZF
[ -f ~/.fzf.zsh ] && source ~/.fzf.zsh

# Winux banner
echo ""
neofetch --ascii_distro ubuntu_small
echo ""
ZSHRC

# Mudar shell padrão para zsh
chsh -s $(which zsh) || true

log_info "ZSH configurado como shell padrão"

# =============================================================================
# Configurar Starship
# =============================================================================
mkdir -p ~/.config
cat > ~/.config/starship.toml << 'STARSHIP'
# Winux Developer Prompt

format = """
[╭─](bold blue) $directory$git_branch$git_status$rust$nodejs$python$golang$java$php$docker_context
[╰─](bold blue) $character"""

[character]
success_symbol = "[❯](bold green)"
error_symbol = "[❯](bold red)"

[directory]
style = "bold cyan"
truncation_length = 3
truncate_to_repo = true

[git_branch]
symbol = " "
style = "bold purple"

[git_status]
style = "bold red"

[rust]
symbol = " "
style = "bold orange"

[nodejs]
symbol = " "
style = "bold green"

[python]
symbol = " "
style = "bold yellow"

[golang]
symbol = " "
style = "bold cyan"

[java]
symbol = " "
style = "bold red"

[php]
symbol = " "
style = "bold blue"

[docker_context]
symbol = " "
style = "bold blue"
STARSHIP

log_info "Starship prompt configurado"

# =============================================================================
# Finalização
# =============================================================================
echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Ambiente de desenvolvimento configurado com sucesso!${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${CYAN}Linguagens instaladas:${NC}"
echo "  • Rust $(rustc --version 2>/dev/null || echo 'instalado')"
echo "  • Node.js $(node --version 2>/dev/null || echo 'instalado')"
echo "  • Python $(python3 --version 2>/dev/null || echo 'instalado')"
echo "  • PHP $(php --version 2>/dev/null | head -1 || echo 'instalado')"
echo "  • Go $(go version 2>/dev/null || echo 'instalado')"
echo "  • Java $(java --version 2>/dev/null | head -1 || echo 'instalado')"
echo ""
echo -e "${CYAN}IDEs instaladas:${NC}"
echo "  • VS Code"
echo "  • IntelliJ IDEA Community"
echo ""
echo -e "${CYAN}Ferramentas:${NC}"
echo "  • Docker, Postman, Insomnia"
echo "  • PostgreSQL, MySQL, Redis, SQLite"
echo "  • Apache, Nginx"
echo "  • Git, tmux, fzf, ripgrep, bat, exa"
echo ""
echo -e "${YELLOW}⚠ Reinicie o terminal ou execute: source ~/.zshrc${NC}"
echo -e "${YELLOW}⚠ Para Docker sem sudo, faça logout/login${NC}"
echo ""
