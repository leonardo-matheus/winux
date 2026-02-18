//! Display settings page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::info;

/// Display settings page
pub struct DisplayPage {
    widget: adw::PreferencesPage,
}

impl DisplayPage {
    /// Create a new display settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Display");
        page.set_icon_name(Some("video-display-symbolic"));

        // Resolution group
        let resolution_group = adw::PreferencesGroup::new();
        resolution_group.set_title("Resolution & Refresh Rate");
        resolution_group.set_description(Some("Configure your display resolution and refresh rate"));

        // Monitor selection (for multi-monitor setups)
        let monitor_row = adw::ComboRow::new();
        monitor_row.set_title("Display");
        monitor_row.set_subtitle("Select display to configure");
        let monitors = gtk4::StringList::new(&["Primary Display", "Secondary Display"]);
        monitor_row.set_model(Some(&monitors));
        resolution_group.add(&monitor_row);

        // Resolution
        let resolution_row = adw::ComboRow::new();
        resolution_row.set_title("Resolution");
        let resolutions = gtk4::StringList::new(&[
            "3840 x 2160 (4K)",
            "2560 x 1440 (QHD)",
            "1920 x 1080 (Full HD)",
            "1680 x 1050",
            "1600 x 900",
            "1366 x 768",
        ]);
        resolution_row.set_model(Some(&resolutions));
        resolution_row.set_selected(2); // Default to 1080p
        resolution_group.add(&resolution_row);

        // Refresh rate
        let refresh_row = adw::ComboRow::new();
        refresh_row.set_title("Refresh Rate");
        let rates = gtk4::StringList::new(&[
            "240 Hz",
            "165 Hz",
            "144 Hz",
            "120 Hz",
            "75 Hz",
            "60 Hz",
        ]);
        refresh_row.set_model(Some(&rates));
        refresh_row.set_selected(5); // Default to 60Hz
        resolution_group.add(&refresh_row);

        // Variable Refresh Rate (VRR/FreeSync/G-Sync)
        let vrr_row = adw::ActionRow::new();
        vrr_row.set_title("Variable Refresh Rate");
        vrr_row.set_subtitle("Enable FreeSync/G-Sync for smoother gaming");
        let vrr_switch = gtk4::Switch::new();
        vrr_switch.set_valign(gtk4::Align::Center);
        vrr_row.add_suffix(&vrr_switch);
        resolution_group.add(&vrr_row);

        page.add(&resolution_group);

        // Display settings group
        let display_group = adw::PreferencesGroup::new();
        display_group.set_title("Display Settings");

        // Scale
        let scale_row = adw::ComboRow::new();
        scale_row.set_title("Scale");
        scale_row.set_subtitle("Adjust the size of text and UI elements");
        let scales = gtk4::StringList::new(&[
            "100%",
            "125%",
            "150%",
            "175%",
            "200%",
        ]);
        scale_row.set_model(Some(&scales));
        scale_row.set_selected(0);
        display_group.add(&scale_row);

        // Orientation
        let orientation_row = adw::ComboRow::new();
        orientation_row.set_title("Orientation");
        let orientations = gtk4::StringList::new(&[
            "Landscape",
            "Portrait",
            "Landscape (flipped)",
            "Portrait (flipped)",
        ]);
        orientation_row.set_model(Some(&orientations));
        display_group.add(&orientation_row);

        page.add(&display_group);

        // HDR group
        let hdr_group = adw::PreferencesGroup::new();
        hdr_group.set_title("HDR");
        hdr_group.set_description(Some("High Dynamic Range settings for supported displays"));

        // HDR toggle
        let hdr_row = adw::ActionRow::new();
        hdr_row.set_title("HDR");
        hdr_row.set_subtitle("Enable High Dynamic Range");
        let hdr_switch = gtk4::Switch::new();
        hdr_switch.set_valign(gtk4::Align::Center);
        hdr_row.add_suffix(&hdr_switch);
        hdr_group.add(&hdr_row);

        // HDR brightness
        let hdr_brightness = adw::ActionRow::new();
        hdr_brightness.set_title("SDR Content Brightness");
        hdr_brightness.set_subtitle("Adjust brightness for non-HDR content");

        let brightness_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.5, 2.0, 0.1);
        brightness_scale.set_value(1.0);
        brightness_scale.set_draw_value(true);
        brightness_scale.set_width_request(200);
        hdr_brightness.add_suffix(&brightness_scale);
        hdr_group.add(&hdr_brightness);

        page.add(&hdr_group);

        // Night Light group
        let night_group = adw::PreferencesGroup::new();
        night_group.set_title("Night Light");
        night_group.set_description(Some("Reduce blue light for better sleep"));

        // Night light toggle
        let night_row = adw::ActionRow::new();
        night_row.set_title("Night Light");
        night_row.set_subtitle("Warm colors to reduce eye strain");
        let night_switch = gtk4::Switch::new();
        night_switch.set_valign(gtk4::Align::Center);
        night_row.add_suffix(&night_switch);
        night_group.add(&night_row);

        // Color temperature
        let temp_row = adw::ActionRow::new();
        temp_row.set_title("Color Temperature");

        let temp_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 3000.0, 6500.0, 100.0);
        temp_scale.set_value(4000.0);
        temp_scale.set_draw_value(true);
        temp_scale.set_width_request(200);
        temp_scale.add_mark(3000.0, gtk4::PositionType::Bottom, Some("Warm"));
        temp_scale.add_mark(6500.0, gtk4::PositionType::Bottom, Some("Cool"));
        temp_row.add_suffix(&temp_scale);
        night_group.add(&temp_row);

        // Schedule
        let schedule_row = adw::ComboRow::new();
        schedule_row.set_title("Schedule");
        let schedules = gtk4::StringList::new(&[
            "Disabled",
            "Sunset to Sunrise",
            "Manual Schedule",
        ]);
        schedule_row.set_model(Some(&schedules));
        night_group.add(&schedule_row);

        page.add(&night_group);

        // Apply button
        let apply_group = adw::PreferencesGroup::new();
        let apply_row = adw::ActionRow::new();

        let apply_btn = gtk4::Button::with_label("Apply Changes");
        apply_btn.add_css_class("suggested-action");
        apply_btn.connect_clicked(|_| {
            info!("Applying display settings");
        });

        let revert_btn = gtk4::Button::with_label("Revert");
        revert_btn.connect_clicked(|_| {
            info!("Reverting display settings");
        });

        let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        btn_box.set_halign(gtk4::Align::End);
        btn_box.set_margin_top(8);
        btn_box.append(&revert_btn);
        btn_box.append(&apply_btn);

        apply_row.set_child(Some(&btn_box));
        apply_group.add(&apply_row);

        page.add(&apply_group);

        DisplayPage { widget: page }
    }

    /// Get the page widget
    pub fn widget(&self) -> &adw::PreferencesPage {
        &self.widget
    }
}

impl Default for DisplayPage {
    fn default() -> Self {
        Self::new()
    }
}
