// AI Model definitions and settings

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Model {
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    #[serde(rename = "o1")]
    O1,
}

impl Model {
    pub fn name(&self) -> &str {
        match self {
            Model::Gpt4o => "GPT-4o",
            Model::O1 => "o1",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Model::Gpt4o => "Fast and capable multimodal model with vision support",
            Model::O1 => "Advanced reasoning model for complex problem solving",
        }
    }

    pub fn max_context(&self) -> u32 {
        match self {
            Model::Gpt4o => 128000,
            Model::O1 => 200000,
        }
    }

    pub fn supports_vision(&self) -> bool {
        match self {
            Model::Gpt4o => true,
            Model::O1 => false,
        }
    }

    pub fn supports_streaming(&self) -> bool {
        match self {
            Model::Gpt4o => true,
            Model::O1 => false,
        }
    }

    pub fn supports_temperature(&self) -> bool {
        match self {
            Model::Gpt4o => true,
            Model::O1 => false, // o1 uses fixed temperature
        }
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for Model {
    fn default() -> Self {
        Model::Gpt4o
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_model: Model,
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_prompt: String,
    pub streaming_enabled: bool,
    pub include_system_context: bool,
    pub terminal_integration: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_model: Model::Gpt4o,
            temperature: 0.7,
            max_tokens: 4096,
            system_prompt: String::new(),
            streaming_enabled: true,
            include_system_context: true,
            terminal_integration: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let config_path = dirs::config_dir()
            .map(|p| p.join("winux-ai").join("settings.toml"));

        if let Some(path) = config_path {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(settings) = toml::from_str(&content) {
                        return settings;
                    }
                }
            }
        }

        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("winux-ai");

        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("settings.toml");

        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;

        Ok(())
    }
}
