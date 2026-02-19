//! Avatar picker dialog

use gtk4::prelude::*;
use gtk4::{Box, Button, FileChooserAction, FileChooserNative, FileFilter, FlowBox, Image, Orientation, ScrolledWindow, Window};
use libadwaita as adw;
use adw::prelude::*;

use std::path::{Path, PathBuf};

/// Avatar picker dialog for selecting user avatars
pub struct AvatarPicker {
    dialog: adw::Dialog,
    selected_path: std::cell::RefCell<Option<PathBuf>>,
}

impl AvatarPicker {
    /// Create a new avatar picker dialog
    pub fn new() -> Self {
        let dialog = adw::Dialog::builder()
            .title("Escolher Avatar")
            .build();

        let content = Box::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Header
        let header = adw::HeaderBar::new();
        header.set_show_end_title_buttons(false);
        header.set_show_start_title_buttons(false);

        let cancel_btn = Button::with_label("Cancelar");
        cancel_btn.connect_clicked(glib::clone!(
            #[weak]
            dialog,
            move |_| {
                dialog.close();
            }
        ));
        header.pack_start(&cancel_btn);

        let select_btn = Button::with_label("Selecionar");
        select_btn.add_css_class("suggested-action");
        header.pack_end(&select_btn);

        content.append(&header);

        // Tabs for different avatar sources
        let stack = gtk4::Stack::new();
        stack.set_vexpand(true);

        // System avatars
        let system_avatars = Self::create_system_avatars_page();
        stack.add_titled(&system_avatars, Some("system"), "Avatares do Sistema");

        // Custom image
        let custom_page = Self::create_custom_image_page();
        stack.add_titled(&custom_page, Some("custom"), "Imagem Personalizada");

        // Camera (if available)
        let camera_page = Self::create_camera_page();
        stack.add_titled(&camera_page, Some("camera"), "Camera");

        let switcher = gtk4::StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_halign(gtk4::Align::Center);
        switcher.set_margin_bottom(12);

        content.append(&switcher);
        content.append(&stack);

        dialog.set_child(Some(&content));

        AvatarPicker {
            dialog,
            selected_path: std::cell::RefCell::new(None),
        }
    }

    /// Show the dialog
    pub fn show<F>(&self, parent: &Window, on_select: F)
    where
        F: Fn(PathBuf) + 'static,
    {
        let selected_path = self.selected_path.clone();
        let dialog = self.dialog.clone();

        // TODO: Connect select button to callback
        // For now, just present the dialog

        self.dialog.present(Some(parent));
    }

    /// Show avatar picker and return selected path
    pub fn pick_avatar(parent: &Window) -> Option<PathBuf> {
        let picker = Self::new();
        picker.dialog.present(Some(parent));

        // Note: In a real implementation, this would need to be async
        // or use a callback pattern
        None
    }

    fn create_system_avatars_page() -> ScrolledWindow {
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();

        let flowbox = FlowBox::new();
        flowbox.set_valign(gtk4::Align::Start);
        flowbox.set_max_children_per_line(6);
        flowbox.set_min_children_per_line(4);
        flowbox.set_selection_mode(gtk4::SelectionMode::Single);
        flowbox.set_homogeneous(true);
        flowbox.set_row_spacing(12);
        flowbox.set_column_spacing(12);
        flowbox.set_margin_top(12);
        flowbox.set_margin_bottom(12);
        flowbox.set_margin_start(12);
        flowbox.set_margin_end(12);

        // Load system avatars from /usr/share/pixmaps/faces/
        let avatar_dirs = [
            "/usr/share/pixmaps/faces",
            "/usr/share/gnome/pixmaps/faces",
            "/usr/share/icons/hicolor/96x96/apps",
        ];

        let mut found_avatars = false;

        for dir in avatar_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "png" || ext == "jpg" || ext == "jpeg" {
                            let avatar_box = Box::new(Orientation::Vertical, 4);

                            if let Ok(texture) = gdk4::Texture::from_filename(&path) {
                                let image = Image::from_paintable(Some(&texture));
                                image.set_pixel_size(80);
                                image.add_css_class("card");

                                avatar_box.append(&image);
                                flowbox.append(&avatar_box);
                                found_avatars = true;
                            }
                        }
                    }
                }
            }
        }

        // If no system avatars found, show placeholder icons
        if !found_avatars {
            let placeholder_icons = [
                "avatar-default-symbolic",
                "user-info-symbolic",
                "face-smile-symbolic",
                "face-cool-symbolic",
                "face-worried-symbolic",
                "face-laugh-symbolic",
            ];

            for icon_name in placeholder_icons {
                let avatar_box = Box::new(Orientation::Vertical, 4);

                let image = Image::from_icon_name(icon_name);
                image.set_pixel_size(64);
                image.add_css_class("card");
                image.set_margin_top(8);
                image.set_margin_bottom(8);
                image.set_margin_start(8);
                image.set_margin_end(8);

                avatar_box.append(&image);
                flowbox.append(&avatar_box);
            }
        }

        scrolled.set_child(Some(&flowbox));
        scrolled
    }

    fn create_custom_image_page() -> Box {
        let content = Box::new(Orientation::Vertical, 24);
        content.set_valign(gtk4::Align::Center);
        content.set_halign(gtk4::Align::Center);
        content.set_vexpand(true);

        // Preview area
        let preview_frame = gtk4::Frame::new(None);
        preview_frame.set_halign(gtk4::Align::Center);

        let preview = adw::Avatar::new(128, Some("Preview"), true);
        preview_frame.set_child(Some(&preview));

        content.append(&preview_frame);

        // Instructions
        let instructions = gtk4::Label::new(Some("Escolha uma imagem do seu computador"));
        instructions.add_css_class("dim-label");
        content.append(&instructions);

        // File chooser button
        let choose_btn = Button::with_label("Escolher Arquivo...");
        choose_btn.add_css_class("pill");
        choose_btn.add_css_class("suggested-action");
        choose_btn.set_halign(gtk4::Align::Center);

        choose_btn.connect_clicked(move |btn| {
            if let Some(window) = btn.root().and_then(|r| r.downcast::<Window>().ok()) {
                let filter = FileFilter::new();
                filter.set_name(Some("Imagens"));
                filter.add_mime_type("image/png");
                filter.add_mime_type("image/jpeg");
                filter.add_mime_type("image/gif");
                filter.add_mime_type("image/webp");
                filter.add_pattern("*.png");
                filter.add_pattern("*.jpg");
                filter.add_pattern("*.jpeg");
                filter.add_pattern("*.gif");
                filter.add_pattern("*.webp");

                let chooser = FileChooserNative::new(
                    Some("Escolher Imagem"),
                    Some(&window),
                    FileChooserAction::Open,
                    Some("Abrir"),
                    Some("Cancelar"),
                );

                chooser.add_filter(&filter);

                chooser.connect_response(move |chooser, response| {
                    if response == gtk4::ResponseType::Accept {
                        if let Some(file) = chooser.file() {
                            if let Some(path) = file.path() {
                                tracing::info!("Selected image: {:?}", path);
                                // TODO: Update preview and store selection
                            }
                        }
                    }
                });

                chooser.show();
            }
        });

        content.append(&choose_btn);

        // Supported formats info
        let formats_label = gtk4::Label::new(Some("Formatos suportados: PNG, JPEG, GIF, WebP"));
        formats_label.add_css_class("dim-label");
        formats_label.add_css_class("caption");
        content.append(&formats_label);

        content
    }

    fn create_camera_page() -> Box {
        let content = Box::new(Orientation::Vertical, 24);
        content.set_valign(gtk4::Align::Center);
        content.set_halign(gtk4::Align::Center);
        content.set_vexpand(true);

        // Camera preview placeholder
        let preview_frame = gtk4::Frame::new(None);
        preview_frame.set_halign(gtk4::Align::Center);

        let preview_box = Box::new(Orientation::Vertical, 12);
        preview_box.set_size_request(256, 256);
        preview_box.set_valign(gtk4::Align::Center);
        preview_box.set_halign(gtk4::Align::Center);

        let camera_icon = Image::from_icon_name("camera-photo-symbolic");
        camera_icon.set_pixel_size(64);
        camera_icon.add_css_class("dim-label");
        preview_box.append(&camera_icon);

        let no_camera_label = gtk4::Label::new(Some("Camera nao disponivel"));
        no_camera_label.add_css_class("dim-label");
        preview_box.append(&no_camera_label);

        preview_frame.set_child(Some(&preview_box));
        content.append(&preview_frame);

        // Take photo button (disabled if no camera)
        let take_btn = Button::with_label("Tirar Foto");
        take_btn.add_css_class("pill");
        take_btn.add_css_class("suggested-action");
        take_btn.set_halign(gtk4::Align::Center);
        take_btn.set_sensitive(false); // Disabled until camera is detected

        content.append(&take_btn);

        // Check for camera availability
        // In a real implementation, this would use PipeWire or V4L2

        content
    }
}

impl Default for AvatarPicker {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick function to show avatar picker dialog
pub fn show_avatar_picker<F>(parent: &Window, on_select: F)
where
    F: Fn(PathBuf) + 'static,
{
    let picker = AvatarPicker::new();
    picker.show(parent, on_select);
}
