//! Messages page - SMS/messaging integration

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::protocol::ConnectionManager;

/// SMS conversation
#[derive(Clone)]
pub struct Conversation {
    pub id: String,
    pub contact_name: String,
    pub phone_number: String,
    pub last_message: String,
    pub timestamp: String,
    pub unread_count: u32,
}

impl Conversation {
    pub fn new(
        id: &str,
        contact_name: &str,
        phone_number: &str,
        last_message: &str,
        timestamp: &str,
        unread_count: u32,
    ) -> Self {
        Self {
            id: id.to_string(),
            contact_name: contact_name.to_string(),
            phone_number: phone_number.to_string(),
            last_message: last_message.to_string(),
            timestamp: timestamp.to_string(),
            unread_count,
        }
    }
}

/// SMS message
#[derive(Clone)]
pub struct SmsMessage {
    pub id: String,
    pub body: String,
    pub timestamp: String,
    pub is_incoming: bool,
    pub status: MessageStatus,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MessageStatus {
    Sent,
    Delivered,
    Read,
    Failed,
}

/// Messages page for SMS/messaging
pub struct MessagesPage {
    widget: gtk4::Box,
    #[allow(dead_code)]
    manager: Rc<RefCell<ConnectionManager>>,
}

impl MessagesPage {
    pub fn new(manager: Rc<RefCell<ConnectionManager>>) -> Self {
        let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        // Left panel - Conversations list
        let left_panel = Self::create_conversations_panel();
        left_panel.set_width_request(300);
        main_box.append(&left_panel);

        // Separator
        let separator = gtk4::Separator::new(gtk4::Orientation::Vertical);
        main_box.append(&separator);

        // Right panel - Chat view
        let right_panel = Self::create_chat_panel();
        right_panel.set_hexpand(true);
        main_box.append(&right_panel);

        Self {
            widget: main_box,
            manager,
        }
    }

    fn create_conversations_panel() -> gtk4::Box {
        let panel = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        header.set_margin_start(12);
        header.set_margin_end(12);
        header.set_margin_top(12);
        header.set_margin_bottom(8);

        let title = gtk4::Label::new(Some("Conversas"));
        title.add_css_class("title-3");
        title.set_hexpand(true);
        title.set_halign(gtk4::Align::Start);
        header.append(&title);

        let new_button = gtk4::Button::from_icon_name("list-add-symbolic");
        new_button.set_tooltip_text(Some("Nova mensagem"));
        header.append(&new_button);

        panel.append(&header);

        // Search entry
        let search = gtk4::SearchEntry::builder()
            .placeholder_text("Buscar conversas...")
            .margin_start(12)
            .margin_end(12)
            .margin_bottom(8)
            .build();
        panel.append(&search);

        // Conversations list
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();

        let list = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .build();
        list.add_css_class("navigation-sidebar");

        // Sample conversations
        let conversations = vec![
            Conversation::new(
                "1",
                "Maria Silva",
                "+55 11 99999-1234",
                "Oi! Voce vem na reuniao hoje?",
                "10:30",
                2,
            ),
            Conversation::new(
                "2",
                "Joao Santos",
                "+55 11 99999-5678",
                "Ok, combinado!",
                "09:15",
                0,
            ),
            Conversation::new(
                "3",
                "Pedro Oliveira",
                "+55 11 99999-9012",
                "Enviei o documento por email",
                "Ontem",
                0,
            ),
            Conversation::new(
                "4",
                "Ana Costa",
                "+55 11 99999-3456",
                "Obrigada pela ajuda!",
                "Ontem",
                0,
            ),
            Conversation::new(
                "5",
                "Carlos Ferreira",
                "+55 11 99999-7890",
                "Voce: Vou verificar e te aviso",
                "Segunda",
                0,
            ),
        ];

        for conv in &conversations {
            let row = Self::create_conversation_row(conv);
            list.append(&row);
        }

        scrolled.set_child(Some(&list));
        panel.append(&scrolled);

        panel
    }

    fn create_conversation_row(conv: &Conversation) -> gtk4::ListBoxRow {
        let row = gtk4::ListBoxRow::new();

        let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        box_.set_margin_start(12);
        box_.set_margin_end(12);
        box_.set_margin_top(8);
        box_.set_margin_bottom(8);

        // Avatar
        let avatar = adw::Avatar::builder()
            .size(40)
            .text(&conv.contact_name)
            .show_initials(true)
            .build();
        box_.append(&avatar);

        // Content
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        content.set_hexpand(true);

        // Name and time row
        let name_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let name = gtk4::Label::new(Some(&conv.contact_name));
        name.set_halign(gtk4::Align::Start);
        name.set_hexpand(true);
        name.add_css_class("heading");
        name_row.append(&name);

        let time = gtk4::Label::new(Some(&conv.timestamp));
        time.add_css_class("dim-label");
        time.add_css_class("caption");
        name_row.append(&time);

        content.append(&name_row);

        // Last message row
        let msg_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let msg = gtk4::Label::new(Some(&conv.last_message));
        msg.set_halign(gtk4::Align::Start);
        msg.set_hexpand(true);
        msg.set_ellipsize(pango::EllipsizeMode::End);
        msg.add_css_class("dim-label");
        msg_row.append(&msg);

        // Unread badge
        if conv.unread_count > 0 {
            let badge = gtk4::Label::new(Some(&conv.unread_count.to_string()));
            badge.add_css_class("badge");
            badge.add_css_class("numeric");
            msg_row.append(&badge);
        }

        content.append(&msg_row);
        box_.append(&content);

        row.set_child(Some(&box_));
        row
    }

    fn create_chat_panel() -> gtk4::Box {
        let panel = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Chat header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        header.set_margin_start(12);
        header.set_margin_end(12);
        header.set_margin_top(12);
        header.set_margin_bottom(12);

        let avatar = adw::Avatar::builder()
            .size(40)
            .text("Maria Silva")
            .show_initials(true)
            .build();
        header.append(&avatar);

        let info = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        info.set_hexpand(true);

        let name = gtk4::Label::new(Some("Maria Silva"));
        name.set_halign(gtk4::Align::Start);
        name.add_css_class("title-4");
        info.append(&name);

        let phone = gtk4::Label::new(Some("+55 11 99999-1234"));
        phone.set_halign(gtk4::Align::Start);
        phone.add_css_class("dim-label");
        phone.add_css_class("caption");
        info.append(&phone);

        header.append(&info);

        // Call button
        let call_btn = gtk4::Button::from_icon_name("call-start-symbolic");
        call_btn.set_tooltip_text(Some("Iniciar chamada"));
        header.append(&call_btn);

        // Menu button
        let menu_btn = gtk4::Button::from_icon_name("view-more-symbolic");
        menu_btn.set_tooltip_text(Some("Mais opcoes"));
        header.append(&menu_btn);

        panel.append(&header);

        // Separator
        panel.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

        // Messages area
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();

        let messages_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        messages_box.set_margin_start(12);
        messages_box.set_margin_end(12);
        messages_box.set_margin_top(12);
        messages_box.set_margin_bottom(12);

        // Sample messages
        let messages = vec![
            ("Oi, tudo bem?", true, "10:00"),
            ("Oi! Tudo otimo, e voce?", false, "10:02"),
            ("Bem tambem! Voce vem na reuniao hoje?", true, "10:05"),
            ("Qual horario mesmo?", false, "10:10"),
            ("15h na sala de conferencias", true, "10:12"),
            ("Perfeito, vou estar la!", false, "10:15"),
            ("Oi! Voce vem na reuniao hoje?", true, "10:30"),
        ];

        for (text, incoming, time) in messages {
            let msg_widget = Self::create_message_bubble(text, incoming, time);
            messages_box.append(&msg_widget);
        }

        scrolled.set_child(Some(&messages_box));
        panel.append(&scrolled);

        // Input area
        let input_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        input_box.set_margin_start(12);
        input_box.set_margin_end(12);
        input_box.set_margin_top(8);
        input_box.set_margin_bottom(12);

        // Emoji button
        let emoji_btn = gtk4::Button::from_icon_name("face-smile-symbolic");
        input_box.append(&emoji_btn);

        // Text entry
        let entry = gtk4::Entry::builder()
            .placeholder_text("Digite uma mensagem...")
            .hexpand(true)
            .build();
        input_box.append(&entry);

        // Attach button
        let attach_btn = gtk4::Button::from_icon_name("mail-attachment-symbolic");
        attach_btn.set_tooltip_text(Some("Anexar arquivo"));
        input_box.append(&attach_btn);

        // Send button
        let send_btn = gtk4::Button::from_icon_name("send-symbolic");
        send_btn.add_css_class("suggested-action");
        send_btn.set_tooltip_text(Some("Enviar mensagem"));
        input_box.append(&send_btn);

        panel.append(&input_box);

        panel
    }

    fn create_message_bubble(text: &str, incoming: bool, time: &str) -> gtk4::Box {
        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        container.set_halign(if incoming { gtk4::Align::Start } else { gtk4::Align::End });

        let bubble = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        bubble.set_margin_start(8);
        bubble.set_margin_end(8);
        bubble.set_margin_top(4);
        bubble.set_margin_bottom(4);

        if incoming {
            bubble.add_css_class("card");
        } else {
            bubble.add_css_class("accent");
            bubble.add_css_class("card");
        }

        // Message text
        let label = gtk4::Label::new(Some(text));
        label.set_wrap(true);
        label.set_wrap_mode(pango::WrapMode::WordChar);
        label.set_max_width_chars(40);
        label.set_halign(gtk4::Align::Start);
        label.set_margin_start(12);
        label.set_margin_end(12);
        label.set_margin_top(8);
        bubble.append(&label);

        // Time
        let time_label = gtk4::Label::new(Some(time));
        time_label.add_css_class("dim-label");
        time_label.add_css_class("caption");
        time_label.set_halign(gtk4::Align::End);
        time_label.set_margin_start(12);
        time_label.set_margin_end(12);
        time_label.set_margin_bottom(8);
        bubble.append(&time_label);

        container.append(&bubble);
        container
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.widget
    }
}
