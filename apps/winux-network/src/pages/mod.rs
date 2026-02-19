//! Network pages module

mod wifi;
mod ethernet;
mod vpn;
mod hotspot;
mod proxy;
mod advanced;

pub use wifi::WifiPage;
pub use ethernet::EthernetPage;
pub use vpn::VpnPage;
pub use hotspot::HotspotPage;
pub use proxy::ProxyPage;
pub use advanced::AdvancedPage;
