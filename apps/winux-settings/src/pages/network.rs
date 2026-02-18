//! Network settings page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::info;

/// Network settings page
pub struct NetworkPage {
    widget: adw::PreferencesPage,
}

impl NetworkPage {
    /// Create a new network settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Network");
        page.set_icon_name(Some("network-wireless-symbolic"));

        // Wi-Fi group
        let wifi_group = adw::PreferencesGroup::new();
        wifi_group.set_title("Wi-Fi");

        // Wi-Fi toggle
        let wifi_toggle = adw::SwitchRow::new();
        wifi_toggle.set_title("Wi-Fi");
        wifi_toggle.set_subtitle("Enable wireless networking");
        wifi_toggle.set_active(true);
        wifi_group.add(&wifi_toggle);

        // Connected network
        let connected_row = adw::ActionRow::new();
        connected_row.set_title("MyNetwork");
        connected_row.set_subtitle("Connected - Excellent signal");
        connected_row.add_prefix(&gtk4::Image::from_icon_name("network-wireless-signal-excellent-symbolic"));

        let disconnect_btn = gtk4::Button::with_label("Disconnect");
        connected_row.add_suffix(&disconnect_btn);
        wifi_group.add(&connected_row);

        // Available networks header
        let networks_label = adw::ActionRow::new();
        networks_label.set_title("Available Networks");
        networks_label.add_css_class("header");
        wifi_group.add(&networks_label);

        // Sample networks
        let networks = [
            ("HomeNetwork", "network-wireless-signal-good-symbolic", true),
            ("OfficeWiFi", "network-wireless-signal-ok-symbolic", true),
            ("CafeHotspot", "network-wireless-signal-weak-symbolic", false),
            ("Guest Network", "network-wireless-signal-ok-symbolic", false),
        ];

        for (name, icon, secured) in networks {
            let row = adw::ActionRow::new();
            row.set_title(name);
            row.set_activatable(true);

            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            if secured {
                row.add_suffix(&gtk4::Image::from_icon_name("network-wireless-encrypted-symbolic"));
            }

            let connect_btn = gtk4::Button::with_label("Connect");
            connect_btn.add_css_class("flat");
            row.add_suffix(&connect_btn);

            wifi_group.add(&row);
        }

        // Scan button
        let scan_row = adw::ActionRow::new();
        let scan_btn = gtk4::Button::with_label("Scan for Networks");
        scan_btn.set_halign(gtk4::Align::Center);
        scan_row.set_child(Some(&scan_btn));
        wifi_group.add(&scan_row);

        page.add(&wifi_group);

        // Wired group
        let wired_group = adw::PreferencesGroup::new();
        wired_group.set_title("Wired");

        // Ethernet status
        let ethernet_row = adw::ActionRow::new();
        ethernet_row.set_title("Ethernet");
        ethernet_row.set_subtitle("Connected - 1 Gbps");
        ethernet_row.add_prefix(&gtk4::Image::from_icon_name("network-wired-symbolic"));

        let details_btn = gtk4::Button::from_icon_name("emblem-system-symbolic");
        details_btn.set_tooltip_text(Some("Connection details"));
        ethernet_row.add_suffix(&details_btn);
        wired_group.add(&ethernet_row);

        page.add(&wired_group);

        // VPN group
        let vpn_group = adw::PreferencesGroup::new();
        vpn_group.set_title("VPN");

        // VPN toggle
        let vpn_toggle = adw::SwitchRow::new();
        vpn_toggle.set_title("VPN");
        vpn_toggle.set_subtitle("Virtual Private Network");
        vpn_group.add(&vpn_toggle);

        // Add VPN button
        let add_vpn_row = adw::ActionRow::new();
        add_vpn_row.set_title("Add VPN");
        add_vpn_row.set_activatable(true);
        add_vpn_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
        add_vpn_row.connect_activated(|_| {
            info!("Add VPN clicked");
        });
        vpn_group.add(&add_vpn_row);

        page.add(&vpn_group);

        // Proxy group
        let proxy_group = adw::PreferencesGroup::new();
        proxy_group.set_title("Proxy");

        // Proxy method
        let proxy_method = adw::ComboRow::new();
        proxy_method.set_title("Proxy Method");
        let methods = gtk4::StringList::new(&[
            "None",
            "Automatic",
            "Manual",
        ]);
        proxy_method.set_model(Some(&methods));
        proxy_group.add(&proxy_method);

        page.add(&proxy_group);

        // Advanced group
        let advanced_group = adw::PreferencesGroup::new();
        advanced_group.set_title("Advanced");

        // Firewall
        let firewall_row = adw::SwitchRow::new();
        firewall_row.set_title("Firewall");
        firewall_row.set_subtitle("Protect your computer from unauthorized access");
        firewall_row.set_active(true);
        advanced_group.add(&firewall_row);

        // Network sharing
        let sharing_row = adw::ActionRow::new();
        sharing_row.set_title("Network Sharing");
        sharing_row.set_subtitle("Share files and media on the local network");
        sharing_row.set_activatable(true);
        sharing_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        advanced_group.add(&sharing_row);

        // DNS settings
        let dns_row = adw::ActionRow::new();
        dns_row.set_title("DNS Settings");
        dns_row.set_subtitle("Configure custom DNS servers");
        dns_row.set_activatable(true);
        dns_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        advanced_group.add(&dns_row);

        page.add(&advanced_group);

        // Connection info group
        let info_group = adw::PreferencesGroup::new();
        info_group.set_title("Connection Information");

        let ip_row = adw::ActionRow::new();
        ip_row.set_title("IP Address");
        ip_row.set_subtitle("192.168.1.100");
        info_group.add(&ip_row);

        let mac_row = adw::ActionRow::new();
        mac_row.set_title("MAC Address");
        mac_row.set_subtitle("00:1A:2B:3C:4D:5E");
        info_group.add(&mac_row);

        let gateway_row = adw::ActionRow::new();
        gateway_row.set_title("Default Gateway");
        gateway_row.set_subtitle("192.168.1.1");
        info_group.add(&gateway_row);

        let dns_info_row = adw::ActionRow::new();
        dns_info_row.set_title("DNS Server");
        dns_info_row.set_subtitle("8.8.8.8, 8.8.4.4");
        info_group.add(&dns_info_row);

        page.add(&info_group);

        NetworkPage { widget: page }
    }

    /// Get the page widget
    pub fn widget(&self) -> &adw::PreferencesPage {
        &self.widget
    }
}

impl Default for NetworkPage {
    fn default() -> Self {
        Self::new()
    }
}
