//! Item row widget for clipboard history list

use glib::Object;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, Image, Label, Box as GtkBox, Orientation};
use std::cell::RefCell;

use crate::history::{ClipboardItem, ContentType};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ItemRow {
        pub item: RefCell<Option<ClipboardItem>>,
        pub icon: RefCell<Option<Image>>,
        pub preview_label: RefCell<Option<Label>>,
        pub time_label: RefCell<Option<Label>>,
        pub type_label: RefCell<Option<Label>>,
        pub pin_indicator: RefCell<Option<Image>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemRow {
        const NAME: &'static str = "WinuxClipboardItemRow";
        type Type = super::ItemRow;
        type ParentType = GtkBox;
    }

    impl ObjectImpl for ItemRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for ItemRow {}
    impl BoxImpl for ItemRow {}
}

glib::wrapper! {
    pub struct ItemRow(ObjectSubclass<imp::ItemRow>)
        @extends GtkBox, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for ItemRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemRow {
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(Orientation::Horizontal);
        self.set_spacing(12);
        self.set_margin_start(12);
        self.set_margin_end(12);
        self.set_margin_top(8);
        self.set_margin_bottom(8);
        self.add_css_class("clipboard-item-row");

        // Type icon
        let icon = Image::builder()
            .icon_name("text-x-generic-symbolic")
            .pixel_size(24)
            .build();
        icon.add_css_class("item-icon");
        self.append(&icon);
        *imp.icon.borrow_mut() = Some(icon);

        // Content area
        let content_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .build();

        // Preview text
        let preview_label = Label::builder()
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(50)
            .single_line_mode(true)
            .build();
        preview_label.add_css_class("item-preview");
        content_box.append(&preview_label);
        *imp.preview_label.borrow_mut() = Some(preview_label);

        // Meta info row
        let meta_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        // Type label
        let type_label = Label::builder()
            .halign(gtk4::Align::Start)
            .build();
        type_label.add_css_class("item-type");
        type_label.add_css_class("dim-label");
        meta_box.append(&type_label);
        *imp.type_label.borrow_mut() = Some(type_label);

        // Separator
        let separator = Label::new(Some("-"));
        separator.add_css_class("dim-label");
        meta_box.append(&separator);

        // Time label
        let time_label = Label::builder()
            .halign(gtk4::Align::Start)
            .build();
        time_label.add_css_class("item-time");
        time_label.add_css_class("dim-label");
        meta_box.append(&time_label);
        *imp.time_label.borrow_mut() = Some(time_label);

        content_box.append(&meta_box);
        self.append(&content_box);

        // Pin indicator
        let pin_indicator = Image::builder()
            .icon_name("view-pin-symbolic")
            .pixel_size(16)
            .visible(false)
            .build();
        pin_indicator.add_css_class("pin-indicator");
        self.append(&pin_indicator);
        *imp.pin_indicator.borrow_mut() = Some(pin_indicator);
    }

    pub fn set_item(&self, item: &ClipboardItem) {
        let imp = self.imp();

        // Store item
        *imp.item.borrow_mut() = Some(item.clone());

        // Update icon
        if let Some(icon) = imp.icon.borrow().as_ref() {
            icon.set_icon_name(Some(item.content_type.icon_name()));
        }

        // Update preview
        if let Some(label) = imp.preview_label.borrow().as_ref() {
            let preview = match item.content_type {
                ContentType::Image => {
                    if let Some((w, h)) = item.image_size {
                        format!("Image {}x{}", w, h)
                    } else {
                        "Image".to_string()
                    }
                }
                ContentType::Files => {
                    if let Some(ref paths) = item.file_paths {
                        if paths.len() == 1 {
                            paths[0].rsplit('/').next().unwrap_or(&paths[0]).to_string()
                        } else {
                            format!("{} files", paths.len())
                        }
                    } else {
                        item.preview.clone()
                    }
                }
                _ => item.preview.clone(),
            };
            label.set_text(&preview);
        }

        // Update type label
        if let Some(label) = imp.type_label.borrow().as_ref() {
            let type_text = format!("{} - {}", item.content_type.display_name(), item.format_size());
            label.set_text(&type_text);
        }

        // Update time
        if let Some(label) = imp.time_label.borrow().as_ref() {
            label.set_text(&item.format_time());
        }

        // Update pin indicator
        if let Some(pin) = imp.pin_indicator.borrow().as_ref() {
            pin.set_visible(item.pinned);
        }

        // Update row styling
        if item.pinned {
            self.add_css_class("pinned");
        } else {
            self.remove_css_class("pinned");
        }
    }

    pub fn get_item(&self) -> Option<ClipboardItem> {
        self.imp().item.borrow().clone()
    }

    pub fn get_item_id(&self) -> Option<u64> {
        self.imp().item.borrow().as_ref().map(|i| i.id)
    }
}

/// CSS styles for item rows
pub fn item_row_css() -> &'static str {
    r#"
    .clipboard-item-row {
        padding: 8px 12px;
        border-radius: 8px;
        transition: background-color 200ms ease;
    }

    .clipboard-item-row:hover {
        background-color: alpha(@accent_bg_color, 0.1);
    }

    .clipboard-item-row.pinned {
        background-color: alpha(@accent_bg_color, 0.08);
    }

    .clipboard-item-row.pinned:hover {
        background-color: alpha(@accent_bg_color, 0.15);
    }

    .item-icon {
        opacity: 0.7;
    }

    .item-preview {
        font-size: 14px;
        font-weight: 500;
    }

    .item-type, .item-time {
        font-size: 12px;
    }

    .pin-indicator {
        opacity: 0.6;
    }
    "#
}
