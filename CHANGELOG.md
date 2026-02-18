# Changelog

Todas as mudancas notaveis neste projeto serao documentadas neste arquivo.

O formato e baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Versionamento Semantico](https://semver.org/lang/pt-BR/).

---

## [1.0.0] - Aurora - 2026-02-18

### Resumo

Primeira versao estavel do Winux OS, uma distribuicao Linux focada em gaming e produtividade
com interface inspirada no Windows 11.

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

#### Aplicacoes Nativas
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
  - Perfis de shell configuráveis
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

#### Documentacao
- Guia do Usuario completo
- Guia do Desenvolvedor
- Documentacao de arquitetura
- FAQ extensivo

### Mudado

- N/A (primeira versao)

### Removido

- N/A (primeira versao)

### Corrigido

- N/A (primeira versao)

### Seguranca

- AppArmor habilitado por padrao
- Firejail para aplicativos sensiveis
- Atualizacoes automaticas de seguranca
- Secure Boot support

---

## Versoes Futuras Planejadas

### [1.1.0] - Borealis (Q2 2026)
- Melhorias no compositor
- Novos efeitos visuais
- Suporte a plugins no shell
- Sincronizacao de configuracoes

### [1.2.0] - Cascade (Q3 2026)
- Android app support (Waydroid)
- Containerizacao melhorada
- Performance improvements
- Novos aplicativos nativos

### [2.0.0] - Dawn (Q4 2026)
- Novo sistema de pacotes
- Instalacao imutavel (opcional)
- Rollback de sistema
- AI assistant integrado

---

## Contribuidores

- Equipe Winux OS
- Comunidade de contribuidores
- Testers beta

---

## Links

- **GitHub**: https://github.com/winux-os/winux
- **Website**: https://winux-os.org
- **Forum**: https://forum.winux-os.org
- **Discord**: https://discord.gg/winux

---

**Winux OS Project - 2026**
*O Melhor dos Dois Mundos*
