// Winux Mail - Account Management
// Copyright (c) 2026 Winux OS Project

use crate::data::cache::EmailCache;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Email provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountProvider {
    Gmail,
    Outlook,
    Yahoo,
    ICloud,
    ProtonMail,
    Custom,
}

impl AccountProvider {
    /// Get auto-discovered settings for known providers
    pub fn get_settings(&self, email: &str) -> Option<ProviderSettings> {
        match self {
            AccountProvider::Gmail => Some(ProviderSettings {
                imap_server: "imap.gmail.com".to_string(),
                imap_port: 993,
                imap_tls: true,
                smtp_server: "smtp.gmail.com".to_string(),
                smtp_port: 587,
                smtp_starttls: true,
                oauth2_supported: true,
                oauth2_auth_url: Some("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
                oauth2_token_url: Some("https://oauth2.googleapis.com/token".to_string()),
            }),
            AccountProvider::Outlook => Some(ProviderSettings {
                imap_server: "outlook.office365.com".to_string(),
                imap_port: 993,
                imap_tls: true,
                smtp_server: "smtp.office365.com".to_string(),
                smtp_port: 587,
                smtp_starttls: true,
                oauth2_supported: true,
                oauth2_auth_url: Some("https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string()),
                oauth2_token_url: Some("https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string()),
            }),
            AccountProvider::Yahoo => Some(ProviderSettings {
                imap_server: "imap.mail.yahoo.com".to_string(),
                imap_port: 993,
                imap_tls: true,
                smtp_server: "smtp.mail.yahoo.com".to_string(),
                smtp_port: 587,
                smtp_starttls: true,
                oauth2_supported: true,
                oauth2_auth_url: Some("https://api.login.yahoo.com/oauth2/request_auth".to_string()),
                oauth2_token_url: Some("https://api.login.yahoo.com/oauth2/get_token".to_string()),
            }),
            AccountProvider::ICloud => Some(ProviderSettings {
                imap_server: "imap.mail.me.com".to_string(),
                imap_port: 993,
                imap_tls: true,
                smtp_server: "smtp.mail.me.com".to_string(),
                smtp_port: 587,
                smtp_starttls: true,
                oauth2_supported: false,
                oauth2_auth_url: None,
                oauth2_token_url: None,
            }),
            AccountProvider::ProtonMail => Some(ProviderSettings {
                imap_server: "127.0.0.1".to_string(), // ProtonMail Bridge
                imap_port: 1143,
                imap_tls: false,
                smtp_server: "127.0.0.1".to_string(),
                smtp_port: 1025,
                smtp_starttls: false,
                oauth2_supported: false,
                oauth2_auth_url: None,
                oauth2_token_url: None,
            }),
            AccountProvider::Custom => None,
        }
    }

    /// Detect provider from email domain
    pub fn detect(email: &str) -> Self {
        let domain = email.split('@').nth(1).unwrap_or("").to_lowercase();

        if domain.contains("gmail") || domain.contains("google") {
            AccountProvider::Gmail
        } else if domain.contains("outlook") || domain.contains("hotmail") || domain.contains("live") || domain.contains("msn") {
            AccountProvider::Outlook
        } else if domain.contains("yahoo") {
            AccountProvider::Yahoo
        } else if domain.contains("icloud") || domain.contains("me.com") || domain.contains("mac.com") {
            AccountProvider::ICloud
        } else if domain.contains("proton") {
            AccountProvider::ProtonMail
        } else {
            AccountProvider::Custom
        }
    }
}

/// Provider-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub imap_server: String,
    pub imap_port: u16,
    pub imap_tls: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_starttls: bool,
    pub oauth2_supported: bool,
    pub oauth2_auth_url: Option<String>,
    pub oauth2_token_url: Option<String>,
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password {
        password: String,
    },
    OAuth2 {
        client_id: String,
        access_token: String,
        refresh_token: String,
        expires_at: i64,
    },
    AppPassword {
        password: String,
    },
}

/// Account settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSettings {
    pub imap_server: String,
    pub imap_port: u16,
    pub imap_tls: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_starttls: bool,
    pub sync_interval_minutes: u32,
    pub sync_days: u32,
    pub notifications_enabled: bool,
    pub notification_sound: bool,
    pub signature: Option<String>,
}

impl Default for AccountSettings {
    fn default() -> Self {
        Self {
            imap_server: String::new(),
            imap_port: 993,
            imap_tls: true,
            smtp_server: String::new(),
            smtp_port: 587,
            smtp_starttls: true,
            sync_interval_minutes: 15,
            sync_days: 30,
            notifications_enabled: true,
            notification_sound: true,
            signature: Some("Sent from Winux Mail".to_string()),
        }
    }
}

/// Email account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub provider: AccountProvider,
    pub auth: AuthMethod,
    pub settings: AccountSettings,
    pub enabled: bool,
    pub last_sync: Option<i64>,
}

impl Account {
    pub fn new(name: String, email: String, provider: AccountProvider) -> Self {
        let id = uuid::Uuid::new_v4().to_string();

        // Apply provider defaults
        let mut settings = AccountSettings::default();
        if let Some(provider_settings) = provider.get_settings(&email) {
            settings.imap_server = provider_settings.imap_server;
            settings.imap_port = provider_settings.imap_port;
            settings.imap_tls = provider_settings.imap_tls;
            settings.smtp_server = provider_settings.smtp_server;
            settings.smtp_port = provider_settings.smtp_port;
            settings.smtp_starttls = provider_settings.smtp_starttls;
        }

        Self {
            id,
            name,
            email,
            provider,
            auth: AuthMethod::Password { password: String::new() },
            settings,
            enabled: true,
            last_sync: None,
        }
    }

    pub fn set_password(&mut self, password: String) {
        self.auth = AuthMethod::Password { password };
    }

    pub fn set_oauth2(&mut self, client_id: String, access_token: String, refresh_token: String, expires_at: i64) {
        self.auth = AuthMethod::OAuth2 {
            client_id,
            access_token,
            refresh_token,
            expires_at,
        };
    }
}

/// Account manager
pub struct AccountManager {
    accounts: HashMap<String, Account>,
    cache: Arc<EmailCache>,
    config_path: PathBuf,
}

impl AccountManager {
    pub fn new(cache: Arc<EmailCache>) -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?
            .join("winux-mail");

        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("accounts.json");
        let accounts = Self::load_accounts(&config_path)?;

        Ok(Self {
            accounts,
            cache,
            config_path,
        })
    }

    fn load_accounts(path: &PathBuf) -> Result<HashMap<String, Account>> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let accounts: Vec<Account> = serde_json::from_str(&content)?;
            Ok(accounts.into_iter().map(|a| (a.id.clone(), a)).collect())
        } else {
            Ok(HashMap::new())
        }
    }

    fn save_accounts(&self) -> Result<()> {
        let accounts: Vec<&Account> = self.accounts.values().collect();
        let content = serde_json::to_string_pretty(&accounts)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn add_account(&mut self, account: Account) -> Result<()> {
        self.accounts.insert(account.id.clone(), account);
        self.save_accounts()?;
        Ok(())
    }

    pub fn remove_account(&mut self, id: &str) -> Result<()> {
        self.accounts.remove(id);
        self.save_accounts()?;
        Ok(())
    }

    pub fn get_account(&self, id: &str) -> Option<&Account> {
        self.accounts.get(id)
    }

    pub fn get_account_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.get_mut(id)
    }

    pub fn get_all_accounts(&self) -> Vec<&Account> {
        self.accounts.values().collect()
    }

    pub fn update_account(&mut self, account: Account) -> Result<()> {
        self.accounts.insert(account.id.clone(), account);
        self.save_accounts()?;
        Ok(())
    }

    /// Store password securely in keyring
    pub async fn store_password(&self, account_id: &str, password: &str) -> Result<()> {
        // Use secret-service for secure password storage
        let collection = secret_service::SecretService::connect(secret_service::EncryptionType::Dh)
            .await?
            .get_default_collection()
            .await?;

        let attributes = HashMap::from([
            ("application", "winux-mail"),
            ("account_id", account_id),
        ]);

        collection.create_item(
            &format!("Winux Mail - {}", account_id),
            attributes,
            password.as_bytes(),
            true,
            "text/plain",
        ).await?;

        Ok(())
    }

    /// Retrieve password from keyring
    pub async fn get_password(&self, account_id: &str) -> Result<String> {
        let collection = secret_service::SecretService::connect(secret_service::EncryptionType::Dh)
            .await?
            .get_default_collection()
            .await?;

        let attributes = HashMap::from([
            ("application", "winux-mail"),
            ("account_id", account_id),
        ]);

        let items = collection.search_items(attributes).await?;

        if let Some(item) = items.first() {
            let secret = item.get_secret().await?;
            Ok(String::from_utf8(secret)?)
        } else {
            Err(anyhow!("Password not found"))
        }
    }

    /// Validate account credentials
    pub async fn validate_account(&self, account: &Account) -> Result<bool> {
        use crate::backend::imap::ImapClient;

        let password = match &account.auth {
            AuthMethod::Password { password } => password.clone(),
            AuthMethod::AppPassword { password } => password.clone(),
            AuthMethod::OAuth2 { access_token, .. } => access_token.clone(),
        };

        let client = ImapClient::new(
            &account.settings.imap_server,
            account.settings.imap_port,
            account.settings.imap_tls,
        );

        client.connect(&account.email, &password).await?;

        Ok(true)
    }
}

/// OAuth2 helper
pub struct OAuth2Helper {
    client_id: String,
    client_secret: Option<String>,
    auth_url: String,
    token_url: String,
    redirect_uri: String,
}

impl OAuth2Helper {
    pub fn new(provider: AccountProvider) -> Option<Self> {
        let settings = provider.get_settings("")?;

        if !settings.oauth2_supported {
            return None;
        }

        Some(Self {
            client_id: Self::get_client_id(provider),
            client_secret: Self::get_client_secret(provider),
            auth_url: settings.oauth2_auth_url?,
            token_url: settings.oauth2_token_url?,
            redirect_uri: "http://localhost:8080/oauth/callback".to_string(),
        })
    }

    fn get_client_id(provider: AccountProvider) -> String {
        // In a real application, these would be stored securely or configured
        match provider {
            AccountProvider::Gmail => std::env::var("GMAIL_CLIENT_ID").unwrap_or_default(),
            AccountProvider::Outlook => std::env::var("OUTLOOK_CLIENT_ID").unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn get_client_secret(provider: AccountProvider) -> Option<String> {
        match provider {
            AccountProvider::Gmail => std::env::var("GMAIL_CLIENT_SECRET").ok(),
            AccountProvider::Outlook => std::env::var("OUTLOOK_CLIENT_SECRET").ok(),
            _ => None,
        }
    }

    /// Get the authorization URL for the user to visit
    pub fn get_auth_url(&self, state: &str) -> String {
        let scopes = "https://mail.google.com/ email profile";

        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
            self.auth_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(scopes),
            urlencoding::encode(state)
        )
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> Result<OAuth2Tokens> {
        let client = reqwest::Client::new();

        let mut params = HashMap::new();
        params.insert("code", code);
        params.insert("client_id", &self.client_id);
        params.insert("redirect_uri", &self.redirect_uri);
        params.insert("grant_type", "authorization_code");

        if let Some(secret) = &self.client_secret {
            params.insert("client_secret", secret.as_str());
        }

        let response = client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await?;

        let tokens: OAuth2TokenResponse = response.json().await?;

        Ok(OAuth2Tokens {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OAuth2Tokens> {
        let client = reqwest::Client::new();

        let mut params = HashMap::new();
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", &self.client_id);
        params.insert("grant_type", "refresh_token");

        if let Some(secret) = &self.client_secret {
            params.insert("client_secret", secret.as_str());
        }

        let response = client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await?;

        let tokens: OAuth2TokenResponse = response.json().await?;

        Ok(OAuth2Tokens {
            access_token: tokens.access_token,
            refresh_token: Some(refresh_token.to_string()),
            expires_in: tokens.expires_in,
        })
    }
}

#[derive(Debug, Deserialize)]
struct OAuth2TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
}

#[derive(Debug, Clone)]
pub struct OAuth2Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
}

// URL encoding helper
mod urlencoding {
    pub fn encode(input: &str) -> String {
        let mut output = String::with_capacity(input.len() * 3);
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    output.push(byte as char);
                }
                _ => {
                    output.push('%');
                    output.push_str(&format!("{:02X}", byte));
                }
            }
        }
        output
    }
}
