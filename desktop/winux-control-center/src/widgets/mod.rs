//! Widgets module for Winux Control Center
//!
//! Contains all the individual control widgets for the control center panel.

pub mod airplane;
pub mod battery;
pub mod bluetooth;
pub mod brightness;
pub mod dnd;
pub mod media;
pub mod nightlight;
pub mod quick_toggle;
pub mod volume;
pub mod wifi;

// Re-export commonly used widgets
pub use airplane::AirplaneModeWidget;
pub use battery::BatteryWidget;
pub use bluetooth::BluetoothWidget;
pub use brightness::BrightnessWidget;
pub use dnd::DoNotDisturbWidget;
pub use media::MediaPlayerWidget;
pub use nightlight::NightLightWidget;
pub use quick_toggle::QuickToggleWidget;
pub use volume::VolumeWidget;
pub use wifi::WifiWidget;
