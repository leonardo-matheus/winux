// Winux Mail - SMTP Client
// Copyright (c) 2026 Winux OS Project

use crate::data::message::{Attachment, Message};

use anyhow::{anyhow, Result};
use lettre::{
    message::{
        header::ContentType,
        Attachment as LettreAttachment, Mailbox, MessageBuilder, MultiPart, SinglePart,
    },
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        client::{Tls, TlsParameters},
    },
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

use std::path::Path;

/// SMTP client for sending emails
pub struct SmtpClient {
    server: String,
    port: u16,
    use_starttls: bool,
}

impl SmtpClient {
    pub fn new(server: &str, port: u16, use_starttls: bool) -> Self {
        Self {
            server: server.to_string(),
            port,
            use_starttls,
        }
    }

    /// Create SMTP transport with password authentication
    pub fn create_transport(&self, username: &str, password: &str) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let creds = Credentials::new(username.to_string(), password.to_string());

        let transport = if self.use_starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.server)?
                .port(self.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.server)?
                .port(self.port)
                .credentials(creds)
                .build()
        };

        Ok(transport)
    }

    /// Create SMTP transport with OAuth2 authentication
    pub fn create_transport_oauth2(&self, username: &str, access_token: &str) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        // OAuth2 SMTP authentication uses XOAUTH2 mechanism
        let auth_string = format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            username, access_token
        );

        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.server)?
            .port(self.port)
            .authentication(vec![Mechanism::Xoauth2])
            .credentials(Credentials::new(username.to_string(), auth_string))
            .build();

        Ok(transport)
    }

    /// Send an email
    pub async fn send(
        &self,
        transport: &AsyncSmtpTransport<Tokio1Executor>,
        from: &str,
        to: &[String],
        cc: &[String],
        bcc: &[String],
        subject: &str,
        text_body: Option<&str>,
        html_body: Option<&str>,
        attachments: &[Attachment],
        reply_to: Option<&str>,
        in_reply_to: Option<&str>,
        references: &[String],
    ) -> Result<()> {
        let from_mailbox: Mailbox = from.parse()
            .map_err(|_| anyhow!("Invalid from address: {}", from))?;

        let mut builder = MessageBuilder::new()
            .from(from_mailbox.clone())
            .subject(subject);

        // Add recipients
        for recipient in to {
            let mailbox: Mailbox = recipient.parse()
                .map_err(|_| anyhow!("Invalid to address: {}", recipient))?;
            builder = builder.to(mailbox);
        }

        for recipient in cc {
            let mailbox: Mailbox = recipient.parse()
                .map_err(|_| anyhow!("Invalid cc address: {}", recipient))?;
            builder = builder.cc(mailbox);
        }

        for recipient in bcc {
            let mailbox: Mailbox = recipient.parse()
                .map_err(|_| anyhow!("Invalid bcc address: {}", recipient))?;
            builder = builder.bcc(mailbox);
        }

        // Reply headers
        if let Some(reply_to_addr) = reply_to {
            let mailbox: Mailbox = reply_to_addr.parse()
                .map_err(|_| anyhow!("Invalid reply-to address"))?;
            builder = builder.reply_to(mailbox);
        }

        if let Some(in_reply_to_id) = in_reply_to {
            builder = builder.in_reply_to(in_reply_to_id.to_string());
        }

        if !references.is_empty() {
            builder = builder.references(references.join(" "));
        }

        // Build the message body
        let message = if attachments.is_empty() {
            // Simple message without attachments
            if let Some(html) = html_body {
                if let Some(text) = text_body {
                    // Both HTML and text
                    builder.multipart(
                        MultiPart::alternative()
                            .singlepart(
                                SinglePart::builder()
                                    .content_type(ContentType::TEXT_PLAIN)
                                    .body(text.to_string())
                            )
                            .singlepart(
                                SinglePart::builder()
                                    .content_type(ContentType::TEXT_HTML)
                                    .body(html.to_string())
                            )
                    )?
                } else {
                    // HTML only
                    builder.body(html.to_string())?
                }
            } else if let Some(text) = text_body {
                // Text only
                builder.body(text.to_string())?
            } else {
                // Empty body
                builder.body(String::new())?
            }
        } else {
            // Message with attachments
            let mut multipart = MultiPart::mixed();

            // Add body
            if let Some(html) = html_body {
                if let Some(text) = text_body {
                    multipart = multipart.multipart(
                        MultiPart::alternative()
                            .singlepart(
                                SinglePart::builder()
                                    .content_type(ContentType::TEXT_PLAIN)
                                    .body(text.to_string())
                            )
                            .singlepart(
                                SinglePart::builder()
                                    .content_type(ContentType::TEXT_HTML)
                                    .body(html.to_string())
                            )
                    );
                } else {
                    multipart = multipart.singlepart(
                        SinglePart::builder()
                            .content_type(ContentType::TEXT_HTML)
                            .body(html.to_string())
                    );
                }
            } else if let Some(text) = text_body {
                multipart = multipart.singlepart(
                    SinglePart::builder()
                        .content_type(ContentType::TEXT_PLAIN)
                        .body(text.to_string())
                );
            }

            // Add attachments
            for attachment in attachments {
                if let Some(data) = &attachment.data {
                    let content_type: ContentType = attachment.mime_type.parse()
                        .unwrap_or(ContentType::parse("application/octet-stream").unwrap());

                    let att = LettreAttachment::new(attachment.filename.clone())
                        .body(data.clone(), content_type);

                    multipart = multipart.singlepart(att);
                }
            }

            builder.multipart(multipart)?
        };

        // Send the message
        transport.send(message).await?;

        Ok(())
    }

    /// Send a simple text email
    pub async fn send_simple(
        &self,
        transport: &AsyncSmtpTransport<Tokio1Executor>,
        from: &str,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<()> {
        self.send(
            transport,
            from,
            &[to.to_string()],
            &[],
            &[],
            subject,
            Some(body),
            None,
            &[],
            None,
            None,
            &[],
        ).await
    }

    /// Send a reply
    pub async fn send_reply(
        &self,
        transport: &AsyncSmtpTransport<Tokio1Executor>,
        original: &Message,
        from: &str,
        body: &str,
        html_body: Option<&str>,
        reply_all: bool,
    ) -> Result<()> {
        let mut to = vec![original.from.clone()];

        if reply_all {
            // Add original recipients except ourselves
            for recipient in &original.to {
                if recipient != from && !to.contains(recipient) {
                    to.push(recipient.clone());
                }
            }
        }

        let subject = if original.subject.starts_with("Re:") {
            original.subject.clone()
        } else {
            format!("Re: {}", original.subject)
        };

        // Build references chain
        let mut references = original.references.clone();
        references.push(original.id.clone());

        self.send(
            transport,
            from,
            &to,
            &original.cc,
            &[],
            &subject,
            Some(body),
            html_body,
            &[],
            None,
            Some(&original.id),
            &references,
        ).await
    }

    /// Forward an email
    pub async fn send_forward(
        &self,
        transport: &AsyncSmtpTransport<Tokio1Executor>,
        original: &Message,
        from: &str,
        to: &[String],
        additional_text: Option<&str>,
    ) -> Result<()> {
        let subject = if original.subject.starts_with("Fwd:") {
            original.subject.clone()
        } else {
            format!("Fwd: {}", original.subject)
        };

        // Build forwarded body
        let forward_header = format!(
            "---------- Forwarded message ----------\n\
            From: {}\n\
            Date: {}\n\
            Subject: {}\n\
            To: {}\n\n",
            original.from,
            original.date.format("%Y-%m-%d %H:%M"),
            original.subject,
            original.to.join(", ")
        );

        let body = if let Some(text) = additional_text {
            format!("{}\n\n{}{}", text, forward_header, original.text_body.as_deref().unwrap_or(""))
        } else {
            format!("{}{}", forward_header, original.text_body.as_deref().unwrap_or(""))
        };

        self.send(
            transport,
            from,
            to,
            &[],
            &[],
            &subject,
            Some(&body),
            None,
            &original.attachments,
            None,
            None,
            &[],
        ).await
    }

    /// Save as draft (sends to Drafts folder via IMAP APPEND)
    pub fn build_draft(
        from: &str,
        to: &[String],
        cc: &[String],
        subject: &str,
        body: &str,
    ) -> Result<Vec<u8>> {
        let from_mailbox: Mailbox = from.parse()
            .map_err(|_| anyhow!("Invalid from address"))?;

        let mut builder = MessageBuilder::new()
            .from(from_mailbox)
            .subject(subject);

        for recipient in to {
            if !recipient.is_empty() {
                if let Ok(mailbox) = recipient.parse::<Mailbox>() {
                    builder = builder.to(mailbox);
                }
            }
        }

        for recipient in cc {
            if !recipient.is_empty() {
                if let Ok(mailbox) = recipient.parse::<Mailbox>() {
                    builder = builder.cc(mailbox);
                }
            }
        }

        let message = builder.body(body.to_string())?;

        Ok(message.formatted())
    }

    /// Verify SMTP connection
    pub async fn verify_connection(&self, username: &str, password: &str) -> Result<bool> {
        let transport = self.create_transport(username, password)?;

        // Test connection by connecting
        transport.test_connection().await
            .map_err(|e| anyhow!("SMTP connection failed: {}", e))
    }
}

/// Email composition helper
pub struct EmailComposer {
    from: String,
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: String,
    text_body: String,
    html_body: Option<String>,
    attachments: Vec<Attachment>,
    reply_to: Option<String>,
    in_reply_to: Option<String>,
    references: Vec<String>,
}

impl EmailComposer {
    pub fn new(from: &str) -> Self {
        Self {
            from: from.to_string(),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            text_body: String::new(),
            html_body: None,
            attachments: Vec::new(),
            reply_to: None,
            in_reply_to: None,
            references: Vec::new(),
        }
    }

    pub fn to(mut self, recipient: &str) -> Self {
        self.to.push(recipient.to_string());
        self
    }

    pub fn cc(mut self, recipient: &str) -> Self {
        self.cc.push(recipient.to_string());
        self
    }

    pub fn bcc(mut self, recipient: &str) -> Self {
        self.bcc.push(recipient.to_string());
        self
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = subject.to_string();
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.text_body = body.to_string();
        self
    }

    pub fn html_body(mut self, html: &str) -> Self {
        self.html_body = Some(html.to_string());
        self
    }

    pub fn attach_file(mut self, path: &Path) -> Result<Self> {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment")
            .to_string();

        let data = std::fs::read(path)?;
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        self.attachments.push(Attachment {
            id: uuid::Uuid::new_v4().to_string(),
            filename,
            mime_type,
            size: data.len() as u64,
            data: Some(data),
        });

        Ok(self)
    }

    pub fn attach(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    pub fn reply_to(mut self, address: &str) -> Self {
        self.reply_to = Some(address.to_string());
        self
    }

    pub fn in_reply_to(mut self, message_id: &str) -> Self {
        self.in_reply_to = Some(message_id.to_string());
        self
    }

    pub fn references(mut self, refs: Vec<String>) -> Self {
        self.references = refs;
        self
    }

    /// Build and send the email
    pub async fn send(self, client: &SmtpClient, transport: &AsyncSmtpTransport<Tokio1Executor>) -> Result<()> {
        client.send(
            transport,
            &self.from,
            &self.to,
            &self.cc,
            &self.bcc,
            &self.subject,
            Some(&self.text_body),
            self.html_body.as_deref(),
            &self.attachments,
            self.reply_to.as_deref(),
            self.in_reply_to.as_deref(),
            &self.references,
        ).await
    }
}
