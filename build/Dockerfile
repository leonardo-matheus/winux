# =============================================================================
# Winux OS - Build Container
# =============================================================================
# Docker container para build reproduzivel da ISO do Winux OS
#
# Uso:
#   docker build -t winux-builder -f build/Dockerfile .
#   docker run --privileged -v $(pwd)/output:/output winux-builder
#
# =============================================================================

FROM ubuntu:24.04

LABEL maintainer="Winux OS Team <team@winux-os.org>"
LABEL description="Container para build da ISO do Winux OS"
LABEL version="2.0"

# Evitar prompts interativos
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=America/Sao_Paulo

# Configuracoes de build (podem ser sobrescritas)
ENV WINUX_VERSION=1.0
ENV WINUX_CODENAME=aurora
ENV BUILD_DIR=/build
ENV OUTPUT_DIR=/output
ENV CACHE_DIR=/cache
ENV COMPRESSION=zstd
ENV COMPRESSION_LEVEL=19
ENV PARALLEL_JOBS=4

# Instalar dependencias do sistema
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Ferramentas essenciais
    build-essential \
    ca-certificates \
    curl \
    git \
    wget \
    gnupg \
    lsb-release \
    software-properties-common \
    # Ferramentas de ISO
    debootstrap \
    squashfs-tools \
    xorriso \
    mtools \
    dosfstools \
    rsync \
    # GRUB e boot
    grub-pc-bin \
    grub-efi-amd64-bin \
    grub-efi-amd64-signed \
    grub-common \
    grub2-common \
    isolinux \
    syslinux-common \
    shim-signed \
    # Desenvolvimento
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libpango1.0-dev \
    libgdk-pixbuf-2.0-dev \
    libglib2.0-dev \
    libcairo2-dev \
    # Utilitarios
    locales \
    sudo \
    file \
    && rm -rf /var/lib/apt/lists/*

# Configurar locale
RUN locale-gen en_US.UTF-8 pt_BR.UTF-8 && \
    update-locale LANG=pt_BR.UTF-8

ENV LANG=pt_BR.UTF-8
ENV LC_ALL=pt_BR.UTF-8

# Instalar Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable --profile minimal

ENV PATH="/root/.cargo/bin:${PATH}"

# Verificar instalacao do Rust
RUN rustc --version && cargo --version

# Criar diretorios de trabalho
RUN mkdir -p /winux /build /output /cache

# Copiar codigo fonte do projeto
COPY . /winux

# Dar permissao de execucao aos scripts
RUN chmod +x /winux/build/scripts/*.sh

# Criar script de entrypoint
RUN cat > /entrypoint.sh << 'EOF'
#!/bin/bash
set -e

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║   WINUX OS - DOCKER BUILD CONTAINER                         ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Configuracao:"
echo "  WINUX_VERSION: ${WINUX_VERSION}"
echo "  WINUX_CODENAME: ${WINUX_CODENAME}"
echo "  COMPRESSION: ${COMPRESSION}"
echo "  PARALLEL_JOBS: ${PARALLEL_JOBS}"
echo ""

cd /winux

# Executar build
exec ./build/scripts/build-winux-iso.sh "${@:-all}"
EOF

RUN chmod +x /entrypoint.sh

# Volume para output
VOLUME ["/output", "/cache"]

# Diretorio de trabalho
WORKDIR /winux

# Entrypoint
ENTRYPOINT ["/entrypoint.sh"]

# Comando padrao (pode ser sobrescrito)
CMD ["all"]
