// Winux Mobile Studio - AAB Builder
// Copyright (c) 2026 Winux OS Project
//
// Build Android App Bundles (AAB) for Play Store:
// - Generate signed AAB files
// - Bundle tool operations
// - Generate APKs from AAB for testing

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use super::apk_builder::SigningConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AabBuildConfig {
    pub project_path: PathBuf,
    pub flavor: Option<String>,
    pub signing_config: Option<SigningConfig>,
    pub obfuscate: bool,
    pub split_by_abi: bool,
    pub split_by_density: bool,
}

#[derive(Clone, Debug)]
pub struct AabBuildResult {
    pub success: bool,
    pub aab_path: Option<PathBuf>,
    pub build_time_secs: u64,
    pub aab_size_bytes: Option<u64>,
    pub error_message: Option<String>,
}

pub struct AabBuilder {
    android_sdk_path: PathBuf,
    java_home: PathBuf,
    bundletool_path: Option<PathBuf>,
}

impl AabBuilder {
    pub fn new() -> Result<Self> {
        let android_sdk_path = Self::detect_android_sdk()?;
        let java_home = Self::detect_java_home()?;
        let bundletool_path = Self::find_bundletool();

        Ok(Self {
            android_sdk_path,
            java_home,
            bundletool_path,
        })
    }

    fn detect_android_sdk() -> Result<PathBuf> {
        if let Ok(sdk_path) = std::env::var("ANDROID_SDK_ROOT") {
            return Ok(PathBuf::from(sdk_path));
        }
        if let Ok(sdk_path) = std::env::var("ANDROID_HOME") {
            return Ok(PathBuf::from(sdk_path));
        }

        let home = dirs::home_dir().context("Could not find home directory")?;
        let common_paths = vec![
            home.join("Android/Sdk"),
            home.join(".android/sdk"),
            PathBuf::from("/opt/android-sdk"),
        ];

        for path in common_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!("Android SDK not found"))
    }

    fn detect_java_home() -> Result<PathBuf> {
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            return Ok(PathBuf::from(java_home));
        }

        let common_paths = vec![
            PathBuf::from("/usr/lib/jvm/java-17-openjdk"),
            PathBuf::from("/usr/lib/jvm/java-17-openjdk-amd64"),
            PathBuf::from("/usr/lib/jvm/java-11-openjdk"),
        ];

        for path in common_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!("Java not found"))
    }

    fn find_bundletool() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let common_paths = vec![
            home.join(".local/bin/bundletool.jar"),
            home.join("Android/bundletool.jar"),
            PathBuf::from("/usr/local/bin/bundletool.jar"),
            PathBuf::from("/opt/bundletool/bundletool.jar"),
        ];

        for path in common_paths {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    pub async fn build(&self, config: &AabBuildConfig) -> Result<AabBuildResult> {
        let start_time = std::time::Instant::now();

        // Build release bundle task
        let task = match &config.flavor {
            Some(flavor) => format!("bundle{}Release", capitalize(flavor)),
            None => "bundleRelease".to_string(),
        };

        let gradle_wrapper = config.project_path.join("gradlew");
        let gradle_cmd = if gradle_wrapper.exists() {
            gradle_wrapper.to_string_lossy().to_string()
        } else {
            "gradle".to_string()
        };

        let mut cmd = Command::new(&gradle_cmd);
        cmd.current_dir(&config.project_path)
            .arg(&task)
            .env("ANDROID_SDK_ROOT", &self.android_sdk_path)
            .env("JAVA_HOME", &self.java_home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add signing config if provided
        if let Some(signing) = &config.signing_config {
            cmd.arg(format!("-Pandroid.injected.signing.store.file={}", signing.keystore_path.display()))
               .arg(format!("-Pandroid.injected.signing.store.password={}", signing.keystore_password))
               .arg(format!("-Pandroid.injected.signing.key.alias={}", signing.key_alias))
               .arg(format!("-Pandroid.injected.signing.key.password={}", signing.key_password));
        }

        let output = cmd.output().await?;
        let build_time = start_time.elapsed().as_secs();

        if output.status.success() {
            let aab_path = self.find_aab_output(&config)?;
            let aab_size = aab_path.as_ref()
                .and_then(|p| std::fs::metadata(p).ok())
                .map(|m| m.len());

            Ok(AabBuildResult {
                success: true,
                aab_path,
                build_time_secs: build_time,
                aab_size_bytes: aab_size,
                error_message: None,
            })
        } else {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(AabBuildResult {
                success: false,
                aab_path: None,
                build_time_secs: build_time,
                aab_size_bytes: None,
                error_message: Some(error),
            })
        }
    }

    fn find_aab_output(&self, config: &AabBuildConfig) -> Result<Option<PathBuf>> {
        let output_dir = config.project_path
            .join("app")
            .join("build")
            .join("outputs")
            .join("bundle")
            .join("release");

        if output_dir.exists() {
            for entry in std::fs::read_dir(&output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "aab").unwrap_or(false) {
                    return Ok(Some(path));
                }
            }
        }

        Ok(None)
    }

    /// Generate APKs from AAB for testing purposes using bundletool
    pub async fn generate_apks_from_aab(
        &self,
        aab_path: &Path,
        output_path: &Path,
        signing_config: Option<&SigningConfig>,
    ) -> Result<()> {
        let bundletool = self.bundletool_path.as_ref()
            .context("bundletool not found. Please install bundletool.")?;

        let mut cmd = Command::new("java");
        cmd.arg("-jar")
           .arg(bundletool)
           .arg("build-apks")
           .arg(format!("--bundle={}", aab_path.display()))
           .arg(format!("--output={}", output_path.display()))
           .arg("--mode=universal");

        if let Some(signing) = signing_config {
            cmd.arg(format!("--ks={}", signing.keystore_path.display()))
               .arg(format!("--ks-pass=pass:{}", signing.keystore_password))
               .arg(format!("--ks-key-alias={}", signing.key_alias))
               .arg(format!("--key-pass=pass:{}", signing.key_password));
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to generate APKs from AAB: {}", error));
        }

        Ok(())
    }

    /// Install APKs from AAB on connected device
    pub async fn install_apks(
        &self,
        apks_path: &Path,
        device_id: Option<&str>,
    ) -> Result<()> {
        let bundletool = self.bundletool_path.as_ref()
            .context("bundletool not found")?;

        let mut cmd = Command::new("java");
        cmd.arg("-jar")
           .arg(bundletool)
           .arg("install-apks")
           .arg(format!("--apks={}", apks_path.display()));

        if let Some(id) = device_id {
            cmd.arg(format!("--device-id={}", id));
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install APKs: {}", error));
        }

        Ok(())
    }

    /// Get AAB info using bundletool
    pub async fn get_aab_info(&self, aab_path: &Path) -> Result<String> {
        let bundletool = self.bundletool_path.as_ref()
            .context("bundletool not found")?;

        let output = Command::new("java")
            .arg("-jar")
            .arg(bundletool)
            .arg("dump")
            .arg("manifest")
            .arg(format!("--bundle={}", aab_path.display()))
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to get AAB info: {}", error))
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

impl Default for AabBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create AabBuilder")
    }
}
