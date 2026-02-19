//! Main window management for the Control Center
//!
//! Handles the popup window creation, positioning, and animations.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation, ScrolledWindow};
use libadwaita as adw;
use tracing::info;

use crate::config::Config;
use crate::widgets;

/// Build the main Control Center window
pub fn build_control_center(app: &Application) {
    let config = Config::load();

    // Create main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Control Center")
        .default_width(config.appearance.width)
        .default_height(600)
        .resizable(false)
        .decorated(false)
        .build();

    window.add_css_class("control-center-window");

    // Create the main content
    let content = build_content(&config);

    // Wrap in scrolled window for long content
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&content)
        .build();

    window.set_child(Some(&scrolled));

    // Handle escape key to close
    let key_controller = gtk::EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            window_clone.close();
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });
    window.add_controller(key_controller);

    // Handle click outside to close (focus lost)
    let focus_controller = gtk::EventControllerFocus::new();
    let window_clone = window.clone();
    focus_controller.connect_leave(move |_| {
        // Slight delay to allow clicking within the window
        let window = window_clone.clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
            if !window.is_active() {
                // Don't auto-close for now as it can be annoying during development
                // window.close();
            }
        });
    });
    window.add_controller(focus_controller);

    // Position the window (top-right corner simulation)
    window.connect_realize(|window| {
        // In a real Wayland setup with layer-shell, we would position this
        // For now, we'll let the window manager handle it
        info!("Control Center window realized");
    });

    // Add slide-up animation class
    window.add_css_class("slide-up");

    window.present();

    info!("Control Center opened");
}

/// Build the main content area
fn build_content(config: &Config) -> Box {
    let main_box = Box::new(Orientation::Vertical, 16);
    main_box.add_css_class("control-center");
    main_box.set_margin_top(8);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(8);
    main_box.set_margin_end(8);

    // Connectivity section (WiFi, Bluetooth, Airplane, AirDrop-like)
    let connectivity = build_connectivity_section(config);
    main_box.append(&connectivity);

    // Quick toggles section (DND, Night Light, etc.)
    let toggles = build_toggles_section(config);
    main_box.append(&toggles);

    // Sliders section (Brightness, Volume)
    let sliders = build_sliders_section(config);
    main_box.append(&sliders);

    // Media player section
    let media = widgets::media::MediaPlayerWidget::new();
    main_box.append(media.widget());

    // Battery section (conditional on laptop)
    let battery = widgets::battery::BatteryWidget::new();
    main_box.append(battery.widget());

    // Screen controls (recording, mirroring)
    let screen_controls = build_screen_controls(config);
    main_box.append(&screen_controls);

    main_box
}

/// Build the connectivity section with WiFi, Bluetooth, Airplane, and AirDrop
fn build_connectivity_section(config: &Config) -> Box {
    let section = Box::new(Orientation::Vertical, 8);

    // 2x2 grid for connectivity toggles
    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(8)
        .column_homogeneous(true)
        .build();

    // WiFi tile
    let wifi = widgets::wifi::WifiWidget::new(config.toggles.wifi_enabled);
    grid.attach(wifi.widget(), 0, 0, 1, 1);

    // Bluetooth tile
    let bluetooth = widgets::bluetooth::BluetoothWidget::new(config.toggles.bluetooth_enabled);
    grid.attach(bluetooth.widget(), 1, 0, 1, 1);

    // Airplane mode tile
    let airplane = widgets::airplane::AirplaneModeWidget::new(config.toggles.airplane_mode);
    grid.attach(airplane.widget(), 0, 1, 1, 1);

    // AirDrop-like / Screen mirroring tile
    let screen_mirror = widgets::quick_toggle::QuickToggleWidget::new(
        "screen-shared-symbolic",
        "Screen Mirror",
        "Off",
        config.toggles.screen_mirroring,
    );
    grid.attach(screen_mirror.widget(), 1, 1, 1, 1);

    section.append(&grid);
    section
}

/// Build the quick toggles section
fn build_toggles_section(config: &Config) -> Box {
    let section = Box::new(Orientation::Vertical, 8);

    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(8)
        .column_homogeneous(true)
        .build();

    // Do Not Disturb
    let dnd = widgets::dnd::DoNotDisturbWidget::new(config.toggles.do_not_disturb);
    grid.attach(dnd.widget(), 0, 0, 1, 1);

    // Night Light
    let night_light = widgets::nightlight::NightLightWidget::new(config.toggles.night_light);
    grid.attach(night_light.widget(), 1, 0, 1, 1);

    section.append(&grid);
    section
}

/// Build the sliders section for brightness and volume
fn build_sliders_section(config: &Config) -> Box {
    let section = Box::new(Orientation::Vertical, 8);

    // Brightness slider
    let brightness = widgets::brightness::BrightnessWidget::new(config.display.brightness);
    section.append(brightness.widget());

    // Volume slider
    let volume = widgets::volume::VolumeWidget::new(config.audio.volume, config.audio.muted);
    section.append(volume.widget());

    section
}

/// Build screen control buttons (recording, mirroring)
fn build_screen_controls(_config: &Config) -> Box {
    let section = Box::new(Orientation::Horizontal, 8);
    section.set_halign(gtk::Align::Center);
    section.set_margin_top(8);

    // Screen recording button
    let record_btn = gtk::Button::builder()
        .icon_name("media-record-symbolic")
        .tooltip_text("Screen Recording")
        .build();
    record_btn.add_css_class("circular");
    record_btn.add_css_class("flat");
    record_btn.connect_clicked(|btn| {
        info!("Screen recording toggled");
        if btn.has_css_class("recording") {
            btn.remove_css_class("recording");
            btn.set_icon_name("media-record-symbolic");
        } else {
            btn.add_css_class("recording");
            btn.set_icon_name("media-playback-stop-symbolic");
        }
    });
    section.append(&record_btn);

    // Screenshot button
    let screenshot_btn = gtk::Button::builder()
        .icon_name("screenshot-symbolic")
        .tooltip_text("Screenshot")
        .build();
    screenshot_btn.add_css_class("circular");
    screenshot_btn.add_css_class("flat");
    screenshot_btn.connect_clicked(|_| {
        info!("Screenshot requested");
        // Launch screenshot tool
        let _ = std::process::Command::new("gnome-screenshot")
            .arg("--interactive")
            .spawn();
    });
    section.append(&screenshot_btn);

    // Settings button
    let settings_btn = gtk::Button::builder()
        .icon_name("emblem-system-symbolic")
        .tooltip_text("Settings")
        .build();
    settings_btn.add_css_class("circular");
    settings_btn.add_css_class("flat");
    settings_btn.connect_clicked(|_| {
        info!("Opening settings");
        let _ = std::process::Command::new("winux-settings")
            .spawn()
            .or_else(|_| std::process::Command::new("gnome-control-center").spawn());
    });
    section.append(&settings_btn);

    section
}
