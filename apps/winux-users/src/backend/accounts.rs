//! AccountsService D-Bus integration
//!
//! This module provides integration with the AccountsService D-Bus API
//! for user and account management operations.
//!
//! AccountsService is part of the freedesktop.org project and provides
//! a D-Bus interface for managing user accounts on Linux systems.
//!
//! D-Bus paths:
//! - Service: org.freedesktop.Accounts
//! - Manager: /org/freedesktop/Accounts
//! - User: /org/freedesktop/Accounts/User{UID}

use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    /// User ID
    pub uid: u64,
    /// Group ID
    pub gid: u64,
    /// Username
    pub username: String,
    /// Real name (display name)
    pub real_name: String,
    /// Home directory path
    pub home_dir: PathBuf,
    /// Login shell
    pub shell: String,
    /// Account type (0 = standard, 1 = administrator)
    pub account_type: AccountType,
    /// Email address
    pub email: String,
    /// Language
    pub language: String,
    /// Path to icon/avatar
    pub icon_file: Option<PathBuf>,
    /// Whether account is locked
    pub locked: bool,
    /// Whether account has automatic login enabled
    pub automatic_login: bool,
    /// Last login timestamp
    pub login_time: Option<i64>,
    /// Password mode
    pub password_mode: PasswordMode,
    /// Password hint
    pub password_hint: String,
}

/// Account type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Standard user with limited privileges
    Standard = 0,
    /// Administrator with sudo/wheel access
    Administrator = 1,
}

impl From<i32> for AccountType {
    fn from(value: i32) -> Self {
        match value {
            1 => AccountType::Administrator,
            _ => AccountType::Standard,
        }
    }
}

/// Password mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordMode {
    /// Regular password authentication
    Regular = 0,
    /// Password must be set at next login
    SetAtLogin = 1,
    /// No password required
    None = 2,
}

impl From<i32> for PasswordMode {
    fn from(value: i32) -> Self {
        match value {
            1 => PasswordMode::SetAtLogin,
            2 => PasswordMode::None,
            _ => PasswordMode::Regular,
        }
    }
}

/// AccountsService D-Bus client
///
/// Provides methods to interact with the system's AccountsService
/// for user management operations.
pub struct AccountsService {
    /// Cached list of users
    users: HashMap<u64, UserAccount>,
    /// Whether the service is available
    available: bool,
}

impl AccountsService {
    /// Create a new AccountsService client
    pub fn new() -> Self {
        let mut service = AccountsService {
            users: HashMap::new(),
            available: false,
        };

        // Try to connect and load users
        if let Err(e) = service.check_service() {
            tracing::warn!("AccountsService not available: {}", e);
        }

        service
    }

    /// Check if the AccountsService is available
    fn check_service(&mut self) -> Result<()> {
        // In a real implementation, this would connect to D-Bus
        // For now, we'll read from /etc/passwd directly
        self.available = std::path::Path::new("/etc/passwd").exists();

        if self.available {
            self.load_users()?;
        }

        Ok(())
    }

    /// Load users from the system
    fn load_users(&mut self) -> Result<()> {
        use crate::backend::passwd::parse_passwd;

        let entries = parse_passwd()?;

        for entry in entries {
            // Only include regular users (UID >= 1000) and root
            if entry.uid >= 1000 || entry.uid == 0 {
                let is_admin = self.check_user_admin(&entry.username);

                let user = UserAccount {
                    uid: entry.uid as u64,
                    gid: entry.gid as u64,
                    username: entry.username.clone(),
                    real_name: entry.gecos.split(',').next().unwrap_or(&entry.username).to_string(),
                    home_dir: PathBuf::from(&entry.home_dir),
                    shell: entry.shell.clone(),
                    account_type: if is_admin { AccountType::Administrator } else { AccountType::Standard },
                    email: String::new(),
                    language: std::env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".to_string()),
                    icon_file: Self::find_user_icon(&entry.username, &entry.home_dir),
                    locked: entry.shell.contains("nologin") || entry.shell.contains("false"),
                    automatic_login: self.check_autologin(&entry.username),
                    login_time: None,
                    password_mode: PasswordMode::Regular,
                    password_hint: String::new(),
                };

                self.users.insert(entry.uid as u64, user);
            }
        }

        Ok(())
    }

    /// Check if user is an administrator
    fn check_user_admin(&self, username: &str) -> bool {
        if let Ok(content) = std::fs::read_to_string("/etc/group") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let group_name = parts[0];
                    let members = parts[3];

                    if group_name == "wheel" || group_name == "sudo" {
                        if members.split(',').any(|m| m == username) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if user has autologin enabled
    fn check_autologin(&self, username: &str) -> bool {
        // Check GDM autologin
        if let Ok(content) = std::fs::read_to_string("/etc/gdm/custom.conf") {
            if content.contains(&format!("AutomaticLogin={}", username)) {
                return true;
            }
        }

        // Check LightDM autologin
        if let Ok(content) = std::fs::read_to_string("/etc/lightdm/lightdm.conf") {
            if content.contains(&format!("autologin-user={}", username)) {
                return true;
            }
        }

        // Check SDDM autologin
        if let Ok(content) = std::fs::read_to_string("/etc/sddm.conf") {
            if content.contains(&format!("User={}", username)) {
                return true;
            }
        }

        false
    }

    /// Find user icon file
    fn find_user_icon(username: &str, home_dir: &str) -> Option<PathBuf> {
        // Check AccountsService icon location
        let accounts_icon = PathBuf::from(format!("/var/lib/AccountsService/icons/{}", username));
        if accounts_icon.exists() {
            return Some(accounts_icon);
        }

        // Check home directory for common avatar files
        let home = PathBuf::from(home_dir);
        let avatar_paths = [
            home.join(".face"),
            home.join(".face.icon"),
            home.join(".avatar"),
            home.join(".local/share/avatars/user.png"),
        ];

        for path in avatar_paths {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// List all users
    pub fn list_users(&self) -> Vec<&UserAccount> {
        self.users.values().collect()
    }

    /// Get user by UID
    pub fn get_user(&self, uid: u64) -> Option<&UserAccount> {
        self.users.get(&uid)
    }

    /// Get user by username
    pub fn get_user_by_name(&self, username: &str) -> Option<&UserAccount> {
        self.users.values().find(|u| u.username == username)
    }

    /// Create a new user account
    ///
    /// This operation requires administrative privileges and will
    /// prompt for authentication via polkit.
    pub fn create_user(&mut self, username: &str, real_name: &str, account_type: AccountType) -> Result<u64> {
        tracing::info!("Creating user: {} ({})", username, real_name);

        // Validate username
        if !Self::is_valid_username(username) {
            return Err(anyhow!("Invalid username: must start with a letter and contain only lowercase letters, numbers, underscores, and hyphens"));
        }

        // In a real implementation, this would call AccountsService via D-Bus
        // org.freedesktop.Accounts.CreateUser(name, fullname, accountType)
        //
        // For now, we'll use useradd command
        let account_type_flag = match account_type {
            AccountType::Administrator => "-G wheel",
            AccountType::Standard => "",
        };

        let output = std::process::Command::new("pkexec")
            .args(["useradd", "-m", "-c", real_name, account_type_flag, username])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                // Reload users
                self.load_users()?;

                // Return new user's UID
                self.get_user_by_name(username)
                    .map(|u| u.uid)
                    .ok_or_else(|| anyhow!("User created but not found"))
            }
            Ok(o) => Err(anyhow!("Failed to create user: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to execute useradd: {}", e)),
        }
    }

    /// Delete a user account
    ///
    /// This operation requires administrative privileges.
    pub fn delete_user(&mut self, uid: u64, remove_files: bool) -> Result<()> {
        let user = self.users.get(&uid)
            .ok_or_else(|| anyhow!("User not found"))?;

        let username = user.username.clone();
        tracing::info!("Deleting user: {}", username);

        // In a real implementation, this would call AccountsService via D-Bus
        // org.freedesktop.Accounts.DeleteUser(uid, removeFiles)

        let mut args = vec!["userdel"];
        if remove_files {
            args.push("-r");
        }
        args.push(&username);

        let output = std::process::Command::new("pkexec")
            .args(&args)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                self.users.remove(&uid);
                Ok(())
            }
            Ok(o) => Err(anyhow!("Failed to delete user: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to execute userdel: {}", e)),
        }
    }

    /// Set user's password
    ///
    /// This operation requires administrative privileges.
    pub fn set_password(&self, uid: u64, password: &str) -> Result<()> {
        let user = self.users.get(&uid)
            .ok_or_else(|| anyhow!("User not found"))?;

        tracing::info!("Setting password for user: {}", user.username);

        // In a real implementation, this would call AccountsService via D-Bus
        // org.freedesktop.Accounts.User.SetPassword(password, hint)
        //
        // For now, we use chpasswd

        let input = format!("{}:{}", user.username, password);

        let mut child = std::process::Command::new("pkexec")
            .args(["chpasswd"])
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(input.as_bytes())?;
        }

        let status = child.wait()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to set password"))
        }
    }

    /// Set user's real name
    pub fn set_real_name(&mut self, uid: u64, real_name: &str) -> Result<()> {
        let user = self.users.get(&uid)
            .ok_or_else(|| anyhow!("User not found"))?;

        tracing::info!("Setting real name for {}: {}", user.username, real_name);

        // In a real implementation: org.freedesktop.Accounts.User.SetRealName

        let output = std::process::Command::new("pkexec")
            .args(["chfn", "-f", real_name, &user.username])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                if let Some(user) = self.users.get_mut(&uid) {
                    user.real_name = real_name.to_string();
                }
                Ok(())
            }
            Ok(o) => Err(anyhow!("Failed to set real name: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to execute chfn: {}", e)),
        }
    }

    /// Set user's account type
    pub fn set_account_type(&mut self, uid: u64, account_type: AccountType) -> Result<()> {
        let user = self.users.get(&uid)
            .ok_or_else(|| anyhow!("User not found"))?;

        tracing::info!("Setting account type for {}: {:?}", user.username, account_type);

        // In a real implementation: org.freedesktop.Accounts.User.SetAccountType

        let (action, group) = match account_type {
            AccountType::Administrator => ("--add", "wheel"),
            AccountType::Standard => ("--remove", "wheel"),
        };

        let output = std::process::Command::new("pkexec")
            .args(["gpasswd", action, &user.username, group])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                if let Some(user) = self.users.get_mut(&uid) {
                    user.account_type = account_type;
                }
                Ok(())
            }
            Ok(o) => Err(anyhow!("Failed to set account type: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to execute gpasswd: {}", e)),
        }
    }

    /// Set user's icon/avatar
    pub fn set_icon_file(&mut self, uid: u64, icon_path: &std::path::Path) -> Result<()> {
        let user = self.users.get(&uid)
            .ok_or_else(|| anyhow!("User not found"))?;

        tracing::info!("Setting icon for {}: {:?}", user.username, icon_path);

        // Copy icon to AccountsService location
        let dest = PathBuf::from(format!("/var/lib/AccountsService/icons/{}", user.username));

        let output = std::process::Command::new("pkexec")
            .args(["cp", icon_path.to_str().unwrap_or_default(), dest.to_str().unwrap_or_default()])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                if let Some(user) = self.users.get_mut(&uid) {
                    user.icon_file = Some(dest);
                }
                Ok(())
            }
            Ok(o) => Err(anyhow!("Failed to set icon: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to copy icon: {}", e)),
        }
    }

    /// Set automatic login
    pub fn set_automatic_login(&mut self, uid: u64, enabled: bool) -> Result<()> {
        let user = self.users.get(&uid)
            .ok_or_else(|| anyhow!("User not found"))?;

        tracing::info!("Setting automatic login for {}: {}", user.username, enabled);

        // In a real implementation: org.freedesktop.Accounts.User.SetAutomaticLogin

        // This would need to modify the display manager configuration
        // For GDM: /etc/gdm/custom.conf
        // For LightDM: /etc/lightdm/lightdm.conf
        // For SDDM: /etc/sddm.conf

        if let Some(user) = self.users.get_mut(&uid) {
            user.automatic_login = enabled;
        }

        tracing::warn!("Automatic login configuration not yet implemented");
        Ok(())
    }

    /// Lock user account
    pub fn set_locked(&mut self, uid: u64, locked: bool) -> Result<()> {
        let user = self.users.get(&uid)
            .ok_or_else(|| anyhow!("User not found"))?;

        tracing::info!("Setting locked for {}: {}", user.username, locked);

        let flag = if locked { "-L" } else { "-U" };

        let output = std::process::Command::new("pkexec")
            .args(["usermod", flag, &user.username])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                if let Some(user) = self.users.get_mut(&uid) {
                    user.locked = locked;
                }
                Ok(())
            }
            Ok(o) => Err(anyhow!("Failed to set locked: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to execute usermod: {}", e)),
        }
    }

    /// Add user to group
    pub fn add_user_to_group(&self, username: &str, group: &str) -> Result<()> {
        tracing::info!("Adding {} to group {}", username, group);

        let output = std::process::Command::new("pkexec")
            .args(["gpasswd", "--add", username, group])
            .output();

        match output {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(anyhow!("Failed to add to group: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to execute gpasswd: {}", e)),
        }
    }

    /// Remove user from group
    pub fn remove_user_from_group(&self, username: &str, group: &str) -> Result<()> {
        tracing::info!("Removing {} from group {}", username, group);

        let output = std::process::Command::new("pkexec")
            .args(["gpasswd", "--delete", username, group])
            .output();

        match output {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(anyhow!("Failed to remove from group: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to execute gpasswd: {}", e)),
        }
    }

    /// Validate username format
    fn is_valid_username(username: &str) -> bool {
        if username.is_empty() || username.len() > 32 {
            return false;
        }

        let first_char = username.chars().next().unwrap();
        if !first_char.is_ascii_lowercase() {
            return false;
        }

        username.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
        })
    }

    /// Refresh user list
    pub fn refresh(&mut self) -> Result<()> {
        self.users.clear();
        self.load_users()
    }

    /// Check if service is available
    pub fn is_available(&self) -> bool {
        self.available
    }
}

impl Default for AccountsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        assert!(AccountsService::is_valid_username("john"));
        assert!(AccountsService::is_valid_username("john_doe"));
        assert!(AccountsService::is_valid_username("john-doe"));
        assert!(AccountsService::is_valid_username("john123"));

        assert!(!AccountsService::is_valid_username(""));
        assert!(!AccountsService::is_valid_username("123john"));
        assert!(!AccountsService::is_valid_username("John"));
        assert!(!AccountsService::is_valid_username("john doe"));
        assert!(!AccountsService::is_valid_username("john.doe"));
    }
}
