//! Saved locations management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::api::Location;

/// Saved locations data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLocations {
    pub locations: Vec<Location>,
    pub default_index: Option<usize>,
}

impl Default for SavedLocations {
    fn default() -> Self {
        Self {
            locations: vec![
                Location {
                    name: "Sao Paulo".to_string(),
                    country: "Brasil".to_string(),
                    latitude: -23.5505,
                    longitude: -46.6333,
                },
            ],
            default_index: Some(0),
        }
    }
}

impl SavedLocations {
    /// Get config file path
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-weather");

        std::fs::create_dir_all(&config_dir).ok();
        config_dir.join("locations.json")
    }

    /// Load saved locations from file
    pub fn load() -> Self {
        let path = Self::config_path();

        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(locations) = serde_json::from_str(&content) {
                    return locations;
                }
            }
        }

        Self::default()
    }

    /// Save locations to file
    pub fn save(&self) {
        let path = Self::config_path();

        if let Ok(content) = serde_json::to_string_pretty(self) {
            std::fs::write(&path, content).ok();
        }
    }

    /// Add a new location
    pub fn add(&mut self, location: Location) {
        // Check if location already exists
        let exists = self.locations.iter().any(|l| {
            (l.latitude - location.latitude).abs() < 0.01 &&
            (l.longitude - location.longitude).abs() < 0.01
        });

        if !exists {
            self.locations.push(location);
        }
    }

    /// Remove a location by index
    pub fn remove(&mut self, index: usize) {
        if index < self.locations.len() {
            self.locations.remove(index);

            // Update default index if needed
            if let Some(default_idx) = self.default_index {
                if default_idx == index {
                    self.default_index = if self.locations.is_empty() {
                        None
                    } else {
                        Some(0)
                    };
                } else if default_idx > index {
                    self.default_index = Some(default_idx - 1);
                }
            }
        }
    }

    /// Set default location
    pub fn set_default(&mut self, index: usize) {
        if index < self.locations.len() {
            self.default_index = Some(index);
        }
    }

    /// Get default location
    pub fn get_default(&self) -> Option<&Location> {
        self.default_index.and_then(|i| self.locations.get(i))
    }

    /// Get all locations
    pub fn all(&self) -> &[Location] {
        &self.locations
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    /// Get location count
    pub fn len(&self) -> usize {
        self.locations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_location() {
        let mut locations = SavedLocations::default();
        let initial_count = locations.len();

        locations.add(Location {
            name: "Rio de Janeiro".to_string(),
            country: "Brasil".to_string(),
            latitude: -22.9068,
            longitude: -43.1729,
        });

        assert_eq!(locations.len(), initial_count + 1);
    }

    #[test]
    fn test_duplicate_location() {
        let mut locations = SavedLocations::default();
        let initial_count = locations.len();

        // Add same location twice
        let loc = Location {
            name: "Rio de Janeiro".to_string(),
            country: "Brasil".to_string(),
            latitude: -22.9068,
            longitude: -43.1729,
        };

        locations.add(loc.clone());
        locations.add(loc);

        assert_eq!(locations.len(), initial_count + 1);
    }
}
