//! Open-Meteo API client
//! Free weather API, no key required
//! https://open-meteo.com/

use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;

use crate::api::Location;
use crate::data::{
    CurrentWeather, DailyForecast, HourlyForecast, WeatherCondition, WeatherData, WindDirection,
};

/// Open-Meteo API client
pub struct OpenMeteoClient;

impl OpenMeteoClient {
    const BASE_URL: &'static str = "https://api.open-meteo.com/v1/forecast";

    /// Fetch weather data for a location
    pub async fn get_weather(latitude: f64, longitude: f64) -> Result<WeatherData> {
        let client = reqwest::Client::new();

        let url = format!(
            "{}?latitude={}&longitude={}&\
            current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,\
            weather_code,cloud_cover,pressure_msl,wind_speed_10m,wind_direction_10m,wind_gusts_10m,\
            uv_index&\
            hourly=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation_probability,\
            precipitation,weather_code,wind_speed_10m,wind_direction_10m,is_day&\
            daily=weather_code,temperature_2m_max,temperature_2m_min,apparent_temperature_max,\
            apparent_temperature_min,sunrise,sunset,uv_index_max,precipitation_sum,\
            precipitation_probability_max,wind_speed_10m_max,wind_gusts_10m_max,wind_direction_10m_dominant&\
            timezone=auto&forecast_days=7",
            Self::BASE_URL, latitude, longitude
        );

        let response: OpenMeteoResponse = client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        Self::parse_response(response)
    }

    /// Search for a location by name
    pub async fn search_location(query: &str) -> Result<Vec<Location>> {
        crate::api::location::search_locations(query).await
    }

    /// Parse the API response into our data structures
    fn parse_response(response: OpenMeteoResponse) -> Result<WeatherData> {
        let timezone = response.timezone.clone();

        // Parse current weather
        let current = response.current.ok_or_else(|| anyhow::anyhow!("No current weather data"))?;

        let current_weather = CurrentWeather {
            temperature: current.temperature_2m,
            apparent_temperature: current.apparent_temperature,
            humidity: current.relative_humidity_2m,
            wind_speed: current.wind_speed_10m,
            wind_direction: WindDirection::new(current.wind_direction_10m),
            wind_gusts: current.wind_gusts_10m.unwrap_or(0.0),
            pressure: current.pressure_msl.unwrap_or(1013.25),
            precipitation: current.precipitation,
            cloud_cover: current.cloud_cover.unwrap_or(0),
            uv_index: current.uv_index.unwrap_or(0.0),
            visibility: 10000.0, // Not provided by Open-Meteo, use default
            condition: WeatherCondition::from_wmo_code(current.weather_code),
            is_day: current.is_day == 1,
            time: Utc::now(),
        };

        // Parse hourly forecast
        let hourly = response.hourly.ok_or_else(|| anyhow::anyhow!("No hourly data"))?;
        let hourly_forecasts: Vec<HourlyForecast> = hourly
            .time
            .iter()
            .enumerate()
            .filter_map(|(i, time_str)| {
                let time = parse_datetime(time_str)?;
                Some(HourlyForecast {
                    time,
                    temperature: *hourly.temperature_2m.get(i)?,
                    apparent_temperature: *hourly.apparent_temperature.get(i)?,
                    humidity: *hourly.relative_humidity_2m.get(i)?,
                    precipitation_probability: *hourly.precipitation_probability.get(i).unwrap_or(&0),
                    precipitation: *hourly.precipitation.get(i).unwrap_or(&0.0),
                    wind_speed: *hourly.wind_speed_10m.get(i)?,
                    wind_direction: WindDirection::new(*hourly.wind_direction_10m.get(i)?),
                    condition: WeatherCondition::from_wmo_code(*hourly.weather_code.get(i)?),
                    is_day: *hourly.is_day.get(i).unwrap_or(&1) == 1,
                })
            })
            .collect();

        // Parse daily forecast
        let daily = response.daily.ok_or_else(|| anyhow::anyhow!("No daily data"))?;
        let daily_forecasts: Vec<DailyForecast> = daily
            .time
            .iter()
            .enumerate()
            .filter_map(|(i, date_str)| {
                let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
                let sunrise = parse_datetime(daily.sunrise.get(i)?)?;
                let sunset = parse_datetime(daily.sunset.get(i)?)?;

                Some(DailyForecast {
                    date,
                    temperature_max: *daily.temperature_2m_max.get(i)?,
                    temperature_min: *daily.temperature_2m_min.get(i)?,
                    apparent_temperature_max: *daily.apparent_temperature_max.get(i)?,
                    apparent_temperature_min: *daily.apparent_temperature_min.get(i)?,
                    precipitation_sum: *daily.precipitation_sum.get(i).unwrap_or(&0.0),
                    precipitation_probability_max: *daily.precipitation_probability_max.get(i).unwrap_or(&0),
                    wind_speed_max: *daily.wind_speed_10m_max.get(i)?,
                    wind_gusts_max: *daily.wind_gusts_10m_max.get(i).unwrap_or(&0.0),
                    wind_direction: WindDirection::new(*daily.wind_direction_10m_dominant.get(i).unwrap_or(&0.0)),
                    uv_index_max: *daily.uv_index_max.get(i).unwrap_or(&0.0),
                    condition: WeatherCondition::from_wmo_code(*daily.weather_code.get(i)?),
                    sunrise,
                    sunset,
                })
            })
            .collect();

        // Get today's sunrise/sunset
        let (sunrise, sunset) = if let Some(first_daily) = daily_forecasts.first() {
            (first_daily.sunrise, first_daily.sunset)
        } else {
            let now = Utc::now();
            (now, now)
        };

        Ok(WeatherData {
            current: current_weather,
            hourly: hourly_forecasts,
            daily: daily_forecasts,
            timezone,
            sunrise,
            sunset,
        })
    }
}

/// Parse datetime string to UTC
fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Try parsing with time
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M") {
        return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
    }

    // Try parsing date only (midnight)
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = d.and_hms_opt(0, 0, 0)?;
        return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
    }

    None
}

// API Response structures

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    timezone: String,
    current: Option<CurrentData>,
    hourly: Option<HourlyData>,
    daily: Option<DailyData>,
}

#[derive(Debug, Deserialize)]
struct CurrentData {
    temperature_2m: f64,
    relative_humidity_2m: i32,
    apparent_temperature: f64,
    is_day: i32,
    precipitation: f64,
    weather_code: i32,
    cloud_cover: Option<i32>,
    pressure_msl: Option<f64>,
    wind_speed_10m: f64,
    wind_direction_10m: f64,
    wind_gusts_10m: Option<f64>,
    uv_index: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct HourlyData {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
    relative_humidity_2m: Vec<i32>,
    apparent_temperature: Vec<f64>,
    precipitation_probability: Option<Vec<i32>>,
    precipitation: Option<Vec<f64>>,
    weather_code: Vec<i32>,
    wind_speed_10m: Vec<f64>,
    wind_direction_10m: Vec<f64>,
    is_day: Option<Vec<i32>>,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    time: Vec<String>,
    weather_code: Vec<i32>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    apparent_temperature_max: Vec<f64>,
    apparent_temperature_min: Vec<f64>,
    sunrise: Vec<String>,
    sunset: Vec<String>,
    uv_index_max: Option<Vec<f64>>,
    precipitation_sum: Option<Vec<f64>>,
    precipitation_probability_max: Option<Vec<i32>>,
    wind_speed_10m_max: Vec<f64>,
    wind_gusts_10m_max: Option<Vec<f64>>,
    wind_direction_10m_dominant: Option<Vec<f64>>,
}
