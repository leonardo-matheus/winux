//! Output module - handles saving, clipboard, and sharing

pub mod save;
pub mod clipboard;
pub mod upload;

pub use save::{save_screenshot, SaveFormat};
pub use clipboard::copy_to_clipboard;
pub use upload::UploadService;
