// Winux Mobile Studio - Build Output Panel
// Copyright (c) 2026 Winux OS Project
//
// Bottom panel with build output, terminal, and logs

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Image, Label, Notebook, Orientation, ScrolledWindow,
    TextView, TextBuffer, WrapMode,
};
use libadwaita as adw;
use adw::prelude::*;

pub fn create_build_panel() -> Box {
    let panel = Box::builder()
        .orientation(Orientation::Vertical)
        .height_request(200)
        .build();

    // Panel header with tabs
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .css_classes(vec!["toolbar"])
        .build();

    // Tabs
    let notebook = Notebook::builder()
        .show_border(false)
        .vexpand(true)
        .build();

    // Build Output tab
    let build_output = create_build_output_view();
    let build_label = create_tab_label("Build Output", "system-run-symbolic");
    notebook.append_page(&build_output, Some(&build_label));

    // Terminal tab
    let terminal = create_terminal_placeholder();
    let terminal_label = create_tab_label("Terminal", "utilities-terminal-symbolic");
    notebook.append_page(&terminal, Some(&terminal_label));

    // Logcat tab
    let logcat = create_logcat_view();
    let logcat_label = create_tab_label("Logcat", "view-list-symbolic");
    notebook.append_page(&logcat, Some(&logcat_label));

    // Problems tab
    let problems = create_problems_view();
    let problems_label = create_tab_label("Problemas", "dialog-warning-symbolic");
    notebook.append_page(&problems, Some(&problems_label));

    panel.append(&notebook);
    panel
}

fn create_tab_label(text: &str, icon_name: &str) -> Box {
    let tab_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .build();

    let icon = Image::from_icon_name(icon_name);
    icon.set_pixel_size(14);
    tab_box.append(&icon);

    let label = Label::new(Some(text));
    tab_box.append(&label);

    tab_box
}

fn create_build_output_view() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    // Toolbar
    let toolbar = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .margin_start(10)
        .margin_end(10)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    let clear_btn = Button::builder()
        .icon_name("edit-clear-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Limpar")
        .build();
    toolbar.append(&clear_btn);

    let scroll_btn = Button::builder()
        .icon_name("go-bottom-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Rolar para o fim")
        .build();
    toolbar.append(&scroll_btn);

    let spacer = Box::builder().hexpand(true).build();
    toolbar.append(&spacer);

    let status_label = Label::builder()
        .label("Pronto")
        .css_classes(vec!["dim-label", "caption"])
        .build();
    toolbar.append(&status_label);

    container.append(&toolbar);

    // Output text view
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let buffer = TextBuffer::new(None);
    buffer.set_text(&get_sample_build_output());

    let text_view = TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .monospace(true)
        .wrap_mode(WrapMode::WordChar)
        .left_margin(10)
        .right_margin(10)
        .top_margin(5)
        .bottom_margin(5)
        .build();

    scrolled.set_child(Some(&text_view));
    container.append(&scrolled);

    container
}

fn create_terminal_placeholder() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    // In a real implementation, this would use VTE terminal
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let buffer = TextBuffer::new(None);
    buffer.set_text("$ # Terminal integrado\n$ # Use para executar comandos\n$ flutter --version\nFlutter 3.19.0 \u{2022} channel stable \u{2022} https://github.com/flutter/flutter.git\nFramework \u{2022} revision (3 months ago) \u{2022} 2024-02-15 12:46:53 -0600\nEngine \u{2022} revision\nTools \u{2022} Dart 3.3.0 \u{2022} DevTools 2.31.0\n$ ");

    let text_view = TextView::builder()
        .buffer(&buffer)
        .editable(true)
        .monospace(true)
        .wrap_mode(WrapMode::WordChar)
        .left_margin(10)
        .right_margin(10)
        .top_margin(5)
        .bottom_margin(5)
        .css_classes(vec!["terminal"])
        .build();

    scrolled.set_child(Some(&text_view));
    container.append(&scrolled);

    container
}

fn create_logcat_view() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    // Logcat toolbar
    let toolbar = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .margin_start(10)
        .margin_end(10)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    let filter_entry = gtk4::SearchEntry::builder()
        .placeholder_text("Filtrar logs...")
        .width_request(200)
        .build();
    toolbar.append(&filter_entry);

    let level_combo = gtk4::DropDown::from_strings(&["Verbose", "Debug", "Info", "Warn", "Error"]);
    level_combo.set_selected(2); // Info by default
    toolbar.append(&level_combo);

    let spacer = Box::builder().hexpand(true).build();
    toolbar.append(&spacer);

    let pause_btn = Button::builder()
        .icon_name("media-playback-pause-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Pausar")
        .build();
    toolbar.append(&pause_btn);

    let clear_btn = Button::builder()
        .icon_name("edit-clear-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Limpar")
        .build();
    toolbar.append(&clear_btn);

    container.append(&toolbar);

    // Logcat output
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let buffer = TextBuffer::new(None);
    buffer.set_text(&get_sample_logcat());

    let text_view = TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .monospace(true)
        .wrap_mode(WrapMode::WordChar)
        .left_margin(10)
        .right_margin(10)
        .top_margin(5)
        .bottom_margin(5)
        .build();

    scrolled.set_child(Some(&text_view));
    container.append(&scrolled);

    container
}

fn create_problems_view() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let list = gtk4::ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .build();

    // Sample problems
    let problems = vec![
        ("warning", "Unused import 'dart:async'", "lib/main.dart:3"),
        ("error", "The method 'doSomething' isn't defined", "lib/home.dart:45"),
        ("warning", "Prefer const constructors", "lib/widgets/button.dart:12"),
    ];

    for (level, message, location) in problems {
        let row = create_problem_row(level, message, location);
        list.append(&row);
    }

    scrolled.set_child(Some(&list));
    container.append(&scrolled);

    container
}

fn create_problem_row(level: &str, message: &str, location: &str) -> gtk4::ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    let icon_name = match level {
        "error" => "dialog-error-symbolic",
        "warning" => "dialog-warning-symbolic",
        _ => "dialog-information-symbolic",
    };

    let icon = Image::from_icon_name(icon_name);
    row_box.append(&icon);

    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let message_label = Label::builder()
        .label(message)
        .halign(gtk4::Align::Start)
        .build();
    info_box.append(&message_label);

    let location_label = Label::builder()
        .label(location)
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    info_box.append(&location_label);

    row_box.append(&info_box);

    gtk4::ListBoxRow::builder()
        .child(&row_box)
        .build()
}

fn get_sample_build_output() -> String {
    r#"Launching build...

> Task :app:preBuild UP-TO-DATE
> Task :app:preDebugBuild UP-TO-DATE
> Task :app:compileDebugAidl NO-SOURCE
> Task :app:compileDebugRenderscript NO-SOURCE
> Task :app:generateDebugBuildConfig UP-TO-DATE
> Task :app:checkDebugAarMetadata UP-TO-DATE
> Task :app:generateDebugResValues UP-TO-DATE
> Task :app:generateDebugResources UP-TO-DATE
> Task :app:mergeDebugResources UP-TO-DATE
> Task :app:packageDebugResources UP-TO-DATE
> Task :app:parseDebugLocalResources UP-TO-DATE
> Task :app:processDebugManifest UP-TO-DATE
> Task :app:mergeDebugShaders UP-TO-DATE
> Task :app:compileDebugShaders NO-SOURCE
> Task :app:generateDebugAssets UP-TO-DATE
> Task :app:mergeDebugAssets UP-TO-DATE
> Task :app:compressDebugAssets UP-TO-DATE
> Task :app:processDebugJavaRes NO-SOURCE
> Task :app:mergeDebugJavaResource UP-TO-DATE
> Task :app:javaPreCompileDebug UP-TO-DATE
> Task :app:compileDebugKotlin UP-TO-DATE
> Task :app:compileDebugJavaWithJavac UP-TO-DATE
> Task :app:mergeDebugNativeDebugMetadata NO-SOURCE
> Task :app:mergeDebugJniLibFolders UP-TO-DATE
> Task :app:dexBuilderDebug UP-TO-DATE
> Task :app:mergeProjectDexDebug UP-TO-DATE
> Task :app:packageDebug UP-TO-DATE
> Task :app:assembleDebug UP-TO-DATE

BUILD SUCCESSFUL in 2s
27 actionable tasks: 27 up-to-date

APK gerado: app/build/outputs/apk/debug/app-debug.apk (12.5 MB)
"#.to_string()
}

fn get_sample_logcat() -> String {
    r#"02-18 10:35:21.123  1234  1234 I MyApp   : Application started
02-18 10:35:21.145  1234  1234 D MyApp   : Initializing database...
02-18 10:35:21.167  1234  1234 D MyApp   : Database initialized successfully
02-18 10:35:21.189  1234  1234 I MyApp   : Loading user preferences
02-18 10:35:21.201  1234  1234 D MyApp   : Preferences loaded: theme=dark, lang=pt-BR
02-18 10:35:21.223  1234  1256 I MyApp   : Network request started: GET /api/users
02-18 10:35:21.456  1234  1256 D MyApp   : Network response: 200 OK (233ms)
02-18 10:35:21.478  1234  1234 I MyApp   : User data loaded successfully
02-18 10:35:21.500  1234  1234 D MyApp   : Rendering home screen
02-18 10:35:22.100  1234  1234 W MyApp   : Image cache miss for avatar_123.jpg
02-18 10:35:22.300  1234  1256 I MyApp   : Downloading image: avatar_123.jpg
02-18 10:35:22.789  1234  1256 D MyApp   : Image downloaded and cached
"#.to_string()
}
