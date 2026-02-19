//! Ethernet connections page
//!
//! Features:
//! - Connection status
//! - IP configuration (DHCP or static)
//! - MAC address
//! - Speed and duplex info

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Image, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, EntryRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow};

/// Ethernet page
pub struct EthernetPage {
    widget: ScrolledWindow,
}

impl EthernetPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Ethernet");
        page.set_icon_name(Some("network-wired-symbolic"));

        // Connection status group
        let status_group = PreferencesGroup::builder()
            .title("Status da Conexao")
            .build();

        let status_row = ActionRow::builder()
            .title("Ethernet (enp3s0)")
            .subtitle("Conectado - 1 Gbps Full Duplex")
            .build();

        let status_icon = Image::from_icon_name("network-wired-symbolic");
        status_icon.add_css_class("success");
        status_row.add_prefix(&status_icon);

        let connected_icon = Image::from_icon_name("emblem-ok-symbolic");
        connected_icon.add_css_class("success");
        status_row.add_suffix(&connected_icon);

        status_group.add(&status_row);
        page.add(&status_group);

        // Connection details
        let details_group = PreferencesGroup::builder()
            .title("Detalhes da Conexao")
            .build();

        let details = [
            ("Endereco IP", "192.168.1.50"),
            ("Mascara de Rede", "255.255.255.0"),
            ("Gateway Padrao", "192.168.1.1"),
            ("DNS Primario", "8.8.8.8"),
            ("DNS Secundario", "8.8.4.4"),
            ("MAC Address", "00:1A:2B:3C:4D:5E"),
            ("Velocidade", "1000 Mbps"),
            ("Duplex", "Full"),
        ];

        for (label, value) in details {
            let row = ActionRow::builder()
                .title(label)
                .subtitle(value)
                .build();
            details_group.add(&row);
        }

        page.add(&details_group);

        // IP Configuration
        let ip_group = PreferencesGroup::builder()
            .title("Configuracao de IP")
            .description("Configure endereco IP e DNS")
            .build();

        let method_row = ComboRow::builder()
            .title("Metodo")
            .subtitle("Como obter endereco IP")
            .build();
        let methods = gtk4::StringList::new(&["DHCP (Automatico)", "Manual (Estatico)", "Link-Local"]);
        method_row.set_model(Some(&methods));
        ip_group.add(&method_row);

        // Manual IP configuration expander
        let manual_expander = ExpanderRow::builder()
            .title("Configuracao Manual")
            .subtitle("Defina IP estatico")
            .build();

        let ip_entry = EntryRow::builder()
            .title("Endereco IP")
            .text("192.168.1.50")
            .build();
        manual_expander.add_row(&ip_entry);

        let mask_entry = EntryRow::builder()
            .title("Mascara de Rede")
            .text("255.255.255.0")
            .build();
        manual_expander.add_row(&mask_entry);

        let gateway_entry = EntryRow::builder()
            .title("Gateway")
            .text("192.168.1.1")
            .build();
        manual_expander.add_row(&gateway_entry);

        ip_group.add(&manual_expander);

        // DNS Configuration
        let dns_expander = ExpanderRow::builder()
            .title("Servidores DNS")
            .subtitle("Configurar servidores de nome")
            .build();

        let auto_dns = SwitchRow::builder()
            .title("DNS Automatico")
            .subtitle("Usar DNS fornecido pelo DHCP")
            .active(true)
            .build();
        dns_expander.add_row(&auto_dns);

        let dns1_entry = EntryRow::builder()
            .title("DNS Primario")
            .text("8.8.8.8")
            .build();
        dns_expander.add_row(&dns1_entry);

        let dns2_entry = EntryRow::builder()
            .title("DNS Secundario")
            .text("8.8.4.4")
            .build();
        dns_expander.add_row(&dns2_entry);

        ip_group.add(&dns_expander);

        // Apply button
        let apply_row = ActionRow::builder().build();
        let apply_btn = Button::with_label("Aplicar Configuracoes");
        apply_btn.add_css_class("suggested-action");
        apply_btn.set_halign(gtk4::Align::Center);
        apply_btn.set_margin_top(8);
        apply_btn.set_margin_bottom(8);
        apply_btn.connect_clicked(|_| {
            tracing::info!("Applying ethernet configuration...");
        });
        apply_row.set_child(Some(&apply_btn));
        ip_group.add(&apply_row);

        page.add(&ip_group);

        // Additional interfaces
        let interfaces_group = PreferencesGroup::builder()
            .title("Outras Interfaces")
            .description("Interfaces de rede adicionais")
            .build();

        let interfaces = [
            ("enp4s0", "Desconectado", "network-wired-disconnected-symbolic"),
            ("docker0", "172.17.0.1", "network-wired-symbolic"),
            ("virbr0", "192.168.122.1", "network-wired-symbolic"),
        ];

        for (name, status, icon) in interfaces {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(status)
                .build();

            let if_icon = Image::from_icon_name(icon);
            row.add_prefix(&if_icon);

            let settings_btn = Button::from_icon_name("emblem-system-symbolic");
            settings_btn.add_css_class("flat");
            settings_btn.set_valign(gtk4::Align::Center);
            settings_btn.set_tooltip_text(Some("Configurar interface"));
            row.add_suffix(&settings_btn);

            interfaces_group.add(&row);
        }

        page.add(&interfaces_group);

        // Wake on LAN
        let wol_group = PreferencesGroup::builder()
            .title("Wake on LAN")
            .description("Ligar computador pela rede")
            .build();

        let wol_switch = SwitchRow::builder()
            .title("Wake on LAN")
            .subtitle("Permitir ligar pela rede")
            .active(false)
            .build();
        wol_group.add(&wol_switch);

        let wol_magic = SwitchRow::builder()
            .title("Magic Packet")
            .subtitle("Requer pacote magico para acordar")
            .active(true)
            .build();
        wol_group.add(&wol_magic);

        page.add(&wol_group);

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

impl Default for EthernetPage {
    fn default() -> Self {
        Self::new()
    }
}
