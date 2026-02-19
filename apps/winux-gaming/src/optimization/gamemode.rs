// GameMode integration
// Feral Interactive's GameMode for Linux gaming optimization

use std::process::Command;
use anyhow::{Result, Context};

/// Check if GameMode is available on the system
pub fn is_available() -> bool {
    Command::new("which")
        .arg("gamemoderun")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if GameMode daemon is running
pub fn is_daemon_running() -> bool {
    Command::new("gamemoded")
        .arg("--status")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get GameMode version
pub fn get_version() -> Option<String> {
    let output = Command::new("gamemoded")
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// GameMode configuration
#[derive(Debug, Clone)]
pub struct GameModeConfig {
    /// CPU governor to use when gaming
    pub cpu_governor: String,
    /// Nice value for game processes
    pub nice_value: i8,
    /// I/O nice value
    pub ioprio: i8,
    /// Whether to apply GPU optimizations
    pub apply_gpu_optimizations: bool,
    /// Custom scripts to run on game start
    pub start_scripts: Vec<String>,
    /// Custom scripts to run on game end
    pub end_scripts: Vec<String>,
}

impl Default for GameModeConfig {
    fn default() -> Self {
        Self {
            cpu_governor: "performance".to_string(),
            nice_value: -10,
            ioprio: 0, // Realtime class
            apply_gpu_optimizations: true,
            start_scripts: Vec::new(),
            end_scripts: Vec::new(),
        }
    }
}

/// Read current GameMode configuration
pub fn read_config() -> Result<GameModeConfig> {
    let config_paths = [
        "/etc/gamemode.ini",
        dirs::home_dir()
            .map(|h| h.join(".config/gamemode.ini"))
            .unwrap_or_default(),
    ];

    for path in config_paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                return parse_gamemode_ini(&content);
            }
        }
    }

    // Return defaults if no config found
    Ok(GameModeConfig::default())
}

/// Write GameMode configuration
pub fn write_config(config: &GameModeConfig) -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let config_path = home.join(".config/gamemode.ini");

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = format!(
        r#"[general]
; GameMode configuration

[cpu]
; CPU governor when gaming
desiredgov={}

[custom]
; Process nice value (-20 to 19, lower = higher priority)
renice={}

[gpu]
; Apply GPU optimizations
apply_gpu_optimisations={}

"#,
        config.cpu_governor,
        config.nice_value,
        if config.apply_gpu_optimizations { "accept-responsibility" } else { "0" }
    );

    std::fs::write(&config_path, content)?;
    Ok(())
}

fn parse_gamemode_ini(content: &str) -> Result<GameModeConfig> {
    let mut config = GameModeConfig::default();

    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].to_string();
            continue;
        }

        // Key=value pair
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos+1..].trim();

            match current_section.as_str() {
                "cpu" => {
                    if key == "desiredgov" {
                        config.cpu_governor = value.to_string();
                    }
                }
                "custom" => {
                    if key == "renice" {
                        config.nice_value = value.parse().unwrap_or(-10);
                    }
                }
                "gpu" => {
                    if key == "apply_gpu_optimisations" {
                        config.apply_gpu_optimizations = value == "accept-responsibility";
                    }
                }
                _ => {}
            }
        }
    }

    Ok(config)
}

/// GameMode statistics
#[derive(Debug, Clone)]
pub struct GameModeStats {
    pub active_clients: u32,
    pub games_running: Vec<String>,
}

/// Get current GameMode statistics
pub fn get_stats() -> Option<GameModeStats> {
    let output = Command::new("gamemoded")
        .arg("--status")
        .arg("-v")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse the output
        // This is a simplified parser
        let active_clients = stdout.matches("active").count() as u32;

        Some(GameModeStats {
            active_clients,
            games_running: Vec::new(),
        })
    } else {
        None
    }
}

/// Manually request GameMode for a process
pub fn request_for_pid(pid: u32) -> Result<()> {
    // Use D-Bus to request GameMode
    Command::new("gamemoded")
        .arg("--request")
        .arg(pid.to_string())
        .status()
        .context("Failed to request GameMode")?;
    Ok(())
}

/// Build command line for launching with GameMode
pub fn build_command(command: &str, args: &[&str]) -> Vec<String> {
    let mut full_command = vec!["gamemoderun".to_string(), command.to_string()];
    full_command.extend(args.iter().map(|s| s.to_string()));
    full_command
}
