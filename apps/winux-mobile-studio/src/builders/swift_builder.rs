// Winux Mobile Studio - Swift Builder
// Copyright (c) 2026 Winux OS Project
//
// Compile Swift projects for Linux:
// - Swift Package Manager integration
// - Build executables and libraries
// - Run Swift tests

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwiftBuildConfig {
    pub project_path: PathBuf,
    pub configuration: SwiftConfiguration,
    pub target: Option<String>,
    pub verbose: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SwiftConfiguration {
    Debug,
    Release,
}

#[derive(Clone, Debug)]
pub struct SwiftBuildResult {
    pub success: bool,
    pub output_path: Option<PathBuf>,
    pub build_time_secs: u64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwiftPackageInfo {
    pub name: String,
    pub targets: Vec<String>,
    pub dependencies: Vec<SwiftDependency>,
    pub products: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwiftDependency {
    pub name: String,
    pub url: Option<String>,
    pub version: Option<String>,
}

pub struct SwiftBuilder {
    swift_path: PathBuf,
}

impl SwiftBuilder {
    pub fn new() -> Result<Self> {
        let swift_path = Self::find_swift()?;

        Ok(Self { swift_path })
    }

    fn find_swift() -> Result<PathBuf> {
        let common_paths = vec![
            PathBuf::from("/usr/bin/swift"),
            PathBuf::from("/usr/local/bin/swift"),
            PathBuf::from("/opt/swift/bin/swift"),
        ];

        for path in common_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        // Try using which
        let output = std::process::Command::new("which")
            .arg("swift")
            .output()?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }

        Err(anyhow::anyhow!("Swift not found. Please install Swift for Linux."))
    }

    /// Get Swift version
    pub async fn get_version(&self) -> Result<String> {
        let output = Command::new(&self.swift_path)
            .arg("--version")
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Build a Swift package
    pub async fn build(&self, config: &SwiftBuildConfig) -> Result<SwiftBuildResult> {
        let start_time = std::time::Instant::now();

        let mut cmd = Command::new(&self.swift_path);
        cmd.current_dir(&config.project_path)
            .arg("build");

        match config.configuration {
            SwiftConfiguration::Debug => {}
            SwiftConfiguration::Release => {
                cmd.arg("-c").arg("release");
            }
        }

        if let Some(target) = &config.target {
            cmd.arg("--target").arg(target);
        }

        if config.verbose {
            cmd.arg("-v");
        }

        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let output = cmd.output().await?;
        let build_time = start_time.elapsed().as_secs();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let warnings: Vec<String> = stderr
            .lines()
            .filter(|line| line.contains("warning:"))
            .map(|s| s.to_string())
            .collect();

        let errors: Vec<String> = stderr
            .lines()
            .filter(|line| line.contains("error:"))
            .map(|s| s.to_string())
            .collect();

        if output.status.success() {
            let output_path = self.find_build_output(&config)?;

            Ok(SwiftBuildResult {
                success: true,
                output_path,
                build_time_secs: build_time,
                warnings,
                errors: vec![],
            })
        } else {
            Ok(SwiftBuildResult {
                success: false,
                output_path: None,
                build_time_secs: build_time,
                warnings,
                errors,
            })
        }
    }

    fn find_build_output(&self, config: &SwiftBuildConfig) -> Result<Option<PathBuf>> {
        let config_dir = match config.configuration {
            SwiftConfiguration::Debug => "debug",
            SwiftConfiguration::Release => "release",
        };

        let build_dir = config.project_path.join(".build").join(config_dir);

        if build_dir.exists() {
            // Find the main executable
            for entry in std::fs::read_dir(&build_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && !path.extension().is_some() {
                    // Likely an executable
                    return Ok(Some(path));
                }
            }
        }

        Ok(None)
    }

    /// Run tests for a Swift package
    pub async fn test(&self, project_path: &Path, filter: Option<&str>) -> Result<SwiftBuildResult> {
        let start_time = std::time::Instant::now();

        let mut cmd = Command::new(&self.swift_path);
        cmd.current_dir(project_path)
            .arg("test");

        if let Some(f) = filter {
            cmd.arg("--filter").arg(f);
        }

        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let output = cmd.output().await?;
        let build_time = start_time.elapsed().as_secs();

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let errors: Vec<String> = stderr
            .lines()
            .filter(|line| line.contains("FAIL") || line.contains("error:"))
            .map(|s| s.to_string())
            .collect();

        Ok(SwiftBuildResult {
            success: output.status.success(),
            output_path: None,
            build_time_secs: build_time,
            warnings: vec![],
            errors,
        })
    }

    /// Clean build artifacts
    pub async fn clean(&self, project_path: &Path) -> Result<()> {
        Command::new(&self.swift_path)
            .current_dir(project_path)
            .arg("package")
            .arg("clean")
            .output()
            .await?;

        Ok(())
    }

    /// Resolve package dependencies
    pub async fn resolve_dependencies(&self, project_path: &Path) -> Result<()> {
        let output = Command::new(&self.swift_path)
            .current_dir(project_path)
            .arg("package")
            .arg("resolve")
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to resolve dependencies: {}", error));
        }

        Ok(())
    }

    /// Update package dependencies
    pub async fn update_dependencies(&self, project_path: &Path) -> Result<()> {
        let output = Command::new(&self.swift_path)
            .current_dir(project_path)
            .arg("package")
            .arg("update")
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to update dependencies: {}", error));
        }

        Ok(())
    }

    /// Get package information
    pub async fn describe_package(&self, project_path: &Path) -> Result<String> {
        let output = Command::new(&self.swift_path)
            .current_dir(project_path)
            .arg("package")
            .arg("describe")
            .arg("--type")
            .arg("json")
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to describe package: {}", error))
        }
    }

    /// Create a new Swift package
    pub async fn create_package(
        &self,
        path: &Path,
        name: &str,
        package_type: SwiftPackageType,
    ) -> Result<()> {
        std::fs::create_dir_all(path)?;

        let type_arg = match package_type {
            SwiftPackageType::Executable => "executable",
            SwiftPackageType::Library => "library",
            SwiftPackageType::SystemModule => "system-module",
        };

        let output = Command::new(&self.swift_path)
            .current_dir(path)
            .arg("package")
            .arg("init")
            .arg("--type")
            .arg(type_arg)
            .arg("--name")
            .arg(name)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create package: {}", error));
        }

        Ok(())
    }

    /// Run a Swift package
    pub async fn run(&self, project_path: &Path, args: &[&str]) -> Result<()> {
        let mut cmd = Command::new(&self.swift_path);
        cmd.current_dir(project_path)
            .arg("run");

        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to run: {}", error));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SwiftPackageType {
    Executable,
    Library,
    SystemModule,
}

impl Default for SwiftBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create SwiftBuilder")
    }
}
