//! Theme management for Winux Terminal

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Terminal color theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Background color
    pub background: String,
    /// Foreground color
    pub foreground: String,
    /// Cursor color
    pub cursor: Option<String>,
    /// Cursor text color
    pub cursor_text: Option<String>,
    /// Selection background color
    pub selection: Option<String>,
    /// Selection text color
    pub selection_text: Option<String>,
    /// 16-color palette (ANSI colors 0-15)
    pub palette: Vec<String>,
    /// Bold color
    pub bold: Option<String>,
    /// Dim color modifier
    pub dim: Option<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self::winux_dark()
    }
}

impl Theme {
    /// Winux Dark theme (default)
    pub fn winux_dark() -> Self {
        Self {
            name: "winux-dark".to_string(),
            background: "#1a1b26".to_string(),
            foreground: "#c0caf5".to_string(),
            cursor: Some("#c0caf5".to_string()),
            cursor_text: Some("#1a1b26".to_string()),
            selection: Some("#33467c".to_string()),
            selection_text: Some("#c0caf5".to_string()),
            palette: vec![
                "#15161e".to_string(), // Black
                "#f7768e".to_string(), // Red
                "#9ece6a".to_string(), // Green
                "#e0af68".to_string(), // Yellow
                "#7aa2f7".to_string(), // Blue
                "#bb9af7".to_string(), // Magenta
                "#7dcfff".to_string(), // Cyan
                "#a9b1d6".to_string(), // White
                "#414868".to_string(), // Bright Black
                "#f7768e".to_string(), // Bright Red
                "#9ece6a".to_string(), // Bright Green
                "#e0af68".to_string(), // Bright Yellow
                "#7aa2f7".to_string(), // Bright Blue
                "#bb9af7".to_string(), // Bright Magenta
                "#7dcfff".to_string(), // Bright Cyan
                "#c0caf5".to_string(), // Bright White
            ],
            bold: None,
            dim: None,
        }
    }

    /// Winux Light theme
    pub fn winux_light() -> Self {
        Self {
            name: "winux-light".to_string(),
            background: "#f5f5f5".to_string(),
            foreground: "#1a1b26".to_string(),
            cursor: Some("#1a1b26".to_string()),
            cursor_text: Some("#f5f5f5".to_string()),
            selection: Some("#c0caf5".to_string()),
            selection_text: Some("#1a1b26".to_string()),
            palette: vec![
                "#1a1b26".to_string(), // Black
                "#d32f2f".to_string(), // Red
                "#388e3c".to_string(), // Green
                "#f57c00".to_string(), // Yellow
                "#1976d2".to_string(), // Blue
                "#7b1fa2".to_string(), // Magenta
                "#0097a7".to_string(), // Cyan
                "#757575".to_string(), // White
                "#424242".to_string(), // Bright Black
                "#e53935".to_string(), // Bright Red
                "#43a047".to_string(), // Bright Green
                "#fb8c00".to_string(), // Bright Yellow
                "#1e88e5".to_string(), // Bright Blue
                "#8e24aa".to_string(), // Bright Magenta
                "#00acc1".to_string(), // Bright Cyan
                "#bdbdbd".to_string(), // Bright White
            ],
            bold: None,
            dim: None,
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            background: "#282a36".to_string(),
            foreground: "#f8f8f2".to_string(),
            cursor: Some("#f8f8f2".to_string()),
            cursor_text: Some("#282a36".to_string()),
            selection: Some("#44475a".to_string()),
            selection_text: Some("#f8f8f2".to_string()),
            palette: vec![
                "#21222c".to_string(), // Black
                "#ff5555".to_string(), // Red
                "#50fa7b".to_string(), // Green
                "#f1fa8c".to_string(), // Yellow
                "#bd93f9".to_string(), // Blue
                "#ff79c6".to_string(), // Magenta
                "#8be9fd".to_string(), // Cyan
                "#f8f8f2".to_string(), // White
                "#6272a4".to_string(), // Bright Black
                "#ff6e6e".to_string(), // Bright Red
                "#69ff94".to_string(), // Bright Green
                "#ffffa5".to_string(), // Bright Yellow
                "#d6acff".to_string(), // Bright Blue
                "#ff92df".to_string(), // Bright Magenta
                "#a4ffff".to_string(), // Bright Cyan
                "#ffffff".to_string(), // Bright White
            ],
            bold: None,
            dim: None,
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            background: "#2e3440".to_string(),
            foreground: "#d8dee9".to_string(),
            cursor: Some("#d8dee9".to_string()),
            cursor_text: Some("#2e3440".to_string()),
            selection: Some("#434c5e".to_string()),
            selection_text: Some("#d8dee9".to_string()),
            palette: vec![
                "#3b4252".to_string(), // Black
                "#bf616a".to_string(), // Red
                "#a3be8c".to_string(), // Green
                "#ebcb8b".to_string(), // Yellow
                "#81a1c1".to_string(), // Blue
                "#b48ead".to_string(), // Magenta
                "#88c0d0".to_string(), // Cyan
                "#e5e9f0".to_string(), // White
                "#4c566a".to_string(), // Bright Black
                "#bf616a".to_string(), // Bright Red
                "#a3be8c".to_string(), // Bright Green
                "#ebcb8b".to_string(), // Bright Yellow
                "#81a1c1".to_string(), // Bright Blue
                "#b48ead".to_string(), // Bright Magenta
                "#8fbcbb".to_string(), // Bright Cyan
                "#eceff4".to_string(), // Bright White
            ],
            bold: None,
            dim: None,
        }
    }

    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark".to_string(),
            background: "#002b36".to_string(),
            foreground: "#839496".to_string(),
            cursor: Some("#839496".to_string()),
            cursor_text: Some("#002b36".to_string()),
            selection: Some("#073642".to_string()),
            selection_text: Some("#93a1a1".to_string()),
            palette: vec![
                "#073642".to_string(), // Black
                "#dc322f".to_string(), // Red
                "#859900".to_string(), // Green
                "#b58900".to_string(), // Yellow
                "#268bd2".to_string(), // Blue
                "#d33682".to_string(), // Magenta
                "#2aa198".to_string(), // Cyan
                "#eee8d5".to_string(), // White
                "#002b36".to_string(), // Bright Black
                "#cb4b16".to_string(), // Bright Red
                "#586e75".to_string(), // Bright Green
                "#657b83".to_string(), // Bright Yellow
                "#839496".to_string(), // Bright Blue
                "#6c71c4".to_string(), // Bright Magenta
                "#93a1a1".to_string(), // Bright Cyan
                "#fdf6e3".to_string(), // Bright White
            ],
            bold: None,
            dim: None,
        }
    }

    /// Monokai theme
    pub fn monokai() -> Self {
        Self {
            name: "monokai".to_string(),
            background: "#272822".to_string(),
            foreground: "#f8f8f2".to_string(),
            cursor: Some("#f8f8f2".to_string()),
            cursor_text: Some("#272822".to_string()),
            selection: Some("#49483e".to_string()),
            selection_text: Some("#f8f8f2".to_string()),
            palette: vec![
                "#272822".to_string(), // Black
                "#f92672".to_string(), // Red
                "#a6e22e".to_string(), // Green
                "#f4bf75".to_string(), // Yellow
                "#66d9ef".to_string(), // Blue
                "#ae81ff".to_string(), // Magenta
                "#a1efe4".to_string(), // Cyan
                "#f8f8f2".to_string(), // White
                "#75715e".to_string(), // Bright Black
                "#f92672".to_string(), // Bright Red
                "#a6e22e".to_string(), // Bright Green
                "#f4bf75".to_string(), // Bright Yellow
                "#66d9ef".to_string(), // Bright Blue
                "#ae81ff".to_string(), // Bright Magenta
                "#a1efe4".to_string(), // Bright Cyan
                "#f9f8f5".to_string(), // Bright White
            ],
            bold: None,
            dim: None,
        }
    }

    /// Gaming theme (high contrast for gaming)
    pub fn gaming() -> Self {
        Self {
            name: "gaming".to_string(),
            background: "#0d0d0d".to_string(),
            foreground: "#00ff00".to_string(),
            cursor: Some("#00ff00".to_string()),
            cursor_text: Some("#0d0d0d".to_string()),
            selection: Some("#003300".to_string()),
            selection_text: Some("#00ff00".to_string()),
            palette: vec![
                "#0d0d0d".to_string(), // Black
                "#ff0055".to_string(), // Red
                "#00ff00".to_string(), // Green
                "#ffff00".to_string(), // Yellow
                "#0066ff".to_string(), // Blue
                "#ff00ff".to_string(), // Magenta
                "#00ffff".to_string(), // Cyan
                "#ffffff".to_string(), // White
                "#333333".to_string(), // Bright Black
                "#ff3377".to_string(), // Bright Red
                "#33ff33".to_string(), // Bright Green
                "#ffff33".to_string(), // Bright Yellow
                "#3388ff".to_string(), // Bright Blue
                "#ff33ff".to_string(), // Bright Magenta
                "#33ffff".to_string(), // Bright Cyan
                "#ffffff".to_string(), // Bright White
            ],
            bold: None,
            dim: None,
        }
    }
}

/// Theme manager
pub struct ThemeManager {
    /// Built-in themes
    themes: HashMap<String, Theme>,
    /// Custom themes directory
    custom_themes_dir: PathBuf,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Register built-in themes
        let builtins = vec![
            Theme::winux_dark(),
            Theme::winux_light(),
            Theme::dracula(),
            Theme::nord(),
            Theme::solarized_dark(),
            Theme::monokai(),
            Theme::gaming(),
        ];

        for theme in builtins {
            themes.insert(theme.name.clone(), theme);
        }

        let custom_themes_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-terminal")
            .join("themes");

        let mut manager = ThemeManager {
            themes,
            custom_themes_dir,
        };

        // Load custom themes
        manager.load_custom_themes();

        manager
    }

    /// Load custom themes from directory
    fn load_custom_themes(&mut self) {
        if !self.custom_themes_dir.exists() {
            return;
        }

        debug!("Loading custom themes from {:?}", self.custom_themes_dir);

        if let Ok(entries) = std::fs::read_dir(&self.custom_themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "toml").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(theme) = toml::from_str::<Theme>(&content) {
                            info!("Loaded custom theme: {}", theme.name);
                            self.themes.insert(theme.name.clone(), theme);
                        }
                    }
                }
            }
        }
    }

    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Theme {
        self.themes
            .get(name)
            .cloned()
            .unwrap_or_else(Theme::winux_dark)
    }

    /// List available themes
    pub fn list_themes(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// Save a custom theme
    pub fn save_theme(&self, theme: &Theme) -> Result<()> {
        std::fs::create_dir_all(&self.custom_themes_dir)?;

        let path = self.custom_themes_dir.join(format!("{}.toml", theme.name));
        let content = toml::to_string_pretty(theme)?;
        std::fs::write(&path, content)?;

        info!("Saved theme: {}", theme.name);
        Ok(())
    }

    /// Delete a custom theme
    pub fn delete_theme(&mut self, name: &str) -> Result<()> {
        let path = self.custom_themes_dir.join(format!("{}.toml", name));
        if path.exists() {
            std::fs::remove_file(&path)?;
            self.themes.remove(name);
            info!("Deleted theme: {}", name);
        }
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_themes() {
        let manager = ThemeManager::new();

        assert!(manager.themes.contains_key("winux-dark"));
        assert!(manager.themes.contains_key("winux-light"));
        assert!(manager.themes.contains_key("dracula"));
        assert!(manager.themes.contains_key("nord"));
    }

    #[test]
    fn test_get_theme() {
        let manager = ThemeManager::new();

        let theme = manager.get_theme("winux-dark");
        assert_eq!(theme.name, "winux-dark");

        // Unknown theme should return default
        let unknown = manager.get_theme("nonexistent");
        assert_eq!(unknown.name, "winux-dark");
    }

    #[test]
    fn test_theme_palette() {
        let theme = Theme::winux_dark();
        assert_eq!(theme.palette.len(), 16);
    }

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::dracula();
        let toml = toml::to_string(&theme).unwrap();
        let parsed: Theme = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.name, theme.name);
        assert_eq!(parsed.background, theme.background);
    }
}
