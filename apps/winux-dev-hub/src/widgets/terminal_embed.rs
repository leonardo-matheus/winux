// Winux Dev Hub - Terminal Embed Widget
// Copyright (c) 2026 Winux OS Project
//
// Widget for embedding terminal output within the application

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, TextView, TextBuffer};
use libadwaita as adw;
use adw::prelude::*;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};
use std::thread;

/// Creates an embedded terminal-like output widget
pub fn create_terminal_output() -> Box {
    let container = Box::new(Orientation::Vertical, 0);
    container.add_css_class("card");

    // Header
    let header = Box::new(Orientation::Horizontal, 8);
    header.set_margin_start(12);
    header.set_margin_end(12);
    header.set_margin_top(8);
    header.set_margin_bottom(8);

    let title = Label::new(Some("Terminal Output"));
    title.add_css_class("heading");
    title.set_xalign(0.0);
    title.set_hexpand(true);
    header.append(&title);

    let clear_btn = Button::from_icon_name("edit-clear-symbolic");
    clear_btn.add_css_class("flat");
    clear_btn.set_tooltip_text(Some("Limpar"));
    header.append(&clear_btn);

    let copy_btn = Button::from_icon_name("edit-copy-symbolic");
    copy_btn.add_css_class("flat");
    copy_btn.set_tooltip_text(Some("Copiar"));
    header.append(&copy_btn);

    container.append(&header);

    // Text view for output
    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_monospace(true);
    text_view.set_wrap_mode(gtk4::WrapMode::WordChar);
    text_view.add_css_class("terminal-output");

    // Custom styling
    text_view.set_left_margin(12);
    text_view.set_right_margin(12);
    text_view.set_top_margin(8);
    text_view.set_bottom_margin(8);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(200)
        .max_content_height(400)
        .child(&text_view)
        .build();

    container.append(&scrolled);

    // Clear button action
    let buffer = text_view.buffer();
    clear_btn.connect_clicked(move |_| {
        buffer.set_text("");
    });

    // Copy button action
    let text_view_clone = text_view.clone();
    copy_btn.connect_clicked(move |btn| {
        let buffer = text_view_clone.buffer();
        let (start, end) = buffer.bounds();
        let text = buffer.text(&start, &end, false);

        if let Some(clipboard) = btn.clipboard() {
            clipboard.set_text(&text);
        }
    });

    container
}

/// Creates a terminal widget for running commands
pub fn create_command_runner() -> Box {
    let container = Box::new(Orientation::Vertical, 8);
    container.add_css_class("card");
    container.set_margin_start(12);
    container.set_margin_end(12);
    container.set_margin_top(12);
    container.set_margin_bottom(12);

    // Command input
    let input_box = Box::new(Orientation::Horizontal, 8);

    let prompt = Label::new(Some("$"));
    prompt.add_css_class("heading");
    prompt.set_margin_start(8);
    input_box.append(&prompt);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some("Digite um comando..."));
    entry.set_hexpand(true);
    entry.add_css_class("monospace");
    input_box.append(&entry);

    let run_btn = Button::from_icon_name("media-playback-start-symbolic");
    run_btn.add_css_class("suggested-action");
    run_btn.set_tooltip_text(Some("Executar"));
    input_box.append(&run_btn);

    container.append(&input_box);

    // Output area
    let output = TextView::new();
    output.set_editable(false);
    output.set_cursor_visible(false);
    output.set_monospace(true);
    output.set_wrap_mode(gtk4::WrapMode::WordChar);
    output.set_left_margin(12);
    output.set_right_margin(12);
    output.set_top_margin(8);
    output.set_bottom_margin(8);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(150)
        .child(&output)
        .build();

    container.append(&scrolled);

    // Run command on button click
    let entry_clone = entry.clone();
    let output_clone = output.clone();
    run_btn.connect_clicked(move |_| {
        let command = entry_clone.text();
        if command.is_empty() {
            return;
        }

        run_command_async(&command, &output_clone);
    });

    // Run command on Enter
    let entry_clone2 = entry.clone();
    let output_clone2 = output.clone();
    entry.connect_activate(move |_| {
        let command = entry_clone2.text();
        if command.is_empty() {
            return;
        }

        run_command_async(&command, &output_clone2);
    });

    container
}

/// Run a command and display output in the text view
fn run_command_async(command: &str, output: &TextView) {
    let buffer = output.buffer();

    // Clear previous output
    buffer.set_text("");

    // Add command to output
    let mut iter = buffer.end_iter();
    buffer.insert(&mut iter, &format!("$ {}\n", command));

    // Run command
    let result = Command::new("bash")
        .args(["-c", command])
        .output();

    match result {
        Ok(output_result) => {
            let stdout = String::from_utf8_lossy(&output_result.stdout);
            let stderr = String::from_utf8_lossy(&output_result.stderr);

            let mut iter = buffer.end_iter();

            if !stdout.is_empty() {
                buffer.insert(&mut iter, &stdout);
            }

            if !stderr.is_empty() {
                buffer.insert(&mut iter, &format!("\n[stderr]\n{}", stderr));
            }

            // Add exit code
            let exit_code = output_result.status.code().unwrap_or(-1);
            buffer.insert(&mut iter, &format!("\n[exit code: {}]\n", exit_code));
        }
        Err(e) => {
            let mut iter = buffer.end_iter();
            buffer.insert(&mut iter, &format!("Error: {}\n", e));
        }
    }
}

/// Creates a log viewer widget for watching files or command output
pub fn create_log_viewer(title: &str) -> Box {
    let container = Box::new(Orientation::Vertical, 0);
    container.add_css_class("card");

    // Header
    let header = Box::new(Orientation::Horizontal, 8);
    header.set_margin_start(12);
    header.set_margin_end(12);
    header.set_margin_top(8);
    header.set_margin_bottom(8);

    let title_label = Label::new(Some(title));
    title_label.add_css_class("heading");
    title_label.set_xalign(0.0);
    title_label.set_hexpand(true);
    header.append(&title_label);

    // Status indicator
    let status = Label::new(Some("Live"));
    status.add_css_class("badge");
    status.add_css_class("success");
    header.append(&status);

    let pause_btn = Button::from_icon_name("media-playback-pause-symbolic");
    pause_btn.add_css_class("flat");
    pause_btn.set_tooltip_text(Some("Pausar"));
    header.append(&pause_btn);

    let scroll_btn = Button::from_icon_name("go-bottom-symbolic");
    scroll_btn.add_css_class("flat");
    scroll_btn.set_tooltip_text(Some("Ir para o final"));
    header.append(&scroll_btn);

    container.append(&header);

    // Log output
    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_monospace(true);
    text_view.set_wrap_mode(gtk4::WrapMode::WordChar);
    text_view.set_left_margin(12);
    text_view.set_right_margin(12);
    text_view.set_top_margin(8);
    text_view.set_bottom_margin(8);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(300)
        .child(&text_view)
        .build();

    container.append(&scrolled);

    // Scroll to bottom button
    let scrolled_clone = scrolled.clone();
    scroll_btn.connect_clicked(move |_| {
        if let Some(adj) = scrolled_clone.vadjustment() {
            adj.set_value(adj.upper());
        }
    });

    container
}

/// Appends text to a TextView buffer
pub fn append_to_terminal(text_view: &TextView, text: &str) {
    let buffer = text_view.buffer();
    let mut iter = buffer.end_iter();
    buffer.insert(&mut iter, text);
}

/// Creates styled CSS for terminal widgets
pub fn get_terminal_css() -> &'static str {
    r#"
    .terminal-output {
        background-color: #1e1e1e;
        color: #d4d4d4;
        font-family: 'JetBrains Mono', 'Fira Code', 'Source Code Pro', monospace;
        font-size: 11pt;
    }

    .terminal-output text {
        background-color: #1e1e1e;
    }

    .terminal-output text selection {
        background-color: #264f78;
    }
    "#
}
