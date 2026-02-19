//! Apps page - Application-based firewall permissions
//!
//! Features:
//! - Per-application profiles
//! - Active connections view
//! - Application rules
//! - Network usage per app

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ScrolledWindow, Spinner};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::AppRow;

/// Applications page for per-app firewall control
pub struct AppsPage {
    widget: ScrolledWindow,
}

impl AppsPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Aplicativos");
        page.set_icon_name(Some("application-x-executable-symbolic"));

        // Active Connections Group
        let active_group = PreferencesGroup::builder()
            .title("Conexoes Ativas")
            .description("Aplicativos com conexoes de rede abertas")
            .build();

        // Refresh button
        let refresh_row = ActionRow::builder()
            .title("Atualizar")
            .subtitle("Verificar conexoes ativas")
            .activatable(true)
            .build();

        let refresh_spinner = Spinner::new();
        refresh_row.add_suffix(&refresh_spinner);

        let refresh_icon = Image::from_icon_name("view-refresh-symbolic");
        refresh_row.add_suffix(&refresh_icon);

        refresh_row.connect_activated({
            let spinner = refresh_spinner.clone();
            move |_| {
                spinner.start();
                tracing::info!("Scanning active connections...");
                glib::timeout_add_seconds_local_once(2, {
                    let spinner = spinner.clone();
                    move || spinner.stop()
                });
            }
        });

        active_group.add(&refresh_row);

        // Sample active connections
        let active_apps = [
            ("Firefox", "firefox", "15 conexoes", "192.168.1.100:443 -> 142.250.185.68:443"),
            ("Spotify", "spotify", "3 conexoes", "192.168.1.100:54321 -> 35.186.224.47:443"),
            ("VS Code", "code", "7 conexoes", "192.168.1.100:52100 -> 13.107.6.158:443"),
            ("Discord", "discord", "4 conexoes", "192.168.1.100:50000 -> 162.159.128.233:443"),
            ("Thunderbird", "thunderbird", "2 conexoes", "192.168.1.100:993 -> 142.250.185.109:993"),
        ];

        for (name, icon, conn_count, sample_conn) in active_apps {
            let row = ExpanderRow::builder()
                .title(name)
                .subtitle(conn_count)
                .build();

            let app_icon = Image::from_icon_name(&format!("{}-symbolic", icon));
            row.add_prefix(&app_icon);

            // Connection details
            let conn_row = ActionRow::builder()
                .title("Conexao Ativa")
                .subtitle(sample_conn)
                .build();
            row.add_row(&conn_row);

            // Block option
            let block_row = ActionRow::builder()
                .title("Bloquear Temporariamente")
                .subtitle("Interromper todas as conexoes deste app")
                .activatable(true)
                .build();
            block_row.add_css_class("error");

            let block_icon = Image::from_icon_name("action-unavailable-symbolic");
            block_row.add_prefix(&block_icon);

            let name_clone = name.to_string();
            block_row.connect_activated(move |_| {
                tracing::info!("Blocking all connections for: {}", name_clone);
            });
            row.add_row(&block_row);

            active_group.add(&row);
        }

        page.add(&active_group);

        // Application Profiles Group (UFW app profiles)
        let profiles_group = PreferencesGroup::builder()
            .title("Perfis de Aplicativos")
            .description("Perfis UFW pre-configurados para aplicativos")
            .build();

        // Sample UFW app profiles
        let app_profiles = [
            ("Apache", "Servidor web Apache", "80,443/tcp"),
            ("Apache Full", "Apache + SSL", "80,443/tcp"),
            ("Apache Secure", "Apache somente HTTPS", "443/tcp"),
            ("Nginx Full", "Servidor web Nginx", "80,443/tcp"),
            ("OpenSSH", "Servidor SSH", "22/tcp"),
            ("Samba", "Compartilhamento de arquivos", "137,138/udp 139,445/tcp"),
            ("CUPS", "Servidor de impressao", "631/tcp"),
            ("Postfix", "Servidor de email", "25/tcp"),
            ("Dovecot IMAP", "Servidor IMAP", "143,993/tcp"),
            ("Dovecot POP3", "Servidor POP3", "110,995/tcp"),
        ];

        for (name, desc, ports) in app_profiles {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} - {}", desc, ports))
                .build();

            let allow_btn = Button::with_label("Permitir");
            allow_btn.add_css_class("success");
            allow_btn.add_css_class("flat");
            allow_btn.set_valign(gtk4::Align::Center);

            let name_clone = name.to_string();
            allow_btn.connect_clicked(move |_| {
                tracing::info!("Allowing UFW app profile: {}", name_clone);
                // UfwBackend::allow_app(&name_clone);
            });

            let deny_btn = Button::with_label("Negar");
            deny_btn.add_css_class("error");
            deny_btn.add_css_class("flat");
            deny_btn.set_valign(gtk4::Align::Center);

            let name_clone = name.to_string();
            deny_btn.connect_clicked(move |_| {
                tracing::info!("Denying UFW app profile: {}", name_clone);
                // UfwBackend::deny_app(&name_clone);
            });

            row.add_suffix(&allow_btn);
            row.add_suffix(&deny_btn);

            profiles_group.add(&row);
        }

        // Show all profiles
        let all_profiles_row = ActionRow::builder()
            .title("Ver Todos os Perfis")
            .subtitle("Listar todos os perfis UFW disponiveis")
            .activatable(true)
            .build();

        let all_icon = Image::from_icon_name("view-more-symbolic");
        all_profiles_row.add_suffix(&all_icon);

        all_profiles_row.connect_activated(|_| {
            tracing::info!("Listing all UFW app profiles...");
            // UfwBackend::list_app_profiles();
        });

        profiles_group.add(&all_profiles_row);

        page.add(&profiles_group);

        // Per-Application Rules Group
        let app_rules_group = PreferencesGroup::builder()
            .title("Regras por Aplicativo")
            .description("Controle granular por aplicacao")
            .build();

        // Sample per-app rules
        let app_rules = [
            ("Firefox", true, true, "Navegador"),
            ("Chrome", true, true, "Navegador"),
            ("Spotify", true, false, "Streaming de musica"),
            ("Steam", true, true, "Jogos"),
            ("Transmission", false, true, "Cliente torrent"),
            ("Telegram", true, false, "Mensagens"),
        ];

        for (name, allow_out, allow_in, category) in app_rules {
            let row = ExpanderRow::builder()
                .title(name)
                .subtitle(category)
                .build();

            let app_icon = Image::from_icon_name("application-x-executable-symbolic");
            row.add_prefix(&app_icon);

            // Outgoing toggle
            let out_switch = SwitchRow::builder()
                .title("Permitir Saida")
                .subtitle("Conexoes de saida")
                .active(allow_out)
                .build();

            let name_clone = name.to_string();
            out_switch.connect_active_notify(move |switch| {
                let allowed = switch.is_active();
                tracing::info!("{} outgoing: {}", name_clone, allowed);
            });
            row.add_row(&out_switch);

            // Incoming toggle
            let in_switch = SwitchRow::builder()
                .title("Permitir Entrada")
                .subtitle("Conexoes de entrada")
                .active(allow_in)
                .build();

            let name_clone = name.to_string();
            in_switch.connect_active_notify(move |switch| {
                let allowed = switch.is_active();
                tracing::info!("{} incoming: {}", name_clone, allowed);
            });
            row.add_row(&in_switch);

            // Specific ports
            let ports_row = ActionRow::builder()
                .title("Portas Especificas")
                .subtitle("Configurar portas permitidas")
                .activatable(true)
                .build();

            let ports_icon = Image::from_icon_name("go-next-symbolic");
            ports_row.add_suffix(&ports_icon);

            row.add_row(&ports_row);

            app_rules_group.add(&row);
        }

        // Add new app rule
        let add_app_row = ActionRow::builder()
            .title("Adicionar Aplicativo")
            .subtitle("Criar regra para novo aplicativo")
            .activatable(true)
            .build();

        let add_icon = Image::from_icon_name("list-add-symbolic");
        add_app_row.add_prefix(&add_icon);

        add_app_row.connect_activated(|_| {
            tracing::info!("Adding new app rule...");
        });

        app_rules_group.add(&add_app_row);

        page.add(&app_rules_group);

        // Network Usage Group
        let usage_group = PreferencesGroup::builder()
            .title("Uso de Rede por Aplicativo")
            .description("Estatisticas de trafego")
            .build();

        let usage_apps = [
            ("Firefox", "1.2 GB", "450 MB"),
            ("Spotify", "890 MB", "12 MB"),
            ("VS Code", "234 MB", "89 MB"),
            ("System Updates", "2.1 GB", "45 KB"),
            ("Discord", "156 MB", "78 MB"),
        ];

        for (name, download, upload) in usage_apps {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("Download: {} | Upload: {}", download, upload))
                .build();

            let stats_box = Box::new(Orientation::Vertical, 2);

            let down_label = Label::new(Some(&format!("v {}", download)));
            down_label.add_css_class("success");
            down_label.add_css_class("caption");

            let up_label = Label::new(Some(&format!("^ {}", upload)));
            up_label.add_css_class("warning");
            up_label.add_css_class("caption");

            stats_box.append(&down_label);
            stats_box.append(&up_label);

            row.add_suffix(&stats_box);

            usage_group.add(&row);
        }

        page.add(&usage_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}

impl Default for AppsPage {
    fn default() -> Self {
        Self::new()
    }
}
