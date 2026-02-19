// Winux Mobile Studio - APK Builder
// Copyright (c) 2026 Winux OS Project
//
// Build APK files from Android projects:
// - Gradle-based builds
// - Debug and release variants
// - Signing with keystore
// - Installing on device via ADB

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApkBuildConfig {
    pub project_path: PathBuf,
    pub build_type: ApkBuildType,
    pub flavor: Option<String>,
    pub signing_config: Option<SigningConfig>,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ApkBuildType {
    Debug,
    Release,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigningConfig {
    pub keystore_path: PathBuf,
    pub keystore_password: String,
    pub key_alias: String,
    pub key_password: String,
}

#[derive(Clone, Debug)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: Option<PathBuf>,
    pub build_time_secs: u64,
    pub apk_size_bytes: Option<u64>,
    pub error_message: Option<String>,
}

pub struct ApkBuilder {
    android_sdk_path: PathBuf,
    java_home: PathBuf,
}

impl ApkBuilder {
    pub fn new() -> Result<Self> {
        let android_sdk_path = Self::detect_android_sdk()?;
        let java_home = Self::detect_java_home()?;

        Ok(Self {
            android_sdk_path,
            java_home,
        })
    }

    fn detect_android_sdk() -> Result<PathBuf> {
        // Check environment variable first
        if let Ok(sdk_path) = std::env::var("ANDROID_SDK_ROOT") {
            return Ok(PathBuf::from(sdk_path));
        }
        if let Ok(sdk_path) = std::env::var("ANDROID_HOME") {
            return Ok(PathBuf::from(sdk_path));
        }

        // Check common locations
        let home = dirs::home_dir().context("Could not find home directory")?;
        let common_paths = vec![
            home.join("Android/Sdk"),
            home.join(".android/sdk"),
            PathBuf::from("/opt/android-sdk"),
            PathBuf::from("/usr/lib/android-sdk"),
        ];

        for path in common_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!("Android SDK not found. Please set ANDROID_SDK_ROOT environment variable."))
    }

    fn detect_java_home() -> Result<PathBuf> {
        // Check environment variable first
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            return Ok(PathBuf::from(java_home));
        }

        // Check common locations
        let common_paths = vec![
            PathBuf::from("/usr/lib/jvm/java-17-openjdk"),
            PathBuf::from("/usr/lib/jvm/java-17-openjdk-amd64"),
            PathBuf::from("/usr/lib/jvm/java-11-openjdk"),
            PathBuf::from("/usr/lib/jvm/default-java"),
        ];

        for path in common_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!("Java not found. Please set JAVA_HOME environment variable."))
    }

    pub async fn build(&self, config: &ApkBuildConfig) -> Result<BuildResult> {
        let start_time = std::time::Instant::now();

        // Determine gradle task
        let task = self.get_gradle_task(config);

        // Run gradle build
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
            let output_path = self.find_apk_output(&config)?;
            let apk_size = output_path.as_ref()
                .and_then(|p| std::fs::metadata(p).ok())
                .map(|m| m.len());

            Ok(BuildResult {
                success: true,
                output_path,
                build_time_secs: build_time,
                apk_size_bytes: apk_size,
                error_message: None,
            })
        } else {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(BuildResult {
                success: false,
                output_path: None,
                build_time_secs: build_time,
                apk_size_bytes: None,
                error_message: Some(error),
            })
        }
    }

    fn get_gradle_task(&self, config: &ApkBuildConfig) -> String {
        let build_type = match config.build_type {
            ApkBuildType::Debug => "Debug",
            ApkBuildType::Release => "Release",
        };

        match &config.flavor {
            Some(flavor) => format!("assemble{}{}", capitalize(flavor), build_type),
            None => format!("assemble{}", build_type),
        }
    }

    fn find_apk_output(&self, config: &ApkBuildConfig) -> Result<Option<PathBuf>> {
        let build_type_str = match config.build_type {
            ApkBuildType::Debug => "debug",
            ApkBuildType::Release => "release",
        };

        let output_dir = config.project_path
            .join("app")
            .join("build")
            .join("outputs")
            .join("apk")
            .join(build_type_str);

        if output_dir.exists() {
            for entry in std::fs::read_dir(&output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "apk").unwrap_or(false) {
                    return Ok(Some(path));
                }
            }
        }

        Ok(None)
    }

    pub async fn clean(&self, project_path: &Path) -> Result<()> {
        let gradle_wrapper = project_path.join("gradlew");
        let gradle_cmd = if gradle_wrapper.exists() {
            gradle_wrapper.to_string_lossy().to_string()
        } else {
            "gradle".to_string()
        };

        Command::new(&gradle_cmd)
            .current_dir(project_path)
            .arg("clean")
            .env("ANDROID_SDK_ROOT", &self.android_sdk_path)
            .env("JAVA_HOME", &self.java_home)
            .output()
            .await?;

        Ok(())
    }

    pub async fn install_on_device(&self, apk_path: &Path, device_id: Option<&str>) -> Result<()> {
        let adb_path = self.android_sdk_path.join("platform-tools").join("adb");

        let mut cmd = Command::new(&adb_path);

        if let Some(id) = device_id {
            cmd.arg("-s").arg(id);
        }

        cmd.arg("install")
           .arg("-r") // Replace existing
           .arg(apk_path);

        let output = cmd.output().await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install APK: {}", error));
        }

        Ok(())
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

impl Default for ApkBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create ApkBuilder")
    }
}
