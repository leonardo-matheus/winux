// MangoHud integration
// Performance overlay for games

use std::collections::HashMap;
use std::process::Command;
use std::path::PathBuf;
use anyhow::{Result, Context};

/// Check if MangoHud is available
pub fn is_available() -> bool {
    Command::new("which")
        .arg("mangohud")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get MangoHud version
pub fn get_version() -> Option<String> {
    let output = Command::new("mangohud")
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// MangoHud preset levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MangoHudPreset {
    /// Minimal - just FPS
    Minimal,
    /// Default - FPS, frametime, CPU/GPU usage
    Default,
    /// Full - all metrics
    Full,
    /// Custom - user-defined
    Custom,
}

impl MangoHudPreset {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Minimal => "0",
            Self::Default => "1",
            Self::Full => "2",
            Self::Custom => "3",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Minimal => "Minimal",
            Self::Default => "Default",
            Self::Full => "Full",
            Self::Custom => "Custom",
        }
    }
}

/// MangoHud position on screen
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MangoHudPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
}

impl MangoHudPosition {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
            Self::TopCenter => "top-center",
            Self::BottomCenter => "bottom-center",
        }
    }
}

/// MangoHud configuration
#[derive(Debug, Clone)]
pub struct MangoHudConfig {
    // Position
    pub position: MangoHudPosition,

    // Metrics to show
    pub show_fps: bool,
    pub show_frametime: bool,
    pub show_cpu_stats: bool,
    pub show_cpu_temp: bool,
    pub show_cpu_power: bool,
    pub show_gpu_stats: bool,
    pub show_gpu_temp: bool,
    pub show_gpu_power: bool,
    pub show_vram: bool,
    pub show_ram: bool,
    pub show_fan: bool,
    pub show_battery: bool,
    pub show_gamemode: bool,
    pub show_wine: bool,
    pub show_fsr: bool,
    pub show_resolution: bool,
    pub show_media_player: bool,

    // FPS limit
    pub fps_limit: Option<u32>,
    pub fps_limit_method: FpsLimitMethod,

    // Logging
    pub log_enabled: bool,
    pub log_duration: Option<u32>, // seconds
    pub log_interval: u32, // ms

    // Appearance
    pub font_size: u32,
    pub background_alpha: f32,
    pub toggle_hud_key: String,
    pub toggle_logging_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FpsLimitMethod {
    /// Late (default) - better input latency
    Late,
    /// Early - more consistent frametimes
    Early,
}

impl Default for MangoHudConfig {
    fn default() -> Self {
        Self {
            position: MangoHudPosition::TopLeft,

            show_fps: true,
            show_frametime: true,
            show_cpu_stats: true,
            show_cpu_temp: true,
            show_cpu_power: false,
            show_gpu_stats: true,
            show_gpu_temp: true,
            show_gpu_power: false,
            show_vram: false,
            show_ram: false,
            show_fan: false,
            show_battery: false,
            show_gamemode: true,
            show_wine: true,
            show_fsr: true,
            show_resolution: false,
            show_media_player: false,

            fps_limit: None,
            fps_limit_method: FpsLimitMethod::Late,

            log_enabled: false,
            log_duration: None,
            log_interval: 100,

            font_size: 24,
            background_alpha: 0.5,
            toggle_hud_key: "Shift_R+F12".to_string(),
            toggle_logging_key: "Shift_L+F2".to_string(),
        }
    }
}

impl MangoHudConfig {
    /// Create minimal config (just FPS)
    pub fn minimal() -> Self {
        Self {
            show_fps: true,
            show_frametime: false,
            show_cpu_stats: false,
            show_cpu_temp: false,
            show_cpu_power: false,
            show_gpu_stats: false,
            show_gpu_temp: false,
            show_gpu_power: false,
            show_vram: false,
            show_ram: false,
            show_fan: false,
            show_battery: false,
            show_gamemode: false,
            show_wine: false,
            show_fsr: false,
            show_resolution: false,
            show_media_player: false,
            ..Default::default()
        }
    }

    /// Create full config (all metrics)
    pub fn full() -> Self {
        Self {
            show_fps: true,
            show_frametime: true,
            show_cpu_stats: true,
            show_cpu_temp: true,
            show_cpu_power: true,
            show_gpu_stats: true,
            show_gpu_temp: true,
            show_gpu_power: true,
            show_vram: true,
            show_ram: true,
            show_fan: true,
            show_battery: true,
            show_gamemode: true,
            show_wine: true,
            show_fsr: true,
            show_resolution: true,
            show_media_player: false,
            ..Default::default()
        }
    }

    /// Generate MangoHud config string
    pub fn to_config_string(&self) -> String {
        let mut options = Vec::new();

        // Position
        options.push(format!("position={}", self.position.as_str()));

        // Metrics
        if self.show_fps { options.push("fps".to_string()); }
        if self.show_frametime { options.push("frame_timing".to_string()); }
        if self.show_cpu_stats { options.push("cpu_stats".to_string()); }
        if self.show_cpu_temp { options.push("cpu_temp".to_string()); }
        if self.show_cpu_power { options.push("cpu_power".to_string()); }
        if self.show_gpu_stats { options.push("gpu_stats".to_string()); }
        if self.show_gpu_temp { options.push("gpu_temp".to_string()); }
        if self.show_gpu_power { options.push("gpu_power".to_string()); }
        if self.show_vram { options.push("vram".to_string()); }
        if self.show_ram { options.push("ram".to_string()); }
        if self.show_fan { options.push("fan".to_string()); }
        if self.show_battery { options.push("battery".to_string()); }
        if self.show_gamemode { options.push("gamemode".to_string()); }
        if self.show_wine { options.push("wine".to_string()); }
        if self.show_fsr { options.push("fsr".to_string()); }
        if self.show_resolution { options.push("resolution".to_string()); }

        // FPS Limit
        if let Some(limit) = self.fps_limit {
            options.push(format!("fps_limit={}", limit));
        }

        // Font size
        options.push(format!("font_size={}", self.font_size));

        // Background
        options.push(format!("background_alpha={}", self.background_alpha));

        // Hotkeys
        options.push(format!("toggle_hud={}", self.toggle_hud_key));
        options.push(format!("toggle_logging={}", self.toggle_logging_key));

        options.join("\n")
    }

    /// Save config to file
    pub fn save(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let config_path = home.join(".config/MangoHud/MangoHud.conf");

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&config_path, self.to_config_string())?;
        Ok(config_path)
    }
}

/// Get environment variables for MangoHud
pub fn get_mangohud_env(preset: &MangoHudPreset) -> HashMap<String, String> {
    let mut env = HashMap::new();

    env.insert("MANGOHUD".to_string(), "1".to_string());
    env.insert("MANGOHUD_CONFIG".to_string(), format!("preset={}", preset.as_str()));

    env
}

/// Get environment variables for custom MangoHud config
pub fn get_custom_env(config: &MangoHudConfig) -> HashMap<String, String> {
    let mut env = HashMap::new();

    env.insert("MANGOHUD".to_string(), "1".to_string());
    env.insert("MANGOHUD_CONFIG".to_string(), config.to_config_string());

    env
}

/// Build command to launch with MangoHud
pub fn build_command(command: &str, args: &[&str]) -> Vec<String> {
    let mut full_command = vec!["mangohud".to_string(), command.to_string()];
    full_command.extend(args.iter().map(|s| s.to_string()));
    full_command
}

/// MangoHud log file info
#[derive(Debug)]
pub struct MangoHudLog {
    pub path: PathBuf,
    pub game_name: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Find MangoHud log files
pub fn find_logs() -> Vec<MangoHudLog> {
    let mut logs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let log_dir = home.join(".local/share/MangoHud");

        if log_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&log_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "csv") {
                        if let Some(name) = path.file_stem() {
                            logs.push(MangoHudLog {
                                path: path.clone(),
                                game_name: name.to_string_lossy().to_string(),
                                timestamp: chrono::Local::now(), // Should parse from filename
                            });
                        }
                    }
                }
            }
        }
    }

    logs
}
