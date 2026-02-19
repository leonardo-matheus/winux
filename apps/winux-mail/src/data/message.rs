// Winux Mail - Message Data Structures
// Copyright (c) 2026 Winux OS Project

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID (Message-ID header)
    pub id: String,

    /// IMAP UID (unique within folder)
    pub uid: Option<u32>,

    /// Account this message belongs to
    pub account_id: String,

    /// Folder path
    pub folder: String,

    /// Subject line
    pub subject: String,

    /// From address (formatted)
    pub from: String,

    /// To addresses
    pub to: Vec<String>,

    /// CC addresses
    pub cc: Vec<String>,

    /// BCC addresses (only for sent/drafts)
    pub bcc: Vec<String>,

    /// Date/time
    pub date: DateTime<Utc>,

    /// Preview text (first ~200 chars)
    pub preview: String,

    /// Plain text body
    pub text_body: Option<String>,

    /// HTML body
    pub html_body: Option<String>,

    /// Attachments
    pub attachments: Vec<Attachment>,

    /// Message flags
    pub flags: MessageFlags,

    /// Starred/flagged
    pub starred: bool,

    /// Labels/tags
    pub labels: Vec<String>,

    /// In-Reply-To header
    pub in_reply_to: Option<String>,

    /// References header (thread)
    pub references: Vec<String>,
}

impl Message {
    pub fn is_read(&self) -> bool {
        self.flags.seen
    }

    pub fn is_draft(&self) -> bool {
        self.flags.draft
    }

    pub fn is_replied(&self) -> bool {
        self.flags.answered
    }

    pub fn has_attachments(&self) -> bool {
        !self.attachments.is_empty()
    }

    /// Get sender name (without email)
    pub fn sender_name(&self) -> &str {
        if let Some(pos) = self.from.find('<') {
            self.from[..pos].trim()
        } else if let Some(pos) = self.from.find('@') {
            &self.from[..pos]
        } else {
            &self.from
        }
    }

    /// Get sender email
    pub fn sender_email(&self) -> &str {
        if let Some(start) = self.from.find('<') {
            if let Some(end) = self.from.find('>') {
                return &self.from[start + 1..end];
            }
        }
        &self.from
    }

    /// Format date for display
    pub fn formatted_date(&self) -> String {
        let now = Utc::now();
        let today = now.date_naive();
        let msg_date = self.date.date_naive();

        if msg_date == today {
            self.date.format("%H:%M").to_string()
        } else if (now - self.date).num_days() < 7 {
            self.date.format("%a %H:%M").to_string()
        } else if msg_date.year() == today.year() {
            self.date.format("%b %d").to_string()
        } else {
            self.date.format("%Y-%m-%d").to_string()
        }
    }

    /// Get short preview
    pub fn short_preview(&self, max_len: usize) -> String {
        if self.preview.len() <= max_len {
            self.preview.clone()
        } else {
            format!("{}...", &self.preview[..max_len])
        }
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            uid: None,
            account_id: String::new(),
            folder: String::new(),
            subject: String::new(),
            from: String::new(),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            date: Utc::now(),
            preview: String::new(),
            text_body: None,
            html_body: None,
            attachments: Vec::new(),
            flags: MessageFlags::default(),
            starred: false,
            labels: Vec::new(),
            in_reply_to: None,
            references: Vec::new(),
        }
    }
}

/// Message flags (IMAP standard flags)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageFlags {
    /// \Seen - Message has been read
    pub seen: bool,

    /// \Answered - Message has been answered
    pub answered: bool,

    /// \Flagged - Message is flagged/starred
    pub flagged: bool,

    /// \Deleted - Message is marked for deletion
    pub deleted: bool,

    /// \Draft - Message is a draft
    pub draft: bool,

    /// \Recent - Message is recent (set by server)
    pub recent: bool,
}

impl MessageFlags {
    pub fn to_imap_flags(&self) -> Vec<&'static str> {
        let mut flags = Vec::new();

        if self.seen {
            flags.push("\\Seen");
        }
        if self.answered {
            flags.push("\\Answered");
        }
        if self.flagged {
            flags.push("\\Flagged");
        }
        if self.deleted {
            flags.push("\\Deleted");
        }
        if self.draft {
            flags.push("\\Draft");
        }

        flags
    }
}

/// Email attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Unique ID
    pub id: String,

    /// Filename
    pub filename: String,

    /// MIME type
    pub mime_type: String,

    /// Size in bytes
    pub size: u64,

    /// Content data (may be None if not loaded)
    #[serde(skip)]
    pub data: Option<Vec<u8>>,
}

impl Attachment {
    pub fn new(filename: String, mime_type: String, data: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            filename,
            mime_type,
            size: data.len() as u64,
            data: Some(data),
        }
    }

    /// Get human-readable size
    pub fn formatted_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.1} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.1} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.1} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.filename.rsplit('.').next()
    }

    /// Check if this is an image
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    /// Check if this is a PDF
    pub fn is_pdf(&self) -> bool {
        self.mime_type == "application/pdf"
    }

    /// Check if this is an archive
    pub fn is_archive(&self) -> bool {
        self.mime_type.contains("zip")
            || self.mime_type.contains("tar")
            || self.mime_type.contains("rar")
            || self.mime_type.contains("7z")
            || self.mime_type.contains("compressed")
    }

    /// Get icon name for this attachment type
    pub fn icon_name(&self) -> &'static str {
        if self.is_image() {
            "image-x-generic-symbolic"
        } else if self.is_pdf() {
            "x-office-document-symbolic"
        } else if self.is_archive() {
            "package-x-generic-symbolic"
        } else if self.mime_type.starts_with("audio/") {
            "audio-x-generic-symbolic"
        } else if self.mime_type.starts_with("video/") {
            "video-x-generic-symbolic"
        } else if self.mime_type.starts_with("text/") {
            "text-x-generic-symbolic"
        } else if self.mime_type.contains("spreadsheet") || self.mime_type.contains("excel") {
            "x-office-spreadsheet-symbolic"
        } else if self.mime_type.contains("presentation") || self.mime_type.contains("powerpoint") {
            "x-office-presentation-symbolic"
        } else if self.mime_type.contains("document") || self.mime_type.contains("word") {
            "x-office-document-symbolic"
        } else {
            "text-x-generic-symbolic"
        }
    }
}

/// Thread of related messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    /// Thread ID (usually the first message's ID)
    pub id: String,

    /// Subject (normalized, without Re:/Fwd:)
    pub subject: String,

    /// All messages in this thread
    pub messages: Vec<Message>,

    /// Participants
    pub participants: Vec<String>,

    /// Most recent message date
    pub last_date: DateTime<Utc>,

    /// Total unread count
    pub unread_count: u32,

    /// Has attachments in any message
    pub has_attachments: bool,

    /// Is any message starred
    pub has_starred: bool,
}

impl Thread {
    pub fn from_messages(messages: Vec<Message>) -> Self {
        let first = messages.first();

        let subject = first
            .map(|m| Self::normalize_subject(&m.subject))
            .unwrap_or_default();

        let mut participants: Vec<String> = messages
            .iter()
            .map(|m| m.sender_name().to_string())
            .collect();
        participants.sort();
        participants.dedup();

        let last_date = messages
            .iter()
            .map(|m| m.date)
            .max()
            .unwrap_or_else(Utc::now);

        let unread_count = messages.iter().filter(|m| !m.is_read()).count() as u32;
        let has_attachments = messages.iter().any(|m| m.has_attachments());
        let has_starred = messages.iter().any(|m| m.starred);

        let id = first.map(|m| m.id.clone()).unwrap_or_default();

        Self {
            id,
            subject,
            messages,
            participants,
            last_date,
            unread_count,
            has_attachments,
            has_starred,
        }
    }

    fn normalize_subject(subject: &str) -> String {
        let mut s = subject.trim();

        // Remove Re:, Fwd:, etc.
        loop {
            let lower = s.to_lowercase();
            if lower.starts_with("re:") {
                s = s[3..].trim();
            } else if lower.starts_with("fwd:") {
                s = s[4..].trim();
            } else if lower.starts_with("fw:") {
                s = s[3..].trim();
            } else {
                break;
            }
        }

        s.to_string()
    }
}

/// Search query
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// Full-text search
    pub text: Option<String>,

    /// From address contains
    pub from: Option<String>,

    /// To address contains
    pub to: Option<String>,

    /// Subject contains
    pub subject: Option<String>,

    /// Has attachments
    pub has_attachment: Option<bool>,

    /// Is unread
    pub is_unread: Option<bool>,

    /// Is starred
    pub is_starred: Option<bool>,

    /// Date after
    pub after: Option<DateTime<Utc>>,

    /// Date before
    pub before: Option<DateTime<Utc>>,

    /// In folder
    pub folder: Option<String>,

    /// Has label
    pub label: Option<String>,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    pub fn from(mut self, from: &str) -> Self {
        self.from = Some(from.to_string());
        self
    }

    pub fn to(mut self, to: &str) -> Self {
        self.to = Some(to.to_string());
        self
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    pub fn has_attachment(mut self, has: bool) -> Self {
        self.has_attachment = Some(has);
        self
    }

    pub fn is_unread(mut self, unread: bool) -> Self {
        self.is_unread = Some(unread);
        self
    }

    pub fn is_starred(mut self, starred: bool) -> Self {
        self.is_starred = Some(starred);
        self
    }

    pub fn after(mut self, date: DateTime<Utc>) -> Self {
        self.after = Some(date);
        self
    }

    pub fn before(mut self, date: DateTime<Utc>) -> Self {
        self.before = Some(date);
        self
    }

    pub fn folder(mut self, folder: &str) -> Self {
        self.folder = Some(folder.to_string());
        self
    }

    /// Convert to IMAP search query
    pub fn to_imap_query(&self) -> String {
        let mut parts = Vec::new();

        if let Some(text) = &self.text {
            parts.push(format!("OR SUBJECT \"{}\" BODY \"{}\"", text, text));
        }

        if let Some(from) = &self.from {
            parts.push(format!("FROM \"{}\"", from));
        }

        if let Some(to) = &self.to {
            parts.push(format!("TO \"{}\"", to));
        }

        if let Some(subject) = &self.subject {
            parts.push(format!("SUBJECT \"{}\"", subject));
        }

        if self.has_attachment == Some(true) {
            // No direct IMAP support, will filter locally
        }

        if self.is_unread == Some(true) {
            parts.push("UNSEEN".to_string());
        } else if self.is_unread == Some(false) {
            parts.push("SEEN".to_string());
        }

        if self.is_starred == Some(true) {
            parts.push("FLAGGED".to_string());
        }

        if let Some(after) = &self.after {
            parts.push(format!("SINCE {}", after.format("%d-%b-%Y")));
        }

        if let Some(before) = &self.before {
            parts.push(format!("BEFORE {}", before.format("%d-%b-%Y")));
        }

        if parts.is_empty() {
            "ALL".to_string()
        } else {
            parts.join(" ")
        }
    }
}
