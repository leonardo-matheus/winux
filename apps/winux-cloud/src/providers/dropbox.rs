//! Dropbox provider implementation
//!
//! Implements OAuth2 authentication and Dropbox API v2

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

/// Dropbox OAuth2 configuration
const DROPBOX_AUTH_URL: &str = "https://www.dropbox.com/oauth2/authorize";
const DROPBOX_TOKEN_URL: &str = "https://api.dropboxapi.com/oauth2/token";
const DROPBOX_API_BASE: &str = "https://api.dropboxapi.com/2";
const DROPBOX_CONTENT_URL: &str = "https://content.dropboxapi.com/2";

/// Dropbox provider
pub struct DropboxProvider {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    tokens: Option<OAuthTokens>,
}

impl DropboxProvider {
    /// Create a new Dropbox provider
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

    /// Convert Dropbox metadata to CloudFile
    fn convert_metadata(metadata: &DropboxMetadata) -> CloudFile {
        match metadata {
            DropboxMetadata::File(f) => CloudFile {
                id: f.id.clone(),
                name: f.name.clone(),
                path: f.path_display.clone().unwrap_or_default(),
                parent_id: f.path_lower.as_ref().and_then(|p| {
                    let path = Path::new(p);
                    path.parent().map(|parent| parent.to_string_lossy().to_string())
                }),
                is_folder: false,
                size: f.size,
                mime_type: mime_guess::from_path(&f.name)
                    .first_or_octet_stream()
                    .to_string(),
                created_at: Utc::now(), // Dropbox doesn't provide creation time
                modified_at: f.client_modified.unwrap_or_else(Utc::now),
                hash: f.content_hash.clone(),
                shared: false,
                download_url: None,
                thumbnail_url: None,
                version: f.rev.clone(),
            },
            DropboxMetadata::Folder(f) => CloudFile {
                id: f.id.clone(),
                name: f.name.clone(),
                path: f.path_display.clone().unwrap_or_default(),
                parent_id: f.path_lower.as_ref().and_then(|p| {
                    let path = Path::new(p);
                    path.parent().map(|parent| parent.to_string_lossy().to_string())
                }),
                is_folder: true,
                size: 0,
                mime_type: "application/x-directory".to_string(),
                created_at: Utc::now(),
                modified_at: Utc::now(),
                hash: None,
                shared: false,
                download_url: None,
                thumbnail_url: None,
                version: None,
            },
            DropboxMetadata::Deleted(d) => CloudFile {
                id: String::new(),
                name: d.name.clone(),
                path: d.path_display.clone().unwrap_or_default(),
                parent_id: None,
                is_folder: false,
                size: 0,
                mime_type: "application/octet-stream".to_string(),
                created_at: Utc::now(),
                modified_at: Utc::now(),
                hash: None,
                shared: false,
                download_url: None,
                thumbnail_url: None,
                version: None,
            },
        }
    }
}

#[async_trait::async_trait]
impl CloudProvider for DropboxProvider {
    fn name(&self) -> &str {
        "Dropbox"
    }

    fn icon(&self) -> &str {
        "dropbox"
    }

    fn is_authenticated(&self) -> bool {
        self.tokens.is_some()
    }

    fn get_auth_url(&self) -> Option<String> {
        let mut url = Url::parse(DROPBOX_AUTH_URL).ok()?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("token_access_type", "offline");

        Some(url.to_string())
    }

    async fn authenticate_oauth(&mut self, code: &str) -> Result<()> {
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            token_type: String,
            expires_in: Option<u64>,
        }

        let response: TokenResponse = self.client
            .post(DROPBOX_TOKEN_URL)
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

        let expires_at = response.expires_in.map(|e| Utc::now() + chrono::Duration::seconds(e as i64));

        self.tokens = Some(OAuthTokens {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type,
            expires_at,
            scopes: vec![],
        });

        Ok(())
    }

    async fn authenticate_basic(&mut self, _username: &str, _password: &str) -> Result<()> {
        Err(anyhow!("Dropbox does not support basic authentication"))
    }

    async fn authenticate_api_key(&mut self, _access_key: &str, _secret_key: &str) -> Result<()> {
        Err(anyhow!("Dropbox does not support API key authentication"))
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
            .post(DROPBOX_TOKEN_URL)
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
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Deserialize)]
        struct SpaceUsage {
            used: u64,
            allocation: Allocation,
        }

        #[derive(Deserialize)]
        #[serde(tag = ".tag")]
        enum Allocation {
            #[serde(rename = "individual")]
            Individual { allocated: u64 },
            #[serde(rename = "team")]
            Team { allocated: u64 },
        }

        let response: SpaceUsage = self.client
            .post(&format!("{}/users/get_space_usage", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await?
            .json()
            .await?;

        let total = match response.allocation {
            Allocation::Individual { allocated } | Allocation::Team { allocated } => allocated,
        };

        Ok(StorageQuota {
            total,
            used: response.used,
            available: total.saturating_sub(response.used),
            trash: None,
        })
    }

    async fn list_files(&self, folder_id: Option<&str>, _options: ListOptions) -> Result<Vec<CloudFile>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct ListFolderRequest<'a> {
            path: &'a str,
            recursive: bool,
            include_deleted: bool,
        }

        let path = folder_id.unwrap_or("");
        let request = ListFolderRequest {
            path,
            recursive: false,
            include_deleted: false,
        };

        #[derive(Deserialize)]
        struct ListFolderResult {
            entries: Vec<DropboxMetadata>,
            cursor: String,
            has_more: bool,
        }

        let response: ListFolderResult = self.client
            .post(&format!("{}/files/list_folder", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.entries.iter().map(Self::convert_metadata).collect())
    }

    async fn get_file(&self, file_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct GetMetadataRequest<'a> {
            path: &'a str,
        }

        let metadata: DropboxMetadata = self.client
            .post(&format!("{}/files/get_metadata", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&GetMetadataRequest { path: file_id })
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_metadata(&metadata))
    }

    async fn search(&self, options: SearchOptions) -> Result<Vec<CloudFile>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct SearchRequest<'a> {
            query: &'a str,
            options: SearchOptions2,
        }

        #[derive(Serialize)]
        struct SearchOptions2 {
            max_results: u32,
        }

        let request = SearchRequest {
            query: &options.query,
            options: SearchOptions2 {
                max_results: options.limit.unwrap_or(100),
            },
        };

        #[derive(Deserialize)]
        struct SearchResult {
            matches: Vec<SearchMatch>,
        }

        #[derive(Deserialize)]
        struct SearchMatch {
            metadata: MetadataWrapper,
        }

        #[derive(Deserialize)]
        struct MetadataWrapper {
            metadata: DropboxMetadata,
        }

        let response: SearchResult = self.client
            .post(&format!("{}/files/search_v2", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.matches.iter().map(|m| Self::convert_metadata(&m.metadata.metadata)).collect())
    }

    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct CreateFolderRequest {
            path: String,
            autorename: bool,
        }

        let path = match parent_id {
            Some(p) => format!("{}/{}", p, name),
            None => format!("/{}", name),
        };

        let request = CreateFolderRequest {
            path,
            autorename: true,
        };

        #[derive(Deserialize)]
        struct CreateFolderResult {
            metadata: DropboxMetadata,
        }

        let response: CreateFolderResult = self.client
            .post(&format!("{}/files/create_folder_v2", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_metadata(&response.metadata))
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

        let path = match parent_id {
            Some(p) => format!("{}/{}", p, file_name),
            None => format!("/{}", file_name),
        };

        #[derive(Serialize)]
        struct UploadArgs {
            path: String,
            mode: WriteMode,
            autorename: bool,
            mute: bool,
        }

        #[derive(Serialize)]
        struct WriteMode {
            #[serde(rename = ".tag")]
            tag: String,
        }

        let args = UploadArgs {
            path,
            mode: WriteMode { tag: "add".to_string() },
            autorename: true,
            mute: false,
        };

        let metadata: DropboxMetadata = self.client
            .post(&format!("{}/files/upload", DROPBOX_CONTENT_URL))
            .bearer_auth(&token.access_token)
            .header("Dropbox-API-Arg", serde_json::to_string(&args)?)
            .header("Content-Type", "application/octet-stream")
            .body(content)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_metadata(&metadata))
    }

    async fn download_file(
        &self,
        file_id: &str,
        local_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct DownloadArgs<'a> {
            path: &'a str,
        }

        let response = self.client
            .post(&format!("{}/files/download", DROPBOX_CONTENT_URL))
            .bearer_auth(&token.access_token)
            .header("Dropbox-API-Arg", serde_json::to_string(&DownloadArgs { path: file_id })?)
            .send()
            .await?;

        let content = response.bytes().await?;
        std::fs::write(local_path, content)?;

        Ok(())
    }

    async fn move_file(&self, file_id: &str, new_parent_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // Get current file name
        let file = self.get_file(file_id).await?;

        #[derive(Serialize)]
        struct MoveRequest {
            from_path: String,
            to_path: String,
            autorename: bool,
        }

        let request = MoveRequest {
            from_path: file_id.to_string(),
            to_path: format!("{}/{}", new_parent_id, file.name),
            autorename: true,
        };

        #[derive(Deserialize)]
        struct MoveResult {
            metadata: DropboxMetadata,
        }

        let response: MoveResult = self.client
            .post(&format!("{}/files/move_v2", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_metadata(&response.metadata))
    }

    async fn rename_file(&self, file_id: &str, new_name: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // Get current file path
        let file = self.get_file(file_id).await?;
        let parent_path = Path::new(&file.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        #[derive(Serialize)]
        struct MoveRequest {
            from_path: String,
            to_path: String,
            autorename: bool,
        }

        let request = MoveRequest {
            from_path: file_id.to_string(),
            to_path: format!("{}/{}", parent_path, new_name),
            autorename: false,
        };

        #[derive(Deserialize)]
        struct MoveResult {
            metadata: DropboxMetadata,
        }

        let response: MoveResult = self.client
            .post(&format!("{}/files/move_v2", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_metadata(&response.metadata))
    }

    async fn copy_file(&self, file_id: &str, dest_parent_id: &str, new_name: Option<&str>) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let file = self.get_file(file_id).await?;
        let name = new_name.unwrap_or(&file.name);

        #[derive(Serialize)]
        struct CopyRequest {
            from_path: String,
            to_path: String,
            autorename: bool,
        }

        let request = CopyRequest {
            from_path: file_id.to_string(),
            to_path: format!("{}/{}", dest_parent_id, name),
            autorename: true,
        };

        #[derive(Deserialize)]
        struct CopyResult {
            metadata: DropboxMetadata,
        }

        let response: CopyResult = self.client
            .post(&format!("{}/files/copy_v2", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_metadata(&response.metadata))
    }

    async fn delete_file(&self, file_id: &str) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct DeleteRequest<'a> {
            path: &'a str,
        }

        self.client
            .post(&format!("{}/files/delete_v2", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&DeleteRequest { path: file_id })
            .send()
            .await?;

        Ok(())
    }

    async fn permanent_delete(&self, file_id: &str) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct DeleteRequest<'a> {
            path: &'a str,
        }

        self.client
            .post(&format!("{}/files/permanently_delete", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&DeleteRequest { path: file_id })
            .send()
            .await?;

        Ok(())
    }

    async fn restore_file(&self, file_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        // Get the latest revision and restore it
        let versions = self.get_versions(file_id).await?;
        if let Some(latest) = versions.first() {
            return self.restore_version(file_id, &latest.id).await;
        }

        Err(anyhow!("No versions found to restore"))
    }

    async fn empty_trash(&self) -> Result<()> {
        // Dropbox doesn't have a traditional trash - deleted files are removed
        Ok(())
    }

    async fn create_shared_link(
        &self,
        file_id: &str,
        expires_at: Option<DateTime<Utc>>,
        _password: Option<&str>,
    ) -> Result<SharedLink> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct CreateLinkRequest<'a> {
            path: &'a str,
            settings: LinkSettings,
        }

        #[derive(Serialize)]
        struct LinkSettings {
            #[serde(skip_serializing_if = "Option::is_none")]
            expires: Option<String>,
        }

        let request = CreateLinkRequest {
            path: file_id,
            settings: LinkSettings {
                expires: expires_at.map(|dt| dt.to_rfc3339()),
            },
        };

        #[derive(Deserialize)]
        struct LinkResult {
            url: String,
            expires: Option<DateTime<Utc>>,
        }

        let response: LinkResult = self.client
            .post(&format!("{}/sharing/create_shared_link_with_settings", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(SharedLink {
            url: response.url,
            expires_at: response.expires,
            public: true,
            editable: false,
            password_protected: false,
            download_count: 0,
        })
    }

    async fn get_shared_link(&self, file_id: &str) -> Result<Option<SharedLink>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct ListLinksRequest<'a> {
            path: &'a str,
            direct_only: bool,
        }

        #[derive(Deserialize)]
        struct LinksResult {
            links: Vec<LinkInfo>,
        }

        #[derive(Deserialize)]
        struct LinkInfo {
            url: String,
            expires: Option<DateTime<Utc>>,
        }

        let response: LinksResult = self.client
            .post(&format!("{}/sharing/list_shared_links", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&ListLinksRequest { path: file_id, direct_only: true })
            .send()
            .await?
            .json()
            .await?;

        if let Some(link) = response.links.first() {
            Ok(Some(SharedLink {
                url: link.url.clone(),
                expires_at: link.expires,
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

        // Get the shared link first
        if let Some(link) = self.get_shared_link(file_id).await? {
            #[derive(Serialize)]
            struct RevokeRequest<'a> {
                url: &'a str,
            }

            self.client
                .post(&format!("{}/sharing/revoke_shared_link", DROPBOX_API_BASE))
                .bearer_auth(&token.access_token)
                .json(&RevokeRequest { url: &link.url })
                .send()
                .await?;
        }

        Ok(())
    }

    async fn get_versions(&self, file_id: &str) -> Result<Vec<FileVersion>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct ListRevisionsRequest<'a> {
            path: &'a str,
            limit: u32,
        }

        #[derive(Deserialize)]
        struct RevisionsResult {
            entries: Vec<FileMetadata>,
        }

        #[derive(Deserialize)]
        struct FileMetadata {
            id: String,
            rev: String,
            size: u64,
            client_modified: DateTime<Utc>,
        }

        let response: RevisionsResult = self.client
            .post(&format!("{}/files/list_revisions", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&ListRevisionsRequest { path: file_id, limit: 100 })
            .send()
            .await?
            .json()
            .await?;

        Ok(response.entries.iter().map(|e| FileVersion {
            id: e.rev.clone(),
            file_id: e.id.clone(),
            version: e.rev.clone(),
            size: e.size,
            modified_at: e.client_modified,
            modified_by: None,
        }).collect())
    }

    async fn restore_version(&self, file_id: &str, version_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct RestoreRequest<'a> {
            path: &'a str,
            rev: &'a str,
        }

        let metadata: DropboxMetadata = self.client
            .post(&format!("{}/files/restore", DROPBOX_API_BASE))
            .bearer_auth(&token.access_token)
            .json(&RestoreRequest { path: file_id, rev: version_id })
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_metadata(&metadata))
    }

    async fn get_changes(&self, cursor: Option<&str>) -> Result<(Vec<CloudFile>, Option<String>)> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        if let Some(c) = cursor {
            // Continue from cursor
            #[derive(Serialize)]
            struct ContinueRequest<'a> {
                cursor: &'a str,
            }

            #[derive(Deserialize)]
            struct ContinueResult {
                entries: Vec<DropboxMetadata>,
                cursor: String,
                has_more: bool,
            }

            let response: ContinueResult = self.client
                .post(&format!("{}/files/list_folder/continue", DROPBOX_API_BASE))
                .bearer_auth(&token.access_token)
                .json(&ContinueRequest { cursor: c })
                .send()
                .await?
                .json()
                .await?;

            let files: Vec<CloudFile> = response.entries.iter().map(Self::convert_metadata).collect();
            let new_cursor = if response.has_more { Some(response.cursor) } else { None };

            Ok((files, new_cursor))
        } else {
            // Get latest cursor
            #[derive(Serialize)]
            struct GetCursorRequest<'a> {
                path: &'a str,
                recursive: bool,
            }

            #[derive(Deserialize)]
            struct CursorResult {
                cursor: String,
            }

            let response: CursorResult = self.client
                .post(&format!("{}/files/list_folder/get_latest_cursor", DROPBOX_API_BASE))
                .bearer_auth(&token.access_token)
                .json(&GetCursorRequest { path: "", recursive: true })
                .send()
                .await?
                .json()
                .await?;

            Ok((vec![], Some(response.cursor)))
        }
    }
}

/// Dropbox metadata types
#[derive(Debug, Deserialize)]
#[serde(tag = ".tag")]
enum DropboxMetadata {
    #[serde(rename = "file")]
    File(FileMetadata),
    #[serde(rename = "folder")]
    Folder(FolderMetadata),
    #[serde(rename = "deleted")]
    Deleted(DeletedMetadata),
}

#[derive(Debug, Deserialize)]
struct FileMetadata {
    id: String,
    name: String,
    path_lower: Option<String>,
    path_display: Option<String>,
    size: u64,
    client_modified: Option<DateTime<Utc>>,
    content_hash: Option<String>,
    rev: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FolderMetadata {
    id: String,
    name: String,
    path_lower: Option<String>,
    path_display: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeletedMetadata {
    name: String,
    path_lower: Option<String>,
    path_display: Option<String>,
}
