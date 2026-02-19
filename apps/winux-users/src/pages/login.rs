//! Login options page

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, PreferencesGroup, PreferencesPage, SwitchRow, StatusPage};

use crate::backend::AccountsService;

use std::cell::RefCell;
use std::rc::Rc;

/// Login options page
pub struct LoginPage {
    widget: ScrolledWindow,
    accounts_service: Rc<RefCell<AccountsService>>,
}

impl LoginPage {
    pub fn new(accounts_service: Rc<RefCell<AccountsService>>) -> Self {
        let page = PreferencesPage::new();
        page.set_title("Login");
        page.set_icon_name(Some("preferences-system-login-symbolic"));

        // Header
        let header_group = PreferencesGroup::new();

        let status = StatusPage::builder()
            .icon_name("preferences-system-login-symbolic")
            .title("Opcoes de Login")
            .description("Configure o comportamento de login e seguranca")
            .build();

        header_group.add(&status);
        page.add(&header_group);

        // Auto-login section
        let autologin_group = PreferencesGroup::builder()
            .title("Login Automatico")
            .description("Fazer login automaticamente ao iniciar")
            .build();

        let autologin_switch = SwitchRow::builder()
            .title("Login Automatico")
            .subtitle("Ignora a tela de login ao iniciar o sistema")
            .active(false)
            .build();
        autologin_group.add(&autologin_switch);

        let autologin_user = ComboRow::builder()
            .title("Usuario")
            .subtitle("Usuario para login automatico")
            .sensitive(false)
            .build();

        // Get list of users
        let users = Self::get_login_users();
        let user_list = gtk4::StringList::new(&users.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        autologin_user.set_model(Some(&user_list));
        autologin_group.add(&autologin_user);

        // Connect switch to enable/disable user selection
        let autologin_user_clone = autologin_user.clone();
        autologin_switch.connect_active_notify(move |switch| {
            autologin_user_clone.set_sensitive(switch.is_active());
        });

        let autologin_warning = ActionRow::builder()
            .title("Aviso de Seguranca")
            .subtitle("Login automatico pode representar um risco de seguranca. Qualquer pessoa com acesso fisico ao computador podera acessar sua conta.")
            .build();
        autologin_warning.add_prefix(&gtk4::Image::from_icon_name("dialog-warning-symbolic"));
        autologin_group.add(&autologin_warning);

        page.add(&autologin_group);

        // Screen lock section
        let lock_group = PreferencesGroup::builder()
            .title("Bloqueio de Tela")
            .description("Configuracoes de bloqueio e desbloqueio")
            .build();

        let lock_on_suspend = SwitchRow::builder()
            .title("Bloquear ao Suspender")
            .subtitle("Requer senha ao retomar de suspensao")
            .active(true)
            .build();
        lock_group.add(&lock_on_suspend);

        let lock_on_idle = SwitchRow::builder()
            .title("Bloquear quando Inativo")
            .subtitle("Bloqueia a tela apos periodo de inatividade")
            .active(true)
            .build();
        lock_group.add(&lock_on_idle);

        let idle_timeout = ComboRow::builder()
            .title("Tempo de Inatividade")
            .subtitle("Tempo antes de bloquear automaticamente")
            .build();
        let timeouts = gtk4::StringList::new(&[
            "1 minuto",
            "2 minutos",
            "5 minutos",
            "10 minutos",
            "15 minutos",
            "30 minutos",
            "1 hora",
            "Nunca",
        ]);
        idle_timeout.set_model(Some(&timeouts));
        idle_timeout.set_selected(2); // 5 minutes default
        lock_group.add(&idle_timeout);

        let show_notifications = SwitchRow::builder()
            .title("Notificacoes na Tela de Bloqueio")
            .subtitle("Mostrar notificacoes quando a tela estiver bloqueada")
            .active(true)
            .build();
        lock_group.add(&show_notifications);

        page.add(&lock_group);

        // Guest account section
        let guest_group = PreferencesGroup::builder()
            .title("Conta de Convidado")
            .description("Permitir acesso temporario ao sistema")
            .build();

        let guest_switch = SwitchRow::builder()
            .title("Conta de Convidado")
            .subtitle("Permite que visitantes usem o computador sem uma conta")
            .active(false)
            .build();
        guest_group.add(&guest_switch);

        let guest_info = ActionRow::builder()
            .title("Sobre a Conta de Convidado")
            .subtitle("A sessao de convidado e temporaria. Todos os arquivos e configuracoes sao removidos ao sair.")
            .build();
        guest_info.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        guest_group.add(&guest_info);

        page.add(&guest_group);

        // Login screen section
        let screen_group = PreferencesGroup::builder()
            .title("Tela de Login")
            .description("Aparencia e comportamento da tela de login")
            .build();

        let show_users = SwitchRow::builder()
            .title("Mostrar Lista de Usuarios")
            .subtitle("Exibe usuarios na tela de login")
            .active(true)
            .build();
        screen_group.add(&show_users);

        let show_banner = SwitchRow::builder()
            .title("Mostrar Mensagem de Boas-vindas")
            .subtitle("Exibe uma mensagem personalizada na tela de login")
            .active(false)
            .build();
        screen_group.add(&show_banner);

        let banner_message = adw::EntryRow::builder()
            .title("Mensagem")
            .text("Bem-vindo ao Winux OS!")
            .sensitive(false)
            .build();
        screen_group.add(&banner_message);

        // Connect banner switch
        let banner_message_clone = banner_message.clone();
        show_banner.connect_active_notify(move |switch| {
            banner_message_clone.set_sensitive(switch.is_active());
        });

        let logo_row = ActionRow::builder()
            .title("Logo Personalizado")
            .subtitle("Usar uma imagem personalizada na tela de login")
            .activatable(true)
            .build();
        logo_row.add_prefix(&gtk4::Image::from_icon_name("image-x-generic-symbolic"));
        let choose_logo_btn = Button::with_label("Escolher");
        choose_logo_btn.add_css_class("flat");
        choose_logo_btn.set_valign(gtk4::Align::Center);
        logo_row.add_suffix(&choose_logo_btn);
        screen_group.add(&logo_row);

        page.add(&screen_group);

        // Authentication section
        let auth_group = PreferencesGroup::builder()
            .title("Autenticacao")
            .description("Metodos de autenticacao disponiveis")
            .build();

        let fingerprint = SwitchRow::builder()
            .title("Impressao Digital")
            .subtitle("Usar leitor de impressao digital para autenticacao")
            .active(false)
            .sensitive(Self::has_fingerprint_reader())
            .build();
        auth_group.add(&fingerprint);

        if !Self::has_fingerprint_reader() {
            let no_fp_row = ActionRow::builder()
                .title("Leitor de Impressao Digital")
                .subtitle("Nenhum leitor de impressao digital detectado")
                .build();
            no_fp_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
            auth_group.add(&no_fp_row);
        }

        let smartcard = SwitchRow::builder()
            .title("Smart Card")
            .subtitle("Permitir autenticacao com smart card")
            .active(false)
            .build();
        auth_group.add(&smartcard);

        page.add(&auth_group);

        // PAM/Security section
        let security_group = PreferencesGroup::builder()
            .title("Seguranca")
            .build();

        let failed_attempts = ComboRow::builder()
            .title("Tentativas de Login")
            .subtitle("Numero de tentativas antes de bloquear")
            .build();
        let attempts = gtk4::StringList::new(&["3", "5", "10", "Ilimitado"]);
        failed_attempts.set_model(Some(&attempts));
        failed_attempts.set_selected(1); // 5 attempts
        security_group.add(&failed_attempts);

        let lockout_time = ComboRow::builder()
            .title("Tempo de Bloqueio")
            .subtitle("Tempo de espera apos falhas de login")
            .build();
        let lockout_times = gtk4::StringList::new(&["30 segundos", "1 minuto", "5 minutos", "10 minutos"]);
        lockout_time.set_model(Some(&lockout_times));
        lockout_time.set_selected(1); // 1 minute
        security_group.add(&lockout_time);

        page.add(&security_group);

        // Apply changes
        let apply_group = PreferencesGroup::new();

        let apply_box = Box::new(Orientation::Horizontal, 12);
        apply_box.set_halign(gtk4::Align::Center);
        apply_box.set_margin_top(24);
        apply_box.set_margin_bottom(24);

        let apply_btn = Button::with_label("Aplicar Alteracoes");
        apply_btn.add_css_class("suggested-action");
        apply_btn.add_css_class("pill");
        apply_btn.connect_clicked(|_| {
            tracing::info!("Apply login settings clicked");
            // TODO: Apply settings via AccountsService
        });
        apply_box.append(&apply_btn);

        apply_group.add(&apply_box);
        page.add(&apply_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        LoginPage {
            widget: scrolled,
            accounts_service,
        }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    fn get_login_users() -> Vec<String> {
        let mut users = Vec::new();

        if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 7 {
                    let username = parts[0];
                    let uid: u32 = parts[2].parse().unwrap_or(0);
                    let shell = parts[6];

                    // Include only regular users with valid shells
                    if uid >= 1000 && uid < 65534 && !shell.contains("nologin") && !shell.contains("false") {
                        users.push(username.to_string());
                    }
                }
            }
        }

        if users.is_empty() {
            users.push(std::env::var("USER").unwrap_or_else(|_| "usuario".to_string()));
        }

        users
    }

    fn has_fingerprint_reader() -> bool {
        // Check if fprintd is available and a device is present
        std::path::Path::new("/var/lib/fprint").exists()
            || std::process::Command::new("fprintd-list")
                .arg(std::env::var("USER").unwrap_or_default())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }
}
