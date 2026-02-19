// Build system modules for different platforms

pub mod apple;
pub mod windows;
pub mod linux;
pub mod universal;

use anyhow::Result;
use std::path::Path;

/// Trait for all builders
pub trait Builder {
    /// Get the name of the builder
    fn name(&self) -> &str;

    /// Get available output formats
    fn formats(&self) -> &[&str];

    /// Check if dependencies are available
    fn check_dependencies(&self) -> Result<Vec<DependencyStatus>>;

    /// Build the project
    fn build(&self, project_path: &Path, format: &str, release: bool) -> Result<BuildCommand>;
}

#[derive(Debug, Clone)]
pub struct DependencyStatus {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub install_hint: String,
}

#[derive(Debug, Clone)]
pub struct BuildCommand {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub working_dir: Option<String>,
}

impl BuildCommand {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            args: Vec::new(),
            env: Vec::new(),
            working_dir: None,
        }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|s| s.to_string()));
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    pub fn working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    pub fn to_shell_command(&self) -> String {
        let mut parts = vec![self.command.clone()];
        parts.extend(self.args.iter().map(|a| {
            if a.contains(' ') {
                format!("\"{}\"", a)
            } else {
                a.clone()
            }
        }));

        let cmd = parts.join(" ");

        if let Some(dir) = &self.working_dir {
            format!("cd {} && {}", dir, cmd)
        } else {
            cmd
        }
    }
}

/// Get all available builders
pub fn get_builders() -> Vec<Box<dyn Builder>> {
    vec![
        Box::new(linux::LinuxBuilder::new()),
        Box::new(windows::WindowsBuilder::new()),
        Box::new(apple::AppleBuilder::new()),
        Box::new(universal::UniversalBuilder::new()),
    ]
}

/// Get builder for a specific platform
pub fn get_builder(platform: &str) -> Option<Box<dyn Builder>> {
    match platform.to_lowercase().as_str() {
        "linux" => Some(Box::new(linux::LinuxBuilder::new())),
        "windows" => Some(Box::new(windows::WindowsBuilder::new())),
        "macos" | "apple" | "ios" => Some(Box::new(apple::AppleBuilder::new())),
        "universal" | "cross" => Some(Box::new(universal::UniversalBuilder::new())),
        _ => None,
    }
}
