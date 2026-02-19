// Winux Mobile Studio - IPA Builder
// Copyright (c) 2026 Winux OS Project
//
// IPA building for iOS (limited without Mac):
// - Re-sign existing IPAs
// - Create .deb packages for jailbreak
// - Install via libimobiledevice
// - Theos project building

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpaBuildConfig {
    pub project_path: PathBuf,
    pub bundle_id: String,
    pub signing_identity: Option<String>,
    pub provisioning_profile: Option<PathBuf>,
    pub target_ios_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebBuildConfig {
    pub project_path: PathBuf,
    pub package_name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub depends: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct IpaBuildResult {
    pub success: bool,
    pub output_path: Option<PathBuf>,
    pub build_time_secs: u64,
    pub error_message: Option<String>,
}

pub struct IpaBuilder {
    theos_path: Option<PathBuf>,
    ldid_path: Option<PathBuf>,
    dpkg_path: Option<PathBuf>,
}

impl IpaBuilder {
    pub fn new() -> Self {
        let theos_path = Self::find_theos();
        let ldid_path = Self::find_binary("ldid");
        let dpkg_path = Self::find_binary("dpkg-deb");

        Self {
            theos_path,
            ldid_path,
            dpkg_path,
        }
    }

    fn find_theos() -> Option<PathBuf> {
        // Check THEOS environment variable
        if let Ok(theos) = std::env::var("THEOS") {
            let path = PathBuf::from(theos);
            if path.exists() {
                return Some(path);
            }
        }

        // Check common locations
        let home = dirs::home_dir()?;
        let common_paths = vec![
            home.join("theos"),
            home.join(".theos"),
            PathBuf::from("/opt/theos"),
            PathBuf::from("/usr/local/theos"),
        ];

        for path in common_paths {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    fn find_binary(name: &str) -> Option<PathBuf> {
        let common_paths = vec![
            PathBuf::from(format!("/usr/bin/{}", name)),
            PathBuf::from(format!("/usr/local/bin/{}", name)),
            PathBuf::from(format!("/opt/bin/{}", name)),
        ];

        for path in common_paths {
            if path.exists() {
                return Some(path);
            }
        }

        // Try using which
        if let Ok(output) = std::process::Command::new("which")
            .arg(name)
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }

        None
    }

    /// Check if required tools are available
    pub fn check_tools(&self) -> ToolsStatus {
        ToolsStatus {
            theos_available: self.theos_path.is_some(),
            ldid_available: self.ldid_path.is_some(),
            dpkg_available: self.dpkg_path.is_some(),
            libimobiledevice_available: Self::find_binary("ideviceinstaller").is_some(),
        }
    }

    /// Re-sign an existing IPA with a new identity
    pub async fn resign_ipa(
        &self,
        ipa_path: &Path,
        output_path: &Path,
        signing_identity: Option<&str>,
    ) -> Result<IpaBuildResult> {
        let start_time = std::time::Instant::now();

        let ldid = self.ldid_path.as_ref()
            .context("ldid not found. Please install ldid for IPA signing.")?;

        // Extract IPA
        let temp_dir = tempfile::tempdir()?;
        let extract_output = Command::new("unzip")
            .arg("-q")
            .arg(ipa_path)
            .arg("-d")
            .arg(temp_dir.path())
            .output()
            .await?;

        if !extract_output.status.success() {
            return Ok(IpaBuildResult {
                success: false,
                output_path: None,
                build_time_secs: start_time.elapsed().as_secs(),
                error_message: Some("Failed to extract IPA".to_string()),
            });
        }

        // Find the .app bundle
        let payload_dir = temp_dir.path().join("Payload");
        let app_dir = std::fs::read_dir(&payload_dir)?
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().map(|ext| ext == "app").unwrap_or(false))
            .map(|e| e.path())
            .context("No .app bundle found in IPA")?;

        // Find the main executable
        let executable_name = app_dir.file_stem()
            .context("Invalid app bundle")?
            .to_string_lossy();
        let executable_path = app_dir.join(&*executable_name);

        // Sign with ldid
        let mut sign_cmd = Command::new(ldid);
        sign_cmd.arg("-S");

        if let Some(identity) = signing_identity {
            sign_cmd.arg(format!("-K{}", identity));
        }

        sign_cmd.arg(&executable_path);
        let sign_output = sign_cmd.output().await?;

        if !sign_output.status.success() {
            return Ok(IpaBuildResult {
                success: false,
                output_path: None,
                build_time_secs: start_time.elapsed().as_secs(),
                error_message: Some(format!("Failed to sign: {}", String::from_utf8_lossy(&sign_output.stderr))),
            });
        }

        // Repackage IPA
        let zip_output = Command::new("zip")
            .arg("-r")
            .arg("-q")
            .arg(output_path)
            .arg("Payload")
            .current_dir(temp_dir.path())
            .output()
            .await?;

        let build_time = start_time.elapsed().as_secs();

        if zip_output.status.success() {
            Ok(IpaBuildResult {
                success: true,
                output_path: Some(output_path.to_path_buf()),
                build_time_secs: build_time,
                error_message: None,
            })
        } else {
            Ok(IpaBuildResult {
                success: false,
                output_path: None,
                build_time_secs: build_time,
                error_message: Some("Failed to create IPA".to_string()),
            })
        }
    }

    /// Build a Theos tweak project
    pub async fn build_theos_project(&self, project_path: &Path) -> Result<IpaBuildResult> {
        let start_time = std::time::Instant::now();

        let theos = self.theos_path.as_ref()
            .context("Theos not found. Please install Theos.")?;

        let output = Command::new("make")
            .current_dir(project_path)
            .env("THEOS", theos)
            .env("THEOS_MAKE_PATH", theos.join("makefiles"))
            .arg("package")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let build_time = start_time.elapsed().as_secs();

        if output.status.success() {
            // Find the generated .deb file
            let packages_dir = project_path.join("packages");
            let deb_path = if packages_dir.exists() {
                std::fs::read_dir(&packages_dir)?
                    .filter_map(|e| e.ok())
                    .find(|e| e.path().extension().map(|ext| ext == "deb").unwrap_or(false))
                    .map(|e| e.path())
            } else {
                None
            };

            Ok(IpaBuildResult {
                success: true,
                output_path: deb_path,
                build_time_secs: build_time,
                error_message: None,
            })
        } else {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(IpaBuildResult {
                success: false,
                output_path: None,
                build_time_secs: build_time,
                error_message: Some(error),
            })
        }
    }

    /// Create a new Theos project
    pub async fn create_theos_project(
        &self,
        output_path: &Path,
        project_type: TheosProjectType,
        project_name: &str,
        bundle_id: &str,
    ) -> Result<()> {
        let theos = self.theos_path.as_ref()
            .context("Theos not found")?;

        std::fs::create_dir_all(output_path)?;

        // Generate Makefile
        let makefile_content = self.generate_theos_makefile(&project_type, project_name);
        std::fs::write(output_path.join("Makefile"), makefile_content)?;

        // Generate control file
        let control_content = self.generate_control_file(project_name, bundle_id);
        let control_dir = output_path.join("control");
        std::fs::create_dir_all(&control_dir)?;
        std::fs::write(control_dir.join("control"), control_content)?;

        // Generate main source file
        let source_content = self.generate_source_file(&project_type);
        std::fs::write(output_path.join("Tweak.x"), source_content)?;

        Ok(())
    }

    fn generate_theos_makefile(&self, project_type: &TheosProjectType, name: &str) -> String {
        let template = match project_type {
            TheosProjectType::Tweak => "TWEAK",
            TheosProjectType::Application => "APPLICATION",
            TheosProjectType::PreferenceBundle => "BUNDLE",
            TheosProjectType::Tool => "TOOL",
            TheosProjectType::Library => "LIBRARY",
        };

        format!(
            r#"THEOS_DEVICE_IP = localhost
THEOS_DEVICE_PORT = 2222
ARCHS = arm64
TARGET = iphone:clang:latest:14.0
INSTALL_TARGET_PROCESSES = SpringBoard

include $(THEOS)/makefiles/common.mk

{}_NAME = {}
{}_FILES = Tweak.x
{}_CFLAGS = -fobjc-arc

include $(THEOS_MAKE_PATH)/{}.mk
"#,
            template, name, template, template, template.to_lowercase()
        )
    }

    fn generate_control_file(&self, name: &str, bundle_id: &str) -> String {
        format!(
            r#"Package: {}
Name: {}
Version: 0.0.1
Architecture: iphoneos-arm
Description: A Theos tweak
Maintainer: Developer
Author: Developer
Section: Tweaks
Depends: mobilesubstrate
"#,
            bundle_id, name
        )
    }

    fn generate_source_file(&self, project_type: &TheosProjectType) -> String {
        match project_type {
            TheosProjectType::Tweak => {
                r#"// Tweak.x
// Generated by Winux Mobile Studio

%hook SpringBoard

- (void)applicationDidFinishLaunching:(id)application {
    %orig;
    NSLog(@"[MyTweak] SpringBoard launched!");
}

%end
"#.to_string()
            }
            _ => "// Main source file\n".to_string(),
        }
    }

    /// Install .deb on device via SSH
    pub async fn install_deb_on_device(
        &self,
        deb_path: &Path,
        device_ip: &str,
        device_port: u16,
    ) -> Result<()> {
        // Copy deb to device
        let scp_output = Command::new("scp")
            .arg("-P")
            .arg(device_port.to_string())
            .arg(deb_path)
            .arg(format!("root@{}:/var/mobile/", device_ip))
            .output()
            .await?;

        if !scp_output.status.success() {
            return Err(anyhow::anyhow!("Failed to copy deb to device"));
        }

        // Install via dpkg
        let deb_name = deb_path.file_name().unwrap().to_string_lossy();
        let ssh_output = Command::new("ssh")
            .arg("-p")
            .arg(device_port.to_string())
            .arg(format!("root@{}", device_ip))
            .arg(format!("dpkg -i /var/mobile/{} && uicache --all", deb_name))
            .output()
            .await?;

        if !ssh_output.status.success() {
            let error = String::from_utf8_lossy(&ssh_output.stderr);
            return Err(anyhow::anyhow!("Failed to install deb: {}", error));
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ToolsStatus {
    pub theos_available: bool,
    pub ldid_available: bool,
    pub dpkg_available: bool,
    pub libimobiledevice_available: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TheosProjectType {
    Tweak,
    Application,
    PreferenceBundle,
    Tool,
    Library,
}

impl Default for IpaBuilder {
    fn default() -> Self {
        Self::new()
    }
}
