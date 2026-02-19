//! Google Drive provider implementation
//!
//! Implements OAuth2 authentication and Google Drive API v3

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use url::Url;

use super::{
    CloudFile, CloudProvider, FileVersion, ListOptions, OAuthTokens, ProgressCallback,
    SearchOptions, SharedLink, StorageQuota,
};

/// Google Drive OAuth2 configuration
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_API_BASE: &str = "https://www.googleapis.com/drive/v3";
const GOOGLE_UPLOAD_URL: &str = "https://www.googleapis.com/upload/drive/v3";

/// Required OAuth2 scopes
const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/drive",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];

/// Google Drive provider
pub struct GoogleDriveProvider {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    tokens: Option<OAuthTokens>,
}

impl GoogleDriveProvider {
    /// Create a new Google Drive provider
    pub fn new(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        Self {
            client: Client::new(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri: redirect_uri.to_string(),
            tokens: None,
        }
    }

    /// Create with existing tokens
    pub fn with_tokens(
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        tokens: OAuthTokens,
    ) -> Self {
        Self {
            client: Client::new(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri: redirect_uri.to_string(),
            tokens: Some(tokens),
        }
    }

    /// Get access token, refreshing if necessary
    async fn get_access_token(&mut self) -> Result<String> {
        let tokens = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // Check if token is expired
        if let Some(expires_at) = tokens.expires_at {
            if expires_at <= Utc::now() {
                self.refresh_auth().await?;
            }
        }

        Ok(self.tokens.as_ref().unwrap().access_token.clone())
    }

    /// Make authenticated API request
    async fn api_request<T: for<'de> Deserialize<'de>>(
        &mut self,
        method: reqwest::Method,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
    ) -> Result<T> {
        let token = self.get_access_token().await?;
        let url = format!("{}{}", GOOGLE_API_BASE, endpoint);

        let mut request = self.client.request(method, &url)
            .bearer_auth(&token);

        if let Some(params) = query {
            request = request.query(params);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error: {}", error_text));
        }

        let result = response.json().await?;
        Ok(result)
    }

    /// Convert Google Drive file to CloudFile
    fn convert_file(file: &GoogleFile) -> CloudFile {
        CloudFile {
            id: file.id.clone(),
            name: file.name.clone(),
            path: file.name.clone(), // Full path needs to be reconstructed
            parent_id: file.parents.first().cloned(),
            is_folder: file.mime_type == "application/vnd.google-apps.folder",
            size: file.size.parse().unwrap_or(0),
            mime_type: file.mime_type.clone(),
            created_at: file.created_time.unwrap_or_else(Utc::now),
            modified_at: file.modified_time.unwrap_or_else(Utc::now),
            hash: file.md5_checksum.clone(),
            shared: file.shared.unwrap_or(false),
            download_url: file.web_content_link.clone(),
            thumbnail_url: file.thumbnail_link.clone(),
            version: file.version.clone(),
        }
    }
}

#[async_trait::async_trait]
impl CloudProvider for GoogleDriveProvider {
    fn name(&self) -> &str {
        "Google Drive"
    }

    fn icon(&self) -> &str {
        "google-drive"
    }

    fn is_authenticated(&self) -> bool {
        self.tokens.is_some()
    }

    fn get_auth_url(&self) -> Option<String> {
        let mut url = Url::parse(GOOGLE_AUTH_URL).ok()?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &SCOPES.join(" "))
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");

        Some(url.to_string())
    }

    async fn authenticate_oauth(&mut self, code: &str) -> Result<()> {
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            token_type: String,
            expires_in: u64,
        }

        let response: TokenResponse = self.client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", code),
                ("redirect_uri", self.redirect_uri.as_str()),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?
            .json()
            .await?;

        self.tokens = Some(OAuthTokens {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(response.expires_in as i64)),
            scopes: SCOPES.iter().map(|s| s.to_string()).collect(),
        });

        Ok(())
    }

    async fn authenticate_basic(&mut self, _username: &str, _password: &str) -> Result<()> {
        Err(anyhow!("Google Drive does not support basic authentication"))
    }

    async fn authenticate_api_key(&mut self, _access_key: &str, _secret_key: &str) -> Result<()> {
        Err(anyhow!("Google Drive does not support API key authentication"))
    }

    async fn refresh_auth(&mut self) -> Result<()> {
        let refresh_token = self.tokens
            .as_ref()
            .and_then(|t| t.refresh_token.clone())
            .ok_or_else(|| anyhow!("No refresh token available"))?;

        #[derive(Deserialize)]
        struct RefreshResponse {
            access_token: String,
            token_type: String,
            expires_in: u64,
        }

        let response: RefreshResponse = self.client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?
            .json()
            .await?;

        if let Some(tokens) = &mut self.tokens {
            tokens.access_token = response.access_token;
            tokens.expires_at = Some(Utc::now() + chrono::Duration::seconds(response.expires_in as i64));
        }

        Ok(())
    }

    async fn get_quota(&self) -> Result<StorageQuota> {
        // Clone self for mutable access (workaround for async trait)
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Deserialize)]
        struct AboutResponse {
            #[serde(rename = "storageQuota")]
            storage_quota: QuotaInfo,
        }

        #[derive(Deserialize)]
        struct QuotaInfo {
            limit: Option<String>,
            usage: String,
            #[serde(rename = "usageInDrive")]
            usage_in_drive: String,
            #[serde(rename = "usageInDriveTrash")]
            usage_in_drive_trash: Option<String>,
        }

        let response: AboutResponse = self.client
            .get(&format!("{}/about", GOOGLE_API_BASE))
            .bearer_auth(&token.access_token)
            .query(&[("fields", "storageQuota")])
            .send()
            .await?
            .json()
            .await?;

        let total = response.storage_quota.limit
            .and_then(|s| s.parse().ok())
            .unwrap_or(u64::MAX);
        let used: u64 = response.storage_quota.usage.parse().unwrap_or(0);
        let trash: Option<u64> = response.storage_quota.usage_in_drive_trash
            .and_then(|s| s.parse().ok());

        Ok(StorageQuota {
            total,
            used,
            available: total.saturating_sub(used),
            trash,
        })
    }

    async fn list_files(&self, folder_id: Option<&str>, options: ListOptions) -> Result<Vec<CloudFile>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let parent = folder_id.unwrap_or("root");
        let mut query = format!("'{}' in parents", parent);

        if !options.include_trashed {
            query.push_str(" and trashed = false");
        }

        if let Some(mime) = &options.mime_type_filter {
            query.push_str(&format!(" and mimeType = '{}'", mime));
        }

        let fields = "files(id,name,mimeType,size,createdTime,modifiedTime,md5Checksum,parents,shared,webContentLink,thumbnailLink,version)";

        let response: GoogleFileList = self.client
            .get(&format!("{}/files", GOOGLE_API_BASE))
            .bearer_auth(&token.access_token)
            .query(&[
                ("q", query.as_str()),
                ("fields", fields),
                ("pageSize", &options.limit.unwrap_or(100).to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(response.files.iter().map(Self::convert_file).collect())
    }

    async fn get_file(&self, file_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let fields = "id,name,mimeType,size,createdTime,modifiedTime,md5Checksum,parents,shared,webContentLink,thumbnailLink,version";

        let file: GoogleFile = self.client
            .get(&format!("{}/files/{}", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .query(&[("fields", fields)])
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_file(&file))
    }

    async fn search(&self, options: SearchOptions) -> Result<Vec<CloudFile>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let mut query = format!("fullText contains '{}'", options.query);

        if let Some(folder_id) = &options.folder_id {
            query.push_str(&format!(" and '{}' in parents", folder_id));
        }

        if !options.include_trashed {
            query.push_str(" and trashed = false");
        }

        let response: GoogleFileList = self.client
            .get(&format!("{}/files", GOOGLE_API_BASE))
            .bearer_auth(&token.access_token)
            .query(&[
                ("q", query.as_str()),
                ("pageSize", &options.limit.unwrap_or(100).to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(response.files.iter().map(Self::convert_file).collect())
    }

    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct CreateFolderRequest<'a> {
            name: &'a str,
            #[serde(rename = "mimeType")]
            mime_type: &'a str,
            parents: Vec<&'a str>,
        }

        let parents = vec![parent_id.unwrap_or("root")];
        let request = CreateFolderRequest {
            name,
            mime_type: "application/vnd.google-apps.folder",
            parents,
        };

        let file: GoogleFile = self.client
            .post(&format!("{}/files", GOOGLE_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_file(&file))
    }

    async fn upload_file(
        &self,
        local_path: &Path,
        parent_id: Option<&str>,
        name: Option<&str>,
        _progress: Option<ProgressCallback>,
    ) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let file_name = name.map(|s| s.to_string())
            .or_else(|| local_path.file_name().map(|n| n.to_string_lossy().to_string()))
            .ok_or_else(|| anyhow!("Could not determine file name"))?;

        let content = std::fs::read(local_path)?;
        let mime_type = mime_guess::from_path(local_path)
            .first_or_octet_stream()
            .to_string();

        #[derive(Serialize)]
        struct FileMetadata<'a> {
            name: &'a str,
            parents: Vec<&'a str>,
        }

        let parents = vec![parent_id.unwrap_or("root")];
        let metadata = FileMetadata {
            name: &file_name,
            parents,
        };

        // Use multipart upload for files
        let boundary = "foo_bar_baz";
        let mut body = Vec::new();

        // Metadata part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body.extend_from_slice(&serde_json::to_vec(&metadata)?);
        body.extend_from_slice(b"\r\n");

        // File content part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes());
        body.extend_from_slice(&content);
        body.extend_from_slice(format!("\r\n--{}--", boundary).as_bytes());

        let file: GoogleFile = self.client
            .post(&format!("{}/files?uploadType=multipart", GOOGLE_UPLOAD_URL))
            .bearer_auth(&token.access_token)
            .header("Content-Type", format!("multipart/related; boundary={}", boundary))
            .body(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_file(&file))
    }

    async fn download_file(
        &self,
        file_id: &str,
        local_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let response = self.client
            .get(&format!("{}/files/{}?alt=media", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?;

        let content = response.bytes().await?;
        std::fs::write(local_path, content)?;

        Ok(())
    }

    async fn move_file(&self, file_id: &str, new_parent_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // First get current parents
        let file = self.get_file(file_id).await?;
        let remove_parents = file.parent_id.unwrap_or_default();

        let response: GoogleFile = self.client
            .patch(&format!("{}/files/{}", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .query(&[
                ("addParents", new_parent_id),
                ("removeParents", &remove_parents),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_file(&response))
    }

    async fn rename_file(&self, file_id: &str, new_name: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct RenameRequest<'a> {
            name: &'a str,
        }

        let response: GoogleFile = self.client
            .patch(&format!("{}/files/{}", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&RenameRequest { name: new_name })
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_file(&response))
    }

    async fn copy_file(&self, file_id: &str, dest_parent_id: &str, new_name: Option<&str>) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct CopyRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
            parents: Vec<&'a str>,
        }

        let request = CopyRequest {
            name: new_name,
            parents: vec![dest_parent_id],
        };

        let response: GoogleFile = self.client
            .post(&format!("{}/files/{}/copy", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_file(&response))
    }

    async fn delete_file(&self, file_id: &str) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct TrashRequest {
            trashed: bool,
        }

        self.client
            .patch(&format!("{}/files/{}", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&TrashRequest { trashed: true })
            .send()
            .await?;

        Ok(())
    }

    async fn permanent_delete(&self, file_id: &str) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        self.client
            .delete(&format!("{}/files/{}", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?;

        Ok(())
    }

    async fn restore_file(&self, file_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct UntrashRequest {
            trashed: bool,
        }

        let response: GoogleFile = self.client
            .patch(&format!("{}/files/{}", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&UntrashRequest { trashed: false })
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_file(&response))
    }

    async fn empty_trash(&self) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        self.client
            .delete(&format!("{}/files/trash", GOOGLE_API_BASE))
            .bearer_auth(&token.access_token)
            .send()
            .await?;

        Ok(())
    }

    async fn create_shared_link(
        &self,
        file_id: &str,
        _expires_at: Option<DateTime<Utc>>,
        _password: Option<&str>,
    ) -> Result<SharedLink> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct PermissionRequest<'a> {
            role: &'a str,
            #[serde(rename = "type")]
            permission_type: &'a str,
        }

        // Create anyone permission
        self.client
            .post(&format!("{}/files/{}/permissions", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&PermissionRequest {
                role: "reader",
                permission_type: "anyone",
            })
            .send()
            .await?;

        // Get the web view link
        let file = self.get_file(file_id).await?;

        Ok(SharedLink {
            url: file.download_url.unwrap_or_default(),
            expires_at: None,
            public: true,
            editable: false,
            password_protected: false,
            download_count: 0,
        })
    }

    async fn get_shared_link(&self, file_id: &str) -> Result<Option<SharedLink>> {
        let file = self.get_file(file_id).await?;

        if file.shared {
            Ok(Some(SharedLink {
                url: file.download_url.unwrap_or_default(),
                expires_at: None,
                public: true,
                editable: false,
                password_protected: false,
                download_count: 0,
            }))
        } else {
            Ok(None)
        }
    }

    async fn revoke_shared_link(&self, file_id: &str) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // List permissions and remove "anyone" permission
        #[derive(Deserialize)]
        struct PermissionList {
            permissions: Vec<Permission>,
        }

        #[derive(Deserialize)]
        struct Permission {
            id: String,
            #[serde(rename = "type")]
            permission_type: String,
        }

        let response: PermissionList = self.client
            .get(&format!("{}/files/{}/permissions", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        for perm in response.permissions {
            if perm.permission_type == "anyone" {
                self.client
                    .delete(&format!("{}/files/{}/permissions/{}", GOOGLE_API_BASE, file_id, perm.id))
                    .bearer_auth(&token.access_token)
                    .send()
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_versions(&self, file_id: &str) -> Result<Vec<FileVersion>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Deserialize)]
        struct RevisionList {
            revisions: Vec<Revision>,
        }

        #[derive(Deserialize)]
        struct Revision {
            id: String,
            #[serde(rename = "modifiedTime")]
            modified_time: DateTime<Utc>,
            size: Option<String>,
        }

        let response: RevisionList = self.client
            .get(&format!("{}/files/{}/revisions", GOOGLE_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.revisions.iter().map(|r| FileVersion {
            id: r.id.clone(),
            file_id: file_id.to_string(),
            version: r.id.clone(),
            size: r.size.as_ref().and_then(|s| s.parse().ok()).unwrap_or(0),
            modified_at: r.modified_time,
            modified_by: None,
        }).collect())
    }

    async fn restore_version(&self, file_id: &str, version_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // Download the version content and re-upload
        let content = self.client
            .get(&format!("{}/files/{}/revisions/{}?alt=media", GOOGLE_API_BASE, file_id, version_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .bytes()
            .await?;

        // Update file content
        let response: GoogleFile = self.client
            .patch(&format!("{}/files/{}?uploadType=media", GOOGLE_UPLOAD_URL, file_id))
            .bearer_auth(&token.access_token)
            .body(content.to_vec())
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_file(&response))
    }

    async fn get_changes(&self, cursor: Option<&str>) -> Result<(Vec<CloudFile>, Option<String>)> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // Get start page token if no cursor provided
        let page_token = if let Some(c) = cursor {
            c.to_string()
        } else {
            #[derive(Deserialize)]
            struct StartTokenResponse {
                #[serde(rename = "startPageToken")]
                start_page_token: String,
            }

            let response: StartTokenResponse = self.client
                .get(&format!("{}/changes/startPageToken", GOOGLE_API_BASE))
                .bearer_auth(&token.access_token)
                .send()
                .await?
                .json()
                .await?;

            response.start_page_token
        };

        #[derive(Deserialize)]
        struct ChangesResponse {
            changes: Vec<Change>,
            #[serde(rename = "newStartPageToken")]
            new_start_page_token: Option<String>,
            #[serde(rename = "nextPageToken")]
            next_page_token: Option<String>,
        }

        #[derive(Deserialize)]
        struct Change {
            file: Option<GoogleFile>,
            removed: Option<bool>,
        }

        let response: ChangesResponse = self.client
            .get(&format!("{}/changes", GOOGLE_API_BASE))
            .bearer_auth(&token.access_token)
            .query(&[("pageToken", &page_token)])
            .send()
            .await?
            .json()
            .await?;

        let files: Vec<CloudFile> = response.changes
            .into_iter()
            .filter_map(|c| c.file.map(|f| Self::convert_file(&f)))
            .collect();

        let new_cursor = response.new_start_page_token.or(response.next_page_token);

        Ok((files, new_cursor))
    }
}

/// Google Drive file response
#[derive(Debug, Deserialize)]
struct GoogleFile {
    id: String,
    name: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    #[serde(default)]
    size: String,
    #[serde(rename = "createdTime")]
    created_time: Option<DateTime<Utc>>,
    #[serde(rename = "modifiedTime")]
    modified_time: Option<DateTime<Utc>>,
    #[serde(rename = "md5Checksum")]
    md5_checksum: Option<String>,
    #[serde(default)]
    parents: Vec<String>,
    shared: Option<bool>,
    #[serde(rename = "webContentLink")]
    web_content_link: Option<String>,
    #[serde(rename = "thumbnailLink")]
    thumbnail_link: Option<String>,
    version: Option<String>,
}

/// Google Drive file list response
#[derive(Debug, Deserialize)]
struct GoogleFileList {
    files: Vec<GoogleFile>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}
