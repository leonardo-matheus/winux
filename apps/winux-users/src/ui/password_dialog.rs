//! Password change dialog

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, PasswordEntry, Window};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

/// Password change dialog
pub struct PasswordDialog {
    dialog: adw::Dialog,
}

impl PasswordDialog {
    /// Create a new password dialog
    pub fn new(username: &str) -> Self {
        let dialog = adw::Dialog::builder()
            .title("Alterar Senha")
            .build();

        let content = Box::new(Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();
        header.set_show_end_title_buttons(false);
        header.set_show_start_title_buttons(false);

        let cancel_btn = Button::with_label("Cancelar");
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });
        header.pack_start(&cancel_btn);

        let change_btn = Button::with_label("Alterar");
        change_btn.add_css_class("suggested-action");
        change_btn.set_sensitive(false); // Enable when passwords match
        header.pack_end(&change_btn);

        content.append(&header);

        // Content
        let page = adw::PreferencesPage::new();

        // User info
        let info_group = adw::PreferencesGroup::new();

        let user_row = adw::ActionRow::builder()
            .title("Usuario")
            .subtitle(username)
            .build();

        let avatar = adw::Avatar::new(48, Some(username), true);
        user_row.add_prefix(&avatar);
        info_group.add(&user_row);

        page.add(&info_group);

        // Password fields
        let password_group = adw::PreferencesGroup::builder()
            .title("Nova Senha")
            .description("Digite uma senha forte com pelo menos 8 caracteres")
            .build();

        // Current password (for non-admin users changing their own password)
        let current_entry = adw::PasswordEntryRow::builder()
            .title("Senha Atual")
            .build();
        password_group.add(&current_entry);

        // New password
        let new_entry = adw::PasswordEntryRow::builder()
            .title("Nova Senha")
            .build();
        password_group.add(&new_entry);

        // Confirm password
        let confirm_entry = adw::PasswordEntryRow::builder()
            .title("Confirmar Senha")
            .build();
        password_group.add(&confirm_entry);

        // Password strength indicator
        let strength_row = adw::ActionRow::builder()
            .title("Forca da Senha")
            .build();

        let strength_bar = gtk4::LevelBar::new();
        strength_bar.set_min_value(0.0);
        strength_bar.set_max_value(4.0);
        strength_bar.set_value(0.0);
        strength_bar.set_hexpand(true);
        strength_bar.set_valign(gtk4::Align::Center);
        strength_bar.add_offset_value("low", 1.0);
        strength_bar.add_offset_value("medium", 2.0);
        strength_bar.add_offset_value("high", 3.0);
        strength_bar.add_offset_value("full", 4.0);
        strength_row.add_suffix(&strength_bar);

        password_group.add(&strength_row);

        page.add(&password_group);

        // Password requirements info
        let requirements_group = adw::PreferencesGroup::builder()
            .title("Requisitos de Senha")
            .build();

        let requirements = [
            ("Pelo menos 8 caracteres", "length"),
            ("Letras maiusculas e minusculas", "case"),
            ("Pelo menos um numero", "number"),
            ("Pelo menos um caractere especial", "special"),
        ];

        let requirement_rows: Vec<(adw::ActionRow, &str)> = requirements
            .iter()
            .map(|(text, id)| {
                let row = adw::ActionRow::builder()
                    .title(*text)
                    .build();

                let icon = gtk4::Image::from_icon_name("emblem-important-symbolic");
                icon.add_css_class("dim-label");
                row.add_prefix(&icon);

                requirements_group.add(&row);
                (row, *id)
            })
            .collect();

        page.add(&requirements_group);

        // Password validation
        let strength_bar_clone = strength_bar.clone();
        let change_btn_clone = change_btn.clone();
        let confirm_entry_clone = confirm_entry.clone();

        new_entry.connect_changed(move |entry| {
            let password = entry.text().to_string();
            let strength = Self::calculate_password_strength(&password);
            strength_bar_clone.set_value(strength as f64);

            // Check if passwords match
            let confirm = confirm_entry_clone.text().to_string();
            let passwords_match = !password.is_empty() && password == confirm;
            change_btn_clone.set_sensitive(passwords_match && strength >= 2);
        });

        let change_btn_clone2 = change_btn.clone();
        let new_entry_clone = new_entry.clone();

        confirm_entry.connect_changed(move |entry| {
            let confirm = entry.text().to_string();
            let password = new_entry_clone.text().to_string();
            let strength = Self::calculate_password_strength(&password);
            let passwords_match = !password.is_empty() && password == confirm;
            change_btn_clone2.set_sensitive(passwords_match && strength >= 2);
        });

        content.append(&page);

        dialog.set_child(Some(&content));
        dialog.set_content_width(450);
        dialog.set_content_height(600);

        PasswordDialog { dialog }
    }

    /// Show the dialog
    pub fn show<F>(parent: &Window, username: &str, on_change: F)
    where
        F: Fn(&str, &str) + 'static,
    {
        let dialog = Self::new(username);
        dialog.dialog.present(Some(parent));
    }

    /// Show password dialog for a user
    pub fn show_for_user<F>(parent: &Window, username: &str, callback: F)
    where
        F: Fn(&str, &str) + 'static,
    {
        Self::show(parent, username, callback);
    }

    /// Calculate password strength (0-4)
    fn calculate_password_strength(password: &str) -> u8 {
        if password.is_empty() {
            return 0;
        }

        let mut strength = 0u8;

        // Length check
        if password.len() >= 8 {
            strength += 1;
        }

        // Uppercase and lowercase
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        if has_upper && has_lower {
            strength += 1;
        }

        // Numbers
        if password.chars().any(|c| c.is_numeric()) {
            strength += 1;
        }

        // Special characters
        let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        if password.chars().any(|c| special_chars.contains(c)) {
            strength += 1;
        }

        strength
    }

    /// Validate password requirements
    pub fn validate_password(password: &str) -> PasswordValidation {
        PasswordValidation {
            has_minimum_length: password.len() >= 8,
            has_uppercase: password.chars().any(|c| c.is_uppercase()),
            has_lowercase: password.chars().any(|c| c.is_lowercase()),
            has_number: password.chars().any(|c| c.is_numeric()),
            has_special: {
                let special = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
                password.chars().any(|c| special.contains(c))
            },
        }
    }
}

/// Password validation result
#[derive(Debug, Clone)]
pub struct PasswordValidation {
    pub has_minimum_length: bool,
    pub has_uppercase: bool,
    pub has_lowercase: bool,
    pub has_number: bool,
    pub has_special: bool,
}

impl PasswordValidation {
    /// Check if password meets minimum requirements
    pub fn is_valid(&self) -> bool {
        self.has_minimum_length && (self.has_uppercase || self.has_lowercase)
    }

    /// Check if password is strong
    pub fn is_strong(&self) -> bool {
        self.has_minimum_length
            && self.has_uppercase
            && self.has_lowercase
            && self.has_number
            && self.has_special
    }

    /// Get strength score (0-4)
    pub fn strength_score(&self) -> u8 {
        let mut score = 0u8;

        if self.has_minimum_length {
            score += 1;
        }
        if self.has_uppercase && self.has_lowercase {
            score += 1;
        }
        if self.has_number {
            score += 1;
        }
        if self.has_special {
            score += 1;
        }

        score
    }
}

/// Create user dialog for adding new users
pub struct CreateUserDialog {
    dialog: adw::Dialog,
}

impl CreateUserDialog {
    pub fn new() -> Self {
        let dialog = adw::Dialog::builder()
            .title("Adicionar Usuario")
            .build();

        let content = Box::new(Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();
        header.set_show_end_title_buttons(false);
        header.set_show_start_title_buttons(false);

        let cancel_btn = Button::with_label("Cancelar");
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });
        header.pack_start(&cancel_btn);

        let create_btn = Button::with_label("Criar");
        create_btn.add_css_class("suggested-action");
        create_btn.set_sensitive(false);
        header.pack_end(&create_btn);

        content.append(&header);

        // Form
        let page = adw::PreferencesPage::new();

        // User info group
        let info_group = adw::PreferencesGroup::builder()
            .title("Informacoes do Usuario")
            .build();

        let avatar_row = adw::ActionRow::builder()
            .title("Avatar")
            .subtitle("Clique para escolher")
            .activatable(true)
            .build();
        let avatar = adw::Avatar::new(64, Some("Novo Usuario"), true);
        avatar_row.add_prefix(&avatar);
        avatar_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        info_group.add(&avatar_row);

        let fullname_entry = adw::EntryRow::builder()
            .title("Nome Completo")
            .build();
        info_group.add(&fullname_entry);

        let username_entry = adw::EntryRow::builder()
            .title("Nome de Usuario")
            .build();
        info_group.add(&username_entry);

        // Auto-generate username from full name
        let username_entry_clone = username_entry.clone();
        fullname_entry.connect_changed(move |entry| {
            let fullname = entry.text().to_string();
            let suggested = Self::suggest_username(&fullname);
            username_entry_clone.set_text(&suggested);
        });

        page.add(&info_group);

        // Account type group
        let type_group = adw::PreferencesGroup::builder()
            .title("Tipo de Conta")
            .build();

        let account_type = adw::ComboRow::builder()
            .title("Tipo")
            .subtitle("Nivel de acesso do usuario")
            .build();
        let types = gtk4::StringList::new(&["Usuario Padrao", "Administrador"]);
        account_type.set_model(Some(&types));
        type_group.add(&account_type);

        page.add(&type_group);

        // Password group
        let password_group = adw::PreferencesGroup::builder()
            .title("Senha")
            .build();

        let password_entry = adw::PasswordEntryRow::builder()
            .title("Senha")
            .build();
        password_group.add(&password_entry);

        let confirm_entry = adw::PasswordEntryRow::builder()
            .title("Confirmar Senha")
            .build();
        password_group.add(&confirm_entry);

        let set_on_login = adw::SwitchRow::builder()
            .title("Definir senha no primeiro login")
            .subtitle("O usuario devera criar uma senha ao fazer login")
            .active(false)
            .build();
        password_group.add(&set_on_login);

        // Toggle password fields visibility based on "set on login"
        let password_entry_clone = password_entry.clone();
        let confirm_entry_clone = confirm_entry.clone();
        set_on_login.connect_active_notify(move |switch| {
            let disabled = switch.is_active();
            password_entry_clone.set_sensitive(!disabled);
            confirm_entry_clone.set_sensitive(!disabled);
        });

        page.add(&password_group);

        // Login options group
        let login_group = adw::PreferencesGroup::builder()
            .title("Opcoes de Login")
            .build();

        let auto_login = adw::SwitchRow::builder()
            .title("Login automatico")
            .subtitle("Fazer login automaticamente ao iniciar")
            .active(false)
            .build();
        login_group.add(&auto_login);

        page.add(&login_group);

        content.append(&page);

        // Validation
        let create_btn_clone = create_btn.clone();
        let username_entry_clone2 = username_entry.clone();
        let password_entry_clone2 = password_entry.clone();
        let confirm_entry_clone2 = confirm_entry.clone();
        let set_on_login_clone = set_on_login.clone();

        let validate = move || {
            let username = username_entry_clone2.text();
            let password = password_entry_clone2.text();
            let confirm = confirm_entry_clone2.text();
            let skip_password = set_on_login_clone.is_active();

            let valid = !username.is_empty()
                && (skip_password || (!password.is_empty() && password == confirm));

            create_btn_clone.set_sensitive(valid);
        };

        let validate_clone = validate.clone();
        username_entry.connect_changed(move |_| validate_clone());

        let validate_clone = validate.clone();
        password_entry.connect_changed(move |_| validate_clone());

        let validate_clone = validate.clone();
        confirm_entry.connect_changed(move |_| validate_clone());

        set_on_login.connect_active_notify(move |_| validate());

        dialog.set_child(Some(&content));
        dialog.set_content_width(450);
        dialog.set_content_height(650);

        CreateUserDialog { dialog }
    }

    pub fn show(&self, parent: &Window) {
        self.dialog.present(Some(parent));
    }

    fn suggest_username(fullname: &str) -> String {
        fullname
            .to_lowercase()
            .split_whitespace()
            .next()
            .unwrap_or("")
            .chars()
            .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            .collect()
    }
}

impl Default for CreateUserDialog {
    fn default() -> Self {
        Self::new()
    }
}
