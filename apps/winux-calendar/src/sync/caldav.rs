//! CalDAV synchronization client

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

use crate::data::{CalendarInfo, CalendarSource, Event, Task};

/// CalDAV account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalDAVAccount {
    /// Account ID
    pub id: String,

    /// Account name (for display)
    pub name: String,

    /// Server URL (e.g., https://caldav.example.com)
    pub server_url: String,

    /// Username
    pub username: String,

    /// Password (should be stored securely)
    pub password: String,

    /// Is this account enabled
    pub enabled: bool,

    /// Last sync timestamp
    pub last_sync: Option<NaiveDateTime>,

    /// Sync interval in minutes
    pub sync_interval: u32,
}

impl CalDAVAccount {
    /// Create a new CalDAV account
    pub fn new(name: &str, server_url: &str, username: &str, password: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            server_url: server_url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            enabled: true,
            last_sync: None,
            sync_interval: 15,
        }
    }

    /// Create account for Google Calendar
    pub fn google(email: &str, app_password: &str) -> Self {
        Self::new(
            "Google Calendar",
            "https://apidata.googleusercontent.com/caldav/v2",
            email,
            app_password,
        )
    }

    /// Create account for Nextcloud
    pub fn nextcloud(server: &str, username: &str, password: &str) -> Self {
        let url = format!("{}/remote.php/dav", server.trim_end_matches('/'));
        Self::new("Nextcloud", &url, username, password)
    }

    /// Create account for iCloud
    pub fn icloud(apple_id: &str, app_password: &str) -> Self {
        Self::new(
            "iCloud",
            "https://caldav.icloud.com",
            apple_id,
            app_password,
        )
    }
}

/// CalDAV client for synchronization
pub struct CalDAVClient {
    account: CalDAVAccount,
    client: reqwest::Client,
}

impl CalDAVClient {
    /// Create a new CalDAV client
    pub fn new(account: CalDAVAccount) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { account, client }
    }

    /// Discover calendars on the server
    pub async fn discover_calendars(&self) -> Result<Vec<CalendarInfo>> {
        let calendars = Vec::new();

        // PROPFIND request to discover calendars
        let url = format!("{}/calendars/{}/",
            self.account.server_url,
            self.account.username);

        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
    <cs:getctag/>
    <c:supported-calendar-component-set/>
  </d:prop>
</d:propfind>"#;

        let response = self.client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.account.username, Some(&self.account.password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(body)
            .send()
            .await
            .context("Failed to discover calendars")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Server returned error: {}", response.status()));
        }

        // Parse XML response (simplified - in production use proper XML parser)
        let _body = response.text().await?;

        // TODO: Parse the XML response to extract calendar information
        // For now, return empty list - would need proper XML parsing

        Ok(calendars)
    }

    /// Sync events from a calendar
    pub async fn sync_events(&self, calendar: &CalendarInfo) -> Result<Vec<Event>> {
        let events = Vec::new();

        let url = calendar.remote_url.as_ref()
            .context("Calendar has no remote URL")?;

        // REPORT request to get events
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VEVENT"/>
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#;

        let response = self.client
            .request(reqwest::Method::from_bytes(b"REPORT").unwrap(), url)
            .basic_auth(&self.account.username, Some(&self.account.password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(body)
            .send()
            .await
            .context("Failed to sync events")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Server returned error: {}", response.status()));
        }

        let _body = response.text().await?;

        // TODO: Parse the XML/iCal response to extract events
        // Would need to extract calendar-data and parse each VEVENT

        Ok(events)
    }

    /// Upload an event to the server
    pub async fn upload_event(&self, calendar: &CalendarInfo, event: &Event) -> Result<()> {
        let base_url = calendar.remote_url.as_ref()
            .context("Calendar has no remote URL")?;

        let url = format!("{}/{}.ics", base_url, event.id);

        // Convert event to iCal format
        let ical_content = self.event_to_ical(event);

        let response = self.client
            .put(&url)
            .basic_auth(&self.account.username, Some(&self.account.password))
            .header("Content-Type", "text/calendar; charset=utf-8")
            .body(ical_content)
            .send()
            .await
            .context("Failed to upload event")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to upload event: {}", response.status()));
        }

        Ok(())
    }

    /// Delete an event from the server
    pub async fn delete_event(&self, calendar: &CalendarInfo, event_id: &str) -> Result<()> {
        let base_url = calendar.remote_url.as_ref()
            .context("Calendar has no remote URL")?;

        let url = format!("{}/{}.ics", base_url, event_id);

        let response = self.client
            .delete(&url)
            .basic_auth(&self.account.username, Some(&self.account.password))
            .send()
            .await
            .context("Failed to delete event")?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow::anyhow!("Failed to delete event: {}", response.status()));
        }

        Ok(())
    }

    /// Sync tasks (VTODO)
    pub async fn sync_tasks(&self, calendar: &CalendarInfo) -> Result<Vec<Task>> {
        let tasks = Vec::new();

        let url = calendar.remote_url.as_ref()
            .context("Calendar has no remote URL")?;

        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VTODO"/>
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#;

        let response = self.client
            .request(reqwest::Method::from_bytes(b"REPORT").unwrap(), url)
            .basic_auth(&self.account.username, Some(&self.account.password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(body)
            .send()
            .await
            .context("Failed to sync tasks")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Server returned error: {}", response.status()));
        }

        let _body = response.text().await?;

        // TODO: Parse tasks from response

        Ok(tasks)
    }

    /// Convert event to iCal format
    fn event_to_ical(&self, event: &Event) -> String {
        use crate::data::ICalExporter;

        let calendar = CalendarInfo::new("Export", "#3584e4");
        ICalExporter::export(&[event.clone()], &calendar)
    }

    /// Test connection to server
    pub async fn test_connection(&self) -> Result<bool> {
        let url = &self.account.server_url;

        let response = self.client
            .request(reqwest::Method::OPTIONS, url)
            .basic_auth(&self.account.username, Some(&self.account.password))
            .send()
            .await
            .context("Failed to connect to server")?;

        Ok(response.status().is_success())
    }
}

/// Sync status
#[derive(Debug, Clone)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Success(NaiveDateTime),
    Error(String),
}

/// Sync manager for handling multiple accounts
pub struct SyncManager {
    accounts: Vec<CalDAVAccount>,
    status: std::collections::HashMap<String, SyncStatus>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            status: std::collections::HashMap::new(),
        }
    }

    pub fn add_account(&mut self, account: CalDAVAccount) {
        self.status.insert(account.id.clone(), SyncStatus::Idle);
        self.accounts.push(account);
    }

    pub fn remove_account(&mut self, id: &str) {
        self.accounts.retain(|a| a.id != id);
        self.status.remove(id);
    }

    pub fn get_status(&self, account_id: &str) -> Option<&SyncStatus> {
        self.status.get(account_id)
    }

    pub async fn sync_account(&mut self, account_id: &str) -> Result<()> {
        let account = self.accounts.iter()
            .find(|a| a.id == account_id)
            .context("Account not found")?
            .clone();

        self.status.insert(account_id.to_string(), SyncStatus::Syncing);

        let client = CalDAVClient::new(account);

        match client.discover_calendars().await {
            Ok(_calendars) => {
                let now = chrono::Local::now().naive_local();
                self.status.insert(account_id.to_string(), SyncStatus::Success(now));
                Ok(())
            }
            Err(e) => {
                self.status.insert(account_id.to_string(), SyncStatus::Error(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn sync_all(&mut self) -> Result<()> {
        let account_ids: Vec<_> = self.accounts.iter()
            .filter(|a| a.enabled)
            .map(|a| a.id.clone())
            .collect();

        for id in account_ids {
            let _ = self.sync_account(&id).await;
        }

        Ok(())
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}
