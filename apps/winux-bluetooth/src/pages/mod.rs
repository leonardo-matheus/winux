//! Pages for the Bluetooth manager

mod devices;
mod scan;
mod transfer;
mod settings;

pub use devices::DevicesPage;
pub use scan::ScanPage;
pub use transfer::TransferPage;
pub use settings::SettingsPage;
