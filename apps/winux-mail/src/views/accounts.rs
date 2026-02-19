// Winux Mail - Accounts View
// Copyright (c) 2026 Winux OS Project

use crate::backend::account::{Account, AccountProvider, AccountSettings};

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Entry, HeaderBar, Label, ListBox, Notebook, Orientation,
    PasswordEntry, ScrolledWindow, Separator, SpinButton, Stack, StackSwitcher,
    Switch, Window,
};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct AccountsView {
    pub window: adw::Window,
    pub accounts_list: ListBox,
    pub stack: Stack,
}

impl AccountsView {
    pub fn new(parent: &impl IsA<Window>) -> Self {
        let window = adw::Window::builder()
            .title("Email Accounts")
            .default_width(800)
            .default_height(600)
            .transient_for(parent)
            .modal(true)
            .build();

        let header = HeaderBar::new();

        let add_btn = Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Add Account")
            .build();

        header.pack_start(&add_btn);

        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        main_box.append(&header);

        // Split view: accounts list | account details
        let content_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .build();

        // Accounts list (left side)
        let list_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .width_request(250)
            .build();

        let list_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .build();

        let accounts_list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(vec!["navigation-sidebar"])
            .build();

        // Placeholder
        let placeholder = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .spacing(12)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let icon = gtk4::Image::builder()
            .icon_name("mail-symbolic")
            .pixel_size(48)
            .css_classes(vec!["dim-label"])
            .build();

        let label = Label::builder()
            .label("No accounts")
            .css_classes(vec!["dim-label"])
            .build();

        let sublabel = Label::builder()
            .label("Add an account to get started")
            .css_classes(vec!["dim-label", "caption"])
            .build();

        placeholder.append(&icon);
        placeholder.append(&label);
        placeholder.append(&sublabel);

        accounts_list.set_placeholder(Some(&placeholder));

        list_scroll.set_child(Some(&accounts_list));
        list_box.append(&list_scroll);

        content_box.append(&list_box);
        content_box.append(&Separator::new(Orientation::Vertical));

        // Stack for different views
        let stack = Stack::builder()
            .hexpand(true)
            .build();

        // Empty state
        let empty_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .spacing(12)
            .build();

        let empty_icon = gtk4::Image::builder()
            .icon_name("applications-email-symbolic")
            .pixel_size(64)
            .css_classes(vec!["dim-label"])
            .build();

        let empty_label = Label::builder()
            .label("Select an account or add a new one")
            .css_classes(vec!["title-2", "dim-label"])
            .build();

        empty_box.append(&empty_icon);
        empty_box.append(&empty_label);

        stack.add_named(&empty_box, Some("empty"));

        // Add account view
        let add_account_view = Self::create_add_account_view();
        stack.add_named(&add_account_view, Some("add"));

        // Account details view (template)
        let details_view = Self::create_account_details_view();
        stack.add_named(&details_view, Some("details"));

        stack.set_visible_child_name("empty");

        content_box.append(&stack);
        main_box.append(&content_box);

        window.set_content(Some(&main_box));

        let view = Self {
            window,
            accounts_list,
            stack,
        };

        view.setup_signals(add_btn);
        view.load_accounts();

        view
    }

    fn create_add_account_view() -> GtkBox {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .spacing(16)
            .build();

        let title = Label::builder()
            .label("Add Email Account")
            .css_classes(vec!["title-1"])
            .halign(gtk4::Align::Start)
            .build();

        container.append(&title);

        // Provider selection
        let providers_label = Label::builder()
            .label("Choose your email provider")
            .css_classes(vec!["heading"])
            .halign(gtk4::Align::Start)
            .margin_top(8)
            .build();

        container.append(&providers_label);

        let providers_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(8)
            .build();

        let providers = vec![
            ("Gmail", "gmail-symbolic", AccountProvider::Gmail),
            ("Outlook", "outlook-symbolic", AccountProvider::Outlook),
            ("Yahoo", "yahoo-symbolic", AccountProvider::Yahoo),
            ("iCloud", "icloud-symbolic", AccountProvider::ICloud),
            ("Other", "mail-symbolic", AccountProvider::Custom),
        ];

        for (name, icon, _provider) in providers {
            let btn = Button::builder()
                .css_classes(vec!["card"])
                .build();

            let content = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(8)
                .margin_start(24)
                .margin_end(24)
                .margin_top(16)
                .margin_bottom(16)
                .build();

            let img = gtk4::Image::builder()
                .icon_name(icon)
                .pixel_size(48)
                .build();

            let lbl = Label::builder()
                .label(name)
                .build();

            content.append(&img);
            content.append(&lbl);

            btn.set_child(Some(&content));
            providers_box.append(&btn);
        }

        container.append(&providers_box);

        // Manual configuration
        let manual_label = Label::builder()
            .label("Or enter your account details")
            .css_classes(vec!["heading"])
            .halign(gtk4::Align::Start)
            .margin_top(24)
            .build();

        container.append(&manual_label);

        // Account info
        let form_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_top(8)
            .build();

        // Name
        let name_entry = adw::EntryRow::builder()
            .title("Name")
            .build();

        // Email
        let email_entry = adw::EntryRow::builder()
            .title("Email Address")
            .build();

        // Password
        let password_entry = adw::PasswordEntryRow::builder()
            .title("Password")
            .build();

        let basic_group = adw::PreferencesGroup::builder()
            .title("Account Information")
            .build();

        basic_group.add(&name_entry);
        basic_group.add(&email_entry);
        basic_group.add(&password_entry);

        form_box.append(&basic_group);

        // Server settings (collapsed by default)
        let server_expander = adw::ExpanderRow::builder()
            .title("Server Settings")
            .subtitle("Configure IMAP and SMTP servers manually")
            .build();

        // IMAP settings
        let imap_server = adw::EntryRow::builder()
            .title("IMAP Server")
            .build();

        let imap_port = adw::SpinRow::builder()
            .title("IMAP Port")
            .adjustment(&gtk4::Adjustment::new(993.0, 1.0, 65535.0, 1.0, 10.0, 0.0))
            .build();

        let imap_ssl = adw::SwitchRow::builder()
            .title("Use SSL/TLS")
            .active(true)
            .build();

        server_expander.add_row(&imap_server);
        server_expander.add_row(&imap_port);
        server_expander.add_row(&imap_ssl);

        // SMTP settings
        let smtp_server = adw::EntryRow::builder()
            .title("SMTP Server")
            .build();

        let smtp_port = adw::SpinRow::builder()
            .title("SMTP Port")
            .adjustment(&gtk4::Adjustment::new(587.0, 1.0, 65535.0, 1.0, 10.0, 0.0))
            .build();

        let smtp_ssl = adw::SwitchRow::builder()
            .title("Use STARTTLS")
            .active(true)
            .build();

        server_expander.add_row(&smtp_server);
        server_expander.add_row(&smtp_port);
        server_expander.add_row(&smtp_ssl);

        let server_group = adw::PreferencesGroup::new();
        server_group.add(&server_expander);

        form_box.append(&server_group);

        container.append(&form_box);

        // Buttons
        let buttons_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::End)
            .margin_top(16)
            .build();

        let cancel_btn = Button::builder()
            .label("Cancel")
            .build();

        let add_btn = Button::builder()
            .label("Add Account")
            .css_classes(vec!["suggested-action"])
            .build();

        buttons_box.append(&cancel_btn);
        buttons_box.append(&add_btn);

        container.append(&buttons_box);

        container
    }

    fn create_account_details_view() -> GtkBox {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .spacing(16)
            .build();

        let title = Label::builder()
            .label("Account Settings")
            .css_classes(vec!["title-1"])
            .halign(gtk4::Align::Start)
            .build();

        container.append(&title);

        // Account info
        let info_group = adw::PreferencesGroup::builder()
            .title("Account Information")
            .build();

        let name_row = adw::EntryRow::builder()
            .title("Display Name")
            .text("John Doe")
            .build();

        let email_row = adw::ActionRow::builder()
            .title("Email Address")
            .subtitle("john.doe@example.com")
            .build();

        info_group.add(&name_row);
        info_group.add(&email_row);

        container.append(&info_group);

        // Sync settings
        let sync_group = adw::PreferencesGroup::builder()
            .title("Synchronization")
            .build();

        let sync_enabled = adw::SwitchRow::builder()
            .title("Enable Sync")
            .active(true)
            .build();

        let sync_interval = adw::ComboRow::builder()
            .title("Sync Interval")
            .subtitle("How often to check for new emails")
            .build();

        let intervals = gtk4::StringList::new(&["Every 5 minutes", "Every 15 minutes", "Every 30 minutes", "Every hour", "Manual"]);
        sync_interval.set_model(Some(&intervals));

        let sync_days = adw::SpinRow::builder()
            .title("Sync Period (days)")
            .subtitle("Number of days to sync")
            .adjustment(&gtk4::Adjustment::new(30.0, 1.0, 365.0, 1.0, 10.0, 0.0))
            .build();

        sync_group.add(&sync_enabled);
        sync_group.add(&sync_interval);
        sync_group.add(&sync_days);

        container.append(&sync_group);

        // Notifications
        let notif_group = adw::PreferencesGroup::builder()
            .title("Notifications")
            .build();

        let notif_enabled = adw::SwitchRow::builder()
            .title("Enable Notifications")
            .active(true)
            .build();

        let notif_sound = adw::SwitchRow::builder()
            .title("Sound")
            .active(true)
            .build();

        notif_group.add(&notif_enabled);
        notif_group.add(&notif_sound);

        container.append(&notif_group);

        // Signature
        let signature_group = adw::PreferencesGroup::builder()
            .title("Signature")
            .build();

        let signature_enabled = adw::SwitchRow::builder()
            .title("Use Signature")
            .active(true)
            .build();

        let signature_text = adw::EntryRow::builder()
            .title("Signature")
            .text("Sent from Winux Mail")
            .build();

        signature_group.add(&signature_enabled);
        signature_group.add(&signature_text);

        container.append(&signature_group);

        // Danger zone
        let danger_group = adw::PreferencesGroup::builder()
            .title("Danger Zone")
            .build();

        let remove_btn = Button::builder()
            .label("Remove Account")
            .css_classes(vec!["destructive-action"])
            .halign(gtk4::Align::Start)
            .margin_top(8)
            .build();

        danger_group.add(&remove_btn);

        container.append(&danger_group);

        container
    }

    fn setup_signals(&self, add_btn: Button) {
        let stack = self.stack.clone();
        add_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("add");
        });

        let stack = self.stack.clone();
        self.accounts_list.connect_row_selected(move |_, row| {
            if row.is_some() {
                stack.set_visible_child_name("details");
            } else {
                stack.set_visible_child_name("empty");
            }
        });
    }

    fn load_accounts(&self) {
        // TODO: Load accounts from account manager
    }

    fn add_account_row(&self, account: &Account) {
        let row = adw::ActionRow::builder()
            .title(&account.name)
            .subtitle(&account.email)
            .build();

        let avatar = adw::Avatar::builder()
            .size(32)
            .text(&account.name)
            .show_initials(true)
            .build();

        row.add_prefix(&avatar);

        // Status indicator
        let status = gtk4::Image::builder()
            .icon_name("emblem-ok-symbolic")
            .css_classes(vec!["success"])
            .build();

        row.add_suffix(&status);

        self.accounts_list.append(&row);
    }

    pub fn present(&self) {
        self.window.present();
    }
}
