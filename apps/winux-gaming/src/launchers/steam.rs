// Steam launcher integration
// Reads Steam library and launches games via Steam or directly with Proton

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use super::{Launcher, LauncherGame};

pub struct SteamLauncher {
    steam_path: Option<PathBuf>,
    library_paths: Vec<PathBuf>,
}

impl SteamLauncher {
    pub fn new() -> Self {
        let steam_path = find_steam_path();
        let library_paths = steam_path
            .as_ref()
            .map(|p| find_library_folders(p))
            .unwrap_or_default();

        Self {
            steam_path,
            library_paths,
        }
    }

    /// Parse Steam's VDF/ACF files to get game info
    fn parse_app_manifest(&self, path: &PathBuf) -> Option<SteamAppManifest> {
        let content = std::fs::read_to_string(path).ok()?;
        parse_acf_file(&content)
    }

    /// Get Steam user data for playtime
    fn get_user_game_stats(&self, app_id: &str) -> Option<SteamGameStats> {
        // Would parse localconfig.vdf for playtime
        None
    }
}

impl Launcher for SteamLauncher {
    fn name(&self) -> &str {
        "Steam"
    }

    fn is_installed(&self) -> bool {
        self.steam_path.is_some()
    }

    fn install_path(&self) -> Option<PathBuf> {
        self.steam_path.clone()
    }

    fn get_games(&self) -> Result<Vec<LauncherGame>> {
        let mut games = Vec::new();

        for library_path in &self.library_paths {
            let steamapps = library_path.join("steamapps");
            if !steamapps.exists() {
                continue;
            }

            // Read all appmanifest files
            if let Ok(entries) = std::fs::read_dir(&steamapps) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "acf") {
                        if let Some(manifest) = self.parse_app_manifest(&path) {
                            let game = LauncherGame {
                                id: manifest.appid.clone(),
                                name: manifest.name,
                                launcher: "Steam".to_string(),
                                install_path: Some(steamapps.join("common").join(&manifest.installdir)),
                                executable: None,
                                installed: manifest.state_flags == "4", // Fully installed
                                size_bytes: manifest.size_on_disk.parse().ok(),
                                last_played: None,
                                playtime_seconds: 0,
                                cover_image: get_steam_cover(&manifest.appid),
                                background_image: get_steam_background(&manifest.appid),
                            };
                            games.push(game);
                        }
                    }
                }
            }
        }

        Ok(games)
    }

    fn launch_game(&self, game_id: &str) -> Result<()> {
        // Launch via Steam protocol
        Command::new("xdg-open")
            .arg(format!("steam://rungameid/{}", game_id))
            .spawn()
            .context("Failed to launch Steam game")?;
        Ok(())
    }

    fn install_game(&self, game_id: &str) -> Result<()> {
        Command::new("xdg-open")
            .arg(format!("steam://install/{}", game_id))
            .spawn()
            .context("Failed to start game installation")?;
        Ok(())
    }

    fn uninstall_game(&self, game_id: &str) -> Result<()> {
        Command::new("xdg-open")
            .arg(format!("steam://uninstall/{}", game_id))
            .spawn()
            .context("Failed to start game uninstallation")?;
        Ok(())
    }
}

#[derive(Debug)]
struct SteamAppManifest {
    appid: String,
    name: String,
    installdir: String,
    state_flags: String,
    size_on_disk: String,
}

fn find_steam_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    // Check common Steam installation paths
    let paths = [
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.steam/steam"), // Flatpak
    ];

    for path in paths {
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn find_library_folders(steam_path: &PathBuf) -> Vec<PathBuf> {
    let mut libraries = vec![steam_path.clone()];

    // Parse libraryfolders.vdf for additional library paths
    let library_file = steam_path.join("steamapps/libraryfolders.vdf");
    if let Ok(content) = std::fs::read_to_string(&library_file) {
        // Simple VDF parsing - look for "path" entries
        for line in content.lines() {
            if line.contains("\"path\"") {
                if let Some(path_start) = line.rfind('"') {
                    let before_last = &line[..path_start];
                    if let Some(path_end) = before_last.rfind('"') {
                        let path_str = &before_last[path_end + 1..];
                        let path = PathBuf::from(path_str);
                        if path.exists() && !libraries.contains(&path) {
                            libraries.push(path);
                        }
                    }
                }
            }
        }
    }

    libraries
}

fn parse_acf_file(content: &str) -> Option<SteamAppManifest> {
    let mut appid = String::new();
    let mut name = String::new();
    let mut installdir = String::new();
    let mut state_flags = String::new();
    let mut size_on_disk = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.contains("\"appid\"") {
            appid = extract_vdf_value(line)?;
        } else if line.contains("\"name\"") {
            name = extract_vdf_value(line)?;
        } else if line.contains("\"installdir\"") {
            installdir = extract_vdf_value(line)?;
        } else if line.contains("\"StateFlags\"") {
            state_flags = extract_vdf_value(line)?;
        } else if line.contains("\"SizeOnDisk\"") {
            size_on_disk = extract_vdf_value(line)?;
        }
    }

    if appid.is_empty() || name.is_empty() {
        return None;
    }

    Some(SteamAppManifest {
        appid,
        name,
        installdir,
        state_flags,
        size_on_disk,
    })
}

fn extract_vdf_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

fn get_steam_cover(app_id: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let cache_path = home.join(format!(
        ".local/share/Steam/appcache/librarycache/{}_library_600x900.jpg",
        app_id
    ));
    if cache_path.exists() {
        Some(cache_path)
    } else {
        None
    }
}

fn get_steam_background(app_id: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let cache_path = home.join(format!(
        ".local/share/Steam/appcache/librarycache/{}_library_hero.jpg",
        app_id
    ));
    if cache_path.exists() {
        Some(cache_path)
    } else {
        None
    }
}

#[derive(Debug)]
struct SteamGameStats {
    playtime_forever: u64,
    playtime_2weeks: u64,
    last_played: u64,
}
