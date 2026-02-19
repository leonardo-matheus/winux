# Winux OS - Documentacao dos Aplicativos

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         WINUX NATIVE APPS                                    │
│                                                                              │
│    Built with Rust + GTK4 + Adwaita for Maximum Performance                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

Este documento descreve todos os aplicativos nativos incluidos no Winux OS, suas funcionalidades, atalhos de teclado e opcoes de configuracao.

---

## Indice

1. [Winux Files](#winux-files)
2. [Winux Terminal](#winux-terminal)
3. [Winux Settings](#winux-settings)
4. [Winux Store](#winux-store)
5. [Winux Monitor](#winux-monitor)
6. [Winux Edit](#winux-edit)
7. [Winux Image](#winux-image)
8. [Winux Player](#winux-player)
9. [Winux About](#winux-about)
10. [Winux Personalize](#winux-personalize)
11. [Winux Environment Manager](#winux-environment-manager)
12. [Winux Dev Hub](#winux-dev-hub)
13. [Winux Builder](#winux-builder)

---

## Winux Files

**Gerenciador de arquivos moderno com suporte multi-plataforma**

```
┌────────────────────────────────────────────────────────────────┐
│ [<] [>] [^]  /home/user/Documents                    [≡] [⚙]  │
├──────────────┬─────────────────────────────────────────────────┤
│ Quick Access │  Name              Size      Modified           │
│ ─────────────│  ───────────────────────────────────────────    │
│ 🏠 Home      │  📁 Projects      --        Feb 18             │
│ 📄 Documents │  📁 Downloads     --        Feb 19             │
│ ⬇️  Downloads │  📄 report.pdf    1.2 MB    Feb 17             │
│ 🎵 Music     │  🖼️ photo.jpg     3.4 MB    Feb 15             │
│ 🖼️  Pictures  │  📦 backup.zip    45 MB     Feb 10             │
│ 🎬 Videos    │                                                 │
│ 💾 Drives    │                                                 │
└──────────────┴─────────────────────────────────────────────────┘
```

### Funcionalidades

| Feature | Descricao |
|:--------|:----------|
| **Navegacao em abas** | Multiplas abas para diferentes diretorios |
| **Preview de arquivos** | Pre-visualizacao de imagens, PDFs, texto |
| **Dual pane** | Visao lado a lado para copiar/mover |
| **Quick access** | Acesso rapido a pastas favoritas |
| **Busca integrada** | Busca recursiva por nome e conteudo |
| **Operacoes em lote** | Selecao multipla e operacoes em massa |

### Compatibilidade de Arquivos

**Windows:**
- `.exe` - Execucao via Wine, visualizacao de info
- `.msi` - Instalacao ou extracao
- `.dll` - Visualizacao de info
- `.lnk` - Resolucao de atalhos
- `.reg` - Importacao de registro
- `.bat`, `.ps1` - Visualizacao e edicao

**macOS:**
- `.dmg` - Montagem e extracao
- `.app` - Navegacao de bundles
- `.pkg` - Visualizacao e extracao
- `.plist` - Edicao de property lists
- `.icns` - Visualizacao de icones

**Linux:**
- `.deb` - Instalacao ou extracao
- `.rpm` - Instalacao ou extracao
- `.AppImage` - Execucao direta
- `.flatpak` - Instalacao
- `.snap` - Instalacao

**Arquivos Compactados:**
- ZIP, RAR, 7z, TAR, GZ, XZ, BZ2
- ISO, IMG (montagem)

### Atalhos de Teclado

| Atalho | Acao |
|:-------|:-----|
| `Ctrl+N` | Nova janela |
| `Ctrl+T` | Nova aba |
| `Ctrl+W` | Fechar aba |
| `Ctrl+L` | Focar barra de endereco |
| `Ctrl+F` | Buscar |
| `Ctrl+H` | Mostrar/ocultar arquivos ocultos |
| `Ctrl+C` | Copiar |
| `Ctrl+X` | Recortar |
| `Ctrl+V` | Colar |
| `Delete` | Mover para lixeira |
| `Shift+Delete` | Excluir permanentemente |
| `F2` | Renomear |
| `F5` | Atualizar |
| `Alt+Left` | Voltar |
| `Alt+Right` | Avancar |
| `Alt+Up` | Pasta pai |
| `Space` | Preview rapido |

### Configuracoes

Arquivo: `~/.config/winux-files/config.toml`

```toml
[general]
show_hidden = false
confirm_delete = true
single_click = false

[view]
default_view = "list"  # list, grid, compact
sort_by = "name"       # name, size, date, type
sort_descending = false
icon_size = 48

[sidebar]
show_places = true
show_devices = true
show_bookmarks = true
```

---

## Winux Terminal

**Emulador de terminal moderno com GPU acceleration**

```
┌────────────────────────────────────────────────────────────────┐
│ Terminal - bash                                    [−] [□] [×] │
├────────────────────────────────────────────────────────────────┤
│ user@winux:~$ neofetch                                         │
│                                                                │
│       ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄    user@winux                      │
│       ██▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀██    -----------                     │
│       ██  ▄▄▄▄▄    ▄▄▄▄▄ ██    OS: Winux 1.0 Aurora            │
│       ██  █████    █████ ██    Kernel: 6.8.0-zen                │
│       ██  ▀▀▀▀▀    ▀▀▀▀▀ ██    Shell: bash 5.2                  │
│       ██▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄██    DE: Winux Shell                  │
│       ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀    Terminal: winux-terminal         │
│                                                                │
│ user@winux:~$ █                                                │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Funcionalidades

| Feature | Descricao |
|:--------|:----------|
| **GPU Acceleration** | Renderizacao via wgpu/Vulkan |
| **Multiplas abas** | Organize sessoes em abas |
| **Split panes** | Divisao horizontal e vertical |
| **Temas** | Temas claros e escuros |
| **Transparencia** | Efeito acrilico no fundo |
| **Perfis** | Perfis de shell configuráveis |
| **Ligatures** | Suporte a fontes com ligatures |
| **Unicode** | Suporte completo a Unicode/emoji |

### Atalhos de Teclado

| Atalho | Acao |
|:-------|:-----|
| `Ctrl+Shift+T` | Nova aba |
| `Ctrl+Shift+W` | Fechar aba |
| `Ctrl+Shift+N` | Nova janela |
| `Ctrl+Tab` | Proxima aba |
| `Ctrl+Shift+Tab` | Aba anterior |
| `Ctrl+Shift+C` | Copiar selecao |
| `Ctrl+Shift+V` | Colar |
| `Ctrl+Shift+F` | Buscar |
| `Ctrl++` | Aumentar fonte |
| `Ctrl+-` | Diminuir fonte |
| `Ctrl+0` | Resetar tamanho da fonte |
| `Ctrl+Shift+E` | Split horizontal |
| `Ctrl+Shift+O` | Split vertical |
| `Alt+Arrow` | Navegar entre splits |

### Configuracoes

Arquivo: `~/.config/winux-terminal/config.toml`

```toml
[general]
default_shell = "/bin/bash"
working_directory = "~"
scrollback_lines = 10000

[appearance]
font_family = "JetBrains Mono"
font_size = 11
theme = "winux-dark"
opacity = 0.95
blur = true

[cursor]
shape = "block"  # block, underline, beam
blink = true
blink_interval = 500

[colors]
# Tema Winux Dark
background = "#1e1e2e"
foreground = "#cdd6f4"
cursor = "#f5e0dc"
selection = "#45475a"
```

---

## Winux Settings

**Central de configuracoes unificada do sistema**

```
┌────────────────────────────────────────────────────────────────┐
│ Configuracoes                                      [−] [□] [×] │
├────────────────────────────────────────────────────────────────┤
│ [🌐 Rede] [📶 Bluetooth] [⚡ Desempenho] [🎨 Aparencia]        │
│ [🖥️ Tela] [🔊 Som] [🔋 Energia] [🌍 Idioma] [ℹ️ Sobre]         │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ╭──────────────────────────────────────────────────────────╮  │
│  │ Modo de Desempenho                                       │  │
│  ├──────────────────────────────────────────────────────────┤  │
│  │ ○ Economico      - Maximiza duracao da bateria          │  │
│  │ ● Balanceado     - Equilibrio entre desempenho          │  │
│  │ ○ Alto Desempenho- Maximo desempenho                    │  │
│  │ ○ Gaming         - Otimizado para jogos                 │  │
│  ╰──────────────────────────────────────────────────────────╯  │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Paginas Disponiveis

| Pagina | Funcionalidades |
|:-------|:----------------|
| **Rede** | Wi-Fi, Ethernet, VPN, Proxy |
| **Bluetooth** | Dispositivos, pareamento, audio |
| **Desempenho** | Modo de energia, CPU governor, GPU |
| **Aparencia** | Tema, cores, fontes |
| **Tela** | Resolucao, refresh rate, escala, luz noturna |
| **Som** | Saida/entrada, volume, alertas |
| **Energia** | Bateria, suspensao, economia |
| **Idioma** | Idioma, teclado, metodo de entrada |
| **Sobre** | Info do sistema, hardware, versao |

### Atalhos de Teclado

| Atalho | Acao |
|:-------|:-----|
| `Ctrl+1-9` | Ir para pagina N |
| `Ctrl+F` | Buscar configuracao |
| `Escape` | Voltar/Fechar |

---

## Winux Store

**Loja de aplicativos unificada**

```
┌────────────────────────────────────────────────────────────────┐
│ Winux Store                      [🔍 Buscar apps...]  [⚙] [×]  │
├────────────────────────────────────────────────────────────────┤
│ [Explorar] [Instalados] [Atualizacoes]                         │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Em Destaque                                                   │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐                   │
│  │ VS Code│ │ Firefox│ │ VLC    │ │ GIMP   │                   │
│  │ ★★★★★  │ │ ★★★★★  │ │ ★★★★☆  │ │ ★★★★☆  │                   │
│  │[Instalar]│[Instalar]│[Instalar]│[Instalar]                  │
│  └────────┘ └────────┘ └────────┘ └────────┘                   │
│                                                                │
│  Categorias                                                    │
│  [Produtividade] [Desenvolvimento] [Jogos] [Multimidia]       │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Funcionalidades

| Feature | Descricao |
|:--------|:----------|
| **Multi-backend** | APT, Flatpak, Snap em uma interface |
| **Busca unificada** | Busque em todas as fontes |
| **Categorias** | Navegue por categoria |
| **Avaliacoes** | Reviews e ratings de usuarios |
| **Atualizacoes** | Gerenciamento centralizado |
| **Screenshots** | Previews dos apps |
| **Permissoes** | Visualize permissoes Flatpak |

### Atalhos de Teclado

| Atalho | Acao |
|:-------|:-----|
| `Ctrl+F` | Buscar |
| `Ctrl+R` | Atualizar lista |
| `Ctrl+U` | Ver atualizacoes |
| `Escape` | Cancelar busca |

---

## Winux Monitor

**Monitor de sistema em tempo real**

```
┌────────────────────────────────────────────────────────────────┐
│ System Monitor                                     [−] [□] [×] │
├────────────────────────────────────────────────────────────────┤
│ [📊 Performance] [📋 Processos] [🚀 Startup] [ℹ️ Sistema]       │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  CPU Usage                        Memory                       │
│  ▓▓▓▓▓▓▓▓░░░░░░░░░░░░ 42%        ▓▓▓▓▓▓▓▓▓▓▓▓░░░░ 8.2/16 GB   │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │     100% ┤                                               │  │
│  │      75% ┤    ╭─╮                                        │  │
│  │      50% ┤╭──╯  ╰─╮  ╭──╮                               │  │
│  │      25% ┤╯       ╰──╯  ╰───────────────────            │  │
│  │       0% ┼────────────────────────────────────           │  │
│  │          0s    30s    60s    90s    120s                │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Funcionalidades

| Aba | Descricao |
|:----|:----------|
| **Performance** | Graficos de CPU, RAM, disco, rede |
| **Processos** | Lista de processos, uso de recursos |
| **Startup** | Aplicativos de inicializacao |
| **Sistema** | Informacoes de hardware |

### Atalhos de Teclado

| Atalho | Acao |
|:-------|:-----|
| `Ctrl+E` | Terminar processo |
| `Ctrl+K` | Matar processo |
| `Ctrl+F` | Buscar processo |
| `F5` | Atualizar |

---

## Winux Edit

**Editor de texto avancado com syntax highlighting**

```
┌────────────────────────────────────────────────────────────────┐
│ Edit - main.rs                                     [−] [□] [×] │
├────────────────────────────────────────────────────────────────┤
│ File  Edit  View  Search  Tools  Help                          │
├────────────────────────────────────────────────────────────────┤
│  1 │ use gtk4::prelude::*;                                     │
│  2 │ use gtk4::Application;                                    │
│  3 │ use libadwaita as adw;                                    │
│  4 │                                                           │
│  5 │ fn main() -> glib::ExitCode {                             │
│  6 │     adw::init().expect("Failed");                         │
│  7 │     let app = Application::builder()                      │
│  8 │         .application_id("org.winux.App")                  │
│  9 │         .build();                                         │
│ 10 │     app.run()                                             │
│ 11 │ }                                                         │
├────────────────────────────────────────────────────────────────┤
│ Rust │ UTF-8 │ LF │ Ln 5, Col 1 │                              │
└────────────────────────────────────────────────────────────────┘
```

### Funcionalidades

| Feature | Descricao |
|:--------|:----------|
| **Syntax Highlighting** | 100+ linguagens via tree-sitter |
| **Numeracao de linhas** | Com destaque da linha atual |
| **Busca e substituicao** | Com suporte a regex |
| **Multiplos encodings** | UTF-8, ISO-8859-1, etc |
| **Auto-indentacao** | Indentacao inteligente |
| **Bracket matching** | Destaque de parenteses |
| **Minimap** | Visao geral do arquivo |
| **Multiplos cursores** | Edicao simultanea |

### Atalhos de Teclado

| Atalho | Acao |
|:-------|:-----|
| `Ctrl+N` | Novo arquivo |
| `Ctrl+O` | Abrir arquivo |
| `Ctrl+S` | Salvar |
| `Ctrl+Shift+S` | Salvar como |
| `Ctrl+Z` | Desfazer |
| `Ctrl+Y` | Refazer |
| `Ctrl+F` | Buscar |
| `Ctrl+H` | Buscar e substituir |
| `Ctrl+G` | Ir para linha |
| `Ctrl+D` | Duplicar linha |
| `Ctrl+/` | Comentar linha |
| `Ctrl+L` | Selecionar linha |
| `Ctrl+Shift+K` | Deletar linha |
| `Alt+Up/Down` | Mover linha |
| `Ctrl+Click` | Adicionar cursor |

---

## Winux Image

**Visualizador de imagens com edicao basica**

### Funcionalidades

| Feature | Descricao |
|:--------|:----------|
| **Formatos** | PNG, JPG, GIF, WebP, SVG, RAW |
| **Zoom** | Zoom suave com scroll |
| **Rotacao** | Rotacao em 90 graus |
| **Slideshow** | Apresentacao de slides |
| **Exif** | Visualizacao de metadados |
| **Crop** | Recorte de imagens |
| **Filtros** | Filtros basicos |

### Atalhos de Teclado

| Atalho | Acao |
|:-------|:-----|
| `Left/Right` | Imagem anterior/proxima |
| `+/-` | Zoom in/out |
| `0` | Zoom 100% |
| `F` | Ajustar a janela |
| `R` | Rotacionar 90 CW |
| `Shift+R` | Rotacionar 90 CCW |
| `F11` | Tela cheia |
| `Space` | Iniciar slideshow |

---

## Winux Player

**Player multimidia com suporte a video e audio**

### Funcionalidades

| Feature | Descricao |
|:--------|:----------|
| **Formatos** | MP4, MKV, AVI, MP3, FLAC, OGG |
| **Playlists** | Gerenciamento de playlists |
| **Legendas** | SRT, ASS, VTT |
| **Equalizador** | Equalizador de 10 bandas |
| **Visualizacao** | Visualizacoes de audio |
| **Picture-in-picture** | Modo flutuante |

### Atalhos de Teclado

| Atalho | Acao |
|:-------|:-----|
| `Space` | Play/Pause |
| `Left/Right` | Seek -/+ 10s |
| `Up/Down` | Volume +/- |
| `M` | Mudo |
| `F` | Tela cheia |
| `S` | Legenda on/off |
| `N/P` | Proxima/anterior |
| `L` | Loop |

---

## Winux About

**Informacoes detalhadas do sistema**

```
┌────────────────────────────────────────────────────────────────┐
│ Sobre o Winux                                      [−] [□] [×] │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│                    🖥️ Winux OS                                  │
│                   Developer Edition                            │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ Sistema Operacional                                    │   │
│  ├────────────────────────────────────────────────────────┤   │
│  │ Nome          │ Winux OS                               │   │
│  │ Versao        │ 1.0                                    │   │
│  │ Codename      │ Aurora                                 │   │
│  │ Kernel        │ 6.8.0-zen                              │   │
│  │ Arquitetura   │ x86_64                                 │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ Hardware                                               │   │
│  ├────────────────────────────────────────────────────────┤   │
│  │ Processador   │ AMD Ryzen 7 5800X                      │   │
│  │ Memoria       │ 32 GB DDR4                             │   │
│  │ GPU           │ NVIDIA RTX 3070                        │   │
│  │ Armazenamento │ 500 GB NVMe                            │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Informacoes Exibidas

- Nome e versao do OS
- Versao do kernel
- Arquitetura do sistema
- Informacoes de CPU (modelo, cores, threads)
- Memoria RAM total
- Informacoes de GPU
- Armazenamento disponivel
- Sessao de desktop
- Servidor grafico (Wayland/X11)
- Hostname e usuario

---

## Winux Personalize

**Personalizacao completa da interface**

```
┌────────────────────────────────────────────────────────────────┐
│ Personalizar                                       [−] [□] [×] │
├────────────────────────────────────────────────────────────────┤
│ [🎨 Estilos] [🖌️ Temas] [🖼️ Wallpapers] [📁 Icones]            │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Escolha seu Estilo                                            │
│                                                                │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐           │
│  │   Windows    │ │    Linux     │ │     Mac      │           │
│  │   ┌──────┐   │ │   ┌──────┐   │ │   ┌──────┐   │           │
│  │   │▓▓▓▓▓▓│   │ │   │▓▓▓▓▓▓│   │ │   │▓▓▓▓▓▓│   │           │
│  │   │      │   │ │   │      │   │ │   │      │   │           │
│  │   └──────┘   │ │   └──────┘   │ │   └──────┘   │           │
│  │   [______]   │ │   [      ]   │ │   [  ····  ] │           │
│  │   ○ Select   │ │   ○ Select   │ │   ○ Select   │           │
│  └──────────────┘ └──────────────┘ └──────────────┘           │
│                                                                │
│            [ Aplicar Estilo Selecionado ]                      │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Modos de Interface

| Modo | Descricao |
|:-----|:----------|
| **Windows Like** | Barra de tarefas fixa na parte inferior, Menu Iniciar |
| **Linux Like** | Barra superior Activities, Dash to Dock |
| **Mac Like** | Menu bar superior, Dock centralizado, botoes coloridos |

### Opcoes de Personalizacao

**Temas:**
- Esquema de cores (Claro/Escuro/Automatico)
- Cor de destaque (Azul, Verde, Roxo, Rosa, Laranja)

**Wallpapers:**
- Winux Default
- Winux Dark
- Winux Gradient
- Personalizado

**Icones:**
- Papirus
- Adwaita
- Winux Icons

---

## Winux Environment Manager

**Gerenciador de variaveis de ambiente**

### Funcionalidades

| Feature | Descricao |
|:--------|:----------|
| **Linguagens** | Config para Node, Java, Python, Rust, Go, PHP |
| **PATH** | Gerenciamento visual do PATH |
| **Perfis** | Perfis de ambiente por projeto |
| **Export** | Exportar configuracoes |

### Linguagens Configuradas

- **Node.js/NVM**: `NVM_DIR`, `NODE_PATH`
- **Java/JDK**: `JAVA_HOME`, `MAVEN_HOME`, `GRADLE_HOME`
- **Python**: `PYTHONPATH`, `PIPENV_*`, `POETRY_*`
- **Rust**: `CARGO_HOME`, `RUSTUP_HOME`
- **Go**: `GOPATH`, `GOROOT`
- **PHP**: `COMPOSER_HOME`

---

## Winux Dev Hub

**Central de desenvolvimento integrada**

```
┌────────────────────────────────────────────────────────────────┐
│ Dev Hub                                           [🔄] [⚙] [×] │
├────────────────────────────────────────────────────────────────┤
│ [📂 Projetos] [🌍 Ambientes] [🔧 Toolchains] [📦 Containers]    │
│ [🗄️ Databases] [⚙️ Servicos]                                    │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Projetos Recentes                                             │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ 📁 winux-os        Rust/GTK4     Ultimo: 2 horas       │   │
│  │ 📁 my-flutter-app  Flutter       Ultimo: 1 dia         │   │
│  │ 📁 web-api         .NET 8        Ultimo: 3 dias        │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                                │
│  Toolchains Instalados                                         │
│  [✓] Rust 1.75   [✓] Node 20   [✓] Python 3.12              │
│  [✓] Go 1.22     [✓] .NET 8    [✓] Java 21                  │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Paginas

| Pagina | Funcionalidade |
|:-------|:---------------|
| **Projetos** | Auto-deteccao e dashboard de projetos |
| **Ambientes** | Variaveis de ambiente por perfil |
| **Toolchains** | Status e gerenciamento de SDKs |
| **Containers** | Docker/Podman management |
| **Databases** | PostgreSQL, MySQL, MongoDB, Redis |
| **Servicos** | systemd services control |

---

## Winux Builder

**Build cross-platform para todas as plataformas**

```
┌────────────────────────────────────────────────────────────────┐
│ Builder                                           [−] [□] [×] │
├────────────────────────────────────────────────────────────────┤
│ [📁 Projeto] [🔨 Build] [📟 Terminal]                           │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Projeto Atual: /home/user/projects/my-app                     │
│  Tipo: Rust/Cargo                                              │
│                                                                │
│  Targets de Build                                              │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ [✓] Linux   .deb  .rpm  .AppImage  .flatpak           │   │
│  │ [✓] Windows .exe  .msi                                 │   │
│  │ [ ] macOS   .app  .dmg  .pkg                          │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                                │
│  Modo: ○ Debug  ● Release                                      │
│                                                                │
│  [ Iniciar Build ] [ Limpar ] [ Configuracoes ]                │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Funcionalidades

| Feature | Descricao |
|:--------|:----------|
| **Auto-deteccao** | Detecta tipo de projeto automaticamente |
| **Multi-target** | Build para multiplas plataformas |
| **Perfis** | Salve configuracoes de build |
| **Terminal** | Output de build em tempo real |
| **Historico** | Historico de builds anteriores |

### Plataformas Suportadas

**Linux:**
- `.deb` (Debian/Ubuntu)
- `.rpm` (Fedora/RHEL)
- `.AppImage`
- `.flatpak`

**Windows:**
- `.exe` (Portable)
- `.msi` (Installer)

**macOS:**
- `.app` (Bundle)
- `.dmg` (Disk Image)
- `.pkg` (Installer)

### Tipos de Projeto

- Rust/Cargo
- .NET/dotnet
- Node.js/npm
- Flutter/dart
- Electron

---

## Arquivos de Configuracao

Todos os apps armazenam configuracoes em:
```
~/.config/winux-<app-name>/
```

Formato padrao: TOML ou JSON

---

**Winux OS Project - 2026**

*Built with Rust and GTK4*
