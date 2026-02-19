// Winux Mail - IMAP Client
// Copyright (c) 2026 Winux OS Project

use crate::data::folder::Folder;
use crate::data::message::{Attachment, Message, MessageFlags};

use anyhow::{anyhow, Result};
use async_imap::types::{Fetch, Name};
use async_native_tls::TlsConnector;
use chrono::{DateTime, Utc};
use tokio::net::TcpStream;

use std::collections::HashSet;

/// IMAP client for fetching emails
pub struct ImapClient {
    server: String,
    port: u16,
    use_tls: bool,
}

impl ImapClient {
    pub fn new(server: &str, port: u16, use_tls: bool) -> Self {
        Self {
            server: server.to_string(),
            port,
            use_tls,
        }
    }

    /// Connect to the IMAP server
    pub async fn connect(&self, username: &str, password: &str) -> Result<ImapSession> {
        let addr = format!("{}:{}", self.server, self.port);
        let tcp_stream = TcpStream::connect(&addr).await?;

        if self.use_tls {
            let tls = TlsConnector::new();
            let tls_stream = tls.connect(&self.server, tcp_stream).await?;

            let client = async_imap::Client::new(tls_stream);
            let session = client
                .login(username, password)
                .await
                .map_err(|(err, _)| err)?;

            Ok(ImapSession {
                session: SessionType::Tls(session),
            })
        } else {
            let client = async_imap::Client::new(tcp_stream);
            let session = client
                .login(username, password)
                .await
                .map_err(|(err, _)| err)?;

            Ok(ImapSession {
                session: SessionType::Plain(session),
            })
        }
    }

    /// Connect using OAuth2
    pub async fn connect_oauth2(&self, username: &str, access_token: &str) -> Result<ImapSession> {
        let addr = format!("{}:{}", self.server, self.port);
        let tcp_stream = TcpStream::connect(&addr).await?;

        let tls = TlsConnector::new();
        let tls_stream = tls.connect(&self.server, tcp_stream).await?;

        let client = async_imap::Client::new(tls_stream);

        // OAuth2 authentication string format
        let auth_string = format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            username, access_token
        );
        let auth_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            auth_string.as_bytes()
        );

        let session = client
            .authenticate("XOAUTH2", XOAuth2Authenticator { token: auth_base64 })
            .await
            .map_err(|(err, _)| err)?;

        Ok(ImapSession {
            session: SessionType::Tls(session),
        })
    }
}

enum SessionType {
    Tls(async_imap::Session<async_native_tls::TlsStream<TcpStream>>),
    Plain(async_imap::Session<TcpStream>),
}

pub struct ImapSession {
    session: SessionType,
}

impl ImapSession {
    /// List all folders
    pub async fn list_folders(&mut self) -> Result<Vec<Folder>> {
        let folders = match &mut self.session {
            SessionType::Tls(session) => {
                session.list(Some(""), Some("*")).await?
            }
            SessionType::Plain(session) => {
                session.list(Some(""), Some("*")).await?
            }
        };

        let mut result = Vec::new();

        for folder in folders.iter() {
            let name = folder.name().to_string();
            let delimiter = folder.delimiter().map(|c| c.to_string());

            result.push(Folder {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.clone(),
                path: name.clone(),
                folder_type: Self::detect_folder_type(&name),
                unread_count: 0,
                total_count: 0,
                parent_id: None,
                delimiter,
                selectable: !folder.attributes().iter().any(|a| matches!(a, async_imap::types::NameAttribute::NoSelect)),
            });
        }

        Ok(result)
    }

    fn detect_folder_type(name: &str) -> crate::data::folder::FolderType {
        use crate::data::folder::FolderType;

        let name_lower = name.to_lowercase();

        if name_lower == "inbox" {
            FolderType::Inbox
        } else if name_lower.contains("sent") {
            FolderType::Sent
        } else if name_lower.contains("draft") {
            FolderType::Drafts
        } else if name_lower.contains("trash") || name_lower.contains("deleted") {
            FolderType::Trash
        } else if name_lower.contains("spam") || name_lower.contains("junk") {
            FolderType::Spam
        } else if name_lower.contains("archive") {
            FolderType::Archive
        } else if name_lower.contains("starred") || name_lower.contains("flagged") {
            FolderType::Starred
        } else {
            FolderType::Custom
        }
    }

    /// Select a folder
    pub async fn select_folder(&mut self, folder: &str) -> Result<FolderStatus> {
        let mailbox = match &mut self.session {
            SessionType::Tls(session) => session.select(folder).await?,
            SessionType::Plain(session) => session.select(folder).await?,
        };

        Ok(FolderStatus {
            exists: mailbox.exists,
            recent: mailbox.recent,
            unseen: mailbox.unseen.unwrap_or(0),
            uid_validity: mailbox.uid_validity.unwrap_or(0),
            uid_next: mailbox.uid_next.unwrap_or(0),
        })
    }

    /// Fetch message headers for a range
    pub async fn fetch_headers(&mut self, start: u32, end: u32) -> Result<Vec<MessageHeader>> {
        let sequence = format!("{}:{}", start, end);
        let fetch_query = "(UID FLAGS ENVELOPE BODYSTRUCTURE RFC822.SIZE INTERNALDATE)";

        let messages = match &mut self.session {
            SessionType::Tls(session) => {
                session.fetch(&sequence, fetch_query).await?
            }
            SessionType::Plain(session) => {
                session.fetch(&sequence, fetch_query).await?
            }
        };

        let mut headers = Vec::new();

        for message in messages.iter() {
            if let Some(header) = Self::parse_fetch_to_header(message) {
                headers.push(header);
            }
        }

        Ok(headers)
    }

    fn parse_fetch_to_header(fetch: &Fetch) -> Option<MessageHeader> {
        let envelope = fetch.envelope()?;

        let subject = envelope.subject
            .as_ref()
            .map(|s| String::from_utf8_lossy(s).to_string())
            .unwrap_or_default();

        let from = envelope.from
            .as_ref()
            .and_then(|addrs| addrs.first())
            .map(|addr| {
                let mailbox = addr.mailbox.as_ref().map(|m| String::from_utf8_lossy(m).to_string()).unwrap_or_default();
                let host = addr.host.as_ref().map(|h| String::from_utf8_lossy(h).to_string()).unwrap_or_default();
                let name = addr.name.as_ref().map(|n| String::from_utf8_lossy(n).to_string());

                if let Some(name) = name {
                    format!("{} <{}@{}>", name, mailbox, host)
                } else {
                    format!("{}@{}", mailbox, host)
                }
            })
            .unwrap_or_default();

        let date = fetch.internal_date()
            .map(|d| DateTime::from_timestamp(d.timestamp(), 0).unwrap_or_default())
            .unwrap_or_else(Utc::now);

        let flags = fetch.flags()
            .map(|f| Self::parse_flags(&f.iter().cloned().collect::<Vec<_>>()))
            .unwrap_or_default();

        Some(MessageHeader {
            uid: fetch.uid?,
            subject,
            from,
            date,
            size: fetch.size? as u64,
            flags,
            has_attachments: false, // Would need to parse BODYSTRUCTURE
        })
    }

    fn parse_flags(flags: &[async_imap::types::Flag<'_>]) -> MessageFlags {
        let mut result = MessageFlags::default();

        for flag in flags {
            match flag {
                async_imap::types::Flag::Seen => result.seen = true,
                async_imap::types::Flag::Answered => result.answered = true,
                async_imap::types::Flag::Flagged => result.flagged = true,
                async_imap::types::Flag::Deleted => result.deleted = true,
                async_imap::types::Flag::Draft => result.draft = true,
                _ => {}
            }
        }

        result
    }

    /// Fetch full message by UID
    pub async fn fetch_message(&mut self, uid: u32) -> Result<Message> {
        let fetch_query = "(UID FLAGS ENVELOPE BODY[] RFC822.SIZE INTERNALDATE)";

        let messages = match &mut self.session {
            SessionType::Tls(session) => {
                session.uid_fetch(uid.to_string(), fetch_query).await?
            }
            SessionType::Plain(session) => {
                session.uid_fetch(uid.to_string(), fetch_query).await?
            }
        };

        let fetch = messages.iter().next()
            .ok_or_else(|| anyhow!("Message not found"))?;

        let body = fetch.body()
            .ok_or_else(|| anyhow!("No body found"))?;

        Self::parse_message(fetch.uid.unwrap_or(uid), body)
    }

    fn parse_message(uid: u32, raw: &[u8]) -> Result<Message> {
        let parsed = mail_parser::MessageParser::default()
            .parse(raw)
            .ok_or_else(|| anyhow!("Failed to parse message"))?;

        let subject = parsed.subject().unwrap_or("(No Subject)").to_string();

        let from = parsed.from()
            .and_then(|addrs| addrs.first())
            .map(|addr| {
                if let Some(name) = addr.name() {
                    format!("{} <{}>", name, addr.address().unwrap_or(""))
                } else {
                    addr.address().unwrap_or("").to_string()
                }
            })
            .unwrap_or_default();

        let to: Vec<String> = parsed.to()
            .map(|addrs| {
                addrs.iter().map(|addr| {
                    addr.address().unwrap_or("").to_string()
                }).collect()
            })
            .unwrap_or_default();

        let cc: Vec<String> = parsed.cc()
            .map(|addrs| {
                addrs.iter().map(|addr| {
                    addr.address().unwrap_or("").to_string()
                }).collect()
            })
            .unwrap_or_default();

        let date = parsed.date()
            .map(|d| DateTime::from_timestamp(d.to_timestamp(), 0).unwrap_or_default())
            .unwrap_or_else(Utc::now);

        let text_body = parsed.body_text(0).map(|s| s.to_string());
        let html_body = parsed.body_html(0).map(|s| s.to_string());

        let preview = text_body.as_ref()
            .map(|t| t.chars().take(200).collect::<String>())
            .unwrap_or_default();

        // Parse attachments
        let mut attachments = Vec::new();
        for part in parsed.attachments() {
            attachments.push(Attachment {
                id: uuid::Uuid::new_v4().to_string(),
                filename: part.attachment_name().unwrap_or("attachment").to_string(),
                mime_type: part.content_type()
                    .map(|ct| ct.c_type.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
                size: part.len() as u64,
                data: Some(part.contents().to_vec()),
            });
        }

        let message_id = parsed.message_id()
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let in_reply_to = parsed.in_reply_to()
            .and_then(|ids| ids.first())
            .map(|s| s.to_string());

        Ok(Message {
            id: message_id,
            uid: Some(uid),
            account_id: String::new(),
            folder: String::new(),
            subject,
            from,
            to,
            cc,
            bcc: Vec::new(),
            date,
            preview,
            text_body,
            html_body,
            attachments,
            flags: MessageFlags::default(),
            starred: false,
            labels: Vec::new(),
            in_reply_to,
            references: Vec::new(),
        })
    }

    /// Search messages
    pub async fn search(&mut self, query: &str) -> Result<Vec<u32>> {
        let search_query = format!("OR SUBJECT \"{}\" FROM \"{}\" TO \"{}\"", query, query, query);

        let uids = match &mut self.session {
            SessionType::Tls(session) => {
                session.uid_search(&search_query).await?
            }
            SessionType::Plain(session) => {
                session.uid_search(&search_query).await?
            }
        };

        Ok(uids.iter().cloned().collect())
    }

    /// Set message flags
    pub async fn set_flags(&mut self, uid: u32, flags: &[&str], add: bool) -> Result<()> {
        let flag_str = flags.join(" ");
        let store_cmd = if add {
            format!("+FLAGS ({})", flag_str)
        } else {
            format!("-FLAGS ({})", flag_str)
        };

        match &mut self.session {
            SessionType::Tls(session) => {
                session.uid_store(uid.to_string(), &store_cmd).await?;
            }
            SessionType::Plain(session) => {
                session.uid_store(uid.to_string(), &store_cmd).await?;
            }
        }

        Ok(())
    }

    /// Mark message as read
    pub async fn mark_read(&mut self, uid: u32) -> Result<()> {
        self.set_flags(uid, &["\\Seen"], true).await
    }

    /// Mark message as unread
    pub async fn mark_unread(&mut self, uid: u32) -> Result<()> {
        self.set_flags(uid, &["\\Seen"], false).await
    }

    /// Star/flag message
    pub async fn star(&mut self, uid: u32) -> Result<()> {
        self.set_flags(uid, &["\\Flagged"], true).await
    }

    /// Unstar message
    pub async fn unstar(&mut self, uid: u32) -> Result<()> {
        self.set_flags(uid, &["\\Flagged"], false).await
    }

    /// Move message to folder
    pub async fn move_to(&mut self, uid: u32, folder: &str) -> Result<()> {
        match &mut self.session {
            SessionType::Tls(session) => {
                session.uid_mv(uid.to_string(), folder).await?;
            }
            SessionType::Plain(session) => {
                session.uid_mv(uid.to_string(), folder).await?;
            }
        }
        Ok(())
    }

    /// Delete message (move to trash)
    pub async fn delete(&mut self, uid: u32) -> Result<()> {
        self.set_flags(uid, &["\\Deleted"], true).await?;

        match &mut self.session {
            SessionType::Tls(session) => {
                session.expunge().await?;
            }
            SessionType::Plain(session) => {
                session.expunge().await?;
            }
        }

        Ok(())
    }

    /// Logout
    pub async fn logout(self) -> Result<()> {
        match self.session {
            SessionType::Tls(mut session) => {
                session.logout().await?;
            }
            SessionType::Plain(mut session) => {
                session.logout().await?;
            }
        }
        Ok(())
    }

    /// IDLE for push notifications
    pub async fn idle(&mut self) -> Result<IdleHandle> {
        match &mut self.session {
            SessionType::Tls(session) => {
                let handle = session.idle().await?;
                Ok(IdleHandle { _inner: () })
            }
            SessionType::Plain(session) => {
                let handle = session.idle().await?;
                Ok(IdleHandle { _inner: () })
            }
        }
    }
}

pub struct IdleHandle {
    _inner: (),
}

pub struct FolderStatus {
    pub exists: u32,
    pub recent: u32,
    pub unseen: u32,
    pub uid_validity: u32,
    pub uid_next: u32,
}

pub struct MessageHeader {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: DateTime<Utc>,
    pub size: u64,
    pub flags: MessageFlags,
    pub has_attachments: bool,
}

/// OAuth2 authenticator
struct XOAuth2Authenticator {
    token: String,
}

impl async_imap::Authenticator for XOAuth2Authenticator {
    type Response = String;

    fn process(&mut self, _data: &[u8]) -> Self::Response {
        self.token.clone()
    }
}
