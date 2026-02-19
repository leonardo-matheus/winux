//! Sync engine - Core synchronization logic

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::database::Database;
use crate::providers::CloudProvider;
use super::{ConflictResolver, ConflictStrategy, DeltaSync, FileWatcher, SyncState, FileSyncStatus, SyncEvent, SyncEventType};

/// Sync engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Local sync folder
    pub local_folder: PathBuf,
    /// Sync direction
    pub direction: SyncDirection,
    /// Conflict resolution strategy
    pub conflict_strategy: ConflictStrategy,
    /// Enable automatic sync
    pub auto_sync: bool,
    /// Sync interval in seconds
    pub sync_interval: u64,
    /// Maximum upload speed (bytes/sec, 0 = unlimited)
    pub max_upload_speed: u64,
    /// Maximum download speed (bytes/sec, 0 = unlimited)
    pub max_download_speed: u64,
    /// Ignored patterns
    pub ignore_patterns: Vec<String>,
    /// Only sync when on WiFi
    pub wifi_only: bool,
    /// Pause when on battery
    pub pause_on_battery: bool,
    /// Enable client-side encryption
    pub encryption_enabled: bool,
    /// Encryption key (if enabled)
    pub encryption_key: Option<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            local_folder: dirs::home_dir()
                .map(|h| h.join("Winux Cloud"))
                .unwrap_or_else(|| PathBuf::from("~/Winux Cloud")),
            direction: SyncDirection::Bidirectional,
            conflict_strategy: ConflictStrategy::KeepBoth,
            auto_sync: true,
            sync_interval: 300, // 5 minutes
            max_upload_speed: 0,
            max_download_speed: 0,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.temp".to_string(),
                "~*".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                ".git/**".to_string(),
                "node_modules/**".to_string(),
                "__pycache__/**".to_string(),
            ],
            wifi_only: false,
            pause_on_battery: true,
            encryption_enabled: false,
            encryption_key: None,
        }
    }
}

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    /// Upload only (local -> cloud)
    UploadOnly,
    /// Download only (cloud -> local)
    DownloadOnly,
    /// Bidirectional sync
    Bidirectional,
}

/// Current sync status
#[derive(Debug, Clone)]
pub struct SyncStatus {
    /// Whether sync is currently running
    pub is_syncing: bool,
    /// Whether sync is paused
    pub is_paused: bool,
    /// Current file being synced
    pub current_file: Option<String>,
    /// Current operation
    pub current_operation: Option<SyncOperation>,
    /// Progress (0-100)
    pub progress: u8,
    /// Bytes transferred in current operation
    pub bytes_transferred: u64,
    /// Total bytes in current operation
    pub bytes_total: u64,
    /// Transfer speed (bytes/sec)
    pub speed: u64,
    /// Files pending sync
    pub pending_count: u64,
    /// Files with conflicts
    pub conflict_count: u64,
    /// Files with errors
    pub error_count: u64,
    /// Last sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Last error message
    pub last_error: Option<String>,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            is_syncing: false,
            is_paused: false,
            current_file: None,
            current_operation: None,
            progress: 0,
            bytes_transferred: 0,
            bytes_total: 0,
            speed: 0,
            pending_count: 0,
            conflict_count: 0,
            error_count: 0,
            last_sync: None,
            last_error: None,
        }
    }
}

/// Current sync operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOperation {
    Upload,
    Download,
    Delete,
    Scanning,
}

/// Sync engine
pub struct SyncEngine {
    config: Arc<RwLock<SyncConfig>>,
    database: Arc<Database>,
    providers: Arc<RwLock<HashMap<String, Box<dyn CloudProvider>>>>,
    status: Arc<RwLock<SyncStatus>>,
    delta_sync: Arc<DeltaSync>,
    conflict_resolver: Arc<ConflictResolver>,
    watcher: Option<FileWatcher>,
    event_sender: mpsc::Sender<SyncEvent>,
    shutdown_sender: Option<mpsc::Sender<()>>,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(
        config: SyncConfig,
        database: Database,
        event_sender: mpsc::Sender<SyncEvent>,
    ) -> Self {
        let config = Arc::new(RwLock::new(config.clone()));
        let database = Arc::new(database);

        Self {
            config: config.clone(),
            database: database.clone(),
            providers: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(SyncStatus::default())),
            delta_sync: Arc::new(DeltaSync::new(database.clone())),
            conflict_resolver: Arc::new(ConflictResolver::new(config.blocking_read().conflict_strategy.clone())),
            watcher: None,
            event_sender,
            shutdown_sender: None,
        }
    }

    /// Add a cloud provider
    pub async fn add_provider(&self, name: &str, provider: Box<dyn CloudProvider>) {
        let mut providers = self.providers.write().await;
        providers.insert(name.to_string(), provider);
    }

    /// Remove a cloud provider
    pub async fn remove_provider(&self, name: &str) {
        let mut providers = self.providers.write().await;
        providers.remove(name);
    }

    /// Get current sync status
    pub async fn get_status(&self) -> SyncStatus {
        self.status.read().await.clone()
    }

    /// Start synchronization
    pub async fn start(&mut self) -> Result<()> {
        let config = self.config.read().await;

        // Start file watcher
        let (watcher, mut rx) = FileWatcher::new(&config.local_folder)?;
        self.watcher = Some(watcher);

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_sender = Some(shutdown_tx);

        // Clone for async tasks
        let status = self.status.clone();
        let providers = self.providers.clone();
        let database = self.database.clone();
        let delta_sync = self.delta_sync.clone();
        let conflict_resolver = self.conflict_resolver.clone();
        let event_sender = self.event_sender.clone();
        let sync_config = self.config.clone();

        // Spawn sync task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(sync_config.read().await.sync_interval)
            );

            loop {
                tokio::select! {
                    // Handle shutdown
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Sync engine shutting down");
                        break;
                    }

                    // Handle file system events
                    Some(event) = rx.recv() => {
                        tracing::debug!("File event: {:?}", event);
                        // Queue for sync
                        if let Err(e) = Self::handle_file_event(
                            &event,
                            &status,
                            &database,
                        ).await {
                            tracing::error!("Error handling file event: {}", e);
                        }
                    }

                    // Periodic sync
                    _ = interval.tick() => {
                        if let Err(e) = Self::run_sync(
                            &status,
                            &providers,
                            &database,
                            &delta_sync,
                            &conflict_resolver,
                            &event_sender,
                            &sync_config,
                        ).await {
                            tracing::error!("Sync error: {}", e);

                            let mut status = status.write().await;
                            status.last_error = Some(e.to_string());
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop synchronization
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(()).await;
        }

        self.watcher = None;

        let mut status = self.status.write().await;
        status.is_syncing = false;

        Ok(())
    }

    /// Pause synchronization
    pub async fn pause(&self) {
        let mut status = self.status.write().await;
        status.is_paused = true;
    }

    /// Resume synchronization
    pub async fn resume(&self) {
        let mut status = self.status.write().await;
        status.is_paused = false;
    }

    /// Force immediate sync
    pub async fn sync_now(&self) -> Result<()> {
        Self::run_sync(
            &self.status,
            &self.providers,
            &self.database,
            &self.delta_sync,
            &self.conflict_resolver,
            &self.event_sender,
            &self.config,
        ).await
    }

    /// Handle file system event
    async fn handle_file_event(
        event: &super::FileEvent,
        status: &Arc<RwLock<SyncStatus>>,
        database: &Arc<Database>,
    ) -> Result<()> {
        match event.kind {
            super::FileEventKind::Created | super::FileEventKind::Modified => {
                // Mark file as pending upload
                database.update_sync_status(&event.path.to_string_lossy(), FileSyncStatus::PendingUpload)?;

                let mut status = status.write().await;
                status.pending_count += 1;
            }
            super::FileEventKind::Deleted => {
                // Mark for remote deletion
                database.mark_deleted(&event.path.to_string_lossy())?;
            }
            super::FileEventKind::Renamed { ref from, to: _ } => {
                // Handle rename
                database.update_path(&from.to_string_lossy(), &event.path.to_string_lossy())?;
            }
        }

        Ok(())
    }

    /// Run synchronization
    async fn run_sync(
        status: &Arc<RwLock<SyncStatus>>,
        providers: &Arc<RwLock<HashMap<String, Box<dyn CloudProvider>>>>,
        database: &Arc<Database>,
        delta_sync: &Arc<DeltaSync>,
        conflict_resolver: &Arc<ConflictResolver>,
        event_sender: &mpsc::Sender<SyncEvent>,
        config: &Arc<RwLock<SyncConfig>>,
    ) -> Result<()> {
        // Check if paused
        {
            let status = status.read().await;
            if status.is_paused {
                return Ok(());
            }
        }

        // Set syncing status
        {
            let mut status = status.write().await;
            status.is_syncing = true;
            status.current_operation = Some(SyncOperation::Scanning);
        }

        let providers = providers.read().await;
        let sync_config = config.read().await;

        for (provider_name, provider) in providers.iter() {
            // Get delta changes
            let cursor = database.get_sync_cursor(provider_name)?;
            let (remote_changes, new_cursor) = provider.get_changes(cursor.as_deref()).await?;

            // Save new cursor
            if let Some(cursor) = new_cursor {
                database.set_sync_cursor(provider_name, &cursor)?;
            }

            // Process remote changes
            for remote_file in remote_changes {
                let local_state = database.get_sync_state(&remote_file.id)?;

                if let Some(state) = local_state {
                    // Check for conflicts
                    if state.local_modified > state.last_sync.unwrap_or(state.remote_modified) {
                        // Local was modified - potential conflict
                        if remote_file.modified_at > state.remote_modified {
                            // Both modified - conflict!
                            let resolution = conflict_resolver.resolve(&state, &remote_file)?;

                            // Apply resolution
                            match resolution {
                                super::ConflictResolution::KeepLocal => {
                                    // Upload local version
                                }
                                super::ConflictResolution::KeepRemote => {
                                    // Download remote version
                                }
                                super::ConflictResolution::KeepBoth { local_suffix, remote_suffix: _ } => {
                                    // Rename local and download remote
                                    let local_path = std::path::Path::new(&state.local_path);
                                    let new_name = format!(
                                        "{}_{}{}",
                                        local_path.file_stem().unwrap_or_default().to_string_lossy(),
                                        local_suffix,
                                        local_path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default()
                                    );
                                    let new_path = local_path.with_file_name(new_name);
                                    std::fs::rename(&state.local_path, &new_path)?;
                                }
                                super::ConflictResolution::AskUser => {
                                    // Mark as conflict for user resolution
                                    database.update_sync_status(&state.local_path, FileSyncStatus::Conflict)?;

                                    let mut status = status.write().await;
                                    status.conflict_count += 1;
                                    continue;
                                }
                            }

                            // Send event
                            let _ = event_sender.send(SyncEvent {
                                id: uuid::Uuid::new_v4().to_string(),
                                timestamp: Utc::now(),
                                event_type: SyncEventType::ConflictResolved,
                                path: state.local_path.clone(),
                                name: remote_file.name.clone(),
                                provider: provider_name.clone(),
                                bytes: None,
                                error: None,
                            }).await;
                        }
                    }
                }

                // Download if needed
                if sync_config.direction != SyncDirection::UploadOnly {
                    let local_path = sync_config.local_folder.join(&remote_file.path);

                    if remote_file.is_folder {
                        std::fs::create_dir_all(&local_path)?;
                    } else {
                        // Update status
                        {
                            let mut status = status.write().await;
                            status.current_file = Some(remote_file.name.clone());
                            status.current_operation = Some(SyncOperation::Download);
                            status.bytes_total = remote_file.size;
                            status.bytes_transferred = 0;
                        }

                        // Download file
                        provider.download_file(&remote_file.id, &local_path, None).await?;

                        // Update database
                        database.update_sync_state(&SyncState {
                            local_path: local_path.to_string_lossy().to_string(),
                            remote_id: remote_file.id.clone(),
                            provider: provider_name.clone(),
                            local_modified: remote_file.modified_at,
                            remote_modified: remote_file.modified_at,
                            local_hash: remote_file.hash.clone(),
                            remote_hash: remote_file.hash.clone(),
                            status: FileSyncStatus::Synced,
                            last_sync: Some(Utc::now()),
                            version: 1,
                        })?;

                        // Send event
                        let _ = event_sender.send(SyncEvent {
                            id: uuid::Uuid::new_v4().to_string(),
                            timestamp: Utc::now(),
                            event_type: SyncEventType::Download,
                            path: local_path.to_string_lossy().to_string(),
                            name: remote_file.name.clone(),
                            provider: provider_name.clone(),
                            bytes: Some(remote_file.size),
                            error: None,
                        }).await;
                    }
                }
            }

            // Upload pending files
            if sync_config.direction != SyncDirection::DownloadOnly {
                let pending_uploads = database.get_pending_uploads(provider_name)?;

                for state in pending_uploads {
                    let local_path = std::path::Path::new(&state.local_path);

                    if !local_path.exists() {
                        continue;
                    }

                    // Update status
                    {
                        let mut status = status.write().await;
                        status.current_file = Some(local_path.file_name().unwrap_or_default().to_string_lossy().to_string());
                        status.current_operation = Some(SyncOperation::Upload);
                        status.bytes_total = std::fs::metadata(local_path)?.len();
                        status.bytes_transferred = 0;
                    }

                    // Upload file
                    let parent_id = state.remote_id.split('/').rev().skip(1).next();
                    let uploaded = provider.upload_file(
                        local_path,
                        parent_id,
                        None,
                        None,
                    ).await?;

                    // Update database
                    database.update_sync_state(&SyncState {
                        local_path: state.local_path.clone(),
                        remote_id: uploaded.id.clone(),
                        provider: provider_name.clone(),
                        local_modified: uploaded.modified_at,
                        remote_modified: uploaded.modified_at,
                        local_hash: uploaded.hash.clone(),
                        remote_hash: uploaded.hash.clone(),
                        status: FileSyncStatus::Synced,
                        last_sync: Some(Utc::now()),
                        version: state.version + 1,
                    })?;

                    // Send event
                    let _ = event_sender.send(SyncEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        event_type: SyncEventType::Upload,
                        path: state.local_path.clone(),
                        name: uploaded.name,
                        provider: provider_name.clone(),
                        bytes: Some(uploaded.size),
                        error: None,
                    }).await;
                }
            }
        }

        // Update status
        {
            let mut status = status.write().await;
            status.is_syncing = false;
            status.current_file = None;
            status.current_operation = None;
            status.last_sync = Some(Utc::now());
            status.pending_count = 0;
        }

        Ok(())
    }

    /// Get sync configuration
    pub async fn get_config(&self) -> SyncConfig {
        self.config.read().await.clone()
    }

    /// Update sync configuration
    pub async fn set_config(&self, config: SyncConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }
}
