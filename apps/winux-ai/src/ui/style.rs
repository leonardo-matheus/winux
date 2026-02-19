// Style constants and utilities

/// Default application styles
pub const STYLE_CSS: &str = include_str!("style.css");

/// Icon names used in the application
pub mod icons {
    pub const APP_ICON: &str = "org.winux.ai";
    pub const SEND: &str = "paper-plane-symbolic";
    pub const ATTACH: &str = "mail-attachment-symbolic";
    pub const COPY: &str = "edit-copy-symbolic";
    pub const COPIED: &str = "emblem-ok-symbolic";
    pub const DELETE: &str = "user-trash-symbolic";
    pub const SETTINGS: &str = "emblem-system-symbolic";
    pub const NEW_CHAT: &str = "list-add-symbolic";
    pub const EXPORT: &str = "document-save-symbolic";
    pub const ERROR: &str = "dialog-error-symbolic";
    pub const USER: &str = "avatar-default-symbolic";
    pub const AI: &str = "face-smile-big-symbolic";
    pub const CODE: &str = "code-context-symbolic";
    pub const TERMINAL: &str = "utilities-terminal-symbolic";
    pub const FILE: &str = "text-x-generic-symbolic";
    pub const IMAGE: &str = "image-x-generic-symbolic";
    pub const TRANSLATE: &str = "accessories-dictionary-symbolic";
    pub const SEARCH: &str = "system-search-symbolic";
}

/// Color constants for theming
pub mod colors {
    pub const USER_BUBBLE_BG: &str = "@accent_bg_color";
    pub const ASSISTANT_BUBBLE_BG: &str = "@card_bg_color";
    pub const CODE_BLOCK_BG: &str = "@headerbar_bg_color";
    pub const ERROR_BG: &str = "alpha(@error_color, 0.1)";
}
