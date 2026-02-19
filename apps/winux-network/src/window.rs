// Main application window

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};

use crate::pages;

pub struct NetworkWindow {
    window: ApplicationWindow,
}

impl NetworkWindow {
    pub fn new(app: &Application) -> Self {
        let header = HeaderBar::new();

        let stack = ViewStack::new();
        stack.set_vexpand(true);

        // WiFi Page
        let wifi_page = pages::WifiPage::new();
        stack.add_titled(wifi_page.widget(), Some("wifi"), "Wi-Fi")
            .set_icon_name(Some("network-wireless-symbolic"));

        // Ethernet Page
        let ethernet_page = pages::EthernetPage::new();
        stack.add_titled(ethernet_page.widget(), Some("ethernet"), "Ethernet")
            .set_icon_name(Some("network-wired-symbolic"));

        // VPN Page
        let vpn_page = pages::VpnPage::new();
        stack.add_titled(vpn_page.widget(), Some("vpn"), "VPN")
            .set_icon_name(Some("network-vpn-symbolic"));

        // Hotspot Page
        let hotspot_page = pages::HotspotPage::new();
        stack.add_titled(hotspot_page.widget(), Some("hotspot"), "Hotspot")
            .set_icon_name(Some("network-wireless-hotspot-symbolic"));

        // Proxy Page
        let proxy_page = pages::ProxyPage::new();
        stack.add_titled(proxy_page.widget(), Some("proxy"), "Proxy")
            .set_icon_name(Some("preferences-system-network-proxy-symbolic"));

        // Advanced Page
        let advanced_page = pages::AdvancedPage::new();
        stack.add_titled(advanced_page.widget(), Some("advanced"), "Avancado")
            .set_icon_name(Some("preferences-system-symbolic"));

        let switcher = ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header.set_title_widget(Some(&switcher));

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Rede")
            .default_width(900)
            .default_height(700)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
