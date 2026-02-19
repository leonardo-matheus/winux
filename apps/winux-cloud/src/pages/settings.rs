//! Settings page - Sync configuration

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, Scale, SpinButton, Adjustment};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow, ComboRow, ExpanderRow};

/// Settings page
pub struct SettingsPage {
    widget: gtk4::ScrolledWindow,
}

impl SettingsPage {
    pub fn new() -> Self {
        let content = Box::new(Orientation::Vertical, 0);

        // Status header
        let status = StatusPage::builder()
            .icon_name("emblem-system-symbolic")
            .title("Configuracoes")
            .description("Configure a sincronizacao e seguranca")
            .build();
        content.append(&status);

        let page = PreferencesPage::new();

        // Sync Folders Group
        let folders_group = PreferencesGroup::builder()
            .title("Pastas de Sincronizacao")
            .description("Escolha quais pastas sincronizar")
            .build();

        let sync_folder_row = ActionRow::builder()
            .title("Pasta Local de Sincronizacao")
            .subtitle("~/Winux Cloud")
            .activatable(true)
            .build();
        sync_folder_row.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));

        let change_btn = Button::with_label("Alterar");
        change_btn.add_css_class("flat");
        change_btn.set_valign(gtk4::Align::Center);
        sync_folder_row.add_suffix(&change_btn);
        folders_group.add(&sync_folder_row);

        // Selective sync expander
        let selective_row = ExpanderRow::builder()
            .title("Sincronizacao Seletiva")
            .subtitle("Escolha pastas especificas para sincronizar")
            .show_enable_switch(true)
            .enable_expansion(true)
            .expanded(false)
            .build();

        let folders_to_sync = [
            ("Documentos", true),
            ("Imagens", true),
            ("Projetos", true),
            ("Videos", false),
            ("Musica", false),
            ("Downloads", false),
        ];

        for (folder, enabled) in folders_to_sync {
            let folder_row = SwitchRow::builder()
                .title(folder)
                .active(enabled)
                .build();
            selective_row.add_row(&folder_row);
        }

        folders_group.add(&selective_row);

        page.add(&folders_group);

        // Bandwidth Group
        let bandwidth_group = PreferencesGroup::builder()
            .title("Limite de Banda")
            .description("Controle a velocidade de sincronizacao")
            .build();

        let limit_download_row = SwitchRow::builder()
            .title("Limitar Download")
            .subtitle("Limitar velocidade de download")
            .active(false)
            .build();
        bandwidth_group.add(&limit_download_row);

        let download_speed_row = ActionRow::builder()
            .title("Velocidade Maxima de Download")
            .subtitle("Ilimitado")
            .build();
        download_speed_row.add_prefix(&gtk4::Image::from_icon_name("go-down-symbolic"));

        let download_adj = Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let download_spin = SpinButton::new(Some(&download_adj), 1.0, 0);
        download_spin.set_valign(gtk4::Align::Center);
        download_spin.set_tooltip_text(Some("MB/s (0 = ilimitado)"));
        download_speed_row.add_suffix(&download_spin);

        let mb_label = gtk4::Label::new(Some("MB/s"));
        mb_label.add_css_class("dim-label");
        mb_label.set_valign(gtk4::Align::Center);
        download_speed_row.add_suffix(&mb_label);
        bandwidth_group.add(&download_speed_row);

        let limit_upload_row = SwitchRow::builder()
            .title("Limitar Upload")
            .subtitle("Limitar velocidade de upload")
            .active(false)
            .build();
        bandwidth_group.add(&limit_upload_row);

        let upload_speed_row = ActionRow::builder()
            .title("Velocidade Maxima de Upload")
            .subtitle("Ilimitado")
            .build();
        upload_speed_row.add_prefix(&gtk4::Image::from_icon_name("go-up-symbolic"));

        let upload_adj = Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let upload_spin = SpinButton::new(Some(&upload_adj), 1.0, 0);
        upload_spin.set_valign(gtk4::Align::Center);
        upload_speed_row.add_suffix(&upload_spin);

        let mb_label2 = gtk4::Label::new(Some("MB/s"));
        mb_label2.add_css_class("dim-label");
        mb_label2.set_valign(gtk4::Align::Center);
        upload_speed_row.add_suffix(&mb_label2);
        bandwidth_group.add(&upload_speed_row);

        page.add(&bandwidth_group);

        // Conflict Resolution Group
        let conflict_group = PreferencesGroup::builder()
            .title("Resolucao de Conflitos")
            .description("Como resolver arquivos modificados em ambos os lados")
            .build();

        let conflict_modes = gtk4::StringList::new(&[
            "Manter Ambos (criar copia)",
            "Local Vence",
            "Remoto Vence",
            "Perguntar Sempre",
        ]);

        let conflict_row = ComboRow::builder()
            .title("Estrategia Padrao")
            .subtitle("Quando um arquivo e modificado localmente e remotamente")
            .model(&conflict_modes)
            .selected(0)
            .build();
        conflict_group.add(&conflict_row);

        page.add(&conflict_group);

        // Security Group
        let security_group = PreferencesGroup::builder()
            .title("Seguranca")
            .description("Proteja seus arquivos na nuvem")
            .build();

        let encryption_row = SwitchRow::builder()
            .title("Criptografia Client-Side")
            .subtitle("Criptografar arquivos antes de enviar para a nuvem")
            .active(false)
            .build();
        security_group.add(&encryption_row);

        let encryption_key_row = ActionRow::builder()
            .title("Chave de Criptografia")
            .subtitle("Configurada")
            .build();
        encryption_key_row.add_prefix(&gtk4::Image::from_icon_name("channel-secure-symbolic"));

        let change_key_btn = Button::with_label("Alterar");
        change_key_btn.add_css_class("flat");
        change_key_btn.set_valign(gtk4::Align::Center);
        encryption_key_row.add_suffix(&change_key_btn);
        security_group.add(&encryption_key_row);

        let zero_knowledge_row = ActionRow::builder()
            .title("Zero-Knowledge")
            .subtitle("Provedores nao podem ler seus arquivos")
            .build();
        zero_knowledge_row.add_prefix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));
        security_group.add(&zero_knowledge_row);

        page.add(&security_group);

        // Version History Group
        let versioning_group = PreferencesGroup::builder()
            .title("Historico de Versoes")
            .description("Manter versoes anteriores dos arquivos")
            .build();

        let versioning_row = SwitchRow::builder()
            .title("Manter Historico")
            .subtitle("Guardar versoes anteriores quando arquivos sao modificados")
            .active(true)
            .build();
        versioning_group.add(&versioning_row);

        let versions_count_row = ActionRow::builder()
            .title("Numero de Versoes")
            .subtitle("Quantas versoes manter por arquivo")
            .build();
        versions_count_row.add_prefix(&gtk4::Image::from_icon_name("view-list-symbolic"));

        let versions_adj = Adjustment::new(10.0, 1.0, 100.0, 1.0, 10.0, 0.0);
        let versions_spin = SpinButton::new(Some(&versions_adj), 1.0, 0);
        versions_spin.set_valign(gtk4::Align::Center);
        versions_count_row.add_suffix(&versions_spin);
        versioning_group.add(&versions_count_row);

        let retention_row = ActionRow::builder()
            .title("Retencao")
            .subtitle("Manter versoes por 30 dias")
            .build();
        retention_row.add_prefix(&gtk4::Image::from_icon_name("x-office-calendar-symbolic"));

        let retention_adj = Adjustment::new(30.0, 1.0, 365.0, 1.0, 10.0, 0.0);
        let retention_spin = SpinButton::new(Some(&retention_adj), 1.0, 0);
        retention_spin.set_valign(gtk4::Align::Center);
        retention_row.add_suffix(&retention_spin);

        let days_label = gtk4::Label::new(Some("dias"));
        days_label.add_css_class("dim-label");
        days_label.set_valign(gtk4::Align::Center);
        retention_row.add_suffix(&days_label);
        versioning_group.add(&retention_row);

        page.add(&versioning_group);

        // Integration Group
        let integration_group = PreferencesGroup::builder()
            .title("Integracao com Sistema")
            .build();

        let nautilus_row = SwitchRow::builder()
            .title("Integracao com Arquivos")
            .subtitle("Mostrar status de sincronizacao no gerenciador de arquivos")
            .active(true)
            .build();
        integration_group.add(&nautilus_row);

        let startup_row = SwitchRow::builder()
            .title("Iniciar com o Sistema")
            .subtitle("Iniciar sincronizacao automaticamente no login")
            .active(true)
            .build();
        integration_group.add(&startup_row);

        let notifications_row = SwitchRow::builder()
            .title("Notificacoes")
            .subtitle("Mostrar notificacoes de sincronizacao")
            .active(true)
            .build();
        integration_group.add(&notifications_row);

        let tray_row = SwitchRow::builder()
            .title("Icone na Bandeja")
            .subtitle("Mostrar icone na area de notificacao")
            .active(true)
            .build();
        integration_group.add(&tray_row);

        page.add(&integration_group);

        // Advanced Group
        let advanced_group = PreferencesGroup::builder()
            .title("Avancado")
            .build();

        let ignore_patterns_row = ActionRow::builder()
            .title("Padroes de Exclusao")
            .subtitle("*.tmp, *.log, node_modules/, .git/")
            .activatable(true)
            .build();
        ignore_patterns_row.add_prefix(&gtk4::Image::from_icon_name("edit-symbolic"));
        ignore_patterns_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        advanced_group.add(&ignore_patterns_row);

        let cache_row = ActionRow::builder()
            .title("Cache Local")
            .subtitle("256 MB utilizados")
            .build();
        cache_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));

        let clear_cache_btn = Button::with_label("Limpar");
        clear_cache_btn.add_css_class("flat");
        clear_cache_btn.set_valign(gtk4::Align::Center);
        cache_row.add_suffix(&clear_cache_btn);
        advanced_group.add(&cache_row);

        let logs_row = ActionRow::builder()
            .title("Logs de Depuracao")
            .subtitle("Ver logs detalhados")
            .activatable(true)
            .build();
        logs_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        logs_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        advanced_group.add(&logs_row);

        let reset_row = ActionRow::builder()
            .title("Resetar Sincronizacao")
            .subtitle("Limpar estado e resincronizar tudo")
            .activatable(true)
            .build();
        reset_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));

        let reset_btn = Button::with_label("Resetar");
        reset_btn.add_css_class("destructive-action");
        reset_btn.set_valign(gtk4::Align::Center);
        reset_row.add_suffix(&reset_btn);
        advanced_group.add(&reset_row);

        page.add(&advanced_group);

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

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}
