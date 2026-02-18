//! Winux Player - Native Video Player for Winux OS
//!
//! A modern media player built with GTK4/Adwaita and GStreamer featuring:
//! - Hardware-accelerated video playback
//! - Playlist support
//! - Subtitles (SRT, ASS, VTT)
//! - Picture-in-Picture mode
//! - Keyboard shortcuts
//! - Speed control and A-B repeat

mod config;
mod controls;
mod player;
mod playlist;
mod subtitles;

pub use config::Config;
pub use controls::VideoControls;
pub use player::PlayerWidget;
pub use playlist::PlaylistManager;
pub use subtitles::SubtitleManager;

use anyhow::Result;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{gdk, gio};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

/// Application ID for Winux Player
const APP_ID: &str = "org.winux.player";

/// Supported video formats
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "webm", "mov", "flv", "wmv", "ogv", "m4v", "3gp",
];

/// Supported audio formats
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "aac", "ogg", "m4a", "wma", "opus"];

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Winux Player");

    // Initialize GStreamer
    gstreamer::init()?;
    info!("GStreamer initialized");

    // Load configuration
    let config = Config::load().unwrap_or_default();

    // Initialize GTK
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    let config_clone = config.clone();
    app.connect_activate(move |app| {
        build_ui(app, &config_clone, None);
    });

    let config_clone = config.clone();
    app.connect_open(move |app, files, _| {
        let uris: Vec<String> = files.iter().filter_map(|f| f.uri().map(|u| u.to_string())).collect();
        build_ui(app, &config_clone, Some(uris));
    });

    // Run the application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);

    Ok(())
}

fn build_ui(app: &adw::Application, config: &Config, initial_files: Option<Vec<String>>) {
    // Check if window already exists
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }

    // Create main window
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Winux Player")
        .default_width(config.window.width)
        .default_height(config.window.height)
        .build();

    // State management
    let is_fullscreen = Rc::new(RefCell::new(false));
    let is_pip = Rc::new(RefCell::new(false));

    // Create overlay for controls
    let overlay = gtk4::Overlay::new();

    // Create player widget
    let player_widget = PlayerWidget::new();

    // Create playlist manager
    let playlist_manager = Rc::new(RefCell::new(PlaylistManager::new()));

    // Create controls
    let controls = VideoControls::new(
        player_widget.clone(),
        playlist_manager.clone(),
    );

    // Create header bar (hidden in fullscreen)
    let header = create_header_bar(&window, &player_widget, &playlist_manager, &is_fullscreen);

    // Create toolbar view
    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);

    // Main content box
    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    // Video area
    let video_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    video_box.set_hexpand(true);
    video_box.set_vexpand(true);

    // Add player video widget
    overlay.set_child(Some(player_widget.video_widget()));

    // Add controls overlay
    let controls_revealer = gtk4::Revealer::new();
    controls_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideUp);
    controls_revealer.set_transition_duration(200);
    controls_revealer.set_reveal_child(true);
    controls_revealer.set_valign(gtk4::Align::End);
    controls_revealer.set_child(Some(controls.widget()));
    overlay.add_overlay(&controls_revealer);

    video_box.append(&overlay);
    main_box.append(&video_box);

    // Sidebar for playlist (toggleable)
    let playlist_sidebar = create_playlist_sidebar(&playlist_manager, &player_widget);
    playlist_sidebar.set_visible(false);
    main_box.append(&playlist_sidebar);

    toolbar_view.set_content(Some(&main_box));
    window.set_content(Some(&toolbar_view));

    // Setup motion controller for showing/hiding controls
    let motion_controller = gtk4::EventControllerMotion::new();
    let controls_revealer_clone = controls_revealer.clone();
    let last_motion_time = Rc::new(RefCell::new(std::time::Instant::now()));
    let last_motion_time_clone = last_motion_time.clone();

    motion_controller.connect_motion(move |_, _x, _y| {
        controls_revealer_clone.set_reveal_child(true);
        *last_motion_time_clone.borrow_mut() = std::time::Instant::now();
    });

    overlay.add_controller(motion_controller);

    // Auto-hide controls after inactivity
    let controls_revealer_clone = controls_revealer.clone();
    let is_fullscreen_clone = is_fullscreen.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        let elapsed = last_motion_time.borrow().elapsed();
        if elapsed > std::time::Duration::from_secs(3) && *is_fullscreen_clone.borrow() {
            controls_revealer_clone.set_reveal_child(false);
        }
        glib::ControlFlow::Continue
    });

    // Setup keyboard shortcuts
    setup_shortcuts(
        &window,
        &player_widget,
        &controls,
        &playlist_sidebar,
        &is_fullscreen,
        &is_pip,
        &header,
    );

    // Apply CSS styling
    apply_css();

    // Load initial files if provided
    if let Some(files) = initial_files {
        for uri in files {
            playlist_manager.borrow_mut().add_item(&uri);
        }
        if let Some(first) = playlist_manager.borrow().current_uri() {
            player_widget.load_uri(&first);
            player_widget.play();
        }
    }

    window.present();
}

fn create_header_bar(
    window: &adw::ApplicationWindow,
    player_widget: &PlayerWidget,
    playlist_manager: &Rc<RefCell<PlaylistManager>>,
    is_fullscreen: &Rc<RefCell<bool>>,
) -> adw::HeaderBar {
    let header = adw::HeaderBar::new();

    // Open file button
    let open_btn = gtk4::Button::from_icon_name("document-open-symbolic");
    open_btn.set_tooltip_text(Some("Open File (Ctrl+O)"));

    let window_clone = window.clone();
    let player_clone = player_widget.clone();
    let playlist_clone = playlist_manager.clone();
    open_btn.connect_clicked(move |_| {
        show_open_dialog(&window_clone, &player_clone, &playlist_clone);
    });
    header.pack_start(&open_btn);

    // Open URL button
    let url_btn = gtk4::Button::from_icon_name("web-browser-symbolic");
    url_btn.set_tooltip_text(Some("Open URL (Ctrl+U)"));

    let window_clone = window.clone();
    let player_clone = player_widget.clone();
    url_btn.connect_clicked(move |_| {
        show_url_dialog(&window_clone, &player_clone);
    });
    header.pack_start(&url_btn);

    // Playlist toggle button
    let playlist_btn = gtk4::ToggleButton::new();
    playlist_btn.set_icon_name("view-list-symbolic");
    playlist_btn.set_tooltip_text(Some("Toggle Playlist (Ctrl+L)"));
    header.pack_end(&playlist_btn);

    // Menu button
    let menu_btn = gtk4::MenuButton::new();
    menu_btn.set_icon_name("open-menu-symbolic");
    menu_btn.set_tooltip_text(Some("Menu"));
    menu_btn.set_menu_model(Some(&create_app_menu()));
    header.pack_end(&menu_btn);

    // Fullscreen button
    let fullscreen_btn = gtk4::Button::from_icon_name("view-fullscreen-symbolic");
    fullscreen_btn.set_tooltip_text(Some("Fullscreen (F11)"));

    let window_clone = window.clone();
    let is_fullscreen_clone = is_fullscreen.clone();
    fullscreen_btn.connect_clicked(move |btn| {
        let mut fs = is_fullscreen_clone.borrow_mut();
        *fs = !*fs;
        if *fs {
            window_clone.fullscreen();
            btn.set_icon_name("view-restore-symbolic");
        } else {
            window_clone.unfullscreen();
            btn.set_icon_name("view-fullscreen-symbolic");
        }
    });
    header.pack_end(&fullscreen_btn);

    header
}

fn create_playlist_sidebar(
    playlist_manager: &Rc<RefCell<PlaylistManager>>,
    player_widget: &PlayerWidget,
) -> gtk4::Box {
    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar.set_width_request(280);
    sidebar.add_css_class("playlist-sidebar");

    // Header
    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    header.set_margin_start(12);
    header.set_margin_end(12);
    header.set_margin_top(12);
    header.set_margin_bottom(8);

    let title = gtk4::Label::new(Some("Playlist"));
    title.add_css_class("title-4");
    title.set_hexpand(true);
    title.set_halign(gtk4::Align::Start);
    header.append(&title);

    let clear_btn = gtk4::Button::from_icon_name("edit-clear-all-symbolic");
    clear_btn.set_tooltip_text(Some("Clear Playlist"));
    let playlist_clone = playlist_manager.clone();
    clear_btn.connect_clicked(move |_| {
        playlist_clone.borrow_mut().clear();
    });
    header.append(&clear_btn);

    sidebar.append(&header);

    // Separator
    sidebar.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    // Playlist scroll area
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    list_box.add_css_class("navigation-sidebar");

    // Connect to playlist manager
    let player_clone = player_widget.clone();
    list_box.connect_row_activated(move |_list, row| {
        if let Some(uri) = row.widget_name().map(|n| n.to_string()) {
            player_clone.load_uri(&uri);
            player_clone.play();
        }
    });

    scroll.set_child(Some(&list_box));
    sidebar.append(&scroll);

    sidebar
}

fn create_app_menu() -> gio::Menu {
    let menu = gio::Menu::new();

    // Playback section
    let playback_section = gio::Menu::new();
    playback_section.append(Some("Play/Pause"), Some("app.play-pause"));
    playback_section.append(Some("Stop"), Some("app.stop"));
    playback_section.append(Some("Previous"), Some("app.previous"));
    playback_section.append(Some("Next"), Some("app.next"));
    menu.append_section(Some("Playback"), &playback_section);

    // Speed section
    let speed_section = gio::Menu::new();
    speed_section.append(Some("0.25x"), Some("app.speed::0.25"));
    speed_section.append(Some("0.5x"), Some("app.speed::0.5"));
    speed_section.append(Some("1.0x (Normal)"), Some("app.speed::1.0"));
    speed_section.append(Some("1.5x"), Some("app.speed::1.5"));
    speed_section.append(Some("2.0x"), Some("app.speed::2.0"));
    menu.append_section(Some("Speed"), &speed_section);

    // View section
    let view_section = gio::Menu::new();
    view_section.append(Some("Fullscreen"), Some("app.fullscreen"));
    view_section.append(Some("Picture-in-Picture"), Some("app.pip"));
    view_section.append(Some("Always on Top"), Some("app.always-on-top"));
    menu.append_section(Some("View"), &view_section);

    // Tools section
    let tools_section = gio::Menu::new();
    tools_section.append(Some("Load Subtitles..."), Some("app.load-subtitles"));
    tools_section.append(Some("A-B Repeat"), Some("app.ab-repeat"));
    tools_section.append(Some("Screenshot"), Some("app.screenshot"));
    menu.append_section(Some("Tools"), &tools_section);

    // Help section
    let help_section = gio::Menu::new();
    help_section.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));
    help_section.append(Some("About Winux Player"), Some("app.about"));
    menu.append_section(None, &help_section);

    menu
}

fn setup_shortcuts(
    window: &adw::ApplicationWindow,
    player_widget: &PlayerWidget,
    controls: &VideoControls,
    playlist_sidebar: &gtk4::Box,
    is_fullscreen: &Rc<RefCell<bool>>,
    is_pip: &Rc<RefCell<bool>>,
    header: &adw::HeaderBar,
) {
    let controller = gtk4::EventControllerKey::new();

    let player = player_widget.clone();
    let controls_clone = controls.clone();
    let sidebar = playlist_sidebar.clone();
    let fs = is_fullscreen.clone();
    let pip = is_pip.clone();
    let window_clone = window.clone();
    let header_clone = header.clone();

    controller.connect_key_pressed(move |_, key, _code, modifiers| {
        let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
        let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);

        match key {
            // Space - Play/Pause
            gdk::Key::space | gdk::Key::k | gdk::Key::K => {
                player.toggle_play();
                glib::Propagation::Stop
            }

            // F11 - Fullscreen
            gdk::Key::F11 => {
                let mut fullscreen = fs.borrow_mut();
                *fullscreen = !*fullscreen;
                if *fullscreen {
                    window_clone.fullscreen();
                    header_clone.set_visible(false);
                } else {
                    window_clone.unfullscreen();
                    header_clone.set_visible(true);
                }
                glib::Propagation::Stop
            }

            // Escape - Exit fullscreen
            gdk::Key::Escape => {
                let mut fullscreen = fs.borrow_mut();
                if *fullscreen {
                    *fullscreen = false;
                    window_clone.unfullscreen();
                    header_clone.set_visible(true);
                }
                glib::Propagation::Stop
            }

            // Left/Right - Seek
            gdk::Key::Left => {
                let seek_amount = if ctrl { 60.0 } else if shift { 5.0 } else { 10.0 };
                player.seek_relative(-seek_amount);
                glib::Propagation::Stop
            }
            gdk::Key::Right => {
                let seek_amount = if ctrl { 60.0 } else if shift { 5.0 } else { 10.0 };
                player.seek_relative(seek_amount);
                glib::Propagation::Stop
            }

            // Up/Down - Volume
            gdk::Key::Up => {
                player.adjust_volume(0.05);
                glib::Propagation::Stop
            }
            gdk::Key::Down => {
                player.adjust_volume(-0.05);
                glib::Propagation::Stop
            }

            // M - Mute
            gdk::Key::m | gdk::Key::M => {
                player.toggle_mute();
                glib::Propagation::Stop
            }

            // F - Fullscreen (alternate)
            gdk::Key::f | gdk::Key::F if !ctrl => {
                let mut fullscreen = fs.borrow_mut();
                *fullscreen = !*fullscreen;
                if *fullscreen {
                    window_clone.fullscreen();
                    header_clone.set_visible(false);
                } else {
                    window_clone.unfullscreen();
                    header_clone.set_visible(true);
                }
                glib::Propagation::Stop
            }

            // P - Picture-in-Picture
            gdk::Key::p | gdk::Key::P if !ctrl => {
                let mut pip_mode = pip.borrow_mut();
                *pip_mode = !*pip_mode;
                // PiP mode implementation would go here
                glib::Propagation::Stop
            }

            // L - Toggle playlist
            gdk::Key::l | gdk::Key::L if ctrl => {
                sidebar.set_visible(!sidebar.is_visible());
                glib::Propagation::Stop
            }

            // Number keys - Speed control
            gdk::Key::_1 if shift => {
                player.set_speed(0.25);
                glib::Propagation::Stop
            }
            gdk::Key::_2 if shift => {
                player.set_speed(0.5);
                glib::Propagation::Stop
            }
            gdk::Key::_3 if shift => {
                player.set_speed(0.75);
                glib::Propagation::Stop
            }
            gdk::Key::_4 if shift => {
                player.set_speed(1.0);
                glib::Propagation::Stop
            }
            gdk::Key::_5 if shift => {
                player.set_speed(1.25);
                glib::Propagation::Stop
            }
            gdk::Key::_6 if shift => {
                player.set_speed(1.5);
                glib::Propagation::Stop
            }
            gdk::Key::_7 if shift => {
                player.set_speed(1.75);
                glib::Propagation::Stop
            }
            gdk::Key::_8 if shift => {
                player.set_speed(2.0);
                glib::Propagation::Stop
            }

            // [ ] - A-B repeat points
            gdk::Key::bracketleft => {
                controls_clone.set_a_point();
                glib::Propagation::Stop
            }
            gdk::Key::bracketright => {
                controls_clone.set_b_point();
                glib::Propagation::Stop
            }
            gdk::Key::backslash => {
                controls_clone.clear_ab_repeat();
                glib::Propagation::Stop
            }

            // Home/End - Beginning/End
            gdk::Key::Home => {
                player.seek_absolute(0.0);
                glib::Propagation::Stop
            }
            gdk::Key::End => {
                player.seek_to_end();
                glib::Propagation::Stop
            }

            // S - Stop
            gdk::Key::s | gdk::Key::S if !ctrl => {
                player.stop();
                glib::Propagation::Stop
            }

            _ => glib::Propagation::Proceed,
        }
    });

    window.add_controller(controller);
}

fn show_open_dialog(
    window: &adw::ApplicationWindow,
    player_widget: &PlayerWidget,
    playlist_manager: &Rc<RefCell<PlaylistManager>>,
) {
    let dialog = gtk4::FileDialog::builder()
        .title("Open Media File")
        .modal(true)
        .build();

    // Create file filter for media files
    let filter = gtk4::FileFilter::new();
    filter.set_name(Some("Media Files"));

    for ext in VIDEO_EXTENSIONS.iter().chain(AUDIO_EXTENSIONS.iter()) {
        filter.add_suffix(ext);
    }

    let filters = gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter);

    // All files filter
    let all_filter = gtk4::FileFilter::new();
    all_filter.set_name(Some("All Files"));
    all_filter.add_pattern("*");
    filters.append(&all_filter);

    dialog.set_filters(Some(&filters));

    let player = player_widget.clone();
    let playlist = playlist_manager.clone();
    dialog.open_multiple(Some(window), None::<&gio::Cancellable>, move |result| {
        if let Ok(files) = result {
            for i in 0..files.n_items() {
                if let Some(file) = files.item(i).and_then(|f| f.downcast::<gio::File>().ok()) {
                    if let Some(uri) = file.uri() {
                        playlist.borrow_mut().add_item(&uri.to_string());
                    }
                }
            }
            if let Some(uri) = playlist.borrow().current_uri() {
                player.load_uri(&uri);
                player.play();
            }
        }
    });
}

fn show_url_dialog(window: &adw::ApplicationWindow, player_widget: &PlayerWidget) {
    let dialog = adw::MessageDialog::new(Some(window), Some("Open URL"), None);
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("open", "Open");
    dialog.set_default_response(Some("open"));
    dialog.set_response_appearance("open", adw::ResponseAppearance::Suggested);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some("Enter URL (http://, https://, rtsp://)"));
    entry.set_width_chars(50);
    entry.set_margin_start(12);
    entry.set_margin_end(12);
    entry.set_margin_top(12);
    entry.set_margin_bottom(12);

    dialog.set_extra_child(Some(&entry));

    let player = player_widget.clone();
    let entry_clone = entry.clone();
    dialog.connect_response(None, move |dialog, response| {
        if response == "open" {
            let url = entry_clone.text().to_string();
            if !url.is_empty() {
                player.load_uri(&url);
                player.play();
            }
        }
        dialog.close();
    });

    dialog.present();
}

fn apply_css() {
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        r#"
        .player-container {
            background-color: #000000;
        }

        .controls-overlay {
            background: linear-gradient(transparent, rgba(0, 0, 0, 0.8));
            padding: 12px;
        }

        .progress-bar {
            min-height: 8px;
        }

        .progress-bar:hover {
            min-height: 12px;
        }

        .time-label {
            font-family: monospace;
            font-size: 12px;
            color: #ffffff;
        }

        .control-button {
            min-width: 36px;
            min-height: 36px;
            border-radius: 18px;
            padding: 8px;
        }

        .control-button:hover {
            background-color: rgba(255, 255, 255, 0.1);
        }

        .play-button {
            min-width: 48px;
            min-height: 48px;
            border-radius: 24px;
        }

        .volume-slider {
            min-width: 100px;
        }

        .playlist-sidebar {
            background-color: @window_bg_color;
            border-left: 1px solid @borders;
        }

        .speed-indicator {
            font-weight: bold;
            color: @accent_color;
        }

        .ab-repeat-active {
            color: #ff6b6b;
        }
        "#,
    );

    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
