<div align="center">

<img src="assets/branding/logo.png" alt="Winux OS Logo" width="200"/>

# WINUX OS

### **O Sistema Operacional Definitivo para Desenvolvedores e Gamers**

*A potência do Linux + A familiaridade do Windows + A elegância do macOS*

[![Version](https://img.shields.io/badge/version-1.2_Blaze-ff6b35?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/leonardo-matheus/winux/releases)
[![License](https://img.shields.io/badge/license-GPL_v3_|_MIT-00ff88?style=for-the-badge&logo=opensourceinitiative&logoColor=white)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-190,000+_LOC-ff00ff?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Apps](https://img.shields.io/badge/Native_Apps-48+-00d4ff?style=for-the-badge&logo=gnome&logoColor=white)](docs/APPS.md)
[![GTK4](https://img.shields.io/badge/UI-GTK4_+_Libadwaita-4A86CF?style=for-the-badge&logo=gnome&logoColor=white)](https://gtk.org)
[![Android](https://img.shields.io/badge/Android-App-3DDC84?style=for-the-badge&logo=android&logoColor=white)](mobile/winux-connect-android/)

[**Download**](#-download) | [**Features**](#-features) | [**Apps**](#-48-apps-nativos) | [**v1.2 Blaze**](#-novidades-v12-blaze) | [**Docs**](docs/)

---

</div>

## O Que é o Winux OS?

**Winux OS** é uma distribuição Linux revolucionária construída do zero para oferecer a **melhor experiência** possível para desenvolvedores e gamers. Com **48 aplicativos nativos** escritos em Rust, **integração com smartphones**, **AI Assistant integrado**, e **cloud sync nativo**.

```
╔══════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                      ║
║     ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗     ██████╗ ███████╗                   ║
║     ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝    ██╔═══██╗██╔════╝                   ║
║     ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝     ██║   ██║███████╗                   ║
║     ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗     ██║   ██║╚════██║                   ║
║     ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗    ╚██████╔╝███████║                   ║
║      ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝     ╚═════╝ ╚══════╝                   ║
║                                                                                      ║
║                           ═══ v1.2 BLAZE ═══                                         ║
║                                                                                      ║
║   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            ║
║   │   48 APPS    │  │   CONNECT    │  │   CLOUD      │  │     AI       │            ║
║   │   NATIVOS    │  │  SMARTPHONE  │  │    SYNC      │  │  ASSISTANT   │            ║
║   └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘            ║
║                                                                                      ║
║   190,000+ LOC Rust │ 7 Temas │ Android App │ GPT-4o │ 6 Cloud Providers            ║
║                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════╝
```

---

## 🔥 Novidades v1.2 Blaze

<table>
<tr>
<td width="50%" valign="top">

### 📱 Winux Connect
**Integração completa com smartphones**

- App Desktop (Rust + GTK4)
- **App Android** (Kotlin + Jetpack Compose)
- Espelhamento de notificações
- Envio/recebimento de SMS
- Transferência de arquivos
- Clipboard sync automático
- Controle de mídia remoto
- Screen mirroring (scrcpy)
- Protocolo KDE Connect compatível

</td>
<td width="50%" valign="top">

### 🤖 Winux AI
**Assistente de IA integrado**

- **GPT-4o** e **o1** via Azure OpenAI
- Streaming responses em tempo real
- Assistente de código com syntax highlighting
- Análise de arquivos e imagens (Vision)
- Terminal helper
- Tradução e resumo de documentos
- D-Bus service para AI em todo o sistema
- Histórico de conversas (SQLite)

</td>
</tr>
<tr>
<td width="50%" valign="top">

### ☁️ Winux Cloud
**Cloud sync nativo**

- **Google Drive**, **OneDrive**, **Dropbox**
- **Nextcloud**, **WebDAV**, **S3**
- OAuth2 authentication
- Sync bidirecional com delta sync
- Resolução de conflitos
- File system watcher (inotify)
- Criptografia client-side
- Selective sync

</td>
<td width="50%" valign="top">

### 🔌 Shell Plugins
**Sistema extensível**

- API para panel widgets
- Notification handlers
- Launcher providers
- Hot reload de plugins
- Sandboxing com 40+ permissões

**Built-in:**
- Weather widget
- System monitor
- Clipboard indicator
- Caffeine, Pomodoro
- Music controls (MPRIS)

</td>
</tr>
</table>

---

## ✨ Features

<table>
<tr>
<td width="50%" valign="top">

### 🚀 Performance Extrema
- **Kernel Linux Zen 6.8+** otimizado
- **Compositor Wayland** (Smithay) nativo
- **PipeWire** para áudio de baixa latência
- **GameMode** e **MangoHud** integrados
- Scripts de otimização automática

### 🎨 Design Premium
- **7 temas** com animações 60fps
- Glassmorphism e blur effects
- 105 ícones SVG customizados
- Cursores modernos multi-DPI
- Plymouth boot animation

### 🔧 Compatibilidade Total
- Arquivos **Windows** (.exe, .msi, .dll, .lnk, .reg)
- Arquivos **macOS** (.dmg, .app, .pkg, .plist, .icns)
- Arquivos **Linux** (.deb, .rpm, .AppImage, .flatpak, .snap)
- ISO/IMG mount e extract

</td>
<td width="50%" valign="top">

### 💻 Desenvolvimento Completo
- **.NET 8**, **C++**, **Rust**, **Swift**, **Go**, **Java**
- **Android Studio** + SDK/NDK
- **Flutter** + **React Native**
- **VS Code**, **JetBrains**, **Neovim**
- **Docker** + **Podman**

### 🎮 Gaming First-Class
- **Steam**, **GOG**, **Epic Games** (via Heroic)
- **Wine 9.0** + **Proton GE**
- **DXVK** + **VKD3D**
- **RetroArch** + emuladores
- Otimização automática por jogo

### 📱 Mobile Development
- **Android** APK/AAB builds
- **iOS** IPA (via Theos/ldid)
- **Flutter** cross-platform
- **React Native** toolchain
- **Swift** para Linux

</td>
</tr>
</table>

---

## 📦 48 Apps Nativos

Todos escritos em **Rust** com **GTK4/Libadwaita** para máxima performance e design moderno.

### Core Apps

| App | Descrição | Features |
|:----|:----------|:---------|
| 📁 **Files** | Gerenciador de arquivos | Multi-abas, preview, compatibilidade Win/Mac/Linux |
| 💻 **Terminal** | Emulador de terminal | GPU-accelerated, tabs, profiles |
| ⚙️ **Settings** | Central de configurações | 15+ páginas, integração sistema |
| 🏪 **Store** | Loja de aplicativos | APT + Flatpak + Snap unificado |
| 📊 **Monitor** | System monitor | CPU, RAM, GPU, rede em tempo real |
| 📝 **Edit** | Editor de texto | Syntax highlighting, LSP |
| 🖼️ **Image** | Visualizador de imagens | Filtros, edição básica, batch |
| 🎬 **Player** | Player multimídia | Vídeo, áudio, playlists |

### Produtividade

| App | Descrição | Features |
|:----|:----------|:---------|
| 📅 **Calendar** | Calendário e tarefas | CalDAV sync, lembretes, tarefas |
| 📝 **Notes** | Notas (estilo Keep) | Markdown, cores, tags, SQLite FTS |
| 👥 **Contacts** | Gerenciador de contatos | CardDAV sync, vCard import/export |
| 📧 **Mail** | Cliente de email | IMAP/SMTP, OAuth2, HTML |
| 🧮 **Calculator** | Calculadora | Básica, científica, programador, conversões |
| 🕐 **Clock** | Relógio | World clock, alarmes, cronômetro, timer |
| 🌤️ **Weather** | Previsão do tempo | Open-Meteo API, 7 dias, hourly |

### Utilitários

| App | Descrição | Features |
|:----|:----------|:---------|
| 📋 **Clipboard** | Gerenciador de clipboard | Histórico, criptografia, Super+V |
| 📸 **Screenshot** | Captura de tela | Região, janela, editor, blur |
| 🎥 **Screencast** | Gravador de tela | H.264/VP9/AV1, áudio, GIF |
| 🎙️ **Recorder** | Gravador de voz | WAV/MP3/OGG/FLAC, waveform |
| 📷 **Camera** | Câmera | Foto, vídeo, filtros, PipeWire |
| 📄 **Documents** | Visualizador PDF | PDF, EPUB, DjVu, anotações |
| 📦 **Archive** | Gerenciador de arquivos | ZIP, RAR, 7z, TAR, ISO |
| 🔤 **Fonts** | Gerenciador de fontes | Preview, instalar, comparar |

### Sistema

| App | Descrição | Features |
|:----|:----------|:---------|
| 🔋 **Power** | Gerenciador de energia | Bateria, perfis, TLP |
| 🌐 **Network** | Gerenciador de rede | WiFi, VPN, Hotspot, Proxy |
| 📶 **Bluetooth** | Gerenciador Bluetooth | Pareamento, transferência |
| 💾 **Disks** | Gerenciador de discos | Partições, SMART, benchmark |
| 👤 **Users** | Gerenciador de usuários | Contas, grupos, permissões |
| 🔥 **Firewall** | Gerenciador de firewall | UFW/FirewallD, regras |
| 🖨️ **Printers** | Gerenciador de impressoras | CUPS, scan, fila |
| 📋 **Logs** | Visualizador de logs | Journal, kernel, apps |
| 🔄 **Updater** | Atualizador de software | APT, Flatpak, Snap, firmware |
| 💾 **Backup** | Sistema de backup | Local, rsync, Restic, cloud |
| ♿ **Accessibility** | Acessibilidade | Screen reader, zoom, filtros |

### Desenvolvimento

| App | Descrição | Features |
|:----|:----------|:---------|
| 🛠️ **Dev Hub** | Central do desenvolvedor | Projetos, ambientes, containers |
| 🏗️ **Builder** | Build cross-platform | .exe, .deb, .dmg, .AppImage |
| 📱 **Mobile Studio** | IDE mobile | Android, iOS, Flutter, React Native |
| 🎮 **Gaming** | Launcher unificado | Steam, GOG, Epic, emuladores |

### v1.2 Blaze (Novos)

| App | Descrição | Features |
|:----|:----------|:---------|
| 📱 **Connect** | Integração smartphone | Notificações, SMS, arquivos, clipboard |
| ☁️ **Cloud** | Cloud sync | 6 providers, delta sync, criptografia |
| 🤖 **AI** | Assistente IA | GPT-4o, o1, código, vision, tradução |

### Desktop Components

| Componente | Descrição |
|:-----------|:----------|
| **Compositor** | Wayland compositor (Smithay) |
| **Panel** | Barra superior com widgets |
| **Shell** | Desktop shell e dock |
| **Launcher** | App launcher estilo Spotlight |
| **Notifications** | Daemon de notificações |
| **Control Center** | Quick settings estilo iOS |
| **Shell Plugins** | Sistema de plugins extensível |

---

## 🎨 7 Temas Premium

| Fluent | Nord | Dracula | Catppuccin |
|:------:|:----:|:-------:|:----------:|
| Glassmorphism | Arctic | Dark Vibrant | 4 Variantes |
| #00d4ff | #88c0d0 | #bd93f9 | #cba6f7 |

**+ Icons (105 SVGs) + Cursors (Multi-DPI) + Plymouth (Boot Animation)**

**Features dos Temas:**
- 80+ animações CSS a 60fps
- Glassmorphism com backdrop-filter
- Micro-interações (hover, active, focus)
- Dark e Light mode
- Accent colors customizáveis

---

## 📊 Estatísticas do Projeto

```
┌────────────────────────────────────────────────────────────────────────┐
│                      WINUX OS v1.2 BLAZE                                │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│   Arquivos Rust           │████████████████████████████│  790+        │
│   Linhas de Código        │████████████████████████████│  190,000+    │
│   Apps Nativos            │████████████████████████████│  48          │
│   Componentes Desktop     │████████████████████████████│  7           │
│   Temas                   │████████████████████████████│  7           │
│   Shell Plugins           │████████████████████████████│  6           │
│   Scripts Shell           │████████████████████████████│  28+         │
│   Ícones SVG              │████████████████████████████│  105         │
│   Cloud Providers         │████████████████████████████│  6           │
│   AI Models               │████████████████████████████│  2 (GPT-4o, o1)│
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 📥 Download

<div align="center">

### Winux OS v1.2 Blaze

| Edição | Tamanho | Link |
|:-------|:--------|:-----|
| **v1.2 Blaze** | ~3.5 GB | [Download](https://github.com/leonardo-matheus/winux/releases) |
| **Android App** | ~15 MB | [Winux Connect APK](mobile/winux-connect-android/) |

**Requisitos Mínimos:** CPU x86_64 SSE4.2, 4GB RAM, 30GB SSD, Vulkan 1.1

**Requisitos Recomendados:** Ryzen 5/i5 6+ cores, 16GB RAM, 100GB NVMe, RTX 3060/RX 6700 XT

</div>

---

## 🚀 Instalação

### USB Bootável

```bash
# Linux
sudo dd if=winux-1.2-blaze.iso of=/dev/sdX bs=4M status=progress

# Windows - Usar Rufus ou balenaEtcher
```

### Build from Source

```bash
git clone https://github.com/leonardo-matheus/winux.git
cd winux

# Dependências (Ubuntu/Debian)
sudo apt install build-essential libgtk-4-dev libadwaita-1-dev

# Build
cargo build --release --workspace

# Build ISO
sudo ./build/scripts/build-winux-iso.sh
```

### Configurar AI Assistant

```bash
# Defina sua API key do Azure OpenAI
export WINUX_AI_API_KEY="sua_api_key_aqui"
export WINUX_AI_ENDPOINT="https://seu-recurso.openai.azure.com"

# Ou crie ~/.config/winux/ai.toml
```

---

## 📚 Documentação

| Documento | Descrição |
|:----------|:----------|
| [**APPS.md**](docs/APPS.md) | Documentação completa dos 48 apps |
| [**DEVELOPER.md**](docs/DEVELOPER.md) | Guia para desenvolvedores |
| [**MOBILE.md**](docs/MOBILE.md) | Desenvolvimento mobile |
| [**CHANGELOG.md**](docs/CHANGELOG.md) | Histórico de mudanças |
| [**index.html**](docs/index.html) | Landing page interativa |

---

## 🗺️ Roadmap

### v1.0 Aurora (2026) ✅
- [x] 44 Apps nativos em Rust
- [x] 7 Temas premium
- [x] Compatibilidade Win/Mac/Linux
- [x] Mobile development
- [x] Gaming otimizado

### v1.2 Blaze (2026) ✅ **ATUAL**
- [x] Winux Connect (smartphone)
- [x] Winux Cloud (6 providers)
- [x] Winux AI (GPT-4o, o1)
- [x] Shell Plugins (6 built-in)
- [x] Android App

### v2.0 Cosmos (2027)
- [ ] ARM64 support
- [ ] Immutable OS mode
- [ ] Container GUI (Podman)
- [ ] Wayland HDR completo
- [ ] Winux Connect iOS

---

## 🤝 Comunidade

<div align="center">

[![Discord](https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/winux)
[![Reddit](https://img.shields.io/badge/Reddit-FF4500?style=for-the-badge&logo=reddit&logoColor=white)](https://reddit.com/r/winuxos)
[![Twitter](https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white)](https://twitter.com/winuxos)
[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/leonardo-matheus/winux)

</div>

---

## 📄 Licença

| Componente | Licença |
|:-----------|:--------|
| Sistema e Scripts | [GPL v3](LICENSE-GPL) |
| Aplicações Winux | [MIT](LICENSE-MIT) |
| Documentação | [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) |

---

<div align="center">

<img src="assets/branding/logo.png" alt="Winux" width="80"/>

**Winux OS v1.2 Blaze** - *O Melhor dos Três Mundos*

**2026** | Feito com ❤️ em Rust

⭐ Se você gostou do projeto, considere dar uma estrela!

</div>
