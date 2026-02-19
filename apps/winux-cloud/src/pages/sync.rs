//! Sync page - Synchronization status and controls

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, ProgressBar, Label};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow};

/// Sync status page
pub struct SyncPage {
    widget: gtk4::ScrolledWindow,
}

impl SyncPage {
    pub fn new() -> Self {
        let content = Box::new(Orientation::Vertical, 0);

        // Status header
        let status = StatusPage::builder()
            .icon_name("emblem-synchronizing-symbolic")
            .title("Sincronizacao")
            .description("Status e controles de sincronizacao")
            .build();
        content.append(&status);

        let page = PreferencesPage::new();

        // Current Sync Status Group
        let current_group = PreferencesGroup::builder()
            .title("Status Atual")
            .build();

        let status_row = ActionRow::builder()
            .title("Status da Sincronizacao")
            .subtitle("Sincronizando 3 arquivos...")
            .build();
        status_row.add_prefix(&gtk4::Image::from_icon_name("emblem-synchronizing-symbolic"));

        let pause_btn = Button::from_icon_name("media-playback-pause-symbolic");
        pause_btn.add_css_class("flat");
        pause_btn.set_valign(gtk4::Align::Center);
        pause_btn.set_tooltip_text(Some("Pausar sincronizacao"));
        status_row.add_suffix(&pause_btn);
        current_group.add(&status_row);

        // Current file progress
        let file_row = ActionRow::builder()
            .title("documento_importante.pdf")
            .subtitle("Google Drive - 2.5 MB de 8.3 MB (30%)")
            .build();
        file_row.add_prefix(&gtk4::Image::from_icon_name("document-symbolic"));

        let progress = ProgressBar::new();
        progress.set_fraction(0.30);
        progress.set_valign(gtk4::Align::Center);
        progress.set_size_request(150, -1);
        file_row.add_suffix(&progress);
        current_group.add(&file_row);

        // Speed info
        let speed_row = ActionRow::builder()
            .title("Velocidade")
            .subtitle("Download: 2.5 MB/s | Upload: 1.2 MB/s")
            .build();
        speed_row.add_prefix(&gtk4::Image::from_icon_name("network-transmit-receive-symbolic"));
        current_group.add(&speed_row);

        page.add(&current_group);

        // Sync Queue Group
        let queue_group = PreferencesGroup::builder()
            .title("Fila de Sincronizacao")
            .description("Arquivos aguardando sincronizacao")
            .build();

        let queue_items = [
            ("projeto_final.zip", "45.2 MB", "Upload para OneDrive", "package-x-generic-symbolic"),
            ("fotos_ferias/", "128 arquivos", "Download do Google Drive", "folder-pictures-symbolic"),
            ("relatorio_2026.docx", "1.2 MB", "Upload para Dropbox", "x-office-document-symbolic"),
        ];

        for (name, size, action, icon) in queue_items {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} - {}", size, action))
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            let cancel_btn = Button::from_icon_name("window-close-symbolic");
            cancel_btn.add_css_class("flat");
            cancel_btn.set_valign(gtk4::Align::Center);
            cancel_btn.set_tooltip_text(Some("Cancelar"));
            row.add_suffix(&cancel_btn);

            queue_group.add(&row);
        }

        let view_all_row = ActionRow::builder()
            .title("Ver fila completa")
            .subtitle("12 itens restantes")
            .activatable(true)
            .build();
        view_all_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        queue_group.add(&view_all_row);

        page.add(&queue_group);

        // Sync Options Group
        let options_group = PreferencesGroup::builder()
            .title("Opcoes de Sincronizacao")
            .build();

        let auto_sync_row = SwitchRow::builder()
            .title("Sincronizacao Automatica")
            .subtitle("Sincronizar automaticamente quando houver alteracoes")
            .active(true)
            .build();
        options_group.add(&auto_sync_row);

        let background_row = SwitchRow::builder()
            .title("Sincronizar em Segundo Plano")
            .subtitle("Continuar sincronizando mesmo com o app fechado")
            .active(true)
            .build();
        options_group.add(&background_row);

        let wifi_row = SwitchRow::builder()
            .title("Apenas Wi-Fi")
            .subtitle("Sincronizar apenas quando conectado ao Wi-Fi")
            .active(false)
            .build();
        options_group.add(&wifi_row);

        let battery_row = SwitchRow::builder()
            .title("Economizar Bateria")
            .subtitle("Pausar sincronizacao quando bateria estiver baixa")
            .active(true)
            .build();
        options_group.add(&battery_row);

        page.add(&options_group);

        // Sync Conflicts Group
        let conflicts_group = PreferencesGroup::builder()
            .title("Conflitos")
            .description("Arquivos com alteracoes conflitantes")
            .build();

        let conflict_row = ActionRow::builder()
            .title("notas_reuniao.txt")
            .subtitle("Modificado localmente e no servidor")
            .activatable(true)
            .build();
        conflict_row.add_prefix(&gtk4::Image::from_icon_name("dialog-warning-symbolic"));

        let resolve_btn = Button::with_label("Resolver");
        resolve_btn.add_css_class("suggested-action");
        resolve_btn.set_valign(gtk4::Align::Center);
        conflict_row.add_suffix(&resolve_btn);
        conflicts_group.add(&conflict_row);

        let no_conflicts_row = ActionRow::builder()
            .title("Nenhum conflito pendente")
            .subtitle("Todos os arquivos estao sincronizados")
            .build();
        no_conflicts_row.add_prefix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));
        // This row would be shown when there are no conflicts
        // conflicts_group.add(&no_conflicts_row);

        page.add(&conflicts_group);

        // Sync Statistics Group
        let stats_group = PreferencesGroup::builder()
            .title("Estatisticas")
            .build();

        let today_row = ActionRow::builder()
            .title("Sincronizado Hoje")
            .subtitle("234 arquivos, 1.2 GB transferidos")
            .build();
        today_row.add_prefix(&gtk4::Image::from_icon_name("x-office-calendar-symbolic"));
        stats_group.add(&today_row);

        let week_row = ActionRow::builder()
            .title("Esta Semana")
            .subtitle("1,456 arquivos, 8.7 GB transferidos")
            .build();
        week_row.add_prefix(&gtk4::Image::from_icon_name("view-list-bullet-symbolic"));
        stats_group.add(&week_row);

        let errors_row = ActionRow::builder()
            .title("Erros de Sincronizacao")
            .subtitle("2 arquivos falharam (clique para ver)")
            .activatable(true)
            .build();
        errors_row.add_prefix(&gtk4::Image::from_icon_name("dialog-error-symbolic"));
        errors_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        stats_group.add(&errors_row);

        page.add(&stats_group);

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

impl Default for SyncPage {
    fn default() -> Self {
        Self::new()
    }
}
