# Winux OS - Guia do Desenvolvedor

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║   ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗    ██████╗ ███████╗██╗   ██╗       ║
║   ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝    ██╔══██╗██╔════╝██║   ██║       ║
║   ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝     ██║  ██║█████╗  ██║   ██║       ║
║   ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗     ██║  ██║██╔══╝  ╚██╗ ██╔╝       ║
║   ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗    ██████╔╝███████╗ ╚████╔╝        ║
║    ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝    ╚═════╝ ╚══════╝  ╚═══╝         ║
║                                                                               ║
║                       DEVELOPER GUIDE                                         ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

Este guia fornece todas as informacoes necessarias para desenvolver aplicacoes no Winux OS, desde a configuracao do ambiente ate a distribuicao de software.

---

## Indice

1. [Setup do Ambiente](#setup-do-ambiente)
2. [Linguagens e Ferramentas](#linguagens-e-ferramentas)
3. [Desenvolvimento de Apps Winux](#desenvolvimento-de-apps-winux)
4. [Cross-Compilation](#cross-compilation)
5. [Build para Cada Plataforma](#build-para-cada-plataforma)
6. [IDEs e Editores](#ides-e-editores)
7. [Containers e Virtualizacao](#containers-e-virtualizacao)
8. [Debugging e Profiling](#debugging-e-profiling)

---

## Setup do Ambiente

### Configuracao Automatica

O Winux vem com scripts que configuram todo o ambiente de desenvolvimento automaticamente:

```bash
# Setup completo de desenvolvimento
sudo /etc/winux/scripts/setup-dev-full.sh

# Setup apenas de variaveis de ambiente
/etc/winux/scripts/setup-environment.sh

# Setup no primeiro login
/etc/winux/scripts/first-login-dev-setup.sh
```

### Usando o Winux Dev Hub

O **Winux Dev Hub** e a central de desenvolvimento que oferece:

- Dashboard de projetos com auto-deteccao
- Gerenciamento de variaveis de ambiente por perfil
- Status de toolchains instalados
- Orquestracao de containers (Docker/Podman)
- Gerenciamento de databases locais
- Controle de servicos do sistema

Para abrir:
```bash
winux-dev-hub
```

### Configuracao Manual de PATH

Se preferir configurar manualmente, adicione ao seu `~/.bashrc` ou `~/.zshrc`:

```bash
# Rust
export CARGO_HOME="$HOME/.cargo"
export RUSTUP_HOME="$HOME/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"

# Go
export GOPATH="$HOME/go"
export GOROOT="/usr/local/go"
export PATH="$GOPATH/bin:$GOROOT/bin:$PATH"

# Node.js (NVM)
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# Java
export JAVA_HOME="/usr/lib/jvm/java-21-openjdk-amd64"
export PATH="$JAVA_HOME/bin:$PATH"

# Android
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
export PATH="$ANDROID_HOME/platform-tools:$PATH"
export PATH="$ANDROID_HOME/emulator:$PATH"

# Flutter
export PATH="$HOME/flutter/bin:$PATH"

# .NET
export DOTNET_ROOT="/usr/share/dotnet"
export PATH="$DOTNET_ROOT:$PATH"
```

---

## Linguagens e Ferramentas

### Rust

O Winux e construido em Rust. Todas as ferramentas estao pre-instaladas:

```bash
# Versao
rustc --version    # 1.75+
cargo --version

# Criar novo projeto
cargo new meu-projeto
cd meu-projeto
cargo build --release

# Componentes uteis
rustup component add clippy     # Linter
rustup component add rustfmt    # Formatador
rustup component add rust-src   # Codigo fonte (para IDE)
rustup component add rust-analyzer  # LSP

# Targets para cross-compilation
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-linux-android
```

**Crates recomendadas para GUI:**
```toml
[dependencies]
gtk4 = "0.8"
libadwaita = "0.6"
relm4 = "0.8"
```

### .NET (C#, F#, VB.NET)

```bash
# Versao
dotnet --version    # 8.0+

# Criar projetos
dotnet new console -n MeuApp           # Console C#
dotnet new webapi -n MinhaAPI          # Web API
dotnet new blazorwasm -n MeuBlazor     # Blazor WebAssembly
dotnet new maui -n MeuMAUI             # MAUI (cross-platform)

# Build e run
dotnet build
dotnet run

# Publish para diferentes plataformas
dotnet publish -c Release -r linux-x64 --self-contained
dotnet publish -c Release -r win-x64 --self-contained
dotnet publish -c Release -r osx-x64 --self-contained
```

### C/C++

```bash
# Compiladores
gcc --version      # GCC 13+
clang --version    # Clang 17+
cmake --version    # CMake 3.28+

# Compilacao basica
gcc -o programa main.c
g++ -std=c++20 -o programa main.cpp

# Com CMake
mkdir build && cd build
cmake ..
make -j$(nproc)

# Cross-compile para Windows
x86_64-w64-mingw32-gcc -o programa.exe main.c
```

**Ferramentas instaladas:**
- GCC/G++ 13+
- Clang/LLVM 17+
- CMake 3.28+
- Make, Ninja
- GDB, LLDB
- Valgrind
- AddressSanitizer

### Java

```bash
# Versao
java --version     # OpenJDK 21
javac --version

# Maven
mvn --version
mvn archetype:generate
mvn clean install

# Gradle
gradle --version
gradle init
gradle build

# Spring Boot
spring init --dependencies=web my-project
```

### Python

```bash
# Versao
python3 --version  # 3.12+
pip3 --version

# Ambientes virtuais
python3 -m venv .venv
source .venv/bin/activate

# Poetry (recomendado)
poetry new meu-projeto
poetry add django
poetry install
poetry run python main.py

# Pipenv
pipenv install
pipenv shell
```

### Node.js

```bash
# NVM para gerenciar versoes
nvm install --lts
nvm use --lts
node --version     # 20+

# npm/yarn/pnpm
npm init
npm install
npm run build

yarn init
yarn add express

pnpm init
pnpm add react
```

### Go

```bash
# Versao
go version        # 1.22+

# Criar modulo
go mod init github.com/user/projeto

# Build
go build
go build -o myapp-linux-amd64

# Cross-compile
GOOS=windows GOARCH=amd64 go build -o myapp.exe
GOOS=darwin GOARCH=amd64 go build -o myapp-mac
```

### Swift (Linux)

```bash
# Versao
swift --version   # 5.9+

# Criar projeto
mkdir MeuProjeto && cd MeuProjeto
swift package init --type executable

# Build
swift build
swift build -c release

# Run
swift run
```

### PHP

```bash
# Versao
php --version     # 8.3+
composer --version

# Criar projeto Laravel
composer create-project laravel/laravel meu-projeto

# Servidor de desenvolvimento
php -S localhost:8000

# Composer
composer install
composer require package/name
```

---

## Desenvolvimento de Apps Winux

### Estrutura de um App Winux

```
meu-app/
├── Cargo.toml           # Configuracao do projeto
├── src/
│   ├── main.rs          # Ponto de entrada
│   ├── window.rs        # Janela principal
│   ├── pages/           # Paginas/views
│   │   ├── mod.rs
│   │   └── home.rs
│   └── widgets/         # Componentes customizados
│       ├── mod.rs
│       └── card.rs
├── resources/
│   ├── style.css        # Estilos CSS
│   └── icons/           # Icones do app
└── data/
    └── org.winux.MeuApp.desktop  # Desktop entry
```

### Cargo.toml Base

```toml
[package]
name = "winux-meu-app"
version = "1.0.0"
edition = "2021"
authors = ["Seu Nome <email@example.com>"]
license = "MIT"

[dependencies]
gtk4 = "0.8"
libadwaita = "0.6"
glib = "0.19"
gio = "0.19"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
strip = true
```

### Exemplo de App Basico

```rust
// main.rs
use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.MeuApp";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    use adw::prelude::*;
    use adw::{ApplicationWindow, HeaderBar};
    use gtk4::{Box, Label, Orientation};

    let header = HeaderBar::new();

    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);

    let title = Label::new(Some("Bem-vindo ao Winux!"));
    title.add_css_class("title-1");
    content.append(&title);

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&header);
    main_box.append(&content);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Meu App")
        .default_width(800)
        .default_height(600)
        .content(&main_box)
        .build();

    window.present();
}
```

### Desktop Entry

```ini
# data/org.winux.MeuApp.desktop
[Desktop Entry]
Name=Meu App
Comment=Descricao do meu app
Exec=winux-meu-app
Icon=org.winux.MeuApp
Terminal=false
Type=Application
Categories=Utility;GTK;
Keywords=exemplo;winux;
```

---

## Cross-Compilation

### Configuracao de Targets

```bash
# Instalar linkers necessarios
sudo apt install gcc-mingw-w64-x86-64  # Windows
sudo apt install gcc-aarch64-linux-gnu # ARM64

# Adicionar targets Rust
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

### .cargo/config.toml

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin-clang"
```

### Compilar para Windows

```bash
# CLI app
cargo build --target x86_64-pc-windows-gnu --release

# Com recursos Windows
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc --release
```

### Compilar para macOS (via osxcross)

```bash
# Instalar osxcross (requer SDK do macOS)
git clone https://github.com/tpoechtrager/osxcross
cd osxcross
./build.sh

# Compilar
export PATH="$PATH:$HOME/osxcross/target/bin"
cargo build --target x86_64-apple-darwin --release
```

---

## Build para Cada Plataforma

### Usando Winux Builder

O **Winux Builder** oferece uma interface grafica para builds cross-platform:

```bash
winux-builder
```

Features:
- Auto-deteccao de tipo de projeto
- Build para Windows (.exe, .msi)
- Build para Linux (.deb, .rpm, .AppImage, .flatpak)
- Build para macOS (.app, .dmg, .pkg)
- Perfis de build salvos
- Terminal integrado

### Linux - Debian Package (.deb)

```bash
# Instalar cargo-deb
cargo install cargo-deb

# Adicionar ao Cargo.toml
[package.metadata.deb]
maintainer = "Seu Nome <email@example.com>"
copyright = "2026, Winux OS Project"
license-file = ["LICENSE", "0"]
depends = "$auto"
extended-description = "Descricao completa do app"
section = "utility"
priority = "optional"
assets = [
    ["target/release/meu-app", "usr/bin/", "755"],
    ["data/*.desktop", "usr/share/applications/", "644"],
    ["resources/icons/*", "usr/share/icons/", "644"]
]

# Build
cargo deb --target x86_64-unknown-linux-gnu
```

### Linux - AppImage

```bash
# Instalar ferramentas
cargo install cargo-appimage

# Criar AppImage
cargo appimage --target x86_64-unknown-linux-gnu

# Ou manualmente
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage
./appimagetool-x86_64.AppImage MeuApp.AppDir
```

### Linux - Flatpak

```yaml
# org.winux.MeuApp.yml
app-id: org.winux.MeuApp
runtime: org.gnome.Platform
runtime-version: '45'
sdk: org.gnome.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
command: meu-app

finish-args:
  - --share=ipc
  - --socket=fallback-x11
  - --socket=wayland
  - --device=dri

modules:
  - name: meu-app
    buildsystem: simple
    build-commands:
      - cargo build --release
      - install -Dm755 target/release/meu-app /app/bin/meu-app
    sources:
      - type: dir
        path: .
```

```bash
# Build
flatpak-builder build-dir org.winux.MeuApp.yml --force-clean
flatpak build-export repo build-dir
flatpak build-bundle repo MeuApp.flatpak org.winux.MeuApp
```

### Windows - Executable (.exe)

```bash
# Cross-compile
cargo build --target x86_64-pc-windows-gnu --release

# Adicionar icone (opcional)
cargo install cargo-winres

# winres.rs
fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/icon.ico");
        res.compile().unwrap();
    }
}
```

### Windows - Installer (.msi)

```bash
# Usar WiX Toolset via wixl (Linux)
sudo apt install wixl

# Criar WiX XML
# meu-app.wxs
```

### macOS - Application Bundle (.app)

```bash
# Estrutura
MeuApp.app/
├── Contents/
│   ├── Info.plist
│   ├── MacOS/
│   │   └── meu-app
│   └── Resources/
│       └── icon.icns

# Info.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>meu-app</string>
    <key>CFBundleIdentifier</key>
    <string>org.winux.MeuApp</string>
    <key>CFBundleName</key>
    <string>Meu App</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
</dict>
</plist>
```

### macOS - Disk Image (.dmg)

```bash
# Usando create-dmg (via brew no mac ou script)
create-dmg \
    --volname "Meu App" \
    --window-pos 200 120 \
    --window-size 600 400 \
    --icon-size 100 \
    --icon "MeuApp.app" 175 190 \
    --app-drop-link 425 190 \
    "MeuApp-1.0.dmg" \
    "MeuApp.app"
```

---

## IDEs e Editores

### VS Code (Recomendado)

Extensoes pre-configuradas:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "ms-dotnettools.csharp",
    "ms-python.python",
    "golang.go",
    "dart-code.flutter",
    "ms-vscode.cpptools"
  ]
}
```

### JetBrains IDEs

```bash
# Instalar Toolbox
wget https://download.jetbrains.com/toolbox/jetbrains-toolbox.tar.gz
tar -xzf jetbrains-toolbox.tar.gz
./jetbrains-toolbox*/jetbrains-toolbox

# IDEs disponiveis:
# - IntelliJ IDEA (Java, Kotlin)
# - CLion (C/C++, Rust)
# - PyCharm (Python)
# - WebStorm (JavaScript/TypeScript)
# - Rider (.NET)
# - GoLand (Go)
```

### Neovim

```lua
-- init.lua configurado com:
-- LSP para todas as linguagens
-- Treesitter para syntax highlighting
-- Telescope para fuzzy finding
-- nvim-dap para debugging
```

---

## Containers e Virtualizacao

### Docker

```bash
# Ja instalado e configurado
docker --version
docker compose --version

# Executar containers
docker run -it ubuntu:latest
docker compose up -d

# Build de imagem
docker build -t meu-app .
```

### Podman (alternativa rootless)

```bash
# Compativel com Docker
podman run -it ubuntu:latest
podman build -t meu-app .
```

### QEMU/KVM

```bash
# Virtualizacao completa
sudo apt install qemu-kvm virt-manager

# Criar VM
virt-manager
```

---

## Debugging e Profiling

### GDB

```bash
# Compilar com debug symbols
cargo build  # Debug mode ja inclui

# Debugar
gdb target/debug/meu-app
(gdb) break main
(gdb) run
(gdb) next
(gdb) print variavel
```

### LLDB

```bash
# Para Rust/C++
lldb target/debug/meu-app
(lldb) breakpoint set --name main
(lldb) run
```

### Valgrind

```bash
# Memory leaks
valgrind --leak-check=full ./meu-app

# Cache profiling
valgrind --tool=cachegrind ./meu-app
```

### Perf

```bash
# Performance profiling
perf record ./meu-app
perf report
```

### Flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bin meu-app
```

---

## Recursos Adicionais

### Links Uteis

- [Rust Book](https://doc.rust-lang.org/book/)
- [GTK4 Rust Book](https://gtk-rs.org/gtk4-rs/stable/latest/book/)
- [Relm4 Book](https://relm4.org/book/stable/)
- [Smithay Book](https://smithay.github.io/book/)

### Exemplos de Codigo

Veja os apps nativos do Winux em `/apps/` como referencia:
- `winux-about` - App simples com informacoes do sistema
- `winux-settings` - App complexo com multiplas paginas
- `winux-dev-hub` - App com integracao de sistema

---

**Winux OS Project - 2026**

*Desenvolvido com Rust e GTK4*
