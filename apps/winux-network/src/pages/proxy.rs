//! Proxy settings page
//!
//! Features:
//! - System proxy configuration
//! - PAC URL support
//! - Manual proxy configuration
//! - Per-protocol proxy settings

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ScrolledWindow, TextView};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, EntryRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow};

/// Proxy page
pub struct ProxyPage {
    widget: ScrolledWindow,
}

impl ProxyPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Proxy");
        page.set_icon_name(Some("preferences-system-network-proxy-symbolic"));

        // Proxy mode group
        let mode_group = PreferencesGroup::builder()
            .title("Modo de Proxy")
            .description("Selecione como o proxy sera configurado")
            .build();

        let mode_row = ComboRow::builder()
            .title("Metodo de Configuracao")
            .subtitle("Como obter configuracoes de proxy")
            .build();
        let modes = gtk4::StringList::new(&[
            "Nenhum (Conexao Direta)",
            "Automatico (WPAD/PAC)",
            "Manual",
        ]);
        mode_row.set_model(Some(&modes));
        mode_group.add(&mode_row);

        page.add(&mode_group);

        // Automatic proxy group
        let auto_group = PreferencesGroup::builder()
            .title("Configuracao Automatica")
            .description("Usar arquivo de auto-configuracao (PAC)")
            .build();

        let pac_entry = EntryRow::builder()
            .title("URL do PAC")
            .text("http://proxy.example.com/proxy.pac")
            .build();
        auto_group.add(&pac_entry);

        let wpad_switch = SwitchRow::builder()
            .title("WPAD")
            .subtitle("Descoberta automatica de proxy (Web Proxy Auto-Discovery)")
            .active(false)
            .build();
        auto_group.add(&wpad_switch);

        page.add(&auto_group);

        // Manual proxy group
        let manual_group = PreferencesGroup::builder()
            .title("Configuracao Manual")
            .description("Configure o proxy para cada protocolo")
            .build();

        // Use same proxy for all
        let same_proxy = SwitchRow::builder()
            .title("Usar mesmo proxy para todos")
            .subtitle("Aplicar configuracao HTTP para todos os protocolos")
            .active(true)
            .build();
        manual_group.add(&same_proxy);

        // HTTP Proxy
        let http_expander = ExpanderRow::builder()
            .title("HTTP Proxy")
            .subtitle("Proxy para trafego HTTP")
            .build();

        let http_host = EntryRow::builder()
            .title("Servidor")
            .text("proxy.example.com")
            .build();
        http_expander.add_row(&http_host);

        let http_port = EntryRow::builder()
            .title("Porta")
            .text("8080")
            .build();
        http_expander.add_row(&http_port);

        manual_group.add(&http_expander);

        // HTTPS Proxy
        let https_expander = ExpanderRow::builder()
            .title("HTTPS Proxy")
            .subtitle("Proxy para trafego HTTPS/SSL")
            .build();

        let https_host = EntryRow::builder()
            .title("Servidor")
            .text("proxy.example.com")
            .build();
        https_expander.add_row(&https_host);

        let https_port = EntryRow::builder()
            .title("Porta")
            .text("8080")
            .build();
        https_expander.add_row(&https_port);

        manual_group.add(&https_expander);

        // FTP Proxy
        let ftp_expander = ExpanderRow::builder()
            .title("FTP Proxy")
            .subtitle("Proxy para trafego FTP")
            .build();

        let ftp_host = EntryRow::builder()
            .title("Servidor")
            .build();
        ftp_expander.add_row(&ftp_host);

        let ftp_port = EntryRow::builder()
            .title("Porta")
            .text("21")
            .build();
        ftp_expander.add_row(&ftp_port);

        manual_group.add(&ftp_expander);

        // SOCKS Proxy
        let socks_expander = ExpanderRow::builder()
            .title("SOCKS Proxy")
            .subtitle("Proxy SOCKS4/SOCKS5")
            .build();

        let socks_version = ComboRow::builder()
            .title("Versao")
            .build();
        let versions = gtk4::StringList::new(&["SOCKS4", "SOCKS5"]);
        socks_version.set_model(Some(&versions));
        socks_version.set_selected(1);
        socks_expander.add_row(&socks_version);

        let socks_host = EntryRow::builder()
            .title("Servidor")
            .build();
        socks_expander.add_row(&socks_host);

        let socks_port = EntryRow::builder()
            .title("Porta")
            .text("1080")
            .build();
        socks_expander.add_row(&socks_port);

        manual_group.add(&socks_expander);

        page.add(&manual_group);

        // Authentication group
        let auth_group = PreferencesGroup::builder()
            .title("Autenticacao")
            .description("Credenciais para o proxy")
            .build();

        let auth_switch = SwitchRow::builder()
            .title("Requer Autenticacao")
            .subtitle("O proxy requer usuario e senha")
            .active(false)
            .build();
        auth_group.add(&auth_switch);

        let username_entry = EntryRow::builder()
            .title("Usuario")
            .build();
        auth_group.add(&username_entry);

        let password_entry = adw::PasswordEntryRow::builder()
            .title("Senha")
            .build();
        auth_group.add(&password_entry);

        page.add(&auth_group);

        // Bypass list group
        let bypass_group = PreferencesGroup::builder()
            .title("Excecoes")
            .description("Enderecos que nao usarao o proxy")
            .build();

        let bypass_row = ActionRow::builder()
            .title("Lista de Excecoes")
            .subtitle("localhost, 127.0.0.0/8, ::1")
            .build();
        bypass_group.add(&bypass_row);

        let local_switch = SwitchRow::builder()
            .title("Ignorar para Enderecos Locais")
            .subtitle("Nao usar proxy para rede local")
            .active(true)
            .build();
        bypass_group.add(&local_switch);

        // Edit bypass list expander
        let bypass_expander = ExpanderRow::builder()
            .title("Editar Excecoes")
            .subtitle("Adicionar ou remover enderecos")
            .build();

        let bypass_entry = EntryRow::builder()
            .title("Adicionar Endereco")
            .build();
        bypass_expander.add_row(&bypass_entry);

        let add_bypass_row = ActionRow::builder().build();
        let add_bypass_btn = Button::with_label("Adicionar");
        add_bypass_btn.add_css_class("suggested-action");
        add_bypass_btn.set_halign(gtk4::Align::Center);
        add_bypass_btn.set_margin_top(4);
        add_bypass_btn.set_margin_bottom(4);
        add_bypass_row.set_child(Some(&add_bypass_btn));
        bypass_expander.add_row(&add_bypass_row);

        // Current bypass entries
        let bypasses = ["localhost", "127.0.0.0/8", "::1", "*.local", "192.168.0.0/16"];
        for addr in bypasses {
            let entry_row = ActionRow::builder()
                .title(addr)
                .build();

            let remove_btn = Button::from_icon_name("list-remove-symbolic");
            remove_btn.add_css_class("flat");
            remove_btn.set_valign(gtk4::Align::Center);

            let addr_clone = addr.to_string();
            remove_btn.connect_clicked(move |_| {
                tracing::info!("Removing bypass: {}", addr_clone);
            });
            entry_row.add_suffix(&remove_btn);

            bypass_expander.add_row(&entry_row);
        }

        bypass_group.add(&bypass_expander);

        page.add(&bypass_group);

        // Environment variables group
        let env_group = PreferencesGroup::builder()
            .title("Variaveis de Ambiente")
            .description("Configurar proxy via variaveis de ambiente")
            .build();

        let env_switch = SwitchRow::builder()
            .title("Definir Variaveis de Ambiente")
            .subtitle("Exportar HTTP_PROXY, HTTPS_PROXY, etc.")
            .active(true)
            .build();
        env_group.add(&env_switch);

        let env_details = ActionRow::builder()
            .title("Variaveis Ativas")
            .subtitle("HTTP_PROXY, HTTPS_PROXY, NO_PROXY")
            .build();
        env_details.add_prefix(&Image::from_icon_name("utilities-terminal-symbolic"));
        env_group.add(&env_details);

        page.add(&env_group);

        // Apply button
        let apply_group = PreferencesGroup::new();
        let apply_row = ActionRow::builder().build();

        let button_box = Box::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(8);
        button_box.set_margin_bottom(8);

        let test_btn = Button::with_label("Testar Conexao");
        test_btn.connect_clicked(|_| {
            tracing::info!("Testing proxy connection...");
        });
        button_box.append(&test_btn);

        let apply_btn = Button::with_label("Aplicar");
        apply_btn.add_css_class("suggested-action");
        apply_btn.connect_clicked(|_| {
            tracing::info!("Applying proxy settings...");
        });
        button_box.append(&apply_btn);

        apply_row.set_child(Some(&button_box));
        apply_group.add(&apply_row);

        page.add(&apply_group);

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

impl Default for ProxyPage {
    fn default() -> Self {
        Self::new()
    }
}
