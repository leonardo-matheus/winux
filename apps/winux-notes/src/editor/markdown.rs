// Winux Notes - Markdown Renderer
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Label, ScrolledWindow, TextBuffer, TextTag, TextTagTable, TextView, Widget, WrapMode};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Markdown renderer that converts markdown to styled GTK text
#[derive(Clone)]
pub struct MarkdownRenderer {
    container: ScrolledWindow,
    text_view: TextView,
    buffer: TextBuffer,
    tag_table: TextTagTable,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        let tag_table = TextTagTable::new();
        Self::setup_tags(&tag_table);

        let buffer = TextBuffer::builder()
            .tag_table(&tag_table)
            .build();

        let text_view = TextView::builder()
            .buffer(&buffer)
            .wrap_mode(WrapMode::Word)
            .editable(false)
            .cursor_visible(false)
            .left_margin(15)
            .right_margin(15)
            .top_margin(10)
            .bottom_margin(10)
            .pixels_above_lines(2)
            .pixels_below_lines(2)
            .build();

        let container = ScrolledWindow::builder()
            .child(&text_view)
            .hexpand(true)
            .vexpand(true)
            .build();

        Self {
            container,
            text_view,
            buffer,
            tag_table,
        }
    }

    fn setup_tags(tag_table: &TextTagTable) {
        // Bold
        let bold = TextTag::builder()
            .name("bold")
            .weight(700)
            .build();
        tag_table.add(&bold);

        // Italic
        let italic = TextTag::builder()
            .name("italic")
            .style(gtk4::pango::Style::Italic)
            .build();
        tag_table.add(&italic);

        // Code inline
        let code = TextTag::builder()
            .name("code")
            .family("monospace")
            .background("#2d2d2d")
            .foreground("#f8f8f2")
            .build();
        tag_table.add(&code);

        // Code block
        let code_block = TextTag::builder()
            .name("code_block")
            .family("monospace")
            .background("#1e1e1e")
            .foreground("#f8f8f2")
            .paragraph_background("#1e1e1e")
            .left_margin(20)
            .build();
        tag_table.add(&code_block);

        // Heading 1
        let h1 = TextTag::builder()
            .name("h1")
            .weight(700)
            .scale(1.8)
            .pixels_above_lines(20)
            .pixels_below_lines(10)
            .build();
        tag_table.add(&h1);

        // Heading 2
        let h2 = TextTag::builder()
            .name("h2")
            .weight(700)
            .scale(1.5)
            .pixels_above_lines(16)
            .pixels_below_lines(8)
            .build();
        tag_table.add(&h2);

        // Heading 3
        let h3 = TextTag::builder()
            .name("h3")
            .weight(700)
            .scale(1.3)
            .pixels_above_lines(12)
            .pixels_below_lines(6)
            .build();
        tag_table.add(&h3);

        // Heading 4
        let h4 = TextTag::builder()
            .name("h4")
            .weight(700)
            .scale(1.1)
            .pixels_above_lines(8)
            .pixels_below_lines(4)
            .build();
        tag_table.add(&h4);

        // Link
        let link = TextTag::builder()
            .name("link")
            .foreground("#64b5f6")
            .underline(gtk4::pango::Underline::Single)
            .build();
        tag_table.add(&link);

        // Quote
        let quote = TextTag::builder()
            .name("quote")
            .style(gtk4::pango::Style::Italic)
            .foreground("#888888")
            .left_margin(20)
            .build();
        tag_table.add(&quote);

        // Strikethrough
        let strikethrough = TextTag::builder()
            .name("strikethrough")
            .strikethrough(true)
            .build();
        tag_table.add(&strikethrough);

        // List item
        let list_item = TextTag::builder()
            .name("list_item")
            .left_margin(20)
            .build();
        tag_table.add(&list_item);
    }

    pub fn widget(&self) -> &Widget {
        self.container.upcast_ref()
    }

    pub fn set_markdown(&self, markdown: &str) {
        self.buffer.set_text("");

        let options = Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS;

        let parser = Parser::new_ext(markdown, options);
        let mut tag_stack: Vec<String> = Vec::new();
        let mut list_depth = 0;
        let mut in_code_block = false;

        for event in parser {
            match event {
                Event::Start(tag) => {
                    match tag {
                        Tag::Heading { level, .. } => {
                            let tag_name = match level {
                                HeadingLevel::H1 => "h1",
                                HeadingLevel::H2 => "h2",
                                HeadingLevel::H3 => "h3",
                                _ => "h4",
                            };
                            tag_stack.push(tag_name.to_string());
                        }
                        Tag::Paragraph => {}
                        Tag::Strong => {
                            tag_stack.push("bold".to_string());
                        }
                        Tag::Emphasis => {
                            tag_stack.push("italic".to_string());
                        }
                        Tag::Strikethrough => {
                            tag_stack.push("strikethrough".to_string());
                        }
                        Tag::CodeBlock(_) => {
                            in_code_block = true;
                            tag_stack.push("code_block".to_string());
                        }
                        Tag::BlockQuote(_) => {
                            tag_stack.push("quote".to_string());
                        }
                        Tag::Link { .. } => {
                            tag_stack.push("link".to_string());
                        }
                        Tag::List(_) => {
                            list_depth += 1;
                        }
                        Tag::Item => {
                            // Insert list bullet/number
                            let indent = "  ".repeat(list_depth - 1);
                            self.insert_text(&format!("{}* ", indent));
                            tag_stack.push("list_item".to_string());
                        }
                        _ => {}
                    }
                }
                Event::End(tag) => {
                    match tag {
                        TagEnd::Heading(_) | TagEnd::Strong | TagEnd::Emphasis
                        | TagEnd::Strikethrough | TagEnd::BlockQuote | TagEnd::Link => {
                            tag_stack.pop();
                        }
                        TagEnd::CodeBlock => {
                            in_code_block = false;
                            tag_stack.pop();
                            self.insert_text("\n");
                        }
                        TagEnd::Paragraph => {
                            self.insert_text("\n\n");
                        }
                        TagEnd::List(_) => {
                            list_depth -= 1;
                            if list_depth == 0 {
                                self.insert_text("\n");
                            }
                        }
                        TagEnd::Item => {
                            tag_stack.pop();
                            self.insert_text("\n");
                        }
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    self.insert_text_with_tags(&text, &tag_stack);
                }
                Event::Code(code) => {
                    self.insert_text_with_tags(&code, &["code".to_string()]);
                }
                Event::SoftBreak => {
                    self.insert_text(" ");
                }
                Event::HardBreak => {
                    self.insert_text("\n");
                }
                Event::Rule => {
                    self.insert_text("\n---\n\n");
                }
                Event::TaskListMarker(checked) => {
                    let marker = if checked { "[x] " } else { "[ ] " };
                    self.insert_text(marker);
                }
                _ => {}
            }
        }
    }

    fn insert_text(&self, text: &str) {
        let mut end = self.buffer.end_iter();
        self.buffer.insert(&mut end, text);
    }

    fn insert_text_with_tags(&self, text: &str, tag_names: &[String]) {
        let start_offset = self.buffer.end_iter().offset();
        self.insert_text(text);

        let start = self.buffer.iter_at_offset(start_offset);
        let end = self.buffer.end_iter();

        for tag_name in tag_names {
            if let Some(tag) = self.tag_table.lookup(tag_name) {
                self.buffer.apply_tag(&tag, &start, &end);
            }
        }
    }

    pub fn clear(&self) {
        self.buffer.set_text("");
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        // Note: Can't really test GTK widgets in unit tests easily
        // This is more of a compile check
        let markdown = "# Heading\n\nSome **bold** and *italic* text.";
        assert!(!markdown.is_empty());
    }
}
