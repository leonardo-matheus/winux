// Windows platform builder
// Builds: .exe, .msi

use super::{Builder, BuildCommand, DependencyStatus};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct WindowsBuilder;

impl WindowsBuilder {
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
                "x86_64-w64-mingw32-gcc" => "sudo pacman -S mingw-w64-gcc".to_string(),
                "wixl" => "sudo pacman -S msitools".to_string(),
                "makensis" => "sudo pacman -S nsis".to_string(),
                "wine" => "sudo pacman -S wine".to_string(),
                _ => format!("Instale {} para continuar", name),
            },
        }
    }
}

impl Builder for WindowsBuilder {
    fn name(&self) -> &str {
        "Windows"
    }

    fn formats(&self) -> &[&str] {
        &["exe", "msi"]
    }

    fn check_dependencies(&self) -> Result<Vec<DependencyStatus>> {
        Ok(vec![
            Self::check_tool("x86_64-w64-mingw32-gcc"),
            Self::check_tool("wixl"),
            Self::check_tool("makensis"),
            Self::check_tool("wine"),
        ])
    }

    fn build(&self, project_path: &Path, format: &str, release: bool) -> Result<BuildCommand> {
        let project_str = project_path.to_string_lossy();
        let mode = if release { "--release" } else { "" };

        match format {
            "exe" => {
                // Cross-compile for Windows using MinGW
                let script = format!(
                    r#"
# Adiciona target Windows se necessario
rustup target add x86_64-pc-windows-gnu 2>/dev/null || true

# Configura o linker para Windows
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc

# Build para Windows
cargo build {mode} --target x86_64-pc-windows-gnu

# Localiza o executavel
EXE_PATH="target/x86_64-pc-windows-gnu/{}/$(basename $(pwd)).exe"
if [ -f "$EXE_PATH" ]; then
    echo "Executavel criado: $EXE_PATH"
    # Strip para reduzir tamanho
    x86_64-w64-mingw32-strip "$EXE_PATH" 2>/dev/null || true
else
    echo "Build completo. Verifique target/x86_64-pc-windows-gnu/{mode}/"
fi
"#,
                    mode = if release { "release" } else { "debug" }
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            "msi" => {
                // Create MSI installer using wixl (WiX for Linux)
                let script = format!(
                    r#"
# Primeiro, faz build do EXE
rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc

cargo build {mode} --target x86_64-pc-windows-gnu

APP_NAME=$(basename $(pwd))
VERSION="1.0.0"
EXE_PATH="target/x86_64-pc-windows-gnu/{build_type}/${{APP_NAME}}.exe"

# Cria arquivo WXS para o wixl
cat > "${{APP_NAME}}.wxs" << 'WXSEOF'
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*"
             Name="__APP_NAME__"
             Language="1033"
             Version="__VERSION__"
             Manufacturer="Winux"
             UpgradeCode="12345678-1234-1234-1234-123456789012">

        <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine"/>

        <MediaTemplate EmbedCab="yes"/>

        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFilesFolder">
                <Directory Id="INSTALLFOLDER" Name="__APP_NAME__">
                    <Component Id="MainExecutable" Guid="*">
                        <File Id="MainExe" Source="__EXE_PATH__" KeyPath="yes"/>
                    </Component>
                </Directory>
            </Directory>
        </Directory>

        <Feature Id="MainFeature" Level="1">
            <ComponentRef Id="MainExecutable"/>
        </Feature>
    </Product>
</Wix>
WXSEOF

# Substitui placeholders
sed -i "s/__APP_NAME__/${{APP_NAME}}/g" "${{APP_NAME}}.wxs"
sed -i "s/__VERSION__/${{VERSION}}/g" "${{APP_NAME}}.wxs"
sed -i "s|__EXE_PATH__|${{EXE_PATH}}|g" "${{APP_NAME}}.wxs"

# Gera MSI com wixl
if command -v wixl &> /dev/null; then
    wixl -v "${{APP_NAME}}.wxs" -o "${{APP_NAME}}.msi"
    echo "MSI criado: ${{APP_NAME}}.msi"
else
    echo "Erro: wixl nao encontrado. Instale: sudo pacman -S msitools"
    exit 1
fi

# Limpa arquivo temporario
rm -f "${{APP_NAME}}.wxs"
"#,
                    mode = if release { "--release" } else { "" },
                    build_type = if release { "release" } else { "debug" }
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            _ => {
                anyhow::bail!("Formato desconhecido para Windows: {}", format)
            }
        }
    }
}

/// Generate NSIS installer script
pub fn generate_nsis_script(app_name: &str, version: &str, exe_path: &str) -> String {
    format!(
        r#"
!include "MUI2.nsh"

Name "{app_name}"
OutFile "{app_name}-{version}-setup.exe"
InstallDir "$PROGRAMFILES\{app_name}"
RequestExecutionLevel admin

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "PortugueseBR"

Section "Install"
    SetOutPath $INSTDIR
    File "{exe_path}"

    CreateDirectory "$SMPROGRAMS\{app_name}"
    CreateShortCut "$SMPROGRAMS\{app_name}\{app_name}.lnk" "$INSTDIR\{app_name}.exe"
    CreateShortCut "$DESKTOP\{app_name}.lnk" "$INSTDIR\{app_name}.exe"

    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\{app_name}.exe"
    Delete "$INSTDIR\uninstall.exe"
    Delete "$SMPROGRAMS\{app_name}\{app_name}.lnk"
    Delete "$DESKTOP\{app_name}.lnk"
    RMDir "$SMPROGRAMS\{app_name}"
    RMDir "$INSTDIR"
SectionEnd
"#
    )
}
