// Pages module for Winux Power

mod battery;
mod profiles;
mod display;
mod devices;
mod statistics;

pub use battery::BatteryPage;
pub use profiles::ProfilesPage;
pub use display::DisplayPage;
pub use devices::DevicesPage;
pub use statistics::StatisticsPage;
