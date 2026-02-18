# Winux OS - Guia Completo de Build da ISO

Este documento descreve como gerar a ISO do Winux OS a partir do codigo-fonte.

## Indice

1. [Requisitos do Sistema](#requisitos-do-sistema)
2. [Preparacao do Ambiente](#preparacao-do-ambiente)
3. [Build Completo](#build-completo)
4. [Build Rapido para Desenvolvimento](#build-rapido-para-desenvolvimento)
5. [Build com Docker](#build-com-docker)
6. [Fases do Build](#fases-do-build)
7. [Customizacao](#customizacao)
8. [Testando a ISO](#testando-a-iso)
9. [Troubleshooting](#troubleshooting)

---

## Requisitos do Sistema

### Hardware Minimo

| Recurso | Minimo | Recomendado |
|---------|--------|-------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 16 GB |
| Disco | 50 GB | 100 GB |
| Rede | Banda larga | Banda larga |

### Sistema Operacional

O build deve ser feito em:
- **Ubuntu 22.04 LTS** ou superior
- **Ubuntu 24.04 LTS** (recomendado)
- Derivados do Ubuntu (Linux Mint, Pop!_OS, etc)

### Software Necessario

O script instala automaticamente as dependencias, mas para referencia:

```bash
# Ferramentas de build
apt install build-essential git curl wget

# Ferramentas de ISO
apt install debootstrap squashfs-tools xorriso mtools dosfstools

# GRUB e boot
apt install grub-pc-bin grub-efi-amd64-bin grub-common isolinux syslinux-common

# Desenvolvimento GTK/Rust
apt install libgtk-4-dev libadwaita-1-dev pkg-config

# Rust (instalado automaticamente se necessario)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Preparacao do Ambiente

### 1. Clonar o Repositorio

```bash
git clone https://github.com/winux-os/winux.git
cd winux
```

### 2. Verificar Estrutura do Projeto

```
winux/
├── apps/                    # Aplicativos Rust
│   ├── winux-files/
│   ├── winux-terminal/
│   ├── winux-settings/
│   └── ...
├── desktop/                 # Componentes do desktop
│   ├── winux-compositor/
│   ├── winux-shell/
│   └── winux-panel/
├── themes/                  # Temas
│   └── winux-fluent/
├── build/                   # Scripts de build
│   ├── scripts/
│   │   ├── build-winux-iso.sh
│   │   └── quick-build.sh
│   ├── config/
│   │   └── calamares/
│   └── Dockerfile
└── system/                  # Configuracoes do sistema
    └── etc/
```

### 3. Verificar Permissoes

O build requer privilegios de root:

```bash
sudo -v  # Verificar se tem acesso sudo
```

---

## Build Completo

### Metodo Padrao

```bash
cd winux
sudo ./build/scripts/build-winux-iso.sh all
```

O build leva aproximadamente **30-60 minutos** dependendo do hardware e conexao.

### Output

Apos o build, os arquivos estarao em `./output/`:

```
output/
├── winux-1.0-aurora-amd64.iso      # ISO principal
├── winux-1.0-aurora-amd64.iso.md5
├── winux-1.0-aurora-amd64.iso.sha256
├── winux-1.0-aurora-amd64.iso.sha512
└── winux-1.0-aurora-amd64.iso.info  # Informacoes do build
```

### Variaveis de Ambiente

Customize o build com variaveis de ambiente:

```bash
# Versao customizada
sudo WINUX_VERSION=1.1 WINUX_CODENAME=beta ./build/scripts/build-winux-iso.sh all

# Diretorio de output diferente
sudo OUTPUT_DIR=/mnt/iso ./build/scripts/build-winux-iso.sh all

# Compressao maxima (ISO menor, build mais lento)
sudo COMPRESSION=xz COMPRESSION_LEVEL=9 ./build/scripts/build-winux-iso.sh all

# Mais jobs paralelos
sudo PARALLEL_JOBS=16 ./build/scripts/build-winux-iso.sh all

# Modo debug
sudo DEBUG=true ./build/scripts/build-winux-iso.sh all
```

### Tabela de Variaveis

| Variavel | Default | Descricao |
|----------|---------|-----------|
| `WINUX_VERSION` | 1.0 | Versao do sistema |
| `WINUX_CODENAME` | aurora | Codenome do release |
| `BUILD_DIR` | /tmp/winux-build | Diretorio de trabalho |
| `OUTPUT_DIR` | ./output | Diretorio da ISO final |
| `CACHE_DIR` | $BUILD_DIR/cache | Cache para builds |
| `COMPRESSION` | zstd | Tipo: zstd, xz, lz4, gzip |
| `COMPRESSION_LEVEL` | 19 | Nivel de compressao |
| `PARALLEL_JOBS` | nproc | Jobs paralelos |
| `FORCE_REBUILD` | false | Forcar reconstrucao |
| `CLEANUP` | false | Limpar apos build |
| `DEBUG` | false | Modo verboso |

---

## Build Rapido para Desenvolvimento

Para desenvolvimento iterativo, use o script de build rapido:

```bash
# Build completo rapido (compressao lz4)
sudo ./build/scripts/quick-build.sh full

# Apenas recompilar apps Rust
sudo ./build/scripts/quick-build.sh apps

# Apenas regenerar squashfs e ISO
sudo ./build/scripts/quick-build.sh squashfs

# Limpar build (manter cache)
sudo ./build/scripts/quick-build.sh clean

# Limpar tudo
sudo ./build/scripts/quick-build.sh clean-all
```

### Diferenças do Build Rapido

| Aspecto | Build Normal | Quick Build |
|---------|--------------|-------------|
| Compressao | zstd (nivel 19) | lz4 (nivel 1) |
| Tamanho ISO | ~2.5 GB | ~3.5 GB |
| Tempo | 30-60 min | 15-25 min |
| Uso | Release | Desenvolvimento |

---

## Build com Docker

### Usando o Dockerfile

```bash
# Construir imagem Docker
cd winux
docker build -t winux-builder -f build/Dockerfile .

# Executar build
docker run --privileged -v $(pwd)/output:/output winux-builder

# Build com variaveis customizadas
docker run --privileged \
  -e WINUX_VERSION=1.1 \
  -e COMPRESSION=xz \
  -v $(pwd)/output:/output \
  winux-builder
```

### Vantagens do Docker

- Ambiente isolado e reproduzivel
- Nao precisa instalar dependencias no host
- Facil de usar em CI/CD
- Consistencia entre diferentes maquinas

---

## Fases do Build

O build e dividido em fases que podem ser executadas individualmente:

### 1. Preparacao (`prepare`)

```bash
sudo ./build/scripts/build-winux-iso.sh prepare
```

- Cria estrutura de diretorios
- Verifica dependencias
- Inicializa logs

### 2. Debootstrap (`debootstrap`)

```bash
sudo ./build/scripts/build-winux-iso.sh debootstrap
```

- Baixa sistema base Ubuntu
- Cria ambiente chroot

### 3. Configuracao Base (`configure`)

```bash
sudo ./build/scripts/build-winux-iso.sh configure
```

- Configura repositorios APT
- Instala pacotes essenciais
- Configura locale, timezone, usuarios

### 4. Apps Rust (`rust-apps`)

```bash
sudo ./build/scripts/build-winux-iso.sh rust-apps
```

- Compila todos os apps Winux
- Gera arquivos .desktop
- Instala no chroot

### 5. Tema (`theme`)

```bash
sudo ./build/scripts/build-winux-iso.sh theme
```

- Copia tema Winux Fluent
- Configura GTK3/GTK4
- Aplica configuracoes visuais

### 6. Calamares (`calamares`)

```bash
sudo ./build/scripts/build-winux-iso.sh calamares
```

- Configura instalador
- Copia branding
- Cria atalhos

### 7. Finalizacao (`finalize`)

```bash
sudo ./build/scripts/build-winux-iso.sh finalize
```

- Habilita servicos systemd
- Configura autologin
- Executa scripts pos-install
- Limpa sistema

### 8. Squashfs (`squashfs`)

```bash
sudo ./build/scripts/build-winux-iso.sh squashfs
```

- Comprime filesystem
- Gera manifest
- Pode levar 10-30 minutos

### 9. Boot (`boot`)

```bash
sudo ./build/scripts/build-winux-iso.sh boot
```

- Copia kernel e initrd
- Configura GRUB (UEFI)
- Configura ISOLINUX (BIOS)

### 10. ISO (`iso`)

```bash
sudo ./build/scripts/build-winux-iso.sh iso
```

- Gera imagem EFI
- Cria ISO hibrida
- Bootavel em UEFI e BIOS

### 11. Checksums (`checksums`)

```bash
sudo ./build/scripts/build-winux-iso.sh checksums
```

- Calcula MD5, SHA256, SHA512
- Gera arquivo de informacoes

### Limpeza (`clean`)

```bash
sudo ./build/scripts/build-winux-iso.sh clean
```

- Remove arquivos temporarios
- Libera espaco em disco

---

## Customizacao

### Adicionar Pacotes

Edite `build/scripts/build-winux-iso.sh`, funcao `phase_configure_base()`:

```bash
# Na secao de instalacao de pacotes, adicione:
apt install -y \
    seu-pacote-aqui \
    outro-pacote
```

### Modificar Configuracoes do Sistema

Edite os arquivos em `system/etc/`:

```
system/etc/
├── sysctl.d/99-winux-performance.conf
├── modprobe.d/winux-blacklist.conf
└── environment.d/winux-gaming.conf
```

### Personalizar o Instalador

Edite os arquivos em `build/config/calamares/`:

```
build/config/calamares/
├── settings.conf           # Configuracao geral
├── branding/winux/         # Visual do instalador
│   ├── branding.desc
│   └── show.qml
└── modules/                # Modulos do instalador
    ├── bootloader.conf
    ├── partition.conf
    ├── users.conf
    └── welcome.conf
```

### Mudar o Tema

Edite os arquivos em `themes/winux-fluent/`:

```
themes/winux-fluent/
├── gtk-4.0/
│   ├── gtk.css
│   └── colors-dark.css
└── index.theme
```

---

## Testando a ISO

### Testar em VM (QEMU)

```bash
# Instalar QEMU
sudo apt install qemu-system-x86 ovmf

# Testar com UEFI
qemu-system-x86_64 \
    -enable-kvm \
    -m 4096 \
    -cpu host \
    -smp 4 \
    -bios /usr/share/ovmf/OVMF.fd \
    -drive file=output/winux-1.0-aurora-amd64.iso,format=raw \
    -boot d

# Testar com BIOS legado
qemu-system-x86_64 \
    -enable-kvm \
    -m 4096 \
    -cpu host \
    -smp 4 \
    -cdrom output/winux-1.0-aurora-amd64.iso \
    -boot d
```

### Testar em VM (VirtualBox)

1. Criar nova VM (Linux, Ubuntu 64-bit)
2. RAM: 4 GB+
3. Habilitar EFI (Configuracoes > Sistema > Habilitar EFI)
4. Armazenamento: Montar a ISO no drive optico
5. Iniciar VM

### Testar em VM (VMware)

1. Criar nova VM
2. Tipo: Linux, Ubuntu 64-bit
3. Firmware: UEFI
4. RAM: 4 GB+
5. Montar ISO e iniciar

### Criar USB Bootavel

```bash
# Identificar dispositivo USB
lsblk

# CUIDADO: substitua /dev/sdX pelo dispositivo correto!
sudo dd if=output/winux-1.0-aurora-amd64.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Ou use ferramentas graficas:
- **balenaEtcher** - https://etcher.balena.io
- **Ventoy** - https://ventoy.net (suporta multiplas ISOs)
- **Rufus** (Windows) - https://rufus.ie

---

## Troubleshooting

### Problema: "debootstrap falhou"

**Causa:** Problemas de rede ou repositorio

**Solucao:**
```bash
# Limpar e tentar novamente
sudo ./build/scripts/build-winux-iso.sh clean
sudo ./build/scripts/build-winux-iso.sh all
```

### Problema: "Sem espaco em disco"

**Causa:** /tmp cheio ou particao pequena

**Solucao:**
```bash
# Usar diretorio diferente
sudo BUILD_DIR=/home/build ./build/scripts/build-winux-iso.sh all
```

### Problema: "squashfs muito lento"

**Causa:** Compressao pesada (zstd nivel 19)

**Solucao:**
```bash
# Usar compressao mais rapida
sudo COMPRESSION=lz4 ./build/scripts/build-winux-iso.sh squashfs
```

### Problema: "Apps Rust nao compilam"

**Causa:** Dependencias faltando

**Solucao:**
```bash
# Instalar dependencias GTK
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config

# Atualizar Rust
rustup update
```

### Problema: "ISO nao boota em UEFI"

**Causa:** Problema na geracao EFI

**Solucao:**
```bash
# Verificar pacotes GRUB
sudo apt install --reinstall grub-efi-amd64-bin

# Rebuildar
sudo FORCE_REBUILD=true ./build/scripts/build-winux-iso.sh boot
sudo ./build/scripts/build-winux-iso.sh iso
```

### Problema: "Chroot nao desmonta"

**Causa:** Processos ainda usando o chroot

**Solucao:**
```bash
# Forcar desmontagem
sudo umount -l /tmp/winux-build/chroot/dev/pts
sudo umount -l /tmp/winux-build/chroot/dev
sudo umount -l /tmp/winux-build/chroot/proc
sudo umount -l /tmp/winux-build/chroot/sys
sudo umount -l /tmp/winux-build/chroot/run

# Ou limpar tudo
sudo ./build/scripts/build-winux-iso.sh clean
```

### Logs e Debug

```bash
# Ver log do build
cat /tmp/winux-build/build-*.log

# Modo debug
sudo DEBUG=true ./build/scripts/build-winux-iso.sh all
```

---

## CI/CD

### GitHub Actions

Exemplo de workflow para build automatico:

```yaml
name: Build ISO

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Build ISO
      run: |
        docker build -t winux-builder -f build/Dockerfile .
        docker run --privileged -v ${{ github.workspace }}/output:/output winux-builder

    - name: Upload ISO
      uses: actions/upload-artifact@v4
      with:
        name: winux-iso
        path: output/*.iso
```

---

## Suporte

- **Issues:** https://github.com/winux-os/winux/issues
- **Discussoes:** https://github.com/winux-os/winux/discussions
- **Wiki:** https://github.com/winux-os/winux/wiki

---

*Documento atualizado em: Fevereiro 2026*
*Versao do Build System: 2.0*
