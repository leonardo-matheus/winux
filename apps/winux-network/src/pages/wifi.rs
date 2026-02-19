//! WiFi networks page
//!
//! Features:
//! - List available networks
//! - Signal strength indicator
//! - Connect/disconnect
//! - Password configuration
//! - Known networks management
//! - Hidden network support

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Image, Label, ListBox, Orientation, ScrolledWindow, Spinner};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow, EntryRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::{NetworkRow, PasswordDialog};

/// WiFi networks page
pub struct WifiPage {
    widget: ScrolledWindow,
}

impl WifiPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Wi-Fi");
        page.set_icon_name(Some("network-wireless-symbolic"));

        // WiFi toggle group
        let toggle_group = PreferencesGroup::builder()
            .title("Wi-Fi")
            .description("Gerencie conexoes sem fio")
            .build();

        let wifi_switch = SwitchRow::builder()
            .title("Wi-Fi")
            .subtitle("Habilitar interface wireless")
            .active(true)
            .build();
        toggle_group.add(&wifi_switch);

        page.add(&toggle_group);

        // Current connection group
        let current_group = PreferencesGroup::builder()
            .title("Conexao Atual")
            .build();

        let connected_row = ActionRow::builder()
            .title("Casa_5G")
            .subtitle("Conectado - Sinal excelente")
            .build();

        let signal_icon = Image::from_icon_name("network-wireless-signal-excellent-symbolic");
        signal_icon.add_css_class("success");
        connected_row.add_prefix(&signal_icon);

        let secured_icon = Image::from_icon_name("network-wireless-encrypted-symbolic");
        connected_row.add_suffix(&secured_icon);

        let disconnect_btn = Button::with_label("Desconectar");
        disconnect_btn.add_css_class("destructive-action");
        disconnect_btn.set_valign(gtk4::Align::Center);
        disconnect_btn.connect_clicked(|_| {
            tracing::info!("Disconnecting from WiFi...");
        });
        connected_row.add_suffix(&disconnect_btn);

        current_group.add(&connected_row);

        // Connection details expander
        let details_expander = ExpanderRow::builder()
            .title("Detalhes da Conexao")
            .subtitle("IP, MAC, velocidade")
            .build();

        let ip_row = ActionRow::builder()
            .title("Endereco IP")
            .subtitle("192.168.1.100")
            .build();
        details_expander.add_row(&ip_row);

        let mac_row = ActionRow::builder()
            .title("MAC Address")
            .subtitle("AA:BB:CC:DD:EE:FF")
            .build();
        details_expander.add_row(&mac_row);

        let speed_row = ActionRow::builder()
            .title("Velocidade")
            .subtitle("866 Mbps")
            .build();
        details_expander.add_row(&speed_row);

        let freq_row = ActionRow::builder()
            .title("Frequencia")
            .subtitle("5 GHz")
            .build();
        details_expander.add_row(&freq_row);

        let security_row = ActionRow::builder()
            .title("Seguranca")
            .subtitle("WPA2/WPA3")
            .build();
        details_expander.add_row(&security_row);

        current_group.add(&details_expander);
        page.add(&current_group);

        // Available networks group
        let available_group = PreferencesGroup::builder()
            .title("Redes Disponiveis")
            .description("Clique para conectar")
            .build();

        // Scan button row
        let scan_row = ActionRow::builder()
            .title("Buscar Redes")
            .subtitle("Atualizar lista de redes")
            .activatable(true)
            .build();

        let scan_spinner = Spinner::new();
        scan_row.add_suffix(&scan_spinner);

        let refresh_icon = Image::from_icon_name("view-refresh-symbolic");
        scan_row.add_suffix(&refresh_icon);

        scan_row.connect_activated({
            let spinner = scan_spinner.clone();
            move |_| {
                spinner.start();
                tracing::info!("Scanning for WiFi networks...");
                // In real implementation, trigger NM scan here
                glib::timeout_add_seconds_local_once(2, {
                    let spinner = spinner.clone();
                    move || spinner.stop()
                });
            }
        });

        available_group.add(&scan_row);

        // Sample available networks
        let networks = [
            ("Vizinho_Net", "network-wireless-signal-good-symbolic", true, 75),
            ("Cafe_WiFi", "network-wireless-signal-ok-symbolic", false, 50),
            ("Escritorio", "network-wireless-signal-excellent-symbolic", true, 95),
            ("Guest_5G", "network-wireless-signal-weak-symbolic", true, 30),
            ("OpenNet", "network-wireless-signal-ok-symbolic", false, 60),
        ];

        for (ssid, icon, secured, _signal) in networks {
            let row = ActionRow::builder()
                .title(ssid)
                .activatable(true)
                .build();

            let signal_icon = Image::from_icon_name(icon);
            row.add_prefix(&signal_icon);

            if secured {
                let lock_icon = Image::from_icon_name("network-wireless-encrypted-symbolic");
                row.add_suffix(&lock_icon);
            }

            let connect_btn = Button::with_label("Conectar");
            connect_btn.add_css_class("flat");
            connect_btn.set_valign(gtk4::Align::Center);

            let ssid_clone = ssid.to_string();
            let secured_clone = secured;
            connect_btn.connect_clicked(move |btn| {
                if secured_clone {
                    tracing::info!("Opening password dialog for {}", ssid_clone);
                    // Show password dialog
                    if let Some(window) = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                        PasswordDialog::show(&window, &ssid_clone, |password| {
                            tracing::info!("Connecting to {} with password", ssid_clone);
                            // Connect via NetworkManager
                        });
                    }
                } else {
                    tracing::info!("Connecting to open network {}", ssid_clone);
                }
            });
            row.add_suffix(&connect_btn);

            available_group.add(&row);
        }

        page.add(&available_group);

        // Known networks group
        let known_group = PreferencesGroup::builder()
            .title("Redes Conhecidas")
            .description("Redes salvas")
            .build();

        let known_networks = [
            ("Casa_5G", true),
            ("Trabalho_WiFi", true),
            ("Aeroporto_Free", false),
        ];

        for (ssid, auto_connect) in known_networks {
            let row = ExpanderRow::builder()
                .title(ssid)
                .subtitle(if auto_connect { "Conectar automaticamente" } else { "Conexao manual" })
                .build();

            let auto_row = SwitchRow::builder()
                .title("Conectar Automaticamente")
                .active(auto_connect)
                .build();
            row.add_row(&auto_row);

            let forget_row = ActionRow::builder()
                .title("Esquecer Rede")
                .activatable(true)
                .build();
            forget_row.add_css_class("error");

            let forget_icon = Image::from_icon_name("user-trash-symbolic");
            forget_row.add_prefix(&forget_icon);

            let ssid_clone = ssid.to_string();
            forget_row.connect_activated(move |_| {
                tracing::info!("Forgetting network: {}", ssid_clone);
            });
            row.add_row(&forget_row);

            known_group.add(&row);
        }

        page.add(&known_group);

        // Hidden network group
        let hidden_group = PreferencesGroup::builder()
            .title("Rede Oculta")
            .description("Conectar a uma rede que nao aparece na lista")
            .build();

        let hidden_expander = ExpanderRow::builder()
            .title("Conectar a Rede Oculta")
            .subtitle("Digite o SSID manualmente")
            .build();

        let ssid_entry = EntryRow::builder()
            .title("Nome da Rede (SSID)")
            .build();
        hidden_expander.add_row(&ssid_entry);

        let security_row = adw::ComboRow::builder()
            .title("Seguranca")
            .build();
        let security_model = gtk4::StringList::new(&["Nenhuma", "WEP", "WPA/WPA2", "WPA3"]);
        security_row.set_model(Some(&security_model));
        security_row.set_selected(2);
        hidden_expander.add_row(&security_row);

        let password_entry = adw::PasswordEntryRow::builder()
            .title("Senha")
            .build();
        hidden_expander.add_row(&password_entry);

        let connect_hidden_row = ActionRow::builder()
            .build();

        let connect_hidden_btn = Button::with_label("Conectar");
        connect_hidden_btn.add_css_class("suggested-action");
        connect_hidden_btn.set_halign(gtk4::Align::Center);
        connect_hidden_btn.set_margin_top(8);
        connect_hidden_btn.set_margin_bottom(8);
        connect_hidden_row.set_child(Some(&connect_hidden_btn));
        hidden_expander.add_row(&connect_hidden_row);

        hidden_group.add(&hidden_expander);
        page.add(&hidden_group);

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

impl Default for WifiPage {
    fn default() -> Self {
        Self::new()
    }
}
