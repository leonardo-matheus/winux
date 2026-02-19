//! Password dialog for WiFi connections
//!
//! Modal dialog for entering WiFi passwords

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Image, Label, Orientation, Window};
use libadwaita as adw;
use adw::prelude::*;
use adw::{MessageDialog, ResponseAppearance};
use std::cell::RefCell;
use std::rc::Rc;

/// Password dialog for WiFi authentication
pub struct PasswordDialog;

impl PasswordDialog {
    /// Show password dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window
    /// * `ssid` - Network SSID
    /// * `callback` - Called with password when user connects
    pub fn show<F>(parent: &Window, ssid: &str, callback: F)
    where
        F: Fn(String) + 'static,
    {
        let dialog = MessageDialog::builder()
            .heading(&format!("Conectar a {}", ssid))
            .body("Digite a senha da rede Wi-Fi")
            .transient_for(parent)
            .modal(true)
            .build();

        // Create password entry
        let entry_box = Box::new(Orientation::Vertical, 12);
        entry_box.set_margin_start(24);
        entry_box.set_margin_end(24);

        let password_entry = adw::PasswordEntryRow::builder()
            .title("Senha")
            .build();

        let list = gtk4::ListBox::new();
        list.add_css_class("boxed-list");
        list.append(&password_entry);

        entry_box.append(&list);

        // Show password checkbox
        let show_password_box = Box::new(Orientation::Horizontal, 8);
        show_password_box.set_margin_top(8);

        let show_check = gtk4::CheckButton::with_label("Mostrar senha");
        show_check.connect_toggled({
            let entry = password_entry.clone();
            move |check| {
                // Note: PasswordEntryRow doesn't have direct visibility toggle
                // This would need custom implementation
            }
        });
        show_password_box.append(&show_check);

        entry_box.append(&show_password_box);

        dialog.set_extra_child(Some(&entry_box));

        // Add responses
        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("connect", "Conectar");

        dialog.set_response_appearance("connect", ResponseAppearance::Suggested);
        dialog.set_default_response(Some("connect"));
        dialog.set_close_response("cancel");

        // Handle response
        let password_entry_clone = password_entry.clone();
        dialog.connect_response(None, move |dialog, response| {
            if response == "connect" {
                let password = password_entry_clone.text().to_string();
                if !password.is_empty() {
                    callback(password);
                }
            }
            dialog.close();
        });

        // Enable connect button only when password is entered
        let dialog_weak = dialog.downgrade();
        password_entry.connect_changed(move |entry| {
            if let Some(dialog) = dialog_weak.upgrade() {
                let has_text = !entry.text().is_empty();
                dialog.set_response_enabled("connect", has_text);
            }
        });

        // Initially disable connect button
        dialog.set_response_enabled("connect", false);

        dialog.present();
    }

    /// Show password dialog for WPA Enterprise
    pub fn show_enterprise<F>(parent: &Window, ssid: &str, callback: F)
    where
        F: Fn(String, String) + 'static,
    {
        let dialog = MessageDialog::builder()
            .heading(&format!("Conectar a {}", ssid))
            .body("Esta rede requer autenticacao empresarial")
            .transient_for(parent)
            .modal(true)
            .build();

        let entry_box = Box::new(Orientation::Vertical, 12);
        entry_box.set_margin_start(24);
        entry_box.set_margin_end(24);

        let list = gtk4::ListBox::new();
        list.add_css_class("boxed-list");

        // Username entry
        let username_entry = adw::EntryRow::builder()
            .title("Usuario")
            .build();
        list.append(&username_entry);

        // Password entry
        let password_entry = adw::PasswordEntryRow::builder()
            .title("Senha")
            .build();
        list.append(&password_entry);

        entry_box.append(&list);
        dialog.set_extra_child(Some(&entry_box));

        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("connect", "Conectar");

        dialog.set_response_appearance("connect", ResponseAppearance::Suggested);
        dialog.set_default_response(Some("connect"));

        let username_clone = username_entry.clone();
        let password_clone = password_entry.clone();

        dialog.connect_response(None, move |dialog, response| {
            if response == "connect" {
                let username = username_clone.text().to_string();
                let password = password_clone.text().to_string();
                callback(username, password);
            }
            dialog.close();
        });

        dialog.present();
    }

    /// Show confirmation dialog for forgetting a network
    pub fn confirm_forget<F>(parent: &Window, ssid: &str, callback: F)
    where
        F: Fn() + 'static,
    {
        let dialog = MessageDialog::builder()
            .heading("Esquecer Rede")
            .body(&format!(
                "Tem certeza que deseja esquecer a rede \"{}\"?\n\nVoce precisara digitar a senha novamente para conectar.",
                ssid
            ))
            .transient_for(parent)
            .modal(true)
            .build();

        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("forget", "Esquecer");

        dialog.set_response_appearance("forget", ResponseAppearance::Destructive);

        dialog.connect_response(None, move |dialog, response| {
            if response == "forget" {
                callback();
            }
            dialog.close();
        });

        dialog.present();
    }

    /// Show error dialog
    pub fn show_error(parent: &Window, title: &str, message: &str) {
        let dialog = MessageDialog::builder()
            .heading(title)
            .body(message)
            .transient_for(parent)
            .modal(true)
            .build();

        dialog.add_response("ok", "OK");
        dialog.set_default_response(Some("ok"));

        dialog.connect_response(None, |dialog, _| {
            dialog.close();
        });

        dialog.present();
    }

    /// Show connection progress dialog
    pub fn show_connecting(parent: &Window, ssid: &str) -> MessageDialog {
        let dialog = MessageDialog::builder()
            .heading(&format!("Conectando a {}...", ssid))
            .transient_for(parent)
            .modal(true)
            .build();

        let spinner = gtk4::Spinner::new();
        spinner.start();
        spinner.set_size_request(32, 32);

        let spinner_box = Box::new(Orientation::Vertical, 12);
        spinner_box.set_halign(gtk4::Align::Center);
        spinner_box.append(&spinner);
        spinner_box.append(&Label::new(Some("Aguarde...")));

        dialog.set_extra_child(Some(&spinner_box));

        dialog.add_response("cancel", "Cancelar");

        dialog.present();
        dialog
    }
}

/// Dialog for adding a hidden network
pub struct HiddenNetworkDialog;

impl HiddenNetworkDialog {
    pub fn show<F>(parent: &Window, callback: F)
    where
        F: Fn(String, String, bool) + 'static,
    {
        let dialog = MessageDialog::builder()
            .heading("Conectar a Rede Oculta")
            .body("Digite as informacoes da rede")
            .transient_for(parent)
            .modal(true)
            .build();

        let entry_box = Box::new(Orientation::Vertical, 12);
        entry_box.set_margin_start(24);
        entry_box.set_margin_end(24);

        let list = gtk4::ListBox::new();
        list.add_css_class("boxed-list");

        // SSID entry
        let ssid_entry = adw::EntryRow::builder()
            .title("Nome da Rede (SSID)")
            .build();
        list.append(&ssid_entry);

        // Security combo
        let security_row = adw::ComboRow::builder()
            .title("Seguranca")
            .build();
        let security_model = gtk4::StringList::new(&["Nenhuma", "WPA/WPA2/WPA3"]);
        security_row.set_model(Some(&security_model));
        security_row.set_selected(1);
        list.append(&security_row);

        // Password entry
        let password_entry = adw::PasswordEntryRow::builder()
            .title("Senha")
            .build();
        list.append(&password_entry);

        entry_box.append(&list);
        dialog.set_extra_child(Some(&entry_box));

        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("connect", "Conectar");

        dialog.set_response_appearance("connect", ResponseAppearance::Suggested);

        let ssid_clone = ssid_entry.clone();
        let password_clone = password_entry.clone();
        let security_clone = security_row.clone();

        dialog.connect_response(None, move |dialog, response| {
            if response == "connect" {
                let ssid = ssid_clone.text().to_string();
                let password = password_clone.text().to_string();
                let secured = security_clone.selected() > 0;
                callback(ssid, password, secured);
            }
            dialog.close();
        });

        dialog.present();
    }
}
