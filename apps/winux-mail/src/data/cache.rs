// Winux Mail - Local Cache (SQLite)
// Copyright (c) 2026 Winux OS Project

use crate::data::folder::Folder;
use crate::data::message::{Attachment, Message, MessageFlags};

use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use std::path::PathBuf;
use std::sync::Mutex;

/// Local email cache using SQLite
pub struct EmailCache {
    conn: Mutex<Connection>,
}

impl EmailCache {
    /// Create or open the email cache database
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Could not find cache directory"))?
            .join("winux-mail");

        std::fs::create_dir_all(&cache_dir)?;

        let db_path = cache_dir.join("mail.db");
        let conn = Connection::open(db_path)?;

        let cache = Self {
            conn: Mutex::new(conn),
        };

        cache.init_schema()?;

        Ok(cache)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            -- Accounts metadata
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                name TEXT NOT NULL,
                last_sync INTEGER
            );

            -- Folders
            CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                folder_type INTEGER NOT NULL,
                unread_count INTEGER DEFAULT 0,
                total_count INTEGER DEFAULT 0,
                uid_validity INTEGER,
                uid_next INTEGER,
                FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_folders_account ON folders(account_id);

            -- Messages
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                uid INTEGER,
                account_id TEXT NOT NULL,
                folder_id TEXT NOT NULL,
                message_id TEXT,
                subject TEXT NOT NULL,
                from_addr TEXT NOT NULL,
                to_addrs TEXT NOT NULL,
                cc_addrs TEXT,
                bcc_addrs TEXT,
                date INTEGER NOT NULL,
                preview TEXT,
                text_body TEXT,
                html_body TEXT,
                seen INTEGER DEFAULT 0,
                answered INTEGER DEFAULT 0,
                flagged INTEGER DEFAULT 0,
                deleted INTEGER DEFAULT 0,
                draft INTEGER DEFAULT 0,
                starred INTEGER DEFAULT 0,
                labels TEXT,
                in_reply_to TEXT,
                refs TEXT,
                has_attachments INTEGER DEFAULT 0,
                FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
                FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_account ON messages(account_id);
            CREATE INDEX IF NOT EXISTS idx_messages_folder ON messages(folder_id);
            CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date DESC);
            CREATE INDEX IF NOT EXISTS idx_messages_uid ON messages(account_id, folder_id, uid);

            -- Attachments
            CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                data BLOB,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_attachments_message ON attachments(message_id);

            -- Full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                subject,
                from_addr,
                text_body,
                content='messages',
                content_rowid='rowid'
            );

            -- Triggers to keep FTS in sync
            CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(rowid, subject, from_addr, text_body)
                VALUES (NEW.rowid, NEW.subject, NEW.from_addr, NEW.text_body);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, subject, from_addr, text_body)
                VALUES('delete', OLD.rowid, OLD.subject, OLD.from_addr, OLD.text_body);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, subject, from_addr, text_body)
                VALUES('delete', OLD.rowid, OLD.subject, OLD.from_addr, OLD.text_body);
                INSERT INTO messages_fts(rowid, subject, from_addr, text_body)
                VALUES (NEW.rowid, NEW.subject, NEW.from_addr, NEW.text_body);
            END;

            -- Sync state
            CREATE TABLE IF NOT EXISTS sync_state (
                account_id TEXT NOT NULL,
                folder_id TEXT NOT NULL,
                last_uid INTEGER,
                last_sync INTEGER,
                PRIMARY KEY (account_id, folder_id),
                FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
                FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
            );
            "#,
        )?;

        Ok(())
    }

    // ==================== Account Operations ====================

    /// Save account metadata
    pub fn save_account(&self, id: &str, email: &str, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO accounts (id, email, name) VALUES (?, ?, ?)",
            params![id, email, name],
        )?;

        Ok(())
    }

    /// Delete account and all associated data
    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute("DELETE FROM accounts WHERE id = ?", params![id])?;

        Ok(())
    }

    /// Update last sync time
    pub fn update_account_sync(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();

        conn.execute(
            "UPDATE accounts SET last_sync = ? WHERE id = ?",
            params![now, id],
        )?;

        Ok(())
    }

    // ==================== Folder Operations ====================

    /// Save folders for an account
    pub fn save_folders(&self, account_id: &str, folders: &[Folder]) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        for folder in folders {
            conn.execute(
                r#"
                INSERT OR REPLACE INTO folders
                (id, account_id, name, path, folder_type, unread_count, total_count)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    folder.id,
                    account_id,
                    folder.name,
                    folder.path,
                    folder.folder_type as i32,
                    folder.unread_count,
                    folder.total_count,
                ],
            )?;
        }

        Ok(())
    }

    /// Get folders for an account
    pub fn get_folders(&self, account_id: &str) -> Result<Vec<Folder>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, path, folder_type, unread_count, total_count
            FROM folders
            WHERE account_id = ?
            ORDER BY folder_type, name
            "#,
        )?;

        let folders = stmt.query_map(params![account_id], |row| {
            let folder_type_int: i32 = row.get(3)?;
            let folder_type = match folder_type_int {
                0 => crate::data::folder::FolderType::Inbox,
                1 => crate::data::folder::FolderType::Sent,
                2 => crate::data::folder::FolderType::Drafts,
                3 => crate::data::folder::FolderType::Trash,
                4 => crate::data::folder::FolderType::Spam,
                5 => crate::data::folder::FolderType::Archive,
                6 => crate::data::folder::FolderType::Starred,
                7 => crate::data::folder::FolderType::All,
                _ => crate::data::folder::FolderType::Custom,
            };

            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                folder_type,
                unread_count: row.get(4)?,
                total_count: row.get(5)?,
                parent_id: None,
                delimiter: None,
                selectable: true,
            })
        })?;

        folders.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Update folder counts
    pub fn update_folder_counts(&self, folder_id: &str, unread: u32, total: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE folders SET unread_count = ?, total_count = ? WHERE id = ?",
            params![unread, total, folder_id],
        )?;

        Ok(())
    }

    // ==================== Message Operations ====================

    /// Save a message
    pub fn save_message(&self, message: &Message) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let to_json = serde_json::to_string(&message.to)?;
        let cc_json = serde_json::to_string(&message.cc)?;
        let bcc_json = serde_json::to_string(&message.bcc)?;
        let labels_json = serde_json::to_string(&message.labels)?;
        let refs_json = serde_json::to_string(&message.references)?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO messages
            (id, uid, account_id, folder_id, message_id, subject, from_addr, to_addrs,
             cc_addrs, bcc_addrs, date, preview, text_body, html_body,
             seen, answered, flagged, deleted, draft, starred, labels,
             in_reply_to, refs, has_attachments)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                message.id,
                message.uid,
                message.account_id,
                message.folder,
                message.id,
                message.subject,
                message.from,
                to_json,
                cc_json,
                bcc_json,
                message.date.timestamp(),
                message.preview,
                message.text_body,
                message.html_body,
                message.flags.seen,
                message.flags.answered,
                message.flags.flagged,
                message.flags.deleted,
                message.flags.draft,
                message.starred,
                labels_json,
                message.in_reply_to,
                refs_json,
                !message.attachments.is_empty(),
            ],
        )?;

        // Save attachments
        for attachment in &message.attachments {
            conn.execute(
                r#"
                INSERT OR REPLACE INTO attachments
                (id, message_id, filename, mime_type, size, data)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
                params![
                    attachment.id,
                    message.id,
                    attachment.filename,
                    attachment.mime_type,
                    attachment.size,
                    attachment.data,
                ],
            )?;
        }

        Ok(())
    }

    /// Get message by ID
    pub fn get_message(&self, id: &str) -> Result<Option<Message>> {
        let conn = self.conn.lock().unwrap();

        let message = conn
            .query_row(
                r#"
                SELECT id, uid, account_id, folder_id, subject, from_addr, to_addrs,
                       cc_addrs, bcc_addrs, date, preview, text_body, html_body,
                       seen, answered, flagged, deleted, draft, starred, labels,
                       in_reply_to, refs
                FROM messages
                WHERE id = ?
                "#,
                params![id],
                |row| {
                    Ok(Self::row_to_message(row)?)
                },
            )
            .optional()?;

        if let Some(mut msg) = message {
            // Load attachments
            let mut stmt = conn.prepare(
                "SELECT id, filename, mime_type, size FROM attachments WHERE message_id = ?",
            )?;

            let attachments = stmt.query_map(params![id], |row| {
                Ok(Attachment {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    mime_type: row.get(2)?,
                    size: row.get(3)?,
                    data: None,
                })
            })?;

            msg.attachments = attachments.collect::<Result<Vec<_>, _>>()?;

            Ok(Some(msg))
        } else {
            Ok(None)
        }
    }

    /// Get messages for a folder
    pub fn get_messages_for_folder(
        &self,
        account_id: &str,
        folder_path: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT m.id, m.uid, m.account_id, m.folder_id, m.subject, m.from_addr, m.to_addrs,
                   m.cc_addrs, m.bcc_addrs, m.date, m.preview, m.text_body, m.html_body,
                   m.seen, m.answered, m.flagged, m.deleted, m.draft, m.starred, m.labels,
                   m.in_reply_to, m.refs, m.has_attachments
            FROM messages m
            JOIN folders f ON m.folder_id = f.id
            WHERE m.account_id = ? AND f.path = ?
            ORDER BY m.date DESC
            LIMIT ? OFFSET ?
            "#,
        )?;

        let messages = stmt.query_map(params![account_id, folder_path, limit, offset], |row| {
            Self::row_to_message(row)
        })?;

        messages.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get unread messages count
    pub fn get_unread_count(&self, account_id: &str, folder_path: &str) -> Result<u32> {
        let conn = self.conn.lock().unwrap();

        let count: u32 = conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM messages m
            JOIN folders f ON m.folder_id = f.id
            WHERE m.account_id = ? AND f.path = ? AND m.seen = 0
            "#,
            params![account_id, folder_path],
            |row| row.get(0),
        )?;

        Ok(count)
    }

    /// Search messages
    pub fn search(&self, account_id: &str, query: &str, limit: u32) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT m.id, m.uid, m.account_id, m.folder_id, m.subject, m.from_addr, m.to_addrs,
                   m.cc_addrs, m.bcc_addrs, m.date, m.preview, m.text_body, m.html_body,
                   m.seen, m.answered, m.flagged, m.deleted, m.draft, m.starred, m.labels,
                   m.in_reply_to, m.refs, m.has_attachments
            FROM messages m
            JOIN messages_fts fts ON m.rowid = fts.rowid
            WHERE m.account_id = ? AND messages_fts MATCH ?
            ORDER BY m.date DESC
            LIMIT ?
            "#,
        )?;

        let messages = stmt.query_map(params![account_id, query, limit], |row| {
            Self::row_to_message(row)
        })?;

        messages.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Update message flags
    pub fn update_flags(&self, message_id: &str, flags: &MessageFlags) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            UPDATE messages
            SET seen = ?, answered = ?, flagged = ?, deleted = ?, draft = ?
            WHERE id = ?
            "#,
            params![
                flags.seen,
                flags.answered,
                flags.flagged,
                flags.deleted,
                flags.draft,
                message_id,
            ],
        )?;

        Ok(())
    }

    /// Mark message as read/unread
    pub fn mark_read(&self, message_id: &str, read: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE messages SET seen = ? WHERE id = ?",
            params![read, message_id],
        )?;

        Ok(())
    }

    /// Star/unstar message
    pub fn set_starred(&self, message_id: &str, starred: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE messages SET starred = ?, flagged = ? WHERE id = ?",
            params![starred, starred, message_id],
        )?;

        Ok(())
    }

    /// Delete message from cache
    pub fn delete_message(&self, message_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute("DELETE FROM messages WHERE id = ?", params![message_id])?;

        Ok(())
    }

    /// Move message to different folder
    pub fn move_message(&self, message_id: &str, new_folder_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE messages SET folder_id = ? WHERE id = ?",
            params![new_folder_id, message_id],
        )?;

        Ok(())
    }

    /// Get attachment data
    pub fn get_attachment_data(&self, attachment_id: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.conn.lock().unwrap();

        let data: Option<Vec<u8>> = conn
            .query_row(
                "SELECT data FROM attachments WHERE id = ?",
                params![attachment_id],
                |row| row.get(0),
            )
            .optional()?;

        Ok(data)
    }

    // ==================== Sync State ====================

    /// Get last synced UID for a folder
    pub fn get_last_uid(&self, account_id: &str, folder_id: &str) -> Result<Option<u32>> {
        let conn = self.conn.lock().unwrap();

        let uid: Option<u32> = conn
            .query_row(
                "SELECT last_uid FROM sync_state WHERE account_id = ? AND folder_id = ?",
                params![account_id, folder_id],
                |row| row.get(0),
            )
            .optional()?;

        Ok(uid)
    }

    /// Update sync state
    pub fn update_sync_state(&self, account_id: &str, folder_id: &str, last_uid: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO sync_state (account_id, folder_id, last_uid, last_sync)
            VALUES (?, ?, ?, ?)
            "#,
            params![account_id, folder_id, last_uid, now],
        )?;

        Ok(())
    }

    /// Clear all cached data for an account
    pub fn clear_account_cache(&self, account_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "DELETE FROM messages WHERE account_id = ?",
            params![account_id],
        )?;

        conn.execute(
            "DELETE FROM folders WHERE account_id = ?",
            params![account_id],
        )?;

        conn.execute(
            "DELETE FROM sync_state WHERE account_id = ?",
            params![account_id],
        )?;

        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> Result<CacheStats> {
        let conn = self.conn.lock().unwrap();

        let message_count: u32 = conn.query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))?;
        let attachment_count: u32 = conn.query_row("SELECT COUNT(*) FROM attachments", [], |row| row.get(0))?;
        let folder_count: u32 = conn.query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0))?;

        // Get database size
        let db_path = dirs::cache_dir()
            .unwrap_or_default()
            .join("winux-mail")
            .join("mail.db");

        let db_size = std::fs::metadata(&db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(CacheStats {
            message_count,
            attachment_count,
            folder_count,
            database_size: db_size,
        })
    }

    // ==================== Helper Functions ====================

    fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<Message> {
        let to_json: String = row.get(6)?;
        let cc_json: String = row.get(7)?;
        let bcc_json: String = row.get(8)?;
        let labels_json: String = row.get(19)?;
        let refs_json: String = row.get(21)?;

        let to: Vec<String> = serde_json::from_str(&to_json).unwrap_or_default();
        let cc: Vec<String> = serde_json::from_str(&cc_json).unwrap_or_default();
        let bcc: Vec<String> = serde_json::from_str(&bcc_json).unwrap_or_default();
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
        let references: Vec<String> = serde_json::from_str(&refs_json).unwrap_or_default();

        let timestamp: i64 = row.get(9)?;
        let date = Utc.timestamp_opt(timestamp, 0).single().unwrap_or_else(Utc::now);

        Ok(Message {
            id: row.get(0)?,
            uid: row.get(1)?,
            account_id: row.get(2)?,
            folder: row.get(3)?,
            subject: row.get(4)?,
            from: row.get(5)?,
            to,
            cc,
            bcc,
            date,
            preview: row.get(10)?,
            text_body: row.get(11)?,
            html_body: row.get(12)?,
            attachments: Vec::new(), // Loaded separately
            flags: MessageFlags {
                seen: row.get(13)?,
                answered: row.get(14)?,
                flagged: row.get(15)?,
                deleted: row.get(16)?,
                draft: row.get(17)?,
                recent: false,
            },
            starred: row.get(18)?,
            labels,
            in_reply_to: row.get(20)?,
            references,
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub message_count: u32,
    pub attachment_count: u32,
    pub folder_count: u32,
    pub database_size: u64,
}

impl CacheStats {
    pub fn formatted_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.database_size >= GB {
            format!("{:.1} GB", self.database_size as f64 / GB as f64)
        } else if self.database_size >= MB {
            format!("{:.1} MB", self.database_size as f64 / MB as f64)
        } else if self.database_size >= KB {
            format!("{:.1} KB", self.database_size as f64 / KB as f64)
        } else {
            format!("{} B", self.database_size)
        }
    }
}
