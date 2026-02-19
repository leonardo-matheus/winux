//! Command palette API
//!
//! Allows plugins to register commands that can be executed from the command palette.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command execution result
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Command executed successfully
    Success,
    /// Command executed with a message
    Message(String),
    /// Command opened something (close palette)
    Opened,
    /// Command failed
    Error(String),
    /// Command requires additional input
    RequiresInput {
        prompt: String,
        placeholder: Option<String>,
    },
    /// Command shows options to choose from
    ShowOptions(Vec<CommandOption>),
}

/// An option for multi-step commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOption {
    /// Option ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Description
    pub description: Option<String>,
    /// Icon
    pub icon: Option<String>,
}

impl CommandOption {
    /// Create a new option
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            description: None,
            icon: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set icon
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
}

/// Context provided to commands
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Current working directory
    pub cwd: String,
    /// Selected text (if any)
    pub selection: Option<String>,
    /// Active file path (if any)
    pub active_file: Option<String>,
    /// Arguments passed to the command
    pub args: Vec<String>,
    /// User input (for RequiresInput results)
    pub input: Option<String>,
    /// Selected option (for ShowOptions results)
    pub selected_option: Option<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            selection: None,
            active_file: None,
            args: Vec::new(),
            input: None,
            selected_option: None,
            env: std::env::vars().collect(),
        }
    }
}

/// A command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Unique command ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Icon
    pub icon: Option<String>,
    /// Category for grouping
    pub category: Option<String>,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Whether command requires confirmation
    pub confirm: bool,
    /// Confirmation message
    pub confirm_message: Option<String>,
    /// Keywords for search
    pub keywords: Vec<String>,
    /// Whether this command is hidden from palette
    pub hidden: bool,
    /// Whether command can run in background
    pub background: bool,
    /// When condition (show only when true)
    pub when: Option<String>,
}

impl Command {
    /// Create a new command
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            icon: None,
            category: None,
            shortcut: None,
            confirm: false,
            confirm_message: None,
            keywords: Vec::new(),
            hidden: false,
            background: false,
            when: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set icon
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Set category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Set keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    /// Require confirmation before executing
    pub fn confirm(mut self, message: Option<&str>) -> Self {
        self.confirm = true;
        self.confirm_message = message.map(String::from);
        self
    }

    /// Add keywords
    pub fn keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Mark as hidden
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Allow background execution
    pub fn background(mut self) -> Self {
        self.background = true;
        self
    }

    /// Set when condition
    pub fn when(mut self, condition: &str) -> Self {
        self.when = Some(condition.to_string());
        self
    }
}

/// Command category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCategory {
    /// Category ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Icon
    pub icon: Option<String>,
    /// Priority (higher = shown first)
    pub priority: i32,
}

impl CommandCategory {
    /// Create a new category
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            icon: None,
            priority: 0,
        }
    }

    /// Set icon
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Trait for command providers
pub trait CommandProvider: Send + Sync {
    /// Get provider ID
    fn id(&self) -> &str;

    /// Get provided commands
    fn commands(&self) -> Vec<Command>;

    /// Get command categories
    fn categories(&self) -> Vec<CommandCategory> {
        Vec::new()
    }

    /// Execute a command
    fn execute(&mut self, command_id: &str, context: &CommandContext) -> CommandResult;

    /// Check if a command is enabled
    fn is_enabled(&self, command_id: &str, context: &CommandContext) -> bool {
        let _ = (command_id, context);
        true
    }

    /// Get command completions (for command line input)
    fn completions(&self, command_id: &str, partial: &str) -> Vec<String> {
        let _ = (command_id, partial);
        Vec::new()
    }
}

/// Builder for creating command providers with closures
pub struct SimpleCommandProvider {
    id: String,
    commands: Vec<(Command, Box<dyn Fn(&CommandContext) -> CommandResult + Send + Sync>)>,
    categories: Vec<CommandCategory>,
}

impl SimpleCommandProvider {
    /// Create a new simple command provider
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            commands: Vec::new(),
            categories: Vec::new(),
        }
    }

    /// Add a command with a handler
    pub fn add_command<F>(mut self, command: Command, handler: F) -> Self
    where
        F: Fn(&CommandContext) -> CommandResult + Send + Sync + 'static,
    {
        self.commands.push((command, Box::new(handler)));
        self
    }

    /// Add a category
    pub fn add_category(mut self, category: CommandCategory) -> Self {
        self.categories.push(category);
        self
    }
}

impl CommandProvider for SimpleCommandProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn commands(&self) -> Vec<Command> {
        self.commands.iter().map(|(c, _)| c.clone()).collect()
    }

    fn categories(&self) -> Vec<CommandCategory> {
        self.categories.clone()
    }

    fn execute(&mut self, command_id: &str, context: &CommandContext) -> CommandResult {
        for (command, handler) in &self.commands {
            if command.id == command_id {
                return handler(context);
            }
        }
        CommandResult::Error(format!("Command not found: {}", command_id))
    }
}

/// Common built-in command IDs
pub mod builtin {
    /// Open settings
    pub const OPEN_SETTINGS: &str = "winux.openSettings";
    /// Open terminal
    pub const OPEN_TERMINAL: &str = "winux.openTerminal";
    /// Open files
    pub const OPEN_FILES: &str = "winux.openFiles";
    /// Toggle do not disturb
    pub const TOGGLE_DND: &str = "winux.toggleDND";
    /// Lock screen
    pub const LOCK_SCREEN: &str = "winux.lockScreen";
    /// Log out
    pub const LOG_OUT: &str = "winux.logOut";
    /// Show about
    pub const SHOW_ABOUT: &str = "winux.showAbout";
    /// Reload shell
    pub const RELOAD_SHELL: &str = "winux.reloadShell";
    /// Toggle dark mode
    pub const TOGGLE_DARK_MODE: &str = "winux.toggleDarkMode";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_builder() {
        let command = Command::new("test.command", "Test Command")
            .with_description("A test command")
            .with_icon("system-run")
            .with_shortcut("Ctrl+T")
            .keywords(&["test", "example"]);

        assert_eq!(command.id, "test.command");
        assert_eq!(command.shortcut, Some("Ctrl+T".to_string()));
        assert_eq!(command.keywords.len(), 2);
    }

    #[test]
    fn test_simple_command_provider() {
        let mut provider = SimpleCommandProvider::new("test")
            .add_command(Command::new("test.hello", "Say Hello"), |_ctx| {
                CommandResult::Message("Hello!".to_string())
            });

        let ctx = CommandContext::default();
        let result = provider.execute("test.hello", &ctx);

        match result {
            CommandResult::Message(msg) => assert_eq!(msg, "Hello!"),
            _ => panic!("Expected Message result"),
        }
    }
}
