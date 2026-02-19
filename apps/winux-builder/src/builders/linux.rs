// Linux platform builder
// Builds: .deb, .rpm, .AppImage, .flatpak

use super::{Builder, BuildCommand, DependencyStatus};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct LinuxBuilder;

impl LinuxBuilder {
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
                "dpkg-deb" => "sudo pacman -S dpkg".to_string(),
                "rpmbuild" => "sudo pacman -S rpm-tools".to_string(),
                "appimagetool" => "Baixe de https://appimage.github.io/appimagetool/".to_string(),
                "flatpak-builder" => "sudo pacman -S flatpak-builder".to_string(),
                "cargo-deb" => "cargo install cargo-deb".to_string(),
                "cargo-rpm" => "cargo install cargo-rpm".to_string(),
                _ => format!("Instale {} para continuar", name),
            },
        }
    }
}

impl Builder for LinuxBuilder {
    fn name(&self) -> &str {
        "Linux"
    }

    fn formats(&self) -> &[&str] {
        &["deb", "rpm", "appimage", "flatpak"]
    }

    fn check_dependencies(&self) -> Result<Vec<DependencyStatus>> {
        Ok(vec![
            Self::check_tool("dpkg-deb"),
            Self::check_tool("rpmbuild"),
            Self::check_tool("appimagetool"),
            Self::check_tool("flatpak-builder"),
            Self::check_tool("cargo-deb"),
            Self::check_tool("cargo-rpm"),
        ])
    }

    fn build(&self, project_path: &Path, format: &str, release: bool) -> Result<BuildCommand> {
        let project_str = project_path.to_string_lossy();
        let _mode = if release { "--release" } else { "" };

        match format {
            "deb" => {
                // Build .deb package using cargo-deb
                let script = format!(
                    r#"
# Verifica se cargo-deb esta instalado
if ! command -v cargo-deb &> /dev/null; then
    echo "Instalando cargo-deb..."
    cargo install cargo-deb
fi

# Build do pacote .deb
cargo deb {}

# Mostra resultado
DEB_FILE=$(ls -t target/debian/*.deb 2>/dev/null | head -1)
if [ -n "$DEB_FILE" ]; then
    echo "Pacote DEB criado: $DEB_FILE"
    echo "Informacoes do pacote:"
    dpkg-deb --info "$DEB_FILE" 2>/dev/null || true
else
    echo "Build completo. Verifique target/debian/"
fi
"#,
                    if release { "--release" } else { "" }
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            "rpm" => {
                // Build .rpm package using cargo-rpm
                let script = format!(
                    r#"
# Verifica se cargo-rpm esta instalado
if ! command -v cargo-rpm &> /dev/null; then
    echo "Instalando cargo-rpm..."
    cargo install cargo-rpm
fi

# Inicializa configuracao RPM se necessario
if [ ! -f ".rpm/spec" ]; then
    cargo rpm init
fi

# Build do pacote .rpm
cargo rpm build {}

# Mostra resultado
RPM_FILE=$(ls -t target/release/rpmbuild/RPMS/*/*.rpm 2>/dev/null | head -1)
if [ -n "$RPM_FILE" ]; then
    echo "Pacote RPM criado: $RPM_FILE"
    rpm -qip "$RPM_FILE" 2>/dev/null || true
else
    echo "Build completo. Verifique target/release/rpmbuild/"
fi
"#,
                    if release { "-r" } else { "" }
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            "appimage" => {
                // Build AppImage
                let script = format!(
                    r#"
# Build do projeto
cargo build {}

APP_NAME=$(basename $(pwd))
BUILD_TYPE={}
APP_DIR="AppDir"

# Cria estrutura do AppDir
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/usr/bin"
mkdir -p "$APP_DIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/256x256/apps"

# Copia binario
cp "target/$BUILD_TYPE/$APP_NAME" "$APP_DIR/usr/bin/" 2>/dev/null || \
cp target/$BUILD_TYPE/$(ls target/$BUILD_TYPE/ | grep -v '\.d$' | head -1) "$APP_DIR/usr/bin/$APP_NAME"

# Cria .desktop
cat > "$APP_DIR/usr/share/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Exec=$APP_NAME
Icon=$APP_NAME
Categories=Utility;
EOF

# Link para .desktop no root
ln -sf "usr/share/applications/$APP_NAME.desktop" "$APP_DIR/$APP_NAME.desktop"

# Cria icone placeholder se nao existir
if [ ! -f "$APP_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png" ]; then
    # Cria icone simples com ImageMagick se disponivel
    if command -v convert &> /dev/null; then
        convert -size 256x256 xc:transparent -fill '#3584e4' \
            -draw "roundrectangle 20,20 236,236 30,30" \
            -fill white -gravity center -pointsize 120 \
            -annotate 0 "$(echo $APP_NAME | head -c 1 | tr '[:lower:]' '[:upper:]')" \
            "$APP_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png"
    else
        # Cria PNG vazio minimo
        echo -n "" > "$APP_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png"
    fi
fi

# Link para icone
ln -sf "usr/share/icons/hicolor/256x256/apps/$APP_NAME.png" "$APP_DIR/$APP_NAME.png"

# Cria AppRun
cat > "$APP_DIR/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${{SELF%/*}}
export PATH="${{HERE}}/usr/bin:${{PATH}}"
exec "${{HERE}}/usr/bin/$(basename ${{APPDIR}})" "$@"
EOF
chmod +x "$APP_DIR/AppRun"

# Build AppImage
if command -v appimagetool &> /dev/null; then
    ARCH=x86_64 appimagetool "$APP_DIR" "$APP_NAME-x86_64.AppImage"
    echo "AppImage criado: $APP_NAME-x86_64.AppImage"
else
    echo "Aviso: appimagetool nao encontrado."
    echo "Baixe de: https://github.com/AppImage/AppImageKit/releases"
    echo "AppDir preparado em: $APP_DIR/"
fi
"#,
                    if release { "--release" } else { "" },
                    if release { "release" } else { "debug" }
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            "flatpak" => {
                // Build Flatpak
                let script = format!(
                    r#"
APP_NAME=$(basename $(pwd))
APP_ID="org.winux.$APP_NAME"

# Build do projeto primeiro
cargo build {}

# Cria manifest do Flatpak
cat > "$APP_NAME.flatpak.yml" << EOF
app-id: $APP_ID
runtime: org.freedesktop.Platform
runtime-version: '23.08'
sdk: org.freedesktop.Sdk
command: $APP_NAME

finish-args:
  - --share=ipc
  - --socket=fallback-x11
  - --socket=wayland
  - --device=dri
  - --filesystem=home

modules:
  - name: $APP_NAME
    buildsystem: simple
    build-commands:
      - install -Dm755 $APP_NAME /app/bin/$APP_NAME
    sources:
      - type: file
        path: target/{}/{}
        dest-filename: $APP_NAME
EOF

# Build do Flatpak
if command -v flatpak-builder &> /dev/null; then
    flatpak-builder --force-clean build-dir "$APP_NAME.flatpak.yml"

    # Cria bundle
    flatpak build-export repo build-dir
    flatpak build-bundle repo "$APP_NAME.flatpak" "$APP_ID"

    echo "Flatpak criado: $APP_NAME.flatpak"
else
    echo "Erro: flatpak-builder nao encontrado"
    echo "Instale: sudo pacman -S flatpak-builder"
    echo "Manifest criado: $APP_NAME.flatpak.yml"
fi
"#,
                    if release { "--release" } else { "" },
                    if release { "release" } else { "debug" },
                    "$(basename $(pwd))"
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            _ => {
                anyhow::bail!("Formato desconhecido para Linux: {}", format)
            }
        }
    }
}
