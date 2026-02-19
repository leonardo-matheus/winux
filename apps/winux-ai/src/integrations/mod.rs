// Integrations module - System and external integrations

mod clipboard;
mod files;
mod system;
mod terminal;

pub use clipboard::ClipboardManager;
pub use files::FileManager;
pub use system::SystemInfo;
pub use terminal::TerminalExecutor;
