//! Source picker widget
//!
//! Allows users to select the screen capture source:
//! - Entire screen
//! - Specific window
//! - Custom region
//! - Multiple monitors

use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::capture::SourceType;
use crate::window::{get_monitors, MonitorInfo};

/// Source picker widget
pub struct SourcePicker {
    widget: gtk::Box,
    state: Rc<RefCell<AppState>>,
}

impl SourcePicker {
    /// Create a new source picker
    pub fn new(state: &Rc<RefCell<AppState>>) -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .build();

        let picker = Self {
            widget,
            state: state.clone(),
        };

        picker.build_ui();
        picker
    }

    /// Get the root widget
    pub fn widget(&self) -> gtk::Box {
        self.widget.clone()
    }

    fn build_ui(&self) {
        // Create toggle buttons for source types
        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .halign(gtk::Align::Center)
            .css_classes(["linked"])
            .build();

        // Screen button
        let screen_btn = self.create_source_button(
            "video-display-symbolic",
            "Screen",
            SourceType::Screen,
        );

        // Window button
        let window_btn = self.create_source_button(
            "window-symbolic",
            "Window",
            SourceType::Window,
        );

        // Region button
        let region_btn = self.create_source_button(
            "edit-select-all-symbolic",
            "Region",
            SourceType::Region,
        );

        // Link toggle buttons
        screen_btn.set_group(Some(&screen_btn));
        window_btn.set_group(Some(&screen_btn));
        region_btn.set_group(Some(&screen_btn));

        // Set initial selection
        match self.state.borrow().source_type {
            SourceType::Screen | SourceType::VirtualScreen => screen_btn.set_active(true),
            SourceType::Window => window_btn.set_active(true),
            SourceType::Region => region_btn.set_active(true),
        }

        button_box.append(&screen_btn);
        button_box.append(&window_btn);
        button_box.append(&region_btn);

        self.widget.append(&button_box);

        // Monitor selection (shown when Screen is selected)
        let monitor_section = self.create_monitor_section();
        self.widget.append(&monitor_section);

        // Update monitor section visibility based on source type
        let state_clone = self.state.clone();
        let monitor_section_clone = monitor_section.clone();

        screen_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                state_clone.borrow_mut().source_type = SourceType::Screen;
                monitor_section_clone.set_visible(true);
            }
        });

        let state_clone = self.state.clone();
        let monitor_section_clone = monitor_section.clone();

        window_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                state_clone.borrow_mut().source_type = SourceType::Window;
                monitor_section_clone.set_visible(false);
            }
        });

        let state_clone = self.state.clone();
        let monitor_section_clone = monitor_section.clone();

        region_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                state_clone.borrow_mut().source_type = SourceType::Region;
                monitor_section_clone.set_visible(false);
            }
        });
    }

    fn create_source_button(
        &self,
        icon: &str,
        label: &str,
        source_type: SourceType,
    ) -> gtk::ToggleButton {
        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(16)
            .margin_end(16)
            .build();

        let icon_widget = gtk::Image::builder()
            .icon_name(icon)
            .pixel_size(32)
            .build();

        let label_widget = gtk::Label::builder()
            .label(label)
            .build();

        content.append(&icon_widget);
        content.append(&label_widget);

        let button = gtk::ToggleButton::builder()
            .child(&content)
            .css_classes(["flat"])
            .build();

        button
    }

    fn create_monitor_section(&self) -> gtk::Box {
        let section = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(12)
            .build();

        let monitors = get_monitors();

        if monitors.len() <= 1 {
            // Single monitor - just show info
            if let Some(monitor) = monitors.first() {
                let info_label = gtk::Label::builder()
                    .label(&format!(
                        "{} - {}",
                        monitor.description,
                        monitor.resolution_string()
                    ))
                    .css_classes(["dim-label"])
                    .build();
                section.append(&info_label);
            }
        } else {
            // Multiple monitors - show selection
            let label = gtk::Label::builder()
                .label("Select Monitor")
                .halign(gtk::Align::Start)
                .css_classes(["heading"])
                .build();
            section.append(&label);

            let list_box = gtk::ListBox::builder()
                .css_classes(["boxed-list"])
                .selection_mode(gtk::SelectionMode::Single)
                .build();

            // Add "All Monitors" option
            let all_row = self.create_monitor_row(None);
            list_box.append(&all_row);

            // Add individual monitors
            for monitor in &monitors {
                let row = self.create_monitor_row(Some(monitor));
                list_box.append(&row);
            }

            // Handle selection
            let state = self.state.clone();
            list_box.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    let index = row.index();
                    if index == 0 {
                        // All monitors
                        state.borrow_mut().source_type = SourceType::VirtualScreen;
                        state.borrow_mut().selected_monitor = None;
                    } else {
                        state.borrow_mut().source_type = SourceType::Screen;
                        state.borrow_mut().selected_monitor = Some((index - 1) as u32);
                    }
                }
            });

            // Select first row by default
            if let Some(first_row) = list_box.row_at_index(0) {
                list_box.select_row(Some(&first_row));
            }

            section.append(&list_box);
        }

        section
    }

    fn create_monitor_row(&self, monitor: Option<&MonitorInfo>) -> adw::ActionRow {
        match monitor {
            Some(m) => {
                adw::ActionRow::builder()
                    .title(&m.description)
                    .subtitle(&format!(
                        "{} @ {:.0} Hz",
                        m.resolution_string(),
                        m.refresh_hz()
                    ))
                    .icon_name("video-display-symbolic")
                    .activatable(true)
                    .build()
            }
            None => {
                adw::ActionRow::builder()
                    .title("All Monitors")
                    .subtitle("Record entire virtual screen")
                    .icon_name("view-dual-symbolic")
                    .activatable(true)
                    .build()
            }
        }
    }
}

/// Show region selection overlay
pub fn show_region_selection(
    window: &impl IsA<gtk::Window>,
    callback: impl Fn(Option<(i32, i32, u32, u32)>) + 'static,
) {
    // This would show a fullscreen overlay for region selection
    // For now, we'll use the portal's built-in region selection

    // In a full implementation, this would:
    // 1. Create a transparent fullscreen window
    // 2. Allow user to draw a rectangle
    // 3. Return the selected region coordinates

    // For Wayland, the portal handles region selection,
    // so this is mainly for X11 fallback

    callback(None); // Use portal selection
}

/// Preview of selected source
pub struct SourcePreview {
    widget: gtk::Box,
}

impl SourcePreview {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .css_classes(["card"])
            .build();

        // Placeholder for preview
        let preview_area = gtk::DrawingArea::builder()
            .width_request(320)
            .height_request(180)
            .build();

        // Draw placeholder
        preview_area.set_draw_func(|_, cr, width, height| {
            // Background
            cr.set_source_rgb(0.2, 0.2, 0.2);
            cr.paint().ok();

            // Border
            cr.set_source_rgb(0.4, 0.4, 0.4);
            cr.set_line_width(2.0);
            cr.rectangle(0.0, 0.0, width as f64, height as f64);
            cr.stroke().ok();

            // Center icon placeholder
            cr.set_source_rgb(0.5, 0.5, 0.5);
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            cr.arc(center_x, center_y, 30.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().ok();
        });

        let label = gtk::Label::builder()
            .label("Preview will appear when recording")
            .css_classes(["dim-label"])
            .margin_bottom(8)
            .build();

        widget.append(&preview_area);
        widget.append(&label);

        Self { widget }
    }

    pub fn widget(&self) -> gtk::Box {
        self.widget.clone()
    }
}
