// Rust project detection and building

use super::{ProjectInfo, ProjectType};
use anyhow::Result;
use std::path::Path;

/// Detect a Rust project from Cargo.toml
pub fn detect(path: &Path) -> Option<ProjectInfo> {
    let cargo_toml = path.join("Cargo.toml");

    if !cargo_toml.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&cargo_toml).ok()?;
    let parsed: toml::Value = toml::from_str(&content).ok()?;

    let package = parsed.get("package")?;

    let name = package
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let version = package
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let description = package
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut detected_files = vec!["Cargo.toml".to_string()];

    // Check for additional Rust files
    if path.join("Cargo.lock").exists() {
        detected_files.push("Cargo.lock".to_string());
    }
    if path.join("src/main.rs").exists() {
        detected_files.push("src/main.rs".to_string());
    }
    if path.join("src/lib.rs").exists() {
        detected_files.push("src/lib.rs".to_string());
    }

    Some(ProjectInfo {
        name,
        path: path.to_string_lossy().to_string(),
        project_type: ProjectType::Rust,
        version,
        description,
        detected_files,
    })
}

/// Get build command for Rust project
pub fn build_command(project: &ProjectInfo, target: &str, release: bool) -> Result<String> {
    let mode = if release { "--release" } else { "" };

    let cmd = match target {
        // Linux targets
        "deb" => format!(
            "cd '{}' && cargo deb {}",
            project.path, mode
        ),
        "rpm" => format!(
            "cd '{}' && cargo rpm build",
            project.path
        ),
        "appimage" => {
            format!(
                r#"cd '{}' && cargo build {} && \
APP_NAME="{}" && \
mkdir -p AppDir/usr/bin && \
cp target/{}/{} AppDir/usr/bin/ && \
cat > AppDir/{}.desktop << EOF
[Desktop Entry]
Type=Application
Name={}
Exec={}
Icon={}
Categories=Utility;
EOF
ln -sf {}.desktop AppDir/{}.desktop && \
ARCH=x86_64 appimagetool AppDir"#,
                project.path,
                mode,
                project.name,
                if release { "release" } else { "debug" },
                project.name,
                project.name,
                project.name,
                project.name,
                project.name,
                project.name,
                project.name,
            )
        }
        "flatpak" => format!(
            "cd '{}' && cargo build {} && flatpak-builder --force-clean build-dir org.winux.{}.yml",
            project.path, mode, project.name
        ),

        // Windows targets
        "exe" => format!(
            "cd '{}' && rustup target add x86_64-pc-windows-gnu && cargo build {} --target x86_64-pc-windows-gnu",
            project.path, mode
        ),
        "msi" => format!(
            "cd '{}' && cargo build {} --target x86_64-pc-windows-gnu && cargo wix",
            project.path, mode
        ),

        // macOS targets
        "app" => format!(
            "cd '{}' && cargo bundle {}",
            project.path, mode
        ),
        "dmg" => format!(
            "cd '{}' && cargo bundle {} && create-dmg target/{}/bundle/osx/{}.app",
            project.path, mode,
            if release { "release" } else { "debug" },
            project.name
        ),

        // Native build
        "native" | "" => format!(
            "cd '{}' && cargo build {}",
            project.path, mode
        ),

        _ => anyhow::bail!("Target nao suportado para Rust: {}", target),
    };

    Ok(cmd)
}

/// Check if cargo extension is available
pub fn check_extension(name: &str) -> bool {
    std::process::Command::new("cargo")
        .arg(name)
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get available cargo extensions for building
pub fn available_extensions() -> Vec<(&'static str, &'static str, bool)> {
    vec![
        ("cargo-deb", "DEB packages", check_extension("deb")),
        ("cargo-rpm", "RPM packages", check_extension("rpm")),
        ("cargo-bundle", "macOS .app bundles", check_extension("bundle")),
        ("cargo-wix", "Windows MSI", check_extension("wix")),
        ("cargo-appimage", "AppImage", check_extension("appimage")),
    ]
}
