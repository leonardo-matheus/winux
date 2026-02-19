//! Settings provider API
//!
//! Allows plugins to add settings pages to the system settings application.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A setting value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// List of strings
    StringList(Vec<String>),
    /// Color value (RGBA)
    Color { r: u8, g: u8, b: u8, a: u8 },
    /// File path
    Path(String),
    /// Enum selection
    Enum { value: String, options: Vec<String> },
    /// Key binding
    KeyBinding(String),
}

impl Default for SettingValue {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl SettingValue {
    /// Get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as int
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            Self::Path(v) => Some(v),
            Self::KeyBinding(v) => Some(v),
            Self::Enum { value, .. } => Some(value),
            _ => None,
        }
    }
}

/// Type of setting control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingType {
    /// Toggle switch
    Toggle,
    /// Text entry
    Text,
    /// Number spinner
    Spinner { min: f64, max: f64, step: f64 },
    /// Slider
    Slider { min: f64, max: f64, step: f64 },
    /// Dropdown selection
    Dropdown { options: Vec<(String, String)> },
    /// Radio buttons
    Radio { options: Vec<(String, String)> },
    /// Color picker
    Color,
    /// Font picker
    Font,
    /// File picker
    File { filter: Option<String> },
    /// Folder picker
    Folder,
    /// Key binding capture
    KeyBinding,
    /// Password entry
    Password,
    /// Multi-line text
    TextArea { rows: u32 },
    /// List editor
    List,
    /// Button (triggers action)
    Button { label: String },
    /// Custom widget
    Custom { widget_id: String },
}

impl Default for SettingType {
    fn default() -> Self {
        Self::Toggle
    }
}

/// A single setting definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    /// Setting key
    pub key: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Setting type
    pub setting_type: SettingType,
    /// Default value
    pub default: SettingValue,
    /// Current value
    pub value: SettingValue,
    /// Whether the setting requires restart
    pub requires_restart: bool,
    /// Whether this setting is experimental
    pub experimental: bool,
    /// Dependency on another setting (show only if condition met)
    pub depends_on: Option<SettingDependency>,
    /// Keywords for search
    pub keywords: Vec<String>,
}

impl Setting {
    /// Create a toggle setting
    pub fn toggle(key: &str, name: &str, default: bool) -> Self {
        Self {
            key: key.to_string(),
            name: name.to_string(),
            description: None,
            setting_type: SettingType::Toggle,
            default: SettingValue::Bool(default),
            value: SettingValue::Bool(default),
            requires_restart: false,
            experimental: false,
            depends_on: None,
            keywords: Vec::new(),
        }
    }

    /// Create a text setting
    pub fn text(key: &str, name: &str, default: &str) -> Self {
        Self {
            key: key.to_string(),
            name: name.to_string(),
            description: None,
            setting_type: SettingType::Text,
            default: SettingValue::String(default.to_string()),
            value: SettingValue::String(default.to_string()),
            requires_restart: false,
            experimental: false,
            depends_on: None,
            keywords: Vec::new(),
        }
    }

    /// Create a slider setting
    pub fn slider(key: &str, name: &str, min: f64, max: f64, step: f64, default: f64) -> Self {
        Self {
            key: key.to_string(),
            name: name.to_string(),
            description: None,
            setting_type: SettingType::Slider { min, max, step },
            default: SettingValue::Float(default),
            value: SettingValue::Float(default),
            requires_restart: false,
            experimental: false,
            depends_on: None,
            keywords: Vec::new(),
        }
    }

    /// Create a dropdown setting
    pub fn dropdown(key: &str, name: &str, options: &[(&str, &str)], default: &str) -> Self {
        let options: Vec<(String, String)> = options
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Self {
            key: key.to_string(),
            name: name.to_string(),
            description: None,
            setting_type: SettingType::Dropdown { options: options.clone() },
            default: SettingValue::String(default.to_string()),
            value: SettingValue::String(default.to_string()),
            requires_restart: false,
            experimental: false,
            depends_on: None,
            keywords: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Mark as requiring restart
    pub fn requires_restart(mut self) -> Self {
        self.requires_restart = true;
        self
    }

    /// Mark as experimental
    pub fn experimental(mut self) -> Self {
        self.experimental = true;
        self
    }

    /// Set dependency
    pub fn depends_on(mut self, dependency: SettingDependency) -> Self {
        self.depends_on = Some(dependency);
        self
    }

    /// Add keywords
    pub fn keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Setting dependency condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDependency {
    /// Key of the setting to depend on
    pub key: String,
    /// Required value
    pub value: SettingValue,
    /// Whether to invert the condition
    pub invert: bool,
}

impl SettingDependency {
    /// Create a dependency that shows when toggle is on
    pub fn when_enabled(key: &str) -> Self {
        Self {
            key: key.to_string(),
            value: SettingValue::Bool(true),
            invert: false,
        }
    }

    /// Create a dependency that shows when toggle is off
    pub fn when_disabled(key: &str) -> Self {
        Self {
            key: key.to_string(),
            value: SettingValue::Bool(false),
            invert: false,
        }
    }

    /// Create a dependency on a specific value
    pub fn when_equals(key: &str, value: SettingValue) -> Self {
        Self {
            key: key.to_string(),
            value,
            invert: false,
        }
    }
}

/// A group of settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingGroup {
    /// Group title
    pub title: String,
    /// Group description
    pub description: Option<String>,
    /// Settings in this group
    pub settings: Vec<Setting>,
}

impl SettingGroup {
    /// Create a new setting group
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            description: None,
            settings: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Add a setting
    pub fn add(mut self, setting: Setting) -> Self {
        self.settings.push(setting);
        self
    }
}

/// A settings page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPage {
    /// Page ID
    pub id: String,
    /// Page title
    pub title: String,
    /// Page description
    pub description: Option<String>,
    /// Page icon
    pub icon: String,
    /// Setting groups
    pub groups: Vec<SettingGroup>,
    /// Keywords for search
    pub keywords: Vec<String>,
    /// Parent page ID (for nested pages)
    pub parent: Option<String>,
    /// Sort priority
    pub priority: i32,
}

impl SettingsPage {
    /// Create a new settings page
    pub fn new(id: &str, title: &str, icon: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            icon: icon.to_string(),
            groups: Vec::new(),
            keywords: Vec::new(),
            parent: None,
            priority: 0,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Add a setting group
    pub fn add_group(mut self, group: SettingGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// Set parent page
    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_string());
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add keywords
    pub fn keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Trait for settings providers
pub trait SettingsProvider: Send + Sync {
    /// Get provider ID
    fn id(&self) -> &str;

    /// Get settings pages provided
    fn pages(&self) -> Vec<SettingsPage>;

    /// Load settings from storage
    fn load(&mut self) -> HashMap<String, SettingValue>;

    /// Save a setting value
    fn save(&mut self, key: &str, value: SettingValue) -> Result<(), String>;

    /// Reset settings to defaults
    fn reset(&mut self) -> Result<(), String>;

    /// Reset a single setting to default
    fn reset_setting(&mut self, key: &str) -> Result<(), String>;

    /// Called when a setting changes
    fn on_change(&mut self, key: &str, value: &SettingValue) {
        let _ = (key, value);
    }

    /// Validate a setting value before saving
    fn validate(&self, key: &str, value: &SettingValue) -> Result<(), String> {
        let _ = (key, value);
        Ok(())
    }

    /// Get export data (for backup)
    fn export(&self) -> HashMap<String, SettingValue> {
        HashMap::new()
    }

    /// Import data (for restore)
    fn import(&mut self, _data: HashMap<String, SettingValue>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_value_as_bool() {
        let value = SettingValue::Bool(true);
        assert_eq!(value.as_bool(), Some(true));

        let value = SettingValue::String("test".to_string());
        assert_eq!(value.as_bool(), None);
    }

    #[test]
    fn test_setting_builder() {
        let setting = Setting::toggle("test.enabled", "Enable Test", true)
            .with_description("Enables the test feature")
            .requires_restart()
            .keywords(&["test", "enable"]);

        assert_eq!(setting.key, "test.enabled");
        assert!(setting.requires_restart);
        assert_eq!(setting.keywords.len(), 2);
    }

    #[test]
    fn test_settings_page_builder() {
        let page = SettingsPage::new("test", "Test Settings", "preferences-system")
            .with_description("Test settings page")
            .add_group(
                SettingGroup::new("General")
                    .add(Setting::toggle("test.enabled", "Enable", true)),
            );

        assert_eq!(page.groups.len(), 1);
        assert_eq!(page.groups[0].settings.len(), 1);
    }
}
