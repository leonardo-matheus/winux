//! Disk management pages module

mod overview;
mod disk_detail;
mod partition;
mod format;

pub use overview::OverviewPage;
pub use disk_detail::DiskDetailPage;
pub use partition::PartitionPage;
pub use format::FormatPage;
