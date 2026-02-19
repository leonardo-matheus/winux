//! System commands provider

use crate::search::{SearchCategory, SearchResult, SearchResultKind};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// System command definition
struct SystemCommand {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
    icon: &'static str,
    command: &'static str,
    dangerous: bool,
}

/// Available system commands
const COMMANDS: &[SystemCommand] = &[
    SystemCommand {
        name: "Shutdown",
        aliases: &["poweroff", "power off", "turn off", "desligar"],
        description: "Turn off the computer",
        icon: "system-shutdown-symbolic",
        command: "shutdown",
        dangerous: true,
    },
    SystemCommand {
        name: "Restart",
        aliases: &["reboot", "reiniciar"],
        description: "Restart the computer",
        icon: "system-reboot-symbolic",
        command: "restart",
        dangerous: true,
    },
    SystemCommand {
        name: "Log Out",
        aliases: &["logout", "sign out", "sair"],
        description: "Log out of your session",
        icon: "system-log-out-symbolic",
        command: "logout",
        dangerous: true,
    },
    SystemCommand {
        name: "Lock Screen",
        aliases: &["lock", "bloquear"],
        description: "Lock the screen",
        icon: "system-lock-screen-symbolic",
        command: "lock",
        dangerous: false,
    },
    SystemCommand {
        name: "Sleep",
        aliases: &["suspend", "suspender"],
        description: "Put the computer to sleep",
        icon: "weather-clear-night-symbolic",
        command: "sleep",
        dangerous: false,
    },
    SystemCommand {
        name: "Settings",
        aliases: &["preferences", "config", "configuracoes"],
        description: "Open system settings",
        icon: "preferences-system-symbolic",
        command: "settings",
        dangerous: false,
    },
    SystemCommand {
        name: "Files",
        aliases: &["file manager", "nautilus", "arquivos"],
        description: "Open file manager",
        icon: "system-file-manager-symbolic",
        command: "files",
        dangerous: false,
    },
    SystemCommand {
        name: "Terminal",
        aliases: &["console", "shell", "cmd"],
        description: "Open terminal",
        icon: "utilities-terminal-symbolic",
        command: "terminal",
        dangerous: false,
    },
    SystemCommand {
        name: "Display Settings",
        aliases: &["monitor", "screen", "display"],
        description: "Configure displays",
        icon: "preferences-desktop-display-symbolic",
        command: "settings display",
        dangerous: false,
    },
    SystemCommand {
        name: "Sound Settings",
        aliases: &["audio", "volume", "som"],
        description: "Configure sound",
        icon: "audio-volume-high-symbolic",
        command: "settings sound",
        dangerous: false,
    },
    SystemCommand {
        name: "Network Settings",
        aliases: &["wifi", "internet", "rede"],
        description: "Configure network",
        icon: "network-wireless-symbolic",
        command: "settings network",
        dangerous: false,
    },
    SystemCommand {
        name: "Bluetooth Settings",
        aliases: &["bluetooth"],
        description: "Configure Bluetooth",
        icon: "bluetooth-active-symbolic",
        command: "settings bluetooth",
        dangerous: false,
    },
    SystemCommand {
        name: "About This Computer",
        aliases: &["system info", "about", "sobre"],
        description: "View system information",
        icon: "help-about-symbolic",
        command: "about",
        dangerous: false,
    },
    SystemCommand {
        name: "Empty Trash",
        aliases: &["trash", "lixeira"],
        description: "Empty the trash",
        icon: "user-trash-symbolic",
        command: "trash --empty",
        dangerous: true,
    },
];

/// System command searcher
pub struct CommandSearcher {
    matcher: SkimMatcherV2,
}

impl CommandSearcher {
    /// Create new command searcher
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Search for matching commands
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(i64, SearchResult)> = Vec::new();

        for cmd in COMMANDS {
            let mut best_score: i64 = 0;

            // Match against name
            if let Some(score) = self.matcher.fuzzy_match(&cmd.name.to_lowercase(), &query_lower) {
                best_score = best_score.max(score);
            }

            // Match against aliases
            for alias in cmd.aliases {
                if let Some(score) = self.matcher.fuzzy_match(&alias.to_lowercase(), &query_lower) {
                    best_score = best_score.max(score);
                }
            }

            // Match against description
            if let Some(score) = self.matcher.fuzzy_match(&cmd.description.to_lowercase(), &query_lower) {
                best_score = best_score.max(score / 2); // Lower weight for description
            }

            if best_score > 20 {
                let result = SearchResult {
                    id: format!("cmd:{}", cmd.command),
                    title: cmd.name.to_string(),
                    subtitle: cmd.description.to_string(),
                    icon: cmd.icon.to_string(),
                    category: SearchCategory::Commands,
                    kind: SearchResultKind::Command {
                        command: cmd.command.to_string(),
                    },
                    score: best_score.min(100) as u32,
                    from_history: false,
                };

                results.push((best_score, result));
            }
        }

        // Sort by score and take top results
        results.sort_by(|a, b| b.0.cmp(&a.0));
        results.into_iter().take(3).map(|(_, r)| r).collect()
    }
}

impl Default for CommandSearcher {
    fn default() -> Self {
        Self::new()
    }
}
