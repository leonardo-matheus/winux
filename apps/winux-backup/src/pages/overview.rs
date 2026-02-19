//! Overview page - Backup dashboard

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage};

use crate::ui::BackupCard;

/// Overview dashboard page
pub struct OverviewPage {
    widget: gtk4::ScrolledWindow,
}

impl OverviewPage {
    pub fn new() -> Self {
        let content = Box::new(Orientation::Vertical, 0);

        // Status header
        let status = StatusPage::builder()
            .icon_name("drive-harddisk-symbolic")
            .title("Seus Backups")
            .description("Gerencie e monitore seus backups")
            .build();
        content.append(&status);

        let page = PreferencesPage::new();

        // Quick Stats Group
        let stats_group = PreferencesGroup::builder()
            .title("Resumo")
            .build();

        let last_backup_row = ActionRow::builder()
            .title("Ultimo Backup")
            .subtitle("Hoje, 14:30 - Home Folder")
            .build();
        last_backup_row.add_prefix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));
        stats_group.add(&last_backup_row);

        let next_backup_row = ActionRow::builder()
            .title("Proximo Backup Agendado")
            .subtitle("Amanha, 03:00 - Sistema Completo")
            .build();
        next_backup_row.add_prefix(&gtk4::Image::from_icon_name("alarm-symbolic"));
        stats_group.add(&next_backup_row);

        let storage_row = ActionRow::builder()
            .title("Espaco Utilizado")
            .subtitle("45.2 GB de 500 GB")
            .build();
        storage_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));

        let progress = ProgressBar::new();
        progress.set_fraction(0.09);
        progress.set_valign(gtk4::Align::Center);
        progress.set_size_request(120, -1);
        storage_row.add_suffix(&progress);
        stats_group.add(&storage_row);

        page.add(&stats_group);

        // Recent Backups Group
        let recent_group = PreferencesGroup::builder()
            .title("Backups Recentes")
            .description("Clique para ver detalhes ou restaurar")
            .build();

        let backups = [
            ("Home Folder", "Hoje, 14:30", "2.3 GB", "success"),
            ("Configuracoes de Apps", "Hoje, 10:00", "156 MB", "success"),
            ("Sistema Completo", "Ontem, 03:00", "42.7 GB", "success"),
            ("Documentos", "18/02/2026", "1.8 GB", "success"),
        ];

        for (name, date, size, status) in backups {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} - {}", date, size))
                .activatable(true)
                .build();

            let icon = match status {
                "success" => "emblem-ok-symbolic",
                "warning" => "dialog-warning-symbolic",
                "error" => "dialog-error-symbolic",
                _ => "folder-symbolic",
            };
            row.add_prefix(&gtk4::Image::from_icon_name(icon));
            row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

            recent_group.add(&row);
        }

        page.add(&recent_group);

        // Quick Actions Group
        let actions_group = PreferencesGroup::builder()
            .title("Acoes Rapidas")
            .build();

        let quick_backup_row = ActionRow::builder()
            .title("Backup Rapido")
            .subtitle("Fazer backup do Home agora")
            .activatable(true)
            .build();
        quick_backup_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));

        let backup_btn = Button::with_label("Iniciar");
        backup_btn.add_css_class("suggested-action");
        backup_btn.set_valign(gtk4::Align::Center);
        quick_backup_row.add_suffix(&backup_btn);
        actions_group.add(&quick_backup_row);

        let verify_row = ActionRow::builder()
            .title("Verificar Integridade")
            .subtitle("Verificar todos os backups")
            .activatable(true)
            .build();
        verify_row.add_prefix(&gtk4::Image::from_icon_name("emblem-synchronizing-symbolic"));

        let verify_btn = Button::with_label("Verificar");
        verify_btn.add_css_class("flat");
        verify_btn.set_valign(gtk4::Align::Center);
        verify_row.add_suffix(&verify_btn);
        actions_group.add(&verify_row);

        page.add(&actions_group);

        // Backup Health Group
        let health_group = PreferencesGroup::builder()
            .title("Saude dos Backups")
            .build();

        let health_items = [
            ("Backups verificados", "Todos os backups estao integros", "emblem-ok-symbolic", true),
            ("Espaco em disco", "Espaco suficiente disponivel", "emblem-ok-symbolic", true),
            ("Agendamento", "Proximo backup em 12 horas", "emblem-ok-symbolic", true),
            ("Backup remoto", "Sincronizado com servidor", "emblem-ok-symbolic", true),
        ];

        for (title, subtitle, icon, _healthy) in health_items {
            let row = ActionRow::builder()
                .title(title)
                .subtitle(subtitle)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));
            health_group.add(&row);
        }

        page.add(&health_group);

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

impl Default for OverviewPage {
    fn default() -> Self {
        Self::new()
    }
}
