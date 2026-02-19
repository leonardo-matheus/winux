//! Location and geolocation services

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Location data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// Response from IP geolocation service
#[derive(Debug, Deserialize)]
struct IpGeoResponse {
    city: Option<String>,
    country: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
}

/// Get current location using IP geolocation
pub async fn get_current_location() -> Result<Location> {
    let client = reqwest::Client::new();

    // Use ip-api.com (free, no key required)
    let response: IpGeoResponse = client
        .get("http://ip-api.com/json/")
        .send()
        .await?
        .json()
        .await?;

    Ok(Location {
        name: response.city.unwrap_or_else(|| "Unknown".to_string()),
        country: response.country.unwrap_or_else(|| "Unknown".to_string()),
        latitude: response.lat.unwrap_or(0.0),
        longitude: response.lon.unwrap_or(0.0),
    })
}

/// Search for a location by name using Open-Meteo Geocoding API
pub async fn search_locations(query: &str) -> Result<Vec<Location>> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=5&language=pt&format=json",
        urlencoding::encode(query)
    );

    #[derive(Debug, Deserialize)]
    struct GeocodingResponse {
        results: Option<Vec<GeocodingResult>>,
    }

    #[derive(Debug, Deserialize)]
    struct GeocodingResult {
        name: String,
        country: Option<String>,
        latitude: f64,
        longitude: f64,
        admin1: Option<String>,
    }

    let response: GeocodingResponse = client
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    let locations = response.results.unwrap_or_default()
        .into_iter()
        .map(|r| {
            let country = match (&r.country, &r.admin1) {
                (Some(c), Some(a)) => format!("{}, {}", a, c),
                (Some(c), None) => c.clone(),
                (None, Some(a)) => a.clone(),
                (None, None) => "Unknown".to_string(),
            };

            Location {
                name: r.name,
                country,
                latitude: r.latitude,
                longitude: r.longitude,
            }
        })
        .collect();

    Ok(locations)
}

// URL encoding helper
mod urlencoding {
    pub fn encode(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "%20".to_string(),
                _ => format!("%{:02X}", c as u32),
            })
            .collect()
    }
}
