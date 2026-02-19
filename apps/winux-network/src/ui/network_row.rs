//! Network row widget for displaying network information
//!
//! A reusable row widget for WiFi networks, ethernet connections, etc.

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;
use adw::ActionRow;

use crate::nm::{AccessPoint, WifiSecurity, signal_strength_icon, security_display};

/// Network row widget
pub struct NetworkRow {
    row: ActionRow,
}

impl NetworkRow {
    /// Create a new network row from an access point
    pub fn from_access_point(ap: &AccessPoint) -> Self {
        let row = ActionRow::builder()
            .title(&ap.ssid)
            .activatable(true)
            .build();

        // Signal strength icon
        let signal_icon = Image::from_icon_name(signal_strength_icon(ap.signal_strength));
        if ap.is_connected {
            signal_icon.add_css_class("success");
        }
        row.add_prefix(&signal_icon);

        // Security icon
        if ap.security != WifiSecurity::None {
            let security_icon = Image::from_icon_name("network-wireless-encrypted-symbolic");
            security_icon.set_tooltip_text(Some(security_display(ap.security)));
            row.add_suffix(&security_icon);
        }

        // Connection status or connect button
        if ap.is_connected {
            row.set_subtitle("Conectado");

            let check_icon = Image::from_icon_name("emblem-ok-symbolic");
            check_icon.add_css_class("success");
            row.add_suffix(&check_icon);
        } else {
            let freq_band = if ap.frequency > 5000 { "5 GHz" } else { "2.4 GHz" };
            row.set_subtitle(&format!("{} - {}%", freq_band, ap.signal_strength));

            let connect_btn = Button::with_label("Conectar");
            connect_btn.add_css_class("flat");
            connect_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&connect_btn);
        }

        Self { row }
    }

    /// Create a simple network row
    pub fn new(title: &str, subtitle: &str, icon: &str) -> Self {
        let row = ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();

        let icon_widget = Image::from_icon_name(icon);
        row.add_prefix(&icon_widget);

        Self { row }
    }

    /// Create a network row with signal strength bar
    pub fn with_signal_bar(title: &str, signal_strength: u8, secured: bool) -> Self {
        let row = ActionRow::builder()
            .title(title)
            .activatable(true)
            .build();

        // Signal icon
        let signal_icon = Image::from_icon_name(signal_strength_icon(signal_strength));
        row.add_prefix(&signal_icon);

        // Signal strength bar
        let signal_bar = ProgressBar::new();
        signal_bar.set_fraction(signal_strength as f64 / 100.0);
        signal_bar.set_valign(gtk4::Align::Center);
        signal_bar.set_size_request(50, -1);
        row.add_suffix(&signal_bar);

        // Security icon
        if secured {
            let lock_icon = Image::from_icon_name("network-wireless-encrypted-symbolic");
            row.add_suffix(&lock_icon);
        }

        Self { row }
    }

    /// Create an ethernet interface row
    pub fn ethernet(name: &str, status: &str, speed: Option<&str>) -> Self {
        let row = ActionRow::builder()
            .title(name)
            .build();

        let icon = if status == "Conectado" {
            "network-wired-symbolic"
        } else {
            "network-wired-disconnected-symbolic"
        };

        let icon_widget = Image::from_icon_name(icon);
        if status == "Conectado" {
            icon_widget.add_css_class("success");
        }
        row.add_prefix(&icon_widget);

        let subtitle = if let Some(s) = speed {
            format!("{} - {}", status, s)
        } else {
            status.to_string()
        };
        row.set_subtitle(&subtitle);

        Self { row }
    }

    /// Create a VPN connection row
    pub fn vpn(name: &str, vpn_type: &str, connected: bool) -> Self {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(vpn_type)
            .build();

        let icon = Image::from_icon_name("network-vpn-symbolic");
        if connected {
            icon.add_css_class("success");
        }
        row.add_prefix(&icon);

        let switch = gtk4::Switch::new();
        switch.set_active(connected);
        switch.set_valign(gtk4::Align::Center);
        row.add_suffix(&switch);

        Self { row }
    }

    /// Get the underlying ActionRow widget
    pub fn widget(&self) -> &ActionRow {
        &self.row
    }

    /// Set the row as connected
    pub fn set_connected(&self, connected: bool) {
        if connected {
            self.row.set_subtitle("Conectado");
        }
    }

    /// Add a connect button
    pub fn add_connect_button<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        let btn = Button::with_label("Conectar");
        btn.add_css_class("flat");
        btn.set_valign(gtk4::Align::Center);
        btn.connect_clicked(move |_| callback());
        self.row.add_suffix(&btn);
    }

    /// Add a disconnect button
    pub fn add_disconnect_button<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        let btn = Button::with_label("Desconectar");
        btn.add_css_class("destructive-action");
        btn.set_valign(gtk4::Align::Center);
        btn.connect_clicked(move |_| callback());
        self.row.add_suffix(&btn);
    }

    /// Add settings button
    pub fn add_settings_button<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        let btn = Button::from_icon_name("emblem-system-symbolic");
        btn.add_css_class("flat");
        btn.set_valign(gtk4::Align::Center);
        btn.set_tooltip_text(Some("Configuracoes"));
        btn.connect_clicked(move |_| callback());
        self.row.add_suffix(&btn);
    }
}

/// Helper to create a list of network rows from access points
pub fn create_network_list(access_points: &[AccessPoint]) -> Vec<NetworkRow> {
    access_points
        .iter()
        .map(NetworkRow::from_access_point)
        .collect()
}

/// Sort access points by signal strength (descending)
pub fn sort_by_signal(mut aps: Vec<AccessPoint>) -> Vec<AccessPoint> {
    aps.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
    aps
}

/// Filter access points by minimum signal strength
pub fn filter_by_signal(aps: &[AccessPoint], min_signal: u8) -> Vec<AccessPoint> {
    aps.iter()
        .filter(|ap| ap.signal_strength >= min_signal)
        .cloned()
        .collect()
}
