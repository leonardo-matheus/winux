//! Nextcloud provider implementation
//!
//! Implements WebDAV protocol for Nextcloud servers

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::path::Path;

use super::{
    CloudFile, CloudProvider, FileVersion, ListOptions, ProgressCallback,
    SearchOptions, SharedLink, StorageQuota,
};

/// Nextcloud provider (WebDAV-based)
pub struct NextcloudProvider {
    client: Client,
    base_url: String,
    username: String,
    password: Option<String>,
}

impl NextcloudProvider {
    /// Create a new Nextcloud provider
    pub fn new(server_url: &str, username: &str) -> Self {
        // Ensure URL ends with /remote.php/dav/files/
        let base_url = if server_url.ends_with('/') {
            format!("{}remote.php/dav/files/{}", server_url, username)
        } else {
            format!("{}/remote.php/dav/files/{}", server_url, username)
        };

        Self {
            client: Client::new(),
            base_url,
            username: username.to_string(),
            password: None,
        }
    }

    /// Create with credentials
    pub fn with_credentials(server_url: &str, username: &str, password: &str) -> Self {
        let mut provider = Self::new(server_url, username);
        provider.password = Some(password.to_string());
        provider
    }

    /// Parse WebDAV PROPFIND response
    fn parse_propfind_response(&self, xml: &str) -> Result<Vec<CloudFile>> {
        // Simple XML parsing for WebDAV responses
        let mut files = Vec::new();

        // Parse multistatus response
        for response in xml.split("<d:response>").skip(1) {
            if let Some(href_start) = response.find("<d:href>") {
                if let Some(href_end) = response.find("</d:href>") {
                    let href = &response[href_start + 8..href_end];
                    let path = urlencoding::decode(href).unwrap_or_default().to_string();

                    // Get properties
                    let is_folder = response.contains("<d:resourcetype><d:collection");
                    let name = Path::new(&path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Parse content length
                    let size = self.extract_property(response, "d:getcontentlength")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    // Parse last modified
                    let modified_at = self.extract_property(response, "d:getlastmodified")
                        .and_then(|s| DateTime::parse_from_rfc2822(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now);

                    // Parse content type
                    let mime_type = self.extract_property(response, "d:getcontenttype")
                        .unwrap_or_else(|| {
                            if is_folder {
                                "application/x-directory".to_string()
                            } else {
                                "application/octet-stream".to_string()
                            }
                        });

                    // Parse etag
                    let etag = self.extract_property(response, "d:getetag");

                    // Parse file ID (Nextcloud specific)
                    let id = self.extract_property(response, "oc:fileid")
                        .unwrap_or_else(|| path.clone());

                    if !name.is_empty() {
                        files.push(CloudFile {
                            id,
                            name,
                            path: path.clone(),
                            parent_id: Path::new(&path).parent().map(|p| p.to_string_lossy().to_string()),
                            is_folder,
                            size,
                            mime_type,
                            created_at: Utc::now(), // WebDAV doesn't provide creation time
                            modified_at,
                            hash: etag,
                            shared: false,
                            download_url: Some(format!("{}{}", self.base_url, path)),
                            thumbnail_url: None,
                            version: None,
                        });
                    }
                }
            }
        }

        Ok(files)
    }

    /// Extract a property from WebDAV XML
    fn extract_property(&self, xml: &str, prop: &str) -> Option<String> {
        let start_tag = format!("<{}>", prop);
        let end_tag = format!("</{}>", prop);

        if let Some(start) = xml.find(&start_tag) {
            if let Some(end) = xml[start..].find(&end_tag) {
                let value = &xml[start + start_tag.len()..start + end];
                return Some(value.to_string());
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl CloudProvider for NextcloudProvider {
    fn name(&self) -> &str {
        "Nextcloud"
    }

    fn icon(&self) -> &str {
        "network-server-symbolic"
    }

    fn is_authenticated(&self) -> bool {
        self.password.is_some()
    }

    fn get_auth_url(&self) -> Option<String> {
        None // Nextcloud uses basic auth, not OAuth2
    }

    async fn authenticate_oauth(&mut self, _code: &str) -> Result<()> {
        Err(anyhow!("Nextcloud uses basic authentication, not OAuth2"))
    }

    async fn authenticate_basic(&mut self, username: &str, password: &str) -> Result<()> {
        self.username = username.to_string();
        self.password = Some(password.to_string());

        // Test connection
        let response = self.client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &self.base_url)
            .basic_auth(&self.username, self.password.as_ref())
            .header("Depth", "0")
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 207 {
            Ok(())
        } else {
            self.password = None;
            Err(anyhow!("Authentication failed: {}", response.status()))
        }
    }

    async fn authenticate_api_key(&mut self, _access_key: &str, _secret_key: &str) -> Result<()> {
        Err(anyhow!("Nextcloud does not support API key authentication"))
    }

    async fn refresh_auth(&mut self) -> Result<()> {
        // Basic auth doesn't need refresh
        Ok(())
    }

    async fn get_quota(&self) -> Result<StorageQuota> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let propfind_body = r#"<?xml version="1.0"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:quota-available-bytes/>
    <d:quota-used-bytes/>
  </d:prop>
</d:propfind>"#;

        let response = self.client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &self.base_url)
            .basic_auth(&self.username, Some(password))
            .header("Depth", "0")
            .header("Content-Type", "application/xml")
            .body(propfind_body)
            .send()
            .await?;

        let xml = response.text().await?;

        let used = self.extract_property(&xml, "d:quota-used-bytes")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let available = self.extract_property(&xml, "d:quota-available-bytes")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok(StorageQuota {
            total: used + available,
            used,
            available,
            trash: None,
        })
    }

    async fn list_files(&self, folder_id: Option<&str>, _options: ListOptions) -> Result<Vec<CloudFile>> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = match folder_id {
            Some(path) => format!("{}/{}", self.base_url, path.trim_start_matches('/')),
            None => self.base_url.clone(),
        };

        let propfind_body = r#"<?xml version="1.0"?>
<d:propfind xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns">
  <d:prop>
    <d:resourcetype/>
    <d:getcontentlength/>
    <d:getlastmodified/>
    <d:getcontenttype/>
    <d:getetag/>
    <oc:fileid/>
  </d:prop>
</d:propfind>"#;

        let response = self.client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(propfind_body)
            .send()
            .await?;

        let xml = response.text().await?;
        let mut files = self.parse_propfind_response(&xml)?;

        // Remove the first entry (the folder itself)
        if !files.is_empty() {
            files.remove(0);
        }

        Ok(files)
    }

    async fn get_file(&self, file_id: &str) -> Result<CloudFile> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = format!("{}/{}", self.base_url, file_id.trim_start_matches('/'));

        let propfind_body = r#"<?xml version="1.0"?>
<d:propfind xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns">
  <d:prop>
    <d:resourcetype/>
    <d:getcontentlength/>
    <d:getlastmodified/>
    <d:getcontenttype/>
    <d:getetag/>
    <oc:fileid/>
  </d:prop>
</d:propfind>"#;

        let response = self.client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(password))
            .header("Depth", "0")
            .header("Content-Type", "application/xml")
            .body(propfind_body)
            .send()
            .await?;

        let xml = response.text().await?;
        let files = self.parse_propfind_response(&xml)?;

        files.into_iter().next().ok_or_else(|| anyhow!("File not found"))
    }

    async fn search(&self, options: SearchOptions) -> Result<Vec<CloudFile>> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // Nextcloud search API
        let search_url = self.base_url.replace("/remote.php/dav/files/", "/remote.php/dav/");

        let search_body = format!(r#"<?xml version="1.0"?>
<d:searchrequest xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns">
  <d:basicsearch>
    <d:select>
      <d:prop>
        <d:resourcetype/>
        <d:getcontentlength/>
        <d:getlastmodified/>
        <d:getcontenttype/>
        <oc:fileid/>
      </d:prop>
    </d:select>
    <d:from>
      <d:scope>
        <d:href>/files/{}</d:href>
        <d:depth>infinity</d:depth>
      </d:scope>
    </d:from>
    <d:where>
      <d:like>
        <d:prop><d:displayname/></d:prop>
        <d:literal>%{}%</d:literal>
      </d:like>
    </d:where>
  </d:basicsearch>
</d:searchrequest>"#, self.username, options.query);

        let response = self.client
            .request(reqwest::Method::from_bytes(b"SEARCH").unwrap(), &search_url)
            .basic_auth(&self.username, Some(password))
            .header("Content-Type", "application/xml")
            .body(search_body)
            .send()
            .await?;

        let xml = response.text().await?;
        self.parse_propfind_response(&xml)
    }

    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let path = match parent_id {
            Some(p) => format!("{}/{}", p.trim_end_matches('/'), name),
            None => name.to_string(),
        };

        let url = format!("{}/{}", self.base_url, path);

        let response = self.client
            .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url)
            .basic_auth(&self.username, Some(password))
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 201 {
            self.get_file(&path).await
        } else {
            Err(anyhow!("Failed to create folder: {}", response.status()))
        }
    }

    async fn upload_file(
        &self,
        local_path: &Path,
        parent_id: Option<&str>,
        name: Option<&str>,
        _progress: Option<ProgressCallback>,
    ) -> Result<CloudFile> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let file_name = name.map(|s| s.to_string())
            .or_else(|| local_path.file_name().map(|n| n.to_string_lossy().to_string()))
            .ok_or_else(|| anyhow!("Could not determine file name"))?;

        let path = match parent_id {
            Some(p) => format!("{}/{}", p.trim_end_matches('/'), file_name),
            None => file_name.clone(),
        };

        let content = std::fs::read(local_path)?;
        let url = format!("{}/{}", self.base_url, path);

        let response = self.client
            .put(&url)
            .basic_auth(&self.username, Some(password))
            .body(content)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 201 || response.status().as_u16() == 204 {
            self.get_file(&path).await
        } else {
            Err(anyhow!("Failed to upload file: {}", response.status()))
        }
    }

    async fn download_file(
        &self,
        file_id: &str,
        local_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = format!("{}/{}", self.base_url, file_id.trim_start_matches('/'));

        let response = self.client
            .get(&url)
            .basic_auth(&self.username, Some(password))
            .send()
            .await?;

        if response.status().is_success() {
            let content = response.bytes().await?;
            std::fs::write(local_path, content)?;
            Ok(())
        } else {
            Err(anyhow!("Failed to download file: {}", response.status()))
        }
    }

    async fn move_file(&self, file_id: &str, new_parent_id: &str) -> Result<CloudFile> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let file = self.get_file(file_id).await?;
        let new_path = format!("{}/{}", new_parent_id.trim_end_matches('/'), file.name);

        let source_url = format!("{}/{}", self.base_url, file_id.trim_start_matches('/'));
        let dest_url = format!("{}/{}", self.base_url, new_path);

        let response = self.client
            .request(reqwest::Method::from_bytes(b"MOVE").unwrap(), &source_url)
            .basic_auth(&self.username, Some(password))
            .header("Destination", &dest_url)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 201 || response.status().as_u16() == 204 {
            self.get_file(&new_path).await
        } else {
            Err(anyhow!("Failed to move file: {}", response.status()))
        }
    }

    async fn rename_file(&self, file_id: &str, new_name: &str) -> Result<CloudFile> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let file = self.get_file(file_id).await?;
        let parent = file.parent_id.unwrap_or_default();
        let new_path = format!("{}/{}", parent.trim_end_matches('/'), new_name);

        let source_url = format!("{}/{}", self.base_url, file_id.trim_start_matches('/'));
        let dest_url = format!("{}/{}", self.base_url, new_path);

        let response = self.client
            .request(reqwest::Method::from_bytes(b"MOVE").unwrap(), &source_url)
            .basic_auth(&self.username, Some(password))
            .header("Destination", &dest_url)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 201 || response.status().as_u16() == 204 {
            self.get_file(&new_path).await
        } else {
            Err(anyhow!("Failed to rename file: {}", response.status()))
        }
    }

    async fn copy_file(&self, file_id: &str, dest_parent_id: &str, new_name: Option<&str>) -> Result<CloudFile> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let file = self.get_file(file_id).await?;
        let name = new_name.unwrap_or(&file.name);
        let new_path = format!("{}/{}", dest_parent_id.trim_end_matches('/'), name);

        let source_url = format!("{}/{}", self.base_url, file_id.trim_start_matches('/'));
        let dest_url = format!("{}/{}", self.base_url, new_path);

        let response = self.client
            .request(reqwest::Method::from_bytes(b"COPY").unwrap(), &source_url)
            .basic_auth(&self.username, Some(password))
            .header("Destination", &dest_url)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 201 || response.status().as_u16() == 204 {
            self.get_file(&new_path).await
        } else {
            Err(anyhow!("Failed to copy file: {}", response.status()))
        }
    }

    async fn delete_file(&self, file_id: &str) -> Result<()> {
        let password = self.password.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = format!("{}/{}", self.base_url, file_id.trim_start_matches('/'));

        let response = self.client
            .delete(&url)
            .basic_auth(&self.username, Some(password))
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(anyhow!("Failed to delete file: {}", response.status()))
        }
    }

    async fn permanent_delete(&self, file_id: &str) -> Result<()> {
        self.delete_file(file_id).await
    }

    async fn restore_file(&self, _file_id: &str) -> Result<CloudFile> {
        Err(anyhow!("Nextcloud restore requires Trashbin API"))
    }

    async fn empty_trash(&self) -> Result<()> {
        Err(anyhow!("Nextcloud empty trash requires Trashbin API"))
    }

    async fn create_shared_link(
        &self,
        _file_id: &str,
        _expires_at: Option<DateTime<Utc>>,
        _password: Option<&str>,
    ) -> Result<SharedLink> {
        Err(anyhow!("Nextcloud sharing requires OCS API"))
    }

    async fn get_shared_link(&self, _file_id: &str) -> Result<Option<SharedLink>> {
        Ok(None)
    }

    async fn revoke_shared_link(&self, _file_id: &str) -> Result<()> {
        Err(anyhow!("Nextcloud sharing requires OCS API"))
    }

    async fn get_versions(&self, _file_id: &str) -> Result<Vec<FileVersion>> {
        Err(anyhow!("Nextcloud versioning requires Versions API"))
    }

    async fn restore_version(&self, _file_id: &str, _version_id: &str) -> Result<CloudFile> {
        Err(anyhow!("Nextcloud versioning requires Versions API"))
    }

    async fn get_changes(&self, _cursor: Option<&str>) -> Result<(Vec<CloudFile>, Option<String>)> {
        // WebDAV doesn't have native change tracking
        // Full sync is required
        Ok((vec![], None))
    }
}
