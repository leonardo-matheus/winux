//! Pairing dialog for Bluetooth device pairing

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bluez::{PairingMethod, BluetoothDevice, DeviceType};

/// Dialog for handling Bluetooth pairing
pub struct PairingDialog {
    dialog: adw::Dialog,
    method: PairingMethod,
    pin_entry: Option<gtk4::Entry>,
    passkey_label: Option<gtk4::Label>,
}

impl PairingDialog {
    /// Create a new pairing dialog for PIN entry
    pub fn new_pin_entry(device_name: &str) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Parear Dispositivo")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Icon
        let icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
        icon.set_pixel_size(64);
        content.append(&icon);

        // Title
        let title = gtk4::Label::new(Some(&format!("Parear com {}", device_name)));
        title.add_css_class("title-2");
        content.append(&title);

        // Description
        let desc = gtk4::Label::new(Some("Digite o PIN mostrado no dispositivo:"));
        desc.add_css_class("dim-label");
        content.append(&desc);

        // PIN entry
        let pin_entry = gtk4::Entry::builder()
            .placeholder_text("PIN")
            .max_length(16)
            .input_purpose(gtk4::InputPurpose::Pin)
            .halign(gtk4::Align::Center)
            .width_chars(12)
            .build();
        pin_entry.add_css_class("monospace");
        content.append(&pin_entry);

        // Buttons
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(16);

        let cancel_btn = gtk4::Button::with_label("Cancelar");
        cancel_btn.add_css_class("pill");

        let pair_btn = gtk4::Button::with_label("Parear");
        pair_btn.add_css_class("pill");
        pair_btn.add_css_class("suggested-action");

        button_box.append(&cancel_btn);
        button_box.append(&pair_btn);
        content.append(&button_box);

        dialog.set_child(Some(&content));

        // Connect close on cancel
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        Self {
            dialog,
            method: PairingMethod::PinCode,
            pin_entry: Some(pin_entry),
            passkey_label: None,
        }
    }

    /// Create a new pairing dialog for passkey display
    pub fn new_passkey_display(device_name: &str, passkey: u32) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Parear Dispositivo")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Icon
        let icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
        icon.set_pixel_size(64);
        content.append(&icon);

        // Title
        let title = gtk4::Label::new(Some(&format!("Parear com {}", device_name)));
        title.add_css_class("title-2");
        content.append(&title);

        // Description
        let desc = gtk4::Label::new(Some("Digite este codigo no outro dispositivo:"));
        desc.add_css_class("dim-label");
        content.append(&desc);

        // Passkey display
        let passkey_label = gtk4::Label::new(Some(&format!("{:06}", passkey)));
        passkey_label.add_css_class("title-1");
        passkey_label.add_css_class("monospace");
        content.append(&passkey_label);

        // Cancel button
        let cancel_btn = gtk4::Button::with_label("Cancelar");
        cancel_btn.add_css_class("pill");
        cancel_btn.set_halign(gtk4::Align::Center);
        cancel_btn.set_margin_top(16);
        content.append(&cancel_btn);

        dialog.set_child(Some(&content));

        // Connect close on cancel
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        Self {
            dialog,
            method: PairingMethod::PasskeyDisplay,
            pin_entry: None,
            passkey_label: Some(passkey_label),
        }
    }

    /// Create a new pairing dialog for passkey entry
    pub fn new_passkey_entry(device_name: &str) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Parear Dispositivo")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Icon
        let icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
        icon.set_pixel_size(64);
        content.append(&icon);

        // Title
        let title = gtk4::Label::new(Some(&format!("Parear com {}", device_name)));
        title.add_css_class("title-2");
        content.append(&title);

        // Description
        let desc = gtk4::Label::new(Some("Digite o codigo mostrado no outro dispositivo:"));
        desc.add_css_class("dim-label");
        content.append(&desc);

        // Passkey entry
        let pin_entry = gtk4::Entry::builder()
            .placeholder_text("000000")
            .max_length(6)
            .input_purpose(gtk4::InputPurpose::Digits)
            .halign(gtk4::Align::Center)
            .width_chars(8)
            .build();
        pin_entry.add_css_class("monospace");
        content.append(&pin_entry);

        // Buttons
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(16);

        let cancel_btn = gtk4::Button::with_label("Cancelar");
        cancel_btn.add_css_class("pill");

        let confirm_btn = gtk4::Button::with_label("Confirmar");
        confirm_btn.add_css_class("pill");
        confirm_btn.add_css_class("suggested-action");

        button_box.append(&cancel_btn);
        button_box.append(&confirm_btn);
        content.append(&button_box);

        dialog.set_child(Some(&content));

        // Connect close on cancel
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        Self {
            dialog,
            method: PairingMethod::PasskeyEntry,
            pin_entry: Some(pin_entry),
            passkey_label: None,
        }
    }

    /// Create a new pairing dialog for passkey confirmation
    pub fn new_passkey_confirmation(device_name: &str, passkey: u32) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Confirmar Pareamento")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Icon
        let icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
        icon.set_pixel_size(64);
        content.append(&icon);

        // Title
        let title = gtk4::Label::new(Some(&format!("Parear com {}", device_name)));
        title.add_css_class("title-2");
        content.append(&title);

        // Description
        let desc = gtk4::Label::new(Some("Confirme se o codigo abaixo corresponde ao mostrado no dispositivo:"));
        desc.add_css_class("dim-label");
        desc.set_wrap(true);
        desc.set_max_width_chars(40);
        content.append(&desc);

        // Passkey display
        let passkey_label = gtk4::Label::new(Some(&format!("{:06}", passkey)));
        passkey_label.add_css_class("title-1");
        passkey_label.add_css_class("monospace");
        content.append(&passkey_label);

        // Buttons
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(16);

        let reject_btn = gtk4::Button::with_label("Nao Corresponde");
        reject_btn.add_css_class("pill");
        reject_btn.add_css_class("destructive-action");

        let confirm_btn = gtk4::Button::with_label("Corresponde");
        confirm_btn.add_css_class("pill");
        confirm_btn.add_css_class("suggested-action");

        button_box.append(&reject_btn);
        button_box.append(&confirm_btn);
        content.append(&button_box);

        dialog.set_child(Some(&content));

        // Connect buttons
        let dialog_clone = dialog.clone();
        reject_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        Self {
            dialog,
            method: PairingMethod::PasskeyConfirmation,
            pin_entry: None,
            passkey_label: Some(passkey_label),
        }
    }

    /// Create a new pairing dialog for Just Works pairing
    pub fn new_just_works(device_name: &str) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Parear Dispositivo")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Icon
        let icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
        icon.set_pixel_size(64);
        content.append(&icon);

        // Title
        let title = gtk4::Label::new(Some(&format!("Parear com {}", device_name)));
        title.add_css_class("title-2");
        content.append(&title);

        // Description
        let desc = gtk4::Label::new(Some("Deseja parear com este dispositivo?"));
        desc.add_css_class("dim-label");
        content.append(&desc);

        // Buttons
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(16);

        let cancel_btn = gtk4::Button::with_label("Cancelar");
        cancel_btn.add_css_class("pill");

        let pair_btn = gtk4::Button::with_label("Parear");
        pair_btn.add_css_class("pill");
        pair_btn.add_css_class("suggested-action");

        button_box.append(&cancel_btn);
        button_box.append(&pair_btn);
        content.append(&button_box);

        dialog.set_child(Some(&content));

        // Connect close on cancel
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        Self {
            dialog,
            method: PairingMethod::JustWorks,
            pin_entry: None,
            passkey_label: None,
        }
    }

    /// Create a pairing progress dialog
    pub fn new_progress(device_name: &str) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Pareando...")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Spinner
        let spinner = gtk4::Spinner::new();
        spinner.set_spinning(true);
        spinner.set_size_request(48, 48);
        content.append(&spinner);

        // Title
        let title = gtk4::Label::new(Some(&format!("Pareando com {}...", device_name)));
        title.add_css_class("title-3");
        content.append(&title);

        // Description
        let desc = gtk4::Label::new(Some("Aguarde enquanto o pareamento e realizado."));
        desc.add_css_class("dim-label");
        content.append(&desc);

        // Cancel button
        let cancel_btn = gtk4::Button::with_label("Cancelar");
        cancel_btn.add_css_class("pill");
        cancel_btn.set_halign(gtk4::Align::Center);
        cancel_btn.set_margin_top(16);
        content.append(&cancel_btn);

        dialog.set_child(Some(&content));

        // Connect close on cancel
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        Self {
            dialog,
            method: PairingMethod::JustWorks,
            pin_entry: None,
            passkey_label: None,
        }
    }

    /// Get the dialog widget
    pub fn widget(&self) -> &adw::Dialog {
        &self.dialog
    }

    /// Present the dialog
    pub fn present(&self, parent: &impl IsA<gtk4::Widget>) {
        self.dialog.present(Some(parent));
    }

    /// Close the dialog
    pub fn close(&self) {
        self.dialog.close();
    }

    /// Get the pairing method
    pub fn method(&self) -> PairingMethod {
        self.method
    }

    /// Get entered PIN (if applicable)
    pub fn get_pin(&self) -> Option<String> {
        self.pin_entry.as_ref().map(|e| e.text().to_string())
    }

    /// Get entered passkey (if applicable)
    pub fn get_passkey(&self) -> Option<u32> {
        self.pin_entry.as_ref().and_then(|e| {
            e.text().parse().ok()
        })
    }

    /// Connect to the pair/confirm button
    pub fn connect_confirm<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        // In a real implementation, we would connect to the appropriate button
        // based on the dialog type
    }

    /// Connect to the cancel button
    pub fn connect_cancel<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        // In a real implementation, we would connect to the cancel button
    }
}

/// Authorization dialog for incoming connections
pub struct AuthorizationDialog {
    dialog: adw::Dialog,
}

impl AuthorizationDialog {
    /// Create a new authorization dialog
    pub fn new(device_name: &str, service: &str) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Solicitacao de Conexao")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Icon
        let icon = gtk4::Image::from_icon_name("dialog-question-symbolic");
        icon.set_pixel_size(64);
        content.append(&icon);

        // Title
        let title = gtk4::Label::new(Some("Permitir Conexao?"));
        title.add_css_class("title-2");
        content.append(&title);

        // Description
        let desc = gtk4::Label::new(Some(&format!(
            "{} deseja usar o servico:\n{}",
            device_name, service
        )));
        desc.add_css_class("dim-label");
        desc.set_wrap(true);
        desc.set_max_width_chars(40);
        content.append(&desc);

        // Buttons
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(16);

        let reject_btn = gtk4::Button::with_label("Recusar");
        reject_btn.add_css_class("pill");
        reject_btn.add_css_class("destructive-action");

        let allow_btn = gtk4::Button::with_label("Permitir");
        allow_btn.add_css_class("pill");
        allow_btn.add_css_class("suggested-action");

        button_box.append(&reject_btn);
        button_box.append(&allow_btn);
        content.append(&button_box);

        dialog.set_child(Some(&content));

        // Connect buttons
        let dialog_clone = dialog.clone();
        reject_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        Self { dialog }
    }

    /// Get the dialog widget
    pub fn widget(&self) -> &adw::Dialog {
        &self.dialog
    }

    /// Present the dialog
    pub fn present(&self, parent: &impl IsA<gtk4::Widget>) {
        self.dialog.present(Some(parent));
    }
}

/// File receive dialog
pub struct FileReceiveDialog {
    dialog: adw::Dialog,
}

impl FileReceiveDialog {
    /// Create a new file receive dialog
    pub fn new(device_name: &str, filename: &str, size: u64) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Receber Arquivo")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Icon
        let icon = gtk4::Image::from_icon_name("folder-download-symbolic");
        icon.set_pixel_size(64);
        content.append(&icon);

        // Title
        let title = gtk4::Label::new(Some("Receber Arquivo?"));
        title.add_css_class("title-2");
        content.append(&title);

        // Description
        let size_str = if size > 1024 * 1024 {
            format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
        } else if size > 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{} bytes", size)
        };

        let desc = gtk4::Label::new(Some(&format!(
            "{} deseja enviar:\n\n{}\n({})",
            device_name, filename, size_str
        )));
        desc.add_css_class("dim-label");
        desc.set_wrap(true);
        desc.set_max_width_chars(40);
        content.append(&desc);

        // Buttons
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(16);

        let reject_btn = gtk4::Button::with_label("Recusar");
        reject_btn.add_css_class("pill");
        reject_btn.add_css_class("destructive-action");

        let accept_btn = gtk4::Button::with_label("Aceitar");
        accept_btn.add_css_class("pill");
        accept_btn.add_css_class("suggested-action");

        button_box.append(&reject_btn);
        button_box.append(&accept_btn);
        content.append(&button_box);

        dialog.set_child(Some(&content));

        // Connect buttons
        let dialog_clone = dialog.clone();
        reject_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        Self { dialog }
    }

    /// Get the dialog widget
    pub fn widget(&self) -> &adw::Dialog {
        &self.dialog
    }

    /// Present the dialog
    pub fn present(&self, parent: &impl IsA<gtk4::Widget>) {
        self.dialog.present(Some(parent));
    }
}
