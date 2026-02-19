// Message Bubble - Individual chat message display

use crate::chat::{Message, MessageRole};
use crate::ui::CodeBlock;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Align};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, CodeBlockKind};

#[derive(Clone)]
pub struct MessageBubble {
    pub widget: GtkBox,
}

impl MessageBubble {
    pub fn new(message: &Message) -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 4);
        widget.add_css_class("message-bubble");

        // Set alignment based on role
        match message.role {
            MessageRole::User => {
                widget.add_css_class("user");
                widget.set_halign(Align::End);
            }
            MessageRole::Assistant => {
                widget.add_css_class("assistant");
                widget.set_halign(Align::Start);
            }
            MessageRole::System => {
                widget.add_css_class("system");
                widget.set_halign(Align::Center);
            }
        }

        // Parse and render content
        let content = message.get_text();
        Self::render_markdown(&widget, &content);

        // Timestamp
        let timestamp = Label::new(Some(&message.timestamp.format("%H:%M").to_string()));
        timestamp.add_css_class("message-timestamp");
        timestamp.set_xalign(if message.role == MessageRole::User { 1.0 } else { 0.0 });
        widget.append(&timestamp);

        Self { widget }
    }

    fn render_markdown(container: &GtkBox, text: &str) {
        let parser = Parser::new(text);
        let mut current_text = String::new();
        let mut in_code_block = false;
        let mut code_language = String::new();
        let mut code_content = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    // Flush current text
                    if !current_text.is_empty() {
                        Self::add_text_label(container, &current_text);
                        current_text.clear();
                    }

                    in_code_block = true;
                    code_language = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        CodeBlockKind::Indented => String::new(),
                    };
                }
                Event::End(TagEnd::CodeBlock) => {
                    // Add code block widget
                    let code_block = CodeBlock::new(&code_content, &code_language);
                    container.append(&code_block.widget);

                    in_code_block = false;
                    code_content.clear();
                    code_language.clear();
                }
                Event::Text(text) => {
                    if in_code_block {
                        code_content.push_str(&text);
                    } else {
                        current_text.push_str(&text);
                    }
                }
                Event::Code(code) => {
                    // Inline code
                    current_text.push_str(&format!("`{}`", code));
                }
                Event::SoftBreak | Event::HardBreak => {
                    current_text.push('\n');
                }
                Event::Start(Tag::Paragraph) => {
                    if !current_text.is_empty() {
                        current_text.push_str("\n\n");
                    }
                }
                Event::Start(Tag::List(_)) => {
                    if !current_text.is_empty() && !current_text.ends_with('\n') {
                        current_text.push('\n');
                    }
                }
                Event::Start(Tag::Item) => {
                    current_text.push_str("  • ");
                }
                Event::End(TagEnd::Item) => {
                    current_text.push('\n');
                }
                Event::Start(Tag::Strong) => {
                    current_text.push_str("**");
                }
                Event::End(TagEnd::Strong) => {
                    current_text.push_str("**");
                }
                Event::Start(Tag::Emphasis) => {
                    current_text.push('_');
                }
                Event::End(TagEnd::Emphasis) => {
                    current_text.push('_');
                }
                Event::Start(Tag::Heading { level, .. }) => {
                    if !current_text.is_empty() {
                        current_text.push_str("\n\n");
                    }
                    for _ in 0..level as usize {
                        current_text.push('#');
                    }
                    current_text.push(' ');
                }
                Event::End(TagEnd::Heading(_)) => {
                    current_text.push('\n');
                }
                _ => {}
            }
        }

        // Flush remaining text
        if !current_text.is_empty() {
            Self::add_text_label(container, &current_text);
        }
    }

    fn add_text_label(container: &GtkBox, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        let label = Label::new(Some(trimmed));
        label.set_wrap(true);
        label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
        label.set_xalign(0.0);
        label.set_selectable(true);
        label.add_css_class("message-content");

        // Enable markup for basic formatting
        label.set_use_markup(false); // Disable for now to avoid XSS-like issues

        container.append(&label);
    }
}
