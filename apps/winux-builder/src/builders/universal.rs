// Universal/Cross-platform builder
// Handles builds for multiple platforms at once

use super::{Builder, BuildCommand, DependencyStatus};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct UniversalBuilder;

impl UniversalBuilder {
    pub fn new() -> Self {
        Self
    }

    fn check_tool(name: &str) -> DependencyStatus {
        let result = Command::new("which").arg(name).output();

        let available = result
            .map(|o| o.status.success())
            .unwrap_or(false);

        let version = if available {
            Command::new(name)
                .arg("--version")
                .output()
                .ok()
                .map(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .to_string()
                })
        } else {
            None
        };

        DependencyStatus {
            name: name.to_string(),
            available,
            version,
            install_hint: match name {
                "cross" => "cargo install cross".to_string(),
                "docker" => "sudo pacman -S docker".to_string(),
                "podman" => "sudo pacman -S podman".to_string(),
                _ => format!("Instale {} para continuar", name),
            },
        }
    }
}

impl Builder for UniversalBuilder {
    fn name(&self) -> &str {
        "Universal (Cross-platform)"
    }

    fn formats(&self) -> &[&str] {
        &["all-linux", "all-desktop", "all"]
    }

    fn check_dependencies(&self) -> Result<Vec<DependencyStatus>> {
        Ok(vec![
            Self::check_tool("cross"),
            Self::check_tool("docker"),
            Self::check_tool("podman"),
        ])
    }

    fn build(&self, project_path: &Path, format: &str, release: bool) -> Result<BuildCommand> {
        let project_str = project_path.to_string_lossy();
        let mode = if release { "--release" } else { "" };

        match format {
            "all-linux" => {
                // Build all Linux formats
                let script = format!(
                    r#"
echo "=== Build Universal Linux ==="
echo ""

APP_NAME=$(basename $(pwd))
RELEASE_MODE="{mode}"

# Array de targets
declare -a FORMATS=("deb" "rpm" "appimage")

for fmt in "${{FORMATS[@]}}"; do
    echo ""
    echo ">>> Construindo $fmt..."
    echo ""

    case $fmt in
        deb)
            if command -v cargo-deb &> /dev/null; then
                cargo deb $RELEASE_MODE
            else
                echo "Pulando DEB: cargo-deb nao instalado"
            fi
            ;;
        rpm)
            if command -v cargo-rpm &> /dev/null; then
                cargo rpm build
            else
                echo "Pulando RPM: cargo-rpm nao instalado"
            fi
            ;;
        appimage)
            echo "AppImage requer configuracao manual"
            ;;
    esac
done

echo ""
echo "=== Build concluido ==="
echo "Verifique os arquivos em target/"
"#
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            "all-desktop" => {
                // Build for Linux, Windows, macOS
                let script = format!(
                    r#"
echo "=== Build Cross-Platform Desktop ==="
echo ""

APP_NAME=$(basename $(pwd))

# Linux nativo
echo ">>> Build Linux nativo..."
cargo build {mode}

# Windows via cross-compilation
echo ""
echo ">>> Build Windows (cross-compile)..."
if rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
    cargo build {mode} --target x86_64-pc-windows-gnu
else
    echo "Adicionando target Windows..."
    rustup target add x86_64-pc-windows-gnu
    cargo build {mode} --target x86_64-pc-windows-gnu 2>/dev/null || \
        echo "Aviso: Cross-compile Windows pode requerer mingw-w64"
fi

# macOS via cross (requer Docker/Podman)
echo ""
echo ">>> Build macOS..."
if command -v cross &> /dev/null; then
    cross build {mode} --target x86_64-apple-darwin 2>/dev/null || \
        echo "Aviso: Cross-compile macOS requer osxcross ou cross com Docker"
else
    echo "Pulando macOS: 'cross' nao instalado (cargo install cross)"
fi

echo ""
echo "=== Builds disponiveis ==="
ls -la target/release/ 2>/dev/null | head -20
ls -la target/x86_64-pc-windows-gnu/release/*.exe 2>/dev/null || true
"#
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            "all" => {
                // Build everything possible
                let script = format!(
                    r#"
echo "=== Build Completo - Todas as Plataformas ==="
echo ""
echo "Este processo pode demorar varios minutos..."
echo ""

APP_NAME=$(basename $(pwd))

# Funcao auxiliar
build_target() {{
    local target=$1
    local desc=$2
    echo ""
    echo ">>> $desc ($target)..."

    if rustup target list | grep -q "$target (installed)"; then
        cargo build {mode} --target $target && echo "OK: $target" || echo "FALHOU: $target"
    else
        rustup target add $target 2>/dev/null
        cargo build {mode} --target $target 2>/dev/null && echo "OK: $target" || echo "FALHOU: $target"
    fi
}}

# Linux nativo
echo ">>> Linux nativo..."
cargo build {mode}

# Linux ARM
build_target "aarch64-unknown-linux-gnu" "Linux ARM64"

# Windows
build_target "x86_64-pc-windows-gnu" "Windows x64"
build_target "i686-pc-windows-gnu" "Windows x86"

# macOS (requer toolchain especial)
echo ""
echo ">>> macOS (requer osxcross)..."
if command -v cross &> /dev/null && command -v docker &> /dev/null; then
    cross build {mode} --target x86_64-apple-darwin 2>/dev/null || echo "FALHOU: macOS x64"
    cross build {mode} --target aarch64-apple-darwin 2>/dev/null || echo "FALHOU: macOS ARM64"
else
    echo "Pulando macOS - requer 'cross' e Docker"
fi

# Resumo
echo ""
echo "=== Resumo do Build ==="
echo ""
echo "Binarios criados:"
find target -maxdepth 3 -type f \( -name "$APP_NAME" -o -name "$APP_NAME.exe" \) 2>/dev/null | while read f; do
    SIZE=$(du -h "$f" | cut -f1)
    echo "  $f ($SIZE)"
done
"#
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            _ => {
                anyhow::bail!("Formato desconhecido para Universal: {}", format)
            }
        }
    }
}

/// Information about supported targets
pub fn get_supported_targets() -> Vec<TargetInfo> {
    vec![
        TargetInfo {
            triple: "x86_64-unknown-linux-gnu".to_string(),
            platform: "Linux".to_string(),
            arch: "x86_64".to_string(),
            description: "Linux 64-bit (glibc)".to_string(),
        },
        TargetInfo {
            triple: "x86_64-unknown-linux-musl".to_string(),
            platform: "Linux".to_string(),
            arch: "x86_64".to_string(),
            description: "Linux 64-bit (musl, static)".to_string(),
        },
        TargetInfo {
            triple: "aarch64-unknown-linux-gnu".to_string(),
            platform: "Linux".to_string(),
            arch: "aarch64".to_string(),
            description: "Linux ARM64".to_string(),
        },
        TargetInfo {
            triple: "x86_64-pc-windows-gnu".to_string(),
            platform: "Windows".to_string(),
            arch: "x86_64".to_string(),
            description: "Windows 64-bit (MinGW)".to_string(),
        },
        TargetInfo {
            triple: "x86_64-pc-windows-msvc".to_string(),
            platform: "Windows".to_string(),
            arch: "x86_64".to_string(),
            description: "Windows 64-bit (MSVC)".to_string(),
        },
        TargetInfo {
            triple: "x86_64-apple-darwin".to_string(),
            platform: "macOS".to_string(),
            arch: "x86_64".to_string(),
            description: "macOS Intel".to_string(),
        },
        TargetInfo {
            triple: "aarch64-apple-darwin".to_string(),
            platform: "macOS".to_string(),
            arch: "aarch64".to_string(),
            description: "macOS Apple Silicon".to_string(),
        },
    ]
}

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub triple: String,
    pub platform: String,
    pub arch: String,
    pub description: String,
}
