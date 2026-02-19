//! S3-compatible storage provider
//!
//! Supports AWS S3, MinIO, DigitalOcean Spaces, Backblaze B2, and other S3-compatible services

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::path::Path;

use super::{
    CloudFile, CloudProvider, FileVersion, ListOptions, ProgressCallback,
    SearchOptions, SharedLink, StorageQuota,
};

/// S3-compatible storage provider
pub struct S3Provider {
    bucket: String,
    region: String,
    endpoint: Option<String>,
    access_key: Option<String>,
    secret_key: Option<String>,
}

impl S3Provider {
    /// Create a new S3 provider
    pub fn new(bucket: &str, region: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            region: region.to_string(),
            endpoint: None,
            access_key: None,
            secret_key: None,
        }
    }

    /// Create with custom endpoint (for MinIO, DigitalOcean Spaces, etc.)
    pub fn with_endpoint(bucket: &str, region: &str, endpoint: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            region: region.to_string(),
            endpoint: Some(endpoint.to_string()),
            access_key: None,
            secret_key: None,
        }
    }

    /// Create with credentials
    pub fn with_credentials(
        bucket: &str,
        region: &str,
        endpoint: Option<&str>,
        access_key: &str,
        secret_key: &str,
    ) -> Self {
        Self {
            bucket: bucket.to_string(),
            region: region.to_string(),
            endpoint: endpoint.map(|s| s.to_string()),
            access_key: Some(access_key.to_string()),
            secret_key: Some(secret_key.to_string()),
        }
    }

    /// Get S3 bucket instance
    fn get_bucket(&self) -> Result<s3::Bucket> {
        let credentials = s3::creds::Credentials::new(
            self.access_key.as_deref(),
            self.secret_key.as_deref(),
            None, // security_token
            None, // session_token
            None, // profile
        )?;

        let region = if let Some(endpoint) = &self.endpoint {
            s3::Region::Custom {
                region: self.region.clone(),
                endpoint: endpoint.clone(),
            }
        } else {
            self.region.parse()?
        };

        let bucket = s3::Bucket::new(&self.bucket, region, credentials)?;
        Ok(bucket)
    }

    /// Convert S3 object to CloudFile
    fn convert_object(object: &s3::serde_types::Object, bucket: &str) -> CloudFile {
        let key = object.key.clone();
        let name = Path::new(&key)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| key.clone());

        let is_folder = key.ends_with('/');

        CloudFile {
            id: key.clone(),
            name: if is_folder { name.trim_end_matches('/').to_string() } else { name },
            path: format!("s3://{}/{}", bucket, key),
            parent_id: Path::new(&key).parent().map(|p| p.to_string_lossy().to_string()),
            is_folder,
            size: object.size as u64,
            mime_type: mime_guess::from_path(&key)
                .first_or_octet_stream()
                .to_string(),
            created_at: Utc::now(), // S3 doesn't track creation time
            modified_at: DateTime::parse_from_rfc3339(&object.last_modified)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            hash: object.e_tag.clone(),
            shared: false,
            download_url: None,
            thumbnail_url: None,
            version: None,
        }
    }
}

#[async_trait::async_trait]
impl CloudProvider for S3Provider {
    fn name(&self) -> &str {
        "S3 Compatible"
    }

    fn icon(&self) -> &str {
        "network-server-symbolic"
    }

    fn is_authenticated(&self) -> bool {
        self.access_key.is_some() && self.secret_key.is_some()
    }

    fn get_auth_url(&self) -> Option<String> {
        None
    }

    async fn authenticate_oauth(&mut self, _code: &str) -> Result<()> {
        Err(anyhow!("S3 uses API key authentication, not OAuth2"))
    }

    async fn authenticate_basic(&mut self, _username: &str, _password: &str) -> Result<()> {
        Err(anyhow!("S3 uses API key authentication, not basic auth"))
    }

    async fn authenticate_api_key(&mut self, access_key: &str, secret_key: &str) -> Result<()> {
        self.access_key = Some(access_key.to_string());
        self.secret_key = Some(secret_key.to_string());

        // Test connection by listing objects
        let bucket = self.get_bucket()?;
        let _ = bucket.list("".to_string(), Some("/".to_string())).await?;

        Ok(())
    }

    async fn refresh_auth(&mut self) -> Result<()> {
        Ok(())
    }

    async fn get_quota(&self) -> Result<StorageQuota> {
        // S3 doesn't have quota concept - return unlimited
        Ok(StorageQuota {
            total: u64::MAX,
            used: 0,
            available: u64::MAX,
            trash: None,
        })
    }

    async fn list_files(&self, folder_id: Option<&str>, _options: ListOptions) -> Result<Vec<CloudFile>> {
        let bucket = self.get_bucket()?;

        let prefix = folder_id.map(|p| {
            let p = p.trim_start_matches('/');
            if p.ends_with('/') || p.is_empty() {
                p.to_string()
            } else {
                format!("{}/", p)
            }
        }).unwrap_or_default();

        let results = bucket.list(prefix.clone(), Some("/".to_string())).await?;

        let mut files = Vec::new();

        for result in results {
            // Add folders (common prefixes)
            for prefix_item in &result.common_prefixes.unwrap_or_default() {
                let key = prefix_item.prefix.clone();
                let name = key.trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or(&key)
                    .to_string();

                files.push(CloudFile {
                    id: key.clone(),
                    name,
                    path: format!("s3://{}/{}", self.bucket, key),
                    parent_id: Some(prefix.clone()),
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
                });
            }

            // Add files
            for object in &result.contents {
                // Skip the folder marker itself
                if object.key == prefix {
                    continue;
                }
                files.push(Self::convert_object(object, &self.bucket));
            }
        }

        Ok(files)
    }

    async fn get_file(&self, file_id: &str) -> Result<CloudFile> {
        let bucket = self.get_bucket()?;
        let key = file_id.trim_start_matches('/');

        let (head, _) = bucket.head_object(key).await?;

        let name = Path::new(key)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| key.to_string());

        let modified_at = head.last_modified
            .and_then(|s| DateTime::parse_from_rfc2822(&s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Ok(CloudFile {
            id: key.to_string(),
            name,
            path: format!("s3://{}/{}", self.bucket, key),
            parent_id: Path::new(key).parent().map(|p| p.to_string_lossy().to_string()),
            is_folder: key.ends_with('/'),
            size: head.content_length.unwrap_or(0) as u64,
            mime_type: head.content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            created_at: Utc::now(),
            modified_at,
            hash: head.e_tag,
            shared: false,
            download_url: None,
            thumbnail_url: None,
            version: None,
        })
    }

    async fn search(&self, options: SearchOptions) -> Result<Vec<CloudFile>> {
        // S3 doesn't have native search - list all and filter
        let bucket = self.get_bucket()?;

        let prefix = options.folder_id.clone().unwrap_or_default();
        let results = bucket.list(prefix, None).await?;

        let query_lower = options.query.to_lowercase();
        let mut files = Vec::new();

        for result in results {
            for object in &result.contents {
                if object.key.to_lowercase().contains(&query_lower) {
                    files.push(Self::convert_object(object, &self.bucket));
                }
            }
        }

        Ok(files)
    }

    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile> {
        let bucket = self.get_bucket()?;

        let key = match parent_id {
            Some(p) => format!("{}/{}/", p.trim_matches('/'), name),
            None => format!("{}/", name),
        };

        // Create empty object with trailing slash
        bucket.put_object(&key, &[]).await?;

        Ok(CloudFile {
            id: key.clone(),
            name: name.to_string(),
            path: format!("s3://{}/{}", self.bucket, key),
            parent_id: parent_id.map(|s| s.to_string()),
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
        })
    }

    async fn upload_file(
        &self,
        local_path: &Path,
        parent_id: Option<&str>,
        name: Option<&str>,
        _progress: Option<ProgressCallback>,
    ) -> Result<CloudFile> {
        let bucket = self.get_bucket()?;

        let file_name = name.map(|s| s.to_string())
            .or_else(|| local_path.file_name().map(|n| n.to_string_lossy().to_string()))
            .ok_or_else(|| anyhow!("Could not determine file name"))?;

        let key = match parent_id {
            Some(p) => format!("{}/{}", p.trim_matches('/'), file_name),
            None => file_name.clone(),
        };

        let content = std::fs::read(local_path)?;
        let mime_type = mime_guess::from_path(local_path)
            .first_or_octet_stream()
            .to_string();

        bucket.put_object_with_content_type(&key, &content, &mime_type).await?;

        self.get_file(&key).await
    }

    async fn download_file(
        &self,
        file_id: &str,
        local_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let bucket = self.get_bucket()?;
        let key = file_id.trim_start_matches('/');

        let response = bucket.get_object(key).await?;
        std::fs::write(local_path, response.bytes())?;

        Ok(())
    }

    async fn move_file(&self, file_id: &str, new_parent_id: &str) -> Result<CloudFile> {
        // S3 doesn't support move - copy then delete
        let file = self.get_file(file_id).await?;
        let new_file = self.copy_file(file_id, new_parent_id, Some(&file.name)).await?;
        self.delete_file(file_id).await?;
        Ok(new_file)
    }

    async fn rename_file(&self, file_id: &str, new_name: &str) -> Result<CloudFile> {
        let file = self.get_file(file_id).await?;
        let parent = file.parent_id.unwrap_or_default();
        let new_file = self.copy_file(file_id, &parent, Some(new_name)).await?;
        self.delete_file(file_id).await?;
        Ok(new_file)
    }

    async fn copy_file(&self, file_id: &str, dest_parent_id: &str, new_name: Option<&str>) -> Result<CloudFile> {
        let bucket = self.get_bucket()?;
        let source_key = file_id.trim_start_matches('/');

        let file = self.get_file(file_id).await?;
        let name = new_name.unwrap_or(&file.name);

        let dest_key = format!("{}/{}", dest_parent_id.trim_matches('/'), name);

        bucket.copy_object_internal(source_key, &dest_key).await?;

        self.get_file(&dest_key).await
    }

    async fn delete_file(&self, file_id: &str) -> Result<()> {
        let bucket = self.get_bucket()?;
        let key = file_id.trim_start_matches('/');

        bucket.delete_object(key).await?;

        Ok(())
    }

    async fn permanent_delete(&self, file_id: &str) -> Result<()> {
        self.delete_file(file_id).await
    }

    async fn restore_file(&self, _file_id: &str) -> Result<CloudFile> {
        Err(anyhow!("S3 does not support trash/restore"))
    }

    async fn empty_trash(&self) -> Result<()> {
        Err(anyhow!("S3 does not support trash"))
    }

    async fn create_shared_link(
        &self,
        file_id: &str,
        expires_at: Option<DateTime<Utc>>,
        _password: Option<&str>,
    ) -> Result<SharedLink> {
        let bucket = self.get_bucket()?;
        let key = file_id.trim_start_matches('/');

        // Default to 7 days if no expiration specified
        let duration = if let Some(exp) = expires_at {
            (exp - Utc::now()).num_seconds() as u32
        } else {
            7 * 24 * 60 * 60 // 7 days
        };

        let url = bucket.presign_get(key, duration, None)?;

        Ok(SharedLink {
            url,
            expires_at,
            public: true,
            editable: false,
            password_protected: false,
            download_count: 0,
        })
    }

    async fn get_shared_link(&self, _file_id: &str) -> Result<Option<SharedLink>> {
        // S3 presigned URLs are generated on-demand
        Ok(None)
    }

    async fn revoke_shared_link(&self, _file_id: &str) -> Result<()> {
        // Presigned URLs can't be revoked - they expire automatically
        Ok(())
    }

    async fn get_versions(&self, file_id: &str) -> Result<Vec<FileVersion>> {
        let bucket = self.get_bucket()?;
        let key = file_id.trim_start_matches('/');

        let versions = bucket.list_object_versions(Some(key.to_string()), None, None, None).await?;

        Ok(versions.versions.iter().map(|v| FileVersion {
            id: v.version_id.clone().unwrap_or_default(),
            file_id: v.key.clone(),
            version: v.version_id.clone().unwrap_or_default(),
            size: v.size as u64,
            modified_at: DateTime::parse_from_rfc3339(&v.last_modified)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            modified_by: None,
        }).collect())
    }

    async fn restore_version(&self, file_id: &str, version_id: &str) -> Result<CloudFile> {
        let bucket = self.get_bucket()?;
        let key = file_id.trim_start_matches('/');

        // Copy the specific version to current
        let source = format!("{}?versionId={}", key, version_id);
        bucket.copy_object_internal(&source, key).await?;

        self.get_file(file_id).await
    }

    async fn get_changes(&self, _cursor: Option<&str>) -> Result<(Vec<CloudFile>, Option<String>)> {
        // S3 doesn't have native change tracking
        Ok((vec![], None))
    }
}
