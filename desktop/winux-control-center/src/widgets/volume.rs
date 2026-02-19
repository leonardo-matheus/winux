//! Volume control widget
//!
//! Provides a slider for controlling audio volume with output device selection.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Adjustment, Box, Button, Image, Label, ListBox, ListBoxRow, Orientation, Popover, Scale, Separator};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

/// Audio output device information
#[derive(Clone, Debug)]
pub struct AudioDevice {
    pub name: String,
    pub device_id: String,
    pub is_default: bool,
    pub device_type: AudioDeviceType,
}

/// Type of audio device
#[derive(Clone, Debug)]
pub enum AudioDeviceType {
    Speaker,
    Headphones,
    Bluetooth,
    HDMI,
    USB,
    Other,
}

impl AudioDeviceType {
    fn icon_name(&self) -> &str {
        match self {
            AudioDeviceType::Speaker => "audio-speakers-symbolic",
            AudioDeviceType::Headphones => "audio-headphones-symbolic",
            AudioDeviceType::Bluetooth => "bluetooth-symbolic",
            AudioDeviceType::HDMI => "video-display-symbolic",
            AudioDeviceType::USB => "audio-card-symbolic",
            AudioDeviceType::Other => "audio-card-symbolic",
        }
    }
}

/// Volume control widget with slider and output device selection
pub struct VolumeWidget {
    container: Box,
    slider: Scale,
    icon: Image,
    level: Rc<Cell<u32>>,
    muted: Rc<Cell<bool>>,
}

impl VolumeWidget {
    /// Create a new volume widget
    pub fn new(initial_level: u32, initial_muted: bool) -> Self {
        let container = Box::new(Orientation::Vertical, 8);
        container.add_css_class("slider-tile");

        // Header with icon, label, and output selector
        let header = Box::new(Orientation::Horizontal, 8);

        let icon = Image::from_icon_name(Self::volume_icon(initial_level, initial_muted));
        icon.add_css_class("slider-icon");
        icon.set_pixel_size(20);

        // Make icon clickable for mute toggle
        let icon_btn = Button::builder()
            .child(&icon)
            .build();
        icon_btn.add_css_class("flat");
        header.append(&icon_btn);

        let label = Label::new(Some("Volume"));
        label.add_css_class("slider-label");
        label.set_hexpand(true);
        label.set_halign(gtk::Align::Start);
        header.append(&label);

        // Output device selector
        let output_btn = Button::builder()
            .icon_name("audio-speakers-symbolic")
            .tooltip_text("Select Output Device")
            .build();
        output_btn.add_css_class("flat");
        output_btn.add_css_class("circular");

        let output_popover = Self::create_output_popover();
        output_popover.set_parent(&output_btn);

        let popover_clone = output_popover.clone();
        output_btn.connect_clicked(move |_| {
            popover_clone.popup();
        });

        header.append(&output_btn);

        // Percentage label
        let percentage = Label::new(Some(&format!("{}%", initial_level)));
        percentage.add_css_class("slider-label");
        header.append(&percentage);

        container.append(&header);

        // Slider
        let adjustment = Adjustment::new(
            initial_level as f64,
            0.0,
            100.0,
            1.0,
            10.0,
            0.0,
        );

        let slider = Scale::new(Orientation::Horizontal, Some(&adjustment));
        slider.add_css_class("control-slider");
        slider.set_draw_value(false);
        slider.set_hexpand(true);

        container.append(&slider);

        let level = Rc::new(Cell::new(initial_level));
        let muted = Rc::new(Cell::new(initial_muted));

        // Connect mute toggle
        let muted_clone = muted.clone();
        let icon_clone = icon.clone();
        let level_clone = level.clone();
        let slider_clone = slider.clone();

        icon_btn.connect_clicked(move |_| {
            let new_muted = !muted_clone.get();
            muted_clone.set(new_muted);
            icon_clone.set_icon_name(Some(Self::volume_icon(level_clone.get(), new_muted)));
            slider_clone.set_sensitive(!new_muted);

            Self::set_system_mute(new_muted);
            info!("Volume muted: {}", new_muted);
        });

        // Connect value change handler
        let level_clone2 = level.clone();
        let muted_clone2 = muted.clone();
        let icon_clone2 = icon.clone();
        let percentage_clone = percentage.clone();

        slider.connect_value_changed(move |scale| {
            let value = scale.value() as u32;
            level_clone2.set(value);
            percentage_clone.set_text(&format!("{}%", value));

            // Update icon based on level
            if !muted_clone2.get() {
                icon_clone2.set_icon_name(Some(Self::volume_icon(value, false)));
            }

            Self::set_system_volume(value);
        });

        Self {
            container,
            slider,
            icon,
            level,
            muted,
        }
    }

    /// Create popover for output device selection
    fn create_output_popover() -> Popover {
        let popover = Popover::new();
        popover.add_css_class("network-list");

        let content = Box::new(Orientation::Vertical, 4);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(8);
        content.set_margin_end(8);

        // Header
        let title = Label::new(Some("Output Device"));
        title.add_css_class("title-4");
        title.set_halign(gtk::Align::Start);
        content.append(&title);

        content.append(&Separator::new(Orientation::Horizontal));

        // Device list
        let list = ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .build();
        list.add_css_class("boxed-list");

        let devices = Self::get_mock_devices();
        for device in &devices {
            let row = Self::create_device_row(device);
            list.append(&row);
            if device.is_default {
                list.select_row(Some(&row));
            }
        }

        // Handle selection
        list.connect_row_selected(|_, row| {
            if let Some(row) = row {
                let index = row.index();
                info!("Selected audio output device at index {}", index);
                // Here we would actually switch the audio output
            }
        });

        content.append(&list);

        // Sound settings button
        let settings_btn = Button::builder()
            .label("Sound Settings...")
            .build();
        settings_btn.add_css_class("flat");
        settings_btn.connect_clicked(|_| {
            info!("Opening sound settings");
            let _ = std::process::Command::new("gnome-control-center")
                .arg("sound")
                .spawn();
        });
        content.append(&settings_btn);

        popover.set_child(Some(&content));
        popover
    }

    /// Create a row for an audio device
    fn create_device_row(device: &AudioDevice) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("network-item");

        let content = Box::new(Orientation::Horizontal, 12);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(8);
        content.set_margin_end(8);

        // Device icon
        let icon = Image::from_icon_name(device.device_type.icon_name());
        icon.set_pixel_size(20);
        content.append(&icon);

        // Device name
        let name = Label::new(Some(&device.name));
        name.add_css_class("network-name");
        name.set_hexpand(true);
        name.set_halign(gtk::Align::Start);
        content.append(&name);

        // Checkmark for default device
        if device.is_default {
            let check = Image::from_icon_name("emblem-ok-symbolic");
            check.add_css_class("accent");
            content.append(&check);
        }

        row.set_child(Some(&content));
        row
    }

    /// Get mock audio devices
    fn get_mock_devices() -> Vec<AudioDevice> {
        vec![
            AudioDevice {
                name: "Built-in Speakers".to_string(),
                device_id: "speakers".to_string(),
                is_default: true,
                device_type: AudioDeviceType::Speaker,
            },
            AudioDevice {
                name: "AirPods Pro".to_string(),
                device_id: "airpods".to_string(),
                is_default: false,
                device_type: AudioDeviceType::Bluetooth,
            },
            AudioDevice {
                name: "HDMI Output".to_string(),
                device_id: "hdmi".to_string(),
                is_default: false,
                device_type: AudioDeviceType::HDMI,
            },
        ]
    }

    /// Get the icon name based on volume level and mute state
    fn volume_icon(level: u32, muted: bool) -> &'static str {
        if muted {
            "audio-volume-muted-symbolic"
        } else {
            match level {
                0 => "audio-volume-muted-symbolic",
                1..=33 => "audio-volume-low-symbolic",
                34..=66 => "audio-volume-medium-symbolic",
                _ => "audio-volume-high-symbolic",
            }
        }
    }

    /// Set the system volume (platform-specific implementation)
    fn set_system_volume(level: u32) {
        info!("Setting volume to {}%", level);

        // Try using pactl on Linux (PulseAudio/PipeWire)
        let _ = std::process::Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", level)])
            .spawn();

        // Alternative: wpctl for WirePlumber
        // let _ = std::process::Command::new("wpctl")
        //     .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", level)])
        //     .spawn();
    }

    /// Set the system mute state
    fn set_system_mute(muted: bool) {
        let mute_str = if muted { "1" } else { "0" };

        // Try using pactl
        let _ = std::process::Command::new("pactl")
            .args(["set-sink-mute", "@DEFAULT_SINK@", mute_str])
            .spawn();
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Get the current volume level
    pub fn level(&self) -> u32 {
        self.level.get()
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.muted.get()
    }

    /// Set the volume level programmatically
    pub fn set_level(&self, level: u32) {
        let clamped = level.min(100);
        self.level.set(clamped);
        self.slider.set_value(clamped as f64);
    }

    /// Set mute state programmatically
    pub fn set_muted(&self, muted: bool) {
        self.muted.set(muted);
        self.icon.set_icon_name(Some(Self::volume_icon(self.level.get(), muted)));
        self.slider.set_sensitive(!muted);
    }
}
