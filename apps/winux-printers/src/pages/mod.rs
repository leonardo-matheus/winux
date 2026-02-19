//! Pages for the printer manager

mod printers;
mod add;
mod jobs;
mod settings;

pub use printers::PrintersPage;
pub use add::AddPrinterPage;
pub use jobs::JobsPage;
pub use settings::SettingsPage;
