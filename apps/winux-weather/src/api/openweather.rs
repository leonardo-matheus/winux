//! OpenWeatherMap API client (alternative provider)
//!
//! This module provides an alternative weather API client using OpenWeatherMap.
//! Note: OpenWeatherMap requires an API key for usage.
//! Get a free API key at: https://openweathermap.org/api
//!
//! For the default implementation, the app uses Open-Meteo which is free and
//! requires no API key.

use anyhow::Result;
use serde::Deserialize;

use crate::api::Location;
use crate::data::{
    CurrentWeather, DailyForecast, HourlyForecast, WeatherCondition, WeatherData, WindDirection,
};

/// OpenWeatherMap API client
pub struct OpenWeatherClient {
    api_key: String,
}

impl OpenWeatherClient {
    const BASE_URL: &'static str = "https://api.openweathermap.org/data/2.5";
    const GEO_URL: &'static str = "https://api.openweathermap.org/geo/1.0";

    /// Create a new client with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// Fetch current weather for a location
    pub async fn get_current_weather(&self, lat: f64, lon: f64) -> Result<OpenWeatherCurrent> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/weather?lat={}&lon={}&appid={}&units=metric&lang=pt_br",
            Self::BASE_URL, lat, lon, self.api_key
        );

        let response: OpenWeatherCurrent = client.get(&url).send().await?.json().await?;
        Ok(response)
    }

    /// Fetch weather forecast for a location
    pub async fn get_forecast(&self, lat: f64, lon: f64) -> Result<OpenWeatherForecast> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/forecast?lat={}&lon={}&appid={}&units=metric&lang=pt_br",
            Self::BASE_URL, lat, lon, self.api_key
        );

        let response: OpenWeatherForecast = client.get(&url).send().await?.json().await?;
        Ok(response)
    }

    /// Search for a location by name
    pub async fn search_location(&self, query: &str) -> Result<Vec<Location>> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/direct?q={}&limit=5&appid={}",
            Self::GEO_URL, query, self.api_key
        );

        #[derive(Debug, Deserialize)]
        struct GeoResult {
            name: String,
            country: String,
            state: Option<String>,
            lat: f64,
            lon: f64,
        }

        let results: Vec<GeoResult> = client.get(&url).send().await?.json().await?;

        let locations = results
            .into_iter()
            .map(|r| {
                let country = match r.state {
                    Some(state) => format!("{}, {}", state, r.country),
                    None => r.country,
                };
                Location {
                    name: r.name,
                    country,
                    latitude: r.lat,
                    longitude: r.lon,
                }
            })
            .collect();

        Ok(locations)
    }

    /// Convert OpenWeatherMap condition code to our WeatherCondition
    fn convert_condition(id: i32) -> WeatherCondition {
        // OpenWeatherMap condition codes:
        // https://openweathermap.org/weather-conditions
        match id {
            200..=202 => WeatherCondition::ThunderstormWithHailSlight,
            210..=221 => WeatherCondition::Thunderstorm,
            230..=232 => WeatherCondition::ThunderstormWithHailHeavy,
            300..=302 => WeatherCondition::DrizzleLight,
            310..=314 => WeatherCondition::DrizzleModerate,
            321 => WeatherCondition::DrizzleDense,
            500..=501 => WeatherCondition::RainSlight,
            502..=504 => WeatherCondition::RainHeavy,
            511 => WeatherCondition::FreezingRainLight,
            520..=522 => WeatherCondition::RainShowersModerate,
            531 => WeatherCondition::RainShowersViolent,
            600..=601 => WeatherCondition::SnowFallSlight,
            602 => WeatherCondition::SnowFallHeavy,
            611..=613 => WeatherCondition::SnowGrains,
            615..=616 => WeatherCondition::SnowFallModerate,
            620..=622 => WeatherCondition::SnowShowersHeavy,
            701 => WeatherCondition::Fog,
            711 | 721 | 731 | 741 | 751 | 761 | 762 | 771 | 781 => WeatherCondition::Fog,
            800 => WeatherCondition::ClearSky,
            801 => WeatherCondition::MainlyClear,
            802 => WeatherCondition::PartlyCloudy,
            803..=804 => WeatherCondition::Overcast,
            _ => WeatherCondition::Unknown,
        }
    }
}

/// OpenWeatherMap current weather response
#[derive(Debug, Deserialize)]
pub struct OpenWeatherCurrent {
    pub coord: Coord,
    pub weather: Vec<Weather>,
    pub main: Main,
    pub visibility: Option<i32>,
    pub wind: Wind,
    pub clouds: Clouds,
    pub dt: i64,
    pub sys: Sys,
    pub timezone: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
}

#[derive(Debug, Deserialize)]
pub struct Weather {
    pub id: i32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
pub struct Main {
    pub temp: f64,
    pub feels_like: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub pressure: i32,
    pub humidity: i32,
    pub sea_level: Option<i32>,
    pub grnd_level: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Wind {
    pub speed: f64,
    pub deg: i32,
    pub gust: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Clouds {
    pub all: i32,
}

#[derive(Debug, Deserialize)]
pub struct Sys {
    pub country: Option<String>,
    pub sunrise: i64,
    pub sunset: i64,
}

/// OpenWeatherMap forecast response
#[derive(Debug, Deserialize)]
pub struct OpenWeatherForecast {
    pub cod: String,
    pub message: i32,
    pub cnt: i32,
    pub list: Vec<ForecastItem>,
    pub city: City,
}

#[derive(Debug, Deserialize)]
pub struct ForecastItem {
    pub dt: i64,
    pub main: Main,
    pub weather: Vec<Weather>,
    pub clouds: Clouds,
    pub wind: Wind,
    pub visibility: Option<i32>,
    pub pop: f64,
    pub sys: ForecastSys,
    pub dt_txt: String,
}

#[derive(Debug, Deserialize)]
pub struct ForecastSys {
    pub pod: String,
}

#[derive(Debug, Deserialize)]
pub struct City {
    pub id: i32,
    pub name: String,
    pub coord: Coord,
    pub country: String,
    pub population: i64,
    pub timezone: i32,
    pub sunrise: i64,
    pub sunset: i64,
}
