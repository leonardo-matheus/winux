// Lutris launcher integration
// Open gaming platform for Linux

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use super::{Launcher, LauncherGame};

pub struct LutrisLauncher {
    config_path: Option<PathBuf>,
    data_path: Option<PathBuf>,
}

impl LutrisLauncher {
    pub fn new() -> Self {
        let config_path = find_lutris_config();
        let data_path = find_lutris_data();

        Self {
            config_path,
            data_path,
        }
    }

    fn parse_game_yml(&self, path: &PathBuf) -> Option<LutrisGame> {
        let content = std::fs::read_to_string(path).ok()?;
        // Simple YAML-like parsing for Lutris game files
        let mut name = String::new();
        let mut slug = String::new();
        let mut runner = String::new();
        let mut directory = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name:") {
                name = line.trim_start_matches("name:").trim().to_string();
            } else if line.starts_with("slug:") {
                slug = line.trim_start_matches("slug:").trim().to_string();
            } else if line.starts_with("runner:") {
                runner = line.trim_start_matches("runner:").trim().to_string();
            } else if line.starts_with("directory:") {
                directory = line.trim_start_matches("directory:").trim().to_string();
            }
        }

        if name.is_empty() || slug.is_empty() {
            return None;
        }

        Some(LutrisGame {
            name,
            slug,
            runner,
            directory,
        })
    }
}

impl Launcher for LutrisLauncher {
    fn name(&self) -> &str {
        "Lutris"
    }

    fn is_installed(&self) -> bool {
        // Check if lutris binary exists
        Command::new("which")
            .arg("lutris")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install_path(&self) -> Option<PathBuf> {
        self.data_path.clone()
    }

    fn get_games(&self) -> Result<Vec<LauncherGame>> {
        let mut games = Vec::new();

        let data_path = self.data_path.as_ref().context("Lutris not installed")?;
        let games_path = data_path.join("games");

        if games_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&games_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "yml") {
                        if let Some(lutris_game) = self.parse_game_yml(&path) {
                            let install_path = if lutris_game.directory.is_empty() {
                                None
                            } else {
                                Some(PathBuf::from(&lutris_game.directory))
                            };

                            let game = LauncherGame {
                                id: lutris_game.slug.clone(),
                                name: lutris_game.name,
                                launcher: format!("Lutris ({})", lutris_game.runner),
                                install_path,
                                executable: None,
                                installed: true,
                                size_bytes: None,
                                last_played: None,
                                playtime_seconds: 0,
                                cover_image: get_lutris_cover(&lutris_game.slug),
                                background_image: None,
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
        Command::new("lutris")
            .arg(format!("lutris:rungame/{}", game_id))
            .spawn()
            .context("Failed to launch game via Lutris")?;
        Ok(())
    }

    fn install_game(&self, game_id: &str) -> Result<()> {
        Command::new("lutris")
            .arg(format!("lutris:install/{}", game_id))
            .spawn()
            .context("Failed to start game installation via Lutris")?;
        Ok(())
    }

    fn uninstall_game(&self, game_id: &str) -> Result<()> {
        Command::new("lutris")
            .arg("--uninstall")
            .arg(game_id)
            .spawn()
            .context("Failed to uninstall game via Lutris")?;
        Ok(())
    }
}

fn find_lutris_config() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let paths = [
        home.join(".config/lutris"),
        home.join(".var/app/net.lutris.Lutris/config/lutris"), // Flatpak
    ];

    for path in paths {
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn find_lutris_data() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let paths = [
        home.join(".local/share/lutris"),
        home.join(".var/app/net.lutris.Lutris/data/lutris"), // Flatpak
    ];

    for path in paths {
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn get_lutris_cover(slug: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let cache_path = home.join(format!(
        ".cache/lutris/coverart/{}.jpg",
        slug
    ));
    if cache_path.exists() {
        Some(cache_path)
    } else {
        None
    }
}

#[derive(Debug)]
struct LutrisGame {
    name: String,
    slug: String,
    runner: String,
    directory: String,
}

/// Lutris runners enumeration
#[derive(Debug, Clone, Copy)]
pub enum LutrisRunner {
    Wine,
    Linux,
    Steam,
    Dosbox,
    ScummVM,
    Libretro,
    Browser,
    Pico8,
}

impl LutrisRunner {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "wine" => Some(Self::Wine),
            "linux" => Some(Self::Linux),
            "steam" => Some(Self::Steam),
            "dosbox" => Some(Self::Dosbox),
            "scummvm" => Some(Self::ScummVM),
            "libretro" => Some(Self::Libretro),
            "browser" => Some(Self::Browser),
            "pico8" => Some(Self::Pico8),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Wine => "Wine",
            Self::Linux => "Native Linux",
            Self::Steam => "Steam",
            Self::Dosbox => "DOSBox",
            Self::ScummVM => "ScummVM",
            Self::Libretro => "RetroArch",
            Self::Browser => "Browser",
            Self::Pico8 => "PICO-8",
        }
    }
}
