// Winux Mail - Compose View
// Copyright (c) 2026 Winux OS Project

use crate::data::message::{Attachment, Message};
use crate::ui::composer::RichTextEditor;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Entry, FileChooserAction, FileChooserNative, FlowBox,
    HeaderBar, Label, Orientation, ResponseType, Separator, Window,
};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct ComposeView {
    pub window: adw::Window,
    pub to_entry: Entry,
    pub cc_entry: Entry,
    pub bcc_entry: Entry,
    pub subject_entry: Entry,
    pub editor: RichTextEditor,
    pub attachments: Rc<RefCell<Vec<Attachment>>>,
    pub attachments_box: FlowBox,
    pub draft_id: Rc<RefCell<Option<String>>>,
    pub reply_to: Rc<RefCell<Option<Message>>>,
}

impl ComposeView {
    pub fn new(parent: &impl IsA<Window>, mailto: Option<&str>) -> Self {
        let window = adw::Window::builder()
            .title("New Message")
            .default_width(700)
            .default_height(600)
            .transient_for(parent)
            .modal(false)
            .build();

        // Header bar
        let header = HeaderBar::new();

        let discard_btn = Button::builder()
            .label("Discard")
            .build();

        let send_btn = Button::builder()
            .label("Send")
            .css_classes(vec!["suggested-action"])
            .build();

        header.pack_start(&discard_btn);
        header.pack_end(&send_btn);

        // Main content
        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        main_box.append(&header);

        // Recipients section
        let recipients_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .spacing(4)
            .build();

        // To field
        let to_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let to_label = Label::builder()
            .label("To:")
            .width_chars(8)
            .halign(gtk4::Align::End)
            .build();

        let to_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("recipient@example.com")
            .build();

        to_box.append(&to_label);
        to_box.append(&to_entry);

        // CC field
        let cc_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let cc_label = Label::builder()
            .label("Cc:")
            .width_chars(8)
            .halign(gtk4::Align::End)
            .build();

        let cc_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("cc@example.com")
            .build();

        cc_box.append(&cc_label);
        cc_box.append(&cc_entry);

        // BCC field
        let bcc_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let bcc_label = Label::builder()
            .label("Bcc:")
            .width_chars(8)
            .halign(gtk4::Align::End)
            .build();

        let bcc_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("bcc@example.com")
            .build();

        bcc_box.append(&bcc_label);
        bcc_box.append(&bcc_entry);

        // Subject field
        let subject_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let subject_label = Label::builder()
            .label("Subject:")
            .width_chars(8)
            .halign(gtk4::Align::End)
            .build();

        let subject_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Subject")
            .build();

        subject_box.append(&subject_label);
        subject_box.append(&subject_entry);

        recipients_box.append(&to_box);
        recipients_box.append(&cc_box);
        recipients_box.append(&bcc_box);
        recipients_box.append(&subject_box);

        main_box.append(&recipients_box);
        main_box.append(&Separator::new(Orientation::Horizontal));

        // Attachments area
        let attachments_box = FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .margin_start(12)
            .margin_end(12)
            .margin_top(4)
            .margin_bottom(4)
            .max_children_per_line(10)
            .min_children_per_line(1)
            .visible(false)
            .build();

        main_box.append(&attachments_box);

        // Rich text editor
        let editor = RichTextEditor::new();
        editor.container.set_vexpand(true);

        main_box.append(&editor.container);

        // Bottom toolbar
        let toolbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let attach_btn = Button::builder()
            .icon_name("mail-attachment-symbolic")
            .tooltip_text("Attach file")
            .build();

        let signature_btn = Button::builder()
            .icon_name("format-text-signature-symbolic")
            .tooltip_text("Insert signature")
            .build();

        let spacer = GtkBox::builder()
            .hexpand(true)
            .build();

        let draft_label = Label::builder()
            .label("")
            .css_classes(vec!["dim-label"])
            .build();

        toolbar.append(&attach_btn);
        toolbar.append(&signature_btn);
        toolbar.append(&spacer);
        toolbar.append(&draft_label);

        main_box.append(&Separator::new(Orientation::Horizontal));
        main_box.append(&toolbar);

        window.set_content(Some(&main_box));

        let compose = Self {
            window,
            to_entry,
            cc_entry,
            bcc_entry,
            subject_entry,
            editor,
            attachments: Rc::new(RefCell::new(Vec::new())),
            attachments_box,
            draft_id: Rc::new(RefCell::new(None)),
            reply_to: Rc::new(RefCell::new(None)),
        };

        // Parse mailto if provided
        if let Some(mailto) = mailto {
            compose.parse_mailto(mailto);
        }

        compose.setup_signals(discard_btn, send_btn, attach_btn, signature_btn, draft_label);

        compose
    }

    fn parse_mailto(&self, mailto: &str) {
        // Parse mailto:user@example.com?subject=Subject&body=Body
        if let Some(rest) = mailto.strip_prefix("mailto:") {
            let parts: Vec<&str> = rest.splitn(2, '?').collect();

            if !parts.is_empty() {
                self.to_entry.set_text(parts[0]);
            }

            if parts.len() > 1 {
                for param in parts[1].split('&') {
                    let kv: Vec<&str> = param.splitn(2, '=').collect();
                    if kv.len() == 2 {
                        let value = urlencoding::decode(kv[1]).unwrap_or_default();
                        match kv[0].to_lowercase().as_str() {
                            "subject" => self.subject_entry.set_text(&value),
                            "cc" => self.cc_entry.set_text(&value),
                            "bcc" => self.bcc_entry.set_text(&value),
                            "body" => self.editor.set_text(&value),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn setup_signals(
        &self,
        discard_btn: Button,
        send_btn: Button,
        attach_btn: Button,
        signature_btn: Button,
        draft_label: Label,
    ) {
        // Discard
        let win = self.window.clone();
        discard_btn.connect_clicked(move |_| {
            // TODO: Confirm discard if content exists
            win.close();
        });

        // Send
        let win = self.window.clone();
        let to = self.to_entry.clone();
        let cc = self.cc_entry.clone();
        let bcc = self.bcc_entry.clone();
        let subject = self.subject_entry.clone();
        let editor = self.editor.text_view.clone();
        let attachments = self.attachments.clone();

        send_btn.connect_clicked(move |btn| {
            let to_text = to.text();
            if to_text.is_empty() {
                // Show error
                return;
            }

            btn.set_sensitive(false);
            btn.set_label("Sending...");

            // TODO: Actually send the email
            tracing::info!("Sending email to: {}", to_text);

            glib::timeout_add_local_once(std::time::Duration::from_secs(1), {
                let win = win.clone();
                move || {
                    win.close();
                }
            });
        });

        // Attach file
        let win = self.window.clone();
        let attachments = self.attachments.clone();
        let attachments_flow = self.attachments_box.clone();

        attach_btn.connect_clicked(move |_| {
            let dialog = FileChooserNative::builder()
                .title("Attach File")
                .action(FileChooserAction::Open)
                .transient_for(&win)
                .modal(true)
                .select_multiple(true)
                .build();

            let attachments = attachments.clone();
            let attachments_flow = attachments_flow.clone();

            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    for file in dialog.files().iter::<gio::File>().flatten() {
                        if let Some(path) = file.path() {
                            let filename = path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();

                            let attachment = Attachment {
                                id: uuid::Uuid::new_v4().to_string(),
                                filename: filename.clone(),
                                mime_type: mime_guess::from_path(&path)
                                    .first_or_octet_stream()
                                    .to_string(),
                                size: std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                                data: None,
                            };

                            // Add UI element
                            let chip = Self::create_attachment_chip(&attachment.filename);
                            attachments_flow.append(&chip);
                            attachments_flow.set_visible(true);

                            attachments.borrow_mut().push(attachment);
                        }
                    }
                }
            });

            dialog.show();
        });

        // Signature
        let editor_clone = self.editor.text_view.clone();
        signature_btn.connect_clicked(move |_| {
            let buffer = editor_clone.buffer();
            let mut end = buffer.end_iter();

            let signature = "\n\n--\nSent from Winux Mail";
            buffer.insert(&mut end, signature);
        });

        // Auto-save draft
        let draft_label_clone = draft_label.clone();
        let subject_clone = self.subject_entry.clone();
        let to_clone = self.to_entry.clone();

        glib::timeout_add_local(std::time::Duration::from_secs(30), move || {
            if !subject_clone.text().is_empty() || !to_clone.text().is_empty() {
                draft_label_clone.set_text("Draft saved");

                let label = draft_label_clone.clone();
                glib::timeout_add_local_once(std::time::Duration::from_secs(3), move || {
                    label.set_text("");
                });
            }
            glib::ControlFlow::Continue
        });
    }

    fn create_attachment_chip(filename: &str) -> GtkBox {
        let chip = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .css_classes(vec!["card"])
            .margin_start(2)
            .margin_end(2)
            .margin_top(2)
            .margin_bottom(2)
            .build();

        let icon = gtk4::Image::builder()
            .icon_name("mail-attachment-symbolic")
            .margin_start(8)
            .build();

        let label = Label::builder()
            .label(filename)
            .max_width_chars(20)
            .ellipsize(pango::EllipsizeMode::Middle)
            .build();

        let remove_btn = Button::builder()
            .icon_name("window-close-symbolic")
            .css_classes(vec!["flat", "circular"])
            .margin_end(4)
            .build();

        let chip_clone = chip.clone();
        remove_btn.connect_clicked(move |_| {
            if let Some(parent) = chip_clone.parent() {
                if let Some(flow) = parent.parent() {
                    if let Ok(flow_box) = flow.downcast::<FlowBox>() {
                        if let Some(child) = parent.downcast_ref::<gtk4::FlowBoxChild>() {
                            flow_box.remove(child);
                            if flow_box.first_child().is_none() {
                                flow_box.set_visible(false);
                            }
                        }
                    }
                }
            }
        });

        chip.append(&icon);
        chip.append(&label);
        chip.append(&remove_btn);

        chip
    }

    pub fn set_reply_to(&self, message: &Message) {
        self.to_entry.set_text(&message.from);
        self.subject_entry.set_text(&format!("Re: {}", message.subject));

        // Quote original message
        let quote = if let Some(text) = &message.text_body {
            format!(
                "\n\n\nOn {}, {} wrote:\n{}",
                message.date.format("%B %d, %Y at %H:%M"),
                message.from,
                text.lines().map(|l| format!("> {}", l)).collect::<Vec<_>>().join("\n")
            )
        } else {
            String::new()
        };

        self.editor.set_text(&quote);
        *self.reply_to.borrow_mut() = Some(message.clone());
    }

    pub fn set_forward(&self, message: &Message) {
        self.subject_entry.set_text(&format!("Fwd: {}", message.subject));

        let forward_text = if let Some(text) = &message.text_body {
            format!(
                "\n\n\n---------- Forwarded message ----------\n\
                From: {}\n\
                Date: {}\n\
                Subject: {}\n\
                To: {}\n\n\
                {}",
                message.from,
                message.date.format("%B %d, %Y at %H:%M"),
                message.subject,
                message.to.join(", "),
                text
            )
        } else {
            String::new()
        };

        self.editor.set_text(&forward_text);

        // Copy attachments
        for attachment in &message.attachments {
            self.attachments.borrow_mut().push(attachment.clone());
        }
    }

    pub fn present(&self) {
        self.window.present();
        self.to_entry.grab_focus();
    }
}

// URL encoding helper (minimal implementation)
mod urlencoding {
    use std::borrow::Cow;

    pub fn decode(input: &str) -> Result<Cow<str>, ()> {
        let mut output = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    output.push(byte as char);
                }
            } else if c == '+' {
                output.push(' ');
            } else {
                output.push(c);
            }
        }

        Ok(Cow::Owned(output))
    }
}
