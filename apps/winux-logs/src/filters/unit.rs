// Winux Logs - Unit Filter
// Copyright (c) 2026 Winux OS Project

use crate::sources::LogEntry;
use std::collections::HashMap;

/// Filter entries by systemd unit
pub fn filter_by_unit(entries: &[LogEntry], units: &[String]) -> Vec<&LogEntry> {
    if units.is_empty() {
        // No filter, return all
        entries.iter().collect()
    } else {
        entries.iter()
            .filter(|e| {
                e.unit.as_ref()
                    .map(|u| units.iter().any(|f| u.contains(f)))
                    .unwrap_or(false)
            })
            .collect()
    }
}

/// Filter entries by exact unit match
pub fn filter_by_exact_unit(entries: &[LogEntry], unit: &str) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|e| e.unit.as_ref().map(|u| u == unit).unwrap_or(false))
        .collect()
}

/// Get all unique units from entries
pub fn get_unique_units(entries: &[LogEntry]) -> Vec<String> {
    let mut units: Vec<String> = entries.iter()
        .filter_map(|e| e.unit.clone())
        .collect();

    units.sort();
    units.dedup();
    units
}

/// Count entries per unit
pub fn count_by_unit(entries: &[LogEntry]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for entry in entries {
        if let Some(ref unit) = entry.unit {
            *counts.entry(unit.clone()).or_insert(0) += 1;
        }
    }

    counts
}

/// Get top N units by entry count
pub fn get_top_units(entries: &[LogEntry], n: usize) -> Vec<(String, usize)> {
    let counts = count_by_unit(entries);
    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(n);
    sorted
}

/// Common systemd unit categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitCategory {
    System,     // systemd-*, init
    Network,    // NetworkManager, dhcpcd, etc.
    Security,   // sshd, firewalld, etc.
    Desktop,    // gdm, gnome-*, etc.
    Hardware,   // udev, bluetooth, etc.
    Storage,    // mount, lvm, etc.
    User,       // User sessions
    Application, // Regular applications
    Unknown,
}

impl UnitCategory {
    pub fn from_unit_name(unit: &str) -> Self {
        let lower = unit.to_lowercase();

        if lower.starts_with("systemd") || lower.contains("init") {
            UnitCategory::System
        } else if lower.contains("network") || lower.contains("dhcp") ||
                  lower.contains("wpa") || lower.contains("wifi") ||
                  lower.contains("nm-") {
            UnitCategory::Network
        } else if lower.contains("ssh") || lower.contains("firewall") ||
                  lower.contains("audit") || lower.contains("polkit") ||
                  lower.contains("auth") {
            UnitCategory::Security
        } else if lower.contains("gdm") || lower.contains("gnome") ||
                  lower.contains("kde") || lower.contains("xorg") ||
                  lower.contains("wayland") || lower.contains("display") {
            UnitCategory::Desktop
        } else if lower.contains("udev") || lower.contains("bluetooth") ||
                  lower.contains("acpi") || lower.contains("power") ||
                  lower.contains("battery") {
            UnitCategory::Hardware
        } else if lower.contains("mount") || lower.contains("lvm") ||
                  lower.contains("cryptsetup") || lower.contains("disk") {
            UnitCategory::Storage
        } else if lower.starts_with("user@") || lower.starts_with("session-") {
            UnitCategory::User
        } else {
            UnitCategory::Application
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            UnitCategory::System => "Sistema",
            UnitCategory::Network => "Rede",
            UnitCategory::Security => "Seguranca",
            UnitCategory::Desktop => "Desktop",
            UnitCategory::Hardware => "Hardware",
            UnitCategory::Storage => "Armazenamento",
            UnitCategory::User => "Usuario",
            UnitCategory::Application => "Aplicativo",
            UnitCategory::Unknown => "Outro",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            UnitCategory::System => "system-run-symbolic",
            UnitCategory::Network => "network-wireless-symbolic",
            UnitCategory::Security => "security-high-symbolic",
            UnitCategory::Desktop => "video-display-symbolic",
            UnitCategory::Hardware => "computer-symbolic",
            UnitCategory::Storage => "drive-harddisk-symbolic",
            UnitCategory::User => "system-users-symbolic",
            UnitCategory::Application => "application-x-executable-symbolic",
            UnitCategory::Unknown => "help-about-symbolic",
        }
    }
}

/// Group units by category
pub fn group_by_category(units: &[String]) -> HashMap<UnitCategory, Vec<String>> {
    let mut groups: HashMap<UnitCategory, Vec<String>> = HashMap::new();

    for unit in units {
        let category = UnitCategory::from_unit_name(unit);
        groups.entry(category).or_default().push(unit.clone());
    }

    // Sort units within each category
    for units in groups.values_mut() {
        units.sort();
    }

    groups
}

/// Parse unit name to extract service name without suffix
pub fn parse_unit_name(unit: &str) -> (&str, Option<&str>) {
    // systemd unit format: name.type (e.g., sshd.service)
    if let Some(dot_pos) = unit.rfind('.') {
        let name = &unit[..dot_pos];
        let suffix = &unit[dot_pos + 1..];
        (name, Some(suffix))
    } else {
        (unit, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::LogLevel;
    use chrono::Local;

    fn make_entry(unit: Option<&str>) -> LogEntry {
        let mut entry = LogEntry::new(Local::now(), LogLevel::Info, "test".to_string());
        entry.unit = unit.map(|s| s.to_string());
        entry
    }

    #[test]
    fn test_filter_by_unit() {
        let entries = vec![
            make_entry(Some("sshd.service")),
            make_entry(Some("NetworkManager.service")),
            make_entry(Some("gdm.service")),
        ];

        let filtered = filter_by_unit(&entries, &["ssh".to_string()]);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_get_unique_units() {
        let entries = vec![
            make_entry(Some("a.service")),
            make_entry(Some("b.service")),
            make_entry(Some("a.service")),
        ];

        let units = get_unique_units(&entries);
        assert_eq!(units.len(), 2);
    }

    #[test]
    fn test_unit_category() {
        assert_eq!(
            UnitCategory::from_unit_name("systemd-journald.service"),
            UnitCategory::System
        );
        assert_eq!(
            UnitCategory::from_unit_name("NetworkManager.service"),
            UnitCategory::Network
        );
        assert_eq!(
            UnitCategory::from_unit_name("sshd.service"),
            UnitCategory::Security
        );
    }
}
