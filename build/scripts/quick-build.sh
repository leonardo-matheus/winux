#!/bin/bash
# =============================================================================
# Winux OS - Quick Build Script
# =============================================================================
# Build rapido para desenvolvimento e testes
# Usa cache agressivo e compressao mais rapida
# =============================================================================

set -euo pipefail

# Configuracoes de Quick Build
export QUICK_BUILD=true
export COMPRESSION=lz4
export COMPRESSION_LEVEL=1
export PARALLEL_JOBS=${PARALLEL_JOBS:-$(nproc)}
export CACHE_DIR="${CACHE_DIR:-/tmp/winux-cache}"
export BUILD_DIR="${BUILD_DIR:-/tmp/winux-quick-build}"
export OUTPUT_DIR="${OUTPUT_DIR:-$(pwd)/output-quick}"

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Script principal
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAIN_BUILD_SCRIPT="${SCRIPT_DIR}/build-winux-iso.sh"

# -----------------------------------------------------------------------------
# Funcoes
# -----------------------------------------------------------------------------
log_info() { echo -e "${GREEN}[QUICK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[QUICK]${NC} $1"; }
log_error() { echo -e "${RED}[QUICK]${NC} $1"; }

show_banner() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                            ║${NC}"
    echo -e "${CYAN}║   ${BOLD}WINUX OS - QUICK BUILD${NC}${CYAN}                                  ║${NC}"
    echo -e "${CYAN}║   Build rapido para desenvolvimento e testes               ║${NC}"
    echo -e "${CYAN}║                                                            ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

show_config() {
    echo -e "${BLUE}Configuracao:${NC}"
    echo "  Compressao: ${COMPRESSION} (nivel ${COMPRESSION_LEVEL})"
    echo "  Jobs paralelos: ${PARALLEL_JOBS}"
    echo "  Build dir: ${BUILD_DIR}"
    echo "  Cache dir: ${CACHE_DIR}"
    echo "  Output dir: ${OUTPUT_DIR}"
    echo ""
}

check_cache() {
    log_info "Verificando cache..."

    # Criar diretorio de cache se nao existir
    mkdir -p "${CACHE_DIR}"/{debootstrap,packages,rust}

    # Verificar se tem cache de debootstrap
    if [[ -d "${CACHE_DIR}/debootstrap" && "$(ls -A ${CACHE_DIR}/debootstrap 2>/dev/null)" ]]; then
        log_info "Cache de debootstrap encontrado - build sera mais rapido"
    fi

    # Verificar cache de Rust
    if [[ -d "${CACHE_DIR}/rust" && "$(ls -A ${CACHE_DIR}/rust 2>/dev/null)" ]]; then
        log_info "Cache de compilacao Rust encontrado"
    fi
}

quick_rebuild_squashfs() {
    # Rebuild apenas o squashfs (mais comum durante desenvolvimento)
    log_info "Recriando apenas squashfs (build incremental)..."

    export FORCE_REBUILD=true
    "${MAIN_BUILD_SCRIPT}" squashfs
    "${MAIN_BUILD_SCRIPT}" boot
    "${MAIN_BUILD_SCRIPT}" iso
    "${MAIN_BUILD_SCRIPT}" checksums
}

quick_rebuild_apps() {
    # Rebuild apenas os apps Rust
    log_info "Recompilando apenas apps Rust..."

    "${MAIN_BUILD_SCRIPT}" rust-apps
}

quick_full() {
    # Build completo com otimizacoes para velocidade
    log_info "Iniciando build completo (modo rapido)..."

    "${MAIN_BUILD_SCRIPT}" all
}

show_help() {
    echo "Uso: $0 [COMANDO]"
    echo ""
    echo "Comandos:"
    echo "  full         Build completo (modo rapido)"
    echo "  squashfs     Rebuild apenas squashfs e ISO"
    echo "  apps         Recompilar apenas apps Rust"
    echo "  clean        Limpar build, manter cache"
    echo "  clean-all    Limpar tudo, incluindo cache"
    echo ""
    echo "Variaveis de ambiente:"
    echo "  CACHE_DIR    Diretorio de cache (default: /tmp/winux-cache)"
    echo "  BUILD_DIR    Diretorio de build (default: /tmp/winux-quick-build)"
    echo "  OUTPUT_DIR   Diretorio de output (default: ./output-quick)"
    echo ""
    echo "Exemplos:"
    echo "  sudo $0 full       # Build completo rapido"
    echo "  sudo $0 squashfs   # Apenas regenerar ISO"
    echo "  sudo $0 apps       # Apenas recompilar apps"
    echo ""
    echo "Diferencas do build normal:"
    echo "  - Usa compressao LZ4 (mais rapida, ISO maior)"
    echo "  - Mantem cache entre builds"
    echo "  - Otimizado para ciclo rapido de desenvolvimento"
    echo ""
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    show_banner

    # Verificar se e root
    if [[ $EUID -ne 0 ]]; then
        log_error "Este script deve ser executado como root"
        echo "Use: sudo $0 $*"
        exit 1
    fi

    # Verificar script principal
    if [[ ! -f "${MAIN_BUILD_SCRIPT}" ]]; then
        log_error "Script principal nao encontrado: ${MAIN_BUILD_SCRIPT}"
        exit 1
    fi

    show_config
    check_cache

    case "${1:-full}" in
        -h|--help|help)
            show_help
            ;;
        full)
            quick_full
            ;;
        squashfs)
            quick_rebuild_squashfs
            ;;
        apps)
            quick_rebuild_apps
            ;;
        clean)
            log_info "Limpando build dir (mantendo cache)..."
            rm -rf "${BUILD_DIR}"
            log_info "Build dir limpo"
            ;;
        clean-all)
            log_info "Limpando tudo (incluindo cache)..."
            rm -rf "${BUILD_DIR}"
            rm -rf "${CACHE_DIR}"
            log_info "Tudo limpo"
            ;;
        *)
            log_error "Comando desconhecido: $1"
            show_help
            exit 1
            ;;
    esac

    echo ""
    log_info "Quick build finalizado!"
    echo ""
}

main "$@"
