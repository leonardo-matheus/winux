// Input Area - Message input with attachment support

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, DropTarget, FileDialog, Image, Label, Orientation,
    ScrolledWindow, TextView, WrapMode, Align,
};
use gtk4::gdk;
use gtk4::gio;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct InputArea {
    pub widget: GtkBox,
    text_view: TextView,
    send_button: Button,
    attachments_box: GtkBox,
    attachments: Rc<RefCell<Vec<Attachment>>>,
}

#[derive(Debug, Clone)]
pub enum Attachment {
    Image(String),
    File(String),
}

impl InputArea {
    pub fn new() -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 8);
        widget.add_css_class("input-area");

        // Attachments preview area
        let attachments_box = GtkBox::new(Orientation::Horizontal, 8);
        attachments_box.set_margin_start(8);
        attachments_box.set_margin_end(8);
        attachments_box.set_visible(false);
        widget.append(&attachments_box);

        // Input container
        let input_container = GtkBox::new(Orientation::Horizontal, 8);
        input_container.add_css_class("input-container");

        // Attach button
        let attach_btn = Button::builder()
            .icon_name("mail-attachment-symbolic")
            .tooltip_text("Attach file (Ctrl+Shift+A)")
            .build();
        attach_btn.add_css_class("attach-button");
        attach_btn.add_css_class("flat");
        input_container.append(&attach_btn);

        // Text input with scroll
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .max_content_height(150)
            .hexpand(true)
            .build();

        let text_view = TextView::builder()
            .wrap_mode(WrapMode::Word)
            .accepts_tab(false)
            .build();
        text_view.add_css_class("input-text");

        scroll.set_child(Some(&text_view));
        input_container.append(&scroll);

        // Send button
        let send_button = Button::builder()
            .icon_name("paper-plane-symbolic")
            .tooltip_text("Send (Enter)")
            .sensitive(false)
            .build();
        send_button.add_css_class("send-button");
        input_container.append(&send_button);

        widget.append(&input_container);

        let attachments: Rc<RefCell<Vec<Attachment>>> = Rc::new(RefCell::new(Vec::new()));

        // Setup drag and drop
        let drop_target = DropTarget::new(gio::File::static_type(), gdk::DragAction::COPY);
        let attachments_clone = attachments.clone();
        let attachments_box_clone = attachments_box.clone();
        drop_target.connect_drop(move |_, value, _, _| {
            if let Ok(file) = value.get::<gio::File>() {
                if let Some(path) = file.path() {
                    let path_str = path.to_string_lossy().to_string();
                    Self::add_attachment_internal(
                        &attachments_clone,
                        &attachments_box_clone,
                        &path_str,
                    );
                }
            }
            true
        });
        widget.add_controller(drop_target);

        // Handle attach button
        let attachments_clone = attachments.clone();
        let attachments_box_clone = attachments_box.clone();
        let widget_clone = widget.clone();
        attach_btn.connect_clicked(move |_| {
            let dialog = FileDialog::builder()
                .title("Attach File")
                .modal(true)
                .build();

            let attachments = attachments_clone.clone();
            let attachments_box = attachments_box_clone.clone();
            dialog.open(
                gtk4::Window::NONE,
                None::<&gio::Cancellable>,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            Self::add_attachment_internal(&attachments, &attachments_box, &path_str);
                        }
                    }
                },
            );
        });

        // Handle text changes for send button sensitivity
        let send_btn_clone = send_button.clone();
        let buffer = text_view.buffer();
        buffer.connect_changed(move |buffer| {
            let has_text = buffer.char_count() > 0;
            send_btn_clone.set_sensitive(has_text);
        });

        Self {
            widget,
            text_view,
            send_button,
            attachments_box,
            attachments,
        }
    }

    fn add_attachment_internal(
        attachments: &Rc<RefCell<Vec<Attachment>>>,
        attachments_box: &GtkBox,
        path: &str,
    ) {
        let attachment = if Self::is_image(path) {
            Attachment::Image(path.to_string())
        } else {
            Attachment::File(path.to_string())
        };

        attachments.borrow_mut().push(attachment.clone());
        attachments_box.set_visible(true);

        // Create preview widget
        let preview = Self::create_attachment_preview(path, &attachment);

        // Add remove button
        let remove_btn = Button::builder()
            .icon_name("window-close-symbolic")
            .build();
        remove_btn.add_css_class("attachment-remove");
        remove_btn.add_css_class("flat");
        remove_btn.add_css_class("circular");

        let attachments_clone = attachments.clone();
        let attachments_box_clone = attachments_box.clone();
        let preview_clone = preview.clone();
        let path_clone = path.to_string();
        remove_btn.connect_clicked(move |_| {
            attachments_clone.borrow_mut().retain(|a| {
                match a {
                    Attachment::Image(p) | Attachment::File(p) => p != &path_clone,
                }
            });
            attachments_box_clone.remove(&preview_clone);
            if attachments_clone.borrow().is_empty() {
                attachments_box_clone.set_visible(false);
            }
        });

        preview.append(&remove_btn);
        attachments_box.append(&preview);
    }

    fn create_attachment_preview(path: &str, attachment: &Attachment) -> GtkBox {
        let container = GtkBox::new(Orientation::Vertical, 4);
        container.add_css_class("attachment-preview");

        match attachment {
            Attachment::Image(_) => {
                let image = Image::from_file(path);
                image.set_pixel_size(64);
                container.append(&image);
            }
            Attachment::File(_) => {
                let icon = Image::from_icon_name("text-x-generic-symbolic");
                icon.set_pixel_size(32);
                container.append(&icon);
            }
        }

        let file_name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let name_label = Label::new(Some(file_name));
        name_label.add_css_class("attachment-name");
        name_label.set_max_width_chars(15);
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
        container.append(&name_label);

        container
    }

    fn is_image(path: &str) -> bool {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        mime.as_ref().starts_with("image/")
    }

    /// Connect send action
    pub fn connect_send<F>(&self, callback: F)
    where
        F: Fn(String, Vec<Attachment>) + 'static + Clone,
    {
        let text_view = self.text_view.clone();
        let attachments = self.attachments.clone();
        let attachments_box = self.attachments_box.clone();

        // Handle send button click
        let callback_clone = callback.clone();
        let text_view_clone = text_view.clone();
        let attachments_clone = attachments.clone();
        let attachments_box_clone = attachments_box.clone();
        self.send_button.connect_clicked(move |btn| {
            let buffer = text_view_clone.buffer();
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            let text = text.trim().to_string();

            if !text.is_empty() {
                let current_attachments = attachments_clone.borrow().clone();
                callback_clone(text, current_attachments);

                // Clear input
                buffer.set_text("");
                attachments_clone.borrow_mut().clear();

                // Clear attachment previews
                while let Some(child) = attachments_box_clone.first_child() {
                    attachments_box_clone.remove(&child);
                }
                attachments_box_clone.set_visible(false);
                btn.set_sensitive(false);
            }
        });

        // Handle Enter key
        let text_view_clone = text_view.clone();
        let send_button = self.send_button.clone();
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, modifier| {
            if key == gdk::Key::Return && !modifier.contains(gdk::ModifierType::SHIFT_MASK) {
                if send_button.is_sensitive() {
                    send_button.emit_clicked();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        self.text_view.add_controller(key_controller);
    }

    /// Set placeholder text
    pub fn set_placeholder(&self, placeholder: &str) {
        // GTK4 TextView doesn't have built-in placeholder
        // We'd need to implement it manually with CSS or overlay
    }

    /// Focus the input
    pub fn focus(&self) {
        self.text_view.grab_focus();
    }

    /// Get current text
    pub fn get_text(&self) -> String {
        let buffer = self.text_view.buffer();
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string()
    }

    /// Clear input
    pub fn clear(&self) {
        self.text_view.buffer().set_text("");
        self.attachments.borrow_mut().clear();

        while let Some(child) = self.attachments_box.first_child() {
            self.attachments_box.remove(&child);
        }
        self.attachments_box.set_visible(false);
    }
}

impl Default for InputArea {
    fn default() -> Self {
        Self::new()
    }
}
