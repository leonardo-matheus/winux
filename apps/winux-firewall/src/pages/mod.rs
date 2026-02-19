//! Firewall pages module

mod overview;
mod rules;
mod apps;
mod logs;

pub use overview::OverviewPage;
pub use rules::RulesPage;
pub use apps::AppsPage;
pub use logs::LogsPage;
