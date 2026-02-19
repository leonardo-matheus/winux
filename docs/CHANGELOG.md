# Changelog

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║   ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗     ██████╗███████╗                ║
║   ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝    ██╔═══██╚════██║                ║
║   ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝     ██║   ██║   ██╔╝                ║
║   ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗     ██║   ██║  ██╔╝                 ║
║   ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗    ╚██████╔╝  ██║                  ║
║    ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝     ╚═════╝   ╚═╝                  ║
║                                                                               ║
║                            CHANGELOG                                          ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

Todas as mudancas notaveis neste projeto serao documentadas neste arquivo.

O formato e baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Versionamento Semantico](https://semver.org/lang/pt-BR/).

---

## [1.0.1] - Aurora Developer - 2026-02-19

### Resumo

Atualizacao com foco em ferramentas de desenvolvimento, novos aplicativos nativos e melhor compatibilidade cross-platform.

### Adicionado

#### Novos Aplicativos Nativos (+5)

- **winux-about**: Aplicativo de informacoes do sistema
  - Informacoes detalhadas de hardware
  - Versao do sistema e kernel
  - Info de CPU, GPU, memoria
  - Desktop session e display server

- **winux-personalize**: Gerenciador de personalizacao
  - Tres modos de interface: Windows, Linux, Mac
  - Configuracao de temas (claro/escuro/auto)
  - Cores de destaque personalizaveis
  - Gerenciamento de wallpapers
  - Temas de icones

- **winux-env-manager**: Gerenciador de variaveis de ambiente
  - Configuracao por linguagem (Node, Java, Python, Rust, Go, PHP)
  - Gerenciamento visual do PATH
  - Perfis de ambiente por projeto
  - Integracao com shell

- **winux-dev-hub**: Central do desenvolvedor
  - Dashboard de projetos com auto-deteccao
  - Gerenciamento de toolchains
  - Controle de containers (Docker/Podman)
  - Gerenciamento de databases locais
  - Controle de servicos do sistema

- **winux-builder**: Build cross-platform
  - Build para Windows (.exe, .msi)
  - Build para Linux (.deb, .rpm, .AppImage, .flatpak)
  - Build para macOS (.app, .dmg, .pkg)
  - Perfis de build salvos
  - Terminal de output integrado

#### Compatibilidade de Arquivos Multi-Plataforma

- **Suporte a arquivos Windows**:
  - `.exe` - Execucao via Wine, visualizacao de info PE
  - `.msi` - Instalacao ou extracao
  - `.dll` - Visualizacao de informacoes
  - `.lnk` - Resolucao de atalhos Windows
  - `.reg` - Importacao de arquivos de registro
  - `.bat`, `.ps1` - Visualizacao e edicao de scripts

- **Suporte a arquivos macOS**:
  - `.dmg` - Montagem e extracao de disk images
  - `.app` - Navegacao de application bundles
  - `.pkg` - Visualizacao e extracao de pacotes
  - `.plist` - Edicao de property lists
  - `.icns` - Visualizacao de icones
  - `.dylib` - Informacoes de bibliotecas

- **Suporte a arquivos Linux**:
  - `.deb` - Instalacao nativa ou extracao
  - `.rpm` - Instalacao via alien ou extracao
  - `.AppImage` - Execucao direta
  - `.flatpak` - Instalacao via Flatpak
  - `.snap` - Instalacao via Snap
  - `.so` - Informacoes de bibliotecas

#### Ambiente de Desenvolvimento Completo

- **Linguagens pre-configuradas**:
  - Rust 1.75+ com cargo, clippy, rustfmt, rust-analyzer
  - .NET 8 (C#, F#, VB.NET) com dotnet CLI
  - C/C++ com GCC 13+, Clang 17+, CMake 3.28+
  - Java 21 (OpenJDK) com Maven e Gradle
  - Python 3.12+ com pip, poetry, pipenv
  - Node.js 20+ via NVM com npm, yarn, pnpm
  - Go 1.22+ com gopls
  - Swift 5.9 (Linux) para server-side
  - PHP 8.3+ com Composer

- **Desenvolvimento Mobile**:
  - Android SDK completo com cmdline-tools
  - Android NDK para desenvolvimento nativo
  - Flutter SDK com hot reload
  - React Native toolchain
  - Emuladores Android com aceleracao KVM

- **Cross-Compilation**:
  - Targets para Windows (mingw-w64)
  - Targets para ARM64
  - osxcross para macOS (opcional)
  - Docker para builds isolados

#### Sistema de Drivers Atualizado

- Scripts de instalacao de drivers NVIDIA
  - Deteccao automatica de GPU
  - Instalacao de driver proprietario
  - Configuracao de Vulkan

- Scripts de configuracao AMD
  - Mesa RADV configurado
  - ROCm para compute (opcional)

#### Documentacao Expandida

- `docs/DEVELOPER.md` - Guia completo para desenvolvedores
- `docs/APPS.md` - Documentacao de todos os apps nativos
- `docs/MOBILE.md` - Desenvolvimento mobile (Android/iOS/Flutter)
- `docs/CHANGELOG.md` - Historico detalhado de mudancas

#### Winux Settings - Novas Paginas

- Pagina de Idioma e Regiao
  - Selecao de idioma do sistema
  - Formato regional (data, hora, numeros)
  - Layout de teclado
  - Metodo de entrada

### Modificado

- **README.md**: Atualizado com todas as novas features
  - Logo ASCII art do Winux
  - 12+ apps nativos documentados
  - Stack tecnologico atualizado
  - Diagramas ASCII de arquitetura

- **Cargo.toml workspace**: Adicionados novos apps ao workspace
  - winux-about
  - winux-personalize
  - winux-env-manager

- **winux-settings**: Expandido com mais configuracoes
  - Pagina de Performance com modos (Economico, Balanceado, Alto, Gaming)
  - Pagina de Energia com controles de bateria
  - Pagina de Idioma completa

- **winux-files**: Melhorado suporte a arquivos
  - Handler para arquivos Windows, macOS e Linux
  - Acoes contextuais por tipo de arquivo

### Corrigido

- Paths de assets nos apps de personalizacao
- Configuracao de ambiente de desenvolvimento
- Scripts de primeiro boot

---

## [1.0.0] - Aurora - 2026-02-18

### Resumo

Primeira versao estavel do Winux OS, uma distribuicao Linux focada em gaming e produtividade com interface inspirada no Windows 11.

### Adicionado

#### Sistema Base

- Base Ubuntu 24.04 LTS (Noble Numbat) para estabilidade
- Kernel Linux Zen 6.8+ com patches de performance
- Otimizacoes de sysctl para gaming e baixa latencia
- Suporte a UEFI e Secure Boot
- Instalador Calamares customizado

#### Ambiente de Desktop

- **winux-compositor**: Compositor Wayland baseado em Smithay
  - Suporte completo a Wayland e XWayland
  - Renderizacao via Vulkan
  - Multi-monitor com diferentes refresh rates
  - HDR support (experimental)
  - VRR/FreeSync/G-Sync support

- **winux-shell**: Shell de desktop moderno
  - Interface Fluent Design inspirada no Windows 11
  - Menu Iniciar com pesquisa integrada
  - Workspaces virtuais
  - Overview mode com visao de todas as janelas
  - Suporte a gestos de touchpad

- **winux-panel**: Barra de tarefas
  - Taskbar com preview de janelas
  - System tray completo
  - Relogio com calendario
  - Quick Settings (Wi-Fi, Bluetooth, Volume, Brilho)
  - Notificacoes integradas

#### Aplicacoes Nativas (8 Apps)

- **winux-files**: Gerenciador de arquivos
  - Navegacao em abas
  - Preview de arquivos
  - Operacoes em lote
  - Integracao com cloud storage
  - Compactacao/descompactacao integrada

- **winux-terminal**: Emulador de terminal
  - Multiplas abas e paineis divididos
  - Temas customizaveis
  - Suporte a transparencia/acrilico
  - Perfis de shell configuraveis
  - Integracao com bash, zsh, fish

- **winux-settings**: Central de configuracoes
  - Sistema (info, atualizacoes, armazenamento)
  - Rede (Wi-Fi, Ethernet, VPN)
  - Personalizacao (temas, wallpapers, fontes)
  - Aplicativos (padroes, inicializacao)
  - Contas (usuarios, sincronizacao)
  - Privacidade (permissoes, historico)
  - Gaming (performance, compatibilidade)

- **winux-store**: Loja de aplicativos
  - Suporte a APT, Flatpak, Snap
  - Avaliacoes e reviews
  - Atualizacoes automaticas
  - Categorias organizadas

- **winux-monitor**: Monitor de sistema
  - Visualizacao de processos
  - Graficos de CPU, RAM, Disco, Rede
  - Historico de uso
  - Gerenciamento de servicos

- **winux-edit**: Editor de texto
  - Destaque de sintaxe para 100+ linguagens
  - Numeracao de linhas
  - Localizacao e substituicao com regex
  - Multiplos encodings
  - Modo escuro

- **winux-image**: Visualizador de imagens
  - Suporte a PNG, JPG, GIF, WebP, SVG
  - Zoom e rotacao
  - Slideshow

- **winux-player**: Player multimidia
  - Suporte a video e audio
  - Playlists
  - Legendas

#### Gaming e Compatibilidade

- **Wine Staging** pre-configurado
  - DXVK para DirectX 9/10/11
  - VKD3D-Proton para DirectX 12
  - FAudio para audio
  - Visual C++ Runtimes
  - .NET Framework

- **Proton** via Steam
  - Proton-GE instalavel via script
  - Configuracao automatica de prefixos
  - Steam integrado e otimizado

- **winux-run**: Launcher de aplicativos Windows
  - Deteccao automatica de requisitos
  - Criacao de prefixos isolados
  - Perfis de compatibilidade

- **GameMode** integrado
  - Otimizacoes automaticas durante jogos
  - Ajuste de governor da CPU
  - Prioridade de processos

- **MangoHud** pre-configurado
  - Overlay de FPS e performance
  - Monitoramento de temperaturas
  - Frame timing graphs

#### Drivers

- **NVIDIA**: Drivers proprietarios 550+
  - Suporte a CUDA e NVENC
  - Vulkan ray tracing
  - DLSS support

- **AMD**: Mesa 24+ com RADV
  - Vulkan nativo
  - Ray tracing (RDNA2+)
  - Encoder/decoder de video

- **Intel**: Drivers Mesa
  - Arc GPUs suportadas
  - Quick Sync Video

#### Audio

- **PipeWire** como servidor de audio padrao
  - Baixa latencia para gaming
  - Compatibilidade com PulseAudio e JACK
  - Bluetooth audio (aptX, LDAC)

#### Sistema de Build

- Scripts de build da ISO automatizados
- Gerador de pacotes .deb
- CI/CD com GitHub Actions
- Testes automatizados

#### Seguranca

- AppArmor habilitado por padrao
- Firejail para aplicativos sensiveis
- Atualizacoes automaticas de seguranca
- Secure Boot support

---

## Versoes Futuras Planejadas

### [1.1.0] - Blaze (Q2 2026)

- [ ] Winux Connect - integracao com smartphone
- [ ] Cloud sync nativo
- [ ] App de Backup integrado
- [ ] Plugins para shell
- [ ] Melhorias no compositor

### [1.2.0] - Cascade (Q3 2026)

- [ ] Android app support (Waydroid)
- [ ] Containerizacao melhorada
- [ ] Performance improvements
- [ ] iOS development tools expandidos

### [2.0.0] - Dawn (Q4 2026)

- [ ] AI assistant integrado
- [ ] Container GUI (Podman)
- [ ] ARM64 support
- [ ] Immutable OS mode
- [ ] Novo sistema de pacotes
- [ ] Rollback de sistema

---

## Estatisticas

### Apps Nativos por Versao

| Versao | Apps | Novos |
|:-------|:----:|:-----:|
| 1.0.0 | 8 | 8 |
| 1.0.1 | 13 | 5 |

### Linhas de Codigo (Rust)

```
Versao 1.0.1
├── Apps            ~25,000 LOC
├── Desktop         ~15,000 LOC
├── Total           ~40,000 LOC
└── Testes          ~3,000 LOC
```

---

## Contribuidores

- Equipe Winux OS
- Comunidade de contribuidores
- Testers beta

---

## Links

| Recurso | URL |
|:--------|:----|
| **GitHub** | https://github.com/winux-os/winux |
| **Website** | https://winux-os.org |
| **Forum** | https://forum.winux-os.org |
| **Discord** | https://discord.gg/winux |
| **Download** | https://gofile.io/d/Y351PH |

---

**Winux OS Project - 2026**

*"O Melhor dos Tres Mundos"*
