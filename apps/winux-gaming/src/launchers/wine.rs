// Wine management module
// Handles Wine prefixes and launching Windows games

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::collections::HashMap;
use anyhow::{Result, Context};

/// Wine prefix configuration
#[derive(Debug, Clone)]
pub struct WinePrefix {
    pub path: PathBuf,
    pub arch: WineArch,
    pub wine_version: Option<String>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
}

/// Wine architecture
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WineArch {
    Win32,
    Win64,
}

impl WineArch {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Win32 => "win32",
            Self::Win64 => "win64",
        }
    }
}

/// Available Wine versions on the system
#[derive(Debug, Clone)]
pub struct WineVersion {
    pub name: String,
    pub path: PathBuf,
    pub version_string: Option<String>,
    pub is_proton: bool,
    pub is_wine_ge: bool,
}

impl WineVersion {
    pub fn display_name(&self) -> String {
        if let Some(ref version) = self.version_string {
            format!("{} ({})", self.name, version)
        } else {
            self.name.clone()
        }
    }
}

/// Wine manager for handling Wine operations
pub struct WineManager {
    available_versions: Vec<WineVersion>,
    default_prefix: PathBuf,
}

impl WineManager {
    pub fn new() -> Self {
        let available_versions = find_wine_versions();
        let default_prefix = dirs::home_dir()
            .map(|h| h.join(".wine"))
            .unwrap_or_else(|| PathBuf::from("/tmp/wine"));

        Self {
            available_versions,
            default_prefix,
        }
    }

    /// Get all available Wine versions
    pub fn get_versions(&self) -> &[WineVersion] {
        &self.available_versions
    }

    /// Get the system Wine version
    pub fn get_system_wine(&self) -> Option<&WineVersion> {
        self.available_versions
            .iter()
            .find(|v| v.name == "System Wine")
    }

    /// Create a new Wine prefix
    pub fn create_prefix(&self, path: &PathBuf, arch: WineArch, wine_path: Option<&PathBuf>) -> Result<WinePrefix> {
        std::fs::create_dir_all(path)?;

        let mut env_vars = HashMap::new();
        env_vars.insert("WINEPREFIX", path.to_string_lossy().to_string());
        env_vars.insert("WINEARCH", arch.as_str().to_string());

        let wine_binary = wine_path
            .map(|p| p.join("bin/wine").to_string_lossy().to_string())
            .unwrap_or_else(|| "wine".to_string());

        // Run wineboot to create the prefix
        let status = Command::new(&wine_binary)
            .arg("wineboot")
            .arg("--init")
            .envs(&env_vars)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("Failed to create Wine prefix")?;

        if !status.success() {
            anyhow::bail!("wineboot failed with status: {}", status);
        }

        Ok(WinePrefix {
            path: path.clone(),
            arch,
            wine_version: wine_path.map(|p| p.to_string_lossy().to_string()),
            created: Some(chrono::Utc::now()),
        })
    }

    /// Run a Windows executable
    pub fn run_executable(
        &self,
        exe_path: &PathBuf,
        prefix: &WinePrefix,
        wine_path: Option<&PathBuf>,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let wine_binary = wine_path
            .map(|p| p.join("bin/wine").to_string_lossy().to_string())
            .unwrap_or_else(|| "wine".to_string());

        let mut command = Command::new(&wine_binary);
        command.arg(exe_path);
        command.env("WINEPREFIX", &prefix.path);

        if let Some(vars) = env_vars {
            command.envs(vars);
        }

        command.spawn().context("Failed to launch Windows executable")?;
        Ok(())
    }

    /// Run Wine configuration (winecfg)
    pub fn run_winecfg(&self, prefix: &WinePrefix, wine_path: Option<&PathBuf>) -> Result<()> {
        let wine_binary = wine_path
            .map(|p| p.join("bin/winecfg").to_string_lossy().to_string())
            .unwrap_or_else(|| "winecfg".to_string());

        Command::new(&wine_binary)
            .env("WINEPREFIX", &prefix.path)
            .spawn()
            .context("Failed to launch winecfg")?;
        Ok(())
    }

    /// Run winetricks
    pub fn run_winetricks(&self, prefix: &WinePrefix, verbs: &[&str]) -> Result<()> {
        let mut command = Command::new("winetricks");
        command.env("WINEPREFIX", &prefix.path);

        for verb in verbs {
            command.arg(verb);
        }

        command.spawn().context("Failed to run winetricks")?;
        Ok(())
    }

    /// Check if DXVK is installed in a prefix
    pub fn is_dxvk_installed(&self, prefix: &WinePrefix) -> bool {
        let d3d11_path = prefix.path.join("drive_c/windows/system32/d3d11.dll");
        if d3d11_path.exists() {
            // Check if it's the DXVK version by looking for specific strings
            if let Ok(content) = std::fs::read(&d3d11_path) {
                return content.windows(4).any(|w| w == b"DXVK");
            }
        }
        false
    }

    /// Install DXVK in a prefix
    pub fn install_dxvk(&self, prefix: &WinePrefix) -> Result<()> {
        self.run_winetricks(prefix, &["dxvk"])
    }
}

fn find_wine_versions() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    // Check system Wine
    if let Ok(output) = Command::new("wine").arg("--version").output() {
        if output.status.success() {
            let version_string = String::from_utf8_lossy(&output.stdout).trim().to_string();
            versions.push(WineVersion {
                name: "System Wine".to_string(),
                path: PathBuf::from("/usr/bin"),
                version_string: Some(version_string),
                is_proton: false,
                is_wine_ge: false,
            });
        }
    }

    // Check for Wine-GE in common locations
    if let Some(home) = dirs::home_dir() {
        let wine_ge_paths = [
            home.join(".local/share/lutris/runners/wine"),
            home.join(".var/app/net.lutris.Lutris/data/lutris/runners/wine"),
        ];

        for base_path in wine_ge_paths {
            if base_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&base_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let name = path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();

                            if name.contains("GE") || name.contains("ge") {
                                versions.push(WineVersion {
                                    name: name.clone(),
                                    path: path.clone(),
                                    version_string: None,
                                    is_proton: false,
                                    is_wine_ge: true,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    versions
}

/// Common DXVK/VKD3D environment variables
pub fn get_dxvk_env_vars(async_compile: bool) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    vars.insert("DXVK_LOG_LEVEL".to_string(), "none".to_string());

    if async_compile {
        vars.insert("DXVK_ASYNC".to_string(), "1".to_string());
    }

    vars
}

/// Common Wine environment variables for gaming
pub fn get_gaming_env_vars(enable_esync: bool, enable_fsync: bool) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    if enable_fsync {
        vars.insert("WINEFSYNC".to_string(), "1".to_string());
    } else if enable_esync {
        vars.insert("WINEESYNC".to_string(), "1".to_string());
    }

    // Disable debugging for better performance
    vars.insert("WINEDEBUG".to_string(), "-all".to_string());

    vars
}
