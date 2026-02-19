//! Weather icons helper

use crate::data::WeatherCondition;

/// Weather icons helper
pub struct WeatherIcons;

impl WeatherIcons {
    /// Get icon name for condition
    pub fn get_icon(condition: WeatherCondition, is_day: bool) -> &'static str {
        condition.icon_name(is_day)
    }

    /// Get icon name for wind
    pub fn wind() -> &'static str {
        "weather-windy-symbolic"
    }

    /// Get icon name for humidity
    pub fn humidity() -> &'static str {
        "weather-fog-symbolic"
    }

    /// Get icon name for pressure
    pub fn pressure() -> &'static str {
        "speedometer-symbolic"
    }

    /// Get icon name for UV
    pub fn uv() -> &'static str {
        "weather-clear-symbolic"
    }

    /// Get icon name for sunrise
    pub fn sunrise() -> &'static str {
        "daytime-sunrise-symbolic"
    }

    /// Get icon name for sunset
    pub fn sunset() -> &'static str {
        "daytime-sunset-symbolic"
    }

    /// Get icon name for precipitation
    pub fn precipitation() -> &'static str {
        "weather-showers-symbolic"
    }

    /// Get icon name for visibility
    pub fn visibility() -> &'static str {
        "view-reveal-symbolic"
    }

    /// Get icon name for location
    pub fn location() -> &'static str {
        "mark-location-symbolic"
    }

    /// Get icon name for search
    pub fn search() -> &'static str {
        "system-search-symbolic"
    }

    /// Get icon name for refresh
    pub fn refresh() -> &'static str {
        "view-refresh-symbolic"
    }

    /// Get large icon name for animated weather (if available)
    pub fn get_animated_icon(condition: WeatherCondition, is_day: bool) -> &'static str {
        // For now, return the same static icon
        // In the future, this could return animated icon resources
        condition.icon_name(is_day)
    }
}
