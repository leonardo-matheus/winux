// Winux Mail - Rich Text Editor for Compose
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Orientation, ScrolledWindow, Separator, TextBuffer,
    TextIter, TextTag, TextTagTable, TextView, ToggleButton,
};
use pango::{Style, Underline, Weight};

/// Rich text editor widget for composing emails
pub struct RichTextEditor {
    pub container: GtkBox,
    pub text_view: TextView,
    pub buffer: TextBuffer,
    pub toolbar: GtkBox,

    // Format buttons
    pub bold_btn: ToggleButton,
    pub italic_btn: ToggleButton,
    pub underline_btn: ToggleButton,
    pub strikethrough_btn: ToggleButton,

    // Text tags
    tag_bold: TextTag,
    tag_italic: TextTag,
    tag_underline: TextTag,
    tag_strikethrough: TextTag,
    tag_link: TextTag,
    tag_quote: TextTag,
}

impl RichTextEditor {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        // Create tag table
        let tag_table = TextTagTable::new();

        // Bold tag
        let tag_bold = TextTag::builder()
            .name("bold")
            .weight(Weight::Bold.into())
            .build();
        tag_table.add(&tag_bold);

        // Italic tag
        let tag_italic = TextTag::builder()
            .name("italic")
            .style(Style::Italic)
            .build();
        tag_table.add(&tag_italic);

        // Underline tag
        let tag_underline = TextTag::builder()
            .name("underline")
            .underline(Underline::Single)
            .build();
        tag_table.add(&tag_underline);

        // Strikethrough tag
        let tag_strikethrough = TextTag::builder()
            .name("strikethrough")
            .strikethrough(true)
            .build();
        tag_table.add(&tag_strikethrough);

        // Link tag
        let tag_link = TextTag::builder()
            .name("link")
            .foreground("blue")
            .underline(Underline::Single)
            .build();
        tag_table.add(&tag_link);

        // Quote tag
        let tag_quote = TextTag::builder()
            .name("quote")
            .foreground("gray")
            .left_margin(20)
            .style(Style::Italic)
            .build();
        tag_table.add(&tag_quote);

        // Create buffer with tags
        let buffer = TextBuffer::new(Some(&tag_table));

        // Create toolbar
        let toolbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // Format buttons
        let bold_btn = ToggleButton::builder()
            .icon_name("format-text-bold-symbolic")
            .tooltip_text("Bold (Ctrl+B)")
            .build();

        let italic_btn = ToggleButton::builder()
            .icon_name("format-text-italic-symbolic")
            .tooltip_text("Italic (Ctrl+I)")
            .build();

        let underline_btn = ToggleButton::builder()
            .icon_name("format-text-underline-symbolic")
            .tooltip_text("Underline (Ctrl+U)")
            .build();

        let strikethrough_btn = ToggleButton::builder()
            .icon_name("format-text-strikethrough-symbolic")
            .tooltip_text("Strikethrough")
            .build();

        toolbar.append(&bold_btn);
        toolbar.append(&italic_btn);
        toolbar.append(&underline_btn);
        toolbar.append(&strikethrough_btn);

        toolbar.append(&Separator::new(Orientation::Vertical));

        // List buttons
        let bullet_btn = Button::builder()
            .icon_name("view-list-bullet-symbolic")
            .tooltip_text("Bullet list")
            .build();

        let numbered_btn = Button::builder()
            .icon_name("view-list-ordered-symbolic")
            .tooltip_text("Numbered list")
            .build();

        toolbar.append(&bullet_btn);
        toolbar.append(&numbered_btn);

        toolbar.append(&Separator::new(Orientation::Vertical));

        // Quote button
        let quote_btn = Button::builder()
            .icon_name("format-indent-more-symbolic")
            .tooltip_text("Quote")
            .build();

        toolbar.append(&quote_btn);

        toolbar.append(&Separator::new(Orientation::Vertical));

        // Link button
        let link_btn = Button::builder()
            .icon_name("insert-link-symbolic")
            .tooltip_text("Insert link")
            .build();

        // Image button
        let image_btn = Button::builder()
            .icon_name("insert-image-symbolic")
            .tooltip_text("Insert image")
            .build();

        toolbar.append(&link_btn);
        toolbar.append(&image_btn);

        // Spacer
        let spacer = GtkBox::builder()
            .hexpand(true)
            .build();
        toolbar.append(&spacer);

        // Clear formatting
        let clear_btn = Button::builder()
            .icon_name("edit-clear-symbolic")
            .tooltip_text("Clear formatting")
            .build();

        toolbar.append(&clear_btn);

        container.append(&toolbar);
        container.append(&Separator::new(Orientation::Horizontal));

        // Text view in scrolled window
        let scroll = ScrolledWindow::builder()
            .vexpand(true)
            .build();

        let text_view = TextView::builder()
            .buffer(&buffer)
            .wrap_mode(gtk4::WrapMode::Word)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        scroll.set_child(Some(&text_view));
        container.append(&scroll);

        let editor = Self {
            container,
            text_view,
            buffer,
            toolbar,
            bold_btn,
            italic_btn,
            underline_btn,
            strikethrough_btn,
            tag_bold,
            tag_italic,
            tag_underline,
            tag_strikethrough,
            tag_link,
            tag_quote,
        };

        editor.setup_signals(clear_btn, quote_btn, bullet_btn, numbered_btn, link_btn);
        editor.setup_keyboard_shortcuts();

        editor
    }

    fn setup_signals(
        &self,
        clear_btn: Button,
        quote_btn: Button,
        bullet_btn: Button,
        numbered_btn: Button,
        link_btn: Button,
    ) {
        // Bold toggle
        let buffer = self.buffer.clone();
        let tag = self.tag_bold.clone();
        self.bold_btn.connect_toggled(move |btn| {
            if let Some((start, end)) = buffer.selection_bounds() {
                if btn.is_active() {
                    buffer.apply_tag(&tag, &start, &end);
                } else {
                    buffer.remove_tag(&tag, &start, &end);
                }
            }
        });

        // Italic toggle
        let buffer = self.buffer.clone();
        let tag = self.tag_italic.clone();
        self.italic_btn.connect_toggled(move |btn| {
            if let Some((start, end)) = buffer.selection_bounds() {
                if btn.is_active() {
                    buffer.apply_tag(&tag, &start, &end);
                } else {
                    buffer.remove_tag(&tag, &start, &end);
                }
            }
        });

        // Underline toggle
        let buffer = self.buffer.clone();
        let tag = self.tag_underline.clone();
        self.underline_btn.connect_toggled(move |btn| {
            if let Some((start, end)) = buffer.selection_bounds() {
                if btn.is_active() {
                    buffer.apply_tag(&tag, &start, &end);
                } else {
                    buffer.remove_tag(&tag, &start, &end);
                }
            }
        });

        // Strikethrough toggle
        let buffer = self.buffer.clone();
        let tag = self.tag_strikethrough.clone();
        self.strikethrough_btn.connect_toggled(move |btn| {
            if let Some((start, end)) = buffer.selection_bounds() {
                if btn.is_active() {
                    buffer.apply_tag(&tag, &start, &end);
                } else {
                    buffer.remove_tag(&tag, &start, &end);
                }
            }
        });

        // Clear formatting
        let buffer = self.buffer.clone();
        clear_btn.connect_clicked(move |_| {
            if let Some((start, end)) = buffer.selection_bounds() {
                buffer.remove_all_tags(&start, &end);
            }
        });

        // Quote
        let buffer = self.buffer.clone();
        let tag = self.tag_quote.clone();
        quote_btn.connect_clicked(move |_| {
            if let Some((start, end)) = buffer.selection_bounds() {
                buffer.apply_tag(&tag, &start, &end);
            }
        });

        // Bullet list
        let buffer = self.buffer.clone();
        bullet_btn.connect_clicked(move |_| {
            Self::insert_list(&buffer, "- ");
        });

        // Numbered list
        let buffer = self.buffer.clone();
        numbered_btn.connect_clicked(move |_| {
            Self::insert_list(&buffer, "1. ");
        });

        // Link
        let buffer = self.buffer.clone();
        let tag = self.tag_link.clone();
        link_btn.connect_clicked(move |_| {
            if let Some((start, end)) = buffer.selection_bounds() {
                buffer.apply_tag(&tag, &start, &end);
            }
        });

        // Update button states on cursor move
        let bold_btn = self.bold_btn.clone();
        let italic_btn = self.italic_btn.clone();
        let underline_btn = self.underline_btn.clone();
        let strikethrough_btn = self.strikethrough_btn.clone();
        let buffer = self.buffer.clone();
        let tag_bold = self.tag_bold.clone();
        let tag_italic = self.tag_italic.clone();
        let tag_underline = self.tag_underline.clone();
        let tag_strikethrough = self.tag_strikethrough.clone();

        self.buffer.connect_mark_set(move |_, iter, mark| {
            if mark.name().map(|n| n == "insert").unwrap_or(false) {
                bold_btn.set_active(iter.has_tag(&tag_bold));
                italic_btn.set_active(iter.has_tag(&tag_italic));
                underline_btn.set_active(iter.has_tag(&tag_underline));
                strikethrough_btn.set_active(iter.has_tag(&tag_strikethrough));
            }
        });
    }

    fn setup_keyboard_shortcuts(&self) {
        let controller = gtk4::EventControllerKey::new();

        let bold_btn = self.bold_btn.clone();
        let italic_btn = self.italic_btn.clone();
        let underline_btn = self.underline_btn.clone();

        controller.connect_key_pressed(move |_, key, _, modifier| {
            let ctrl = modifier.contains(gdk4::ModifierType::CONTROL_MASK);

            if ctrl {
                match key {
                    gdk4::Key::b => {
                        bold_btn.set_active(!bold_btn.is_active());
                        bold_btn.emit_by_name::<()>("toggled", &[]);
                        return glib::Propagation::Stop;
                    }
                    gdk4::Key::i => {
                        italic_btn.set_active(!italic_btn.is_active());
                        italic_btn.emit_by_name::<()>("toggled", &[]);
                        return glib::Propagation::Stop;
                    }
                    gdk4::Key::u => {
                        underline_btn.set_active(!underline_btn.is_active());
                        underline_btn.emit_by_name::<()>("toggled", &[]);
                        return glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }

            glib::Propagation::Proceed
        });

        self.text_view.add_controller(controller);
    }

    fn insert_list(buffer: &TextBuffer, prefix: &str) {
        if let Some((start, end)) = buffer.selection_bounds() {
            let text = buffer.text(&start, &end, true);
            let lines: Vec<&str> = text.split('\n').collect();

            let numbered = prefix == "1. ";
            let new_text: String = lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    if numbered {
                        format!("{}. {}", i + 1, line)
                    } else {
                        format!("{}{}", prefix, line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            buffer.delete(&mut start.clone(), &mut end.clone());
            buffer.insert(&mut start.clone(), &new_text);
        } else {
            let cursor = buffer.iter_at_mark(&buffer.get_insert());
            buffer.insert(&mut cursor.clone(), prefix);
        }
    }

    pub fn set_text(&self, text: &str) {
        self.buffer.set_text(text);
    }

    pub fn get_text(&self) -> String {
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();
        self.buffer.text(&start, &end, true).to_string()
    }

    pub fn get_html(&self) -> String {
        // Convert rich text to HTML
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();

        let mut html = String::new();
        let mut current = start.clone();
        let mut open_tags: Vec<&str> = Vec::new();

        while current.offset() < end.offset() {
            // Check for tag changes
            let has_bold = current.has_tag(&self.tag_bold);
            let has_italic = current.has_tag(&self.tag_italic);
            let has_underline = current.has_tag(&self.tag_underline);
            let has_strike = current.has_tag(&self.tag_strikethrough);

            // Close tags that are no longer active
            while let Some(tag) = open_tags.last() {
                let should_close = match *tag {
                    "b" => !has_bold,
                    "i" => !has_italic,
                    "u" => !has_underline,
                    "s" => !has_strike,
                    _ => false,
                };

                if should_close {
                    html.push_str(&format!("</{}>", tag));
                    open_tags.pop();
                } else {
                    break;
                }
            }

            // Open new tags
            if has_bold && !open_tags.contains(&"b") {
                html.push_str("<b>");
                open_tags.push("b");
            }
            if has_italic && !open_tags.contains(&"i") {
                html.push_str("<i>");
                open_tags.push("i");
            }
            if has_underline && !open_tags.contains(&"u") {
                html.push_str("<u>");
                open_tags.push("u");
            }
            if has_strike && !open_tags.contains(&"s") {
                html.push_str("<s>");
                open_tags.push("s");
            }

            // Get character
            let ch = current.char();
            match ch {
                '<' => html.push_str("&lt;"),
                '>' => html.push_str("&gt;"),
                '&' => html.push_str("&amp;"),
                '\n' => html.push_str("<br>\n"),
                _ => html.push(ch),
            }

            current.forward_char();
        }

        // Close remaining tags
        for tag in open_tags.iter().rev() {
            html.push_str(&format!("</{}>", tag));
        }

        html
    }

    pub fn insert_signature(&self, signature: &str) {
        let mut end = self.buffer.end_iter();
        self.buffer.insert(&mut end, &format!("\n\n--\n{}", signature));
    }

    pub fn clear(&self) {
        self.buffer.set_text("");
    }

    pub fn focus(&self) {
        self.text_view.grab_focus();
    }
}

/// Simple plain text editor (fallback)
pub struct PlainTextEditor {
    pub container: ScrolledWindow,
    pub text_view: TextView,
    pub buffer: TextBuffer,
}

impl PlainTextEditor {
    pub fn new() -> Self {
        let container = ScrolledWindow::builder()
            .vexpand(true)
            .build();

        let buffer = TextBuffer::new(None::<&TextTagTable>);

        let text_view = TextView::builder()
            .buffer(&buffer)
            .wrap_mode(gtk4::WrapMode::Word)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        container.set_child(Some(&text_view));

        Self {
            container,
            text_view,
            buffer,
        }
    }

    pub fn set_text(&self, text: &str) {
        self.buffer.set_text(text);
    }

    pub fn get_text(&self) -> String {
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();
        self.buffer.text(&start, &end, true).to_string()
    }
}
