#!/bin/bash
# =============================================================================
# Winux OS - First Login Developer Setup
# =============================================================================
# Executado no primeiro login para configurar ferramentas de usuário
# =============================================================================

MARKER_FILE="$HOME/.winux-dev-setup-done"

# Se já foi executado, sair
if [[ -f "$MARKER_FILE" ]]; then
    exit 0
fi

# Cores
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${CYAN}"
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     Bem-vindo ao Winux OS Developer Edition!                ║"
echo "║     Configurando seu ambiente de desenvolvimento...         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# =============================================================================
# Rust
# =============================================================================
echo -e "${GREEN}[1/4]${NC} Instalando Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
    source "$HOME/.cargo/env"
    rustup component add rustfmt clippy rust-analyzer 2>/dev/null
fi

# =============================================================================
# NVM + Node.js
# =============================================================================
echo -e "${GREEN}[2/4]${NC} Instalando Node.js via NVM..."
if [[ ! -d "$HOME/.nvm" ]]; then
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash 2>/dev/null
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    nvm install --lts 2>/dev/null
    nvm alias default node 2>/dev/null
    npm install -g yarn pnpm typescript 2>/dev/null
fi

# =============================================================================
# Go
# =============================================================================
echo -e "${GREEN}[3/4]${NC} Configurando Go..."
if ! command -v go &> /dev/null; then
    GO_VERSION="1.22.0"
    wget -q "https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz" -O /tmp/go.tar.gz
    sudo tar -C /usr/local -xzf /tmp/go.tar.gz 2>/dev/null
    rm /tmp/go.tar.gz
fi

# =============================================================================
# Shell Configuration
# =============================================================================
echo -e "${GREEN}[4/4]${NC} Configurando shell..."

# Starship prompt
if ! command -v starship &> /dev/null; then
    curl -sS https://starship.rs/install.sh | sh -s -- -y 2>/dev/null
fi

# Configurar .bashrc
cat >> ~/.bashrc << 'BASHRC'

# Winux Developer Environment
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$PATH:/usr/local/go/bin:$HOME/go/bin"
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# Aliases
alias ll='ls -la --color=auto'
alias g='git'
alias dc='docker compose'

# Starship prompt
eval "$(starship init bash)"
BASHRC

# Configurar Starship
mkdir -p ~/.config
cat > ~/.config/starship.toml << 'STARSHIP'
format = """$directory$git_branch$git_status$rust$nodejs$python$golang$java$character"""

[character]
success_symbol = "[❯](bold green)"
error_symbol = "[❯](bold red)"

[directory]
style = "bold cyan"
truncation_length = 3

[git_branch]
symbol = " "
style = "bold purple"
STARSHIP

# Marcar como concluído
touch "$MARKER_FILE"

echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Ambiente de desenvolvimento configurado!${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Reinicie o terminal para aplicar as configurações.${NC}"
echo ""
