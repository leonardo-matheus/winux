// .NET project detection and building

use super::{ProjectInfo, ProjectType};
use anyhow::Result;
use std::path::Path;

/// Detect a .NET project
pub fn detect(path: &Path) -> Option<ProjectInfo> {
    // Look for .csproj, .fsproj, or .sln files
    let mut detected_files = Vec::new();
    let mut project_file = None;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".csproj") || name.ends_with(".fsproj") {
                detected_files.push(name.clone());
                if project_file.is_none() {
                    project_file = Some(entry.path());
                }
            } else if name.ends_with(".sln") {
                detected_files.push(name);
            }
        }
    }

    let project_file = project_file?;
    let content = std::fs::read_to_string(&project_file).ok()?;

    // Extract project name from file
    let name = project_file
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "DotNetProject".to_string());

    // Try to extract version from project file
    let version = extract_property(&content, "Version")
        .or_else(|| extract_property(&content, "AssemblyVersion"));

    let description = extract_property(&content, "Description");

    Some(ProjectInfo {
        name,
        path: path.to_string_lossy().to_string(),
        project_type: ProjectType::DotNet,
        version,
        description,
        detected_files,
    })
}

/// Extract property from csproj/fsproj XML
fn extract_property(content: &str, property: &str) -> Option<String> {
    let start_tag = format!("<{}>", property);
    let end_tag = format!("</{}>", property);

    let start = content.find(&start_tag)?;
    let end = content.find(&end_tag)?;

    if start < end {
        let value_start = start + start_tag.len();
        Some(content[value_start..end].trim().to_string())
    } else {
        None
    }
}

/// Get build command for .NET project
pub fn build_command(project: &ProjectInfo, target: &str, release: bool) -> Result<String> {
    let config = if release { "Release" } else { "Debug" };

    let cmd = match target {
        // Linux targets
        "deb" => format!(
            r#"cd '{}' && \
dotnet publish -c {} -r linux-x64 --self-contained true && \
mkdir -p deb/DEBIAN deb/usr/bin deb/usr/share/applications && \
cp -r bin/{}/net*/linux-x64/publish/* deb/usr/bin/ && \
cat > deb/DEBIAN/control << EOF
Package: {}
Version: {}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Winux Builder
Description: {}
EOF
dpkg-deb --build deb {}.deb"#,
            project.path,
            config,
            config,
            project.name.to_lowercase().replace(' ', "-"),
            project.version.as_deref().unwrap_or("1.0.0"),
            project.description.as_deref().unwrap_or(&project.name),
            project.name,
        ),
        "rpm" => format!(
            "cd '{}' && dotnet publish -c {} -r linux-x64 --self-contained true",
            project.path, config
        ),

        // Windows targets
        "exe" => format!(
            "cd '{}' && dotnet publish -c {} -r win-x64 --self-contained true -p:PublishSingleFile=true",
            project.path, config
        ),
        "msi" => format!(
            "cd '{}' && dotnet publish -c {} -r win-x64 --self-contained true && \
echo 'MSI creation requires WiX Toolset or similar tool'",
            project.path, config
        ),

        // macOS targets
        "app" => format!(
            "cd '{}' && dotnet publish -c {} -r osx-x64 --self-contained true",
            project.path, config
        ),
        "dmg" => format!(
            "cd '{}' && dotnet publish -c {} -r osx-x64 --self-contained true && \
echo 'DMG creation requires create-dmg or similar tool'",
            project.path, config
        ),

        // Native build
        "native" | "" => format!(
            "cd '{}' && dotnet build -c {}",
            project.path, config
        ),

        // Publish
        "publish" => format!(
            "cd '{}' && dotnet publish -c {}",
            project.path, config
        ),

        _ => anyhow::bail!("Target nao suportado para .NET: {}", target),
    };

    Ok(cmd)
}

/// Get .NET runtime identifiers
pub fn runtime_identifiers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("linux-x64", "Linux x64"),
        ("linux-arm64", "Linux ARM64"),
        ("win-x64", "Windows x64"),
        ("win-x86", "Windows x86"),
        ("win-arm64", "Windows ARM64"),
        ("osx-x64", "macOS Intel"),
        ("osx-arm64", "macOS Apple Silicon"),
    ]
}
