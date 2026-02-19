//! UI module - user interface components

pub mod overlay;
pub mod toolbar;
pub mod preview;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, Button, Box as GtkBox, Orientation, Label, glib};
use libadwaita as adw;
use adw::prelude::*;
use adw::{HeaderBar, ApplicationWindow as AdwApplicationWindow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::capture::{self, CaptureMode};

/// Main application window
pub struct MainWindow {
    window: AdwApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &Application, state: Rc<RefCell<AppState>>) -> Self {
        // Apply dark theme
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

        // Header bar
        let header_bar = HeaderBar::new();
        header_bar.set_title_widget(Some(&Label::new(Some("Winux Screenshot"))));

        // Timer dropdown
        let timer_dropdown = gtk::DropDown::from_strings(&[
            "No delay",
            "3 seconds",
            "5 seconds",
            "10 seconds",
        ]);
        timer_dropdown.set_tooltip_text(Some("Capture delay"));

        {
            let state = state.clone();
            timer_dropdown.connect_selected_notify(move |dropdown| {
                let delay = match dropdown.selected() {
                    0 => 0,
                    1 => 3,
                    2 => 5,
                    3 => 10,
                    _ => 0,
                };
                state.borrow_mut().timer_delay = delay;
            });
        }

        header_bar.pack_end(&timer_dropdown);

        // Capture mode buttons
        let fullscreen_btn = Button::builder()
            .icon_name(CaptureMode::Fullscreen.icon())
            .tooltip_text("Capture entire screen (Print Screen)")
            .build();

        let window_btn = Button::builder()
            .icon_name(CaptureMode::Window.icon())
            .tooltip_text("Capture active window (Alt+Print)")
            .build();

        let region_btn = Button::builder()
            .icon_name(CaptureMode::Region.icon())
            .tooltip_text("Capture selected region (Shift+Print)")
            .build();

        // Mode selection box
        let mode_box = GtkBox::new(Orientation::Horizontal, 0);
        mode_box.add_css_class("linked");
        mode_box.append(&fullscreen_btn);
        mode_box.append(&window_btn);
        mode_box.append(&region_btn);

        // Main content area
        let content_box = GtkBox::new(Orientation::Vertical, 24);
        content_box.set_margin_top(48);
        content_box.set_margin_bottom(48);
        content_box.set_margin_start(48);
        content_box.set_margin_end(48);
        content_box.set_valign(gtk::Align::Center);
        content_box.set_halign(gtk::Align::Center);

        // Icon
        let icon = gtk::Image::from_icon_name("camera-photo-symbolic");
        icon.set_pixel_size(96);
        icon.add_css_class("dim-label");
        content_box.append(&icon);

        // Title
        let title = Label::new(Some("Take a Screenshot"));
        title.add_css_class("title-1");
        content_box.append(&title);

        // Subtitle
        let subtitle = Label::new(Some("Choose how you want to capture your screen"));
        subtitle.add_css_class("dim-label");
        content_box.append(&subtitle);

        // Large capture buttons
        let capture_buttons_box = GtkBox::new(Orientation::Horizontal, 12);
        capture_buttons_box.set_halign(gtk::Align::Center);
        capture_buttons_box.set_margin_top(24);

        let large_fullscreen = create_capture_button(
            "Fullscreen",
            "Capture entire screen",
            CaptureMode::Fullscreen.icon(),
        );

        let large_window = create_capture_button(
            "Window",
            "Capture active window",
            CaptureMode::Window.icon(),
        );

        let large_region = create_capture_button(
            "Region",
            "Select area to capture",
            CaptureMode::Region.icon(),
        );

        capture_buttons_box.append(&large_fullscreen);
        capture_buttons_box.append(&large_window);
        capture_buttons_box.append(&large_region);
        content_box.append(&capture_buttons_box);

        // Keyboard shortcuts info
        let shortcuts_label = Label::new(Some(
            "Keyboard shortcuts: Print Screen (fullscreen) | Alt+Print (window) | Shift+Print (region)"
        ));
        shortcuts_label.add_css_class("dim-label");
        shortcuts_label.add_css_class("caption");
        shortcuts_label.set_margin_top(24);
        content_box.append(&shortcuts_label);

        // Main box
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header_bar);
        main_box.append(&content_box);

        // Window
        let window = AdwApplicationWindow::builder()
            .application(app)
            .title("Winux Screenshot")
            .default_width(600)
            .default_height(400)
            .content(&main_box)
            .build();

        // Connect button handlers
        {
            let app = app.clone();
            let state = state.clone();
            let window_clone = window.clone();
            fullscreen_btn.connect_clicked(move |_| {
                state.borrow_mut().capture_mode = CaptureMode::Fullscreen;
                window_clone.set_visible(false);
                let app = app.clone();
                let state = state.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    capture::start_capture(&app, &state);
                });
            });
        }

        {
            let app = app.clone();
            let state = state.clone();
            let window_clone = window.clone();
            window_btn.connect_clicked(move |_| {
                state.borrow_mut().capture_mode = CaptureMode::Window;
                window_clone.set_visible(false);
                let app = app.clone();
                let state = state.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    capture::start_capture(&app, &state);
                });
            });
        }

        {
            let app = app.clone();
            let state = state.clone();
            let window_clone = window.clone();
            region_btn.connect_clicked(move |_| {
                state.borrow_mut().capture_mode = CaptureMode::Region;
                window_clone.set_visible(false);
                let app = app.clone();
                let state = state.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    capture::start_capture(&app, &state);
                });
            });
        }

        // Large button handlers
        {
            let app = app.clone();
            let state = state.clone();
            let window_clone = window.clone();
            large_fullscreen.connect_clicked(move |_| {
                state.borrow_mut().capture_mode = CaptureMode::Fullscreen;
                window_clone.set_visible(false);
                let app = app.clone();
                let state = state.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    capture::start_capture(&app, &state);
                });
            });
        }

        {
            let app = app.clone();
            let state = state.clone();
            let window_clone = window.clone();
            large_window.connect_clicked(move |_| {
                state.borrow_mut().capture_mode = CaptureMode::Window;
                window_clone.set_visible(false);
                let app = app.clone();
                let state = state.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    capture::start_capture(&app, &state);
                });
            });
        }

        {
            let app = app.clone();
            let state = state.clone();
            let window_clone = window.clone();
            large_region.connect_clicked(move |_| {
                state.borrow_mut().capture_mode = CaptureMode::Region;
                window_clone.set_visible(false);
                let app = app.clone();
                let state = state.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    capture::start_capture(&app, &state);
                });
            });
        }

        // Keyboard shortcuts
        let controller = gtk::EventControllerKey::new();
        {
            let app = app.clone();
            let state = state.clone();
            let window_clone = window.clone();
            controller.connect_key_pressed(move |_, key, _, modifier| {
                use gtk::gdk::Key;

                let mode = if modifier.contains(gtk::gdk::ModifierType::ALT_MASK) {
                    Some(CaptureMode::Window)
                } else if modifier.contains(gtk::gdk::ModifierType::SHIFT_MASK) {
                    Some(CaptureMode::Region)
                } else if key == Key::Print {
                    Some(CaptureMode::Fullscreen)
                } else {
                    None
                };

                if let Some(mode) = mode {
                    state.borrow_mut().capture_mode = mode;
                    window_clone.set_visible(false);
                    let app = app.clone();
                    let state = state.clone();
                    glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                        capture::start_capture(&app, &state);
                    });
                    return glib::Propagation::Stop;
                }

                glib::Propagation::Proceed
            });
        }
        window.add_controller(controller);

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn create_capture_button(title: &str, subtitle: &str, icon: &str) -> Button {
    let content_box = GtkBox::new(Orientation::Vertical, 8);
    content_box.set_margin_top(16);
    content_box.set_margin_bottom(16);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let icon_widget = gtk::Image::from_icon_name(icon);
    icon_widget.set_pixel_size(48);
    content_box.append(&icon_widget);

    let title_label = Label::new(Some(title));
    title_label.add_css_class("heading");
    content_box.append(&title_label);

    let subtitle_label = Label::new(Some(subtitle));
    subtitle_label.add_css_class("dim-label");
    subtitle_label.add_css_class("caption");
    content_box.append(&subtitle_label);

    let button = Button::builder()
        .child(&content_box)
        .build();

    button.add_css_class("flat");

    button
}
