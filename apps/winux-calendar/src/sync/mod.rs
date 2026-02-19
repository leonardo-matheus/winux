//! Synchronization modules

mod caldav;
mod local;

pub use caldav::{CalDAVClient, CalDAVAccount};
pub use local::LocalStorage;
