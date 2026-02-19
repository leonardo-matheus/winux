//! UFW (Uncomplicated Firewall) backend
//!
//! Primary firewall backend for Ubuntu/Debian based systems.
//! All operations require root privileges (sudo).

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::process::{Command, Output};
use tracing::{debug, error, info, warn};

use super::{
    Action, Direction, FirewallOps, FirewallRule, FirewallStatus, LogLevel, Policy, Protocol,
};

/// UFW backend implementation
pub struct UfwBackend;

impl UfwBackend {
    pub fn new() -> Self {
        Self
    }

    /// Execute a UFW command with sudo
    fn execute(&self, args: &[&str]) -> Result<Output> {
        let output = Command::new("pkexec")
            .arg("ufw")
            .args(args)
            .output()
            .context("Failed to execute UFW command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("UFW command failed: {}", stderr);
            return Err(anyhow!("UFW command failed: {}", stderr));
        }

        Ok(output)
    }

    /// Execute a UFW command without sudo (for status checks)
    fn execute_no_sudo(&self, args: &[&str]) -> Result<Output> {
        let output = Command::new("ufw")
            .args(args)
            .output()
            .context("Failed to execute UFW command")?;

        Ok(output)
    }

    /// Parse UFW status output
    fn parse_status(output: &str) -> FirewallStatus {
        let enabled = output.contains("Status: active");

        let default_incoming = if output.contains("Default: deny (incoming)") {
            Policy::Deny
        } else if output.contains("Default: reject (incoming)") {
            Policy::Reject
        } else {
            Policy::Allow
        };

        let default_outgoing = if output.contains("allow (outgoing)") {
            Policy::Allow
        } else if output.contains("deny (outgoing)") {
            Policy::Deny
        } else {
            Policy::Reject
        };

        let logging = if output.contains("Logging: on (low)") {
            LogLevel::Low
        } else if output.contains("Logging: on (medium)") {
            LogLevel::Medium
        } else if output.contains("Logging: on (high)") {
            LogLevel::High
        } else if output.contains("Logging: on (full)") {
            LogLevel::Full
        } else {
            LogLevel::Off
        };

        // Count rules (lines that start with "[")
        let rules_count = output.lines().filter(|l| l.trim().starts_with('[') || l.contains("ALLOW") || l.contains("DENY")).count();

        FirewallStatus {
            enabled,
            default_incoming,
            default_outgoing,
            logging,
            rules_count,
        }
    }

    /// Parse UFW status numbered output to extract rules
    fn parse_rules(output: &str) -> Vec<FirewallRule> {
        let mut rules = Vec::new();

        // Pattern: [ 1] 22/tcp                     ALLOW IN    Anywhere
        let rule_regex = Regex::new(
            r"\[\s*(\d+)\]\s+(\S+)\s+(ALLOW|DENY|REJECT|LIMIT)\s+(IN|OUT)?\s*(.*)?"
        ).unwrap();

        for line in output.lines() {
            if let Some(caps) = rule_regex.captures(line) {
                let id = caps.get(1).map(|m| m.as_str().to_string());
                let port_proto = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let action_str = caps.get(3).map(|m| m.as_str()).unwrap_or("ALLOW");
                let direction_str = caps.get(4).map(|m| m.as_str()).unwrap_or("IN");
                let from_to = caps.get(5).map(|m| m.as_str()).unwrap_or("Anywhere");

                // Parse port and protocol
                let (port, protocol) = if port_proto.contains('/') {
                    let parts: Vec<&str> = port_proto.split('/').collect();
                    let port = parts.first().map(|s| s.to_string());
                    let proto = match parts.get(1) {
                        Some(&"tcp") => Protocol::Tcp,
                        Some(&"udp") => Protocol::Udp,
                        _ => Protocol::Both,
                    };
                    (port, proto)
                } else {
                    (Some(port_proto.to_string()), Protocol::Both)
                };

                let action = match action_str {
                    "ALLOW" => Action::Allow,
                    "DENY" => Action::Deny,
                    "REJECT" => Action::Reject,
                    "LIMIT" => Action::Limit,
                    _ => Action::Allow,
                };

                let direction = match direction_str {
                    "IN" => Direction::In,
                    "OUT" => Direction::Out,
                    _ => Direction::In,
                };

                let from_ip = if from_to != "Anywhere" && !from_to.is_empty() {
                    Some(from_to.trim().to_string())
                } else {
                    None
                };

                rules.push(FirewallRule {
                    id,
                    action,
                    direction,
                    protocol,
                    port,
                    from_ip,
                    to_ip: None,
                    interface: None,
                    comment: None,
                });
            }
        }

        rules
    }

    /// Build UFW command arguments for a rule
    fn build_rule_args(rule: &FirewallRule) -> Vec<String> {
        let mut args = Vec::new();

        // Action
        args.push(rule.action.to_string().to_lowercase());

        // Direction
        match rule.direction {
            Direction::In => args.push("in".to_string()),
            Direction::Out => args.push("out".to_string()),
            Direction::Both => {} // Don't specify direction for both
        }

        // From IP
        if let Some(ref from_ip) = rule.from_ip {
            args.push("from".to_string());
            args.push(from_ip.clone());
        }

        // To IP
        if let Some(ref to_ip) = rule.to_ip {
            args.push("to".to_string());
            args.push(to_ip.clone());
        }

        // Port and protocol
        if let Some(ref port) = rule.port {
            args.push("port".to_string());
            args.push(port.clone());
            args.push("proto".to_string());
            args.push(rule.protocol.to_string());
        }

        // Interface
        if let Some(ref iface) = rule.interface {
            args.push("on".to_string());
            args.push(iface.clone());
        }

        // Comment
        if let Some(ref comment) = rule.comment {
            args.push("comment".to_string());
            args.push(comment.clone());
        }

        args
    }

    /// Allow a specific port
    pub fn allow_port(&self, port: &str, protocol: &str) -> Result<()> {
        info!("Allowing port {}/{}", port, protocol);
        self.execute(&["allow", &format!("{}/{}", port, protocol)])?;
        Ok(())
    }

    /// Deny a specific port
    pub fn deny_port(&self, port: &str, protocol: &str) -> Result<()> {
        info!("Denying port {}/{}", port, protocol);
        self.execute(&["deny", &format!("{}/{}", port, protocol)])?;
        Ok(())
    }

    /// Allow an application profile
    pub fn allow_app(&self, app_name: &str) -> Result<()> {
        info!("Allowing app profile: {}", app_name);
        self.execute(&["allow", app_name])?;
        Ok(())
    }

    /// Deny an application profile
    pub fn deny_app(&self, app_name: &str) -> Result<()> {
        info!("Denying app profile: {}", app_name);
        self.execute(&["deny", app_name])?;
        Ok(())
    }

    /// List available application profiles
    pub fn list_app_profiles(&self) -> Result<Vec<String>> {
        let output = self.execute(&["app", "list"])?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let profiles: Vec<String> = stdout
            .lines()
            .skip(1) // Skip "Available applications:" header
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(profiles)
    }

    /// Get info about an application profile
    pub fn app_info(&self, app_name: &str) -> Result<String> {
        let output = self.execute(&["app", "info", app_name])?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Set logging level
    pub fn set_logging(&self, level: LogLevel) -> Result<()> {
        let level_str = match level {
            LogLevel::Off => "off",
            LogLevel::Low => "low",
            LogLevel::Medium => "medium",
            LogLevel::High => "high",
            LogLevel::Full => "full",
        };

        info!("Setting UFW logging to: {}", level_str);
        self.execute(&["logging", level_str])?;
        Ok(())
    }

    /// Limit connections (rate limiting)
    pub fn limit_port(&self, port: &str, protocol: &str) -> Result<()> {
        info!("Limiting port {}/{}", port, protocol);
        self.execute(&["limit", &format!("{}/{}", port, protocol)])?;
        Ok(())
    }

    /// Insert a rule at a specific position
    pub fn insert_rule(&self, position: usize, rule: &FirewallRule) -> Result<()> {
        let mut args = vec!["insert".to_string(), position.to_string()];
        args.extend(Self::build_rule_args(rule));

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        info!("Inserting rule at position {}", position);
        self.execute(&args_ref)?;
        Ok(())
    }

    /// Show raw status output
    pub fn raw_status(&self) -> Result<String> {
        let output = Command::new("sudo")
            .args(["ufw", "status", "verbose"])
            .output()
            .context("Failed to get UFW status")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Show numbered rules
    pub fn numbered_status(&self) -> Result<String> {
        let output = Command::new("sudo")
            .args(["ufw", "status", "numbered"])
            .output()
            .context("Failed to get UFW numbered status")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Default for UfwBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FirewallOps for UfwBackend {
    fn is_enabled(&self) -> Result<bool> {
        let output = Command::new("ufw")
            .arg("status")
            .output()
            .context("Failed to check UFW status")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("Status: active"))
    }

    fn enable(&self) -> Result<()> {
        info!("Enabling UFW firewall");
        // Use --force to skip the confirmation prompt
        self.execute(&["--force", "enable"])?;
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        info!("Disabling UFW firewall");
        self.execute(&["disable"])?;
        Ok(())
    }

    fn status(&self) -> Result<FirewallStatus> {
        let output = Command::new("sudo")
            .args(["ufw", "status", "verbose"])
            .output()
            .context("Failed to get UFW status")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Self::parse_status(&stdout))
    }

    fn list_rules(&self) -> Result<Vec<FirewallRule>> {
        let output = Command::new("sudo")
            .args(["ufw", "status", "numbered"])
            .output()
            .context("Failed to list UFW rules")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Self::parse_rules(&stdout))
    }

    fn add_rule(&self, rule: &FirewallRule) -> Result<()> {
        let args = Self::build_rule_args(rule);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        info!("Adding UFW rule: {:?}", args);
        self.execute(&args_ref)?;
        Ok(())
    }

    fn delete_rule(&self, rule_id: &str) -> Result<()> {
        info!("Deleting UFW rule: {}", rule_id);
        // Delete by rule number
        self.execute(&["--force", "delete", rule_id])?;
        Ok(())
    }

    fn set_default_incoming(&self, policy: Policy) -> Result<()> {
        info!("Setting default incoming policy to: {}", policy);
        self.execute(&["default", &policy.to_string(), "incoming"])?;
        Ok(())
    }

    fn set_default_outgoing(&self, policy: Policy) -> Result<()> {
        info!("Setting default outgoing policy to: {}", policy);
        self.execute(&["default", &policy.to_string(), "outgoing"])?;
        Ok(())
    }

    fn reload(&self) -> Result<()> {
        info!("Reloading UFW firewall");
        self.execute(&["reload"])?;
        Ok(())
    }

    fn reset(&self) -> Result<()> {
        warn!("Resetting UFW to defaults - all rules will be deleted!");
        self.execute(&["--force", "reset"])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        let output = r#"
Status: active
Logging: on (low)
Default: deny (incoming), allow (outgoing), disabled (routed)
New profiles: skip

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW IN    Anywhere
80/tcp                     ALLOW IN    Anywhere
"#;

        let status = UfwBackend::parse_status(output);
        assert!(status.enabled);
        assert_eq!(status.default_incoming, Policy::Deny);
        assert_eq!(status.default_outgoing, Policy::Allow);
        assert_eq!(status.logging, LogLevel::Low);
    }

    #[test]
    fn test_parse_rules() {
        let output = r#"
Status: active

     To                         Action      From
     --                         ------      ----
[ 1] 22/tcp                     ALLOW IN    Anywhere
[ 2] 80/tcp                     ALLOW IN    Anywhere
[ 3] 443/tcp                    ALLOW IN    Anywhere
[ 4] 3306/tcp                   DENY IN     Anywhere
"#;

        let rules = UfwBackend::parse_rules(output);
        assert_eq!(rules.len(), 4);

        assert_eq!(rules[0].port, Some("22".to_string()));
        assert_eq!(rules[0].protocol, Protocol::Tcp);
        assert_eq!(rules[0].action, Action::Allow);

        assert_eq!(rules[3].action, Action::Deny);
    }
}
