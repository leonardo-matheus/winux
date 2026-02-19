// Heroic Games Launcher integration
// Supports GOG Galaxy and Epic Games Store

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use super::{Launcher, LauncherGame};

pub struct HeroicLauncher {
    config_path: Option<PathBuf>,
    gog_installed: bool,
    epic_installed: bool,
}

impl HeroicLauncher {
    pub fn new() -> Self {
        let config_path = find_heroic_config();
        let (gog_installed, epic_installed) = check_heroic_stores(&config_path);

        Self {
            config_path,
            gog_installed,
            epic_installed,
        }
    }

    fn get_gog_games(&self) -> Result<Vec<LauncherGame>> {
        let mut games = Vec::new();
        let config_path = self.config_path.as_ref().context("Heroic not installed")?;

        let gog_library = config_path.join("gog_store/library.json");
        if gog_library.exists() {
            if let Ok(content) = std::fs::read_to_string(&gog_library) {
                if let Ok(library) = serde_json::from_str::<GogLibrary>(&content) {
                    for game in library.games {
                        games.push(LauncherGame {
                            id: game.app_name.clone(),
                            name: game.title,
                            launcher: "GOG".to_string(),
                            install_path: game.install_path.map(PathBuf::from),
                            executable: None,
                            installed: game.is_installed,
                            size_bytes: game.install_size,
                            last_played: None,
                            playtime_seconds: 0,
                            cover_image: game.art_cover.map(PathBuf::from),
                            background_image: game.art_background.map(PathBuf::from),
                        });
                    }
                }
            }
        }

        Ok(games)
    }

    fn get_epic_games(&self) -> Result<Vec<LauncherGame>> {
        let mut games = Vec::new();
        let config_path = self.config_path.as_ref().context("Heroic not installed")?;

        let epic_library = config_path.join("store/library.json");
        if epic_library.exists() {
            if let Ok(content) = std::fs::read_to_string(&epic_library) {
                if let Ok(library) = serde_json::from_str::<EpicLibrary>(&content) {
                    for game in library.library {
                        games.push(LauncherGame {
                            id: game.app_name.clone(),
                            name: game.title,
                            launcher: "Epic".to_string(),
                            install_path: game.install_path.map(PathBuf::from),
                            executable: None,
                            installed: game.is_installed,
                            size_bytes: game.install_size,
                            last_played: None,
                            playtime_seconds: 0,
                            cover_image: game.art_cover.map(PathBuf::from),
                            background_image: None,
                        });
                    }
                }
            }
        }

        Ok(games)
    }
}

impl Launcher for HeroicLauncher {
    fn name(&self) -> &str {
        "Heroic"
    }

    fn is_installed(&self) -> bool {
        self.config_path.is_some()
    }

    fn install_path(&self) -> Option<PathBuf> {
        self.config_path.clone()
    }

    fn get_games(&self) -> Result<Vec<LauncherGame>> {
        let mut all_games = Vec::new();

        if self.gog_installed {
            if let Ok(gog_games) = self.get_gog_games() {
                all_games.extend(gog_games);
            }
        }

        if self.epic_installed {
            if let Ok(epic_games) = self.get_epic_games() {
                all_games.extend(epic_games);
            }
        }

        Ok(all_games)
    }

    fn launch_game(&self, game_id: &str) -> Result<()> {
        Command::new("heroic")
            .arg("--launch")
            .arg(game_id)
            .spawn()
            .context("Failed to launch game via Heroic")?;
        Ok(())
    }

    fn install_game(&self, game_id: &str) -> Result<()> {
        Command::new("heroic")
            .arg("--install")
            .arg(game_id)
            .spawn()
            .context("Failed to start game installation via Heroic")?;
        Ok(())
    }

    fn uninstall_game(&self, game_id: &str) -> Result<()> {
        Command::new("heroic")
            .arg("--uninstall")
            .arg(game_id)
            .spawn()
            .context("Failed to uninstall game via Heroic")?;
        Ok(())
    }
}

fn find_heroic_config() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let paths = [
        home.join(".config/heroic"),
        home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic"), // Flatpak
    ];

    for path in paths {
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn check_heroic_stores(config_path: &Option<PathBuf>) -> (bool, bool) {
    let Some(path) = config_path else {
        return (false, false);
    };

    let gog_installed = path.join("gog_store/library.json").exists();
    let epic_installed = path.join("store/library.json").exists();

    (gog_installed, epic_installed)
}

#[derive(Debug, Deserialize)]
struct GogLibrary {
    games: Vec<GogGame>,
}

#[derive(Debug, Deserialize)]
struct GogGame {
    app_name: String,
    title: String,
    install_path: Option<String>,
    is_installed: bool,
    install_size: Option<u64>,
    art_cover: Option<String>,
    art_background: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EpicLibrary {
    library: Vec<EpicGame>,
}

#[derive(Debug, Deserialize)]
struct EpicGame {
    app_name: String,
    title: String,
    install_path: Option<String>,
    is_installed: bool,
    install_size: Option<u64>,
    art_cover: Option<String>,
}
