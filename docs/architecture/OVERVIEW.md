# Winux OS - Visao Geral da Arquitetura

> Documentacao tecnica da arquitetura do sistema

---

## Diagrama de Componentes

```
+==============================================================================+
|                              WINUX OS 1.0 AURORA                             |
+==============================================================================+

                            +------------------------+
                            |     USER INTERFACE     |
                            +------------------------+
                                       |
    +------------------------------------------------------------------+
    |                        WINUX APPLICATIONS                        |
    +------------------------------------------------------------------+
    |  +----------+  +----------+  +----------+  +----------+          |
    |  |  Files   |  | Terminal |  | Settings |  |  Store   |          |
    |  | (Rust)   |  |  (Rust)  |  |  (Rust)  |  |  (Rust)  |          |
    |  +----------+  +----------+  +----------+  +----------+          |
    |                                                                  |
    |  +----------+  +----------+  +-----------------------------------+
    |  | Monitor  |  |   Edit   |  |  Third-Party Apps (Flatpak/APT)  |
    |  | (Rust)   |  |  (Rust)  |  |                                  |
    |  +----------+  +----------+  +-----------------------------------+
    +------------------------------------------------------------------+
                                       |
                            +----------+----------+
                            |                     |
    +------------------------------------------------------------------+
    |                        WINUX DESKTOP SHELL                       |
    +------------------------------------------------------------------+
    |  +-----------------+  +-----------------+  +-----------------+   |
    |  |  winux-shell    |  |  winux-panel    |  | Notification    |   |
    |  |  (Rust/GTK4)    |  |  (Rust/GTK4)    |  | Daemon          |   |
    |  |                 |  |                 |  |                 |   |
    |  | - Window Mgmt   |  | - Taskbar       |  | - D-Bus Service |   |
    |  | - Workspaces    |  | - System Tray   |  | - Toast Popups  |   |
    |  | - App Launcher  |  | - Clock         |  | - Action Center |   |
    |  | - Overview      |  | - Quick Settings|  |                 |   |
    |  +-----------------+  +-----------------+  +-----------------+   |
    +------------------------------------------------------------------+
                                       |
    +------------------------------------------------------------------+
    |                      WINUX COMPOSITOR                            |
    +------------------------------------------------------------------+
    |                        (Smithay - Rust)                          |
    |  +-----------------------------------------------------------+  |
    |  |  +-------------+  +-------------+  +-------------+        |  |
    |  |  |   Wayland   |  |  XWayland   |  |   Input     |        |  |
    |  |  |   Backend   |  |   Bridge    |  |   Handler   |        |  |
    |  |  +-------------+  +-------------+  +-------------+        |  |
    |  |                                                           |  |
    |  |  +-------------+  +-------------+  +-------------+        |  |
    |  |  |    DRM      |  |   Vulkan    |  |   Multi-    |        |  |
    |  |  |   Backend   |  |  Renderer   |  |   Monitor   |        |  |
    |  |  +-------------+  +-------------+  +-------------+        |  |
    |  +-----------------------------------------------------------+  |
    +------------------------------------------------------------------+
                                       |
    +------------------------------------------------------------------+
    |                    COMPATIBILITY LAYER                           |
    +------------------------------------------------------------------+
    |  +-------------------+  +-------------------+  +---------------+ |
    |  |       WINE        |  |      PROTON       |  |   winux-run   | |
    |  |                   |  |                   |  |   (Launcher)  | |
    |  | - Wine Staging    |  | - Proton-GE       |  |               | |
    |  | - DXVK            |  | - Valve Proton    |  | - Auto-config | |
    |  | - VKD3D-Proton    |  | - Steam Runtime   |  | - Prefix Mgmt | |
    |  | - FAudio          |  |                   |  | - Game Mode   | |
    |  +-------------------+  +-------------------+  +---------------+ |
    +------------------------------------------------------------------+
                                       |
    +------------------------------------------------------------------+
    |                      SYSTEM SERVICES                             |
    +------------------------------------------------------------------+
    |  +-----------+  +-----------+  +-----------+  +-----------+     |
    |  | PipeWire  |  | NetworkMgr|  | systemd   |  | D-Bus     |     |
    |  | (Audio)   |  | (Network) |  | (Init)    |  | (IPC)     |     |
    |  +-----------+  +-----------+  +-----------+  +-----------+     |
    |                                                                  |
    |  +-----------+  +-----------+  +-----------+  +-----------+     |
    |  | udev      |  | PolicyKit |  | AccountsSvc| | UDisks2   |     |
    |  | (Devices) |  | (Auth)    |  | (Users)   |  | (Storage) |     |
    |  +-----------+  +-----------+  +-----------+  +-----------+     |
    +------------------------------------------------------------------+
                                       |
    +------------------------------------------------------------------+
    |                         LINUX KERNEL                             |
    +------------------------------------------------------------------+
    |                     (Zen Kernel + Patches)                       |
    |  +-----------------------------------------------------------+  |
    |  |  +-------------+  +-------------+  +-------------+        |  |
    |  |  |    DRM      |  |   AMDGPU    |  |   NVIDIA    |        |  |
    |  |  |  Subsystem  |  |   Driver    |  |   Driver    |        |  |
    |  |  +-------------+  +-------------+  +-------------+        |  |
    |  |                                                           |  |
    |  |  +-------------+  +-------------+  +-------------+        |  |
    |  |  |   Futex2    |  |  io_uring   |  |   NTFS3     |        |  |
    |  |  |  (Proton)   |  |  (Async I/O)|  |   Driver    |        |  |
    |  |  +-------------+  +-------------+  +-------------+        |  |
    |  +-----------------------------------------------------------+  |
    +------------------------------------------------------------------+
                                       |
    +------------------------------------------------------------------+
    |                          HARDWARE                                |
    +------------------------------------------------------------------+
    |   CPU (x86_64)  |  GPU (AMD/NVIDIA/Intel)  |  Storage  |  Input |
    +------------------------------------------------------------------+
```

---

## Stack Tecnologico

### Camada Base

| Componente | Tecnologia | Versao | Descricao |
|------------|------------|--------|-----------|
| Distro Base | Ubuntu LTS | 24.04 | Sistema base estavel |
| Kernel | Linux Zen | 6.8+ | Kernel otimizado para desktop/gaming |
| Init | systemd | 255+ | Sistema de inicializacao |
| Libc | glibc | 2.39+ | Biblioteca C padrao |

### Camada Grafica

| Componente | Tecnologia | Descricao |
|------------|------------|-----------|
| Display Server | Wayland | Protocolo de display moderno |
| Compositor | Smithay | Framework de compositor em Rust |
| Renderer | Vulkan/OpenGL | Renderizacao GPU acelerada |
| XWayland | XWayland | Compatibilidade X11 |

### Camada de Desktop

| Componente | Tecnologia | Descricao |
|------------|------------|-----------|
| Toolkit | GTK4 | Interface grafica |
| Widget Library | libadwaita | Componentes GNOME |
| Linguagem | Rust | Seguranca e performance |
| Temas | Fluent Design | Visual inspirado no Windows 11 |

### Camada de Audio

| Componente | Tecnologia | Descricao |
|------------|------------|-----------|
| Audio Server | PipeWire | Audio moderno de baixa latencia |
| Session Manager | WirePlumber | Gerenciamento de sessao |
| Compatibility | PulseAudio/JACK | Compat com apps legados |

### Camada de Gaming

| Componente | Tecnologia | Descricao |
|------------|------------|-----------|
| Wine | Wine Staging | Execucao de apps Windows |
| Proton | Proton-GE | Wine otimizado para Steam |
| DirectX 9-11 | DXVK | Traducao DX para Vulkan |
| DirectX 12 | VKD3D-Proton | Traducao DX12 para Vulkan |
| Performance | GameMode | Otimizacoes em tempo real |
| Monitoring | MangoHud | Overlay de performance |

---

## Fluxo de Dados

### Boot e Inicializacao

```
+----------+     +----------+     +----------+     +----------+
|  UEFI/   | --> |  GRUB/   | --> |  Linux   | --> | systemd  |
|  BIOS    |     | systemd- |     |  Kernel  |     |  init    |
|          |     |  boot    |     |          |     |          |
+----------+     +----------+     +----------+     +----------+
                                                        |
                                                        v
+------------------------------------------------------------------+
|                        SYSTEMD TARGETS                           |
+------------------------------------------------------------------+
|  basic.target --> multi-user.target --> graphical.target         |
+------------------------------------------------------------------+
                                                        |
                                                        v
                                  +------------------------+
                                  |   Display Manager      |
                                  |   (GDM/SDDM/custom)    |
                                  +------------------------+
                                                        |
                                            +-----------+-----------+
                                            |                       |
                                            v                       v
                                  +---------------+       +---------------+
                                  |  User Login   |       | Auto-login    |
                                  |    (PAM)      |       |  (optional)   |
                                  +---------------+       +---------------+
                                            |                       |
                                            +-----------+-----------+
                                                        |
                                                        v
                                  +------------------------+
                                  |   Winux Session        |
                                  +------------------------+
                                            |
                      +---------------------+---------------------+
                      |                     |                     |
                      v                     v                     v
            +---------------+     +---------------+     +---------------+
            |  winux-       |     |    D-Bus      |     |   PipeWire    |
            |  compositor   |     |   Session     |     |    Daemon     |
            +---------------+     +---------------+     +---------------+
                      |
                      v
            +---------------+
            |  winux-shell  |
            |  winux-panel  |
            +---------------+
```

### Fluxo de Renderizacao

```
+-------------------+
|   Application     |
| (GTK4/Qt/Custom)  |
+-------------------+
         |
         | Wayland Protocol
         v
+-------------------+
| winux-compositor  |
|                   |
|  +-------------+  |
|  | Scene Graph |  |
|  +-------------+  |
|         |         |
|         v         |
|  +-------------+  |
|  |   Damage    |  |
|  |  Tracking   |  |
|  +-------------+  |
|         |         |
|         v         |
|  +-------------+  |
|  |   Vulkan    |  |
|  |  Renderer   |  |
|  +-------------+  |
+-------------------+
         |
         | DRM/KMS
         v
+-------------------+
|       GPU         |
+-------------------+
         |
         v
+-------------------+
|     Display       |
+-------------------+
```

### Fluxo de Input

```
+-------------------+
|  Physical Input   |
| (Keyboard/Mouse/  |
|  Gamepad/Touch)   |
+-------------------+
         |
         | /dev/input/eventX
         v
+-------------------+
|     libinput      |
+-------------------+
         |
         v
+-------------------+
| winux-compositor  |
|                   |
|  +-------------+  |
|  |   Input     |  |
|  |   Handler   |  |
|  +-------------+  |
|         |         |
|  +------+------+  |
|  |             |  |
|  v             v  |
| Focus        Raw  |
| Follows      Input|
| Pointer      to   |
|             Game  |
+-------------------+
         |
         | Wayland Events
         v
+-------------------+
|   Application     |
+-------------------+
```

### Fluxo de Audio

```
+-------------------+
|   Application     |
| (Game/Music/etc)  |
+-------------------+
         |
         | PipeWire/PulseAudio API
         v
+-------------------+
|     PipeWire      |
|                   |
|  +-------------+  |
|  |   Graph     |  |
|  |   Engine    |  |
|  +-------------+  |
|         |         |
|  +------+------+  |
|  |             |  |
|  v             v  |
| ALSA         BT   |
| Sink        Sink  |
+-------------------+
         |
         v
+-------------------+
|   Audio Hardware  |
| (Speakers/Headset)|
+-------------------+
```

### Fluxo de Gaming (Wine/Proton)

```
+-------------------+
|   Windows Game    |
|     (.exe)        |
+-------------------+
         |
         | winux-run / Steam
         v
+-------------------+
|      Wine/        |
|      Proton       |
|                   |
|  +-------------+  |
|  | PE Loader   |  |
|  +-------------+  |
|         |         |
|  +------+------+  |
|  |             |  |
|  v             v  |
| Wine        Native|
| DLLs        .so   |
+-------------------+
         |
         | DirectX Calls
         v
+-------------------+     +-------------------+
|       DXVK        |     |   VKD3D-Proton    |
|   (DX9/10/11)     |     |      (DX12)       |
+-------------------+     +-------------------+
         |                         |
         +------------+------------+
                      |
                      | Vulkan API
                      v
+-------------------+
|   Mesa / NVIDIA   |
|      Driver       |
+-------------------+
         |
         v
+-------------------+
|       GPU         |
+-------------------+
```

---

## Comunicacao entre Componentes

### D-Bus Services

```
+------------------------------------------------------------------+
|                         D-BUS SYSTEM BUS                         |
+------------------------------------------------------------------+
|                                                                  |
|  org.freedesktop.NetworkManager    - Rede                        |
|  org.freedesktop.UPower            - Energia                     |
|  org.freedesktop.UDisks2           - Discos                      |
|  org.freedesktop.Accounts          - Contas                      |
|  org.freedesktop.PolicyKit1        - Autorizacao                 |
|  org.bluez                         - Bluetooth                   |
|                                                                  |
+------------------------------------------------------------------+

+------------------------------------------------------------------+
|                        D-BUS SESSION BUS                         |
+------------------------------------------------------------------+
|                                                                  |
|  org.winux.Shell                   - Winux Shell                 |
|  org.winux.Panel                   - Painel/Taskbar              |
|  org.winux.Notifications           - Notificacoes                |
|  org.winux.Settings                - Configuracoes               |
|  org.winux.ScreenSaver             - Protetor de Tela            |
|                                                                  |
|  org.freedesktop.Notifications     - Notificacoes padrao         |
|  org.freedesktop.ScreenSaver       - Screen saver padrao         |
|  org.freedesktop.FileManager1      - Gerenciador de arquivos     |
|                                                                  |
+------------------------------------------------------------------+
```

### IPC entre Aplicacoes

```
+---------------+                           +---------------+
|    App A      |                           |    App B      |
+---------------+                           +---------------+
       |                                           |
       |  +-----------------------------------+    |
       +->|          D-Bus Session            |<---+
          |                                   |
          |  - Method Calls                   |
          |  - Signals                        |
          |  - Properties                     |
          +-----------------------------------+

+---------------+                           +---------------+
|    Shell      |                           |   Compositor  |
+---------------+                           +---------------+
       |                                           |
       |  +-----------------------------------+    |
       +->|      Wayland Protocol             |<---+
          |                                   |
          |  - wl_surface                     |
          |  - xdg_shell                      |
          |  - layer_shell                    |
          |  - Custom protocols              |
          +-----------------------------------+
```

---

## Seguranca

### Modelo de Seguranca

```
+------------------------------------------------------------------+
|                        SECURITY LAYERS                           |
+------------------------------------------------------------------+
|                                                                  |
|  +------------------------------------------------------------+  |
|  |                    USER SPACE                              |  |
|  |  +----------------+  +----------------+  +----------------+ |  |
|  |  |    AppArmor    |  |    Firejail    |  |   Flatpak      | |  |
|  |  |   (Profiles)   |  |   (Sandbox)    |  |  (Portals)     | |  |
|  |  +----------------+  +----------------+  +----------------+ |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  +------------------------------------------------------------+  |
|  |                    SYSTEM LEVEL                            |  |
|  |  +----------------+  +----------------+  +----------------+ |  |
|  |  |   PolicyKit    |  |     sudo       |  |     PAM        | |  |
|  |  |  (Authorization)|  |   (Elevation)  |  |(Authentication)| |  |
|  |  +----------------+  +----------------+  +----------------+ |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  +------------------------------------------------------------+  |
|  |                    KERNEL LEVEL                            |  |
|  |  +----------------+  +----------------+  +----------------+ |  |
|  |  |   Namespaces   |  |    cgroups     |  |    seccomp     | |  |
|  |  |  (Isolation)   |  |  (Resources)   |  |  (Syscalls)    | |  |
|  |  +----------------+  +----------------+  +----------------+ |  |
|  +------------------------------------------------------------+  |
|                                                                  |
+------------------------------------------------------------------+
```

---

## Estrutura de Diretorios do Sistema

```
/
├── bin/                    # Binarios essenciais (symlink para usr/bin)
├── boot/                   # Bootloader e kernel
│   ├── grub/
│   └── vmlinuz-*
├── dev/                    # Dispositivos
├── etc/                    # Configuracoes do sistema
│   ├── winux/              # Configuracoes especificas do Winux
│   ├── sysctl.d/           # Tunables do kernel
│   ├── modprobe.d/         # Configuracoes de modulos
│   └── systemd/            # Configuracoes do systemd
├── home/                   # Diretorios de usuarios
│   └── user/
│       ├── .config/        # Configuracoes de apps
│       ├── .local/         # Dados locais
│       └── .wine/          # Wine prefix
├── lib/                    # Bibliotecas (symlink)
├── media/                  # Midias removiveis
├── mnt/                    # Pontos de montagem
├── opt/                    # Software adicional
├── proc/                   # Processos (virtual)
├── root/                   # Home do root
├── run/                    # Runtime data
├── srv/                    # Servicos
├── sys/                    # Sistema (virtual)
├── tmp/                    # Arquivos temporarios
├── usr/                    # Hierarquia secundaria
│   ├── bin/                # Binarios de usuario
│   ├── lib/                # Bibliotecas
│   ├── share/
│   │   ├── winux/          # Recursos do Winux
│   │   │   ├── backgrounds/
│   │   │   ├── themes/
│   │   │   └── scripts/
│   │   ├── applications/   # Desktop files
│   │   └── icons/          # Icones
│   └── local/              # Software local
└── var/                    # Dados variaveis
    ├── cache/
    ├── log/
    └── lib/
```

---

## Referencias

- [Wayland Protocol](https://wayland.freedesktop.org/)
- [Smithay Documentation](https://smithay.github.io/smithay/)
- [GTK4 Architecture](https://docs.gtk.org/gtk4/)
- [PipeWire Design](https://pipewire.org/)
- [Wine Wiki](https://wiki.winehq.org/)
- [DXVK GitHub](https://github.com/doitsujin/dxvk)
- [Freedesktop Specifications](https://specifications.freedesktop.org/)

---

**Winux OS Project - 2026**
*Arquitetura projetada para performance e usabilidade*
