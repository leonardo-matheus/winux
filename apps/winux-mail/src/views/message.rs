// Winux Mail - Message View (Email Content)
// Copyright (c) 2026 Winux OS Project

use crate::data::message::{Attachment, Message};

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, FlowBox, Frame, Label, Orientation, PolicyType,
    ScrolledWindow, Separator, TextView, WrapMode,
};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct MessageView {
    pub container: GtkBox,
    pub header_box: GtkBox,
    pub content_scroll: ScrolledWindow,
    pub attachments_box: FlowBox,
    pub current_message: Rc<RefCell<Option<Message>>>,
    pub subject_label: Label,
    pub from_label: Label,
    pub to_label: Label,
    pub date_label: Label,
    pub body_view: TextView,
    pub on_reply: Rc<RefCell<Option<Box<dyn Fn(&Message)>>>>,
    pub on_forward: Rc<RefCell<Option<Box<dyn Fn(&Message)>>>>,
}

impl MessageView {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .build();

        // Toolbar
        let toolbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let reply_btn = Button::builder()
            .icon_name("mail-reply-sender-symbolic")
            .tooltip_text("Reply")
            .build();

        let reply_all_btn = Button::builder()
            .icon_name("mail-reply-all-symbolic")
            .tooltip_text("Reply All")
            .build();

        let forward_btn = Button::builder()
            .icon_name("mail-forward-symbolic")
            .tooltip_text("Forward")
            .build();

        let star_btn = Button::builder()
            .icon_name("starred-symbolic")
            .tooltip_text("Star")
            .build();

        let archive_btn = Button::builder()
            .icon_name("folder-symbolic")
            .tooltip_text("Archive")
            .build();

        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Delete")
            .build();

        let spacer = GtkBox::builder()
            .hexpand(true)
            .build();

        let print_btn = Button::builder()
            .icon_name("printer-symbolic")
            .tooltip_text("Print")
            .build();

        let more_btn = Button::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("More")
            .build();

        toolbar.append(&reply_btn);
        toolbar.append(&reply_all_btn);
        toolbar.append(&forward_btn);
        toolbar.append(&Separator::new(Orientation::Vertical));
        toolbar.append(&star_btn);
        toolbar.append(&archive_btn);
        toolbar.append(&delete_btn);
        toolbar.append(&spacer);
        toolbar.append(&print_btn);
        toolbar.append(&more_btn);

        container.append(&toolbar);
        container.append(&Separator::new(Orientation::Horizontal));

        // Scrollable content
        let content_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .hscrollbar_policy(PolicyType::Never)
            .build();

        let content_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(16)
            .margin_end(16)
            .margin_top(16)
            .margin_bottom(16)
            .spacing(12)
            .build();

        // Header
        let header_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();

        let subject_label = Label::builder()
            .label("No message selected")
            .css_classes(vec!["title-1"])
            .halign(gtk4::Align::Start)
            .wrap(true)
            .wrap_mode(pango::WrapMode::Word)
            .build();

        let meta_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();

        // From row
        let from_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let from_title = Label::builder()
            .label("From:")
            .css_classes(vec!["dim-label"])
            .width_chars(8)
            .halign(gtk4::Align::Start)
            .build();

        let from_label = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .selectable(true)
            .build();

        from_box.append(&from_title);
        from_box.append(&from_label);

        // To row
        let to_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let to_title = Label::builder()
            .label("To:")
            .css_classes(vec!["dim-label"])
            .width_chars(8)
            .halign(gtk4::Align::Start)
            .build();

        let to_label = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .selectable(true)
            .wrap(true)
            .build();

        to_box.append(&to_title);
        to_box.append(&to_label);

        // Date row
        let date_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let date_title = Label::builder()
            .label("Date:")
            .css_classes(vec!["dim-label"])
            .width_chars(8)
            .halign(gtk4::Align::Start)
            .build();

        let date_label = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .build();

        date_box.append(&date_title);
        date_box.append(&date_label);

        meta_box.append(&from_box);
        meta_box.append(&to_box);
        meta_box.append(&date_box);

        header_box.append(&subject_label);
        header_box.append(&meta_box);

        content_box.append(&header_box);
        content_box.append(&Separator::new(Orientation::Horizontal));

        // Attachments
        let attachments_frame = Frame::builder()
            .margin_top(8)
            .margin_bottom(8)
            .visible(false)
            .build();

        let attachments_inner = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .spacing(8)
            .build();

        let attachments_label = Label::builder()
            .label("Attachments")
            .css_classes(vec!["heading"])
            .halign(gtk4::Align::Start)
            .build();

        let attachments_box = FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .max_children_per_line(5)
            .min_children_per_line(1)
            .build();

        attachments_inner.append(&attachments_label);
        attachments_inner.append(&attachments_box);
        attachments_frame.set_child(Some(&attachments_inner));

        content_box.append(&attachments_frame);

        // Body - text view for plain text, will be replaced with WebView for HTML
        let body_frame = Frame::builder()
            .build();

        let body_view = TextView::builder()
            .editable(false)
            .cursor_visible(false)
            .wrap_mode(WrapMode::Word)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        body_frame.set_child(Some(&body_view));
        content_box.append(&body_frame);

        content_scroll.set_child(Some(&content_box));
        container.append(&content_scroll);

        // Empty state
        let empty_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .spacing(12)
            .vexpand(true)
            .build();

        let empty_icon = gtk4::Image::builder()
            .icon_name("mail-read-symbolic")
            .pixel_size(64)
            .css_classes(vec!["dim-label"])
            .build();

        let empty_label = Label::builder()
            .label("Select a message to read")
            .css_classes(vec!["title-2", "dim-label"])
            .build();

        empty_box.append(&empty_icon);
        empty_box.append(&empty_label);

        let message_view = Self {
            container,
            header_box,
            content_scroll,
            attachments_box,
            current_message: Rc::new(RefCell::new(None)),
            subject_label,
            from_label,
            to_label,
            date_label,
            body_view,
            on_reply: Rc::new(RefCell::new(None)),
            on_forward: Rc::new(RefCell::new(None)),
        };

        message_view.setup_signals(
            reply_btn,
            reply_all_btn,
            forward_btn,
            star_btn,
            archive_btn,
            delete_btn,
        );

        message_view
    }

    fn setup_signals(
        &self,
        reply_btn: Button,
        reply_all_btn: Button,
        forward_btn: Button,
        star_btn: Button,
        archive_btn: Button,
        delete_btn: Button,
    ) {
        let current = self.current_message.clone();
        let on_reply = self.on_reply.clone();
        reply_btn.connect_clicked(move |_| {
            if let Some(msg) = current.borrow().as_ref() {
                if let Some(callback) = on_reply.borrow().as_ref() {
                    callback(msg);
                }
            }
        });

        let current = self.current_message.clone();
        let on_forward = self.on_forward.clone();
        forward_btn.connect_clicked(move |_| {
            if let Some(msg) = current.borrow().as_ref() {
                if let Some(callback) = on_forward.borrow().as_ref() {
                    callback(msg);
                }
            }
        });

        let current = self.current_message.clone();
        star_btn.connect_clicked(move |btn| {
            if let Some(msg) = current.borrow_mut().as_mut() {
                msg.starred = !msg.starred;
                btn.set_icon_name(if msg.starred {
                    "starred-symbolic"
                } else {
                    "non-starred-symbolic"
                });
            }
        });
    }

    pub fn display_message(&mut self, message: &Message) {
        self.subject_label.set_label(&message.subject);
        self.from_label.set_label(&message.from);
        self.to_label.set_label(&message.to.join(", "));
        self.date_label.set_label(&message.date.format("%B %d, %Y at %H:%M").to_string());

        // Display body
        if let Some(html) = &message.html_body {
            // TODO: Use WebKit for HTML rendering
            // For now, fall back to text
            if let Some(text) = &message.text_body {
                self.body_view.buffer().set_text(text);
            } else {
                self.body_view.buffer().set_text("(HTML content - view in browser)");
            }
        } else if let Some(text) = &message.text_body {
            self.body_view.buffer().set_text(text);
        } else {
            self.body_view.buffer().set_text("(No content)");
        }

        // Display attachments
        self.display_attachments(&message.attachments);

        *self.current_message.borrow_mut() = Some(message.clone());
    }

    fn display_attachments(&self, attachments: &[Attachment]) {
        // Clear existing
        while let Some(child) = self.attachments_box.first_child() {
            self.attachments_box.remove(&child);
        }

        for attachment in attachments {
            let btn = Button::builder()
                .build();

            let content = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .margin_start(8)
                .margin_end(8)
                .margin_top(4)
                .margin_bottom(4)
                .build();

            let icon = gtk4::Image::builder()
                .icon_name(Self::icon_for_mime(&attachment.mime_type))
                .build();

            let info_box = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .build();

            let name_label = Label::builder()
                .label(&attachment.filename)
                .halign(gtk4::Align::Start)
                .build();

            let size_label = Label::builder()
                .label(&Self::format_size(attachment.size))
                .css_classes(vec!["dim-label", "caption"])
                .halign(gtk4::Align::Start)
                .build();

            info_box.append(&name_label);
            info_box.append(&size_label);

            content.append(&icon);
            content.append(&info_box);

            btn.set_child(Some(&content));

            let filename = attachment.filename.clone();
            btn.connect_clicked(move |_| {
                tracing::info!("Opening attachment: {}", filename);
            });

            self.attachments_box.append(&btn);
        }

        // Show/hide attachments section
        if let Some(parent) = self.attachments_box.parent() {
            if let Some(frame) = parent.parent() {
                frame.set_visible(!attachments.is_empty());
            }
        }
    }

    fn icon_for_mime(mime_type: &str) -> &'static str {
        if mime_type.starts_with("image/") {
            "image-x-generic-symbolic"
        } else if mime_type.starts_with("audio/") {
            "audio-x-generic-symbolic"
        } else if mime_type.starts_with("video/") {
            "video-x-generic-symbolic"
        } else if mime_type == "application/pdf" {
            "x-office-document-symbolic"
        } else if mime_type.contains("zip") || mime_type.contains("tar") || mime_type.contains("compressed") {
            "package-x-generic-symbolic"
        } else if mime_type.starts_with("text/") {
            "text-x-generic-symbolic"
        } else {
            "text-x-generic-symbolic"
        }
    }

    fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn clear(&mut self) {
        self.subject_label.set_label("No message selected");
        self.from_label.set_label("");
        self.to_label.set_label("");
        self.date_label.set_label("");
        self.body_view.buffer().set_text("");
        *self.current_message.borrow_mut() = None;
    }

    pub fn set_on_reply<F: Fn(&Message) + 'static>(&self, callback: F) {
        *self.on_reply.borrow_mut() = Some(Box::new(callback));
    }

    pub fn set_on_forward<F: Fn(&Message) + 'static>(&self, callback: F) {
        *self.on_forward.borrow_mut() = Some(Box::new(callback));
    }
}
