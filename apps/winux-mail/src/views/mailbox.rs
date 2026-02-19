// Winux Mail - Mailbox View (Email List)
// Copyright (c) 2026 Winux OS Project

use crate::data::message::Message;
use crate::ui::message_row::MessageRow;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, Label, ListBox, Orientation,
    ScrolledWindow, SelectionMode, Separator,
};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct MailboxView {
    pub container: GtkBox,
    pub list_box: ListBox,
    pub messages: Rc<RefCell<Vec<Message>>>,
    pub selected_messages: Rc<RefCell<Vec<String>>>,
    pub select_all_check: CheckButton,
    pub toolbar: GtkBox,
    pub on_message_selected: Rc<RefCell<Option<Box<dyn Fn(&Message)>>>>,
}

impl MailboxView {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .width_request(350)
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

        let select_all_check = CheckButton::builder()
            .tooltip_text("Select all")
            .build();

        let refresh_btn = Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Refresh")
            .build();

        let archive_btn = Button::builder()
            .icon_name("folder-symbolic")
            .tooltip_text("Archive")
            .sensitive(false)
            .build();

        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Delete")
            .sensitive(false)
            .build();

        let spam_btn = Button::builder()
            .icon_name("mail-mark-junk-symbolic")
            .tooltip_text("Mark as spam")
            .sensitive(false)
            .build();

        let more_btn = Button::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("More actions")
            .sensitive(false)
            .build();

        // Spacer
        let spacer = GtkBox::builder()
            .hexpand(true)
            .build();

        // Message count
        let count_label = Label::builder()
            .label("0 messages")
            .css_classes(vec!["dim-label"])
            .build();

        toolbar.append(&select_all_check);
        toolbar.append(&refresh_btn);
        toolbar.append(&Separator::new(Orientation::Vertical));
        toolbar.append(&archive_btn);
        toolbar.append(&delete_btn);
        toolbar.append(&spam_btn);
        toolbar.append(&more_btn);
        toolbar.append(&spacer);
        toolbar.append(&count_label);

        container.append(&toolbar);
        container.append(&Separator::new(Orientation::Horizontal));

        // Message list
        let scroll = ScrolledWindow::builder()
            .vexpand(true)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .css_classes(vec!["boxed-list"])
            .build();

        // Placeholder for empty state
        let placeholder = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .spacing(12)
            .build();

        let icon = gtk4::Image::builder()
            .icon_name("mail-inbox-symbolic")
            .pixel_size(64)
            .css_classes(vec!["dim-label"])
            .build();

        let label = Label::builder()
            .label("No messages")
            .css_classes(vec!["title-2", "dim-label"])
            .build();

        let sublabel = Label::builder()
            .label("Your inbox is empty")
            .css_classes(vec!["dim-label"])
            .build();

        placeholder.append(&icon);
        placeholder.append(&label);
        placeholder.append(&sublabel);

        list_box.set_placeholder(Some(&placeholder));

        scroll.set_child(Some(&list_box));
        container.append(&scroll);

        let mailbox = Self {
            container,
            list_box,
            messages: Rc::new(RefCell::new(Vec::new())),
            selected_messages: Rc::new(RefCell::new(Vec::new())),
            select_all_check,
            toolbar,
            on_message_selected: Rc::new(RefCell::new(None)),
        };

        mailbox.setup_signals(archive_btn, delete_btn, spam_btn, count_label);

        mailbox
    }

    fn setup_signals(
        &self,
        archive_btn: Button,
        delete_btn: Button,
        spam_btn: Button,
        count_label: Label,
    ) {
        // Select all
        let list = self.list_box.clone();
        let messages = self.messages.clone();
        let selected = self.selected_messages.clone();
        self.select_all_check.connect_toggled(move |check| {
            if check.is_active() {
                let all_ids: Vec<String> = messages.borrow().iter().map(|m| m.id.clone()).collect();
                *selected.borrow_mut() = all_ids;
            } else {
                selected.borrow_mut().clear();
            }
            // Update UI
        });

        // Message selection
        let on_selected = self.on_message_selected.clone();
        let messages = self.messages.clone();
        self.list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let idx = row.index() as usize;
                let msgs = messages.borrow();
                if let Some(message) = msgs.get(idx) {
                    if let Some(callback) = on_selected.borrow().as_ref() {
                        callback(message);
                    }
                }
            }
        });
    }

    pub fn set_messages(&mut self, messages: Vec<Message>) {
        // Clear existing
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        // Add new messages
        for message in &messages {
            let row = MessageRow::new(message);
            self.list_box.append(&row.row);
        }

        *self.messages.borrow_mut() = messages;
    }

    pub fn add_message(&mut self, message: Message) {
        let row = MessageRow::new(&message);
        self.list_box.prepend(&row.row);
        self.messages.borrow_mut().insert(0, message);
    }

    pub fn filter(&self, query: &str) {
        let query_lower = query.to_lowercase();

        let mut idx = 0;
        while let Some(row) = self.list_box.row_at_index(idx) {
            let messages = self.messages.borrow();
            if let Some(message) = messages.get(idx as usize) {
                let matches = message.subject.to_lowercase().contains(&query_lower)
                    || message.from.to_lowercase().contains(&query_lower)
                    || message.preview.to_lowercase().contains(&query_lower);

                row.set_visible(matches || query.is_empty());
            }
            idx += 1;
        }
    }

    pub fn set_on_message_selected<F: Fn(&Message) + 'static>(&self, callback: F) {
        *self.on_message_selected.borrow_mut() = Some(Box::new(callback));
    }

    pub fn get_selected_message(&self) -> Option<Message> {
        if let Some(row) = self.list_box.selected_row() {
            let idx = row.index() as usize;
            let messages = self.messages.borrow();
            messages.get(idx).cloned()
        } else {
            None
        }
    }

    pub fn refresh(&self) {
        // Trigger refresh from server
    }

    pub fn mark_selected_as_read(&self) {
        // Mark selected messages as read
    }

    pub fn delete_selected(&self) {
        // Delete selected messages
    }

    pub fn archive_selected(&self) {
        // Archive selected messages
    }
}
