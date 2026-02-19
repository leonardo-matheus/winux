//! Hotspot/Access Point page
//!
//! Features:
//! - Create WiFi hotspot
//! - Configure SSID and password
//! - View connected devices
//! - Bandwidth monitoring

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ProgressBar, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, EntryRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow, PasswordEntryRow};

/// Hotspot page
pub struct HotspotPage {
    widget: ScrolledWindow,
}

impl HotspotPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Hotspot");
        page.set_icon_name(Some("network-wireless-hotspot-symbolic"));

        // Hotspot toggle group
        let toggle_group = PreferencesGroup::builder()
            .title("Ponto de Acesso Wi-Fi")
            .description("Compartilhe sua conexao de internet")
            .build();

        let hotspot_switch = SwitchRow::builder()
            .title("Hotspot")
            .subtitle("Desativado")
            .active(false)
            .build();

        hotspot_switch.connect_active_notify(|switch| {
            if switch.is_active() {
                switch.set_subtitle("Ativo - Compartilhando internet");
                tracing::info!("Hotspot enabled");
            } else {
                switch.set_subtitle("Desativado");
                tracing::info!("Hotspot disabled");
            }
        });

        toggle_group.add(&hotspot_switch);
        page.add(&toggle_group);

        // Configuration group
        let config_group = PreferencesGroup::builder()
            .title("Configuracao")
            .description("Configure nome e senha do hotspot")
            .build();

        let ssid_entry = EntryRow::builder()
            .title("Nome da Rede (SSID)")
            .text("Winux-Hotspot")
            .build();
        config_group.add(&ssid_entry);

        let password_entry = PasswordEntryRow::builder()
            .title("Senha")
            .text("winux2026")
            .build();
        config_group.add(&password_entry);

        // Security type
        let security_row = ComboRow::builder()
            .title("Seguranca")
            .subtitle("Tipo de criptografia")
            .build();
        let security_types = gtk4::StringList::new(&["WPA2/WPA3", "WPA2", "WPA3", "Nenhuma (Aberta)"]);
        security_row.set_model(Some(&security_types));
        config_group.add(&security_row);

        // Band selection
        let band_row = ComboRow::builder()
            .title("Banda")
            .subtitle("Frequencia do Wi-Fi")
            .build();
        let bands = gtk4::StringList::new(&["2.4 GHz", "5 GHz", "Automatico"]);
        band_row.set_model(Some(&bands));
        band_row.set_selected(2);
        config_group.add(&band_row);

        // Channel selection
        let channel_row = ComboRow::builder()
            .title("Canal")
            .build();
        let channels = gtk4::StringList::new(&[
            "Automatico", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"
        ]);
        channel_row.set_model(Some(&channels));
        config_group.add(&channel_row);

        // Max clients
        let max_clients = adw::SpinRow::builder()
            .title("Clientes Maximos")
            .subtitle("Limite de dispositivos conectados")
            .adjustment(&gtk4::Adjustment::new(10.0, 1.0, 50.0, 1.0, 5.0, 0.0))
            .build();
        config_group.add(&max_clients);

        page.add(&config_group);

        // Internet sharing
        let sharing_group = PreferencesGroup::builder()
            .title("Compartilhamento de Internet")
            .description("Selecione a conexao a compartilhar")
            .build();

        let share_row = ComboRow::builder()
            .title("Compartilhar de")
            .subtitle("Interface de origem")
            .build();
        let interfaces = gtk4::StringList::new(&["Ethernet (enp3s0)", "Wi-Fi (wlan0)", "USB Tethering"]);
        share_row.set_model(Some(&interfaces));
        sharing_group.add(&share_row);

        page.add(&sharing_group);

        // Connected devices group
        let devices_group = PreferencesGroup::builder()
            .title("Dispositivos Conectados")
            .description("0 dispositivos")
            .build();

        let no_devices_row = ActionRow::builder()
            .title("Nenhum dispositivo conectado")
            .subtitle("Inicie o hotspot para permitir conexoes")
            .build();
        no_devices_row.add_prefix(&Image::from_icon_name("computer-symbolic"));
        devices_group.add(&no_devices_row);

        // Sample connected devices (would be populated dynamically)
        let sample_devices = [
            ("iPhone de Joao", "192.168.43.100", "AA:BB:CC:11:22:33", "15 MB"),
            ("Galaxy S24", "192.168.43.101", "DD:EE:FF:44:55:66", "32 MB"),
        ];

        for (name, ip, mac, data) in sample_devices {
            let device_row = ExpanderRow::builder()
                .title(name)
                .subtitle(ip)
                .build();

            device_row.add_prefix(&Image::from_icon_name("phone-symbolic"));

            let mac_row = ActionRow::builder()
                .title("MAC Address")
                .subtitle(mac)
                .build();
            device_row.add_row(&mac_row);

            let data_row = ActionRow::builder()
                .title("Dados Usados")
                .subtitle(data)
                .build();
            device_row.add_row(&data_row);

            let block_row = ActionRow::builder()
                .title("Bloquear Dispositivo")
                .activatable(true)
                .build();
            block_row.add_css_class("error");
            block_row.add_prefix(&Image::from_icon_name("action-unavailable-symbolic"));

            let device_name = name.to_string();
            block_row.connect_activated(move |_| {
                tracing::info!("Blocking device: {}", device_name);
            });
            device_row.add_row(&block_row);

            // Hidden by default - would be shown when hotspot is active
            // devices_group.add(&device_row);
        }

        page.add(&devices_group);

        // Statistics group
        let stats_group = PreferencesGroup::builder()
            .title("Estatisticas")
            .build();

        let upload_row = ActionRow::builder()
            .title("Upload Total")
            .subtitle("0 MB")
            .build();
        upload_row.add_prefix(&Image::from_icon_name("go-up-symbolic"));
        stats_group.add(&upload_row);

        let download_row = ActionRow::builder()
            .title("Download Total")
            .subtitle("0 MB")
            .build();
        download_row.add_prefix(&Image::from_icon_name("go-down-symbolic"));
        stats_group.add(&download_row);

        let time_row = ActionRow::builder()
            .title("Tempo Ativo")
            .subtitle("--:--:--")
            .build();
        time_row.add_prefix(&Image::from_icon_name("preferences-system-time-symbolic"));
        stats_group.add(&time_row);

        page.add(&stats_group);

        // QR Code group
        let qr_group = PreferencesGroup::builder()
            .title("Compartilhar Conexao")
            .description("Gere um QR Code para facilitar a conexao")
            .build();

        let qr_row = ActionRow::builder()
            .title("Gerar QR Code")
            .subtitle("Escaneie para conectar automaticamente")
            .activatable(true)
            .build();

        qr_row.add_prefix(&Image::from_icon_name("view-grid-symbolic"));
        qr_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));

        qr_row.connect_activated(|_| {
            tracing::info!("Generating QR Code for hotspot...");
            // Would show QR code dialog
        });

        qr_group.add(&qr_row);

        page.add(&qr_group);

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

impl Default for HotspotPage {
    fn default() -> Self {
        Self::new()
    }
}
