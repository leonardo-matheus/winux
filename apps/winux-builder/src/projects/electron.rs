// Electron project detection and building

use super::{ProjectInfo, ProjectType};
use anyhow::Result;
use std::path::Path;

/// Detect an Electron project from package.json
pub fn detect(path: &Path) -> Option<ProjectInfo> {
    let package_json = path.join("package.json");

    if !package_json.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&package_json).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Check if it's an Electron project
    let is_electron = parsed
        .get("devDependencies")
        .and_then(|d| d.get("electron"))
        .is_some()
        || parsed
            .get("dependencies")
            .and_then(|d| d.get("electron"))
            .is_some();

    if !is_electron {
        return None;
    }

    let name = parsed
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("electron-app")
        .to_string();

    let version = parsed
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let description = parsed
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut detected_files = vec!["package.json".to_string()];

    // Check for common Electron files
    for file in &["main.js", "index.js", "electron.js", "package-lock.json", "yarn.lock"] {
        if path.join(file).exists() {
            detected_files.push(file.to_string());
        }
    }

    // Check for electron-builder config
    if path.join("electron-builder.yml").exists() {
        detected_files.push("electron-builder.yml".to_string());
    }
    if path.join("electron-builder.json").exists() {
        detected_files.push("electron-builder.json".to_string());
    }

    Some(ProjectInfo {
        name,
        path: path.to_string_lossy().to_string(),
        project_type: ProjectType::Electron,
        version,
        description,
        detected_files,
    })
}

/// Get build command for Electron project
pub fn build_command(project: &ProjectInfo, target: &str, _release: bool) -> Result<String> {
    // Check for package manager
    let pkg_manager = if Path::new(&project.path).join("yarn.lock").exists() {
        "yarn"
    } else if Path::new(&project.path).join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else {
        "npm"
    };

    let cmd = match target {
        // Linux targets
        "deb" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --linux deb",
            project.path, pkg_manager, pkg_manager
        ),
        "rpm" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --linux rpm",
            project.path, pkg_manager, pkg_manager
        ),
        "appimage" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --linux AppImage",
            project.path, pkg_manager, pkg_manager
        ),
        "snap" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --linux snap",
            project.path, pkg_manager, pkg_manager
        ),

        // Windows targets
        "exe" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --win portable",
            project.path, pkg_manager, pkg_manager
        ),
        "msi" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --win msi",
            project.path, pkg_manager, pkg_manager
        ),
        "nsis" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --win nsis",
            project.path, pkg_manager, pkg_manager
        ),

        // macOS targets
        "dmg" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --mac dmg",
            project.path, pkg_manager, pkg_manager
        ),
        "mas" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --mac mas",
            project.path, pkg_manager, pkg_manager
        ),

        // All platforms
        "all-linux" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --linux",
            project.path, pkg_manager, pkg_manager
        ),
        "all-win" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --win",
            project.path, pkg_manager, pkg_manager
        ),
        "all-mac" => format!(
            "cd '{}' && {} install && {} run build && electron-builder --mac",
            project.path, pkg_manager, pkg_manager
        ),

        // Development
        "dev" => format!(
            "cd '{}' && {} install && {} run start",
            project.path, pkg_manager, pkg_manager
        ),

        _ => anyhow::bail!("Target nao suportado para Electron: {}", target),
    };

    Ok(cmd)
}

/// Generate electron-builder config
pub fn generate_builder_config(project: &ProjectInfo) -> String {
    format!(
        r#"appId: org.winux.{}
productName: {}
directories:
  output: dist

linux:
  target:
    - AppImage
    - deb
    - rpm
  category: Utility
  icon: build/icons

win:
  target:
    - nsis
    - portable
  icon: build/icon.ico

mac:
  target:
    - dmg
    - zip
  icon: build/icon.icns
  category: public.app-category.utilities

nsis:
  oneClick: false
  allowToChangeInstallationDirectory: true

dmg:
  contents:
    - x: 130
      y: 220
    - x: 410
      y: 220
      type: link
      path: /Applications
"#,
        project.name.to_lowercase().replace(' ', "-"),
        project.name,
    )
}
