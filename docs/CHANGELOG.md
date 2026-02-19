# Changelog

```
+==============================================================================+
|                                                                              |
|   ██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗    ██████╗ ██╗      █████╗        |
|   ██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝    ██╔══██╗██║     ██╔══██╗       |
|   ██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝     ██████╔╝██║     ███████║       |
|   ██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗     ██╔══██╗██║     ██╔══██║       |
|   ╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗    ██████╔╝███████╗██║  ██║       |
|    ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝    ╚═════╝ ╚══════╝╚═╝  ╚═╝       |
|                                                                              |
|                      CHANGELOG v1.2 BLAZE                                    |
+==============================================================================+
```

Todas as mudancas notaveis neste projeto serao documentadas neste arquivo.

O formato e baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Versionamento Semantico](https://semver.org/lang/pt-BR/).

---

## [1.2.0] - Blaze - 2026-02-19

### Resumo

**Blaze** e a maior atualizacao do Winux OS ate agora! Inclui AI Assistant integrado com GPT-4o,
sincronizacao na nuvem com 6 provedores, integracao com smartphone via Winux Connect, sistema
de plugins para o shell e 35 novos aplicativos nativos.

### Novidades Principais

#### AI Assistant - Inteligencia Artificial Integrada

- **winux-ai**: Aplicativo desktop de IA com interface moderna
  - Chat com GPT-4o e o1 (Azure OpenAI)
  - Streaming de respostas em tempo real
  - Analise de imagens com Vision API
  - Assistente de codigo com syntax highlighting
  - Historico de conversas persistente
  - Atalho global: `Super+Space`
  - Interface com temas claro/escuro
  - Animacoes fluidas e design Winux

- **winux-ai-service**: D-Bus daemon para AI do sistema
  - Servico em background acessivel por qualquer app
  - Cache de respostas para economia de tokens
  - Rate limiting respeitando limites da Azure
  - Configuracao via `/etc/winux/ai-service.toml`

#### Winux Connect - Integracao com Smartphone

- **winux-connect**: App desktop para conexao com smartphone
  - Descoberta automatica via mDNS/DNS-SD
  - Pareamento por QR Code
  - Notificacoes do telefone no desktop
  - Envio e recebimento de SMS
  - Transferencia de arquivos (drag & drop, ate 50MB/s)
  - Clipboard compartilhado
  - Controle de midia remoto
  - Espelhamento de tela (screen mirror)
  - Localizacao do telefone (ring)
  - Compativel com protocolo KDE Connect

- **winux-connect-android**: App Android completo
  - Kotlin + Jetpack Compose
  - Material 3 com tema dinamico
  - Descoberta automatica do PC
  - Pareamento seguro com criptografia
  - Notificacao de dispositivo pareado
  - Transferencia de arquivos em background
  - Suporte a Android 8.0+

#### Cloud Sync - Sincronizacao na Nuvem

- **winux-cloud**: Aplicativo de sincronizacao de arquivos
  - 6 provedores suportados:
    - Google Drive (OAuth2)
    - Microsoft OneDrive (OAuth2)
    - Dropbox (OAuth2)
    - Amazon S3
    - Azure Blob Storage
    - WebDAV (qualquer servidor)
  - Sincronizacao em background com inotify
  - Deteccao de conflitos com 3-way merge
  - Criptografia client-side opcional (AES-256-GCM)
  - Bandeja do sistema com status
  - Configuracao de pastas por provedor
  - Historico de sincronizacao

#### Shell Plugins - Extensibilidade

- **winux-shell-plugins**: Sistema de plugins para o shell
  - API de plugins em Rust
  - Suporte a plugins em Lua (mlua)
  - Hot reload sem reiniciar o shell
  - Sandboxing com seccomp
  - Plugin store integrada
  - Ciclo de vida completo (init, update, shutdown)
  - Widgets, indicadores e acoes

- **6 Plugins Inclusos**:
  - `weather-widget`: Clima atual e previsao
  - `system-monitor-widget`: CPU, RAM, rede em tempo real
  - `clipboard-indicator`: Historico de clipboard
  - `pomodoro`: Timer de produtividade
  - `caffeine`: Impedir suspensao
  - `music-controls`: Controle MPRIS

### Novos Aplicativos (+35)

#### Utilitarios de Sistema
- **winux-welcome**: Assistente de primeiro uso
- **winux-launcher**: Lancador tipo Spotlight (Alt+Space)
- **winux-notifications**: Daemon de notificacoes
- **winux-control-center**: Quick Settings expandido
- **winux-screenshot**: Captura de tela com anotacoes
- **winux-screencast**: Gravacao de tela
- **winux-recorder**: Gravador de voz
- **winux-clipboard**: Gerenciador de clipboard

#### Gerenciamento de Sistema
- **winux-network**: Gerenciador de rede avancado
- **winux-bluetooth**: Gerenciador Bluetooth
- **winux-disks**: Gerenciador de discos/particoes
- **winux-users**: Gerenciador de usuarios
- **winux-updater**: Atualizador do sistema
- **winux-power**: Gerenciador de energia
- **winux-firewall**: Gerenciador de firewall
- **winux-printers**: Gerenciador de impressoras
- **winux-accessibility**: Configuracoes de acessibilidade
- **winux-backup**: Backup e restauracao
- **winux-logs**: Visualizador de logs do sistema

#### Produtividade
- **winux-calendar**: Calendario e tarefas
- **winux-notes**: Notas estilo Google Keep
- **winux-contacts**: Gerenciador de contatos
- **winux-mail**: Cliente de email (IMAP/SMTP)
- **winux-weather**: Previsao do tempo
- **winux-calculator**: Calculadora cientifica
- **winux-clock**: Relogio, alarmes, timers, cronometro

#### Multimidia
- **winux-camera**: Aplicativo de camera
- **winux-fonts**: Gerenciador de fontes

#### Documentos
- **winux-documents**: Visualizador de PDF
- **winux-archive**: Gerenciador de arquivos compactados

#### Gaming
- **winux-gaming**: Launcher de games integrado

#### Desenvolvimento
- **winux-mobile-studio**: IDE para desenvolvimento mobile
  - Android (Kotlin/Java)
  - iOS (Swift)
  - Flutter
  - React Native
  - Emuladores integrados

### Temas e Visual

#### Novos Temas (+4)
- **Winux Dark** (padrao): Tema escuro elegante
- **Winux Light**: Tema claro minimalista
- **Midnight Blue**: Azul escuro profissional
- **Forest Green**: Verde natureza

#### Tema de Icones
- **Winux Icons**: 500+ icones vetoriais
  - Estilo Fluent Design
  - Cores Winux (cyan, magenta, green)
  - Todas as categorias de apps

#### Tema de Cursor
- **Winux Cursors**: Cursores modernos
  - 12 estados animados
  - Suporte a HiDPI (24px, 32px, 48px)
  - Cores coordenadas com o tema

### Melhorias

- **Desktop Environment**:
  - Performance do compositor melhorada
  - Animacoes mais suaves (60fps)
  - Suporte melhorado a multi-monitor
  - Novo overview de workspaces

- **Sistema**:
  - Scripts de otimizacao de performance
  - Melhor deteccao de hardware
  - Instalacao de drivers simplificada
  - Boot mais rapido

### Estatisticas da Versao

| Metrica | Valor |
|:--------|:------|
| **Apps Nativos** | 48 |
| **Linhas de Codigo** | 190,000+ |
| **Arquivos Rust** | 790+ |
| **Temas** | 7 |
| **Icones** | 500+ |
| **Plugins** | 6 |
| **Provedores Cloud** | 6 |

### Requisitos de Sistema

| Componente | Minimo | Recomendado |
|:-----------|:-------|:------------|
| **CPU** | x86_64 SSE4.2 | 4+ cores |
| **RAM** | 4 GB | 16 GB |
| **Disco** | 40 GB | 100 GB SSD |
| **GPU** | Vulkan 1.1 | RTX/RX 6000+ |

---

## [1.0.1] - Aurora Developer - 2026-02-19

### Resumo

Atualizacao com foco em ferramentas de desenvolvimento, novos aplicativos nativos e melhor compatibilidade cross-platform.

### Adicionado

#### Novos Aplicativos Nativos (+5)

- **winux-about**: Aplicativo de informacoes do sistema
- **winux-personalize**: Gerenciador de personalizacao
- **winux-env-manager**: Gerenciador de variaveis de ambiente
- **winux-dev-hub**: Central do desenvolvedor
- **winux-builder**: Build cross-platform

#### Ambiente de Desenvolvimento

- Rust 1.75+, .NET 8, C/C++ (GCC 13+/Clang 17+)
- Java 21, Python 3.12+, Node.js 20+, Go 1.22+
- Android SDK, Flutter, React Native
- Cross-compilation para Windows, Linux, macOS, ARM64

---

## [1.0.0] - Aurora - 2026-02-18

### Resumo

Primeira versao estavel do Winux OS - distribuicao Linux focada em gaming e produtividade.

### Destaques

- Base Ubuntu 24.04 LTS com kernel Zen 6.8+
- Compositor Wayland (Smithay) com Vulkan
- 8 apps nativos em Rust/GTK4
- Gaming otimizado com Wine, Proton, DXVK
- Drivers NVIDIA/AMD/Intel pre-configurados
- PipeWire para audio de baixa latencia

---

## Versoes Futuras

### [1.3.0] - Cascade (Q3 2026)
- Android app support (Waydroid)
- Containerizacao melhorada
- iOS development tools expandidos

### [2.0.0] - Dawn (Q4 2026)
- ARM64 support
- Immutable OS mode
- Novo sistema de pacotes com rollback

---

## Links

| Recurso | URL |
|:--------|:----|
| **GitHub** | https://github.com/winux-os/winux |
| **Website** | https://winux.dev |
| **Discord** | https://discord.gg/winux |
| **Download** | https://gofile.io/d/winux-blaze |

---

**Winux OS v1.2 Blaze - 2026**

*"AI-Powered Linux for Gamers & Developers"*
