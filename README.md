# 🐧 Winux OS

> **"O Melhor dos Dois Mundos"**
> Gaming + Produtividade | Linux + Windows Experience

```
██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗
██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝
██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝
██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗
╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗
 ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝
```

## Sobre

Winux OS é uma distribuição Linux baseada no Ubuntu 24.04 LTS, projetada para oferecer:

- **Interface Familiar**: Desktop inspirado no Windows 11 com Fluent Design
- **Gaming de Alto Desempenho**: Wine, Proton, DXVK integrados nativamente
- **Aplicações Nativas em Rust**: Suite completa de apps modernos e performáticos
- **Otimizações Agressivas**: Kernel zen com tunables para máxima performance

## Versão

- **Versão**: 1.0 Aurora
- **Base**: Ubuntu 24.04 LTS (Noble Numbat)
- **Licença**: GPL v3 + MIT (aplicações próprias)

## Requisitos

### Mínimos
- CPU: x86_64 com suporte SSE4.2
- RAM: 4 GB
- Armazenamento: 30 GB (SSD recomendado)
- GPU: Vulkan 1.1 compatível

### Recomendados
- CPU: AMD Ryzen 5 / Intel Core i5 (6+ cores)
- RAM: 16 GB DDR4/DDR5
- Armazenamento: 100 GB NVMe SSD
- GPU: NVIDIA RTX 3060 / AMD RX 6700 XT ou superior

## Estrutura do Projeto

```
winux/
├── apps/                   # Aplicações nativas Rust
│   ├── winux-files/
│   ├── winux-terminal/
│   ├── winux-settings/
│   ├── winux-store/
│   ├── winux-monitor/
│   └── winux-edit/
├── build/                  # Sistema de build
├── compatibility/          # Wine/Proton integration
├── desktop/                # Desktop Environment
│   ├── winux-compositor/
│   ├── winux-panel/
│   └── winux-shell/
├── docs/                   # Documentação
├── drivers/                # Scripts de drivers
├── kernel/                 # Kernel customizado
├── system/                 # Configurações de sistema
└── themes/                 # Temas visuais
```

## Sprints de Desenvolvimento

| Sprint | Foco | Status |
|--------|------|--------|
| 01-02 | Infraestrutura e Build Base | 🔄 Em andamento |
| 03-04 | Kernel e Drivers | ⏳ Pendente |
| 05-06 | Compositor e Desktop Base | ⏳ Pendente |
| 07-08 | Shell Components | ⏳ Pendente |
| 09-10 | Core Apps (Files, Terminal, Settings) | ⏳ Pendente |
| 11-12 | Core Apps (Store, Monitor, Edit) | ⏳ Pendente |
| 13-14 | Compatibilidade Windows | ⏳ Pendente |
| 15-16 | Build System e Installer | ⏳ Pendente |
| 17-18 | Polish e Release | ⏳ Pendente |

## Branches

- `main` - Releases estáveis
- `develop` - Integração de desenvolvimento
- `sprint/XX-nome` - Branches de sprint

## Build

```bash
# Clonar repositório
git clone https://github.com/winux-os/winux.git
cd winux

# Build da ISO (requer Ubuntu/Debian)
sudo ./build/scripts/build-winux-iso.sh
```

## Contribuindo

Veja [CONTRIBUTING.md](CONTRIBUTING.md) para guidelines de contribuição.

## Licença

- Sistema e scripts: GPL v3
- Aplicações Winux: MIT

---

**Winux OS Project - 2026**
