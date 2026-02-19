// Winux Notes - SQLite Storage
// Copyright (c) 2026 Winux OS Project

use crate::data::{Note, NoteColor, Notebook, ChecklistItem};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Storage backend using SQLite
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Create a new storage instance
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .context("Failed to open database")?;

        let storage = Self { conn };
        storage.init_schema()?;

        Ok(storage)
    }

    /// Create an in-memory storage for testing
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    /// Get the path to the database file
    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .context("Could not find data directory")?
            .join("winux-notes");
        Ok(data_dir.join("notes.db"))
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS notebooks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                icon TEXT,
                parent_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES notebooks(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                notebook_id TEXT,
                color TEXT NOT NULL DEFAULT 'default',
                pinned INTEGER NOT NULL DEFAULT 0,
                favorite INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS note_tags (
                note_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (note_id, tag),
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS checklist_items (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                text TEXT NOT NULL,
                checked INTEGER NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_notes_notebook ON notes(notebook_id);
            CREATE INDEX IF NOT EXISTS idx_notes_pinned ON notes(pinned);
            CREATE INDEX IF NOT EXISTS idx_notes_favorite ON notes(favorite);
            CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at);
            CREATE INDEX IF NOT EXISTS idx_note_tags_tag ON note_tags(tag);
            CREATE INDEX IF NOT EXISTS idx_checklist_note ON checklist_items(note_id);

            -- Full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
                title,
                content,
                content='notes',
                content_rowid='rowid'
            );

            -- Triggers to keep FTS in sync
            CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
                INSERT INTO notes_fts(rowid, title, content) VALUES (NEW.rowid, NEW.title, NEW.content);
            END;

            CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, title, content) VALUES('delete', OLD.rowid, OLD.title, OLD.content);
            END;

            CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, title, content) VALUES('delete', OLD.rowid, OLD.title, OLD.content);
                INSERT INTO notes_fts(rowid, title, content) VALUES (NEW.rowid, NEW.title, NEW.content);
            END;
            "#,
        )?;

        Ok(())
    }

    // ==================== Notebook Operations ====================

    /// Create a new notebook
    pub fn create_notebook(&self, notebook: &Notebook) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO notebooks (id, name, description, icon, parent_id, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                notebook.id,
                notebook.name,
                notebook.description,
                notebook.icon,
                notebook.parent_id,
                notebook.created_at.to_rfc3339(),
                notebook.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get all notebooks
    pub fn get_notebooks(&self) -> Result<Vec<Notebook>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, icon, parent_id, created_at, updated_at FROM notebooks ORDER BY name"
        )?;

        let notebooks = stmt.query_map([], |row| {
            Ok(Notebook {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                parent_id: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(notebooks)
    }

    /// Get a notebook by ID
    pub fn get_notebook(&self, id: &str) -> Result<Option<Notebook>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, icon, parent_id, created_at, updated_at FROM notebooks WHERE id = ?1"
        )?;

        let notebook = stmt.query_row([id], |row| {
            Ok(Notebook {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                parent_id: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        }).optional()?;

        Ok(notebook)
    }

    /// Update a notebook
    pub fn update_notebook(&self, notebook: &Notebook) -> Result<()> {
        self.conn.execute(
            r#"UPDATE notebooks SET name = ?2, description = ?3, icon = ?4, parent_id = ?5, updated_at = ?6
               WHERE id = ?1"#,
            params![
                notebook.id,
                notebook.name,
                notebook.description,
                notebook.icon,
                notebook.parent_id,
                notebook.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Delete a notebook
    pub fn delete_notebook(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM notebooks WHERE id = ?1", [id])?;
        Ok(())
    }

    // ==================== Note Operations ====================

    /// Create a new note
    pub fn create_note(&self, note: &Note) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO notes (id, title, content, notebook_id, color, pinned, favorite, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                note.id,
                note.title,
                note.content,
                note.notebook_id,
                note.color.to_string(),
                note.pinned as i32,
                note.favorite as i32,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
            ],
        )?;

        // Insert tags
        for tag in &note.tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag) VALUES (?1, ?2)",
                params![note.id, tag],
            )?;
        }

        // Insert checklist items
        for (pos, item) in note.checklist.iter().enumerate() {
            self.conn.execute(
                r#"INSERT INTO checklist_items (id, note_id, text, checked, position)
                   VALUES (?1, ?2, ?3, ?4, ?5)"#,
                params![item.id, note.id, item.text, item.checked as i32, pos as i32],
            )?;
        }

        Ok(())
    }

    /// Get all notes
    pub fn get_all_notes(&self) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, title, content, notebook_id, color, pinned, favorite, created_at, updated_at
               FROM notes ORDER BY pinned DESC, updated_at DESC"#
        )?;

        let notes: Vec<Note> = stmt.query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                notebook_id: row.get(3)?,
                color: NoteColor::from_str(&row.get::<_, String>(4)?),
                pinned: row.get::<_, i32>(5)? != 0,
                favorite: row.get::<_, i32>(6)? != 0,
                tags: Vec::new(),
                checklist: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        // Load tags and checklist for each note
        let notes = notes.into_iter().map(|mut note| {
            note.tags = self.get_note_tags(&note.id).unwrap_or_default();
            note.checklist = self.get_checklist_items(&note.id).unwrap_or_default();
            note
        }).collect();

        Ok(notes)
    }

    /// Get notes by notebook
    pub fn get_notes_by_notebook(&self, notebook_id: &str) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, title, content, notebook_id, color, pinned, favorite, created_at, updated_at
               FROM notes WHERE notebook_id = ?1 ORDER BY pinned DESC, updated_at DESC"#
        )?;

        let notes: Vec<Note> = stmt.query_map([notebook_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                notebook_id: row.get(3)?,
                color: NoteColor::from_str(&row.get::<_, String>(4)?),
                pinned: row.get::<_, i32>(5)? != 0,
                favorite: row.get::<_, i32>(6)? != 0,
                tags: Vec::new(),
                checklist: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        let notes = notes.into_iter().map(|mut note| {
            note.tags = self.get_note_tags(&note.id).unwrap_or_default();
            note.checklist = self.get_checklist_items(&note.id).unwrap_or_default();
            note
        }).collect();

        Ok(notes)
    }

    /// Get pinned notes
    pub fn get_pinned_notes(&self) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, title, content, notebook_id, color, pinned, favorite, created_at, updated_at
               FROM notes WHERE pinned = 1 ORDER BY updated_at DESC"#
        )?;

        let notes: Vec<Note> = stmt.query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                notebook_id: row.get(3)?,
                color: NoteColor::from_str(&row.get::<_, String>(4)?),
                pinned: row.get::<_, i32>(5)? != 0,
                favorite: row.get::<_, i32>(6)? != 0,
                tags: Vec::new(),
                checklist: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        let notes = notes.into_iter().map(|mut note| {
            note.tags = self.get_note_tags(&note.id).unwrap_or_default();
            note.checklist = self.get_checklist_items(&note.id).unwrap_or_default();
            note
        }).collect();

        Ok(notes)
    }

    /// Get favorite notes
    pub fn get_favorite_notes(&self) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, title, content, notebook_id, color, pinned, favorite, created_at, updated_at
               FROM notes WHERE favorite = 1 ORDER BY pinned DESC, updated_at DESC"#
        )?;

        let notes: Vec<Note> = stmt.query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                notebook_id: row.get(3)?,
                color: NoteColor::from_str(&row.get::<_, String>(4)?),
                pinned: row.get::<_, i32>(5)? != 0,
                favorite: row.get::<_, i32>(6)? != 0,
                tags: Vec::new(),
                checklist: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        let notes = notes.into_iter().map(|mut note| {
            note.tags = self.get_note_tags(&note.id).unwrap_or_default();
            note.checklist = self.get_checklist_items(&note.id).unwrap_or_default();
            note
        }).collect();

        Ok(notes)
    }

    /// Get a note by ID
    pub fn get_note(&self, id: &str) -> Result<Option<Note>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, title, content, notebook_id, color, pinned, favorite, created_at, updated_at
               FROM notes WHERE id = ?1"#
        )?;

        let note = stmt.query_row([id], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                notebook_id: row.get(3)?,
                color: NoteColor::from_str(&row.get::<_, String>(4)?),
                pinned: row.get::<_, i32>(5)? != 0,
                favorite: row.get::<_, i32>(6)? != 0,
                tags: Vec::new(),
                checklist: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        }).optional()?;

        let note = note.map(|mut n| {
            n.tags = self.get_note_tags(&n.id).unwrap_or_default();
            n.checklist = self.get_checklist_items(&n.id).unwrap_or_default();
            n
        });

        Ok(note)
    }

    /// Update a note
    pub fn update_note(&self, note: &Note) -> Result<()> {
        self.conn.execute(
            r#"UPDATE notes SET title = ?2, content = ?3, notebook_id = ?4, color = ?5,
               pinned = ?6, favorite = ?7, updated_at = ?8 WHERE id = ?1"#,
            params![
                note.id,
                note.title,
                note.content,
                note.notebook_id,
                note.color.to_string(),
                note.pinned as i32,
                note.favorite as i32,
                note.updated_at.to_rfc3339(),
            ],
        )?;

        // Update tags
        self.conn.execute("DELETE FROM note_tags WHERE note_id = ?1", [&note.id])?;
        for tag in &note.tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag) VALUES (?1, ?2)",
                params![note.id, tag],
            )?;
        }

        // Update checklist
        self.conn.execute("DELETE FROM checklist_items WHERE note_id = ?1", [&note.id])?;
        for (pos, item) in note.checklist.iter().enumerate() {
            self.conn.execute(
                r#"INSERT INTO checklist_items (id, note_id, text, checked, position)
                   VALUES (?1, ?2, ?3, ?4, ?5)"#,
                params![item.id, note.id, item.text, item.checked as i32, pos as i32],
            )?;
        }

        Ok(())
    }

    /// Delete a note
    pub fn delete_note(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Search notes by text (full-text search)
    pub fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT n.id, n.title, n.content, n.notebook_id, n.color, n.pinned, n.favorite, n.created_at, n.updated_at
               FROM notes n
               JOIN notes_fts f ON n.rowid = f.rowid
               WHERE notes_fts MATCH ?1
               ORDER BY n.pinned DESC, n.updated_at DESC"#
        )?;

        let notes: Vec<Note> = stmt.query_map([query], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                notebook_id: row.get(3)?,
                color: NoteColor::from_str(&row.get::<_, String>(4)?),
                pinned: row.get::<_, i32>(5)? != 0,
                favorite: row.get::<_, i32>(6)? != 0,
                tags: Vec::new(),
                checklist: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        let notes = notes.into_iter().map(|mut note| {
            note.tags = self.get_note_tags(&note.id).unwrap_or_default();
            note.checklist = self.get_checklist_items(&note.id).unwrap_or_default();
            note
        }).collect();

        Ok(notes)
    }

    /// Get notes by tag
    pub fn get_notes_by_tag(&self, tag: &str) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT n.id, n.title, n.content, n.notebook_id, n.color, n.pinned, n.favorite, n.created_at, n.updated_at
               FROM notes n
               JOIN note_tags t ON n.id = t.note_id
               WHERE t.tag = ?1
               ORDER BY n.pinned DESC, n.updated_at DESC"#
        )?;

        let notes: Vec<Note> = stmt.query_map([tag], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                notebook_id: row.get(3)?,
                color: NoteColor::from_str(&row.get::<_, String>(4)?),
                pinned: row.get::<_, i32>(5)? != 0,
                favorite: row.get::<_, i32>(6)? != 0,
                tags: Vec::new(),
                checklist: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        let notes = notes.into_iter().map(|mut note| {
            note.tags = self.get_note_tags(&note.id).unwrap_or_default();
            note.checklist = self.get_checklist_items(&note.id).unwrap_or_default();
            note
        }).collect();

        Ok(notes)
    }

    /// Get all unique tags
    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT DISTINCT tag FROM note_tags ORDER BY tag")?;
        let tags = stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tags)
    }

    // ==================== Helper Functions ====================

    fn get_note_tags(&self, note_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT tag FROM note_tags WHERE note_id = ?1 ORDER BY tag")?;
        let tags = stmt.query_map([note_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tags)
    }

    fn get_checklist_items(&self, note_id: &str) -> Result<Vec<ChecklistItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, text, checked FROM checklist_items WHERE note_id = ?1 ORDER BY position"
        )?;
        let items = stmt.query_map([note_id], |row| {
            Ok(ChecklistItem {
                id: row.get(0)?,
                text: row.get(1)?,
                checked: row.get::<_, i32>(2)? != 0,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        Ok(items)
    }

    // ==================== Export Functions ====================

    /// Export a note to Markdown format
    pub fn export_to_markdown(&self, note_id: &str) -> Result<String> {
        let note = self.get_note(note_id)?
            .context("Note not found")?;

        let mut md = String::new();

        // Title
        md.push_str(&format!("# {}\n\n", note.title));

        // Metadata
        md.push_str(&format!("_Created: {}_  \n", note.created_at.format("%Y-%m-%d %H:%M")));
        md.push_str(&format!("_Updated: {}_  \n", note.updated_at.format("%Y-%m-%d %H:%M")));

        if !note.tags.is_empty() {
            md.push_str(&format!("_Tags: {}_\n", note.tags.join(", ")));
        }
        md.push_str("\n---\n\n");

        // Content
        md.push_str(&note.content);
        md.push_str("\n");

        // Checklist
        if !note.checklist.is_empty() {
            md.push_str("\n## Checklist\n\n");
            for item in &note.checklist {
                let checkbox = if item.checked { "[x]" } else { "[ ]" };
                md.push_str(&format!("- {} {}\n", checkbox, item.text));
            }
        }

        Ok(md)
    }

    /// Export all notes to Markdown files (returns a map of filename to content)
    pub fn export_all_to_markdown(&self) -> Result<Vec<(String, String)>> {
        let notes = self.get_all_notes()?;
        let mut exports = Vec::new();

        for note in notes {
            let filename = format!("{}.md", sanitize_filename(&note.title));
            let content = self.export_to_markdown(&note.id)?;
            exports.push((filename, content));
        }

        Ok(exports)
    }
}

/// Sanitize a string to be used as a filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_note() {
        let storage = Storage::in_memory().unwrap();
        let note = Note::new("Test Note", "This is test content");
        storage.create_note(&note).unwrap();

        let retrieved = storage.get_note(&note.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Test Note");
        assert_eq!(retrieved.content, "This is test content");
    }

    #[test]
    fn test_create_and_get_notebook() {
        let storage = Storage::in_memory().unwrap();
        let notebook = Notebook::new("Work");
        storage.create_notebook(&notebook).unwrap();

        let notebooks = storage.get_notebooks().unwrap();
        assert_eq!(notebooks.len(), 1);
        assert_eq!(notebooks[0].name, "Work");
    }

    #[test]
    fn test_notes_by_notebook() {
        let storage = Storage::in_memory().unwrap();

        let notebook = Notebook::new("Work");
        storage.create_notebook(&notebook).unwrap();

        let note1 = Note::new("Note 1", "Content 1").with_notebook(&notebook.id);
        let note2 = Note::new("Note 2", "Content 2").with_notebook(&notebook.id);
        let note3 = Note::new("Note 3", "Content 3"); // No notebook

        storage.create_note(&note1).unwrap();
        storage.create_note(&note2).unwrap();
        storage.create_note(&note3).unwrap();

        let work_notes = storage.get_notes_by_notebook(&notebook.id).unwrap();
        assert_eq!(work_notes.len(), 2);

        let all_notes = storage.get_all_notes().unwrap();
        assert_eq!(all_notes.len(), 3);
    }

    #[test]
    fn test_tags() {
        let storage = Storage::in_memory().unwrap();
        let mut note = Note::new("Tagged Note", "Content");
        note.tags = vec!["work".to_string(), "important".to_string()];
        storage.create_note(&note).unwrap();

        let tags = storage.get_all_tags().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"work".to_string()));
        assert!(tags.contains(&"important".to_string()));

        let work_notes = storage.get_notes_by_tag("work").unwrap();
        assert_eq!(work_notes.len(), 1);
    }

    #[test]
    fn test_search() {
        let storage = Storage::in_memory().unwrap();

        let note1 = Note::new("Shopping List", "Buy milk and eggs");
        let note2 = Note::new("Meeting Notes", "Discussed project timeline");
        let note3 = Note::new("Recipe", "Eggs and flour");

        storage.create_note(&note1).unwrap();
        storage.create_note(&note2).unwrap();
        storage.create_note(&note3).unwrap();

        let results = storage.search_notes("eggs").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_export_markdown() {
        let storage = Storage::in_memory().unwrap();
        let mut note = Note::new("Test Export", "This is the content");
        note.tags = vec!["test".to_string()];
        note.add_checklist_item("Item 1");
        note.add_checklist_item("Item 2");
        storage.create_note(&note).unwrap();

        let md = storage.export_to_markdown(&note.id).unwrap();
        assert!(md.contains("# Test Export"));
        assert!(md.contains("This is the content"));
        assert!(md.contains("Tags: test"));
        assert!(md.contains("[ ] Item 1"));
        assert!(md.contains("[ ] Item 2"));
    }
}
