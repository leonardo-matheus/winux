// Chat View - Main chat display area

use crate::chat::{Conversation, Message, MessageRole};
use crate::ui::MessageBubble;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Label, Orientation, ScrolledWindow, Widget,
    PolicyType, Align,
};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct ChatView {
    pub widget: GtkBox,
    scroll: ScrolledWindow,
    messages_box: GtkBox,
    typing_indicator: GtkBox,
    streaming_label: Rc<RefCell<Option<Label>>>,
    welcome_view: GtkBox,
}

impl ChatView {
    pub fn new() -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        widget.set_hexpand(true);
        widget.set_vexpand(true);
        widget.add_css_class("chat-view");

        // Create scrolled window
        let scroll = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .build();
        scroll.add_css_class("chat-scroll");

        // Messages container
        let messages_box = GtkBox::new(Orientation::Vertical, 8);
        messages_box.set_margin_top(16);
        messages_box.set_margin_bottom(16);
        messages_box.set_margin_start(16);
        messages_box.set_margin_end(16);
        messages_box.add_css_class("messages-container");

        scroll.set_child(Some(&messages_box));

        // Welcome view
        let welcome_view = Self::create_welcome_view();
        messages_box.append(&welcome_view);

        // Typing indicator
        let typing_indicator = Self::create_typing_indicator();
        typing_indicator.set_visible(false);
        messages_box.append(&typing_indicator);

        widget.append(&scroll);

        Self {
            widget,
            scroll,
            messages_box,
            typing_indicator,
            streaming_label: Rc::new(RefCell::new(None)),
            welcome_view,
        }
    }

    fn create_welcome_view() -> GtkBox {
        let container = GtkBox::new(Orientation::Vertical, 16);
        container.add_css_class("welcome-view");
        container.set_valign(Align::Center);
        container.set_halign(Align::Center);

        // Icon
        let icon = gtk4::Image::from_icon_name("face-smile-big-symbolic");
        icon.set_pixel_size(64);
        icon.add_css_class("welcome-icon");
        container.append(&icon);

        // Title
        let title = Label::new(Some("Welcome to Winux AI"));
        title.add_css_class("welcome-title");
        container.append(&title);

        // Subtitle
        let subtitle = Label::new(Some("Your intelligent assistant powered by Azure OpenAI"));
        subtitle.add_css_class("welcome-subtitle");
        container.append(&subtitle);

        // Suggestions
        let suggestions_box = GtkBox::new(Orientation::Horizontal, 8);
        suggestions_box.set_halign(Align::Center);

        let suggestions = [
            ("Explain code", "code-context-symbolic"),
            ("Help with terminal", "utilities-terminal-symbolic"),
            ("Translate text", "accessories-dictionary-symbolic"),
            ("Analyze file", "document-open-symbolic"),
        ];

        for (text, icon_name) in suggestions {
            let btn = Self::create_suggestion_button(text, icon_name);
            suggestions_box.append(&btn);
        }

        container.append(&suggestions_box);
        container
    }

    fn create_suggestion_button(text: &str, icon_name: &str) -> gtk4::Button {
        let content = GtkBox::new(Orientation::Vertical, 8);
        content.set_halign(Align::Center);

        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(24);
        content.append(&icon);

        let label = Label::new(Some(text));
        content.append(&label);

        let button = gtk4::Button::builder()
            .child(&content)
            .build();
        button.add_css_class("suggestion-button");
        button
    }

    fn create_typing_indicator() -> GtkBox {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.add_css_class("typing-indicator");
        container.add_css_class("message-bubble");
        container.add_css_class("assistant");
        container.set_halign(Align::Start);

        let dots = GtkBox::new(Orientation::Horizontal, 4);
        dots.add_css_class("typing-dots");

        for _ in 0..3 {
            let dot = GtkBox::new(Orientation::Horizontal, 0);
            dot.set_size_request(8, 8);
            dots.append(&dot);
        }

        container.append(&dots);

        let label = Label::new(Some("AI is thinking..."));
        label.add_css_class("dim-label");
        container.append(&label);

        container
    }

    /// Add a message to the chat
    pub fn add_message(&self, message: &Message) {
        // Hide welcome view
        self.welcome_view.set_visible(false);

        let bubble = MessageBubble::new(message);
        self.messages_box.insert_child_after(&bubble.widget, Some(&self.welcome_view));

        // Scroll to bottom
        self.scroll_to_bottom();
    }

    /// Load a full conversation
    pub fn load_conversation(&self, conversation: &Conversation) {
        self.clear();

        for message in &conversation.messages {
            self.add_message(message);
        }

        self.scroll_to_bottom();
    }

    /// Clear all messages
    pub fn clear(&self) {
        // Remove all children except welcome and typing indicator
        while let Some(child) = self.messages_box.first_child() {
            if child == self.welcome_view.clone().upcast::<Widget>()
                || child == self.typing_indicator.clone().upcast::<Widget>()
            {
                if let Some(next) = child.next_sibling() {
                    if next != self.typing_indicator.clone().upcast::<Widget>() {
                        self.messages_box.remove(&next);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                self.messages_box.remove(&child);
            }
        }

        self.welcome_view.set_visible(true);
    }

    /// Show typing indicator
    pub fn show_typing_indicator(&self) {
        self.typing_indicator.set_visible(true);
        self.scroll_to_bottom();
    }

    /// Hide typing indicator
    pub fn hide_typing_indicator(&self) {
        self.typing_indicator.set_visible(false);
    }

    /// Start streaming a message
    pub fn start_streaming_message(&self, message: &Message) {
        self.welcome_view.set_visible(false);

        let bubble = GtkBox::new(Orientation::Vertical, 4);
        bubble.add_css_class("message-bubble");
        bubble.add_css_class("assistant");
        bubble.set_halign(Align::Start);

        let label = Label::new(Some(""));
        label.set_wrap(true);
        label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
        label.set_xalign(0.0);
        label.set_selectable(true);
        label.add_css_class("message-content");
        bubble.append(&label);

        // Add cursor
        let cursor = Label::new(Some("|"));
        cursor.add_css_class("streaming-cursor");
        bubble.append(&cursor);

        self.messages_box.insert_child_after(&bubble, Some(&self.welcome_view));

        *self.streaming_label.borrow_mut() = Some(label);
        self.scroll_to_bottom();
    }

    /// Append text to streaming message
    pub fn append_to_streaming_message(&self, text: &str) {
        if let Some(label) = self.streaming_label.borrow().as_ref() {
            let current = label.text();
            label.set_text(&format!("{}{}", current, text));
            self.scroll_to_bottom();
        }
    }

    /// Finish streaming message
    pub fn finish_streaming_message(&self) {
        *self.streaming_label.borrow_mut() = None;

        // Remove cursor from last bubble
        // The bubble will be replaced with a proper MessageBubble on next refresh
    }

    /// Show error message
    pub fn show_error(&self, error: &str) {
        let error_box = GtkBox::new(Orientation::Horizontal, 8);
        error_box.add_css_class("error-message");
        error_box.set_halign(Align::Center);

        let icon = gtk4::Image::from_icon_name("dialog-error-symbolic");
        error_box.append(&icon);

        let label = Label::new(Some(error));
        label.set_wrap(true);
        error_box.append(&label);

        self.messages_box.insert_child_after(&error_box, Some(&self.welcome_view));
        self.scroll_to_bottom();
    }

    fn scroll_to_bottom(&self) {
        let adj = self.scroll.vadjustment();
        // Use idle_add to ensure scroll happens after layout
        glib::idle_add_local_once(move || {
            adj.set_value(adj.upper() - adj.page_size());
        });
    }
}

impl Default for ChatView {
    fn default() -> Self {
        Self::new()
    }
}
