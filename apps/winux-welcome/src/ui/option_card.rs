// Winux Welcome - Option Card
// Reusable card component for selecting options

use gtk4::prelude::*;
use gtk4::{Box, CheckButton, Frame, Label, Orientation};

/// Configuration for creating an option card
pub struct OptionCardConfig {
    pub title: String,
    pub description: String,
    pub icon_name: Option<String>,
    pub is_selected: bool,
}

/// Creates a selectable option card with preview area
pub fn create_option_card(config: OptionCardConfig, radio_group: Option<&CheckButton>) -> (Box, CheckButton) {
    let card = Box::new(Orientation::Vertical, 12);
    card.add_css_class("card");
    card.add_css_class("option-card");
    card.set_size_request(200, 250);

    let inner = Box::new(Orientation::Vertical, 12);
    inner.set_margin_top(16);
    inner.set_margin_bottom(16);
    inner.set_margin_start(16);
    inner.set_margin_end(16);

    // Icon or preview area
    let preview = Frame::new(None);
    preview.set_size_request(168, 100);
    preview.add_css_class("view");

    if let Some(icon_name) = &config.icon_name {
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(64);
        icon.set_halign(gtk4::Align::Center);
        icon.set_valign(gtk4::Align::Center);
        preview.set_child(Some(&icon));
    }

    inner.append(&preview);

    // Title
    let title_label = Label::new(Some(&config.title));
    title_label.add_css_class("title-4");
    inner.append(&title_label);

    // Description
    let desc_label = Label::new(Some(&config.description));
    desc_label.add_css_class("dim-label");
    desc_label.add_css_class("caption");
    desc_label.set_wrap(true);
    desc_label.set_justify(gtk4::Justification::Center);
    inner.append(&desc_label);

    // Radio button
    let radio = if let Some(group) = radio_group {
        let r = CheckButton::new();
        r.set_group(Some(group));
        r
    } else {
        CheckButton::new()
    };
    radio.set_active(config.is_selected);

    let radio_box = Box::new(Orientation::Horizontal, 8);
    radio_box.set_halign(gtk4::Align::Center);
    radio_box.append(&radio);

    let select_label = Label::new(Some("Selecionar"));
    radio_box.append(&select_label);

    inner.append(&radio_box);

    card.append(&inner);

    // Make entire card clickable
    let gesture = gtk4::GestureClick::new();
    let radio_clone = radio.clone();
    gesture.connect_released(move |_, _, _, _| {
        radio_clone.set_active(true);
    });
    card.add_controller(gesture);

    (card, radio)
}

/// Creates a simple toggle card (checkbox instead of radio)
pub fn create_toggle_card(config: OptionCardConfig) -> (Box, CheckButton) {
    let card = Box::new(Orientation::Vertical, 8);
    card.add_css_class("card");
    card.add_css_class("toggle-card");
    card.set_size_request(150, 100);

    let inner = Box::new(Orientation::Vertical, 8);
    inner.set_margin_top(12);
    inner.set_margin_bottom(12);
    inner.set_margin_start(12);
    inner.set_margin_end(12);

    // Icon
    if let Some(icon_name) = &config.icon_name {
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(32);
        inner.append(&icon);
    }

    // Title
    let title_label = Label::new(Some(&config.title));
    title_label.add_css_class("heading");
    inner.append(&title_label);

    // Checkbox
    let check = CheckButton::new();
    check.set_active(config.is_selected);
    check.set_halign(gtk4::Align::Center);
    inner.append(&check);

    card.append(&inner);

    // Make card clickable
    let gesture = gtk4::GestureClick::new();
    let check_clone = check.clone();
    gesture.connect_released(move |_, _, _, _| {
        check_clone.set_active(!check_clone.is_active());
    });
    card.add_controller(gesture);

    (card, check)
}
