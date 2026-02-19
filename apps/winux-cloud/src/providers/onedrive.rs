//! OneDrive provider implementation
//!
//! Implements OAuth2 authentication and Microsoft Graph API

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

/// Microsoft OAuth2 configuration
const MS_AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const GRAPH_API_BASE: &str = "https://graph.microsoft.com/v1.0";

/// Required OAuth2 scopes
const SCOPES: &[&str] = &[
    "Files.ReadWrite.All",
    "User.Read",
    "offline_access",
];

/// OneDrive provider
pub struct OneDriveProvider {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    tokens: Option<OAuthTokens>,
}

impl OneDriveProvider {
    /// Create a new OneDrive provider
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

    /// Convert OneDrive item to CloudFile
    fn convert_item(item: &DriveItem) -> CloudFile {
        CloudFile {
            id: item.id.clone(),
            name: item.name.clone(),
            path: item.parent_reference.as_ref()
                .map(|p| format!("{}/{}", p.path.clone().unwrap_or_default(), &item.name))
                .unwrap_or_else(|| item.name.clone()),
            parent_id: item.parent_reference.as_ref().map(|p| p.id.clone()),
            is_folder: item.folder.is_some(),
            size: item.size.unwrap_or(0),
            mime_type: item.file.as_ref()
                .and_then(|f| f.mime_type.clone())
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            created_at: item.created_date_time.unwrap_or_else(Utc::now),
            modified_at: item.last_modified_date_time.unwrap_or_else(Utc::now),
            hash: item.file.as_ref()
                .and_then(|f| f.hashes.as_ref())
                .and_then(|h| h.sha256_hash.clone()),
            shared: item.shared.is_some(),
            download_url: item.download_url.clone(),
            thumbnail_url: None,
            version: item.e_tag.clone(),
        }
    }
}

#[async_trait::async_trait]
impl CloudProvider for OneDriveProvider {
    fn name(&self) -> &str {
        "OneDrive"
    }

    fn icon(&self) -> &str {
        "onedrive"
    }

    fn is_authenticated(&self) -> bool {
        self.tokens.is_some()
    }

    fn get_auth_url(&self) -> Option<String> {
        let mut url = Url::parse(MS_AUTH_URL).ok()?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &SCOPES.join(" "))
            .append_pair("response_mode", "query");

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
            .post(MS_TOKEN_URL)
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
        Err(anyhow!("OneDrive does not support basic authentication"))
    }

    async fn authenticate_api_key(&mut self, _access_key: &str, _secret_key: &str) -> Result<()> {
        Err(anyhow!("OneDrive does not support API key authentication"))
    }

    async fn refresh_auth(&mut self) -> Result<()> {
        let refresh_token = self.tokens
            .as_ref()
            .and_then(|t| t.refresh_token.clone())
            .ok_or_else(|| anyhow!("No refresh token available"))?;

        #[derive(Deserialize)]
        struct RefreshResponse {
            access_token: String,
            refresh_token: Option<String>,
            token_type: String,
            expires_in: u64,
        }

        let response: RefreshResponse = self.client
            .post(MS_TOKEN_URL)
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
            if let Some(new_refresh) = response.refresh_token {
                tokens.refresh_token = Some(new_refresh);
            }
            tokens.expires_at = Some(Utc::now() + chrono::Duration::seconds(response.expires_in as i64));
        }

        Ok(())
    }

    async fn get_quota(&self) -> Result<StorageQuota> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Deserialize)]
        struct DriveResponse {
            quota: QuotaInfo,
        }

        #[derive(Deserialize)]
        struct QuotaInfo {
            total: u64,
            used: u64,
            remaining: u64,
            deleted: Option<u64>,
        }

        let response: DriveResponse = self.client
            .get(&format!("{}/me/drive", GRAPH_API_BASE))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(StorageQuota {
            total: response.quota.total,
            used: response.quota.used,
            available: response.quota.remaining,
            trash: response.quota.deleted,
        })
    }

    async fn list_files(&self, folder_id: Option<&str>, _options: ListOptions) -> Result<Vec<CloudFile>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = match folder_id {
            Some(id) => format!("{}/me/drive/items/{}/children", GRAPH_API_BASE, id),
            None => format!("{}/me/drive/root/children", GRAPH_API_BASE),
        };

        let response: DriveItemList = self.client
            .get(&url)
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.value.iter().map(Self::convert_item).collect())
    }

    async fn get_file(&self, file_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let item: DriveItem = self.client
            .get(&format!("{}/me/drive/items/{}", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_item(&item))
    }

    async fn search(&self, options: SearchOptions) -> Result<Vec<CloudFile>> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let response: DriveItemList = self.client
            .get(&format!("{}/me/drive/root/search(q='{}')", GRAPH_API_BASE, options.query))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.value.iter().map(Self::convert_item).collect())
    }

    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct CreateFolderRequest<'a> {
            name: &'a str,
            folder: EmptyObject,
            #[serde(rename = "@microsoft.graph.conflictBehavior")]
            conflict_behavior: &'a str,
        }

        #[derive(Serialize)]
        struct EmptyObject {}

        let url = match parent_id {
            Some(id) => format!("{}/me/drive/items/{}/children", GRAPH_API_BASE, id),
            None => format!("{}/me/drive/root/children", GRAPH_API_BASE),
        };

        let request = CreateFolderRequest {
            name,
            folder: EmptyObject {},
            conflict_behavior: "rename",
        };

        let item: DriveItem = self.client
            .post(&url)
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_item(&item))
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

        // Use simple upload for small files (< 4MB)
        let url = match parent_id {
            Some(id) => format!("{}/me/drive/items/{}:/{}:/content", GRAPH_API_BASE, id, file_name),
            None => format!("{}/me/drive/root:/{}:/content", GRAPH_API_BASE, file_name),
        };

        let item: DriveItem = self.client
            .put(&url)
            .bearer_auth(&token.access_token)
            .body(content)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_item(&item))
    }

    async fn download_file(
        &self,
        file_id: &str,
        local_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let response = self.client
            .get(&format!("{}/me/drive/items/{}/content", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?;

        let content = response.bytes().await?;
        std::fs::write(local_path, content)?;

        Ok(())
    }

    async fn move_file(&self, file_id: &str, new_parent_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct MoveRequest<'a> {
            #[serde(rename = "parentReference")]
            parent_reference: ParentRef<'a>,
        }

        #[derive(Serialize)]
        struct ParentRef<'a> {
            id: &'a str,
        }

        let request = MoveRequest {
            parent_reference: ParentRef { id: new_parent_id },
        };

        let item: DriveItem = self.client
            .patch(&format!("{}/me/drive/items/{}", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_item(&item))
    }

    async fn rename_file(&self, file_id: &str, new_name: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct RenameRequest<'a> {
            name: &'a str,
        }

        let item: DriveItem = self.client
            .patch(&format!("{}/me/drive/items/{}", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&RenameRequest { name: new_name })
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_item(&item))
    }

    async fn copy_file(&self, file_id: &str, dest_parent_id: &str, new_name: Option<&str>) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        #[derive(Serialize)]
        struct CopyRequest<'a> {
            #[serde(rename = "parentReference")]
            parent_reference: ParentRef<'a>,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
        }

        #[derive(Serialize)]
        struct ParentRef<'a> {
            id: &'a str,
        }

        let request = CopyRequest {
            parent_reference: ParentRef { id: dest_parent_id },
            name: new_name,
        };

        // Copy is async, returns a monitor URL
        let response = self.client
            .post(&format!("{}/me/drive/items/{}/copy", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?;

        // Wait for copy to complete
        if let Some(monitor_url) = response.headers().get("Location") {
            let monitor_url = monitor_url.to_str()?;

            // Poll until complete
            loop {
                let status: CopyStatus = self.client
                    .get(monitor_url)
                    .send()
                    .await?
                    .json()
                    .await?;

                if status.status == "completed" {
                    if let Some(resource_id) = status.resource_id {
                        return self.get_file(&resource_id).await;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        Err(anyhow!("Copy operation did not return a monitor URL"))
    }

    async fn delete_file(&self, file_id: &str) -> Result<()> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        self.client
            .delete(&format!("{}/me/drive/items/{}", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?;

        Ok(())
    }

    async fn permanent_delete(&self, file_id: &str) -> Result<()> {
        // OneDrive doesn't have separate permanent delete - items go to recycle bin
        self.delete_file(file_id).await
    }

    async fn restore_file(&self, _file_id: &str) -> Result<CloudFile> {
        Err(anyhow!("OneDrive restore from trash requires different API"))
    }

    async fn empty_trash(&self) -> Result<()> {
        Err(anyhow!("OneDrive empty trash requires different API"))
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
            #[serde(rename = "type")]
            link_type: &'a str,
            scope: &'a str,
            #[serde(rename = "expirationDateTime", skip_serializing_if = "Option::is_none")]
            expiration: Option<String>,
        }

        let request = CreateLinkRequest {
            link_type: "view",
            scope: "anonymous",
            expiration: expires_at.map(|dt| dt.to_rfc3339()),
        };

        #[derive(Deserialize)]
        struct LinkResponse {
            link: LinkInfo,
        }

        #[derive(Deserialize)]
        struct LinkInfo {
            #[serde(rename = "webUrl")]
            web_url: String,
        }

        let response: LinkResponse = self.client
            .post(&format!("{}/me/drive/items/{}/createLink", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(SharedLink {
            url: response.link.web_url,
            expires_at,
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

        // List permissions and delete them
        #[derive(Deserialize)]
        struct PermissionList {
            value: Vec<Permission>,
        }

        #[derive(Deserialize)]
        struct Permission {
            id: String,
            link: Option<LinkInfo>,
        }

        #[derive(Deserialize)]
        struct LinkInfo {
            scope: String,
        }

        let response: PermissionList = self.client
            .get(&format!("{}/me/drive/items/{}/permissions", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        for perm in response.value {
            if perm.link.map(|l| l.scope == "anonymous").unwrap_or(false) {
                self.client
                    .delete(&format!("{}/me/drive/items/{}/permissions/{}", GRAPH_API_BASE, file_id, perm.id))
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
        struct VersionList {
            value: Vec<DriveItemVersion>,
        }

        #[derive(Deserialize)]
        struct DriveItemVersion {
            id: String,
            #[serde(rename = "lastModifiedDateTime")]
            last_modified: DateTime<Utc>,
            size: Option<u64>,
        }

        let response: VersionList = self.client
            .get(&format!("{}/me/drive/items/{}/versions", GRAPH_API_BASE, file_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.value.iter().map(|v| FileVersion {
            id: v.id.clone(),
            file_id: file_id.to_string(),
            version: v.id.clone(),
            size: v.size.unwrap_or(0),
            modified_at: v.last_modified,
            modified_by: None,
        }).collect())
    }

    async fn restore_version(&self, file_id: &str, version_id: &str) -> Result<CloudFile> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let item: DriveItem = self.client
            .post(&format!("{}/me/drive/items/{}/versions/{}/restoreVersion", GRAPH_API_BASE, file_id, version_id))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(Self::convert_item(&item))
    }

    async fn get_changes(&self, cursor: Option<&str>) -> Result<(Vec<CloudFile>, Option<String>)> {
        let token = self.tokens.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;

        let url = match cursor {
            Some(c) => c.to_string(),
            None => format!("{}/me/drive/root/delta", GRAPH_API_BASE),
        };

        #[derive(Deserialize)]
        struct DeltaResponse {
            value: Vec<DriveItem>,
            #[serde(rename = "@odata.deltaLink")]
            delta_link: Option<String>,
            #[serde(rename = "@odata.nextLink")]
            next_link: Option<String>,
        }

        let response: DeltaResponse = self.client
            .get(&url)
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .json()
            .await?;

        let files: Vec<CloudFile> = response.value.iter().map(Self::convert_item).collect();
        let new_cursor = response.delta_link.or(response.next_link);

        Ok((files, new_cursor))
    }
}

/// OneDrive item response
#[derive(Debug, Deserialize)]
struct DriveItem {
    id: String,
    name: String,
    size: Option<u64>,
    #[serde(rename = "createdDateTime")]
    created_date_time: Option<DateTime<Utc>>,
    #[serde(rename = "lastModifiedDateTime")]
    last_modified_date_time: Option<DateTime<Utc>>,
    #[serde(rename = "eTag")]
    e_tag: Option<String>,
    #[serde(rename = "parentReference")]
    parent_reference: Option<ParentReference>,
    file: Option<FileInfo>,
    folder: Option<FolderInfo>,
    shared: Option<SharedInfo>,
    #[serde(rename = "@microsoft.graph.downloadUrl")]
    download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ParentReference {
    id: String,
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FileInfo {
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    hashes: Option<FileHashes>,
}

#[derive(Debug, Deserialize)]
struct FileHashes {
    #[serde(rename = "sha256Hash")]
    sha256_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FolderInfo {
    #[serde(rename = "childCount")]
    child_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SharedInfo {
    scope: String,
}

#[derive(Debug, Deserialize)]
struct DriveItemList {
    value: Vec<DriveItem>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CopyStatus {
    status: String,
    #[serde(rename = "resourceId")]
    resource_id: Option<String>,
}
