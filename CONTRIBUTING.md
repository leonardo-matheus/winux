# Contribuindo para o Winux OS

Obrigado pelo interesse em contribuir com o Winux OS!

## Como Contribuir

### Reportando Bugs

1. Verifique se o bug já não foi reportado
2. Abra uma issue com:
   - Descrição clara do problema
   - Passos para reproduzir
   - Comportamento esperado vs atual
   - Informações do sistema (GPU, drivers, etc.)

### Sugerindo Funcionalidades

1. Abra uma issue com tag `enhancement`
2. Descreva a funcionalidade proposta
3. Explique o caso de uso

### Pull Requests

1. Fork o repositório
2. Crie uma branch a partir de `develop`:
   ```bash
   git checkout -b feature/minha-feature develop
   ```
3. Faça suas alterações
4. Teste localmente
5. Commit com mensagens claras:
   ```bash
   git commit -m "feat: adiciona suporte a XYZ"
   ```
6. Push e abra um PR para `develop`

## Padrões de Código

### Rust

- Use `rustfmt` para formatação
- Use `clippy` para linting
- Documente funções públicas
- Escreva testes para novas funcionalidades

```bash
cargo fmt --all
cargo clippy --all-targets
cargo test
```

### Commits

Seguimos Conventional Commits:

- `feat:` - Nova funcionalidade
- `fix:` - Correção de bug
- `docs:` - Documentação
- `style:` - Formatação
- `refactor:` - Refatoração
- `test:` - Testes
- `chore:` - Manutenção

### Branches

- `main` - Releases estáveis
- `develop` - Desenvolvimento
- `sprint/XX-nome` - Sprints
- `feature/nome` - Features
- `fix/nome` - Correções

## Estrutura do Projeto

```
winux/
├── apps/          # Aplicações Rust
├── build/         # Sistema de build
├── compatibility/ # Wine/Proton
├── desktop/       # Desktop Environment
├── docs/          # Documentação
├── drivers/       # Scripts de drivers
├── kernel/        # Kernel customizado
├── system/        # Configs de sistema
└── themes/        # Temas visuais
```

## Ambiente de Desenvolvimento

### Requisitos

- Rust 1.75+
- GTK4 development libraries
- Ubuntu 22.04+ (para build da ISO)

### Setup

```bash
# Instalar dependências (Ubuntu/Debian)
sudo apt install -y \
    build-essential \
    libgtk-4-dev \
    libadwaita-1-dev \
    libvte-2.91-gtk4-dev

# Build
cargo build --workspace

# Testes
cargo test --workspace
```

## Código de Conduta

- Seja respeitoso
- Aceite feedback construtivo
- Foque no que é melhor para a comunidade

## Dúvidas?

Abra uma issue com a tag `question`.

---

**Winux OS Team**
