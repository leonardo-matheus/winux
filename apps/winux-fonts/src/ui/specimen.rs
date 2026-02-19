// Specimen view widget - displays font specimen with customizable text

use gtk4::prelude::*;
use gtk4::{Box, Frame, Label, Orientation, TextView, TextBuffer};
use gdk4::RGBA;
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::FontInfo;

/// Specimen view widget for font preview
#[derive(Clone)]
pub struct SpecimenView {
    container: Frame,
    label: Label,
    font_info: Rc<RefCell<Option<FontInfo>>>,
    text: Rc<RefCell<String>>,
    size: Rc<RefCell<i32>>,
    text_color: Rc<RefCell<RGBA>>,
    bg_color: Rc<RefCell<RGBA>>,
}

impl SpecimenView {
    pub fn new() -> Self {
        let font_info = Rc::new(RefCell::new(None));
        let text = Rc::new(RefCell::new(String::from("The quick brown fox jumps over the lazy dog")));
        let size = Rc::new(RefCell::new(24));
        let text_color = Rc::new(RefCell::new(RGBA::new(1.0, 1.0, 1.0, 1.0)));
        let bg_color = Rc::new(RefCell::new(RGBA::new(0.15, 0.15, 0.15, 1.0)));

        let container = Frame::new(None);
        container.add_css_class("card");
        container.set_margin_top(8);
        container.set_margin_bottom(8);

        let inner_box = Box::new(Orientation::Vertical, 0);
        inner_box.set_margin_top(24);
        inner_box.set_margin_bottom(24);
        inner_box.set_margin_start(24);
        inner_box.set_margin_end(24);

        let label = Label::builder()
            .label(&text.borrow())
            .wrap(true)
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Start)
            .build();

        // Apply initial styling
        Self::apply_styling(&label, *size.borrow(), &text_color.borrow(), &font_info.borrow());

        inner_box.append(&label);
        container.set_child(Some(&inner_box));

        // Apply background color via CSS
        let css_provider = gtk4::CssProvider::new();
        let bg = bg_color.borrow();
        let css = format!(
            ".specimen-bg {{ background-color: rgba({}, {}, {}, {}); }}",
            (bg.red() * 255.0) as u8,
            (bg.green() * 255.0) as u8,
            (bg.blue() * 255.0) as u8,
            bg.alpha()
        );
        css_provider.load_from_string(&css);
        inner_box.add_css_class("specimen-bg");

        if let Some(display) = gdk4::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        Self {
            container,
            label,
            font_info,
            text,
            size,
            text_color,
            bg_color,
        }
    }

    /// Apply font styling to label
    fn apply_styling(
        label: &Label,
        size: i32,
        color: &RGBA,
        font: &Option<FontInfo>,
    ) {
        let attr_list = pango::AttrList::new();

        // Size
        let size_attr = pango::AttrInt::new_size(size * pango::SCALE);
        attr_list.insert(size_attr);

        // Font family
        if let Some(fi) = font {
            let font_desc = pango::FontDescription::from_string(&fi.to_pango_string());
            let font_attr = pango::AttrFontDesc::new(&font_desc);
            attr_list.insert(font_attr);
        }

        // Color
        let color_attr = pango::AttrColor::new_foreground(
            (color.red() * 65535.0) as u16,
            (color.green() * 65535.0) as u16,
            (color.blue() * 65535.0) as u16,
        );
        attr_list.insert(color_attr);

        label.set_attributes(Some(&attr_list));
    }

    /// Get the widget
    pub fn widget(&self) -> &Frame {
        &self.container
    }

    /// Set the sample text
    pub fn set_text(&self, text: &str) {
        *self.text.borrow_mut() = text.to_string();
        self.label.set_label(text);
        self.update_styling();
    }

    /// Set the font
    pub fn set_font(&self, font: &FontInfo) {
        *self.font_info.borrow_mut() = Some(font.clone());
        self.update_styling();
    }

    /// Set the font size
    pub fn set_size(&self, size: i32) {
        *self.size.borrow_mut() = size;
        self.update_styling();
    }

    /// Set the text color
    pub fn set_text_color(&self, color: RGBA) {
        *self.text_color.borrow_mut() = color;
        self.update_styling();
    }

    /// Set the background color
    pub fn set_bg_color(&self, color: RGBA) {
        *self.bg_color.borrow_mut() = color;
        self.update_background();
    }

    /// Update styling
    fn update_styling(&self) {
        Self::apply_styling(
            &self.label,
            *self.size.borrow(),
            &self.text_color.borrow(),
            &self.font_info.borrow(),
        );
    }

    /// Update background color
    fn update_background(&self) {
        let css_provider = gtk4::CssProvider::new();
        let bg = self.bg_color.borrow();
        let css = format!(
            ".specimen-bg {{ background-color: rgba({}, {}, {}, {}); border-radius: 12px; }}",
            (bg.red() * 255.0) as u8,
            (bg.green() * 255.0) as u8,
            (bg.blue() * 255.0) as u8,
            bg.alpha()
        );
        css_provider.load_from_string(&css);

        if let Some(display) = gdk4::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    /// Get current text
    pub fn text(&self) -> String {
        self.text.borrow().clone()
    }

    /// Get current font
    pub fn font(&self) -> Option<FontInfo> {
        self.font_info.borrow().clone()
    }
}

impl Default for SpecimenView {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-size specimen view
pub struct MultiSizeSpecimen {
    container: Box,
    font_info: Rc<RefCell<Option<FontInfo>>>,
    text: Rc<RefCell<String>>,
}

impl MultiSizeSpecimen {
    pub fn new() -> Self {
        let font_info = Rc::new(RefCell::new(None));
        let text = Rc::new(RefCell::new(String::from("The quick brown fox")));

        let container = Box::new(Orientation::Vertical, 12);

        // Create multiple size previews
        let sizes = [12, 16, 20, 24, 32, 48, 64, 72];

        for size in sizes {
            let row = Box::new(Orientation::Horizontal, 12);

            // Size label
            let size_label = Label::builder()
                .label(&format!("{}pt", size))
                .width_request(48)
                .halign(gtk4::Align::End)
                .css_classes(vec!["dim-label", "caption"])
                .build();
            row.append(&size_label);

            // Sample text
            let sample = Label::builder()
                .label(&text.borrow())
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .ellipsize(pango::EllipsizeMode::End)
                .build();

            let attr_list = pango::AttrList::new();
            let size_attr = pango::AttrInt::new_size(size * pango::SCALE);
            attr_list.insert(size_attr);
            sample.set_attributes(Some(&attr_list));

            row.append(&sample);
            container.append(&row);
        }

        Self {
            container,
            font_info,
            text,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the font
    pub fn set_font(&self, font: &FontInfo) {
        *self.font_info.borrow_mut() = Some(font.clone());
        self.update();
    }

    /// Set the sample text
    pub fn set_text(&self, text: &str) {
        *self.text.borrow_mut() = text.to_string();
        self.update();
    }

    /// Update all samples
    fn update(&self) {
        let sizes = [12, 16, 20, 24, 32, 48, 64, 72];
        let mut child = self.container.first_child();
        let mut idx = 0;

        while let Some(row) = child {
            if let Some(row_box) = row.downcast_ref::<Box>() {
                // Get the sample label (second child)
                if let Some(first) = row_box.first_child() {
                    if let Some(sample) = first.next_sibling() {
                        if let Some(label) = sample.downcast_ref::<Label>() {
                            label.set_label(&self.text.borrow());

                            let attr_list = pango::AttrList::new();
                            let size_attr = pango::AttrInt::new_size(sizes[idx] * pango::SCALE);
                            attr_list.insert(size_attr);

                            if let Some(font) = self.font_info.borrow().as_ref() {
                                let font_desc = pango::FontDescription::from_string(&font.family);
                                let font_attr = pango::AttrFontDesc::new(&font_desc);
                                attr_list.insert(font_attr);
                            }

                            label.set_attributes(Some(&attr_list));
                        }
                    }
                }
            }

            child = row.next_sibling();
            idx += 1;
        }
    }
}

impl Default for MultiSizeSpecimen {
    fn default() -> Self {
        Self::new()
    }
}
