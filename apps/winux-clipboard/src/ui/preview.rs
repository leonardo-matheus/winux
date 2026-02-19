//! Preview panel for clipboard content

use glib::Object;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, Label, Image, Picture, ScrolledWindow, TextView, Box as GtkBox, Orientation, Button, Separator};
use std::cell::RefCell;

use crate::history::{ClipboardItem, ContentType};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct PreviewPanel {
        pub current_item: RefCell<Option<ClipboardItem>>,
        pub stack: RefCell<Option<gtk4::Stack>>,
        pub text_view: RefCell<Option<TextView>>,
        pub image_picture: RefCell<Option<Picture>>,
        pub files_list: RefCell<Option<GtkBox>>,
        pub empty_label: RefCell<Option<Label>>,
        pub info_label: RefCell<Option<Label>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreviewPanel {
        const NAME: &'static str = "WinuxClipboardPreviewPanel";
        type Type = super::PreviewPanel;
        type ParentType = GtkBox;
    }

    impl ObjectImpl for PreviewPanel {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for PreviewPanel {}
    impl BoxImpl for PreviewPanel {}
}

glib::wrapper! {
    pub struct PreviewPanel(ObjectSubclass<imp::PreviewPanel>)
        @extends GtkBox, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for PreviewPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewPanel {
    pub fn new() -> Self {
        Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(Orientation::Vertical);
        self.set_spacing(0);
        self.set_width_request(350);
        self.add_css_class("preview-panel");

        // Header
        let header = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(16)
            .margin_end(16)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        let title = Label::builder()
            .label("Preview")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        title.add_css_class("title-4");
        header.append(&title);

        self.append(&header);

        // Separator
        let separator = Separator::new(Orientation::Horizontal);
        self.append(&separator);

        // Content stack
        let stack = gtk4::Stack::builder()
            .vexpand(true)
            .transition_type(gtk4::StackTransitionType::Crossfade)
            .transition_duration(150)
            .build();

        // Empty state
        let empty_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .spacing(12)
            .build();

        let empty_icon = Image::builder()
            .icon_name("edit-paste-symbolic")
            .pixel_size(48)
            .opacity(0.5)
            .build();
        empty_box.append(&empty_icon);

        let empty_label = Label::builder()
            .label("Select an item to preview")
            .build();
        empty_label.add_css_class("dim-label");
        empty_box.append(&empty_label);
        *imp.empty_label.borrow_mut() = Some(empty_label);

        stack.add_named(&empty_box, Some("empty"));

        // Text preview
        let text_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();

        let text_view = TextView::builder()
            .editable(false)
            .cursor_visible(false)
            .wrap_mode(gtk4::WrapMode::Word)
            .margin_start(16)
            .margin_end(16)
            .margin_top(12)
            .margin_bottom(12)
            .monospace(true)
            .build();
        text_view.add_css_class("preview-text");
        text_scroll.set_child(Some(&text_view));
        *imp.text_view.borrow_mut() = Some(text_view);

        stack.add_named(&text_scroll, Some("text"));

        // Image preview
        let image_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .margin_start(16)
            .margin_end(16)
            .margin_top(16)
            .margin_bottom(16)
            .build();

        let picture = Picture::builder()
            .can_shrink(true)
            .keep_aspect_ratio(true)
            .build();
        picture.add_css_class("preview-image");
        image_box.append(&picture);
        *imp.image_picture.borrow_mut() = Some(picture);

        stack.add_named(&image_box, Some("image"));

        // Files preview
        let files_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();

        let files_list = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(16)
            .margin_end(16)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        files_scroll.set_child(Some(&files_list));
        *imp.files_list.borrow_mut() = Some(files_list);

        stack.add_named(&files_scroll, Some("files"));

        // Set initial state
        stack.set_visible_child_name("empty");
        *imp.stack.borrow_mut() = Some(stack.clone());
        self.append(&stack);

        // Info bar
        let info_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(16)
            .margin_end(16)
            .margin_top(8)
            .margin_bottom(12)
            .build();

        let info_label = Label::builder()
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        info_label.add_css_class("dim-label");
        info_label.add_css_class("caption");
        info_box.append(&info_label);
        *imp.info_label.borrow_mut() = Some(info_label);

        self.append(&info_box);
    }

    pub fn set_item(&self, item: Option<&ClipboardItem>) {
        let imp = self.imp();

        *imp.current_item.borrow_mut() = item.cloned();

        let stack = imp.stack.borrow();
        let stack = stack.as_ref().unwrap();

        match item {
            None => {
                stack.set_visible_child_name("empty");
                if let Some(label) = imp.info_label.borrow().as_ref() {
                    label.set_text("");
                }
            }
            Some(item) => {
                match item.content_type {
                    ContentType::Text | ContentType::Html | ContentType::Rtf => {
                        self.show_text_preview(item);
                        stack.set_visible_child_name("text");
                    }
                    ContentType::Image => {
                        self.show_image_preview(item);
                        stack.set_visible_child_name("image");
                    }
                    ContentType::Files => {
                        self.show_files_preview(item);
                        stack.set_visible_child_name("files");
                    }
                }

                // Update info
                if let Some(label) = imp.info_label.borrow().as_ref() {
                    let info = format!(
                        "{} - {} - {}",
                        item.content_type.display_name(),
                        item.format_size(),
                        item.format_time()
                    );
                    label.set_text(&info);
                }
            }
        }
    }

    fn show_text_preview(&self, item: &ClipboardItem) {
        let imp = self.imp();

        if let Some(text_view) = imp.text_view.borrow().as_ref() {
            let buffer = text_view.buffer();

            let text = if item.content_type == ContentType::Html {
                // Show HTML source
                item.html.as_ref().unwrap_or(&item.content)
            } else {
                &item.content
            };

            buffer.set_text(text);
        }
    }

    fn show_image_preview(&self, item: &ClipboardItem) {
        let imp = self.imp();

        if let Some(picture) = imp.image_picture.borrow().as_ref() {
            if let Some(ref path) = item.image_path {
                picture.set_filename(Some(path));
            }
        }
    }

    fn show_files_preview(&self, item: &ClipboardItem) {
        let imp = self.imp();

        if let Some(files_list) = imp.files_list.borrow().as_ref() {
            // Clear existing children
            while let Some(child) = files_list.first_child() {
                files_list.remove(&child);
            }

            if let Some(ref paths) = item.file_paths {
                for path in paths {
                    let row = self.create_file_row(path);
                    files_list.append(&row);
                }
            }
        }
    }

    fn create_file_row(&self, path: &str) -> GtkBox {
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        row.add_css_class("file-row");

        // File icon
        let icon_name = if std::path::Path::new(path).is_dir() {
            "folder-symbolic"
        } else {
            "text-x-generic-symbolic"
        };

        let icon = Image::builder()
            .icon_name(icon_name)
            .pixel_size(16)
            .build();
        row.append(&icon);

        // File name
        let name = path.rsplit('/').next().unwrap_or(path);
        let label = Label::builder()
            .label(name)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .ellipsize(gtk4::pango::EllipsizeMode::Middle)
            .tooltip_text(path)
            .build();
        row.append(&label);

        row
    }

    pub fn get_current_item(&self) -> Option<ClipboardItem> {
        self.imp().current_item.borrow().clone()
    }

    pub fn clear(&self) {
        self.set_item(None);
    }
}

/// CSS styles for preview panel
pub fn preview_css() -> &'static str {
    r#"
    .preview-panel {
        background-color: @card_bg_color;
        border-left: 1px solid @borders;
    }

    .preview-text {
        font-family: monospace;
        font-size: 13px;
        background: transparent;
    }

    .preview-image {
        border-radius: 8px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .file-row {
        padding: 8px 12px;
        border-radius: 6px;
        background-color: alpha(@card_bg_color, 0.5);
    }

    .file-row:hover {
        background-color: alpha(@accent_bg_color, 0.1);
    }
    "#
}
