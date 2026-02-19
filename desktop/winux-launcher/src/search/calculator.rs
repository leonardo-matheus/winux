//! Calculator and unit conversion provider

use crate::search::{SearchCategory, SearchResult, SearchResultKind};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use tracing::debug;

/// Math expression patterns
static MATH_EXPR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[\d\s\+\-\*\/\(\)\.\^%]+$").unwrap()
});

/// Conversion pattern: "100 usd to brl", "10km to miles"
static CONVERSION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^([\d.]+)\s*([a-zA-Z]+)\s+(?:to|in|as)\s+([a-zA-Z]+)$").unwrap()
});

/// Simple conversion pattern: "100 usd brl"
static SIMPLE_CONVERSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^([\d.]+)\s*([a-zA-Z]+)\s+([a-zA-Z]+)$").unwrap()
});

/// Calculator and converter
pub struct Calculator {
    /// Unit conversion factors to base units
    conversions: HashMap<&'static str, (&'static str, f64, &'static str)>,
}

impl Calculator {
    /// Create new calculator
    pub fn new() -> Self {
        let mut conversions = HashMap::new();

        // Length conversions (base: meters)
        conversions.insert("m", ("length", 1.0, "meters"));
        conversions.insert("meters", ("length", 1.0, "meters"));
        conversions.insert("km", ("length", 1000.0, "kilometers"));
        conversions.insert("kilometers", ("length", 1000.0, "kilometers"));
        conversions.insert("cm", ("length", 0.01, "centimeters"));
        conversions.insert("centimeters", ("length", 0.01, "centimeters"));
        conversions.insert("mm", ("length", 0.001, "millimeters"));
        conversions.insert("millimeters", ("length", 0.001, "millimeters"));
        conversions.insert("mi", ("length", 1609.344, "miles"));
        conversions.insert("miles", ("length", 1609.344, "miles"));
        conversions.insert("yd", ("length", 0.9144, "yards"));
        conversions.insert("yards", ("length", 0.9144, "yards"));
        conversions.insert("ft", ("length", 0.3048, "feet"));
        conversions.insert("feet", ("length", 0.3048, "feet"));
        conversions.insert("in", ("length", 0.0254, "inches"));
        conversions.insert("inches", ("length", 0.0254, "inches"));

        // Weight conversions (base: kilograms)
        conversions.insert("kg", ("weight", 1.0, "kilograms"));
        conversions.insert("kilograms", ("weight", 1.0, "kilograms"));
        conversions.insert("g", ("weight", 0.001, "grams"));
        conversions.insert("grams", ("weight", 0.001, "grams"));
        conversions.insert("mg", ("weight", 0.000001, "milligrams"));
        conversions.insert("milligrams", ("weight", 0.000001, "milligrams"));
        conversions.insert("lb", ("weight", 0.453592, "pounds"));
        conversions.insert("lbs", ("weight", 0.453592, "pounds"));
        conversions.insert("pounds", ("weight", 0.453592, "pounds"));
        conversions.insert("oz", ("weight", 0.0283495, "ounces"));
        conversions.insert("ounces", ("weight", 0.0283495, "ounces"));
        conversions.insert("ton", ("weight", 1000.0, "tons"));
        conversions.insert("tons", ("weight", 1000.0, "tons"));

        // Temperature conversions (special handling)
        conversions.insert("c", ("temperature", 0.0, "Celsius"));
        conversions.insert("celsius", ("temperature", 0.0, "Celsius"));
        conversions.insert("f", ("temperature", 0.0, "Fahrenheit"));
        conversions.insert("fahrenheit", ("temperature", 0.0, "Fahrenheit"));
        conversions.insert("k", ("temperature", 0.0, "Kelvin"));
        conversions.insert("kelvin", ("temperature", 0.0, "Kelvin"));

        // Volume conversions (base: liters)
        conversions.insert("l", ("volume", 1.0, "liters"));
        conversions.insert("liters", ("volume", 1.0, "liters"));
        conversions.insert("ml", ("volume", 0.001, "milliliters"));
        conversions.insert("milliliters", ("volume", 0.001, "milliliters"));
        conversions.insert("gal", ("volume", 3.78541, "gallons"));
        conversions.insert("gallons", ("volume", 3.78541, "gallons"));
        conversions.insert("qt", ("volume", 0.946353, "quarts"));
        conversions.insert("quarts", ("volume", 0.946353, "quarts"));
        conversions.insert("pt", ("volume", 0.473176, "pints"));
        conversions.insert("pints", ("volume", 0.473176, "pints"));
        conversions.insert("cup", ("volume", 0.236588, "cups"));
        conversions.insert("cups", ("volume", 0.236588, "cups"));

        // Time conversions (base: seconds)
        conversions.insert("s", ("time", 1.0, "seconds"));
        conversions.insert("sec", ("time", 1.0, "seconds"));
        conversions.insert("seconds", ("time", 1.0, "seconds"));
        conversions.insert("min", ("time", 60.0, "minutes"));
        conversions.insert("minutes", ("time", 60.0, "minutes"));
        conversions.insert("h", ("time", 3600.0, "hours"));
        conversions.insert("hr", ("time", 3600.0, "hours"));
        conversions.insert("hours", ("time", 3600.0, "hours"));
        conversions.insert("d", ("time", 86400.0, "days"));
        conversions.insert("days", ("time", 86400.0, "days"));
        conversions.insert("wk", ("time", 604800.0, "weeks"));
        conversions.insert("weeks", ("time", 604800.0, "weeks"));

        // Data size conversions (base: bytes)
        conversions.insert("b", ("data", 1.0, "bytes"));
        conversions.insert("bytes", ("data", 1.0, "bytes"));
        conversions.insert("kb", ("data", 1024.0, "kilobytes"));
        conversions.insert("kilobytes", ("data", 1024.0, "kilobytes"));
        conversions.insert("mb", ("data", 1048576.0, "megabytes"));
        conversions.insert("megabytes", ("data", 1048576.0, "megabytes"));
        conversions.insert("gb", ("data", 1073741824.0, "gigabytes"));
        conversions.insert("gigabytes", ("data", 1073741824.0, "gigabytes"));
        conversions.insert("tb", ("data", 1099511627776.0, "terabytes"));
        conversions.insert("terabytes", ("data", 1099511627776.0, "terabytes"));

        // Speed conversions (base: m/s)
        conversions.insert("mps", ("speed", 1.0, "m/s"));
        conversions.insert("kph", ("speed", 0.277778, "km/h"));
        conversions.insert("kmh", ("speed", 0.277778, "km/h"));
        conversions.insert("mph", ("speed", 0.44704, "mph"));

        Self { conversions }
    }

    /// Evaluate a mathematical expression
    pub fn evaluate(&self, query: &str) -> Option<SearchResult> {
        let query = query.trim();

        // Check if it looks like a math expression
        if !MATH_EXPR.is_match(query) {
            return None;
        }

        // Try to evaluate the expression
        match meval::eval_str(query) {
            Ok(result) => {
                let result_str = self.format_number(result);

                Some(SearchResult {
                    id: format!("calc:{}", query),
                    title: result_str.clone(),
                    subtitle: format!("{} =", query),
                    icon: "accessories-calculator-symbolic".to_string(),
                    category: SearchCategory::Calculator,
                    kind: SearchResultKind::Calculator {
                        expression: query.to_string(),
                        result: result_str,
                    },
                    score: 100,
                    from_history: false,
                })
            }
            Err(e) => {
                debug!("Math evaluation failed: {}", e);
                None
            }
        }
    }

    /// Perform unit conversion
    pub fn convert(&self, query: &str) -> Option<SearchResult> {
        let query = query.trim().to_lowercase();

        // Try to match conversion patterns
        let captures = CONVERSION_PATTERN
            .captures(&query)
            .or_else(|| SIMPLE_CONVERSION.captures(&query))?;

        let value: f64 = captures.get(1)?.as_str().parse().ok()?;
        let from_unit = captures.get(2)?.as_str();
        let to_unit = captures.get(3)?.as_str();

        // Get conversion info
        let from_info = self.conversions.get(from_unit)?;
        let to_info = self.conversions.get(to_unit)?;

        // Check if same category
        if from_info.0 != to_info.0 {
            return None;
        }

        // Handle temperature specially
        if from_info.0 == "temperature" {
            return self.convert_temperature(value, from_unit, to_unit);
        }

        // Convert: value * from_factor / to_factor
        let result = value * from_info.1 / to_info.1;
        let result_str = self.format_number(result);

        let title = format!("{} {}", result_str, to_info.2);
        let subtitle = format!("{} {} =", self.format_number(value), from_info.2);

        Some(SearchResult {
            id: format!("conv:{}:{}:{}", value, from_unit, to_unit),
            title,
            subtitle,
            icon: "view-refresh-symbolic".to_string(),
            category: SearchCategory::Conversion,
            kind: SearchResultKind::Conversion {
                from_value: self.format_number(value),
                from_unit: from_info.2.to_string(),
                to_value: result_str.clone(),
                to_unit: to_info.2.to_string(),
                result: result_str,
            },
            score: 95,
            from_history: false,
        })
    }

    /// Convert temperature units
    fn convert_temperature(&self, value: f64, from: &str, to: &str) -> Option<SearchResult> {
        let from_unit = self.normalize_temp_unit(from);
        let to_unit = self.normalize_temp_unit(to);

        // Convert to Celsius first
        let celsius = match from_unit {
            "c" => value,
            "f" => (value - 32.0) * 5.0 / 9.0,
            "k" => value - 273.15,
            _ => return None,
        };

        // Convert from Celsius to target
        let result = match to_unit {
            "c" => celsius,
            "f" => celsius * 9.0 / 5.0 + 32.0,
            "k" => celsius + 273.15,
            _ => return None,
        };

        let from_name = self.temp_unit_name(from_unit);
        let to_name = self.temp_unit_name(to_unit);

        let result_str = self.format_number(result);
        let title = format!("{} {}", result_str, to_name);
        let subtitle = format!("{} {} =", self.format_number(value), from_name);

        Some(SearchResult {
            id: format!("conv:{}:{}:{}", value, from, to),
            title,
            subtitle,
            icon: "weather-clear-symbolic".to_string(),
            category: SearchCategory::Conversion,
            kind: SearchResultKind::Conversion {
                from_value: self.format_number(value),
                from_unit: from_name.to_string(),
                to_value: result_str.clone(),
                to_unit: to_name.to_string(),
                result: result_str,
            },
            score: 95,
            from_history: false,
        })
    }

    /// Normalize temperature unit name
    fn normalize_temp_unit(&self, unit: &str) -> &str {
        match unit.to_lowercase().as_str() {
            "c" | "celsius" => "c",
            "f" | "fahrenheit" => "f",
            "k" | "kelvin" => "k",
            _ => unit,
        }
    }

    /// Get temperature unit display name
    fn temp_unit_name(&self, unit: &str) -> &str {
        match unit {
            "c" => "Celsius",
            "f" => "Fahrenheit",
            "k" => "Kelvin",
            _ => unit,
        }
    }

    /// Format a number for display
    fn format_number(&self, num: f64) -> String {
        if num.abs() >= 1_000_000.0 || (num.abs() < 0.0001 && num != 0.0) {
            // Use scientific notation for very large or small numbers
            format!("{:.6e}", num)
        } else if num.fract() == 0.0 {
            // Integer
            format!("{}", num as i64)
        } else {
            // Float with reasonable precision
            let formatted = format!("{:.6}", num);
            formatted.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
