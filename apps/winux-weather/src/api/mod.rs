//! Weather API clients

mod openmeteo;
mod openweather;
mod location;

pub use openmeteo::OpenMeteoClient;
pub use openweather::OpenWeatherClient;
pub use location::{Location, get_current_location};
