//! Progress dialog for long-running operations

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, ProgressBar};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Progress dialog for archive operations
pub struct ProgressDialog {
    dialog: adw::Window,
    title_label: Label,
    status_label: Label,
    file_label: Label,
    progress_bar: ProgressBar,
    cancel_button: Button,
    close_button: Button,
    cancel_flag: Arc<AtomicBool>,
    is_complete: Rc<RefCell<bool>>,
}

impl ProgressDialog {
    /// Create a new progress dialog
    pub fn new(parent: &adw::ApplicationWindow, title: &str) -> Self {
        // Create dialog window
        let dialog = adw::Window::builder()
            .title(title)
            .modal(true)
            .transient_for(parent)
            .default_width(400)
            .default_height(150)
            .resizable(false)
            .deletable(false)
            .build();

        // Create content
        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);

        // Title label
        let title_label = Label::new(Some(title));
        title_label.add_css_class("title-2");
        title_label.set_xalign(0.0);

        // Status label
        let status_label = Label::new(Some("Preparing..."));
        status_label.set_xalign(0.0);
        status_label.add_css_class("dim-label");

        // File label
        let file_label = Label::new(None);
        file_label.set_xalign(0.0);
        file_label.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);

        // Progress bar
        let progress_bar = ProgressBar::new();
        progress_bar.set_show_text(true);

        // Buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::End);
        button_box.set_margin_top(12);

        let cancel_button = Button::with_label("Cancel");

        let close_button = Button::with_label("Close");
        close_button.add_css_class("suggested-action");
        close_button.set_visible(false);

        button_box.append(&cancel_button);
        button_box.append(&close_button);

        // Assemble content
        content.append(&title_label);
        content.append(&status_label);
        content.append(&file_label);
        content.append(&progress_bar);
        content.append(&button_box);

        dialog.set_content(Some(&content));

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let is_complete = Rc::new(RefCell::new(false));

        let progress_dialog = Self {
            dialog,
            title_label,
            status_label,
            file_label,
            progress_bar,
            cancel_button,
            close_button,
            cancel_flag,
            is_complete,
        };

        // Connect cancel button
        let cancel_flag_clone = progress_dialog.cancel_flag.clone();
        let cancel_btn_clone = progress_dialog.cancel_button.clone();
        progress_dialog.cancel_button.connect_clicked(move |_| {
            cancel_flag_clone.store(true, Ordering::Relaxed);
            cancel_btn_clone.set_sensitive(false);
            cancel_btn_clone.set_label("Cancelling...");
        });

        // Connect close button
        let dialog_clone = progress_dialog.dialog.clone();
        progress_dialog.close_button.connect_clicked(move |_| {
            dialog_clone.close();
        });

        progress_dialog
    }

    /// Show the dialog
    pub fn show(&self) {
        self.dialog.present();
    }

    /// Close the dialog
    pub fn close(&self) {
        self.dialog.close();
    }

    /// Update progress
    pub fn update(&self, current: usize, total: usize, current_file: &str) {
        let fraction = if total > 0 {
            current as f64 / total as f64
        } else {
            0.0
        };

        self.progress_bar.set_fraction(fraction);
        self.progress_bar.set_text(Some(&format!("{}/{}", current, total)));

        self.status_label.set_text(&format!("Processing {} of {}", current + 1, total));
        self.file_label.set_text(current_file);
    }

    /// Update with byte progress
    pub fn update_bytes(&self, bytes_processed: u64, total_bytes: u64, current_file: &str) {
        let fraction = if total_bytes > 0 {
            bytes_processed as f64 / total_bytes as f64
        } else {
            0.0
        };

        self.progress_bar.set_fraction(fraction);
        self.progress_bar.set_text(Some(&format!(
            "{} / {}",
            format_size(bytes_processed),
            format_size(total_bytes)
        )));

        self.file_label.set_text(current_file);
    }

    /// Set complete state
    pub fn set_complete(&self, message: &str) {
        *self.is_complete.borrow_mut() = true;

        self.progress_bar.set_fraction(1.0);
        self.progress_bar.set_text(Some("Complete"));

        self.status_label.set_text(message);
        self.file_label.set_text("");

        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
    }

    /// Set error state
    pub fn set_error(&self, message: &str) {
        *self.is_complete.borrow_mut() = true;

        self.status_label.set_text(message);
        self.status_label.add_css_class("error");

        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
    }

    /// Set cancelled state
    pub fn set_cancelled(&self) {
        *self.is_complete.borrow_mut() = true;

        self.status_label.set_text("Operation cancelled");

        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
    }

    /// Get cancel flag for checking in operation
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel_flag.clone()
    }

    /// Check if operation was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }

    /// Check if operation is complete
    pub fn is_complete(&self) -> bool {
        *self.is_complete.borrow()
    }

    /// Pulse the progress bar (for indeterminate progress)
    pub fn pulse(&self) {
        self.progress_bar.pulse();
    }

    /// Set indeterminate mode
    pub fn set_indeterminate(&self, message: &str) {
        self.status_label.set_text(message);
        self.progress_bar.set_show_text(false);

        // Start pulsing
        let progress_bar = self.progress_bar.clone();
        let is_complete = self.is_complete.clone();

        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if *is_complete.borrow() {
                return gtk4::glib::ControlFlow::Break;
            }
            progress_bar.pulse();
            gtk4::glib::ControlFlow::Continue
        });
    }
}

/// Format file size
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Create a simple confirmation dialog
pub fn confirm_dialog(
    parent: &adw::ApplicationWindow,
    title: &str,
    message: &str,
    confirm_label: &str,
    destructive: bool,
) -> adw::MessageDialog {
    let dialog = adw::MessageDialog::new(Some(parent), Some(title), Some(message));

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("confirm", confirm_label);

    if destructive {
        dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
    } else {
        dialog.set_response_appearance("confirm", adw::ResponseAppearance::Suggested);
    }

    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    dialog
}

/// Create a password input dialog
pub fn password_dialog(
    parent: &adw::ApplicationWindow,
    title: &str,
) -> (adw::MessageDialog, gtk4::PasswordEntry) {
    let dialog = adw::MessageDialog::new(
        Some(parent),
        Some(title),
        Some("Enter password to decrypt the archive:"),
    );

    let password_entry = gtk4::PasswordEntry::new();
    password_entry.set_show_peek_icon(true);

    dialog.set_extra_child(Some(&password_entry));

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("ok", "OK");
    dialog.set_response_appearance("ok", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("ok"));

    (dialog, password_entry)
}
