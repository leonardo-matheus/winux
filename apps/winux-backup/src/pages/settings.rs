//! Settings page - Backup application settings

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, ComboRow, EntryRow, ExpanderRow, PreferencesGroup,
    PreferencesPage, SpinRow, SwitchRow,
};

/// Settings page
pub struct SettingsPage {
    widget: gtk4::ScrolledWindow,
}

impl SettingsPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();

        // Storage Backends Group
        let backends_group = PreferencesGroup::builder()
            .title("Backends de Armazenamento")
            .description("Configure destinos de backup")
            .build();

        // Local storage
        let local_config = ExpanderRow::builder()
            .title("Armazenamento Local")
            .subtitle("/media/backup/winux")
            .build();
        local_config.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));

        let local_path = EntryRow::builder()
            .title("Caminho Padrao")
            .text("/media/backup/winux")
            .build();

        let browse_btn = Button::from_icon_name("folder-open-symbolic");
        browse_btn.add_css_class("flat");
        browse_btn.set_valign(gtk4::Align::Center);
        local_path.add_suffix(&browse_btn);
        local_config.add_row(&local_path);

        backends_group.add(&local_config);

        // Rsync/SSH config
        let rsync_config = ExpanderRow::builder()
            .title("Rsync (SSH)")
            .subtitle("backup.exemplo.com")
            .build();
        rsync_config.add_prefix(&gtk4::Image::from_icon_name("network-server-symbolic"));

        let rsync_host = EntryRow::builder()
            .title("Host")
            .text("backup.exemplo.com")
            .build();
        rsync_config.add_row(&rsync_host);

        let rsync_port = SpinRow::builder()
            .title("Porta SSH")
            .adjustment(&gtk4::Adjustment::new(22.0, 1.0, 65535.0, 1.0, 100.0, 0.0))
            .build();
        rsync_config.add_row(&rsync_port);

        let rsync_user = EntryRow::builder()
            .title("Usuario")
            .text("backup")
            .build();
        rsync_config.add_row(&rsync_user);

        let rsync_path = EntryRow::builder()
            .title("Caminho Remoto")
            .text("/backups/winux")
            .build();
        rsync_config.add_row(&rsync_path);

        let rsync_key = EntryRow::builder()
            .title("Chave SSH")
            .text("~/.ssh/id_backup")
            .build();
        rsync_config.add_row(&rsync_key);

        let test_rsync = ActionRow::builder()
            .title("Testar Conexao")
            .activatable(true)
            .build();
        test_rsync.add_suffix(&Button::with_label("Testar"));
        rsync_config.add_row(&test_rsync);

        backends_group.add(&rsync_config);

        // Restic config
        let restic_config = ExpanderRow::builder()
            .title("Restic")
            .subtitle("Repositorio configurado")
            .build();
        restic_config.add_prefix(&gtk4::Image::from_icon_name("package-x-generic-symbolic"));

        let restic_repo = EntryRow::builder()
            .title("Repositorio")
            .text("/media/backup/restic-repo")
            .build();
        restic_config.add_row(&restic_repo);

        let restic_password = EntryRow::builder()
            .title("Senha do Repositorio")
            .text("••••••••")
            .build();
        restic_config.add_row(&restic_password);

        let init_restic = ActionRow::builder()
            .title("Inicializar Repositorio")
            .subtitle("Criar novo repositorio Restic")
            .activatable(true)
            .build();
        init_restic.add_suffix(&Button::with_label("Inicializar"));
        restic_config.add_row(&init_restic);

        backends_group.add(&restic_config);

        // Cloud config
        let cloud_config = ExpanderRow::builder()
            .title("Armazenamento em Nuvem")
            .subtitle("Nenhum configurado")
            .build();
        cloud_config.add_prefix(&gtk4::Image::from_icon_name("weather-overcast-symbolic"));

        let cloud_services = [
            ("Google Drive", "drive-symbolic", false),
            ("Dropbox", "folder-remote-symbolic", false),
            ("OneDrive", "folder-remote-symbolic", false),
        ];

        for (name, icon, connected) in cloud_services {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(if connected { "Conectado" } else { "Nao conectado" })
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            let connect_btn = Button::with_label(if connected { "Desconectar" } else { "Conectar" });
            connect_btn.add_css_class("flat");
            connect_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&connect_btn);

            cloud_config.add_row(&row);
        }

        backends_group.add(&cloud_config);

        page.add(&backends_group);

        // Encryption Group
        let encryption_group = PreferencesGroup::builder()
            .title("Criptografia")
            .description("Configuracoes de seguranca dos backups")
            .build();

        let default_encryption = SwitchRow::builder()
            .title("Criptografia Padrao")
            .subtitle("Criptografar novos backups por padrao")
            .active(false)
            .build();
        encryption_group.add(&default_encryption);

        let encryption_algo = ComboRow::builder()
            .title("Algoritmo")
            .build();
        let algorithms = gtk4::StringList::new(&[
            "AES-256-GCM",
            "ChaCha20-Poly1305",
        ]);
        encryption_algo.set_model(Some(&algorithms));
        encryption_group.add(&encryption_algo);

        let key_derivation = ComboRow::builder()
            .title("Derivacao de Chave")
            .build();
        let kdf = gtk4::StringList::new(&[
            "Argon2id",
            "scrypt",
            "PBKDF2",
        ]);
        key_derivation.set_model(Some(&kdf));
        encryption_group.add(&key_derivation);

        let change_password = ActionRow::builder()
            .title("Alterar Senha Mestre")
            .subtitle("Usada para criptografar backups")
            .activatable(true)
            .build();
        change_password.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        encryption_group.add(&change_password);

        page.add(&encryption_group);

        // Compression Group
        let compression_group = PreferencesGroup::builder()
            .title("Compressao")
            .build();

        let default_compression = ComboRow::builder()
            .title("Compressao Padrao")
            .subtitle("Nivel de compressao para novos backups")
            .build();
        let compression_levels = gtk4::StringList::new(&[
            "Nenhuma",
            "Rapida (LZ4)",
            "Balanceada (ZSTD)",
            "Maxima (LZMA)",
        ]);
        default_compression.set_model(Some(&compression_levels));
        default_compression.set_selected(2);
        compression_group.add(&default_compression);

        let compression_level = SpinRow::builder()
            .title("Nivel de Compressao")
            .subtitle("1 (rapido) a 9 (maximo)")
            .adjustment(&gtk4::Adjustment::new(6.0, 1.0, 9.0, 1.0, 1.0, 0.0))
            .build();
        compression_group.add(&compression_level);

        page.add(&compression_group);

        // Performance Group
        let perf_group = PreferencesGroup::builder()
            .title("Desempenho")
            .build();

        let parallel_ops = SpinRow::builder()
            .title("Operacoes Paralelas")
            .subtitle("Numero de arquivos processados simultaneamente")
            .adjustment(&gtk4::Adjustment::new(4.0, 1.0, 16.0, 1.0, 2.0, 0.0))
            .build();
        perf_group.add(&parallel_ops);

        let io_priority = ComboRow::builder()
            .title("Prioridade de I/O")
            .subtitle("Impacto no desempenho do sistema")
            .build();
        let priorities = gtk4::StringList::new(&[
            "Baixa (idle)",
            "Normal",
            "Alta",
        ]);
        io_priority.set_model(Some(&priorities));
        io_priority.set_selected(0);
        perf_group.add(&io_priority);

        let bandwidth_limit = SpinRow::builder()
            .title("Limite de Banda (MB/s)")
            .subtitle("0 = sem limite")
            .adjustment(&gtk4::Adjustment::new(0.0, 0.0, 1000.0, 1.0, 10.0, 0.0))
            .build();
        perf_group.add(&bandwidth_limit);

        page.add(&perf_group);

        // Verification Group
        let verify_group = PreferencesGroup::builder()
            .title("Verificacao")
            .build();

        let verify_after_backup = SwitchRow::builder()
            .title("Verificar Apos Backup")
            .subtitle("Verificar integridade automaticamente")
            .active(true)
            .build();
        verify_group.add(&verify_after_backup);

        let periodic_verify = SwitchRow::builder()
            .title("Verificacao Periodica")
            .subtitle("Verificar backups existentes periodicamente")
            .active(true)
            .build();
        verify_group.add(&periodic_verify);

        let verify_interval = ComboRow::builder()
            .title("Intervalo de Verificacao")
            .build();
        let intervals = gtk4::StringList::new(&[
            "Semanal",
            "Quinzenal",
            "Mensal",
        ]);
        verify_interval.set_model(Some(&intervals));
        verify_interval.set_selected(1);
        verify_group.add(&verify_interval);

        let checksum_algo = ComboRow::builder()
            .title("Algoritmo de Checksum")
            .build();
        let checksums = gtk4::StringList::new(&[
            "SHA-256",
            "BLAKE3",
            "XXH3",
        ]);
        checksum_algo.set_model(Some(&checksums));
        checksum_algo.set_selected(1);
        verify_group.add(&checksum_algo);

        page.add(&verify_group);

        // Logging Group
        let log_group = PreferencesGroup::builder()
            .title("Logs")
            .build();

        let log_level = ComboRow::builder()
            .title("Nivel de Log")
            .build();
        let levels = gtk4::StringList::new(&[
            "Erro",
            "Aviso",
            "Info",
            "Debug",
        ]);
        log_level.set_model(Some(&levels));
        log_level.set_selected(2);
        log_group.add(&log_level);

        let log_retention = SpinRow::builder()
            .title("Retencao de Logs")
            .subtitle("Dias para manter logs")
            .adjustment(&gtk4::Adjustment::new(30.0, 7.0, 365.0, 1.0, 30.0, 0.0))
            .build();
        log_group.add(&log_retention);

        let view_logs = ActionRow::builder()
            .title("Ver Logs")
            .subtitle("Abrir arquivo de log")
            .activatable(true)
            .build();
        view_logs.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        log_group.add(&view_logs);

        page.add(&log_group);

        // Data Management Group
        let data_group = PreferencesGroup::builder()
            .title("Gerenciamento de Dados")
            .build();

        let export_config = ActionRow::builder()
            .title("Exportar Configuracoes")
            .subtitle("Salvar configuracoes em arquivo")
            .activatable(true)
            .build();
        export_config.add_suffix(&gtk4::Image::from_icon_name("document-save-symbolic"));
        data_group.add(&export_config);

        let import_config = ActionRow::builder()
            .title("Importar Configuracoes")
            .subtitle("Carregar configuracoes de arquivo")
            .activatable(true)
            .build();
        import_config.add_suffix(&gtk4::Image::from_icon_name("document-open-symbolic"));
        data_group.add(&import_config);

        let reset_config = ActionRow::builder()
            .title("Redefinir Configuracoes")
            .subtitle("Restaurar configuracoes padrao")
            .activatable(true)
            .build();

        let reset_btn = Button::with_label("Redefinir");
        reset_btn.add_css_class("destructive-action");
        reset_btn.set_valign(gtk4::Align::Center);
        reset_config.add_suffix(&reset_btn);
        data_group.add(&reset_config);

        page.add(&data_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
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
