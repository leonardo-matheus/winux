//! Weather data structures

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone, Local};

/// Weather condition codes from WMO
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,
    Fog,
    DepositingRimeFog,
    DrizzleLight,
    DrizzleModerate,
    DrizzleDense,
    FreezingDrizzleLight,
    FreezingDrizzleDense,
    RainSlight,
    RainModerate,
    RainHeavy,
    FreezingRainLight,
    FreezingRainHeavy,
    SnowFallSlight,
    SnowFallModerate,
    SnowFallHeavy,
    SnowGrains,
    RainShowersSlight,
    RainShowersModerate,
    RainShowersViolent,
    SnowShowersSlight,
    SnowShowersHeavy,
    Thunderstorm,
    ThunderstormWithHailSlight,
    ThunderstormWithHailHeavy,
    Unknown,
}

impl WeatherCondition {
    /// Convert WMO weather code to WeatherCondition
    pub fn from_wmo_code(code: i32) -> Self {
        match code {
            0 => Self::ClearSky,
            1 => Self::MainlyClear,
            2 => Self::PartlyCloudy,
            3 => Self::Overcast,
            45 => Self::Fog,
            48 => Self::DepositingRimeFog,
            51 => Self::DrizzleLight,
            53 => Self::DrizzleModerate,
            55 => Self::DrizzleDense,
            56 => Self::FreezingDrizzleLight,
            57 => Self::FreezingDrizzleDense,
            61 => Self::RainSlight,
            63 => Self::RainModerate,
            65 => Self::RainHeavy,
            66 => Self::FreezingRainLight,
            67 => Self::FreezingRainHeavy,
            71 => Self::SnowFallSlight,
            73 => Self::SnowFallModerate,
            75 => Self::SnowFallHeavy,
            77 => Self::SnowGrains,
            80 => Self::RainShowersSlight,
            81 => Self::RainShowersModerate,
            82 => Self::RainShowersViolent,
            85 => Self::SnowShowersSlight,
            86 => Self::SnowShowersHeavy,
            95 => Self::Thunderstorm,
            96 => Self::ThunderstormWithHailSlight,
            99 => Self::ThunderstormWithHailHeavy,
            _ => Self::Unknown,
        }
    }

    /// Get icon name for this weather condition
    pub fn icon_name(&self, is_day: bool) -> &'static str {
        match self {
            Self::ClearSky | Self::MainlyClear => {
                if is_day { "weather-clear-symbolic" }
                else { "weather-clear-night-symbolic" }
            }
            Self::PartlyCloudy => {
                if is_day { "weather-few-clouds-symbolic" }
                else { "weather-few-clouds-night-symbolic" }
            }
            Self::Overcast => "weather-overcast-symbolic",
            Self::Fog | Self::DepositingRimeFog => "weather-fog-symbolic",
            Self::DrizzleLight | Self::DrizzleModerate | Self::DrizzleDense |
            Self::RainSlight | Self::RainModerate | Self::FreezingDrizzleLight |
            Self::FreezingDrizzleDense | Self::FreezingRainLight => "weather-showers-symbolic",
            Self::RainHeavy | Self::FreezingRainHeavy |
            Self::RainShowersSlight | Self::RainShowersModerate |
            Self::RainShowersViolent => "weather-showers-scattered-symbolic",
            Self::SnowFallSlight | Self::SnowFallModerate | Self::SnowFallHeavy |
            Self::SnowGrains | Self::SnowShowersSlight | Self::SnowShowersHeavy => "weather-snow-symbolic",
            Self::Thunderstorm | Self::ThunderstormWithHailSlight |
            Self::ThunderstormWithHailHeavy => "weather-storm-symbolic",
            Self::Unknown => "weather-severe-alert-symbolic",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::ClearSky => "Ceu limpo",
            Self::MainlyClear => "Predominantemente limpo",
            Self::PartlyCloudy => "Parcialmente nublado",
            Self::Overcast => "Nublado",
            Self::Fog => "Neblina",
            Self::DepositingRimeFog => "Neblina com geada",
            Self::DrizzleLight => "Garoa leve",
            Self::DrizzleModerate => "Garoa moderada",
            Self::DrizzleDense => "Garoa intensa",
            Self::FreezingDrizzleLight => "Garoa congelante leve",
            Self::FreezingDrizzleDense => "Garoa congelante intensa",
            Self::RainSlight => "Chuva leve",
            Self::RainModerate => "Chuva moderada",
            Self::RainHeavy => "Chuva forte",
            Self::FreezingRainLight => "Chuva congelante leve",
            Self::FreezingRainHeavy => "Chuva congelante forte",
            Self::SnowFallSlight => "Neve leve",
            Self::SnowFallModerate => "Neve moderada",
            Self::SnowFallHeavy => "Neve intensa",
            Self::SnowGrains => "Graos de neve",
            Self::RainShowersSlight => "Pancadas de chuva leves",
            Self::RainShowersModerate => "Pancadas de chuva moderadas",
            Self::RainShowersViolent => "Pancadas de chuva intensas",
            Self::SnowShowersSlight => "Pancadas de neve leves",
            Self::SnowShowersHeavy => "Pancadas de neve intensas",
            Self::Thunderstorm => "Tempestade",
            Self::ThunderstormWithHailSlight => "Tempestade com granizo leve",
            Self::ThunderstormWithHailHeavy => "Tempestade com granizo forte",
            Self::Unknown => "Desconhecido",
        }
    }

    /// Check if this is a rainy condition
    pub fn is_rainy(&self) -> bool {
        matches!(
            self,
            Self::DrizzleLight | Self::DrizzleModerate | Self::DrizzleDense |
            Self::RainSlight | Self::RainModerate | Self::RainHeavy |
            Self::RainShowersSlight | Self::RainShowersModerate | Self::RainShowersViolent |
            Self::FreezingDrizzleLight | Self::FreezingDrizzleDense |
            Self::FreezingRainLight | Self::FreezingRainHeavy
        )
    }

    /// Check if this is a snowy condition
    pub fn is_snowy(&self) -> bool {
        matches!(
            self,
            Self::SnowFallSlight | Self::SnowFallModerate | Self::SnowFallHeavy |
            Self::SnowGrains | Self::SnowShowersSlight | Self::SnowShowersHeavy
        )
    }

    /// Check if this is a stormy condition
    pub fn is_stormy(&self) -> bool {
        matches!(
            self,
            Self::Thunderstorm | Self::ThunderstormWithHailSlight | Self::ThunderstormWithHailHeavy
        )
    }

    /// Check if this is a cloudy condition
    pub fn is_cloudy(&self) -> bool {
        matches!(
            self,
            Self::PartlyCloudy | Self::Overcast | Self::Fog | Self::DepositingRimeFog
        )
    }
}

/// Wind direction from degrees
#[derive(Debug, Clone, Copy)]
pub struct WindDirection {
    pub degrees: f64,
}

impl WindDirection {
    pub fn new(degrees: f64) -> Self {
        Self { degrees }
    }

    pub fn cardinal(&self) -> &'static str {
        let d = self.degrees;
        if d >= 337.5 || d < 22.5 {
            "N"
        } else if d >= 22.5 && d < 67.5 {
            "NE"
        } else if d >= 67.5 && d < 112.5 {
            "L"
        } else if d >= 112.5 && d < 157.5 {
            "SE"
        } else if d >= 157.5 && d < 202.5 {
            "S"
        } else if d >= 202.5 && d < 247.5 {
            "SO"
        } else if d >= 247.5 && d < 292.5 {
            "O"
        } else {
            "NO"
        }
    }

    pub fn full_name(&self) -> &'static str {
        let d = self.degrees;
        if d >= 337.5 || d < 22.5 {
            "Norte"
        } else if d >= 22.5 && d < 67.5 {
            "Nordeste"
        } else if d >= 67.5 && d < 112.5 {
            "Leste"
        } else if d >= 112.5 && d < 157.5 {
            "Sudeste"
        } else if d >= 157.5 && d < 202.5 {
            "Sul"
        } else if d >= 202.5 && d < 247.5 {
            "Sudoeste"
        } else if d >= 247.5 && d < 292.5 {
            "Oeste"
        } else {
            "Noroeste"
        }
    }
}

/// Current weather data
#[derive(Debug, Clone)]
pub struct CurrentWeather {
    pub temperature: f64,
    pub apparent_temperature: f64,
    pub humidity: i32,
    pub wind_speed: f64,
    pub wind_direction: WindDirection,
    pub wind_gusts: f64,
    pub pressure: f64,
    pub precipitation: f64,
    pub cloud_cover: i32,
    pub uv_index: f64,
    pub visibility: f64,
    pub condition: WeatherCondition,
    pub is_day: bool,
    pub time: DateTime<Utc>,
}

/// Hourly forecast data point
#[derive(Debug, Clone)]
pub struct HourlyForecast {
    pub time: DateTime<Utc>,
    pub temperature: f64,
    pub apparent_temperature: f64,
    pub humidity: i32,
    pub precipitation_probability: i32,
    pub precipitation: f64,
    pub wind_speed: f64,
    pub wind_direction: WindDirection,
    pub condition: WeatherCondition,
    pub is_day: bool,
}

/// Daily forecast data point
#[derive(Debug, Clone)]
pub struct DailyForecast {
    pub date: chrono::NaiveDate,
    pub temperature_max: f64,
    pub temperature_min: f64,
    pub apparent_temperature_max: f64,
    pub apparent_temperature_min: f64,
    pub precipitation_sum: f64,
    pub precipitation_probability_max: i32,
    pub wind_speed_max: f64,
    pub wind_gusts_max: f64,
    pub wind_direction: WindDirection,
    pub uv_index_max: f64,
    pub condition: WeatherCondition,
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,
}

impl DailyForecast {
    /// Get day name in Portuguese
    pub fn day_name(&self) -> String {
        let today = chrono::Local::now().date_naive();
        let tomorrow = today + chrono::Duration::days(1);

        if self.date == today {
            "Hoje".to_string()
        } else if self.date == tomorrow {
            "Amanha".to_string()
        } else {
            match self.date.weekday() {
                chrono::Weekday::Mon => "Segunda",
                chrono::Weekday::Tue => "Terca",
                chrono::Weekday::Wed => "Quarta",
                chrono::Weekday::Thu => "Quinta",
                chrono::Weekday::Fri => "Sexta",
                chrono::Weekday::Sat => "Sabado",
                chrono::Weekday::Sun => "Domingo",
            }.to_string()
        }
    }

    /// Get formatted date
    pub fn formatted_date(&self) -> String {
        format!("{}/{}", self.date.day(), self.date.month())
    }
}

/// Complete weather data for a location
#[derive(Debug, Clone)]
pub struct WeatherData {
    pub current: CurrentWeather,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
    pub timezone: String,
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,
}

impl WeatherData {
    /// Check if it's currently daytime
    pub fn is_daytime(&self) -> bool {
        let now = Utc::now();
        now >= self.sunrise && now < self.sunset
    }

    /// Get next 24 hours of forecast
    pub fn next_24_hours(&self) -> Vec<&HourlyForecast> {
        let now = Utc::now();
        self.hourly
            .iter()
            .filter(|h| h.time >= now)
            .take(24)
            .collect()
    }

    /// Get next 7 days of forecast
    pub fn next_7_days(&self) -> Vec<&DailyForecast> {
        self.daily.iter().take(7).collect()
    }
}

/// UV Index levels
#[derive(Debug, Clone, Copy)]
pub enum UvLevel {
    Low,
    Moderate,
    High,
    VeryHigh,
    Extreme,
}

impl UvLevel {
    pub fn from_index(index: f64) -> Self {
        if index < 3.0 {
            Self::Low
        } else if index < 6.0 {
            Self::Moderate
        } else if index < 8.0 {
            Self::High
        } else if index < 11.0 {
            Self::VeryHigh
        } else {
            Self::Extreme
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Low => "Baixo",
            Self::Moderate => "Moderado",
            Self::High => "Alto",
            Self::VeryHigh => "Muito Alto",
            Self::Extreme => "Extremo",
        }
    }

    pub fn recommendation(&self) -> &'static str {
        match self {
            Self::Low => "Nenhuma protecao necessaria",
            Self::Moderate => "Use protetor solar",
            Self::High => "Protecao necessaria, evite sol do meio-dia",
            Self::VeryHigh => "Protecao extra necessaria",
            Self::Extreme => "Evite exposicao ao sol",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Low => "#4CAF50",
            Self::Moderate => "#FFEB3B",
            Self::High => "#FF9800",
            Self::VeryHigh => "#F44336",
            Self::Extreme => "#9C27B0",
        }
    }
}
