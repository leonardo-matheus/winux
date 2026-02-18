// Winux Terminal - GPU-accelerated terminal emulator
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Notebook, Label, Orientation, HeaderBar};
use libadwaita as adw;
use vte4::{Terminal, PtyFlags};
use glib::clone;
use std::env;

const APP_ID: &str = "org.winux.terminal";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    let new_tab_btn = Button::builder()
        .icon_name("tab-new-symbolic")
        .tooltip_text("New Tab (Ctrl+Shift+T)")
        .build();

    let menu_btn = Button::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Menu")
        .build();

    header.pack_start(&new_tab_btn);
    header.pack_end(&menu_btn);

    let notebook = Notebook::builder()
        .scrollable(true)
        .show_border(false)
        .build();

    add_terminal_tab(&notebook);

    new_tab_btn.connect_clicked(clone!(@weak notebook => move |_| {
        add_terminal_tab(&notebook);
    }));

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&notebook);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Terminal")
        .default_width(900)
        .default_height(600)
        .build();

    window.set_titlebar(Some(&header));
    window.set_child(Some(&main_box));

    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_string(r#"
        window { background-color: #1a1b26; }
        notebook tab { padding: 8px 16px; background-color: #24283b; }
        notebook tab:checked { background-color: #1a1b26; }
    "#);

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.present();
}

fn add_terminal_tab(notebook: &Notebook) {
    let terminal = Terminal::new();
    terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
    terminal.set_scrollback_lines(10000);

    let font_desc = pango::FontDescription::from_string("Monospace 11");
    terminal.set_font(Some(&font_desc));

    let fg = gtk4::gdk::RGBA::parse("#c0caf5").unwrap();
    let bg = gtk4::gdk::RGBA::parse("#1a1b26").unwrap();
    let palette: [gtk4::gdk::RGBA; 16] = [
        gtk4::gdk::RGBA::parse("#15161e").unwrap(),
        gtk4::gdk::RGBA::parse("#f7768e").unwrap(),
        gtk4::gdk::RGBA::parse("#9ece6a").unwrap(),
        gtk4::gdk::RGBA::parse("#e0af68").unwrap(),
        gtk4::gdk::RGBA::parse("#7aa2f7").unwrap(),
        gtk4::gdk::RGBA::parse("#bb9af7").unwrap(),
        gtk4::gdk::RGBA::parse("#7dcfff").unwrap(),
        gtk4::gdk::RGBA::parse("#a9b1d6").unwrap(),
        gtk4::gdk::RGBA::parse("#414868").unwrap(),
        gtk4::gdk::RGBA::parse("#f7768e").unwrap(),
        gtk4::gdk::RGBA::parse("#9ece6a").unwrap(),
        gtk4::gdk::RGBA::parse("#e0af68").unwrap(),
        gtk4::gdk::RGBA::parse("#7aa2f7").unwrap(),
        gtk4::gdk::RGBA::parse("#bb9af7").unwrap(),
        gtk4::gdk::RGBA::parse("#7dcfff").unwrap(),
        gtk4::gdk::RGBA::parse("#c0caf5").unwrap(),
    ];
    terminal.set_colors(Some(&fg), Some(&bg), &palette);

    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    terminal.spawn_async(
        PtyFlags::DEFAULT,
        None,
        &[&shell],
        &[],
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        None::<&gio::Cancellable>,
        |_| {},
    );

    terminal.connect_child_exited(clone!(@weak notebook => move |term, _| {
        if let Some(parent) = term.parent() {
            notebook.remove_page(notebook.page_num(&parent));
            if notebook.n_pages() == 0 {
                if let Some(win) = notebook.root().and_then(|r| r.downcast::<ApplicationWindow>().ok()) {
                    win.close();
                }
            }
        }
    }));

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&terminal)
        .build();

    let tab_label = Label::new(Some(&format!("Terminal {}", notebook.n_pages() + 1)));
    notebook.append_page(&scrolled, Some(&tab_label));
    notebook.set_current_page(Some(notebook.n_pages() - 1));
    terminal.grab_focus();
}
