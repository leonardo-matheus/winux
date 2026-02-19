// Winux Notes - Rich Text Editor
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{TextBuffer, TextIter, TextTag, TextTagTable, TextView, Widget, WrapMode};
use std::cell::RefCell;
use std::rc::Rc;

/// Format actions for rich text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatAction {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Code,
    Heading1,
    Heading2,
    Heading3,
    BulletList,
    NumberedList,
    Quote,
    Link,
    Clear,
}

/// Rich text editor with formatting support
#[derive(Clone)]
pub struct RichTextEditor {
    text_view: TextView,
    buffer: TextBuffer,
    tag_table: TextTagTable,
}

impl RichTextEditor {
    pub fn new() -> Self {
        let tag_table = TextTagTable::new();
        Self::setup_tags(&tag_table);

        let buffer = TextBuffer::builder()
            .tag_table(&tag_table)
            .build();

        let text_view = TextView::builder()
            .buffer(&buffer)
            .wrap_mode(WrapMode::Word)
            .left_margin(15)
            .right_margin(15)
            .top_margin(10)
            .bottom_margin(10)
            .pixels_above_lines(2)
            .pixels_below_lines(2)
            .hexpand(true)
            .vexpand(true)
            .build();

        // Apply dark theme styling
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(
            r#"
            textview {
                background-color: transparent;
                color: #e0e0e0;
            }
            textview text {
                background-color: transparent;
                color: #e0e0e0;
            }
            "#,
        );

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not get default display"),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        Self {
            text_view,
            buffer,
            tag_table,
        }
    }

    fn setup_tags(tag_table: &TextTagTable) {
        // Bold tag
        let bold = TextTag::builder()
            .name("bold")
            .weight(700)
            .build();
        tag_table.add(&bold);

        // Italic tag
        let italic = TextTag::builder()
            .name("italic")
            .style(gtk4::pango::Style::Italic)
            .build();
        tag_table.add(&italic);

        // Underline tag
        let underline = TextTag::builder()
            .name("underline")
            .underline(gtk4::pango::Underline::Single)
            .build();
        tag_table.add(&underline);

        // Strikethrough tag
        let strikethrough = TextTag::builder()
            .name("strikethrough")
            .strikethrough(true)
            .build();
        tag_table.add(&strikethrough);

        // Code tag (monospace)
        let code = TextTag::builder()
            .name("code")
            .family("monospace")
            .background("#2d2d2d")
            .foreground("#f8f8f2")
            .build();
        tag_table.add(&code);

        // Heading 1
        let h1 = TextTag::builder()
            .name("heading1")
            .weight(700)
            .scale(1.6)
            .pixels_above_lines(16)
            .pixels_below_lines(8)
            .build();
        tag_table.add(&h1);

        // Heading 2
        let h2 = TextTag::builder()
            .name("heading2")
            .weight(700)
            .scale(1.4)
            .pixels_above_lines(12)
            .pixels_below_lines(6)
            .build();
        tag_table.add(&h2);

        // Heading 3
        let h3 = TextTag::builder()
            .name("heading3")
            .weight(700)
            .scale(1.2)
            .pixels_above_lines(8)
            .pixels_below_lines(4)
            .build();
        tag_table.add(&h3);

        // Quote
        let quote = TextTag::builder()
            .name("quote")
            .style(gtk4::pango::Style::Italic)
            .foreground("#888888")
            .left_margin(20)
            .build();
        tag_table.add(&quote);

        // Link
        let link = TextTag::builder()
            .name("link")
            .foreground("#64b5f6")
            .underline(gtk4::pango::Underline::Single)
            .build();
        tag_table.add(&link);
    }

    pub fn widget(&self) -> &Widget {
        self.text_view.upcast_ref()
    }

    pub fn apply_format(&self, action: FormatAction) {
        let (has_selection, start, end) = self.buffer.selection_bounds();
        if !has_selection {
            return;
        }

        let mut start = start;
        let mut end = end;

        let tag_name = match action {
            FormatAction::Bold => "bold",
            FormatAction::Italic => "italic",
            FormatAction::Underline => "underline",
            FormatAction::Strikethrough => "strikethrough",
            FormatAction::Code => "code",
            FormatAction::Heading1 => "heading1",
            FormatAction::Heading2 => "heading2",
            FormatAction::Heading3 => "heading3",
            FormatAction::Quote => "quote",
            FormatAction::Link => "link",
            FormatAction::Clear => {
                self.buffer.remove_all_tags(&start, &end);
                return;
            }
            FormatAction::BulletList | FormatAction::NumberedList => {
                self.insert_list(action == FormatAction::NumberedList);
                return;
            }
        };

        if let Some(tag) = self.tag_table.lookup(tag_name) {
            // Toggle tag: if already applied, remove it; otherwise add it
            if start.has_tag(&tag) {
                self.buffer.remove_tag(&tag, &start, &end);
            } else {
                self.buffer.apply_tag(&tag, &start, &end);
            }
        }
    }

    fn insert_list(&self, numbered: bool) {
        let (has_selection, start, end) = self.buffer.selection_bounds();

        if has_selection {
            // Get the selected text
            let text = self.buffer.text(&start, &end, false);
            let lines: Vec<&str> = text.split('\n').collect();

            // Create new list text
            let mut list_text = String::new();
            for (i, line) in lines.iter().enumerate() {
                if numbered {
                    list_text.push_str(&format!("{}. {}\n", i + 1, line.trim()));
                } else {
                    list_text.push_str(&format!("- {}\n", line.trim()));
                }
            }

            // Remove trailing newline
            list_text.pop();

            // Replace selection
            let mut start = start;
            let mut end = end;
            self.buffer.delete(&mut start, &mut end);
            self.buffer.insert(&mut start, &list_text);
        } else {
            // Insert at cursor
            let prefix = if numbered { "1. " } else { "- " };
            self.buffer.insert_at_cursor(prefix);
        }
    }

    pub fn set_content(&self, content: &str) {
        self.buffer.set_text(content);
    }

    pub fn get_content(&self) -> String {
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();
        self.buffer.text(&start, &end, false).to_string()
    }

    pub fn clear(&self) {
        self.buffer.set_text("");
    }

    pub fn insert_at_cursor(&self, text: &str) {
        self.buffer.insert_at_cursor(text);
    }

    pub fn connect_changed<F: Fn() + 'static>(&self, callback: F) {
        self.buffer.connect_changed(move |_| {
            callback();
        });
    }
}

impl Default for RichTextEditor {
    fn default() -> Self {
        Self::new()
    }
}
