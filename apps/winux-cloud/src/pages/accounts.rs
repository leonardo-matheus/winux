//! Accounts page - Connected cloud accounts

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, ListBox, SelectionMode};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage};

use crate::ui::AccountRow;

/// Accounts management page
pub struct AccountsPage {
    widget: gtk4::ScrolledWindow,
}

impl AccountsPage {
    pub fn new() -> Self {
        let content = Box::new(Orientation::Vertical, 0);

        // Status header
        let status = StatusPage::builder()
            .icon_name("cloud-symbolic")
            .title("Contas na Nuvem")
            .description("Conecte e gerencie suas contas de armazenamento")
            .build();
        content.append(&status);

        let page = PreferencesPage::new();

        // Connected Accounts Group
        let connected_group = PreferencesGroup::builder()
            .title("Contas Conectadas")
            .description("Suas contas de armazenamento em nuvem")
            .build();

        let accounts = [
            ("Google Drive", "leonardo.silva@gmail.com", "15 GB usados de 100 GB", "google-drive", true),
            ("OneDrive", "leonardo@outlook.com", "8.5 GB usados de 1 TB", "onedrive", true),
            ("Dropbox", "leonardo@dropbox.com", "2.1 GB usados de 2 GB", "dropbox", false),
        ];

        for (provider, email, storage, icon, syncing) in accounts {
            let row = ActionRow::builder()
                .title(provider)
                .subtitle(&format!("{}\n{}", email, storage))
                .activatable(true)
                .build();

            // Provider icon
            let icon_widget = gtk4::Image::from_icon_name(icon);
            icon_widget.set_pixel_size(32);
            row.add_prefix(&icon_widget);

            // Sync status indicator
            let status_icon = if syncing {
                gtk4::Image::from_icon_name("emblem-synchronizing-symbolic")
            } else {
                gtk4::Image::from_icon_name("emblem-ok-symbolic")
            };
            status_icon.set_tooltip_text(Some(if syncing { "Sincronizando..." } else { "Sincronizado" }));
            row.add_suffix(&status_icon);

            // Settings button
            let settings_btn = Button::from_icon_name("emblem-system-symbolic");
            settings_btn.add_css_class("flat");
            settings_btn.set_valign(gtk4::Align::Center);
            settings_btn.set_tooltip_text(Some("Configuracoes da conta"));
            row.add_suffix(&settings_btn);

            connected_group.add(&row);
        }

        page.add(&connected_group);

        // Add New Account Group
        let add_group = PreferencesGroup::builder()
            .title("Adicionar Conta")
            .description("Conecte um novo servico de armazenamento")
            .build();

        let providers = [
            ("Google Drive", "google-drive", "OAuth2 - Armazenamento Google"),
            ("OneDrive", "onedrive", "OAuth2 - Armazenamento Microsoft"),
            ("Dropbox", "dropbox", "OAuth2 - Sincronizacao Dropbox"),
            ("Nextcloud", "network-server-symbolic", "WebDAV - Self-hosted"),
            ("WebDAV Generico", "network-workgroup-symbolic", "WebDAV - Qualquer servidor"),
            ("S3 Compatible", "network-server-symbolic", "AWS S3, MinIO, DigitalOcean Spaces"),
        ];

        for (name, icon, description) in providers {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(description)
                .activatable(true)
                .build();

            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            let connect_btn = Button::with_label("Conectar");
            connect_btn.add_css_class("suggested-action");
            connect_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&connect_btn);

            add_group.add(&row);
        }

        page.add(&add_group);

        // Storage Overview Group
        let storage_group = PreferencesGroup::builder()
            .title("Visao Geral do Armazenamento")
            .build();

        let total_row = ActionRow::builder()
            .title("Espaco Total Disponivel")
            .subtitle("1.1 TB combinados")
            .build();
        total_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));
        storage_group.add(&total_row);

        let used_row = ActionRow::builder()
            .title("Espaco Utilizado")
            .subtitle("25.6 GB (2.3%)")
            .build();
        used_row.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));

        let progress = gtk4::ProgressBar::new();
        progress.set_fraction(0.023);
        progress.set_valign(gtk4::Align::Center);
        progress.set_size_request(120, -1);
        used_row.add_suffix(&progress);
        storage_group.add(&used_row);

        let files_row = ActionRow::builder()
            .title("Arquivos Sincronizados")
            .subtitle("12,456 arquivos em 3 contas")
            .build();
        files_row.add_prefix(&gtk4::Image::from_icon_name("document-open-symbolic"));
        storage_group.add(&files_row);

        page.add(&storage_group);

        content.append(&page);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&content)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}

impl Default for AccountsPage {
    fn default() -> Self {
        Self::new()
    }
}
