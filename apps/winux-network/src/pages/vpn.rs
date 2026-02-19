//! VPN connections page
//!
//! Features:
//! - Import OpenVPN/WireGuard configs
//! - Add VPN manually
//! - Connect/disconnect
//! - VPN status monitoring

use gtk4::prelude::*;
use gtk4::{Box, Button, FileDialog, Image, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, EntryRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow, PasswordEntryRow};

/// VPN page
pub struct VpnPage {
    widget: ScrolledWindow,
}

impl VpnPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("VPN");
        page.set_icon_name(Some("network-vpn-symbolic"));

        // Active VPN group
        let active_group = PreferencesGroup::builder()
            .title("VPN Ativa")
            .build();

        let no_vpn_row = ActionRow::builder()
            .title("Nenhuma VPN conectada")
            .subtitle("Conecte a uma VPN para proteger sua conexao")
            .build();

        let vpn_icon = Image::from_icon_name("network-vpn-symbolic");
        vpn_icon.add_css_class("dim-label");
        no_vpn_row.add_prefix(&vpn_icon);

        active_group.add(&no_vpn_row);
        page.add(&active_group);

        // Saved VPNs
        let saved_group = PreferencesGroup::builder()
            .title("VPNs Salvas")
            .description("Conexoes VPN configuradas")
            .build();

        let vpns = [
            ("Trabalho VPN", "OpenVPN", false),
            ("Casa WireGuard", "WireGuard", false),
            ("ProtonVPN", "OpenVPN", false),
        ];

        for (name, vpn_type, connected) in vpns {
            let row = ExpanderRow::builder()
                .title(name)
                .subtitle(vpn_type)
                .build();

            let type_icon = match vpn_type {
                "WireGuard" => "network-vpn-symbolic",
                _ => "network-vpn-symbolic",
            };
            row.add_prefix(&Image::from_icon_name(type_icon));

            // Connect switch
            let connect_switch = gtk4::Switch::new();
            connect_switch.set_active(connected);
            connect_switch.set_valign(gtk4::Align::Center);

            let vpn_name = name.to_string();
            connect_switch.connect_state_set(move |_, state| {
                if state {
                    tracing::info!("Connecting to VPN: {}", vpn_name);
                } else {
                    tracing::info!("Disconnecting from VPN: {}", vpn_name);
                }
                glib::Propagation::Proceed
            });
            row.add_suffix(&connect_switch);

            // VPN details
            let auto_connect = SwitchRow::builder()
                .title("Conectar Automaticamente")
                .active(false)
                .build();
            row.add_row(&auto_connect);

            let details_row = ActionRow::builder()
                .title("Ver Detalhes")
                .activatable(true)
                .build();
            details_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
            row.add_row(&details_row);

            let delete_row = ActionRow::builder()
                .title("Remover VPN")
                .activatable(true)
                .build();
            delete_row.add_css_class("error");
            delete_row.add_prefix(&Image::from_icon_name("user-trash-symbolic"));
            row.add_row(&delete_row);

            saved_group.add(&row);
        }

        page.add(&saved_group);

        // Import VPN group
        let import_group = PreferencesGroup::builder()
            .title("Importar VPN")
            .description("Importar arquivo de configuracao")
            .build();

        // Import OpenVPN
        let openvpn_row = ActionRow::builder()
            .title("Importar OpenVPN (.ovpn)")
            .subtitle("Importar arquivo de configuracao OpenVPN")
            .activatable(true)
            .build();

        openvpn_row.add_prefix(&Image::from_icon_name("document-open-symbolic"));
        openvpn_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));

        openvpn_row.connect_activated(|row| {
            let dialog = FileDialog::builder()
                .title("Importar OpenVPN")
                .modal(true)
                .build();

            let filter = gtk4::FileFilter::new();
            filter.add_pattern("*.ovpn");
            filter.set_name(Some("OpenVPN Config"));

            let filters = gio::ListStore::new::<gtk4::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            if let Some(window) = row.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                dialog.open(Some(&window), gio::Cancellable::NONE, |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            tracing::info!("Importing OpenVPN config: {:?}", path);
                        }
                    }
                });
            }
        });

        import_group.add(&openvpn_row);

        // Import WireGuard
        let wireguard_row = ActionRow::builder()
            .title("Importar WireGuard (.conf)")
            .subtitle("Importar arquivo de configuracao WireGuard")
            .activatable(true)
            .build();

        wireguard_row.add_prefix(&Image::from_icon_name("document-open-symbolic"));
        wireguard_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));

        wireguard_row.connect_activated(|row| {
            let dialog = FileDialog::builder()
                .title("Importar WireGuard")
                .modal(true)
                .build();

            let filter = gtk4::FileFilter::new();
            filter.add_pattern("*.conf");
            filter.set_name(Some("WireGuard Config"));

            let filters = gio::ListStore::new::<gtk4::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            if let Some(window) = row.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                dialog.open(Some(&window), gio::Cancellable::NONE, |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            tracing::info!("Importing WireGuard config: {:?}", path);
                        }
                    }
                });
            }
        });

        import_group.add(&wireguard_row);

        page.add(&import_group);

        // Add VPN manually
        let manual_group = PreferencesGroup::builder()
            .title("Adicionar VPN Manualmente")
            .build();

        // OpenVPN manual
        let openvpn_expander = ExpanderRow::builder()
            .title("OpenVPN")
            .subtitle("Configurar conexao OpenVPN")
            .build();

        let ovpn_name = EntryRow::builder()
            .title("Nome da Conexao")
            .build();
        openvpn_expander.add_row(&ovpn_name);

        let ovpn_server = EntryRow::builder()
            .title("Servidor")
            .text("vpn.example.com")
            .build();
        openvpn_expander.add_row(&ovpn_server);

        let ovpn_port = EntryRow::builder()
            .title("Porta")
            .text("1194")
            .build();
        openvpn_expander.add_row(&ovpn_port);

        let ovpn_protocol = ComboRow::builder()
            .title("Protocolo")
            .build();
        let protocols = gtk4::StringList::new(&["UDP", "TCP"]);
        ovpn_protocol.set_model(Some(&protocols));
        openvpn_expander.add_row(&ovpn_protocol);

        let ovpn_auth = ComboRow::builder()
            .title("Autenticacao")
            .build();
        let auth_types = gtk4::StringList::new(&["Certificado", "Usuario/Senha", "Certificado + Senha"]);
        ovpn_auth.set_model(Some(&auth_types));
        openvpn_expander.add_row(&ovpn_auth);

        let ovpn_user = EntryRow::builder()
            .title("Usuario")
            .build();
        openvpn_expander.add_row(&ovpn_user);

        let ovpn_pass = PasswordEntryRow::builder()
            .title("Senha")
            .build();
        openvpn_expander.add_row(&ovpn_pass);

        let save_ovpn_row = ActionRow::builder().build();
        let save_ovpn_btn = Button::with_label("Salvar OpenVPN");
        save_ovpn_btn.add_css_class("suggested-action");
        save_ovpn_btn.set_halign(gtk4::Align::Center);
        save_ovpn_btn.set_margin_top(8);
        save_ovpn_btn.set_margin_bottom(8);
        save_ovpn_row.set_child(Some(&save_ovpn_btn));
        openvpn_expander.add_row(&save_ovpn_row);

        manual_group.add(&openvpn_expander);

        // WireGuard manual
        let wg_expander = ExpanderRow::builder()
            .title("WireGuard")
            .subtitle("Configurar conexao WireGuard")
            .build();

        let wg_name = EntryRow::builder()
            .title("Nome da Conexao")
            .build();
        wg_expander.add_row(&wg_name);

        let wg_private = PasswordEntryRow::builder()
            .title("Chave Privada")
            .build();
        wg_expander.add_row(&wg_private);

        let wg_address = EntryRow::builder()
            .title("Endereco")
            .text("10.0.0.2/24")
            .build();
        wg_expander.add_row(&wg_address);

        let wg_dns = EntryRow::builder()
            .title("DNS")
            .text("1.1.1.1")
            .build();
        wg_expander.add_row(&wg_dns);

        // Peer section
        let peer_label = ActionRow::builder()
            .title("Peer")
            .build();
        peer_label.add_css_class("header");
        wg_expander.add_row(&peer_label);

        let wg_pubkey = EntryRow::builder()
            .title("Chave Publica do Peer")
            .build();
        wg_expander.add_row(&wg_pubkey);

        let wg_endpoint = EntryRow::builder()
            .title("Endpoint")
            .text("vpn.example.com:51820")
            .build();
        wg_expander.add_row(&wg_endpoint);

        let wg_allowed = EntryRow::builder()
            .title("IPs Permitidos")
            .text("0.0.0.0/0")
            .build();
        wg_expander.add_row(&wg_allowed);

        let wg_keepalive = EntryRow::builder()
            .title("Keepalive")
            .text("25")
            .build();
        wg_expander.add_row(&wg_keepalive);

        let save_wg_row = ActionRow::builder().build();
        let save_wg_btn = Button::with_label("Salvar WireGuard");
        save_wg_btn.add_css_class("suggested-action");
        save_wg_btn.set_halign(gtk4::Align::Center);
        save_wg_btn.set_margin_top(8);
        save_wg_btn.set_margin_bottom(8);
        save_wg_row.set_child(Some(&save_wg_btn));
        wg_expander.add_row(&save_wg_row);

        manual_group.add(&wg_expander);

        page.add(&manual_group);

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

impl Default for VpnPage {
    fn default() -> Self {
        Self::new()
    }
}
