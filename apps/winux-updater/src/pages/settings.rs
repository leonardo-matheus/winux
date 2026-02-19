//! Settings page - Configure update behavior

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, SwitchRow, ComboRow, SpinRow, ExpanderRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::UpdateManager;

pub struct SettingsPage {
    widget: ScrolledWindow,
    update_manager: Rc<RefCell<UpdateManager>>,
}

impl SettingsPage {
    pub fn new(update_manager: Rc<RefCell<UpdateManager>>) -> Self {
        let page = PreferencesPage::new();

        // Automatic updates section
        let auto_group = PreferencesGroup::builder()
            .title("Atualizacoes Automaticas")
            .description("Configurar verificacao e instalacao automatica")
            .build();

        // Auto check
        let auto_check = SwitchRow::builder()
            .title("Verificar Automaticamente")
            .subtitle("Verificar por atualizacoes periodicamente")
            .active(true)
            .build();
        auto_group.add(&auto_check);

        // Check frequency
        let check_freq = ComboRow::builder()
            .title("Frequencia de Verificacao")
            .subtitle("Com que frequencia verificar por atualizacoes")
            .build();
        let frequencies = gtk4::StringList::new(&[
            "A cada hora",
            "A cada 6 horas",
            "Diariamente",
            "Semanalmente",
        ]);
        check_freq.set_model(Some(&frequencies));
        check_freq.set_selected(2); // Daily
        auto_group.add(&check_freq);

        // Auto download
        let auto_download = SwitchRow::builder()
            .title("Baixar Automaticamente")
            .subtitle("Baixar atualizacoes em segundo plano")
            .active(true)
            .build();
        auto_group.add(&auto_download);

        // Auto install security
        let auto_security = SwitchRow::builder()
            .title("Instalar Seguranca Automaticamente")
            .subtitle("Instalar atualizacoes de seguranca criticas sem confirmacao")
            .active(false)
            .build();
        auto_group.add(&auto_security);

        // Auto install all
        let auto_install = SwitchRow::builder()
            .title("Instalar Todas Automaticamente")
            .subtitle("Instalar todas atualizacoes sem confirmacao")
            .active(false)
            .build();
        auto_group.add(&auto_install);

        page.add(&auto_group);

        // Notification settings
        let notify_group = PreferencesGroup::builder()
            .title("Notificacoes")
            .build();

        let notify_available = SwitchRow::builder()
            .title("Atualizacoes Disponiveis")
            .subtitle("Notificar quando houver novas atualizacoes")
            .active(true)
            .build();
        notify_group.add(&notify_available);

        let notify_security = SwitchRow::builder()
            .title("Atualizacoes de Seguranca")
            .subtitle("Notificacao especial para atualizacoes de seguranca")
            .active(true)
            .build();
        notify_group.add(&notify_security);

        let notify_complete = SwitchRow::builder()
            .title("Atualizacao Concluida")
            .subtitle("Notificar quando atualizacoes forem instaladas")
            .active(true)
            .build();
        notify_group.add(&notify_complete);

        let notify_restart = SwitchRow::builder()
            .title("Reinicio Necessario")
            .subtitle("Lembrar quando reinicio for necessario")
            .active(true)
            .build();
        notify_group.add(&notify_restart);

        page.add(&notify_group);

        // Maintenance window
        let maintenance_group = PreferencesGroup::builder()
            .title("Janela de Manutencao")
            .description("Horario preferido para atualizacoes automaticas")
            .build();

        let enable_window = SwitchRow::builder()
            .title("Usar Janela de Manutencao")
            .subtitle("Agendar atualizacoes para horario especifico")
            .active(false)
            .build();
        maintenance_group.add(&enable_window);

        let start_time = ComboRow::builder()
            .title("Hora de Inicio")
            .build();
        let start_times = gtk4::StringList::new(&[
            "00:00", "01:00", "02:00", "03:00", "04:00", "05:00",
            "06:00", "07:00", "08:00", "09:00", "10:00", "11:00",
            "12:00", "13:00", "14:00", "15:00", "16:00", "17:00",
            "18:00", "19:00", "20:00", "21:00", "22:00", "23:00",
        ]);
        start_time.set_model(Some(&start_times));
        start_time.set_selected(2); // 02:00
        maintenance_group.add(&start_time);

        let duration = ComboRow::builder()
            .title("Duracao")
            .build();
        let durations = gtk4::StringList::new(&[
            "1 hora", "2 horas", "3 horas", "4 horas", "6 horas",
        ]);
        duration.set_model(Some(&durations));
        duration.set_selected(1); // 2 hours
        maintenance_group.add(&duration);

        let weekdays_only = SwitchRow::builder()
            .title("Apenas Dias Uteis")
            .subtitle("Nao atualizar nos fins de semana")
            .active(false)
            .build();
        maintenance_group.add(&weekdays_only);

        page.add(&maintenance_group);

        // Update sources
        let sources_group = PreferencesGroup::builder()
            .title("Fontes de Atualizacao")
            .description("Configurar quais fontes verificar")
            .build();

        let apt_enabled = SwitchRow::builder()
            .title("Sistema (APT)")
            .subtitle("Pacotes do sistema Debian/Ubuntu")
            .active(true)
            .build();
        apt_enabled.add_prefix(&gtk4::Image::from_icon_name("package-x-generic-symbolic"));
        sources_group.add(&apt_enabled);

        let flatpak_enabled = SwitchRow::builder()
            .title("Flatpak")
            .subtitle("Aplicativos sandboxed")
            .active(true)
            .build();
        flatpak_enabled.add_prefix(&gtk4::Image::from_icon_name("application-x-executable-symbolic"));
        sources_group.add(&flatpak_enabled);

        let snap_enabled = SwitchRow::builder()
            .title("Snap")
            .subtitle("Pacotes Snap")
            .active(true)
            .build();
        snap_enabled.add_prefix(&gtk4::Image::from_icon_name("application-x-addon-symbolic"));
        sources_group.add(&snap_enabled);

        let fwupd_enabled = SwitchRow::builder()
            .title("Firmware (fwupd)")
            .subtitle("Atualizacoes de firmware")
            .active(true)
            .build();
        fwupd_enabled.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-solidstate-symbolic"));
        sources_group.add(&fwupd_enabled);

        page.add(&sources_group);

        // Download settings
        let download_group = PreferencesGroup::builder()
            .title("Download")
            .build();

        let metered = SwitchRow::builder()
            .title("Pausar em Conexao Limitada")
            .subtitle("Nao baixar em redes com limite de dados")
            .active(true)
            .build();
        download_group.add(&metered);

        let parallel = SpinRow::builder()
            .title("Downloads Paralelos")
            .subtitle("Numero maximo de downloads simultaneos")
            .adjustment(&gtk4::Adjustment::new(2.0, 1.0, 10.0, 1.0, 1.0, 0.0))
            .build();
        download_group.add(&parallel);

        let bandwidth = ComboRow::builder()
            .title("Limite de Banda")
            .subtitle("Limitar velocidade de download")
            .build();
        let limits = gtk4::StringList::new(&[
            "Sem limite",
            "1 MB/s",
            "5 MB/s",
            "10 MB/s",
            "50 MB/s",
        ]);
        bandwidth.set_model(Some(&limits));
        download_group.add(&bandwidth);

        page.add(&download_group);

        // Advanced settings
        let advanced_group = PreferencesGroup::builder()
            .title("Avancado")
            .build();

        let keep_packages = SwitchRow::builder()
            .title("Manter Cache de Pacotes")
            .subtitle("Nao limpar pacotes baixados apos instalacao")
            .active(false)
            .build();
        advanced_group.add(&keep_packages);

        let auto_remove = SwitchRow::builder()
            .title("Remover Pacotes Orfaos")
            .subtitle("Remover automaticamente dependencias nao utilizadas")
            .active(true)
            .build();
        advanced_group.add(&auto_remove);

        let clean_cache = ActionRow::builder()
            .title("Limpar Cache")
            .subtitle("Remover pacotes em cache (156 MB)")
            .activatable(true)
            .build();
        clean_cache.add_prefix(&gtk4::Image::from_icon_name("edit-clear-symbolic"));

        let clean_btn = Button::with_label("Limpar");
        clean_btn.add_css_class("flat");
        clean_btn.set_valign(gtk4::Align::Center);
        clean_cache.add_suffix(&clean_btn);
        advanced_group.add(&clean_cache);

        // Refresh mirrors
        let mirrors_row = ActionRow::builder()
            .title("Atualizar Lista de Repositorios")
            .subtitle("Executar apt update")
            .activatable(true)
            .build();
        mirrors_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));

        let refresh_btn = Button::with_label("Atualizar");
        refresh_btn.add_css_class("flat");
        refresh_btn.set_valign(gtk4::Align::Center);
        mirrors_row.add_suffix(&refresh_btn);
        advanced_group.add(&mirrors_row);

        // Repair packages
        let repair_row = ActionRow::builder()
            .title("Reparar Pacotes")
            .subtitle("Corrigir pacotes quebrados (dpkg --configure -a)")
            .activatable(true)
            .build();
        repair_row.add_prefix(&gtk4::Image::from_icon_name("applications-utilities-symbolic"));

        let repair_btn = Button::with_label("Reparar");
        repair_btn.add_css_class("flat");
        repair_btn.set_valign(gtk4::Align::Center);
        repair_row.add_suffix(&repair_btn);
        advanced_group.add(&repair_row);

        page.add(&advanced_group);

        // PPAs and repositories
        let repos_group = PreferencesGroup::builder()
            .title("Repositorios")
            .build();

        let manage_repos = ActionRow::builder()
            .title("Gerenciar Repositorios")
            .subtitle("Adicionar, remover ou editar fontes de software")
            .activatable(true)
            .build();
        manage_repos.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));
        manage_repos.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        repos_group.add(&manage_repos);

        let flatpak_remotes = ActionRow::builder()
            .title("Flatpak Remotes")
            .subtitle("Gerenciar repositorios Flatpak")
            .activatable(true)
            .build();
        flatpak_remotes.add_prefix(&gtk4::Image::from_icon_name("folder-remote-symbolic"));
        flatpak_remotes.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        repos_group.add(&flatpak_remotes);

        page.add(&repos_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            update_manager,
        }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}
