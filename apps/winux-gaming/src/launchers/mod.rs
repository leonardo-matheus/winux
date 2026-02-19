// Launchers module - Integration with various game launchers
// Provides unified API for launching games from different sources

pub mod steam;
pub mod lutris;
pub mod heroic;
pub mod wine;
pub mod proton;

use std::path::PathBuf;
use anyhow::Result;

/// Represents a game launcher
pub trait Launcher {
    /// Get the name of the launcher
    fn name(&self) -> &str;

    /// Check if the launcher is installed
    fn is_installed(&self) -> bool;

    /// Get the installation path
    fn install_path(&self) -> Option<PathBuf>;

    /// Get all games from this launcher
    fn get_games(&self) -> Result<Vec<LauncherGame>>;

    /// Launch a game by its ID
    fn launch_game(&self, game_id: &str) -> Result<()>;

    /// Install a game
    fn install_game(&self, game_id: &str) -> Result<()>;

    /// Uninstall a game
    fn uninstall_game(&self, game_id: &str) -> Result<()>;
}

/// Represents a game from any launcher
#[derive(Debug, Clone)]
pub struct LauncherGame {
    pub id: String,
    pub name: String,
    pub launcher: String,
    pub install_path: Option<PathBuf>,
    pub executable: Option<PathBuf>,
    pub installed: bool,
    pub size_bytes: Option<u64>,
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
    pub playtime_seconds: u64,
    pub cover_image: Option<PathBuf>,
    pub background_image: Option<PathBuf>,
}

impl LauncherGame {
    pub fn playtime_hours(&self) -> f64 {
        self.playtime_seconds as f64 / 3600.0
    }

    pub fn size_formatted(&self) -> String {
        match self.size_bytes {
            Some(bytes) => {
                if bytes >= 1_073_741_824 {
                    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
                } else if bytes >= 1_048_576 {
                    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
                } else {
                    format!("{} KB", bytes / 1024)
                }
            }
            None => "Desconhecido".to_string(),
        }
    }
}

/// Get all available launchers
pub fn get_available_launchers() -> Vec<Box<dyn Launcher>> {
    let mut launchers: Vec<Box<dyn Launcher>> = Vec::new();

    // Add Steam
    let steam = steam::SteamLauncher::new();
    if steam.is_installed() {
        launchers.push(Box::new(steam));
    }

    // Add Heroic (GOG + Epic)
    let heroic = heroic::HeroicLauncher::new();
    if heroic.is_installed() {
        launchers.push(Box::new(heroic));
    }

    // Add Lutris
    let lutris = lutris::LutrisLauncher::new();
    if lutris.is_installed() {
        launchers.push(Box::new(lutris));
    }

    launchers
}

/// Get all games from all launchers
pub fn get_all_games() -> Result<Vec<LauncherGame>> {
    let mut all_games = Vec::new();

    for launcher in get_available_launchers() {
        if let Ok(games) = launcher.get_games() {
            all_games.extend(games);
        }
    }

    // Sort by last played (most recent first)
    all_games.sort_by(|a, b| {
        match (&b.last_played, &a.last_played) {
            (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        }
    });

    Ok(all_games)
}
