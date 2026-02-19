// Apple platform builder (macOS, iOS)
// Builds: .app, .dmg, .pkg, .ipa

use super::{Builder, BuildCommand, DependencyStatus};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct AppleBuilder;

impl AppleBuilder {
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
                "xcrun" => "Requer Xcode no macOS".to_string(),
                "create-dmg" => "npm install -g create-dmg".to_string(),
                "pkgbuild" => "Disponivel no macOS com Xcode".to_string(),
                _ => format!("Instale {} para continuar", name),
            },
        }
    }
}

impl Builder for AppleBuilder {
    fn name(&self) -> &str {
        "Apple (macOS/iOS)"
    }

    fn formats(&self) -> &[&str] {
        &["app", "dmg", "pkg", "ipa"]
    }

    fn check_dependencies(&self) -> Result<Vec<DependencyStatus>> {
        Ok(vec![
            Self::check_tool("xcrun"),
            Self::check_tool("create-dmg"),
            Self::check_tool("pkgbuild"),
            Self::check_tool("codesign"),
        ])
    }

    fn build(&self, project_path: &Path, format: &str, release: bool) -> Result<BuildCommand> {
        let project_str = project_path.to_string_lossy();

        match format {
            "app" => {
                // Build .app bundle
                // This is a simplified command - actual implementation depends on project type
                Ok(BuildCommand::new("cargo")
                    .args(&["bundle", "--release"])
                    .working_dir(&project_str)
                    .env("MACOSX_DEPLOYMENT_TARGET", "10.15"))
            }
            "dmg" => {
                // Create DMG from .app bundle
                let mode = if release { "--release" } else { "" };
                let script = format!(
                    r#"
# Build the app first
cargo build {mode}

# Create DMG using create-dmg or hdiutil
APP_NAME=$(basename {project_str})
DMG_NAME="${{APP_NAME}}.dmg"

if command -v create-dmg &> /dev/null; then
    create-dmg \
        --volname "${{APP_NAME}}" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --app-drop-link 450 185 \
        "${{DMG_NAME}}" \
        "target/release/${{APP_NAME}}.app"
else
    # Fallback to hdiutil
    hdiutil create -volname "${{APP_NAME}}" -srcfolder "target/release/${{APP_NAME}}.app" -ov -format UDZO "${{DMG_NAME}}"
fi

echo "DMG criado: ${{DMG_NAME}}"
"#
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            "pkg" => {
                // Create PKG installer
                let mode = if release { "--release" } else { "" };
                let script = format!(
                    r#"
# Build first
cargo build {mode}

APP_NAME=$(basename {project_str})
PKG_NAME="${{APP_NAME}}.pkg"

pkgbuild \
    --root "target/release/${{APP_NAME}}.app" \
    --identifier "org.winux.${{APP_NAME}}" \
    --version "1.0.0" \
    --install-location "/Applications" \
    "${{PKG_NAME}}"

echo "PKG criado: ${{PKG_NAME}}"
"#
                );

                Ok(BuildCommand::new("bash")
                    .args(&["-c", &script])
                    .working_dir(&project_str))
            }
            "ipa" => {
                // Build iOS IPA (requires Xcode and valid signing)
                let script = r#"
# Requires Xcode and valid iOS signing certificate
if ! command -v xcrun &> /dev/null; then
    echo "Erro: xcrun nao encontrado. Instale o Xcode."
    exit 1
fi

# Archive the app
xcrun xcodebuild archive \
    -scheme "$(basename $(pwd))" \
    -archivePath "build/archive.xcarchive"

# Export IPA
xcrun xcodebuild -exportArchive \
    -archivePath "build/archive.xcarchive" \
    -exportPath "build/ipa" \
    -exportOptionsPlist "ExportOptions.plist"

echo "IPA criado em build/ipa/"
"#;

                Ok(BuildCommand::new("bash")
                    .args(&["-c", script])
                    .working_dir(&project_str))
            }
            _ => {
                anyhow::bail!("Formato desconhecido para Apple: {}", format)
            }
        }
    }
}
