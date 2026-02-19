//! FirewallD backend
//!
//! Alternative firewall backend for Fedora/RHEL/CentOS based systems.
//! Uses firewall-cmd for configuration.

use anyhow::{anyhow, Context, Result};
use std::process::{Command, Output};
use tracing::{debug, error, info, warn};

use super::{
    Action, Direction, FirewallOps, FirewallRule, FirewallStatus, LogLevel, Policy, Protocol,
};

/// FirewallD zones
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Public,
    Home,
    Work,
    Internal,
    Trusted,
    Drop,
    Block,
    External,
    Dmz,
}

impl Zone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Zone::Public => "public",
            Zone::Home => "home",
            Zone::Work => "work",
            Zone::Internal => "internal",
            Zone::Trusted => "trusted",
            Zone::Drop => "drop",
            Zone::Block => "block",
            Zone::External => "external",
            Zone::Dmz => "dmz",
        }
    }
}

impl std::fmt::Display for Zone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// FirewallD backend implementation
pub struct FirewalldBackend {
    default_zone: Zone,
}

impl FirewalldBackend {
    pub fn new() -> Self {
        Self {
            default_zone: Zone::Public,
        }
    }

    pub fn with_zone(zone: Zone) -> Self {
        Self { default_zone: zone }
    }

    /// Execute a firewall-cmd command with pkexec
    fn execute(&self, args: &[&str]) -> Result<Output> {
        let output = Command::new("pkexec")
            .arg("firewall-cmd")
            .args(args)
            .output()
            .context("Failed to execute firewall-cmd")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("firewall-cmd failed: {}", stderr);
            return Err(anyhow!("firewall-cmd failed: {}", stderr));
        }

        Ok(output)
    }

    /// Execute without sudo (for queries)
    fn execute_query(&self, args: &[&str]) -> Result<Output> {
        let output = Command::new("firewall-cmd")
            .args(args)
            .output()
            .context("Failed to execute firewall-cmd")?;

        Ok(output)
    }

    /// Get the default zone
    pub fn get_default_zone(&self) -> Result<String> {
        let output = self.execute_query(&["--get-default-zone"])?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Set the default zone
    pub fn set_default_zone(&self, zone: Zone) -> Result<()> {
        info!("Setting default zone to: {}", zone);
        self.execute(&["--set-default-zone", zone.as_str()])?;
        Ok(())
    }

    /// List all zones
    pub fn list_zones(&self) -> Result<Vec<String>> {
        let output = self.execute_query(&["--get-zones"])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.split_whitespace().map(|s| s.to_string()).collect())
    }

    /// List active zones
    pub fn list_active_zones(&self) -> Result<String> {
        let output = self.execute_query(&["--get-active-zones"])?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Add a service to a zone
    pub fn add_service(&self, service: &str, zone: Option<Zone>, permanent: bool) -> Result<()> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let mut args = vec!["--add-service", service, "--zone", zone_name.as_str()];

        if permanent {
            args.push("--permanent");
        }

        info!("Adding service {} to zone {}", service, zone_name);
        self.execute(&args)?;
        Ok(())
    }

    /// Remove a service from a zone
    pub fn remove_service(&self, service: &str, zone: Option<Zone>, permanent: bool) -> Result<()> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let mut args = vec!["--remove-service", service, "--zone", zone_name.as_str()];

        if permanent {
            args.push("--permanent");
        }

        info!("Removing service {} from zone {}", service, zone_name);
        self.execute(&args)?;
        Ok(())
    }

    /// Add a port to a zone
    pub fn add_port(&self, port: &str, protocol: &str, zone: Option<Zone>, permanent: bool) -> Result<()> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let port_spec = format!("{}/{}", port, protocol);
        let mut args = vec!["--add-port", &port_spec, "--zone", zone_name.as_str()];

        if permanent {
            args.push("--permanent");
        }

        info!("Adding port {} to zone {}", port_spec, zone_name);
        self.execute(&args)?;
        Ok(())
    }

    /// Remove a port from a zone
    pub fn remove_port(&self, port: &str, protocol: &str, zone: Option<Zone>, permanent: bool) -> Result<()> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let port_spec = format!("{}/{}", port, protocol);
        let mut args = vec!["--remove-port", &port_spec, "--zone", zone_name.as_str()];

        if permanent {
            args.push("--permanent");
        }

        info!("Removing port {} from zone {}", port_spec, zone_name);
        self.execute(&args)?;
        Ok(())
    }

    /// List services in a zone
    pub fn list_services(&self, zone: Option<Zone>) -> Result<Vec<String>> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let output = self.execute_query(&["--list-services", "--zone", zone_name.as_str()])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.split_whitespace().map(|s| s.to_string()).collect())
    }

    /// List ports in a zone
    pub fn list_ports(&self, zone: Option<Zone>) -> Result<Vec<String>> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let output = self.execute_query(&["--list-ports", "--zone", zone_name.as_str()])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.split_whitespace().map(|s| s.to_string()).collect())
    }

    /// List all configured rules for a zone
    pub fn list_all(&self, zone: Option<Zone>) -> Result<String> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let output = self.execute_query(&["--list-all", "--zone", zone_name.as_str()])?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Add rich rule
    pub fn add_rich_rule(&self, rule: &str, zone: Option<Zone>, permanent: bool) -> Result<()> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let mut args = vec!["--add-rich-rule", rule, "--zone", zone_name.as_str()];

        if permanent {
            args.push("--permanent");
        }

        info!("Adding rich rule to zone {}: {}", zone_name, rule);
        self.execute(&args)?;
        Ok(())
    }

    /// Remove rich rule
    pub fn remove_rich_rule(&self, rule: &str, zone: Option<Zone>, permanent: bool) -> Result<()> {
        let zone_name = zone.unwrap_or(self.default_zone);
        let mut args = vec!["--remove-rich-rule", rule, "--zone", zone_name.as_str()];

        if permanent {
            args.push("--permanent");
        }

        info!("Removing rich rule from zone {}", zone_name);
        self.execute(&args)?;
        Ok(())
    }

    /// Block an IP address
    pub fn block_ip(&self, ip: &str, zone: Option<Zone>, permanent: bool) -> Result<()> {
        let rule = format!("rule family='ipv4' source address='{}' reject", ip);
        self.add_rich_rule(&rule, zone, permanent)
    }

    /// Allow an IP address
    pub fn allow_ip(&self, ip: &str, zone: Option<Zone>, permanent: bool) -> Result<()> {
        let rule = format!("rule family='ipv4' source address='{}' accept", ip);
        self.add_rich_rule(&rule, zone, permanent)
    }

    /// Enable panic mode (block all traffic)
    pub fn panic_on(&self) -> Result<()> {
        warn!("Enabling panic mode - all network traffic will be blocked!");
        self.execute(&["--panic-on"])?;
        Ok(())
    }

    /// Disable panic mode
    pub fn panic_off(&self) -> Result<()> {
        info!("Disabling panic mode");
        self.execute(&["--panic-off"])?;
        Ok(())
    }

    /// Check if panic mode is enabled
    pub fn is_panic(&self) -> Result<bool> {
        let output = self.execute_query(&["--query-panic"])?;
        Ok(output.status.success())
    }

    /// Get firewalld state
    pub fn state(&self) -> Result<String> {
        let output = self.execute_query(&["--state"])?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Convert FirewallRule to rich rule string
    fn rule_to_rich_rule(rule: &FirewallRule) -> String {
        let mut parts = vec!["rule".to_string()];

        // Family (assume IPv4 for now)
        parts.push("family='ipv4'".to_string());

        // Source IP
        if let Some(ref from_ip) = rule.from_ip {
            parts.push(format!("source address='{}'", from_ip));
        }

        // Destination IP
        if let Some(ref to_ip) = rule.to_ip {
            parts.push(format!("destination address='{}'", to_ip));
        }

        // Port and protocol
        if let Some(ref port) = rule.port {
            let proto = match rule.protocol {
                Protocol::Tcp => "tcp",
                Protocol::Udp => "udp",
                _ => "tcp",
            };
            parts.push(format!("port port='{}' protocol='{}'", port, proto));
        }

        // Action
        let action = match rule.action {
            Action::Allow => "accept",
            Action::Deny => "drop",
            Action::Reject => "reject",
            Action::Limit => "accept", // TODO: Add rate limiting
        };
        parts.push(action.to_string());

        parts.join(" ")
    }
}

impl Default for FirewalldBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FirewallOps for FirewalldBackend {
    fn is_enabled(&self) -> Result<bool> {
        let state = self.state()?;
        Ok(state == "running")
    }

    fn enable(&self) -> Result<()> {
        info!("Starting firewalld service");
        Command::new("pkexec")
            .args(["systemctl", "start", "firewalld"])
            .output()
            .context("Failed to start firewalld")?;

        Command::new("pkexec")
            .args(["systemctl", "enable", "firewalld"])
            .output()
            .context("Failed to enable firewalld")?;

        Ok(())
    }

    fn disable(&self) -> Result<()> {
        info!("Stopping firewalld service");
        Command::new("pkexec")
            .args(["systemctl", "stop", "firewalld"])
            .output()
            .context("Failed to stop firewalld")?;

        Ok(())
    }

    fn status(&self) -> Result<FirewallStatus> {
        let is_running = self.is_enabled()?;
        let default_zone = self.get_default_zone()?;

        // Determine policies based on zone
        let (incoming, outgoing) = match default_zone.as_str() {
            "drop" => (Policy::Deny, Policy::Deny),
            "block" => (Policy::Reject, Policy::Allow),
            "trusted" => (Policy::Allow, Policy::Allow),
            _ => (Policy::Deny, Policy::Allow), // Most zones deny incoming by default
        };

        let rules = self.list_rules()?;

        Ok(FirewallStatus {
            enabled: is_running,
            default_incoming: incoming,
            default_outgoing: outgoing,
            logging: LogLevel::Medium, // FirewallD doesn't have the same logging levels
            rules_count: rules.len(),
        })
    }

    fn list_rules(&self) -> Result<Vec<FirewallRule>> {
        let mut rules = Vec::new();

        // Get services as rules
        let services = self.list_services(None)?;
        for service in services {
            rules.push(FirewallRule {
                id: Some(format!("service:{}", service)),
                action: Action::Allow,
                direction: Direction::In,
                protocol: Protocol::Both,
                port: None,
                from_ip: None,
                to_ip: None,
                interface: None,
                comment: Some(format!("Service: {}", service)),
            });
        }

        // Get ports as rules
        let ports = self.list_ports(None)?;
        for port_spec in ports {
            let parts: Vec<&str> = port_spec.split('/').collect();
            if parts.len() == 2 {
                let protocol = match parts[1] {
                    "tcp" => Protocol::Tcp,
                    "udp" => Protocol::Udp,
                    _ => Protocol::Both,
                };

                rules.push(FirewallRule {
                    id: Some(format!("port:{}", port_spec)),
                    action: Action::Allow,
                    direction: Direction::In,
                    protocol,
                    port: Some(parts[0].to_string()),
                    from_ip: None,
                    to_ip: None,
                    interface: None,
                    comment: None,
                });
            }
        }

        Ok(rules)
    }

    fn add_rule(&self, rule: &FirewallRule) -> Result<()> {
        if let Some(ref port) = rule.port {
            let proto = match rule.protocol {
                Protocol::Tcp => "tcp",
                Protocol::Udp => "udp",
                _ => "tcp",
            };

            match rule.action {
                Action::Allow => {
                    self.add_port(port, proto, None, true)?;
                }
                _ => {
                    // Use rich rules for deny/reject
                    let rich_rule = Self::rule_to_rich_rule(rule);
                    self.add_rich_rule(&rich_rule, None, true)?;
                }
            }
        }

        // Reload to apply changes
        self.reload()?;
        Ok(())
    }

    fn delete_rule(&self, rule_id: &str) -> Result<()> {
        if rule_id.starts_with("service:") {
            let service = rule_id.strip_prefix("service:").unwrap();
            self.remove_service(service, None, true)?;
        } else if rule_id.starts_with("port:") {
            let port_spec = rule_id.strip_prefix("port:").unwrap();
            let parts: Vec<&str> = port_spec.split('/').collect();
            if parts.len() == 2 {
                self.remove_port(parts[0], parts[1], None, true)?;
            }
        }

        self.reload()?;
        Ok(())
    }

    fn set_default_incoming(&self, policy: Policy) -> Result<()> {
        // In firewalld, this is done by changing zones
        let zone = match policy {
            Policy::Allow => Zone::Trusted,
            Policy::Deny => Zone::Drop,
            Policy::Reject => Zone::Block,
        };

        self.set_default_zone(zone)
    }

    fn set_default_outgoing(&self, _policy: Policy) -> Result<()> {
        // FirewallD doesn't have a simple outgoing policy like UFW
        // This would require rich rules for all outgoing traffic
        warn!("FirewallD doesn't support simple outgoing policy changes");
        Ok(())
    }

    fn reload(&self) -> Result<()> {
        info!("Reloading firewalld configuration");
        self.execute(&["--reload"])?;
        Ok(())
    }

    fn reset(&self) -> Result<()> {
        warn!("Resetting firewalld to defaults");
        // Remove all custom rules and set default zone
        self.set_default_zone(Zone::Public)?;
        self.reload()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_as_str() {
        assert_eq!(Zone::Public.as_str(), "public");
        assert_eq!(Zone::Home.as_str(), "home");
        assert_eq!(Zone::Drop.as_str(), "drop");
    }

    #[test]
    fn test_rule_to_rich_rule() {
        let rule = FirewallRule {
            id: None,
            action: Action::Allow,
            direction: Direction::In,
            protocol: Protocol::Tcp,
            port: Some("22".to_string()),
            from_ip: Some("192.168.1.0/24".to_string()),
            to_ip: None,
            interface: None,
            comment: None,
        };

        let rich_rule = FirewalldBackend::rule_to_rich_rule(&rule);
        assert!(rich_rule.contains("port='22'"));
        assert!(rich_rule.contains("protocol='tcp'"));
        assert!(rich_rule.contains("source address='192.168.1.0/24'"));
        assert!(rich_rule.contains("accept"));
    }
}
