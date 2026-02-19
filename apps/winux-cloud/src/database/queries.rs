//! Database query implementations

use anyhow::Result;
use rusqlite::{Connection, params, OptionalExtension};
use chrono::{DateTime, Utc};

use crate::sync::{SyncState, FileSyncStatus, SyncEvent, SyncEventType};
use crate::providers::{CloudAccount, ProviderType, StorageQuota};
use super::SyncStats;

// === Sync State Queries ===

/// Get sync state by remote ID
pub fn get_sync_state(conn: &Connection, remote_id: &str) -> Result<Option<SyncState>> {
    let mut stmt = conn.prepare(
        "SELECT local_path, remote_id, provider, local_modified, remote_modified,
                local_hash, remote_hash, status, last_sync, version
         FROM sync_state WHERE remote_id = ? AND is_deleted = 0"
    )?;

    let result = stmt.query_row(params![remote_id], |row| {
        Ok(SyncState {
            local_path: row.get(0)?,
            remote_id: row.get(1)?,
            provider: row.get(2)?,
            local_modified: parse_datetime(&row.get::<_, String>(3)?),
            remote_modified: parse_datetime(&row.get::<_, String>(4)?),
            local_hash: row.get(5)?,
            remote_hash: row.get(6)?,
            status: parse_status(&row.get::<_, String>(7)?),
            last_sync: row.get::<_, Option<String>>(8)?.map(|s| parse_datetime(&s)),
            version: row.get(9)?,
        })
    }).optional()?;

    Ok(result)
}

/// Get sync state by local path
pub fn get_sync_state_by_path(conn: &Connection, local_path: &str) -> Result<Option<SyncState>> {
    let mut stmt = conn.prepare(
        "SELECT local_path, remote_id, provider, local_modified, remote_modified,
                local_hash, remote_hash, status, last_sync, version
         FROM sync_state WHERE local_path = ? AND is_deleted = 0"
    )?;

    let result = stmt.query_row(params![local_path], |row| {
        Ok(SyncState {
            local_path: row.get(0)?,
            remote_id: row.get(1)?,
            provider: row.get(2)?,
            local_modified: parse_datetime(&row.get::<_, String>(3)?),
            remote_modified: parse_datetime(&row.get::<_, String>(4)?),
            local_hash: row.get(5)?,
            remote_hash: row.get(6)?,
            status: parse_status(&row.get::<_, String>(7)?),
            last_sync: row.get::<_, Option<String>>(8)?.map(|s| parse_datetime(&s)),
            version: row.get(9)?,
        })
    }).optional()?;

    Ok(result)
}

/// Insert or update sync state
pub fn upsert_sync_state(conn: &Connection, state: &SyncState) -> Result<()> {
    conn.execute(
        "INSERT INTO sync_state (local_path, remote_id, provider, local_modified, remote_modified,
                                 local_hash, remote_hash, status, last_sync, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(local_path) DO UPDATE SET
             remote_id = excluded.remote_id,
             provider = excluded.provider,
             local_modified = excluded.local_modified,
             remote_modified = excluded.remote_modified,
             local_hash = excluded.local_hash,
             remote_hash = excluded.remote_hash,
             status = excluded.status,
             last_sync = excluded.last_sync,
             version = excluded.version,
             is_deleted = 0",
        params![
            state.local_path,
            state.remote_id,
            state.provider,
            state.local_modified.to_rfc3339(),
            state.remote_modified.to_rfc3339(),
            state.local_hash,
            state.remote_hash,
            format_status(state.status),
            state.last_sync.map(|dt| dt.to_rfc3339()),
            state.version,
        ],
    )?;

    Ok(())
}

/// Update sync status for a file
pub fn update_sync_status(conn: &Connection, local_path: &str, status: FileSyncStatus) -> Result<()> {
    conn.execute(
        "UPDATE sync_state SET status = ? WHERE local_path = ?",
        params![format_status(status), local_path],
    )?;

    Ok(())
}

/// Mark file as deleted
pub fn mark_deleted(conn: &Connection, local_path: &str) -> Result<()> {
    conn.execute(
        "UPDATE sync_state SET is_deleted = 1 WHERE local_path = ?",
        params![local_path],
    )?;

    Ok(())
}

/// Update file path
pub fn update_path(conn: &Connection, old_path: &str, new_path: &str) -> Result<()> {
    conn.execute(
        "UPDATE sync_state SET local_path = ? WHERE local_path = ?",
        params![new_path, old_path],
    )?;

    Ok(())
}

/// Delete sync state
pub fn delete_sync_state(conn: &Connection, local_path: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM sync_state WHERE local_path = ?",
        params![local_path],
    )?;

    Ok(())
}

/// Get all sync states
pub fn get_all_sync_states(conn: &Connection) -> Result<Vec<SyncState>> {
    let mut stmt = conn.prepare(
        "SELECT local_path, remote_id, provider, local_modified, remote_modified,
                local_hash, remote_hash, status, last_sync, version
         FROM sync_state WHERE is_deleted = 0"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SyncState {
            local_path: row.get(0)?,
            remote_id: row.get(1)?,
            provider: row.get(2)?,
            local_modified: parse_datetime(&row.get::<_, String>(3)?),
            remote_modified: parse_datetime(&row.get::<_, String>(4)?),
            local_hash: row.get(5)?,
            remote_hash: row.get(6)?,
            status: parse_status(&row.get::<_, String>(7)?),
            last_sync: row.get::<_, Option<String>>(8)?.map(|s| parse_datetime(&s)),
            version: row.get(9)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

/// Get pending uploads
pub fn get_pending_uploads(conn: &Connection, provider: &str) -> Result<Vec<SyncState>> {
    let mut stmt = conn.prepare(
        "SELECT local_path, remote_id, provider, local_modified, remote_modified,
                local_hash, remote_hash, status, last_sync, version
         FROM sync_state WHERE provider = ? AND status = 'pending_upload' AND is_deleted = 0"
    )?;

    let rows = stmt.query_map(params![provider], |row| {
        Ok(SyncState {
            local_path: row.get(0)?,
            remote_id: row.get(1)?,
            provider: row.get(2)?,
            local_modified: parse_datetime(&row.get::<_, String>(3)?),
            remote_modified: parse_datetime(&row.get::<_, String>(4)?),
            local_hash: row.get(5)?,
            remote_hash: row.get(6)?,
            status: parse_status(&row.get::<_, String>(7)?),
            last_sync: row.get::<_, Option<String>>(8)?.map(|s| parse_datetime(&s)),
            version: row.get(9)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

/// Get pending downloads
pub fn get_pending_downloads(conn: &Connection, provider: &str) -> Result<Vec<SyncState>> {
    let mut stmt = conn.prepare(
        "SELECT local_path, remote_id, provider, local_modified, remote_modified,
                local_hash, remote_hash, status, last_sync, version
         FROM sync_state WHERE provider = ? AND status = 'pending_download' AND is_deleted = 0"
    )?;

    let rows = stmt.query_map(params![provider], |row| {
        Ok(SyncState {
            local_path: row.get(0)?,
            remote_id: row.get(1)?,
            provider: row.get(2)?,
            local_modified: parse_datetime(&row.get::<_, String>(3)?),
            remote_modified: parse_datetime(&row.get::<_, String>(4)?),
            local_hash: row.get(5)?,
            remote_hash: row.get(6)?,
            status: parse_status(&row.get::<_, String>(7)?),
            last_sync: row.get::<_, Option<String>>(8)?.map(|s| parse_datetime(&s)),
            version: row.get(9)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

/// Get conflicts
pub fn get_conflicts(conn: &Connection) -> Result<Vec<SyncState>> {
    let mut stmt = conn.prepare(
        "SELECT local_path, remote_id, provider, local_modified, remote_modified,
                local_hash, remote_hash, status, last_sync, version
         FROM sync_state WHERE status = 'conflict' AND is_deleted = 0"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SyncState {
            local_path: row.get(0)?,
            remote_id: row.get(1)?,
            provider: row.get(2)?,
            local_modified: parse_datetime(&row.get::<_, String>(3)?),
            remote_modified: parse_datetime(&row.get::<_, String>(4)?),
            local_hash: row.get(5)?,
            remote_hash: row.get(6)?,
            status: parse_status(&row.get::<_, String>(7)?),
            last_sync: row.get::<_, Option<String>>(8)?.map(|s| parse_datetime(&s)),
            version: row.get(9)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

// === Sync Cursor Queries ===

pub fn get_sync_cursor(conn: &Connection, provider: &str) -> Result<Option<String>> {
    let result: Option<String> = conn.query_row(
        "SELECT cursor FROM sync_cursors WHERE provider = ?",
        params![provider],
        |row| row.get(0),
    ).optional()?;

    Ok(result)
}

pub fn set_sync_cursor(conn: &Connection, provider: &str, cursor: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO sync_cursors (provider, cursor) VALUES (?1, ?2)
         ON CONFLICT(provider) DO UPDATE SET cursor = excluded.cursor",
        params![provider, cursor],
    )?;

    Ok(())
}

// === Account Queries ===

pub fn get_accounts(conn: &Connection) -> Result<Vec<CloudAccount>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider, name, email, avatar_url, quota_total, quota_used, last_sync, sync_enabled
         FROM accounts"
    )?;

    let rows = stmt.query_map([], |row| {
        let quota_total: Option<i64> = row.get(5)?;
        let quota_used: Option<i64> = row.get(6)?;

        Ok(CloudAccount {
            id: row.get(0)?,
            provider: parse_provider(&row.get::<_, String>(1)?),
            name: row.get(2)?,
            email: row.get(3)?,
            avatar_url: row.get(4)?,
            quota: quota_total.map(|total| StorageQuota {
                total: total as u64,
                used: quota_used.unwrap_or(0) as u64,
                available: (total - quota_used.unwrap_or(0)) as u64,
                trash: None,
            }),
            last_sync: row.get::<_, Option<String>>(7)?.map(|s| parse_datetime(&s)),
            sync_enabled: row.get::<_, i32>(8)? != 0,
            credentials: None,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

pub fn get_account(conn: &Connection, id: &str) -> Result<Option<CloudAccount>> {
    let result = conn.query_row(
        "SELECT id, provider, name, email, avatar_url, quota_total, quota_used, last_sync, sync_enabled
         FROM accounts WHERE id = ?",
        params![id],
        |row| {
            let quota_total: Option<i64> = row.get(5)?;
            let quota_used: Option<i64> = row.get(6)?;

            Ok(CloudAccount {
                id: row.get(0)?,
                provider: parse_provider(&row.get::<_, String>(1)?),
                name: row.get(2)?,
                email: row.get(3)?,
                avatar_url: row.get(4)?,
                quota: quota_total.map(|total| StorageQuota {
                    total: total as u64,
                    used: quota_used.unwrap_or(0) as u64,
                    available: (total - quota_used.unwrap_or(0)) as u64,
                    trash: None,
                }),
                last_sync: row.get::<_, Option<String>>(7)?.map(|s| parse_datetime(&s)),
                sync_enabled: row.get::<_, i32>(8)? != 0,
                credentials: None,
            })
        },
    ).optional()?;

    Ok(result)
}

pub fn save_account(conn: &Connection, account: &CloudAccount) -> Result<()> {
    let (quota_total, quota_used) = account.quota.as_ref()
        .map(|q| (Some(q.total as i64), Some(q.used as i64)))
        .unwrap_or((None, None));

    conn.execute(
        "INSERT INTO accounts (id, provider, name, email, avatar_url, quota_total, quota_used, last_sync, sync_enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             email = excluded.email,
             avatar_url = excluded.avatar_url,
             quota_total = excluded.quota_total,
             quota_used = excluded.quota_used,
             last_sync = excluded.last_sync,
             sync_enabled = excluded.sync_enabled",
        params![
            account.id,
            format_provider(account.provider),
            account.name,
            account.email,
            account.avatar_url,
            quota_total,
            quota_used,
            account.last_sync.map(|dt| dt.to_rfc3339()),
            if account.sync_enabled { 1 } else { 0 },
        ],
    )?;

    Ok(())
}

pub fn delete_account(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM accounts WHERE id = ?", params![id])?;
    Ok(())
}

// === Activity Log Queries ===

pub fn add_event(conn: &Connection, event: &SyncEvent) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_log (id, timestamp, event_type, path, name, provider, bytes, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            event.id,
            event.timestamp.to_rfc3339(),
            format_event_type(event.event_type),
            event.path,
            event.name,
            event.provider,
            event.bytes.map(|b| b as i64),
            event.error,
        ],
    )?;

    Ok(())
}

pub fn get_recent_events(conn: &Connection, limit: u32) -> Result<Vec<SyncEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, event_type, path, name, provider, bytes, error
         FROM activity_log ORDER BY timestamp DESC LIMIT ?"
    )?;

    let rows = stmt.query_map(params![limit], |row| {
        Ok(SyncEvent {
            id: row.get(0)?,
            timestamp: parse_datetime(&row.get::<_, String>(1)?),
            event_type: parse_event_type(&row.get::<_, String>(2)?),
            path: row.get(3)?,
            name: row.get(4)?,
            provider: row.get(5)?,
            bytes: row.get::<_, Option<i64>>(6)?.map(|b| b as u64),
            error: row.get(7)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

pub fn get_events_for_date(conn: &Connection, date: &str) -> Result<Vec<SyncEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, event_type, path, name, provider, bytes, error
         FROM activity_log WHERE date(timestamp) = ? ORDER BY timestamp DESC"
    )?;

    let rows = stmt.query_map(params![date], |row| {
        Ok(SyncEvent {
            id: row.get(0)?,
            timestamp: parse_datetime(&row.get::<_, String>(1)?),
            event_type: parse_event_type(&row.get::<_, String>(2)?),
            path: row.get(3)?,
            name: row.get(4)?,
            provider: row.get(5)?,
            bytes: row.get::<_, Option<i64>>(6)?.map(|b| b as u64),
            error: row.get(7)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

pub fn clear_old_events(conn: &Connection, days: u32) -> Result<u64> {
    let changes = conn.execute(
        "DELETE FROM activity_log WHERE timestamp < datetime('now', ?)",
        params![format!("-{} days", days)],
    )?;

    Ok(changes as u64)
}

// === Config Queries ===

pub fn get_config(conn: &Connection, key: &str) -> Result<Option<String>> {
    let result: Option<String> = conn.query_row(
        "SELECT value FROM config WHERE key = ?",
        params![key],
        |row| row.get(0),
    ).optional()?;

    Ok(result)
}

pub fn set_config(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO config (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;

    Ok(())
}

// === Stats Queries ===

pub fn get_stats(conn: &Connection) -> Result<SyncStats> {
    let total_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sync_state WHERE is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    let synced_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sync_state WHERE status = 'synced' AND is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    let pending_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sync_state WHERE status IN ('pending_upload', 'pending_download') AND is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    let conflict_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sync_state WHERE status = 'conflict' AND is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    let error_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sync_state WHERE status = 'error' AND is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    Ok(SyncStats {
        total_files: total_files as u64,
        synced_files: synced_files as u64,
        pending_files: pending_files as u64,
        conflict_files: conflict_files as u64,
        error_files: error_files as u64,
        total_bytes: 0, // Would need to track this separately
    })
}

// === Helper Functions ===

fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_status(s: &str) -> FileSyncStatus {
    match s {
        "synced" => FileSyncStatus::Synced,
        "pending_upload" => FileSyncStatus::PendingUpload,
        "pending_download" => FileSyncStatus::PendingDownload,
        "syncing" => FileSyncStatus::Syncing,
        "conflict" => FileSyncStatus::Conflict,
        "error" => FileSyncStatus::Error,
        "ignored" => FileSyncStatus::Ignored,
        _ => FileSyncStatus::Synced,
    }
}

fn format_status(status: FileSyncStatus) -> &'static str {
    match status {
        FileSyncStatus::Synced => "synced",
        FileSyncStatus::PendingUpload => "pending_upload",
        FileSyncStatus::PendingDownload => "pending_download",
        FileSyncStatus::Syncing => "syncing",
        FileSyncStatus::Conflict => "conflict",
        FileSyncStatus::Error => "error",
        FileSyncStatus::Ignored => "ignored",
    }
}

fn parse_provider(s: &str) -> ProviderType {
    match s {
        "google_drive" => ProviderType::GoogleDrive,
        "onedrive" => ProviderType::OneDrive,
        "dropbox" => ProviderType::Dropbox,
        "nextcloud" => ProviderType::Nextcloud,
        "webdav" => ProviderType::WebDav,
        "s3" => ProviderType::S3,
        _ => ProviderType::WebDav,
    }
}

fn format_provider(provider: ProviderType) -> &'static str {
    match provider {
        ProviderType::GoogleDrive => "google_drive",
        ProviderType::OneDrive => "onedrive",
        ProviderType::Dropbox => "dropbox",
        ProviderType::Nextcloud => "nextcloud",
        ProviderType::WebDav => "webdav",
        ProviderType::S3 => "s3",
    }
}

fn parse_event_type(s: &str) -> SyncEventType {
    match s {
        "upload" => SyncEventType::Upload,
        "download" => SyncEventType::Download,
        "delete" => SyncEventType::Delete,
        "move" => SyncEventType::Move,
        "rename" => SyncEventType::Rename,
        "conflict_resolved" => SyncEventType::ConflictResolved,
        "error" => SyncEventType::Error,
        _ => SyncEventType::Error,
    }
}

fn format_event_type(event_type: SyncEventType) -> &'static str {
    match event_type {
        SyncEventType::Upload => "upload",
        SyncEventType::Download => "download",
        SyncEventType::Delete => "delete",
        SyncEventType::Move => "move",
        SyncEventType::Rename => "rename",
        SyncEventType::ConflictResolved => "conflict_resolved",
        SyncEventType::Error => "error",
    }
}
