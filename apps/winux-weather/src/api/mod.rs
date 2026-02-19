//! Weather API clients

mod openmeteo;
mod location;

pub use openmeteo::OpenMeteoClient;
pub use location::{Location, get_current_location};
