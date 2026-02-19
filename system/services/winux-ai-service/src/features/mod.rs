//! AI features module

pub mod complete;
pub mod chat;
pub mod summarize;
pub mod translate;
pub mod code;
pub mod vision;

pub use complete::CompleteFeature;
pub use chat::ChatFeature;
pub use summarize::SummarizeFeature;
pub use translate::TranslateFeature;
pub use code::CodeFeature;
pub use vision::VisionFeature;
