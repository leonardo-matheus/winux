// Flutter project detection and building

use super::{ProjectInfo, ProjectType};
use anyhow::Result;
use std::path::Path;

/// Detect a Flutter project from pubspec.yaml
pub fn detect(path: &Path) -> Option<ProjectInfo> {
    let pubspec = path.join("pubspec.yaml");

    if !pubspec.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&pubspec).ok()?;

    // Check if it uses Flutter SDK
    if !content.contains("flutter:") && !content.contains("sdk: flutter") {
        return None;
    }

    // Parse YAML manually (simple extraction)
    let name = extract_yaml_value(&content, "name")
        .unwrap_or_else(|| "flutter_app".to_string());

    let version = extract_yaml_value(&content, "version");
    let description = extract_yaml_value(&content, "description");

    let mut detected_files = vec!["pubspec.yaml".to_string()];

    // Check for common Flutter files/directories
    for item in &["lib/main.dart", "pubspec.lock", "android", "ios", "linux", "windows", "macos", "web"] {
        if path.join(item).exists() {
            detected_files.push(item.to_string());
        }
    }

    Some(ProjectInfo {
        name,
        path: path.to_string_lossy().to_string(),
        project_type: ProjectType::Flutter,
        version,
        description,
        detected_files,
    })
}

/// Simple YAML value extraction
fn extract_yaml_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{}:", key)) {
            let value = trimmed
                .strip_prefix(&format!("{}:", key))?
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// Get build command for Flutter project
pub fn build_command(project: &ProjectInfo, target: &str, release: bool) -> Result<String> {
    let mode = if release { "--release" } else { "--debug" };

    let cmd = match target {
        // Android targets
        "apk" => format!(
            "cd '{}' && flutter build apk {}",
            project.path, mode
        ),
        "aab" => format!(
            "cd '{}' && flutter build appbundle {}",
            project.path, mode
        ),

        // iOS target
        "ipa" => format!(
            "cd '{}' && flutter build ipa {}",
            project.path, mode
        ),

        // Linux targets
        "linux" => format!(
            "cd '{}' && flutter build linux {}",
            project.path, mode
        ),
        "deb" => format!(
            r#"cd '{}' && flutter build linux {} && \
APP_NAME="{}" && \
VERSION="{}" && \
mkdir -p deb/DEBIAN deb/usr/bin deb/usr/share/applications deb/usr/share/icons/hicolor/256x256/apps && \
cp -r build/linux/x64/release/bundle/* deb/usr/bin/ && \
cat > deb/DEBIAN/control << EOF
Package: ${{APP_NAME}}
Version: ${{VERSION}}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Winux Builder
Description: Flutter application
EOF
cat > deb/usr/share/applications/${{APP_NAME}}.desktop << EOF
[Desktop Entry]
Type=Application
Name=${{APP_NAME}}
Exec=/usr/bin/${{APP_NAME}}
Icon=${{APP_NAME}}
Categories=Utility;
EOF
dpkg-deb --build deb ${{APP_NAME}}.deb"#,
            project.path,
            mode,
            project.name,
            project.version.as_deref().unwrap_or("1.0.0"),
        ),
        "rpm" => format!(
            "cd '{}' && flutter build linux {}",
            project.path, mode
        ),
        "appimage" => format!(
            r#"cd '{}' && flutter build linux {} && \
APP_NAME="{}" && \
mkdir -p AppDir/usr/bin && \
cp -r build/linux/x64/release/bundle/* AppDir/usr/bin/ && \
cat > AppDir/${{APP_NAME}}.desktop << EOF
[Desktop Entry]
Type=Application
Name=${{APP_NAME}}
Exec=${{APP_NAME}}
Icon=${{APP_NAME}}
Categories=Utility;
EOF
cat > AppDir/AppRun << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${{SELF%/*}}
export LD_LIBRARY_PATH="${{HERE}}/usr/bin/lib:${{LD_LIBRARY_PATH}}"
exec "${{HERE}}/usr/bin/$(basename ${{APPDIR}})" "$@"
EOF
chmod +x AppDir/AppRun && \
ARCH=x86_64 appimagetool AppDir"#,
            project.path,
            mode,
            project.name,
        ),

        // Windows targets
        "windows" | "exe" => format!(
            "cd '{}' && flutter build windows {}",
            project.path, mode
        ),
        "msix" => format!(
            "cd '{}' && flutter pub run msix:create",
            project.path
        ),

        // macOS targets
        "macos" | "app" => format!(
            "cd '{}' && flutter build macos {}",
            project.path, mode
        ),
        "dmg" => format!(
            "cd '{}' && flutter build macos {} && \
create-dmg build/macos/Build/Products/Release/*.app",
            project.path, mode
        ),

        // Web target
        "web" => format!(
            "cd '{}' && flutter build web {}",
            project.path, mode
        ),

        // Clean and get
        "clean" => format!(
            "cd '{}' && flutter clean && flutter pub get",
            project.path
        ),

        _ => anyhow::bail!("Target nao suportado para Flutter: {}", target),
    };

    Ok(cmd)
}

/// Get available Flutter targets based on platform directories
pub fn available_targets(project_path: &Path) -> Vec<(&'static str, &'static str, bool)> {
    vec![
        ("android", "Android (APK/AAB)", project_path.join("android").exists()),
        ("ios", "iOS (IPA)", project_path.join("ios").exists()),
        ("linux", "Linux Desktop", project_path.join("linux").exists()),
        ("windows", "Windows Desktop", project_path.join("windows").exists()),
        ("macos", "macOS Desktop", project_path.join("macos").exists()),
        ("web", "Web Application", project_path.join("web").exists()),
    ]
}

/// Enable Flutter platform support
pub fn enable_platform(platform: &str) -> String {
    format!("flutter config --enable-{}-desktop && flutter create --platforms={} .", platform, platform)
}
