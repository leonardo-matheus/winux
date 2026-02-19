// Proton management module
// Handles Proton versions and launching games with Proton

use std::path::PathBuf;
use std::process::Command;
use std::collections::HashMap;
use anyhow::{Result, Context};

/// Proton version information
#[derive(Debug, Clone)]
pub struct ProtonVersion {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
    pub is_experimental: bool,
    pub is_ge: bool,
    pub compatdata_path: Option<PathBuf>,
}

impl ProtonVersion {
    pub fn display_name(&self) -> String {
        let mut name = self.name.clone();
        if self.is_experimental {
            name.push_str(" (Experimental)");
        }
        if self.is_ge {
            name.push_str(" (GE)");
        }
        name
    }
}

/// Proton compatibility data (prefix)
#[derive(Debug, Clone)]
pub struct ProtonCompatData {
    pub path: PathBuf,
    pub app_id: String,
    pub proton_version: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Proton manager
pub struct ProtonManager {
    steam_path: Option<PathBuf>,
    available_versions: Vec<ProtonVersion>,
}

impl ProtonManager {
    pub fn new() -> Self {
        let steam_path = find_steam_path();
        let available_versions = find_proton_versions(&steam_path);

        Self {
            steam_path,
            available_versions,
        }
    }

    /// Get all available Proton versions
    pub fn get_versions(&self) -> &[ProtonVersion] {
        &self.available_versions
    }

    /// Get default Proton version (experimental or latest stable)
    pub fn get_default(&self) -> Option<&ProtonVersion> {
        self.available_versions
            .iter()
            .find(|v| v.is_experimental)
            .or_else(|| self.available_versions.first())
    }

    /// Get Proton-GE versions
    pub fn get_ge_versions(&self) -> Vec<&ProtonVersion> {
        self.available_versions
            .iter()
            .filter(|v| v.is_ge)
            .collect()
    }

    /// Get compatdata for a Steam app
    pub fn get_compatdata(&self, app_id: &str) -> Option<ProtonCompatData> {
        let steam_path = self.steam_path.as_ref()?;
        let compat_path = steam_path.join(format!("steamapps/compatdata/{}", app_id));

        if compat_path.exists() {
            let size = dir_size(&compat_path).ok();

            Some(ProtonCompatData {
                path: compat_path,
                app_id: app_id.to_string(),
                proton_version: None,
                size_bytes: size,
            })
        } else {
            None
        }
    }

    /// Create compatdata for a new game
    pub fn create_compatdata(&self, app_id: &str, proton: &ProtonVersion) -> Result<ProtonCompatData> {
        let steam_path = self.steam_path.as_ref().context("Steam not found")?;
        let compat_path = steam_path.join(format!("steamapps/compatdata/{}", app_id));

        std::fs::create_dir_all(&compat_path)?;

        // Initialize the prefix with Proton
        let env_vars = get_proton_env_vars(&compat_path, &proton.path);

        Command::new(proton.path.join("proton"))
            .arg("run")
            .arg("wineboot")
            .envs(&env_vars)
            .status()
            .context("Failed to initialize Proton prefix")?;

        Ok(ProtonCompatData {
            path: compat_path,
            app_id: app_id.to_string(),
            proton_version: Some(proton.name.clone()),
            size_bytes: None,
        })
    }

    /// Run a game with Proton
    pub fn run_game(
        &self,
        exe_path: &PathBuf,
        app_id: &str,
        proton: &ProtonVersion,
        extra_env: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let steam_path = self.steam_path.as_ref().context("Steam not found")?;
        let compat_path = steam_path.join(format!("steamapps/compatdata/{}", app_id));

        // Ensure compatdata exists
        if !compat_path.exists() {
            self.create_compatdata(app_id, proton)?;
        }

        let mut env_vars = get_proton_env_vars(&compat_path, &proton.path);

        if let Some(extra) = extra_env {
            env_vars.extend(extra);
        }

        Command::new(proton.path.join("proton"))
            .arg("run")
            .arg(exe_path)
            .envs(&env_vars)
            .spawn()
            .context("Failed to launch game with Proton")?;

        Ok(())
    }

    /// Delete compatdata for a game
    pub fn delete_compatdata(&self, app_id: &str) -> Result<()> {
        let steam_path = self.steam_path.as_ref().context("Steam not found")?;
        let compat_path = steam_path.join(format!("steamapps/compatdata/{}", app_id));

        if compat_path.exists() {
            std::fs::remove_dir_all(&compat_path)?;
        }

        Ok(())
    }
}

fn find_steam_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let paths = [
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
    ];

    for path in paths {
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn find_proton_versions(steam_path: &Option<PathBuf>) -> Vec<ProtonVersion> {
    let mut versions = Vec::new();

    if let Some(steam) = steam_path {
        // Official Proton versions in Steam's common folder
        let common_path = steam.join("steamapps/common");
        if common_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&common_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if name.starts_with("Proton") && path.join("proton").exists() {
                        let is_experimental = name.contains("Experimental");
                        versions.push(ProtonVersion {
                            name: name.clone(),
                            path: path.clone(),
                            version: None,
                            is_experimental,
                            is_ge: false,
                            compatdata_path: None,
                        });
                    }
                }
            }
        }

        // Proton-GE in compatibility tools
        let compat_tools_path = steam.join("compatibilitytools.d");
        if compat_tools_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&compat_tools_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if (name.contains("GE") || name.contains("Proton")) && path.join("proton").exists() {
                        versions.push(ProtonVersion {
                            name: name.clone(),
                            path: path.clone(),
                            version: None,
                            is_experimental: false,
                            is_ge: name.contains("GE"),
                            compatdata_path: None,
                        });
                    }
                }
            }
        }
    }

    // Also check user's home for Proton-GE
    if let Some(home) = dirs::home_dir() {
        let user_compat_path = home.join(".steam/root/compatibilitytools.d");
        if user_compat_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&user_compat_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if name.contains("GE") && path.join("proton").exists() {
                        // Avoid duplicates
                        if !versions.iter().any(|v| v.name == name) {
                            versions.push(ProtonVersion {
                                name: name.clone(),
                                path: path.clone(),
                                version: None,
                                is_experimental: false,
                                is_ge: true,
                                compatdata_path: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort: Experimental first, then by name descending (newer versions first)
    versions.sort_by(|a, b| {
        if a.is_experimental != b.is_experimental {
            return b.is_experimental.cmp(&a.is_experimental);
        }
        b.name.cmp(&a.name)
    });

    versions
}

/// Get environment variables for running with Proton
pub fn get_proton_env_vars(compat_path: &PathBuf, proton_path: &PathBuf) -> HashMap<String, String> {
    let mut env = HashMap::new();

    env.insert("STEAM_COMPAT_DATA_PATH".to_string(), compat_path.to_string_lossy().to_string());
    env.insert("STEAM_COMPAT_CLIENT_INSTALL_PATH".to_string(),
        proton_path.parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    );

    // Enable Fsync if supported
    env.insert("PROTON_NO_FSYNC".to_string(), "0".to_string());

    // Disable debugging
    env.insert("WINEDEBUG".to_string(), "-all".to_string());

    env
}

/// Get recommended Proton environment for gaming
pub fn get_gaming_proton_env(
    compat_path: &PathBuf,
    proton_path: &PathBuf,
    enable_mangohud: bool,
    enable_gamemode: bool,
) -> HashMap<String, String> {
    let mut env = get_proton_env_vars(compat_path, proton_path);

    if enable_mangohud {
        env.insert("MANGOHUD".to_string(), "1".to_string());
    }

    if enable_gamemode {
        env.insert("GAMEMODERUNEXEC".to_string(), "1".to_string());
    }

    // Performance optimizations
    env.insert("PROTON_ENABLE_NVAPI".to_string(), "1".to_string());
    env.insert("PROTON_HIDE_NVIDIA_GPU".to_string(), "0".to_string());

    env
}

fn dir_size(path: &PathBuf) -> Result<u64> {
    let mut total = 0;

    if path.is_dir() {
        for entry in walkdir::WalkDir::new(path).into_iter().flatten() {
            if entry.file_type().is_file() {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }

    Ok(total)
}
