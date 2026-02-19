//! Image processing module - filters and effects

mod filters;
mod effects;

pub use filters::{FilterType, apply_filter, FilterPreset};
pub use effects::{Effect, apply_effect, EffectParameters};
