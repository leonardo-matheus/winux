# Winux OS - Planejamento de Sprints

## Visão Geral

O desenvolvimento do Winux OS está organizado em 18 sprints, agrupados em 9 releases incrementais.

## Sprint Backlog

### 🏗️ Sprint 01-02: Infraestrutura
**Objetivo:** Estrutura base do projeto e sistema de build

#### Sprint 01
- [x] Inicializar repositório Git
- [x] Criar estrutura de diretórios
- [x] Configurar Cargo workspace
- [x] Criar README e documentação base
- [ ] Configurar CI/CD básico

#### Sprint 02
- [ ] Script de build da ISO (estrutura)
- [ ] Configuração do Calamares (base)
- [ ] Makefile principal
- [ ] Scripts de utilidade

---

### 🔧 Sprint 03-04: Kernel e Drivers
**Objetivo:** Kernel customizado e detecção de hardware

#### Sprint 03
- [ ] Configuração do kernel zen
- [ ] Patches de performance
- [ ] Script de build do kernel
- [ ] Pacote .deb do kernel

#### Sprint 04
- [ ] Script de detecção de GPU
- [ ] Instalador NVIDIA
- [ ] Instalador AMD/Mesa
- [ ] Instalador Intel
- [ ] winux-driver-manager

---

### 🖥️ Sprint 05-06: Compositor e Desktop Base
**Objetivo:** Compositor Wayland funcional

#### Sprint 05
- [ ] Estrutura do compositor (smithay)
- [ ] Backend DRM
- [ ] Renderização básica
- [ ] Suporte a input

#### Sprint 06
- [ ] Window management
- [ ] XWayland support
- [ ] Multi-monitor
- [ ] Configuração de teclado/mouse

---

### 🎨 Sprint 07-08: Shell Components
**Objetivo:** Taskbar, Menu Iniciar, System Tray

#### Sprint 07
- [ ] Winux Panel (estrutura)
- [ ] Taskbar widget
- [ ] App launcher
- [ ] Window list

#### Sprint 08
- [ ] Menu Iniciar
- [ ] System tray
- [ ] Clock widget
- [ ] Quick settings

---

### 📁 Sprint 09-10: Core Apps (Parte 1)
**Objetivo:** Files, Terminal, Settings

#### Sprint 09
- [ ] Winux Files (navegação)
- [ ] Winux Files (operações)
- [ ] Winux Terminal (básico)

#### Sprint 10
- [ ] Winux Terminal (tabs, themes)
- [ ] Winux Settings (estrutura)
- [ ] Winux Settings (módulos)

---

### 🛒 Sprint 11-12: Core Apps (Parte 2)
**Objetivo:** Store, Monitor, Edit

#### Sprint 11
- [ ] Winux Store (backend)
- [ ] Winux Store (UI)
- [ ] Winux Monitor (processos)

#### Sprint 12
- [ ] Winux Monitor (performance)
- [ ] Winux Edit (editor básico)
- [ ] Winux Edit (syntax highlighting)

---

### 🎮 Sprint 13-14: Compatibilidade Windows
**Objetivo:** Wine, Proton, Gaming

#### Sprint 13
- [ ] Configuração Wine
- [ ] Prefix management
- [ ] DXVK/VKD3D setup
- [ ] winux-run launcher

#### Sprint 14
- [ ] Steam integration
- [ ] Proton-GE installer
- [ ] GameMode config
- [ ] MangoHud config
- [ ] Testes de jogos

---

### 📦 Sprint 15-16: Build System e Installer
**Objetivo:** ISO bootável com instalador

#### Sprint 15
- [ ] Script de build completo
- [ ] Squashfs creation
- [ ] GRUB/systemd-boot config
- [ ] ISO generation

#### Sprint 16
- [ ] Calamares branding
- [ ] Slideshow de instalação
- [ ] Post-install scripts
- [ ] First-run wizard

---

### ✨ Sprint 17-18: Polish e Release
**Objetivo:** QA final e release 1.0

#### Sprint 17
- [ ] Testes em hardware real
- [ ] Bug fixing
- [ ] Performance tuning
- [ ] Security review

#### Sprint 18
- [ ] Documentação final
- [ ] Website/landing page
- [ ] Release notes
- [ ] Build ISO final
- [ ] Mirrors setup

---

## Métricas de Progresso

| Sprint | Status | Progresso |
|--------|--------|-----------|
| 01-02 | 🔄 Em andamento | 60% |
| 03-04 | ⏳ Pendente | 0% |
| 05-06 | ⏳ Pendente | 0% |
| 07-08 | ⏳ Pendente | 0% |
| 09-10 | ⏳ Pendente | 0% |
| 11-12 | ⏳ Pendente | 0% |
| 13-14 | ⏳ Pendente | 0% |
| 15-16 | ⏳ Pendente | 0% |
| 17-18 | ⏳ Pendente | 0% |

---

**Última atualização:** Fevereiro 2026
