//! UI module - user interface components
//!
//! Provides the GTK4/libadwaita user interface for the screencast application.

pub mod source_picker;
pub mod controls;
pub mod preview;

use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::recording::RecordingState;
use source_picker::SourcePicker;
use controls::RecordingControls;

/// Main application window
pub struct MainWindow {
    pub window: adw::ApplicationWindow,
}

impl MainWindow {
    /// Create a new main window
    pub fn new(app: &gtk::Application, state: Rc<RefCell<AppState>>) -> adw::ApplicationWindow {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Screencast")
            .default_width(500)
            .default_height(600)
            .build();

        // Create header bar
        let header = adw::HeaderBar::new();

        // Add menu button
        let menu_button = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&create_app_menu())
            .build();
        header.pack_end(&menu_button);

        // Main content
        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.append(&header);

        // Toast overlay for notifications
        let toast_overlay = adw::ToastOverlay::new();

        // Scrolled window for main content
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(24)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        // Source selection section
        let source_section = create_source_section(&state);
        main_box.append(&source_section);

        // Recording options section
        let options_section = create_options_section(&state);
        main_box.append(&options_section);

        // Quality settings section
        let quality_section = create_quality_section(&state);
        main_box.append(&quality_section);

        // Recording controls
        let controls = RecordingControls::new(&state);
        main_box.append(&controls.widget());

        scrolled.set_child(Some(&main_box));
        toast_overlay.set_child(Some(&scrolled));
        content.append(&toast_overlay);

        window.set_content(Some(&content));

        // Set up keyboard shortcuts
        setup_shortcuts(&window, &state);

        window
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn create_app_menu() -> gio::Menu {
    let menu = gio::Menu::new();

    let section1 = gio::Menu::new();
    section1.append(Some("Open Recordings Folder"), Some("app.open-folder"));
    section1.append(Some("Preferences"), Some("app.preferences"));
    menu.append_section(None, &section1);

    let section2 = gio::Menu::new();
    section2.append(Some("Keyboard Shortcuts"), Some("win.show-help-overlay"));
    section2.append(Some("About Screencast"), Some("app.about"));
    menu.append_section(None, &section2);

    menu
}

fn create_source_section(state: &Rc<RefCell<AppState>>) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 12);

    // Section title
    let title = gtk::Label::builder()
        .label("Capture Source")
        .css_classes(["title-4"])
        .halign(gtk::Align::Start)
        .build();
    section.append(&title);

    // Source picker
    let picker = SourcePicker::new(state);
    section.append(&picker.widget());

    section
}

fn create_options_section(state: &Rc<RefCell<AppState>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Recording Options")
        .build();

    // Audio toggle - System
    let system_audio_row = adw::SwitchRow::builder()
        .title("System Audio")
        .subtitle("Include audio from applications")
        .build();

    system_audio_row.set_active(state.borrow().include_system_audio);

    let state_clone = state.clone();
    system_audio_row.connect_active_notify(move |row| {
        state_clone.borrow_mut().include_system_audio = row.is_active();
    });

    group.add(&system_audio_row);

    // Audio toggle - Microphone
    let mic_row = adw::SwitchRow::builder()
        .title("Microphone")
        .subtitle("Include microphone input")
        .build();

    mic_row.set_active(state.borrow().include_microphone);

    let state_clone = state.clone();
    mic_row.connect_active_notify(move |row| {
        state_clone.borrow_mut().include_microphone = row.is_active();
    });

    group.add(&mic_row);

    // Mouse clicks visualization
    let mouse_clicks_row = adw::SwitchRow::builder()
        .title("Show Mouse Clicks")
        .subtitle("Highlight clicks in the recording")
        .build();

    mouse_clicks_row.set_active(state.borrow().show_mouse_clicks);

    let state_clone = state.clone();
    mouse_clicks_row.connect_active_notify(move |row| {
        state_clone.borrow_mut().show_mouse_clicks = row.is_active();
    });

    group.add(&mouse_clicks_row);

    // Key presses visualization
    let key_presses_row = adw::SwitchRow::builder()
        .title("Show Key Presses")
        .subtitle("Display pressed keys on screen")
        .build();

    key_presses_row.set_active(state.borrow().show_key_presses);

    let state_clone = state.clone();
    key_presses_row.connect_active_notify(move |row| {
        state_clone.borrow_mut().show_key_presses = row.is_active();
    });

    group.add(&key_presses_row);

    // Timer delay
    let delay_row = adw::SpinRow::builder()
        .title("Delay Before Recording")
        .subtitle("Seconds to wait before starting")
        .adjustment(&gtk::Adjustment::new(
            state.borrow().timer_delay as f64,
            0.0,
            60.0,
            1.0,
            5.0,
            0.0,
        ))
        .build();

    let state_clone = state.clone();
    delay_row.connect_value_notify(move |row| {
        state_clone.borrow_mut().timer_delay = row.value() as u32;
    });

    group.add(&delay_row);

    group
}

fn create_quality_section(state: &Rc<RefCell<AppState>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Quality Settings")
        .build();

    // Resolution
    let resolution_row = adw::ComboRow::builder()
        .title("Resolution")
        .model(&gtk::StringList::new(&[
            "Original",
            "1080p (1920x1080)",
            "720p (1280x720)",
            "480p (854x480)",
        ]))
        .build();

    let state_clone = state.clone();
    resolution_row.connect_selected_notify(move |row| {
        use crate::Resolution;
        let resolution = match row.selected() {
            0 => Resolution::Original,
            1 => Resolution::R1080p,
            2 => Resolution::R720p,
            3 => Resolution::R480p,
            _ => Resolution::Original,
        };
        state_clone.borrow_mut().config.resolution = resolution;
    });

    group.add(&resolution_row);

    // Framerate
    let fps_row = adw::ComboRow::builder()
        .title("Frame Rate")
        .model(&gtk::StringList::new(&["24 FPS", "30 FPS", "60 FPS"]))
        .selected(1) // Default to 30 FPS
        .build();

    let state_clone = state.clone();
    fps_row.connect_selected_notify(move |row| {
        use crate::Framerate;
        let framerate = match row.selected() {
            0 => Framerate::Fps24,
            1 => Framerate::Fps30,
            2 => Framerate::Fps60,
            _ => Framerate::Fps30,
        };
        state_clone.borrow_mut().config.framerate = framerate;
    });

    group.add(&fps_row);

    // Video codec
    let codec_row = adw::ComboRow::builder()
        .title("Video Codec")
        .model(&gtk::StringList::new(&[
            "H.264 (AVC)",
            "H.265 (HEVC)",
            "VP9",
            "AV1",
        ]))
        .build();

    let state_clone = state.clone();
    codec_row.connect_selected_notify(move |row| {
        use crate::VideoCodec;
        let codec = match row.selected() {
            0 => VideoCodec::H264,
            1 => VideoCodec::H265,
            2 => VideoCodec::VP9,
            3 => VideoCodec::AV1,
            _ => VideoCodec::H264,
        };
        state_clone.borrow_mut().config.codec = codec;
    });

    group.add(&codec_row);

    // Output format
    let format_row = adw::ComboRow::builder()
        .title("Output Format")
        .model(&gtk::StringList::new(&["MP4", "WebM", "MKV", "GIF"]))
        .build();

    let state_clone = state.clone();
    format_row.connect_selected_notify(move |row| {
        use crate::OutputFormat;
        let format = match row.selected() {
            0 => OutputFormat::MP4,
            1 => OutputFormat::WebM,
            2 => OutputFormat::MKV,
            3 => OutputFormat::GIF,
            _ => OutputFormat::MP4,
        };
        state_clone.borrow_mut().config.format = format;
    });

    group.add(&format_row);

    // Bitrate
    let bitrate_row = adw::SpinRow::builder()
        .title("Video Bitrate")
        .subtitle("kbps (higher = better quality, larger file)")
        .adjustment(&gtk::Adjustment::new(
            state.borrow().config.video_bitrate as f64,
            1000.0,
            50000.0,
            500.0,
            1000.0,
            0.0,
        ))
        .build();

    let state_clone = state.clone();
    bitrate_row.connect_value_notify(move |row| {
        state_clone.borrow_mut().config.video_bitrate = row.value() as u32;
    });

    group.add(&bitrate_row);

    group
}

fn setup_shortcuts(window: &adw::ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let controller = gtk::ShortcutController::new();
    controller.set_scope(gtk::ShortcutScope::Global);

    // Escape to cancel recording
    let state_clone = state.clone();
    let escape_action = gtk::CallbackAction::new(move |_, _| {
        if state_clone.borrow().recording_state.is_active() {
            crate::recording::cancel_recording(&state_clone);
        }
        glib::Propagation::Stop
    });

    controller.add_shortcut(gtk::Shortcut::new(
        gtk::ShortcutTrigger::parse_string("Escape"),
        Some(escape_action),
    ));

    window.add_controller(controller);
}

/// Show an about dialog
pub fn show_about_dialog(window: &impl IsA<gtk::Window>) {
    let dialog = adw::AboutDialog::builder()
        .application_name("Screencast")
        .application_icon("camera-video-symbolic")
        .version("1.0.0")
        .developer_name("Winux Team")
        .website("https://winux.org")
        .issue_url("https://github.com/winux-os/winux/issues")
        .license_type(gtk::License::Gpl30)
        .developers(vec!["Winux Team".to_string()])
        .copyright("2024 Winux Project")
        .build();

    dialog.present(Some(window));
}

/// Show a notification toast
pub fn show_toast(window: &adw::ApplicationWindow, message: &str) {
    if let Some(content) = window.content() {
        if let Some(overlay) = content.first_child() {
            if let Ok(toast_overlay) = overlay.downcast::<adw::ToastOverlay>() {
                let toast = adw::Toast::new(message);
                toast_overlay.add_toast(toast);
            }
        }
    }
}
