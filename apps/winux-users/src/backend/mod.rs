//! Backend module - System integration

mod accounts;
mod passwd;

pub use accounts::AccountsService;
pub use passwd::{PasswdEntry, GroupEntry, parse_passwd, parse_group, parse_shadow};
