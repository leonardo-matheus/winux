// Font card widget - displays font in a card format for grid view

use gtk4::prelude::*;
use gtk4::{Box, Frame, Label, Orientation, Button, GestureClick};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::FontInfo;

/// Font card widget for grid display
#[derive(Clone)]
pub struct FontCard {
    container: Frame,
    font_info: Rc<RefCell<FontInfo>>,
}

impl FontCard {
    pub fn new(font: &FontInfo) -> Self {
        let font_info = Rc::new(RefCell::new(font.clone()));

        let container = Frame::new(None);
        container.add_css_class("card");
        container.set_margin_start(4);
        container.set_margin_end(4);
        container.set_margin_top(4);
        container.set_margin_bottom(4);

        let content = Box::new(Orientation::Vertical, 8);
        content.set_margin_top(16);
        content.set_margin_bottom(16);
        content.set_margin_start(16);
        content.set_margin_end(16);

        // Font preview (large text)
        let preview_label = Label::builder()
            .label("Aa")
            .halign(gtk4::Align::Center)
            .build();

        // Apply font to preview
        let attr_list = pango::AttrList::new();
        let font_desc = pango::FontDescription::from_string(&format!("{} 48", font.family));
        let font_attr = pango::AttrFontDesc::new(&font_desc);
        attr_list.insert(font_attr);
        preview_label.set_attributes(Some(&attr_list));

        content.append(&preview_label);

        // Font name
        let name_label = Label::builder()
            .label(&font.family)
            .css_classes(vec!["title-4"])
            .halign(gtk4::Align::Center)
            .ellipsize(pango::EllipsizeMode::End)
            .max_width_chars(20)
            .build();
        content.append(&name_label);

        // Font style
        let style_label = Label::builder()
            .label(&font.style)
            .css_classes(vec!["dim-label"])
            .halign(gtk4::Align::Center)
            .build();
        content.append(&style_label);

        // Category badge
        let category_box = Box::new(Orientation::Horizontal, 4);
        category_box.set_halign(gtk4::Align::Center);
        category_box.set_margin_top(8);

        let category_label = Label::builder()
            .label(&font.category.to_string())
            .css_classes(vec!["caption", "dim-label"])
            .build();

        let badge = Box::new(Orientation::Horizontal, 4);
        badge.add_css_class("badge");
        badge.append(&category_label);
        category_box.append(&badge);

        content.append(&category_box);

        // Click gesture
        let gesture = GestureClick::new();
        {
            let fi = font_info.clone();
            gesture.connect_released(move |_, _, _, _| {
                // In a full app, this would open font details
                let font = fi.borrow();
                println!("Clicked on font: {} {}", font.family, font.style);
            });
        }
        container.add_controller(gesture);

        container.set_child(Some(&content));

        Self {
            container,
            font_info,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &Frame {
        &self.container
    }

    /// Get font info
    pub fn font_info(&self) -> FontInfo {
        self.font_info.borrow().clone()
    }

    /// Update the card with new font info
    pub fn set_font_info(&self, font: &FontInfo) {
        *self.font_info.borrow_mut() = font.clone();
        // In a full implementation, update the labels
    }

    /// Set selected state
    pub fn set_selected(&self, selected: bool) {
        if selected {
            self.container.add_css_class("selected");
        } else {
            self.container.remove_css_class("selected");
        }
    }
}

/// Create a compact font row for list view
pub fn create_font_row(font: &FontInfo) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(&font.family)
        .subtitle(&format!("{} - {}", font.style, font.category))
        .activatable(true)
        .build();

    // Font preview
    let preview = Label::new(Some("Aa"));
    let attr_list = pango::AttrList::new();
    let font_desc = pango::FontDescription::from_string(&format!("{} 16", font.family));
    let font_attr = pango::AttrFontDesc::new(&font_desc);
    attr_list.insert(font_attr);
    preview.set_attributes(Some(&attr_list));
    preview.set_width_request(48);
    row.add_prefix(&preview);

    // Category icon
    let icon = match font.category {
        crate::fonts::FontCategory::Monospace => "utilities-terminal-symbolic",
        crate::fonts::FontCategory::Serif => "format-text-italic-symbolic",
        crate::fonts::FontCategory::SansSerif => "format-text-bold-symbolic",
        crate::fonts::FontCategory::Display => "format-text-larger-symbolic",
        crate::fonts::FontCategory::Handwriting => "format-text-underline-symbolic",
        _ => "font-x-generic-symbolic",
    };
    let icon_widget = gtk4::Image::from_icon_name(icon);
    row.add_suffix(&icon_widget);

    // Arrow
    row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

    row
}
