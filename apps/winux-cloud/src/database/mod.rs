//! Database module for sync state and metadata storage
//!
//! Uses SQLite for persistent storage of:
//! - Sync state for each file
//! - Account credentials (encrypted)
//! - Activity log
//! - Configuration

mod schema;
mod queries;

pub use schema::*;
pub use queries::*;

use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

use crate::sync::{SyncState, FileSyncStatus, SyncEvent};
use crate::providers::{CloudAccount, ProviderType};

/// Database wrapper
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open or create database at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Create tables
        conn.execute_batch(SCHEMA)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open in-memory database (for testing)
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // === Sync State Methods ===

    /// Get sync state for a file by remote ID
    pub fn get_sync_state(&self, remote_id: &str) -> Result<Option<SyncState>> {
        let conn = self.conn.lock().unwrap();
        queries::get_sync_state(&conn, remote_id)
    }

    /// Get sync state for a file by local path
    pub fn get_sync_state_by_path(&self, local_path: &str) -> Result<Option<SyncState>> {
        let conn = self.conn.lock().unwrap();
        queries::get_sync_state_by_path(&conn, local_path)
    }

    /// Update or insert sync state
    pub fn update_sync_state(&self, state: &SyncState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::upsert_sync_state(&conn, state)
    }

    /// Update sync status for a file
    pub fn update_sync_status(&self, local_path: &str, status: FileSyncStatus) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::update_sync_status(&conn, local_path, status)
    }

    /// Mark file as deleted
    pub fn mark_deleted(&self, local_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::mark_deleted(&conn, local_path)
    }

    /// Update file path (for renames/moves)
    pub fn update_path(&self, old_path: &str, new_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::update_path(&conn, old_path, new_path)
    }

    /// Delete sync state
    pub fn delete_sync_state(&self, local_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::delete_sync_state(&conn, local_path)
    }

    /// Get all sync states
    pub fn get_all_sync_states(&self) -> Result<Vec<SyncState>> {
        let conn = self.conn.lock().unwrap();
        queries::get_all_sync_states(&conn)
    }

    /// Get pending uploads for a provider
    pub fn get_pending_uploads(&self, provider: &str) -> Result<Vec<SyncState>> {
        let conn = self.conn.lock().unwrap();
        queries::get_pending_uploads(&conn, provider)
    }

    /// Get pending downloads for a provider
    pub fn get_pending_downloads(&self, provider: &str) -> Result<Vec<SyncState>> {
        let conn = self.conn.lock().unwrap();
        queries::get_pending_downloads(&conn, provider)
    }

    /// Get files with conflicts
    pub fn get_conflicts(&self) -> Result<Vec<SyncState>> {
        let conn = self.conn.lock().unwrap();
        queries::get_conflicts(&conn)
    }

    // === Sync Cursor Methods ===

    /// Get sync cursor for a provider
    pub fn get_sync_cursor(&self, provider: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        queries::get_sync_cursor(&conn, provider)
    }

    /// Set sync cursor for a provider
    pub fn set_sync_cursor(&self, provider: &str, cursor: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::set_sync_cursor(&conn, provider, cursor)
    }

    // === Account Methods ===

    /// Get all accounts
    pub fn get_accounts(&self) -> Result<Vec<CloudAccount>> {
        let conn = self.conn.lock().unwrap();
        queries::get_accounts(&conn)
    }

    /// Get account by ID
    pub fn get_account(&self, id: &str) -> Result<Option<CloudAccount>> {
        let conn = self.conn.lock().unwrap();
        queries::get_account(&conn, id)
    }

    /// Save account
    pub fn save_account(&self, account: &CloudAccount) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::save_account(&conn, account)
    }

    /// Delete account
    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::delete_account(&conn, id)
    }

    // === Activity Log Methods ===

    /// Add activity event
    pub fn add_event(&self, event: &SyncEvent) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::add_event(&conn, event)
    }

    /// Get recent events
    pub fn get_recent_events(&self, limit: u32) -> Result<Vec<SyncEvent>> {
        let conn = self.conn.lock().unwrap();
        queries::get_recent_events(&conn, limit)
    }

    /// Get events for a specific date
    pub fn get_events_for_date(&self, date: &str) -> Result<Vec<SyncEvent>> {
        let conn = self.conn.lock().unwrap();
        queries::get_events_for_date(&conn, date)
    }

    /// Clear old events
    pub fn clear_old_events(&self, days: u32) -> Result<u64> {
        let conn = self.conn.lock().unwrap();
        queries::clear_old_events(&conn, days)
    }

    // === Configuration Methods ===

    /// Get configuration value
    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        queries::get_config(&conn, key)
    }

    /// Set configuration value
    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        queries::set_config(&conn, key, value)
    }

    // === Statistics Methods ===

    /// Get sync statistics
    pub fn get_stats(&self) -> Result<SyncStats> {
        let conn = self.conn.lock().unwrap();
        queries::get_stats(&conn)
    }
}

/// Sync statistics
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    pub total_files: u64,
    pub synced_files: u64,
    pub pending_files: u64,
    pub conflict_files: u64,
    pub error_files: u64,
    pub total_bytes: u64,
}
