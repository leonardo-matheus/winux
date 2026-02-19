// Conversation Database - SQLite storage for chat history

use crate::chat::{Conversation, Message, MessageRole, MessageContent};
use anyhow::{anyhow, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct ConversationDatabase {
    conn: Mutex<Connection>,
}

impl ConversationDatabase {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_database_path()?;

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };

        db.initialize_schema()?;
        Ok(db)
    }

    fn get_database_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow!("Could not find data directory"))?;
        Ok(data_dir.join("winux-ai").join("conversations.db"))
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Database lock poisoned"))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                model TEXT NOT NULL,
                system_prompt TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                image_path TEXT,
                file_path TEXT,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_conversation
             ON messages(conversation_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversations_updated
             ON conversations(updated_at DESC)",
            [],
        )?;

        Ok(())
    }

    /// Save a conversation (insert or update)
    pub fn save_conversation(&self, conversation: &Conversation) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return,
        };

        let metadata_json = serde_json::to_string(&conversation.metadata).unwrap_or_default();

        let _ = conn.execute(
            "INSERT OR REPLACE INTO conversations
             (id, title, model, system_prompt, created_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                conversation.id,
                conversation.title,
                conversation.model,
                conversation.system_prompt,
                conversation.created_at.to_rfc3339(),
                conversation.updated_at.to_rfc3339(),
                metadata_json,
            ],
        );

        // Save messages
        for message in &conversation.messages {
            self.save_message(&conversation.id, message);
        }
    }

    /// Save a single message
    pub fn save_message(&self, conversation_id: &str, message: &Message) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return,
        };

        let content_json = serde_json::to_string(&message.content).unwrap_or_default();

        let _ = conn.execute(
            "INSERT OR REPLACE INTO messages
             (id, conversation_id, role, content, timestamp, image_path, file_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                message.id,
                conversation_id,
                message.role.to_string(),
                content_json,
                message.timestamp.to_rfc3339(),
                message.image_path,
                message.file_path,
            ],
        );
    }

    /// Load a conversation by ID
    pub fn load_conversation(&self, id: &str) -> Option<Conversation> {
        let conn = self.conn.lock().ok()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, model, system_prompt, created_at, updated_at, metadata
                 FROM conversations WHERE id = ?1",
            )
            .ok()?;

        let conversation = stmt
            .query_row([id], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let model: String = row.get(2)?;
                let system_prompt: Option<String> = row.get(3)?;
                let created_at: String = row.get(4)?;
                let updated_at: String = row.get(5)?;
                let metadata_str: String = row.get(6)?;

                Ok((id, title, model, system_prompt, created_at, updated_at, metadata_str))
            })
            .ok()?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&conversation.4)
            .ok()?
            .with_timezone(&chrono::Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&conversation.5)
            .ok()?
            .with_timezone(&chrono::Utc);
        let metadata = serde_json::from_str(&conversation.6).unwrap_or_default();

        // Load messages
        let messages = self.load_messages(&conversation.0)?;

        Some(Conversation {
            id: conversation.0,
            title: conversation.1,
            model: conversation.2,
            system_prompt: conversation.3,
            created_at,
            updated_at,
            messages,
            metadata,
        })
    }

    /// Load messages for a conversation
    fn load_messages(&self, conversation_id: &str) -> Option<Vec<Message>> {
        let conn = self.conn.lock().ok()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, role, content, timestamp, image_path, file_path
                 FROM messages WHERE conversation_id = ?1 ORDER BY timestamp ASC",
            )
            .ok()?;

        let messages = stmt
            .query_map([conversation_id], |row| {
                let id: String = row.get(0)?;
                let role_str: String = row.get(1)?;
                let content_str: String = row.get(2)?;
                let timestamp_str: String = row.get(3)?;
                let image_path: Option<String> = row.get(4)?;
                let file_path: Option<String> = row.get(5)?;

                Ok((id, role_str, content_str, timestamp_str, image_path, file_path))
            })
            .ok()?;

        let mut result = Vec::new();
        for msg_result in messages {
            if let Ok(msg) = msg_result {
                let role = match msg.1.as_str() {
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "system" => MessageRole::System,
                    _ => continue,
                };

                let content: Vec<MessageContent> = serde_json::from_str(&msg.2)
                    .unwrap_or_else(|_| vec![MessageContent::Text { text: msg.2.clone() }]);

                let timestamp = chrono::DateTime::parse_from_rfc3339(&msg.3)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                result.push(Message {
                    id: msg.0,
                    role,
                    content,
                    timestamp,
                    image_path: msg.4,
                    file_path: msg.5,
                });
            }
        }

        Some(result)
    }

    /// Get list of recent conversations
    pub fn get_recent_conversations(&self, limit: usize) -> Vec<ConversationSummary> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut stmt = match conn.prepare(
            "SELECT id, title, updated_at,
             (SELECT COUNT(*) FROM messages WHERE conversation_id = conversations.id) as message_count
             FROM conversations ORDER BY updated_at DESC LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = match stmt.query_map([limit], |row| {
            Ok(ConversationSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                updated_at: row.get(2)?,
                message_count: row.get(3)?,
            })
        }) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        rows.filter_map(|r| r.ok()).collect()
    }

    /// Search conversations
    pub fn search_conversations(&self, query: &str, limit: usize) -> Vec<ConversationSummary> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut stmt = match conn.prepare(
            "SELECT DISTINCT c.id, c.title, c.updated_at,
             (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as message_count
             FROM conversations c
             LEFT JOIN messages m ON c.id = m.conversation_id
             WHERE c.title LIKE ?1 OR m.content LIKE ?1
             ORDER BY c.updated_at DESC LIMIT ?2",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let search_pattern = format!("%{}%", query);

        let rows = match stmt.query_map(params![search_pattern, limit], |row| {
            Ok(ConversationSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                updated_at: row.get(2)?,
                message_count: row.get(3)?,
            })
        }) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        rows.filter_map(|r| r.ok()).collect()
    }

    /// Delete a conversation
    pub fn delete_conversation(&self, id: &str) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return,
        };

        // Delete messages first (SQLite may not have FK support enabled)
        let _ = conn.execute("DELETE FROM messages WHERE conversation_id = ?1", [id]);
        let _ = conn.execute("DELETE FROM conversations WHERE id = ?1", [id]);
    }

    /// Get database statistics
    pub fn get_stats(&self) -> DatabaseStats {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => {
                return DatabaseStats {
                    total_conversations: 0,
                    total_messages: 0,
                    database_size_bytes: 0,
                }
            }
        };

        let total_conversations: i64 = conn
            .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
            .unwrap_or(0);

        let total_messages: i64 = conn
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .unwrap_or(0);

        let database_size_bytes = Self::get_database_path()
            .ok()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .unwrap_or(0);

        DatabaseStats {
            total_conversations: total_conversations as u64,
            total_messages: total_messages as u64,
            database_size_bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub message_count: i64,
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_conversations: u64,
    pub total_messages: u64,
    pub database_size_bytes: u64,
}
