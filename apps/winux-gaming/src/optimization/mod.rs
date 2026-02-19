// Optimization module - Gaming performance optimizations
// GameMode, MangoHud, and system performance tuning

pub mod gamemode;
pub mod mangohud;
pub mod performance;

use std::collections::HashMap;

/// Combined optimization settings for a game
#[derive(Debug, Clone)]
pub struct GameOptimization {
    pub gamemode_enabled: bool,
    pub mangohud_enabled: bool,
    pub mangohud_preset: mangohud::MangoHudPreset,
    pub performance_profile: performance::PerformanceProfile,
    pub custom_env_vars: HashMap<String, String>,
}

impl Default for GameOptimization {
    fn default() -> Self {
        Self {
            gamemode_enabled: true,
            mangohud_enabled: false,
            mangohud_preset: mangohud::MangoHudPreset::Default,
            performance_profile: performance::PerformanceProfile::Balanced,
            custom_env_vars: HashMap::new(),
        }
    }
}

impl GameOptimization {
    /// Create gaming-optimized settings
    pub fn gaming() -> Self {
        Self {
            gamemode_enabled: true,
            mangohud_enabled: true,
            mangohud_preset: mangohud::MangoHudPreset::Default,
            performance_profile: performance::PerformanceProfile::Performance,
            custom_env_vars: HashMap::new(),
        }
    }

    /// Create power-saving settings
    pub fn battery() -> Self {
        Self {
            gamemode_enabled: false,
            mangohud_enabled: false,
            mangohud_preset: mangohud::MangoHudPreset::Minimal,
            performance_profile: performance::PerformanceProfile::PowerSave,
            custom_env_vars: HashMap::new(),
        }
    }

    /// Get all environment variables for launching the game
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env = self.custom_env_vars.clone();

        // Add MangoHud vars if enabled
        if self.mangohud_enabled {
            env.extend(mangohud::get_mangohud_env(&self.mangohud_preset));
        }

        env
    }

    /// Get the launch prefix (gamemoderun, mangohud, etc.)
    pub fn get_launch_prefix(&self) -> Vec<String> {
        let mut prefix = Vec::new();

        if self.gamemode_enabled && gamemode::is_available() {
            prefix.push("gamemoderun".to_string());
        }

        if self.mangohud_enabled && mangohud::is_available() {
            prefix.push("mangohud".to_string());
        }

        prefix
    }
}

/// Apply optimizations before launching a game
pub fn apply_pre_launch_optimizations(opt: &GameOptimization) -> anyhow::Result<()> {
    // Apply performance profile
    performance::apply_profile(&opt.performance_profile)?;

    Ok(())
}

/// Restore system state after game exits
pub fn restore_post_game() -> anyhow::Result<()> {
    // GameMode handles this automatically
    // But we might want to restore other settings

    Ok(())
}
