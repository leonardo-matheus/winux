// Timezone data for World Clock

/// Common timezones with city name, region, and timezone string
pub const COMMON_TIMEZONES: &[(&str, &str, &str)] = &[
    // Americas
    ("Nova York", "Estados Unidos", "America/New_York"),
    ("Los Angeles", "Estados Unidos", "America/Los_Angeles"),
    ("Chicago", "Estados Unidos", "America/Chicago"),
    ("Toronto", "Canada", "America/Toronto"),
    ("Vancouver", "Canada", "America/Vancouver"),
    ("Cidade do Mexico", "Mexico", "America/Mexico_City"),
    ("Sao Paulo", "Brasil", "America/Sao_Paulo"),
    ("Buenos Aires", "Argentina", "America/Argentina/Buenos_Aires"),
    ("Santiago", "Chile", "America/Santiago"),
    ("Bogota", "Colombia", "America/Bogota"),
    ("Lima", "Peru", "America/Lima"),
    ("Caracas", "Venezuela", "America/Caracas"),

    // Europe
    ("Londres", "Reino Unido", "Europe/London"),
    ("Paris", "Franca", "Europe/Paris"),
    ("Berlim", "Alemanha", "Europe/Berlin"),
    ("Madrid", "Espanha", "Europe/Madrid"),
    ("Roma", "Italia", "Europe/Rome"),
    ("Amsterdam", "Holanda", "Europe/Amsterdam"),
    ("Zurique", "Suica", "Europe/Zurich"),
    ("Viena", "Austria", "Europe/Vienna"),
    ("Estocolmo", "Suecia", "Europe/Stockholm"),
    ("Oslo", "Noruega", "Europe/Oslo"),
    ("Copenhague", "Dinamarca", "Europe/Copenhagen"),
    ("Varsovia", "Polonia", "Europe/Warsaw"),
    ("Praga", "Republica Tcheca", "Europe/Prague"),
    ("Atenas", "Grecia", "Europe/Athens"),
    ("Moscou", "Russia", "Europe/Moscow"),
    ("Lisboa", "Portugal", "Europe/Lisbon"),
    ("Dublin", "Irlanda", "Europe/Dublin"),

    // Asia
    ("Toquio", "Japao", "Asia/Tokyo"),
    ("Pequim", "China", "Asia/Shanghai"),
    ("Hong Kong", "Hong Kong", "Asia/Hong_Kong"),
    ("Cingapura", "Cingapura", "Asia/Singapore"),
    ("Seul", "Coreia do Sul", "Asia/Seoul"),
    ("Mumbai", "India", "Asia/Kolkata"),
    ("Nova Delhi", "India", "Asia/Kolkata"),
    ("Bangkok", "Tailandia", "Asia/Bangkok"),
    ("Jakarta", "Indonesia", "Asia/Jakarta"),
    ("Manila", "Filipinas", "Asia/Manila"),
    ("Taipei", "Taiwan", "Asia/Taipei"),
    ("Kuala Lumpur", "Malasia", "Asia/Kuala_Lumpur"),
    ("Dubai", "Emirados Arabes", "Asia/Dubai"),
    ("Tel Aviv", "Israel", "Asia/Jerusalem"),
    ("Istambul", "Turquia", "Europe/Istanbul"),

    // Oceania
    ("Sydney", "Australia", "Australia/Sydney"),
    ("Melbourne", "Australia", "Australia/Melbourne"),
    ("Brisbane", "Australia", "Australia/Brisbane"),
    ("Perth", "Australia", "Australia/Perth"),
    ("Auckland", "Nova Zelandia", "Pacific/Auckland"),
    ("Wellington", "Nova Zelandia", "Pacific/Auckland"),

    // Africa
    ("Cairo", "Egito", "Africa/Cairo"),
    ("Joanesburgo", "Africa do Sul", "Africa/Johannesburg"),
    ("Lagos", "Nigeria", "Africa/Lagos"),
    ("Nairobi", "Quenia", "Africa/Nairobi"),
    ("Casablanca", "Marrocos", "Africa/Casablanca"),
];

/// Get timezone info by city name
pub fn get_timezone_by_city(city: &str) -> Option<(&'static str, &'static str, &'static str)> {
    COMMON_TIMEZONES
        .iter()
        .find(|(c, _, _)| c.eq_ignore_ascii_case(city))
        .copied()
}

/// Get all cities in a region
pub fn get_cities_by_region(region: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    COMMON_TIMEZONES
        .iter()
        .filter(|(_, r, _)| r.eq_ignore_ascii_case(region))
        .copied()
        .collect()
}

/// Get all unique regions
pub fn get_regions() -> Vec<&'static str> {
    let mut regions: Vec<&str> = COMMON_TIMEZONES
        .iter()
        .map(|(_, r, _)| *r)
        .collect();
    regions.sort();
    regions.dedup();
    regions
}

/// Search cities by partial name
pub fn search_cities(query: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    let query_lower = query.to_lowercase();
    COMMON_TIMEZONES
        .iter()
        .filter(|(city, region, _)| {
            city.to_lowercase().contains(&query_lower)
                || region.to_lowercase().contains(&query_lower)
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_timezone_by_city() {
        let result = get_timezone_by_city("Nova York");
        assert!(result.is_some());
        let (city, region, tz) = result.unwrap();
        assert_eq!(city, "Nova York");
        assert_eq!(region, "Estados Unidos");
        assert_eq!(tz, "America/New_York");
    }

    #[test]
    fn test_search_cities() {
        let results = search_cities("york");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(c, _, _)| *c == "Nova York"));
    }

    #[test]
    fn test_get_regions() {
        let regions = get_regions();
        assert!(regions.contains(&"Brasil"));
        assert!(regions.contains(&"Japao"));
    }
}
