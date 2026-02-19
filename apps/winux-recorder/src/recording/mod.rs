//! Recording management module

pub mod session;
pub mod storage;

pub use session::{RecordingSession, RecordingState};
pub use storage::{Recording, RecordingMetadata};
