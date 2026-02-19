//! History page - View update history and rollback

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, Calendar};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage, ExpanderRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::UpdateManager;

pub struct HistoryPage {
    widget: ScrolledWindow,
    update_manager: Rc<RefCell<UpdateManager>>,
}

impl HistoryPage {
    pub fn new(update_manager: Rc<RefCell<UpdateManager>>) -> Self {
        let page = PreferencesPage::new();

        // Statistics summary
        let stats_group = PreferencesGroup::builder()
            .title("Estatisticas")
            .build();

        let total_row = ActionRow::builder()
            .title("Total de Atualizacoes")
            .subtitle("desde a instalacao do sistema")
            .build();
        total_row.add_prefix(&gtk4::Image::from_icon_name("view-list-symbolic"));
        let total_label = Label::new(Some("247"));
        total_label.add_css_class("title-2");
        total_label.set_valign(gtk4::Align::Center);
        total_row.add_suffix(&total_label);
        stats_group.add(&total_row);

        let security_row = ActionRow::builder()
            .title("Atualizacoes de Seguranca")
            .build();
        security_row.add_prefix(&gtk4::Image::from_icon_name("security-high-symbolic"));
        let security_label = Label::new(Some("45"));
        security_label.add_css_class("title-2");
        security_label.set_valign(gtk4::Align::Center);
        security_row.add_suffix(&security_label);
        stats_group.add(&security_row);

        let last_update_row = ActionRow::builder()
            .title("Ultima Atualizacao")
            .subtitle("19 de Fevereiro de 2026, 14:35")
            .build();
        last_update_row.add_prefix(&gtk4::Image::from_icon_name("x-office-calendar-symbolic"));
        stats_group.add(&last_update_row);

        page.add(&stats_group);

        // Recent updates
        let recent_group = PreferencesGroup::builder()
            .title("Atualizacoes Recentes")
            .description("Ultimas 30 dias")
            .build();

        // Today
        let today_expander = ExpanderRow::builder()
            .title("Hoje - 19 de Fevereiro")
            .subtitle("5 pacotes atualizados")
            .build();
        today_expander.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));

        let today_updates = vec![
            ("firefox", "130.0 -> 131.0", "14:35", true),
            ("libc6", "2.38-10 -> 2.38-12", "14:35", true),
            ("openssl", "3.0.13-1 -> 3.0.14-1", "14:35", true),
            ("curl", "8.5.0-1 -> 8.5.0-2", "14:35", false),
            ("wget", "1.21.4-1 -> 1.21.4-2", "14:35", false),
        ];

        for (name, version, time, is_security) in today_updates {
            let row = Self::create_history_row(name, version, time, is_security);
            today_expander.add_row(&row);
        }

        recent_group.add(&today_expander);

        // Yesterday
        let yesterday_expander = ExpanderRow::builder()
            .title("Ontem - 18 de Fevereiro")
            .subtitle("12 pacotes atualizados")
            .build();
        yesterday_expander.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));

        let yesterday_updates = vec![
            ("linux-image-6.8.0-44", "6.8.0-43 -> 6.8.0-44", "10:15", true),
            ("nvidia-driver-550", "550.54.14 -> 550.67.01", "10:15", false),
            ("com.spotify.Client (Flatpak)", "1.2.24 -> 1.2.25", "10:20", false),
            ("mesa-vulkan-drivers", "24.0.4 -> 24.0.5", "10:15", false),
        ];

        for (name, version, time, is_security) in yesterday_updates {
            let row = Self::create_history_row(name, version, time, is_security);
            yesterday_expander.add_row(&row);
        }

        recent_group.add(&yesterday_expander);

        // Last week
        let week_expander = ExpanderRow::builder()
            .title("Semana Passada")
            .subtitle("28 pacotes atualizados")
            .build();
        week_expander.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));

        let week_updates = vec![
            ("gnome-shell", "45.3 -> 45.4", "12 Fev, 16:42", false),
            ("pipewire", "1.0.1 -> 1.0.3", "12 Fev, 16:42", false),
            ("org.gimp.GIMP (Flatpak)", "2.10.34 -> 2.10.36", "11 Fev, 09:30", false),
            ("System Firmware (fwupd)", "1.11.0 -> 1.12.1", "10 Fev, 14:00", true),
        ];

        for (name, version, time, is_security) in week_updates {
            let row = Self::create_history_row(name, version, time, is_security);
            week_expander.add_row(&row);
        }

        recent_group.add(&week_expander);

        // Last month
        let month_expander = ExpanderRow::builder()
            .title("Mes Passado - Janeiro 2026")
            .subtitle("156 pacotes atualizados")
            .build();
        month_expander.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));

        let month_updates = vec![
            ("linux-image-6.8.0-42", "6.8.0-41 -> 6.8.0-42", "28 Jan", true),
            ("chromium", "120.0 -> 121.0", "25 Jan", false),
            ("code (Snap)", "1.84.2 -> 1.85.1", "20 Jan", false),
        ];

        for (name, version, time, is_security) in month_updates {
            let row = Self::create_history_row(name, version, time, is_security);
            month_expander.add_row(&row);
        }

        recent_group.add(&month_expander);

        page.add(&recent_group);

        // Rollback section
        let rollback_group = PreferencesGroup::builder()
            .title("Rollback")
            .description("Reverter atualizacoes (requer snapshots)")
            .build();

        let snapshot_row = ActionRow::builder()
            .title("Timeshift Snapshots")
            .subtitle("3 snapshots disponiveis")
            .activatable(true)
            .build();
        snapshot_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));
        snapshot_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        rollback_group.add(&snapshot_row);

        let btrfs_row = ActionRow::builder()
            .title("Btrfs Snapshots")
            .subtitle("Sistema de arquivos nao suporta")
            .sensitive(false)
            .build();
        btrfs_row.add_prefix(&gtk4::Image::from_icon_name("drive-multidisk-symbolic"));
        rollback_group.add(&btrfs_row);

        let apt_history_row = ActionRow::builder()
            .title("Historico APT")
            .subtitle("Ver /var/log/apt/history.log")
            .activatable(true)
            .build();
        apt_history_row.add_prefix(&gtk4::Image::from_icon_name("text-x-generic-symbolic"));
        apt_history_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        rollback_group.add(&apt_history_row);

        // Downgrade package
        let downgrade_row = ActionRow::builder()
            .title("Fazer Downgrade de Pacote")
            .subtitle("Reverter um pacote especifico para versao anterior")
            .activatable(true)
            .build();
        downgrade_row.add_prefix(&gtk4::Image::from_icon_name("go-down-symbolic"));
        downgrade_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        rollback_group.add(&downgrade_row);

        page.add(&rollback_group);

        // Export section
        let export_group = PreferencesGroup::builder()
            .title("Exportar")
            .build();

        let export_row = ActionRow::builder()
            .title("Exportar Historico")
            .subtitle("Exportar lista de atualizacoes para arquivo")
            .activatable(true)
            .build();
        export_row.add_prefix(&gtk4::Image::from_icon_name("document-save-symbolic"));

        let export_btn = Button::with_label("Exportar");
        export_btn.add_css_class("flat");
        export_btn.set_valign(gtk4::Align::Center);
        export_row.add_suffix(&export_btn);
        export_group.add(&export_row);

        page.add(&export_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            update_manager,
        }
    }

    fn create_history_row(name: &str, version: &str, time: &str, is_security: bool) -> ActionRow {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(&format!("{} - {}", version, time))
            .build();

        if is_security {
            let security_icon = gtk4::Image::from_icon_name("security-high-symbolic");
            security_icon.set_tooltip_text(Some("Atualizacao de seguranca"));
            security_icon.add_css_class("warning");
            row.add_suffix(&security_icon);
        }

        // Info button
        let info_btn = Button::builder()
            .icon_name("dialog-information-symbolic")
            .valign(gtk4::Align::Center)
            .tooltip_text("Ver detalhes")
            .build();
        info_btn.add_css_class("flat");
        row.add_suffix(&info_btn);

        row
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}
