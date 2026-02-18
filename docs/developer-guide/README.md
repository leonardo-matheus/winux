# Winux OS - Guia do Desenvolvedor

> Documentacao tecnica para contribuidores e desenvolvedores

---

## Indice

1. [Arquitetura do Sistema](#arquitetura-do-sistema)
2. [Como Contribuir](#como-contribuir)
3. [Build das Aplicacoes](#build-das-aplicacoes)
4. [Criacao da ISO](#criacao-da-iso)
5. [Testes](#testes)
6. [Release Process](#release-process)

---

## Arquitetura do Sistema

### Visao Geral

O Winux OS e uma distribuicao Linux baseada no Ubuntu 24.04 LTS, com foco em:
- Interface familiar para usuarios Windows
- Performance otimizada para gaming
- Aplicacoes nativas em Rust

### Stack Tecnologico

| Camada | Tecnologia |
|--------|------------|
| Base | Ubuntu 24.04 LTS (Noble Numbat) |
| Kernel | Linux Zen com patches de performance |
| Display Server | Wayland (compositor proprio) |
| Compositor | Smithay (Rust) |
| Desktop Shell | winux-shell (Rust/GTK4) |
| Toolkit | GTK4 + libadwaita |
| Audio | PipeWire + WirePlumber |
| Packaging | APT + Flatpak |
| Gaming | Wine + Proton + DXVK + VKD3D |

### Componentes Principais

```
winux/
├── apps/                   # Aplicacoes nativas
│   ├── winux-files/        # Gerenciador de arquivos
│   ├── winux-terminal/     # Emulador de terminal
│   ├── winux-settings/     # Configuracoes do sistema
│   ├── winux-store/        # Loja de aplicativos
│   ├── winux-monitor/      # Monitor de sistema
│   └── winux-edit/         # Editor de texto
│
├── desktop/                # Ambiente de desktop
│   ├── winux-compositor/   # Compositor Wayland (Smithay)
│   ├── winux-panel/        # Barra de tarefas
│   └── winux-shell/        # Shell integrado
│
├── compatibility/          # Camada de compatibilidade
│   ├── scripts/            # Scripts de configuracao Wine/Proton
│   └── configs/            # Configuracoes padrao
│
├── system/                 # Configuracoes de sistema
│   └── etc/                # Arquivos de configuracao
│
├── build/                  # Sistema de build
│   └── scripts/            # Scripts de construcao
│
├── drivers/                # Gerenciamento de drivers
└── themes/                 # Temas visuais
```

### Dependencias de Build

#### Sistema Base
```bash
# Ubuntu/Debian
sudo apt install build-essential git curl wget

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Bibliotecas de desenvolvimento
sudo apt install \
    libgtk-4-dev \
    libadwaita-1-dev \
    libpango1.0-dev \
    libcairo2-dev \
    libglib2.0-dev \
    libgio2.0-dev \
    libsystemd-dev \
    libpipewire-0.3-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libinput-dev \
    libudev-dev \
    libdrm-dev \
    libgbm-dev \
    libvulkan-dev \
    pkg-config \
    cmake \
    meson \
    ninja-build
```

#### Ferramentas de ISO
```bash
sudo apt install \
    debootstrap \
    squashfs-tools \
    xorriso \
    grub-pc-bin \
    grub-efi-amd64-bin \
    mtools \
    dosfstools
```

---

## Como Contribuir

### Fluxo de Trabalho Git

1. **Fork do Repositorio**
   ```bash
   # Clone seu fork
   git clone https://github.com/SEU_USUARIO/winux.git
   cd winux

   # Adicione o upstream
   git remote add upstream https://github.com/winux-os/winux.git
   ```

2. **Criar Branch de Feature**
   ```bash
   # Atualize develop
   git checkout develop
   git pull upstream develop

   # Crie branch de feature
   git checkout -b feature/minha-feature
   ```

3. **Commits**
   ```bash
   # Use Conventional Commits
   git commit -m "feat(component): add new feature"
   git commit -m "fix(shell): resolve crash on startup"
   git commit -m "docs(readme): update installation guide"
   ```

4. **Push e Pull Request**
   ```bash
   git push origin feature/minha-feature
   # Abra PR no GitHub para develop
   ```

### Conventional Commits

Usamos o padrao [Conventional Commits](https://www.conventionalcommits.org/):

| Prefixo | Uso |
|---------|-----|
| `feat` | Nova feature |
| `fix` | Correcao de bug |
| `docs` | Documentacao |
| `style` | Formatacao (sem mudanca de codigo) |
| `refactor` | Refatoracao |
| `perf` | Melhoria de performance |
| `test` | Adicao de testes |
| `chore` | Manutencao geral |
| `build` | Sistema de build |
| `ci` | CI/CD |

### Estrutura de Branch

```
main           # Releases estaveis
  │
develop        # Integracao de desenvolvimento
  │
  ├── feature/xxx    # Features em desenvolvimento
  ├── fix/xxx        # Correcoes
  ├── sprint/XX-nome # Branches de sprint
  └── release/X.X    # Preparacao de release
```

### Code Style

#### Rust
```bash
# Formatar codigo
cargo fmt

# Verificar linting
cargo clippy -- -W clippy::all

# Pre-commit hook sugerido
#!/bin/bash
cargo fmt --check
cargo clippy -- -D warnings
```

#### Configuracao do Editor

`.editorconfig`:
```ini
root = true

[*]
indent_style = space
indent_size = 4
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.{yml,yaml,json,toml}]
indent_size = 2

[Makefile]
indent_style = tab
```

### Revisao de Codigo

Todos os PRs requerem:
1. Pelo menos 1 aprovacao de reviewer
2. CI passando (build + tests)
3. Sem conflitos com develop
4. Commits squashed se necessario

---

## Build das Aplicacoes

### Workspace Cargo

O projeto usa Cargo workspace para gerenciar multiplos crates:

```toml
# Cargo.toml (raiz)
[workspace]
members = [
    "apps/winux-files",
    "apps/winux-terminal",
    "apps/winux-settings",
    "apps/winux-store",
    "apps/winux-monitor",
    "apps/winux-edit",
    "desktop/winux-compositor",
    "desktop/winux-panel",
    "desktop/winux-shell",
]

[workspace.package]
version = "1.0.0"
authors = ["Winux OS Team"]
edition = "2021"
license = "MIT"
repository = "https://github.com/winux-os/winux"

[workspace.dependencies]
gtk4 = "0.7"
libadwaita = "0.5"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
anyhow = "1"
tracing = "0.1"
```

### Build Individual

```bash
# Build de um componente especifico
cargo build -p winux-shell

# Build em modo release
cargo build -p winux-shell --release

# Executar
cargo run -p winux-shell
```

### Build Completo

```bash
# Build de todo o workspace
cargo build --workspace

# Release build
cargo build --workspace --release

# Com otimizacoes agressivas
RUSTFLAGS="-C target-cpu=native" cargo build --workspace --release
```

### Usando o Makefile

```bash
# Build de desenvolvimento
make build

# Build de release
make release

# Limpar builds
make clean

# Executar testes
make test

# Instalar localmente
make install

# Build da ISO
make iso
```

### Estrutura de um Aplicativo

Exemplo: `apps/winux-files/`

```
winux-files/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Application struct
│   ├── window.rs         # Main window
│   ├── sidebar.rs        # Sidebar component
│   ├── file_view.rs      # File listing
│   ├── operations.rs     # File operations
│   └── config.rs         # Settings
├── resources/
│   ├── icons/
│   ├── ui/               # .ui files (GTK Builder)
│   └── style.css
└── data/
    ├── org.winux.Files.desktop
    ├── org.winux.Files.metainfo.xml
    └── org.winux.Files.gschema.xml
```

### Geracao de Pacotes .deb

```bash
# Instalar cargo-deb
cargo install cargo-deb

# Gerar pacote
cargo deb -p winux-files

# Output: target/debian/winux-files_1.0.0_amd64.deb
```

Configuracao em `Cargo.toml`:

```toml
[package.metadata.deb]
maintainer = "Winux OS Team <team@winux-os.org>"
copyright = "2026, Winux OS Project"
license-file = ["LICENSE", "0"]
extended-description = "Gerenciador de arquivos do Winux OS"
depends = "$auto"
section = "utils"
priority = "optional"
assets = [
    ["target/release/winux-files", "usr/bin/", "755"],
    ["resources/*", "usr/share/winux-files/", "644"],
    ["data/org.winux.Files.desktop", "usr/share/applications/", "644"],
]
```

---

## Criacao da ISO

### Processo de Build

O build da ISO segue estas etapas:

```
1. Preparacao     -> Criar diretorios de trabalho
2. Debootstrap    -> Sistema base Ubuntu
3. Chroot         -> Configurar ambiente
4. Pacotes        -> Instalar componentes Winux
5. Customizacao   -> Aplicar temas e configs
6. Squashfs       -> Comprimir sistema
7. ISO            -> Gerar imagem bootavel
```

### Script Principal

```bash
# Build completo (requer root)
sudo ./build/scripts/build-winux-iso.sh all

# Fases individuais
sudo ./build/scripts/build-winux-iso.sh prepare
sudo ./build/scripts/build-winux-iso.sh debootstrap
sudo ./build/scripts/build-winux-iso.sh install
sudo ./build/scripts/build-winux-iso.sh squashfs
sudo ./build/scripts/build-winux-iso.sh iso

# Limpar builds anteriores
sudo ./build/scripts/build-winux-iso.sh clean
```

### Variaveis de Ambiente

```bash
# Diretorio de build (default: /tmp/winux-build)
export BUILD_DIR=/mnt/fast-storage/winux-build

# Diretorio de output (default: ./output)
export OUTPUT_DIR=/mnt/releases

# Versao customizada
export WINUX_VERSION="1.1"
export WINUX_CODENAME="borealis"
```

### Customizacao da ISO

Para customizar a ISO, edite os arquivos em:

- `build/configs/packages.list` - Lista de pacotes
- `build/configs/chroot-scripts/` - Scripts executados no chroot
- `build/branding/` - Logotipos e branding
- `build/calamares/` - Configuracao do instalador

### Testando a ISO

```bash
# Testar com QEMU
qemu-system-x86_64 \
    -enable-kvm \
    -m 4G \
    -cpu host \
    -cdrom output/winux-1.0-aurora-amd64.iso \
    -boot d

# Com UEFI (requer OVMF)
qemu-system-x86_64 \
    -enable-kvm \
    -m 4G \
    -cpu host \
    -bios /usr/share/OVMF/OVMF_CODE.fd \
    -cdrom output/winux-1.0-aurora-amd64.iso \
    -boot d
```

---

## Testes

### Testes Unitarios

```bash
# Executar todos os testes
cargo test --workspace

# Testes de um componente
cargo test -p winux-files

# Com output verbose
cargo test --workspace -- --nocapture

# Testes especificos
cargo test -p winux-files test_copy_file
```

### Testes de Integracao

```bash
# Testes de integracao (requerem display)
cargo test --workspace --features integration-tests

# Com Xvfb (headless)
xvfb-run cargo test --workspace --features integration-tests
```

### Testes de Performance

```bash
# Benchmarks
cargo bench --workspace

# Profiling com flamegraph
cargo install flamegraph
sudo cargo flamegraph -p winux-shell
```

### CI/CD Pipeline

GitHub Actions workflow:

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [develop]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-4-dev libadwaita-1-dev

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Build
        run: cargo build --workspace

      - name: Test
        run: cargo test --workspace

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Format
        run: cargo fmt --check
```

---

## Release Process

### Versionamento

Seguimos [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (ex: 1.2.3)
- MAJOR: Mudancas incompativeis
- MINOR: Features retrocompativeis
- PATCH: Bug fixes retrocompativeis

### Checklist de Release

1. **Preparacao**
   - [ ] Todos os PRs mergeados em develop
   - [ ] Testes passando
   - [ ] CHANGELOG atualizado
   - [ ] Versao atualizada em Cargo.toml

2. **Branch de Release**
   ```bash
   git checkout develop
   git pull
   git checkout -b release/1.0.0
   ```

3. **Finalizacao**
   - [ ] Testes finais
   - [ ] Build da ISO
   - [ ] Teste em hardware real
   - [ ] Documentacao atualizada

4. **Merge e Tag**
   ```bash
   # Merge para main
   git checkout main
   git merge release/1.0.0

   # Tag
   git tag -a v1.0.0 -m "Release 1.0.0 Aurora"
   git push origin main --tags

   # Merge de volta para develop
   git checkout develop
   git merge main
   git push origin develop
   ```

5. **Publicacao**
   - Upload ISO para mirrors
   - Release no GitHub
   - Anuncio em redes sociais

### Script de Release

```bash
# Criar release automatizada
./build/scripts/create-release.sh 1.0.0 aurora

# Isso executa:
# 1. Build de todos os componentes
# 2. Geracao da ISO
# 3. Calculo de checksums
# 4. Assinatura GPG (se configurado)
# 5. Upload para mirrors (se configurado)
```

---

## Recursos Adicionais

### Documentacao

- [Arquitetura Detalhada](../architecture/OVERVIEW.md)
- [API Reference](https://docs.winux-os.org/api)
- [Wiki](https://wiki.winux-os.org)

### Comunidade

- **Discord**: discord.gg/winux
- **Forum**: forum.winux-os.org
- **Matrix**: #winux:matrix.org

### Links Uteis

- [GTK4 Documentation](https://docs.gtk.org/gtk4/)
- [Smithay Book](https://smithay.github.io/smithay/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Freedesktop Specifications](https://specifications.freedesktop.org/)

---

**Winux OS Project - 2026**
*Desenvolvido pela comunidade, para a comunidade*
