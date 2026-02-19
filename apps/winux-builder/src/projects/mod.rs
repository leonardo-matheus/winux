// Project detection and management modules

pub mod rust;
pub mod dotnet;
pub mod electron;
pub mod flutter;

use anyhow::Result;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Information about a detected project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub project_type: ProjectType,
    pub version: Option<String>,
    pub description: Option<String>,
    pub detected_files: Vec<String>,
}

/// Supported project types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    DotNet,
    Electron,
    Flutter,
    Unknown,
}

impl ProjectType {
    pub fn as_str(&self) -> &str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::DotNet => ".NET",
            ProjectType::Electron => "Electron",
            ProjectType::Flutter => "Flutter",
            ProjectType::Unknown => "Desconhecido",
        }
    }

    pub fn icon_name(&self) -> &str {
        match self {
            ProjectType::Rust => "application-x-executable-symbolic",
            ProjectType::DotNet => "application-x-addon-symbolic",
            ProjectType::Electron => "applications-internet-symbolic",
            ProjectType::Flutter => "phone-symbolic",
            ProjectType::Unknown => "dialog-question-symbolic",
        }
    }

    pub fn supported_targets(&self) -> Vec<&str> {
        match self {
            ProjectType::Rust => vec![
                "deb", "rpm", "appimage", "flatpak",
                "exe", "msi",
                "app", "dmg",
            ],
            ProjectType::DotNet => vec![
                "deb", "rpm",
                "exe", "msi",
                "app", "dmg",
            ],
            ProjectType::Electron => vec![
                "deb", "rpm", "appimage", "snap",
                "exe", "msi", "nsis",
                "dmg", "mas",
            ],
            ProjectType::Flutter => vec![
                "apk", "aab",
                "ipa",
                "exe", "msix",
                "deb", "rpm", "appimage",
                "app", "dmg",
                "web",
            ],
            ProjectType::Unknown => vec![],
        }
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detect project type from directory
pub fn detect_project_type(path: &Path) -> Result<ProjectInfo> {
    // Check for Rust project (Cargo.toml)
    if let Some(info) = rust::detect(path) {
        return Ok(info);
    }

    // Check for .NET project (*.csproj, *.fsproj, *.sln)
    if let Some(info) = dotnet::detect(path) {
        return Ok(info);
    }

    // Check for Electron project (package.json with electron)
    if let Some(info) = electron::detect(path) {
        return Ok(info);
    }

    // Check for Flutter project (pubspec.yaml)
    if let Some(info) = flutter::detect(path) {
        return Ok(info);
    }

    // Unknown project
    Ok(ProjectInfo {
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        path: path.to_string_lossy().to_string(),
        project_type: ProjectType::Unknown,
        version: None,
        description: None,
        detected_files: vec![],
    })
}

/// Get build command for a project
pub fn get_build_command(project: &ProjectInfo, target: &str, release: bool) -> Result<String> {
    match project.project_type {
        ProjectType::Rust => rust::build_command(project, target, release),
        ProjectType::DotNet => dotnet::build_command(project, target, release),
        ProjectType::Electron => electron::build_command(project, target, release),
        ProjectType::Flutter => flutter::build_command(project, target, release),
        ProjectType::Unknown => {
            anyhow::bail!("Tipo de projeto desconhecido. Selecione manualmente o tipo de build.")
        }
    }
}

/// Scan directory for projects
pub fn scan_for_projects(path: &Path, max_depth: usize) -> Vec<ProjectInfo> {
    let mut projects = Vec::new();

    // Check root directory
    if let Ok(info) = detect_project_type(path) {
        if info.project_type != ProjectType::Unknown {
            projects.push(info);
        }
    }

    // Scan subdirectories
    if max_depth > 0 {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let name = entry_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Skip hidden and common non-project directories
                    if name.starts_with('.')
                        || name == "node_modules"
                        || name == "target"
                        || name == "build"
                        || name == "bin"
                        || name == "obj"
                    {
                        continue;
                    }

                    projects.extend(scan_for_projects(&entry_path, max_depth - 1));
                }
            }
        }
    }

    projects
}
