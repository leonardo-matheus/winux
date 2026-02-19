//! Cloud providers module
//!
//! Provides different cloud storage backends:
//! - Google Drive: OAuth2-based Google storage
//! - OneDrive: OAuth2-based Microsoft storage
//! - Dropbox: OAuth2-based Dropbox storage
//! - Nextcloud: WebDAV-based self-hosted storage
//! - WebDAV: Generic WebDAV protocol
//! - S3: S3-compatible storage (AWS, MinIO, etc.)

mod google_drive;
mod onedrive;
mod dropbox;
mod nextcloud;
mod webdav;
mod s3;

pub use google_drive::GoogleDriveProvider;
pub use onedrive::OneDriveProvider;
pub use dropbox::DropboxProvider;
pub use nextcloud::NextcloudProvider;
pub use webdav::WebDavProvider;
pub use s3::S3Provider;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use chrono::{DateTime, Utc};

/// Cloud file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFile {
    /// Unique file ID on the provider
    pub id: String,
    /// File name
    pub name: String,
    /// Full path in cloud storage
    pub path: String,
    /// Parent folder ID
    pub parent_id: Option<String>,
    /// Whether this is a folder
    pub is_folder: bool,
    /// File size in bytes
    pub size: u64,
    /// MIME type
    pub mime_type: String,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last modification time
    pub modified_at: DateTime<Utc>,
    /// Content hash (MD5 or SHA256 depending on provider)
    pub hash: Option<String>,
    /// Whether file is shared
    pub shared: bool,
    /// Download URL (if available)
    pub download_url: Option<String>,
    /// Thumbnail URL (if available)
    pub thumbnail_url: Option<String>,
    /// File version/revision
    pub version: Option<String>,
}

/// Cloud storage quota information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuota {
    /// Total storage in bytes
    pub total: u64,
    /// Used storage in bytes
    pub used: u64,
    /// Available storage in bytes
    pub available: u64,
    /// Storage in trash
    pub trash: Option<u64>,
}

/// Shared link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedLink {
    /// Link URL
    pub url: String,
    /// Expiration time (if any)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether anyone with link can access
    pub public: bool,
    /// Whether link allows editing
    pub editable: bool,
    /// Password protected
    pub password_protected: bool,
    /// Download count
    pub download_count: u64,
}

/// File version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    /// Version ID
    pub id: String,
    /// File ID
    pub file_id: String,
    /// Version number
    pub version: String,
    /// Size in bytes
    pub size: u64,
    /// Modification time
    pub modified_at: DateTime<Utc>,
    /// Who modified it
    pub modified_by: Option<String>,
}

/// OAuth2 tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// Expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Token scopes
    pub scopes: Vec<String>,
}

/// Provider credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderCredentials {
    /// OAuth2 authentication
    OAuth2(OAuthTokens),
    /// Basic authentication (username/password)
    Basic {
        username: String,
        password: String,
    },
    /// API key authentication
    ApiKey {
        access_key: String,
        secret_key: String,
        region: Option<String>,
    },
}

/// Upload/download progress
#[derive(Debug, Clone)]
pub struct TransferProgress {
    /// Current file being transferred
    pub file_name: String,
    /// Bytes transferred
    pub bytes_transferred: u64,
    /// Total bytes
    pub bytes_total: u64,
    /// Transfer speed in bytes/sec
    pub speed: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(TransferProgress) + Send + Sync>;

/// Listing options
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    /// Include trashed files
    pub include_trashed: bool,
    /// Maximum results
    pub limit: Option<u32>,
    /// Pagination token
    pub page_token: Option<String>,
    /// Sort by field
    pub sort_by: Option<String>,
    /// Sort direction
    pub sort_ascending: bool,
    /// Filter by MIME type
    pub mime_type_filter: Option<String>,
}

/// Search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Search query
    pub query: String,
    /// Search in specific folder
    pub folder_id: Option<String>,
    /// Include trashed files
    pub include_trashed: bool,
    /// Maximum results
    pub limit: Option<u32>,
}

/// Common trait for all cloud providers
#[async_trait::async_trait]
pub trait CloudProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Get provider icon name
    fn icon(&self) -> &str;

    /// Check if provider is authenticated
    fn is_authenticated(&self) -> bool;

    /// Get OAuth2 authorization URL (for OAuth2 providers)
    fn get_auth_url(&self) -> Option<String>;

    /// Complete OAuth2 authentication with authorization code
    async fn authenticate_oauth(&mut self, code: &str) -> Result<()>;

    /// Authenticate with username/password (for WebDAV/Nextcloud)
    async fn authenticate_basic(&mut self, username: &str, password: &str) -> Result<()>;

    /// Authenticate with API keys (for S3)
    async fn authenticate_api_key(&mut self, access_key: &str, secret_key: &str) -> Result<()>;

    /// Refresh authentication tokens
    async fn refresh_auth(&mut self) -> Result<()>;

    /// Get storage quota
    async fn get_quota(&self) -> Result<StorageQuota>;

    /// List files in a folder
    async fn list_files(&self, folder_id: Option<&str>, options: ListOptions) -> Result<Vec<CloudFile>>;

    /// Get file metadata
    async fn get_file(&self, file_id: &str) -> Result<CloudFile>;

    /// Search for files
    async fn search(&self, options: SearchOptions) -> Result<Vec<CloudFile>>;

    /// Create a folder
    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile>;

    /// Upload a file
    async fn upload_file(
        &self,
        local_path: &Path,
        parent_id: Option<&str>,
        name: Option<&str>,
        progress: Option<ProgressCallback>,
    ) -> Result<CloudFile>;

    /// Download a file
    async fn download_file(
        &self,
        file_id: &str,
        local_path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<()>;

    /// Move a file
    async fn move_file(&self, file_id: &str, new_parent_id: &str) -> Result<CloudFile>;

    /// Rename a file
    async fn rename_file(&self, file_id: &str, new_name: &str) -> Result<CloudFile>;

    /// Copy a file
    async fn copy_file(&self, file_id: &str, dest_parent_id: &str, new_name: Option<&str>) -> Result<CloudFile>;

    /// Delete a file (move to trash)
    async fn delete_file(&self, file_id: &str) -> Result<()>;

    /// Permanently delete a file
    async fn permanent_delete(&self, file_id: &str) -> Result<()>;

    /// Restore file from trash
    async fn restore_file(&self, file_id: &str) -> Result<CloudFile>;

    /// Empty trash
    async fn empty_trash(&self) -> Result<()>;

    /// Create a shared link
    async fn create_shared_link(
        &self,
        file_id: &str,
        expires_at: Option<DateTime<Utc>>,
        password: Option<&str>,
    ) -> Result<SharedLink>;

    /// Get shared link for a file
    async fn get_shared_link(&self, file_id: &str) -> Result<Option<SharedLink>>;

    /// Revoke shared link
    async fn revoke_shared_link(&self, file_id: &str) -> Result<()>;

    /// Get file versions
    async fn get_versions(&self, file_id: &str) -> Result<Vec<FileVersion>>;

    /// Restore a specific version
    async fn restore_version(&self, file_id: &str, version_id: &str) -> Result<CloudFile>;

    /// Get changes since last sync (delta sync)
    async fn get_changes(&self, cursor: Option<&str>) -> Result<(Vec<CloudFile>, Option<String>)>;
}

/// Provider type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    GoogleDrive,
    OneDrive,
    Dropbox,
    Nextcloud,
    WebDav,
    S3,
}

impl ProviderType {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::GoogleDrive => "Google Drive",
            Self::OneDrive => "OneDrive",
            Self::Dropbox => "Dropbox",
            Self::Nextcloud => "Nextcloud",
            Self::WebDav => "WebDAV",
            Self::S3 => "S3 Compatible",
        }
    }

    /// Get icon name
    pub fn icon_name(&self) -> &str {
        match self {
            Self::GoogleDrive => "google-drive",
            Self::OneDrive => "onedrive",
            Self::Dropbox => "dropbox",
            Self::Nextcloud => "network-server-symbolic",
            Self::WebDav => "network-workgroup-symbolic",
            Self::S3 => "network-server-symbolic",
        }
    }

    /// Check if provider uses OAuth2
    pub fn uses_oauth2(&self) -> bool {
        matches!(self, Self::GoogleDrive | Self::OneDrive | Self::Dropbox)
    }
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAccount {
    /// Unique account ID
    pub id: String,
    /// Provider type
    pub provider: ProviderType,
    /// Account name/email
    pub name: String,
    /// Account email
    pub email: Option<String>,
    /// Profile picture URL
    pub avatar_url: Option<String>,
    /// Storage quota
    pub quota: Option<StorageQuota>,
    /// Last sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Whether sync is enabled
    pub sync_enabled: bool,
    /// Credentials (encrypted in storage)
    #[serde(skip)]
    pub credentials: Option<ProviderCredentials>,
}
