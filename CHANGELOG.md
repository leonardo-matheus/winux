# Changelog

Todas as mudancas notaveis neste projeto serao documentadas neste arquivo.

O formato e baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Versionamento Semantico](https://semver.org/lang/pt-BR/).

> Para o changelog completo e detalhado, veja [docs/CHANGELOG.md](docs/CHANGELOG.md)

---

## [1.0.1] - Aurora Developer - 2026-02-19

### Adicionado

- 5 novos aplicativos nativos:
  - **winux-about**: Informacoes do sistema
  - **winux-personalize**: Personalizacao (modos Win/Mac/Linux)
  - **winux-env-manager**: Variaveis de ambiente
  - **winux-dev-hub**: Central do desenvolvedor
  - **winux-builder**: Build cross-platform

- Compatibilidade com arquivos de todas as plataformas:
  - Windows: .exe, .msi, .dll, .lnk, .reg, .bat, .ps1
  - macOS: .dmg, .app, .pkg, .plist, .icns, .dylib
  - Linux: .deb, .rpm, .AppImage, .flatpak, .snap

- Ambiente de desenvolvimento completo:
  - Rust, .NET, C/C++, Java, Python, Node.js, Go, Swift, PHP
  - Android SDK + NDK
  - Flutter SDK
  - React Native

- Documentacao expandida:
  - `docs/DEVELOPER.md` - Guia do desenvolvedor
  - `docs/APPS.md` - Documentacao dos apps
  - `docs/MOBILE.md` - Desenvolvimento mobile
  - `docs/CHANGELOG.md` - Changelog detalhado

### Modificado

- README.md atualizado com todas as features
- winux-settings com paginas de Idioma e Performance
- winux-files com handlers multi-plataforma

---

## [1.0.0] - Aurora - 2026-02-18

### Adicionado

- Sistema base Ubuntu 24.04 LTS
- Kernel Linux Zen 6.8+
- Desktop Environment (Compositor, Shell, Panel)
- 8 aplicativos nativos em Rust
- Gaming stack (Wine, Proton, DXVK, GameMode)
- Drivers NVIDIA/AMD/Intel
- PipeWire audio
- Sistema de build ISO

### Seguranca

- AppArmor habilitado
- Secure Boot support
- Atualizacoes automaticas

---

## Versoes Futuras

- **1.1.0 Blaze** (Q2 2026): Cloud sync, Backup, Plugins
- **1.2.0 Cascade** (Q3 2026): Waydroid, Performance
- **2.0.0 Dawn** (Q4 2026): AI Assistant, ARM64, Immutable mode

---

**Winux OS Project - 2026**
