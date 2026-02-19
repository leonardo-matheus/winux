//! Firewall backend implementations
//!
//! Supports multiple firewall backends:
//! - UFW (Uncomplicated Firewall) - primary
//! - FirewallD - alternative for Fedora/RHEL based systems

pub mod ufw;
pub mod firewalld;

use anyhow::Result;
use std::process::Command;

/// Detect which firewall backend is available
pub fn detect_backend() -> FirewallBackend {
    // Check for UFW first (Ubuntu/Debian)
    if Command::new("which")
        .arg("ufw")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return FirewallBackend::Ufw;
    }

    // Check for firewall-cmd (FirewallD)
    if Command::new("which")
        .arg("firewall-cmd")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return FirewallBackend::FirewallD;
    }

    // Fallback to iptables
    FirewallBackend::Iptables
}

/// Available firewall backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallBackend {
    Ufw,
    FirewallD,
    Iptables,
}

impl std::fmt::Display for FirewallBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirewallBackend::Ufw => write!(f, "UFW"),
            FirewallBackend::FirewallD => write!(f, "FirewallD"),
            FirewallBackend::Iptables => write!(f, "iptables"),
        }
    }
}

/// Common firewall operations trait
pub trait FirewallOps {
    /// Check if the firewall is enabled
    fn is_enabled(&self) -> Result<bool>;

    /// Enable the firewall
    fn enable(&self) -> Result<()>;

    /// Disable the firewall
    fn disable(&self) -> Result<()>;

    /// Get current status
    fn status(&self) -> Result<FirewallStatus>;

    /// List all rules
    fn list_rules(&self) -> Result<Vec<FirewallRule>>;

    /// Add a new rule
    fn add_rule(&self, rule: &FirewallRule) -> Result<()>;

    /// Delete a rule
    fn delete_rule(&self, rule_id: &str) -> Result<()>;

    /// Set default incoming policy
    fn set_default_incoming(&self, policy: Policy) -> Result<()>;

    /// Set default outgoing policy
    fn set_default_outgoing(&self, policy: Policy) -> Result<()>;

    /// Reload firewall rules
    fn reload(&self) -> Result<()>;

    /// Reset to defaults
    fn reset(&self) -> Result<()>;
}

/// Firewall status information
#[derive(Debug, Clone)]
pub struct FirewallStatus {
    pub enabled: bool,
    pub default_incoming: Policy,
    pub default_outgoing: Policy,
    pub logging: LogLevel,
    pub rules_count: usize,
}

/// Firewall rule representation
#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub id: Option<String>,
    pub action: Action,
    pub direction: Direction,
    pub protocol: Protocol,
    pub port: Option<String>,
    pub from_ip: Option<String>,
    pub to_ip: Option<String>,
    pub interface: Option<String>,
    pub comment: Option<String>,
}

impl FirewallRule {
    pub fn new() -> Self {
        Self {
            id: None,
            action: Action::Allow,
            direction: Direction::In,
            protocol: Protocol::Tcp,
            port: None,
            from_ip: None,
            to_ip: None,
            interface: None,
            comment: None,
        }
    }

    pub fn allow_port(port: u16, protocol: Protocol) -> Self {
        Self {
            id: None,
            action: Action::Allow,
            direction: Direction::In,
            protocol,
            port: Some(port.to_string()),
            from_ip: None,
            to_ip: None,
            interface: None,
            comment: None,
        }
    }

    pub fn deny_port(port: u16, protocol: Protocol) -> Self {
        Self {
            id: None,
            action: Action::Deny,
            direction: Direction::In,
            protocol,
            port: Some(port.to_string()),
            from_ip: None,
            to_ip: None,
            interface: None,
            comment: None,
        }
    }
}

impl Default for FirewallRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Firewall action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Allow,
    Deny,
    Reject,
    Limit,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Allow => write!(f, "ALLOW"),
            Action::Deny => write!(f, "DENY"),
            Action::Reject => write!(f, "REJECT"),
            Action::Limit => write!(f, "LIMIT"),
        }
    }
}

/// Traffic direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
    Both,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::In => write!(f, "IN"),
            Direction::Out => write!(f, "OUT"),
            Direction::Both => write!(f, "BOTH"),
        }
    }
}

/// Network protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
    Both,
    Any,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
            Protocol::Both => write!(f, "tcp/udp"),
            Protocol::Any => write!(f, "any"),
        }
    }
}

/// Default policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Allow,
    Deny,
    Reject,
}

impl std::fmt::Display for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Policy::Allow => write!(f, "allow"),
            Policy::Deny => write!(f, "deny"),
            Policy::Reject => write!(f, "reject"),
        }
    }
}

/// Logging level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Off,
    Low,
    Medium,
    High,
    Full,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Off => write!(f, "off"),
            LogLevel::Low => write!(f, "low"),
            LogLevel::Medium => write!(f, "medium"),
            LogLevel::High => write!(f, "high"),
            LogLevel::Full => write!(f, "full"),
        }
    }
}

/// Security preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Home,
    Public,
    Server,
    Custom,
}

impl Preset {
    /// Get description for the preset
    pub fn description(&self) -> &'static str {
        match self {
            Preset::Home => "Allows local network discovery and file sharing",
            Preset::Public => "Restrictive, blocks most incoming connections",
            Preset::Server => "Allows SSH, HTTP, HTTPS only",
            Preset::Custom => "User-defined configuration",
        }
    }

    /// Get rules for this preset
    pub fn rules(&self) -> Vec<FirewallRule> {
        match self {
            Preset::Home => vec![
                FirewallRule::allow_port(22, Protocol::Tcp),   // SSH
                FirewallRule::allow_port(445, Protocol::Tcp),  // SMB
                FirewallRule::allow_port(137, Protocol::Udp),  // NetBIOS
                FirewallRule::allow_port(138, Protocol::Udp),  // NetBIOS
                FirewallRule::allow_port(5353, Protocol::Udp), // mDNS
            ],
            Preset::Public => vec![
                // Only allow established connections
            ],
            Preset::Server => vec![
                FirewallRule::allow_port(22, Protocol::Tcp),   // SSH
                FirewallRule::allow_port(80, Protocol::Tcp),   // HTTP
                FirewallRule::allow_port(443, Protocol::Tcp),  // HTTPS
            ],
            Preset::Custom => vec![],
        }
    }
}
