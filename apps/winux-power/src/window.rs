// Main window for Winux Power

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::PowerManager;
use crate::pages::{BatteryPage, ProfilesPage, DisplayPage, DevicesPage, StatisticsPage};

pub struct PowerWindow {
    window: adw::ApplicationWindow,
}

impl PowerWindow {
    pub fn new(app: &Application) -> Self {
        let manager = Rc::new(RefCell::new(PowerManager::new()));

        let header = adw::HeaderBar::new();

        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);

        // Battery page
        let battery_page = BatteryPage::new(manager.clone());
        stack.add_titled(battery_page.widget(), Some("battery"), "Bateria")
            .set_icon_name(Some("battery-full-symbolic"));

        // Power profiles page
        let profiles_page = ProfilesPage::new(manager.clone());
        stack.add_titled(profiles_page.widget(), Some("profiles"), "Perfis")
            .set_icon_name(Some("speedometer-symbolic"));

        // Display power page
        let display_page = DisplayPage::new(manager.clone());
        stack.add_titled(display_page.widget(), Some("display"), "Tela")
            .set_icon_name(Some("preferences-desktop-display-symbolic"));

        // Devices page (USB/peripherals)
        let devices_page = DevicesPage::new(manager.clone());
        stack.add_titled(devices_page.widget(), Some("devices"), "Dispositivos")
            .set_icon_name(Some("drive-removable-media-symbolic"));

        // Statistics page
        let statistics_page = StatisticsPage::new(manager.clone());
        stack.add_titled(statistics_page.widget(), Some("statistics"), "Estatisticas")
            .set_icon_name(Some("utilities-system-monitor-symbolic"));

        let switcher = adw::ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header.set_title_widget(Some(&switcher));

        // Quick profile button in header
        let profile_button = gtk4::MenuButton::new();
        profile_button.set_icon_name("power-profile-balanced-symbolic");
        profile_button.set_tooltip_text(Some("Perfil de Energia Atual"));

        let profile_menu = gio::Menu::new();
        profile_menu.append(Some("Alto Desempenho"), Some("app.profile-performance"));
        profile_menu.append(Some("Balanceado"), Some("app.profile-balanced"));
        profile_menu.append(Some("Economia"), Some("app.profile-power-saver"));
        profile_button.set_menu_model(Some(&profile_menu));

        header.pack_end(&profile_button);

        // Battery indicator in header
        let battery_box = Box::new(Orientation::Horizontal, 6);
        let battery_icon = gtk4::Image::from_icon_name("battery-level-80-charging-symbolic");
        let battery_label = gtk4::Label::new(Some("80%"));
        battery_label.add_css_class("dim-label");
        battery_box.append(&battery_icon);
        battery_box.append(&battery_label);
        header.pack_start(&battery_box);

        // Update battery indicator periodically
        let manager_clone = manager.clone();
        let battery_icon_clone = battery_icon.clone();
        let battery_label_clone = battery_label.clone();
        glib::timeout_add_seconds_local(5, move || {
            let mgr = manager_clone.borrow();
            let percentage = mgr.get_battery_percentage();
            let charging = mgr.is_charging();

            battery_label_clone.set_text(&format!("{}%", percentage));

            let icon_name = get_battery_icon_name(percentage, charging);
            battery_icon_clone.set_icon_name(Some(icon_name));

            glib::ControlFlow::Continue
        });

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Gerenciador de Energia")
            .default_width(900)
            .default_height(650)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn get_battery_icon_name(percentage: u32, charging: bool) -> &'static str {
    let level = match percentage {
        0..=10 => "empty",
        11..=20 => "caution",
        21..=30 => "low",
        31..=50 => "50",
        51..=70 => "60",
        71..=90 => "80",
        _ => "full",
    };

    if charging {
        match level {
            "empty" => "battery-level-10-charging-symbolic",
            "caution" => "battery-level-20-charging-symbolic",
            "low" => "battery-level-30-charging-symbolic",
            "50" => "battery-level-50-charging-symbolic",
            "60" => "battery-level-60-charging-symbolic",
            "80" => "battery-level-80-charging-symbolic",
            _ => "battery-level-100-charging-symbolic",
        }
    } else {
        match level {
            "empty" => "battery-level-10-symbolic",
            "caution" => "battery-level-20-symbolic",
            "low" => "battery-level-30-symbolic",
            "50" => "battery-level-50-symbolic",
            "60" => "battery-level-60-symbolic",
            "80" => "battery-level-80-symbolic",
            _ => "battery-level-100-symbolic",
        }
    }
}
