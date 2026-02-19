// CardDAV Synchronization
// Supports Google Contacts, Nextcloud, and other CardDAV servers

use crate::data::{contact::Contact, vcard};
use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CardDAV account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDavAccount {
    pub id: String,
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: Option<String>,
    pub sync_token: Option<String>,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

impl CardDavAccount {
    pub fn new(name: &str, url: &str, username: &str, password: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            url: url.to_string(),
            username: username.to_string(),
            password,
            sync_token: None,
            last_sync: None,
        }
    }
}

/// CardDAV client for synchronization
pub struct CardDavClient {
    client: reqwest::Client,
    account: CardDavAccount,
}

impl CardDavClient {
    pub fn new(account: CardDavAccount) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, account }
    }

    fn get_auth_header(&self) -> Result<HeaderValue> {
        let password = self.account.password.as_deref().unwrap_or("");
        let credentials = format!("{}:{}", self.account.username, password);
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            credentials.as_bytes(),
        );
        HeaderValue::from_str(&format!("Basic {}", encoded))
            .context("Failed to create auth header")
    }

    /// Discover the addressbook URL from the principal URL
    pub async fn discover_addressbook(&self) -> Result<String> {
        // First, find the current user principal
        let propfind_body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:current-user-principal/>
  </d:prop>
</d:propfind>"#;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
        headers.insert("Depth", HeaderValue::from_static("0"));
        headers.insert(AUTHORIZATION, self.get_auth_header()?);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &self.account.url)
            .headers(headers)
            .body(propfind_body)
            .send()
            .await?;

        let body = response.text().await?;

        // Parse principal URL from response
        let principal_url = self.extract_principal_url(&body)?;

        // Now find the addressbook home
        let propfind_body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:prop>
    <card:addressbook-home-set/>
  </d:prop>
</d:propfind>"#;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
        headers.insert("Depth", HeaderValue::from_static("0"));
        headers.insert(AUTHORIZATION, self.get_auth_header()?);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &principal_url)
            .headers(headers)
            .body(propfind_body)
            .send()
            .await?;

        let body = response.text().await?;
        self.extract_addressbook_url(&body)
    }

    /// Fetch all contacts from the server
    pub async fn fetch_all_contacts(&self, addressbook_url: &str) -> Result<Vec<(String, String, Contact)>> {
        let report_body = r#"<?xml version="1.0" encoding="utf-8"?>
<card:addressbook-query xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:prop>
    <d:getetag/>
    <card:address-data/>
  </d:prop>
</card:addressbook-query>"#;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
        headers.insert("Depth", HeaderValue::from_static("1"));
        headers.insert(AUTHORIZATION, self.get_auth_header()?);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"REPORT").unwrap(), addressbook_url)
            .headers(headers)
            .body(report_body)
            .send()
            .await?;

        let body = response.text().await?;
        self.parse_addressbook_response(&body)
    }

    /// Fetch contacts that changed since last sync
    pub async fn fetch_changes(&self, addressbook_url: &str, sync_token: &str) -> Result<SyncResult> {
        let report_body = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<d:sync-collection xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:sync-token>{}</d:sync-token>
  <d:sync-level>1</d:sync-level>
  <d:prop>
    <d:getetag/>
    <card:address-data/>
  </d:prop>
</d:sync-collection>"#,
            sync_token
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
        headers.insert(AUTHORIZATION, self.get_auth_header()?);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"REPORT").unwrap(), addressbook_url)
            .headers(headers)
            .body(report_body)
            .send()
            .await?;

        let body = response.text().await?;
        self.parse_sync_response(&body)
    }

    /// Upload a new or updated contact
    pub async fn put_contact(&self, addressbook_url: &str, contact: &Contact) -> Result<String> {
        let vcard_data = vcard::generate_vcard(contact);
        let contact_url = format!("{}/{}.vcf", addressbook_url.trim_end_matches('/'), contact.id);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/vcard; charset=utf-8"));
        headers.insert(AUTHORIZATION, self.get_auth_header()?);

        // If we have an etag, use If-Match for optimistic locking
        if let Some(etag) = &contact.etag {
            headers.insert("If-Match", HeaderValue::from_str(etag)?);
        }

        let response = self
            .client
            .put(&contact_url)
            .headers(headers)
            .body(vcard_data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to upload contact: {}", response.status()));
        }

        // Get the new etag from response
        let new_etag = response
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        Ok(new_etag)
    }

    /// Delete a contact from the server
    pub async fn delete_contact(&self, contact_url: &str, etag: Option<&str>) -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.get_auth_header()?);

        if let Some(etag) = etag {
            headers.insert("If-Match", HeaderValue::from_str(etag)?);
        }

        let response = self
            .client
            .delete(contact_url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow!("Failed to delete contact: {}", response.status()));
        }

        Ok(())
    }

    // Helper methods for XML parsing (simplified - a real implementation would use proper XML parser)
    fn extract_principal_url(&self, xml: &str) -> Result<String> {
        // Simple regex extraction - in production use proper XML parser
        let re = regex::Regex::new(r"<d:href>([^<]+)</d:href>").unwrap();
        for cap in re.captures_iter(xml) {
            let href = &cap[1];
            if href.contains("principal") || href.contains("user") {
                return Ok(self.resolve_url(href));
            }
        }
        Err(anyhow!("Could not find principal URL"))
    }

    fn extract_addressbook_url(&self, xml: &str) -> Result<String> {
        let re = regex::Regex::new(r"<d:href>([^<]+)</d:href>").unwrap();
        for cap in re.captures_iter(xml) {
            let href = &cap[1];
            if href.contains("addressbook") || href.contains("contacts") || href.contains("carddav") {
                return Ok(self.resolve_url(href));
            }
        }
        Err(anyhow!("Could not find addressbook URL"))
    }

    fn resolve_url(&self, href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else {
            let base = url::Url::parse(&self.account.url).unwrap();
            base.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string())
        }
    }

    fn parse_addressbook_response(&self, xml: &str) -> Result<Vec<(String, String, Contact)>> {
        let mut contacts = Vec::new();

        // Extract response elements (simplified parsing)
        let href_re = regex::Regex::new(r"<d:href>([^<]+)</d:href>").unwrap();
        let etag_re = regex::Regex::new(r"<d:getetag>([^<]+)</d:getetag>").unwrap();
        let vcard_re = regex::Regex::new(r"<card:address-data[^>]*>([\s\S]*?)</card:address-data>").unwrap();

        // Split by response elements
        let response_re = regex::Regex::new(r"<d:response>([\s\S]*?)</d:response>").unwrap();

        for response_cap in response_re.captures_iter(xml) {
            let response_content = &response_cap[1];

            let href = href_re.captures(response_content)
                .map(|c| c[1].to_string())
                .unwrap_or_default();

            let etag = etag_re.captures(response_content)
                .map(|c| c[1].trim_matches('"').to_string())
                .unwrap_or_default();

            if let Some(vcard_cap) = vcard_re.captures(response_content) {
                let vcard_data = html_escape::decode_html_entities(&vcard_cap[1]).to_string();
                if let Ok(mut parsed) = vcard::parse_vcard(&vcard_data) {
                    if let Some(mut contact) = parsed.pop() {
                        contact.etag = Some(etag.clone());
                        contact.carddav_href = Some(self.resolve_url(&href));
                        contacts.push((href, etag, contact));
                    }
                }
            }
        }

        Ok(contacts)
    }

    fn parse_sync_response(&self, xml: &str) -> Result<SyncResult> {
        let mut result = SyncResult {
            new_sync_token: None,
            changed: Vec::new(),
            deleted: Vec::new(),
        };

        // Extract new sync token
        let token_re = regex::Regex::new(r"<d:sync-token>([^<]+)</d:sync-token>").unwrap();
        if let Some(cap) = token_re.captures(xml) {
            result.new_sync_token = Some(cap[1].to_string());
        }

        // Parse changed/deleted items
        let response_re = regex::Regex::new(r"<d:response>([\s\S]*?)</d:response>").unwrap();
        let href_re = regex::Regex::new(r"<d:href>([^<]+)</d:href>").unwrap();
        let status_re = regex::Regex::new(r"<d:status>([^<]+)</d:status>").unwrap();

        for response_cap in response_re.captures_iter(xml) {
            let response_content = &response_cap[1];

            let href = href_re.captures(response_content)
                .map(|c| c[1].to_string())
                .unwrap_or_default();

            let status = status_re.captures(response_content)
                .map(|c| c[1].to_string())
                .unwrap_or_default();

            if status.contains("404") {
                result.deleted.push(href);
            } else {
                // This is a changed item - fetch the full vCard
                let contacts = self.parse_addressbook_response(response_content).unwrap_or_default();
                for (h, e, c) in contacts {
                    result.changed.push((h, e, c));
                }
            }
        }

        Ok(result)
    }
}

/// Result of a sync operation
#[derive(Debug)]
pub struct SyncResult {
    pub new_sync_token: Option<String>,
    pub changed: Vec<(String, String, Contact)>,
    pub deleted: Vec<String>,
}

/// Perform a full sync with a CardDAV server
pub async fn sync_contacts(
    account: &mut CardDavAccount,
    local_contacts: &mut Vec<Contact>,
) -> Result<SyncStats> {
    let client = CardDavClient::new(account.clone());

    // Discover addressbook URL
    let addressbook_url = client.discover_addressbook().await?;

    let mut stats = SyncStats::default();

    if let Some(sync_token) = &account.sync_token {
        // Incremental sync
        let result = client.fetch_changes(&addressbook_url, sync_token).await?;

        for (href, etag, remote_contact) in result.changed {
            if let Some(local) = local_contacts.iter_mut().find(|c| c.carddav_href.as_ref() == Some(&href)) {
                // Update existing contact
                *local = remote_contact;
                stats.updated += 1;
            } else {
                // New contact
                local_contacts.push(remote_contact);
                stats.added += 1;
            }
        }

        for href in result.deleted {
            if let Some(pos) = local_contacts.iter().position(|c| c.carddav_href.as_ref() == Some(&href)) {
                local_contacts.remove(pos);
                stats.deleted += 1;
            }
        }

        if let Some(token) = result.new_sync_token {
            account.sync_token = Some(token);
        }
    } else {
        // Full sync
        let remote_contacts = client.fetch_all_contacts(&addressbook_url).await?;

        // Create a map of local contacts by href
        let local_map: HashMap<String, usize> = local_contacts
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.carddav_href.clone().map(|h| (h, i)))
            .collect();

        for (href, etag, remote_contact) in remote_contacts {
            if let Some(&idx) = local_map.get(&href) {
                // Update if etag differs
                if local_contacts[idx].etag.as_ref() != Some(&etag) {
                    local_contacts[idx] = remote_contact;
                    stats.updated += 1;
                }
            } else {
                local_contacts.push(remote_contact);
                stats.added += 1;
            }
        }
    }

    account.last_sync = Some(chrono::Utc::now());
    Ok(stats)
}

/// Statistics from a sync operation
#[derive(Debug, Default)]
pub struct SyncStats {
    pub added: usize,
    pub updated: usize,
    pub deleted: usize,
}

impl std::fmt::Display for SyncStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sync complete: {} added, {} updated, {} deleted",
            self.added, self.updated, self.deleted
        )
    }
}

// Minimal HTML entity decoder for vCard data
mod html_escape {
    pub fn decode_html_entities(s: &str) -> String {
        s.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
    }
}
