#!/bin/bash
#===============================================================================
# Winux OS - Complete Developer Environment Setup
# Installs all development tools and languages
#===============================================================================

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

echo "========================================"
echo "   Winux OS Developer Setup"
echo "========================================"
echo ""

#-------------------------------------------------------------------------------
# Install NVM and Node.js
#-------------------------------------------------------------------------------
setup_nodejs() {
    log_info "Setting up Node.js with NVM..."
    
    export NVM_DIR="$HOME/.nvm"
    
    if [ ! -d "$NVM_DIR" ]; then
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
        
        [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
        
        nvm install --lts
        nvm use --lts
        nvm alias default node
        
        log_success "Node.js $(node --version) installed with NVM"
    else
        log_warn "NVM already installed"
        [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    fi
    
    # Install global npm packages
    npm install -g pnpm yarn typescript ts-node nodemon eslint prettier
    log_success "NPM global packages installed"
}

#-------------------------------------------------------------------------------
# Setup PHP with Composer
#-------------------------------------------------------------------------------
setup_php() {
    log_info "Setting up PHP with Composer..."
    
    if ! command -v composer &> /dev/null; then
        php -r "copy('https://getcomposer.org/installer', 'composer-setup.php');"
        php composer-setup.php --install-dir=/usr/local/bin --filename=composer 2>/dev/null || \
        sudo php composer-setup.php --install-dir=/usr/local/bin --filename=composer
        php -r "unlink('composer-setup.php');"
        log_success "Composer installed"
    else
        log_warn "Composer already installed"
    fi
    
    # Configure Apache for PHP
    if command -v a2enmod &> /dev/null; then
        sudo a2enmod php8.* rewrite ssl headers 2>/dev/null || true
        log_success "Apache PHP modules enabled"
    fi
}

#-------------------------------------------------------------------------------
# Setup Java (Spring/Maven)
#-------------------------------------------------------------------------------
setup_java() {
    log_info "Setting up Java environment..."
    
    # Set JAVA_HOME
    if [ -d "/usr/lib/jvm/java-21-openjdk-amd64" ]; then
        export JAVA_HOME="/usr/lib/jvm/java-21-openjdk-amd64"
    elif [ -d "/usr/lib/jvm/java-17-openjdk-amd64" ]; then
        export JAVA_HOME="/usr/lib/jvm/java-17-openjdk-amd64"
    fi
    
    # Add to profile
    if ! grep -q "JAVA_HOME" "$HOME/.bashrc" 2>/dev/null; then
        echo "export JAVA_HOME=$JAVA_HOME" >> "$HOME/.bashrc"
        echo 'export PATH=$JAVA_HOME/bin:$PATH' >> "$HOME/.bashrc"
    fi
    
    # Install Spring Boot CLI (optional)
    if ! command -v spring &> /dev/null; then
        log_info "Spring Boot CLI can be installed with: sdk install springboot"
    fi
    
    log_success "Java environment configured"
}

#-------------------------------------------------------------------------------
# Setup Python with pip
#-------------------------------------------------------------------------------
setup_python() {
    log_info "Setting up Python environment..."
    
    # Upgrade pip
    python3 -m pip install --upgrade pip 2>/dev/null || \
    python3 -m pip install --upgrade pip --user
    
    # Install common Python packages
    python3 -m pip install --user pipenv virtualenv poetry black flake8 mypy pylint
    
    log_success "Python environment configured"
}

#-------------------------------------------------------------------------------
# Setup Rust with Cargo, Tauri, and Windows build support
#-------------------------------------------------------------------------------
setup_rust() {
    log_info "Setting up Rust environment..."
    
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        log_warn "Rust already installed"
        source "$HOME/.cargo/env" 2>/dev/null || true
    fi
    
    # Install nightly toolchain
    rustup toolchain install nightly
    rustup default stable
    
    # Install Windows cross-compilation target
    rustup target add x86_64-pc-windows-gnu
    rustup target add x86_64-pc-windows-msvc
    
    # Install Tauri CLI
    cargo install tauri-cli
    cargo install create-tauri-app
    
    # Install other useful Rust tools
    cargo install cargo-watch cargo-edit cargo-expand cargo-audit
    
    log_success "Rust environment configured with Tauri and Windows targets"
}

#-------------------------------------------------------------------------------
# Configure IDE permissions
#-------------------------------------------------------------------------------
setup_ide_permissions() {
    log_info "Configuring IDE permissions..."
    
    # VS Code
    if command -v code &> /dev/null; then
        # Increase file watchers for VS Code
        if ! grep -q "fs.inotify.max_user_watches" /etc/sysctl.conf 2>/dev/null; then
            echo "fs.inotify.max_user_watches=524288" | sudo tee -a /etc/sysctl.conf
            sudo sysctl -p 2>/dev/null || true
        fi
        log_success "VS Code permissions configured"
    fi
    
    # IntelliJ IDEA
    if [ -d "/snap/intellij-idea-community" ] || command -v idea &> /dev/null; then
        # Create idea.properties if needed
        mkdir -p "$HOME/.config/JetBrains"
        log_success "IntelliJ IDEA permissions configured"
    fi
}

#-------------------------------------------------------------------------------
# Setup Docker permissions
#-------------------------------------------------------------------------------
setup_docker() {
    log_info "Setting up Docker permissions..."
    
    if command -v docker &> /dev/null; then
        # Add user to docker group
        sudo usermod -aG docker "$USER" 2>/dev/null || true
        log_success "Docker permissions configured (logout/login required)"
    else
        log_warn "Docker not installed"
    fi
}

#-------------------------------------------------------------------------------
# Configure Git
#-------------------------------------------------------------------------------
setup_git() {
    log_info "Configuring Git..."
    
    # Set default branch name
    git config --global init.defaultBranch main
    
    # Set helpful aliases
    git config --global alias.co checkout
    git config --global alias.br branch
    git config --global alias.ci commit
    git config --global alias.st status
    git config --global alias.lg "log --oneline --graph --all"
    
    # Better diff
    git config --global core.pager "delta --dark"
    
    log_success "Git configured with helpful aliases"
}

#-------------------------------------------------------------------------------
# Write shell configuration
#-------------------------------------------------------------------------------
write_shell_config() {
    log_info "Writing shell configuration..."
    
    cat >> "$HOME/.bashrc" << 'BASHEOF'

# Winux Developer Environment
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

# NVM
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"

# Rust
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

# Java
[ -d "/usr/lib/jvm/java-21-openjdk-amd64" ] && export JAVA_HOME="/usr/lib/jvm/java-21-openjdk-amd64"

# Helpful aliases
alias ll='ls -la'
alias la='ls -A'
alias ..='cd ..'
alias ...='cd ../..'
alias g='git'
alias dc='docker-compose'

# Development shortcuts
alias serve='python3 -m http.server'
alias jsonformat='python3 -m json.tool'

BASHEOF

    log_success "Shell configuration updated"
}

#-------------------------------------------------------------------------------
# Main
#-------------------------------------------------------------------------------
main() {
    setup_nodejs
    setup_php
    setup_java
    setup_python
    setup_rust
    setup_ide_permissions
    setup_docker
    setup_git
    write_shell_config
    
    echo ""
    echo "========================================"
    log_success "Developer environment setup complete!"
    echo "========================================"
    echo ""
    echo "Installed:"
    echo "  - Node.js with NVM, pnpm, yarn"
    echo "  - PHP with Composer and Apache"
    echo "  - Java with Maven/Gradle"
    echo "  - Python 3 with pip, pipenv, poetry"
    echo "  - Rust with Cargo, Tauri, nightly, Windows targets"
    echo ""
    echo "Please logout and login again to apply all changes."
}

main "$@"
