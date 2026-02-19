//! Database schema definition

/// SQLite schema for the cloud sync database
pub const SCHEMA: &str = r#"
-- Sync state table - tracks sync status for each file
CREATE TABLE IF NOT EXISTS sync_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    local_path TEXT NOT NULL UNIQUE,
    remote_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    local_modified TEXT NOT NULL,
    remote_modified TEXT NOT NULL,
    local_hash TEXT,
    remote_hash TEXT,
    status TEXT NOT NULL DEFAULT 'synced',
    last_sync TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for fast lookups
CREATE INDEX IF NOT EXISTS idx_sync_state_remote_id ON sync_state(remote_id);
CREATE INDEX IF NOT EXISTS idx_sync_state_provider ON sync_state(provider);
CREATE INDEX IF NOT EXISTS idx_sync_state_status ON sync_state(status);
CREATE INDEX IF NOT EXISTS idx_sync_state_local_path ON sync_state(local_path);

-- Sync cursors - tracks delta sync position for each provider
CREATE TABLE IF NOT EXISTS sync_cursors (
    provider TEXT PRIMARY KEY,
    cursor TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Cloud accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    name TEXT NOT NULL,
    email TEXT,
    avatar_url TEXT,
    quota_total INTEGER,
    quota_used INTEGER,
    last_sync TEXT,
    sync_enabled INTEGER NOT NULL DEFAULT 1,
    credentials_encrypted TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_accounts_provider ON accounts(provider);

-- Activity log table
CREATE TABLE IF NOT EXISTS activity_log (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL,
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    bytes INTEGER,
    error TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_activity_timestamp ON activity_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_activity_event_type ON activity_log(event_type);
CREATE INDEX IF NOT EXISTS idx_activity_provider ON activity_log(provider);

-- Configuration table
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- File versions table (for local version tracking)
CREATE TABLE IF NOT EXISTS file_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    local_path TEXT NOT NULL,
    version_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    size INTEGER NOT NULL,
    hash TEXT,
    modified_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (local_path) REFERENCES sync_state(local_path) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_versions_local_path ON file_versions(local_path);

-- Shared links table
CREATE TABLE IF NOT EXISTS shared_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    local_path TEXT NOT NULL,
    remote_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    url TEXT NOT NULL,
    expires_at TEXT,
    password_protected INTEGER NOT NULL DEFAULT 0,
    download_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (local_path) REFERENCES sync_state(local_path) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_shared_links_local_path ON shared_links(local_path);

-- Conflicts table
CREATE TABLE IF NOT EXISTS conflicts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    local_path TEXT NOT NULL,
    remote_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    local_modified TEXT NOT NULL,
    remote_modified TEXT NOT NULL,
    local_size INTEGER NOT NULL,
    remote_size INTEGER NOT NULL,
    local_hash TEXT,
    remote_hash TEXT,
    resolution TEXT,
    resolved_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_conflicts_local_path ON conflicts(local_path);
CREATE INDEX IF NOT EXISTS idx_conflicts_resolved ON conflicts(resolution);

-- Selective sync folders table
CREATE TABLE IF NOT EXISTS selective_sync (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL,
    account_id TEXT NOT NULL,
    remote_path TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(provider, account_id, remote_path)
);

CREATE INDEX IF NOT EXISTS idx_selective_sync_account ON selective_sync(account_id);

-- Trigger to update updated_at on sync_state changes
CREATE TRIGGER IF NOT EXISTS update_sync_state_timestamp
    AFTER UPDATE ON sync_state
    FOR EACH ROW
BEGIN
    UPDATE sync_state SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Trigger to update updated_at on accounts changes
CREATE TRIGGER IF NOT EXISTS update_accounts_timestamp
    AFTER UPDATE ON accounts
    FOR EACH ROW
BEGIN
    UPDATE accounts SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Trigger to update updated_at on config changes
CREATE TRIGGER IF NOT EXISTS update_config_timestamp
    AFTER UPDATE ON config
    FOR EACH ROW
BEGIN
    UPDATE config SET updated_at = datetime('now') WHERE key = OLD.key;
END;
"#;
