# =============================================================================
# Winux OS - Makefile Principal
# =============================================================================

.PHONY: all build clean test apps desktop iso help

# Variáveis
CARGO := cargo
BUILD_SCRIPT := ./build/scripts/build-winux-iso.sh

# -----------------------------------------------------------------------------
# Targets Principais
# -----------------------------------------------------------------------------

all: apps ## Build todas as aplicações

help: ## Mostra esta ajuda
	@echo "Winux OS - Comandos disponíveis:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
	@echo ""

# -----------------------------------------------------------------------------
# Aplicações Rust
# -----------------------------------------------------------------------------

apps: ## Build todas as aplicações Rust
	$(CARGO) build --workspace

apps-release: ## Build aplicações em modo release
	$(CARGO) build --workspace --release

test: ## Executa testes
	$(CARGO) test --workspace

fmt: ## Formata código
	$(CARGO) fmt --all

clippy: ## Executa linter
	$(CARGO) clippy --all-targets --all-features

# -----------------------------------------------------------------------------
# Componentes Individuais
# -----------------------------------------------------------------------------

files: ## Build Winux Files
	$(CARGO) build -p winux-files

terminal: ## Build Winux Terminal
	$(CARGO) build -p winux-terminal

settings: ## Build Winux Settings
	$(CARGO) build -p winux-settings

store: ## Build Winux Store
	$(CARGO) build -p winux-store

monitor: ## Build Winux Monitor
	$(CARGO) build -p winux-monitor

edit: ## Build Winux Edit
	$(CARGO) build -p winux-edit

compositor: ## Build Winux Compositor
	$(CARGO) build -p winux-compositor

panel: ## Build Winux Panel
	$(CARGO) build -p winux-panel

shell: ## Build Winux Shell
	$(CARGO) build -p winux-shell

# -----------------------------------------------------------------------------
# Desktop Components
# -----------------------------------------------------------------------------

desktop: compositor panel shell ## Build todos os componentes desktop

# -----------------------------------------------------------------------------
# ISO Build (requer Linux + root)
# -----------------------------------------------------------------------------

iso: ## Build ISO completa (requer sudo)
	sudo $(BUILD_SCRIPT) all

iso-prepare: ## Preparar ambiente de build
	sudo $(BUILD_SCRIPT) prepare

iso-clean: ## Limpar arquivos de build
	sudo $(BUILD_SCRIPT) clean

# -----------------------------------------------------------------------------
# Limpeza
# -----------------------------------------------------------------------------

clean: ## Limpar artefatos de build
	$(CARGO) clean
	rm -rf target/
	rm -rf output/*.iso

clean-all: clean iso-clean ## Limpar tudo

# -----------------------------------------------------------------------------
# Desenvolvimento
# -----------------------------------------------------------------------------

dev-files: ## Executar Winux Files em modo dev
	$(CARGO) run -p winux-files

dev-terminal: ## Executar Winux Terminal em modo dev
	$(CARGO) run -p winux-terminal

dev-settings: ## Executar Winux Settings em modo dev
	$(CARGO) run -p winux-settings

# -----------------------------------------------------------------------------
# Documentação
# -----------------------------------------------------------------------------

docs: ## Gerar documentação
	$(CARGO) doc --workspace --no-deps --open
