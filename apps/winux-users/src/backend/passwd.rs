//! /etc/passwd and /etc/group parsing utilities
//!
//! This module provides utilities for parsing system user and group
//! information from the standard Unix files.

use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

/// Entry from /etc/passwd
#[derive(Debug, Clone)]
pub struct PasswdEntry {
    /// Username
    pub username: String,
    /// Password placeholder (usually 'x' indicating shadow file)
    pub password: String,
    /// User ID
    pub uid: u32,
    /// Primary group ID
    pub gid: u32,
    /// GECOS field (real name and other info)
    pub gecos: String,
    /// Home directory
    pub home_dir: String,
    /// Login shell
    pub shell: String,
}

impl PasswdEntry {
    /// Parse a line from /etc/passwd
    pub fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 7 {
            return None;
        }

        Some(PasswdEntry {
            username: parts[0].to_string(),
            password: parts[1].to_string(),
            uid: parts[2].parse().ok()?,
            gid: parts[3].parse().ok()?,
            gecos: parts[4].to_string(),
            home_dir: parts[5].to_string(),
            shell: parts[6].to_string(),
        })
    }

    /// Get the real name from GECOS field
    pub fn real_name(&self) -> &str {
        self.gecos.split(',').next().unwrap_or(&self.username)
    }

    /// Check if this is a system user
    pub fn is_system_user(&self) -> bool {
        self.uid < 1000 && self.uid != 0
    }

    /// Check if this is a regular user
    pub fn is_regular_user(&self) -> bool {
        self.uid >= 1000 && self.uid < 65534
    }

    /// Check if user can login (has valid shell)
    pub fn can_login(&self) -> bool {
        !self.shell.contains("nologin") && !self.shell.contains("false")
    }
}

/// Entry from /etc/group
#[derive(Debug, Clone)]
pub struct GroupEntry {
    /// Group name
    pub name: String,
    /// Password placeholder
    pub password: String,
    /// Group ID
    pub gid: u32,
    /// List of member usernames
    pub members: Vec<String>,
}

impl GroupEntry {
    /// Parse a line from /etc/group
    pub fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 4 {
            return None;
        }

        let members: Vec<String> = if parts[3].is_empty() {
            Vec::new()
        } else {
            parts[3].split(',').map(|s| s.to_string()).collect()
        };

        Some(GroupEntry {
            name: parts[0].to_string(),
            password: parts[1].to_string(),
            gid: parts[2].parse().ok()?,
            members,
        })
    }

    /// Check if this is a system group
    pub fn is_system_group(&self) -> bool {
        self.gid < 1000
    }

    /// Check if a user is a member of this group
    pub fn has_member(&self, username: &str) -> bool {
        self.members.iter().any(|m| m == username)
    }
}

/// Entry from /etc/shadow (requires root)
#[derive(Debug, Clone)]
pub struct ShadowEntry {
    /// Username
    pub username: String,
    /// Encrypted password
    pub password_hash: String,
    /// Days since epoch of last password change
    pub last_changed: Option<i64>,
    /// Minimum days between password changes
    pub min_age: Option<i64>,
    /// Maximum days between password changes
    pub max_age: Option<i64>,
    /// Days before expiry to warn user
    pub warn_period: Option<i64>,
    /// Days after expiry until account is disabled
    pub inactive_period: Option<i64>,
    /// Days since epoch when account expires
    pub expiry_date: Option<i64>,
}

impl ShadowEntry {
    /// Parse a line from /etc/shadow
    pub fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 9 {
            return None;
        }

        Some(ShadowEntry {
            username: parts[0].to_string(),
            password_hash: parts[1].to_string(),
            last_changed: parts[2].parse().ok(),
            min_age: parts[3].parse().ok(),
            max_age: parts[4].parse().ok(),
            warn_period: parts[5].parse().ok(),
            inactive_period: parts[6].parse().ok(),
            expiry_date: parts[7].parse().ok(),
        })
    }

    /// Check if password is locked
    pub fn is_locked(&self) -> bool {
        self.password_hash.starts_with('!') || self.password_hash.starts_with('*')
    }

    /// Check if password needs to be changed
    pub fn password_expired(&self) -> bool {
        // If max_age is set and last_changed is known
        if let (Some(max_age), Some(last_changed)) = (self.max_age, self.last_changed) {
            if max_age > 0 {
                let now = chrono::Utc::now().timestamp() / 86400; // Days since epoch
                return now > last_changed + max_age;
            }
        }
        false
    }

    /// Check if account is expired
    pub fn account_expired(&self) -> bool {
        if let Some(expiry) = self.expiry_date {
            if expiry > 0 {
                let now = chrono::Utc::now().timestamp() / 86400;
                return now > expiry;
            }
        }
        false
    }
}

/// Parse /etc/passwd file
pub fn parse_passwd() -> Result<Vec<PasswdEntry>> {
    parse_passwd_file("/etc/passwd")
}

/// Parse a passwd-format file
pub fn parse_passwd_file<P: AsRef<Path>>(path: P) -> Result<Vec<PasswdEntry>> {
    let content = fs::read_to_string(path.as_ref())
        .map_err(|e| anyhow!("Failed to read passwd file: {}", e))?;

    let entries: Vec<PasswdEntry> = content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(PasswdEntry::from_line)
        .collect();

    Ok(entries)
}

/// Parse /etc/group file
pub fn parse_group() -> Result<Vec<GroupEntry>> {
    parse_group_file("/etc/group")
}

/// Parse a group-format file
pub fn parse_group_file<P: AsRef<Path>>(path: P) -> Result<Vec<GroupEntry>> {
    let content = fs::read_to_string(path.as_ref())
        .map_err(|e| anyhow!("Failed to read group file: {}", e))?;

    let entries: Vec<GroupEntry> = content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(GroupEntry::from_line)
        .collect();

    Ok(entries)
}

/// Parse /etc/shadow file (requires root privileges)
pub fn parse_shadow() -> Result<Vec<ShadowEntry>> {
    parse_shadow_file("/etc/shadow")
}

/// Parse a shadow-format file
pub fn parse_shadow_file<P: AsRef<Path>>(path: P) -> Result<Vec<ShadowEntry>> {
    let content = fs::read_to_string(path.as_ref())
        .map_err(|e| anyhow!("Failed to read shadow file (may require root): {}", e))?;

    let entries: Vec<ShadowEntry> = content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(ShadowEntry::from_line)
        .collect();

    Ok(entries)
}

/// Get all groups a user belongs to
pub fn get_user_groups(username: &str) -> Result<Vec<GroupEntry>> {
    let groups = parse_group()?;

    let user_groups: Vec<GroupEntry> = groups
        .into_iter()
        .filter(|g| g.has_member(username))
        .collect();

    Ok(user_groups)
}

/// Get user's primary group
pub fn get_primary_group(gid: u32) -> Result<Option<GroupEntry>> {
    let groups = parse_group()?;
    Ok(groups.into_iter().find(|g| g.gid == gid))
}

/// Get all users who can login
pub fn get_login_users() -> Result<Vec<PasswdEntry>> {
    let entries = parse_passwd()?;

    let login_users: Vec<PasswdEntry> = entries
        .into_iter()
        .filter(|e| e.can_login() && e.is_regular_user())
        .collect();

    Ok(login_users)
}

/// Get a specific user by username
pub fn get_user_by_name(username: &str) -> Result<Option<PasswdEntry>> {
    let entries = parse_passwd()?;
    Ok(entries.into_iter().find(|e| e.username == username))
}

/// Get a specific user by UID
pub fn get_user_by_uid(uid: u32) -> Result<Option<PasswdEntry>> {
    let entries = parse_passwd()?;
    Ok(entries.into_iter().find(|e| e.uid == uid))
}

/// Get a specific group by name
pub fn get_group_by_name(name: &str) -> Result<Option<GroupEntry>> {
    let groups = parse_group()?;
    Ok(groups.into_iter().find(|g| g.name == name))
}

/// Get a specific group by GID
pub fn get_group_by_gid(gid: u32) -> Result<Option<GroupEntry>> {
    let groups = parse_group()?;
    Ok(groups.into_iter().find(|g| g.gid == gid))
}

/// Check if a user is in a specific group
pub fn is_user_in_group(username: &str, group_name: &str) -> Result<bool> {
    let group = get_group_by_name(group_name)?;
    Ok(group.map(|g| g.has_member(username)).unwrap_or(false))
}

/// Check if a user is an administrator (in wheel or sudo group)
pub fn is_user_admin(username: &str) -> Result<bool> {
    let in_wheel = is_user_in_group(username, "wheel")?;
    let in_sudo = is_user_in_group(username, "sudo")?;
    Ok(in_wheel || in_sudo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_passwd_line() {
        let line = "john:x:1000:1000:John Doe,,,:/home/john:/bin/bash";
        let entry = PasswdEntry::from_line(line).unwrap();

        assert_eq!(entry.username, "john");
        assert_eq!(entry.uid, 1000);
        assert_eq!(entry.gid, 1000);
        assert_eq!(entry.real_name(), "John Doe");
        assert_eq!(entry.home_dir, "/home/john");
        assert_eq!(entry.shell, "/bin/bash");
        assert!(entry.can_login());
        assert!(entry.is_regular_user());
    }

    #[test]
    fn test_parse_group_line() {
        let line = "wheel:x:10:root,admin,john";
        let entry = GroupEntry::from_line(line).unwrap();

        assert_eq!(entry.name, "wheel");
        assert_eq!(entry.gid, 10);
        assert_eq!(entry.members, vec!["root", "admin", "john"]);
        assert!(entry.has_member("john"));
        assert!(!entry.has_member("jane"));
        assert!(entry.is_system_group());
    }

    #[test]
    fn test_nologin_shell() {
        let line = "daemon:x:2:2:daemon:/:/usr/sbin/nologin";
        let entry = PasswdEntry::from_line(line).unwrap();

        assert!(!entry.can_login());
        assert!(entry.is_system_user());
    }

    #[test]
    fn test_empty_group_members() {
        let line = "nogroup:x:65534:";
        let entry = GroupEntry::from_line(line).unwrap();

        assert_eq!(entry.members.len(), 0);
    }
}
