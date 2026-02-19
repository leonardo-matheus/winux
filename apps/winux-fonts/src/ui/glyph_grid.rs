// Glyph grid widget - displays all glyphs/characters in a font

use gtk4::prelude::*;
use gtk4::{
    Box, FlowBox, Frame, Label, Orientation, ScrolledWindow,
    SelectionMode, DropDown, StringList,
};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::{FontInfo, font_info::CharacterRange};

/// Glyph grid widget
#[derive(Clone)]
pub struct GlyphGrid {
    container: Box,
    flowbox: FlowBox,
    font_info: Rc<RefCell<Option<FontInfo>>>,
    current_range: Rc<RefCell<CharacterRange>>,
    font_size: Rc<RefCell<i32>>,
}

impl GlyphGrid {
    pub fn new() -> Self {
        let font_info = Rc::new(RefCell::new(None));
        let font_size = Rc::new(RefCell::new(24));
        let current_range = Rc::new(RefCell::new(CharacterRange {
            name: "Basic Latin".into(),
            start: 0x0020,
            end: 0x007F,
        }));

        let container = Box::new(Orientation::Vertical, 12);

        // Toolbar
        let toolbar = Box::new(Orientation::Horizontal, 12);
        toolbar.set_margin_top(8);
        toolbar.set_margin_bottom(8);

        // Range selector
        let range_label = Label::new(Some("Bloco Unicode:"));
        toolbar.append(&range_label);

        let ranges = CharacterRange::common_ranges();
        let range_names: Vec<&str> = ranges.iter().map(|r| r.name.as_str()).collect();
        let range_list = StringList::new(&range_names);
        let range_dropdown = DropDown::builder()
            .model(&range_list)
            .build();

        {
            let cr = current_range.clone();
            let fi = font_info.clone();
            let fb_clone = Rc::new(RefCell::new(None::<FlowBox>));

            range_dropdown.connect_selected_notify(move |dropdown| {
                let idx = dropdown.selected() as usize;
                let ranges = CharacterRange::common_ranges();
                if idx < ranges.len() {
                    *cr.borrow_mut() = ranges[idx].clone();
                    // Would update grid here
                }
            });
        }

        toolbar.append(&range_dropdown);

        // Spacer
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&spacer);

        // Size selector
        let size_label = Label::new(Some("Tamanho:"));
        toolbar.append(&size_label);

        let sizes = StringList::new(&["16", "24", "32", "48", "64"]);
        let size_dropdown = DropDown::builder()
            .model(&sizes)
            .selected(1) // 24
            .build();

        {
            let fs = font_size.clone();
            size_dropdown.connect_selected_notify(move |dropdown| {
                let sizes = [16, 24, 32, 48, 64];
                let idx = dropdown.selected() as usize;
                if idx < sizes.len() {
                    *fs.borrow_mut() = sizes[idx];
                    // Would update grid here
                }
            });
        }

        toolbar.append(&size_dropdown);

        container.append(&toolbar);

        // Glyph flowbox
        let flowbox = FlowBox::builder()
            .valign(gtk4::Align::Start)
            .max_children_per_line(16)
            .min_children_per_line(8)
            .selection_mode(SelectionMode::Single)
            .homogeneous(true)
            .row_spacing(4)
            .column_spacing(4)
            .build();

        // Populate with initial glyphs
        Self::populate_grid(&flowbox, &current_range.borrow(), *font_size.borrow(), &font_info.borrow());

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&flowbox)
            .build();

        container.append(&scrolled);

        // Selected glyph info
        let info_box = Box::new(Orientation::Horizontal, 12);
        info_box.set_margin_top(8);
        info_box.add_css_class("toolbar");

        let glyph_preview = Label::builder()
            .label("A")
            .width_request(64)
            .css_classes(vec!["title-1"])
            .build();
        info_box.append(&glyph_preview);

        let glyph_info = Box::new(Orientation::Vertical, 4);

        let unicode_label = Label::builder()
            .label("Unicode: U+0041")
            .halign(gtk4::Align::Start)
            .build();
        glyph_info.append(&unicode_label);

        let name_label = Label::builder()
            .label("LATIN CAPITAL LETTER A")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["dim-label"])
            .build();
        glyph_info.append(&name_label);

        info_box.append(&glyph_info);
        container.append(&info_box);

        // Connect selection
        {
            let preview = glyph_preview.clone();
            let unicode = unicode_label.clone();
            let name = name_label.clone();

            flowbox.connect_selected_children_changed(move |fb| {
                if let Some(child) = fb.selected_children().first() {
                    if let Some(frame) = child.child() {
                        if let Some(label) = frame.first_child() {
                            if let Some(lbl) = label.downcast_ref::<Label>() {
                                let text = lbl.text();
                                if let Some(ch) = text.chars().next() {
                                    preview.set_label(&text);
                                    unicode.set_label(&format!("Unicode: U+{:04X}", ch as u32));
                                    name.set_label(&get_unicode_name(ch));
                                }
                            }
                        }
                    }
                }
            });
        }

        Self {
            container,
            flowbox,
            font_info,
            current_range,
            font_size,
        }
    }

    /// Populate the grid with glyphs
    fn populate_grid(
        flowbox: &FlowBox,
        range: &CharacterRange,
        size: i32,
        font: &Option<FontInfo>,
    ) {
        // Clear existing
        while let Some(child) = flowbox.first_child() {
            flowbox.remove(&child);
        }

        // Add glyphs
        for code in range.start..=range.end {
            if let Some(ch) = char::from_u32(code) {
                // Skip control characters
                if ch.is_control() {
                    continue;
                }

                let glyph = Self::create_glyph_cell(ch, size, font);
                flowbox.append(&glyph);
            }
        }
    }

    /// Create a single glyph cell
    fn create_glyph_cell(ch: char, size: i32, font: &Option<FontInfo>) -> Frame {
        let frame = Frame::new(None);
        frame.set_size_request(size + 16, size + 16);

        let label = Label::builder()
            .label(&ch.to_string())
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        // Apply font if specified
        let attr_list = pango::AttrList::new();
        let size_attr = pango::AttrInt::new_size(size * pango::SCALE);
        attr_list.insert(size_attr);

        if let Some(fi) = font {
            let font_desc = pango::FontDescription::from_string(&fi.family);
            let font_attr = pango::AttrFontDesc::new(&font_desc);
            attr_list.insert(font_attr);
        }

        label.set_attributes(Some(&attr_list));

        frame.set_child(Some(&label));
        frame
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the font to display
    pub fn set_font(&self, font: &FontInfo) {
        *self.font_info.borrow_mut() = Some(font.clone());
        Self::populate_grid(
            &self.flowbox,
            &self.current_range.borrow(),
            *self.font_size.borrow(),
            &self.font_info.borrow(),
        );
    }

    /// Set the character range
    pub fn set_range(&self, range: CharacterRange) {
        *self.current_range.borrow_mut() = range;
        Self::populate_grid(
            &self.flowbox,
            &self.current_range.borrow(),
            *self.font_size.borrow(),
            &self.font_info.borrow(),
        );
    }

    /// Set the glyph size
    pub fn set_size(&self, size: i32) {
        *self.font_size.borrow_mut() = size;
        Self::populate_grid(
            &self.flowbox,
            &self.current_range.borrow(),
            *self.font_size.borrow(),
            &self.font_info.borrow(),
        );
    }
}

impl Default for GlyphGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Get Unicode character name (simplified)
fn get_unicode_name(ch: char) -> String {
    let code = ch as u32;
    match code {
        0x0020 => "SPACE".to_string(),
        0x0021 => "EXCLAMATION MARK".to_string(),
        0x0022 => "QUOTATION MARK".to_string(),
        0x0023 => "NUMBER SIGN".to_string(),
        0x0024 => "DOLLAR SIGN".to_string(),
        0x0025 => "PERCENT SIGN".to_string(),
        0x0026 => "AMPERSAND".to_string(),
        0x0027 => "APOSTROPHE".to_string(),
        0x0028 => "LEFT PARENTHESIS".to_string(),
        0x0029 => "RIGHT PARENTHESIS".to_string(),
        0x002A => "ASTERISK".to_string(),
        0x002B => "PLUS SIGN".to_string(),
        0x002C => "COMMA".to_string(),
        0x002D => "HYPHEN-MINUS".to_string(),
        0x002E => "FULL STOP".to_string(),
        0x002F => "SOLIDUS".to_string(),
        0x0030..=0x0039 => format!("DIGIT {}", ch),
        0x003A => "COLON".to_string(),
        0x003B => "SEMICOLON".to_string(),
        0x003C => "LESS-THAN SIGN".to_string(),
        0x003D => "EQUALS SIGN".to_string(),
        0x003E => "GREATER-THAN SIGN".to_string(),
        0x003F => "QUESTION MARK".to_string(),
        0x0040 => "COMMERCIAL AT".to_string(),
        0x0041..=0x005A => format!("LATIN CAPITAL LETTER {}", ch),
        0x005B => "LEFT SQUARE BRACKET".to_string(),
        0x005C => "REVERSE SOLIDUS".to_string(),
        0x005D => "RIGHT SQUARE BRACKET".to_string(),
        0x005E => "CIRCUMFLEX ACCENT".to_string(),
        0x005F => "LOW LINE".to_string(),
        0x0060 => "GRAVE ACCENT".to_string(),
        0x0061..=0x007A => format!("LATIN SMALL LETTER {}", ch.to_uppercase()),
        0x007B => "LEFT CURLY BRACKET".to_string(),
        0x007C => "VERTICAL LINE".to_string(),
        0x007D => "RIGHT CURLY BRACKET".to_string(),
        0x007E => "TILDE".to_string(),
        _ => format!("U+{:04X}", code),
    }
}
