//! Notifications page - Bidirectional notification mirroring

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::protocol::ConnectionManager;
use crate::ui::NotificationRow;

/// Notification data from phone
#[derive(Clone)]
pub struct PhoneNotification {
    pub id: String,
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub timestamp: String,
    pub icon: Option<String>,
    pub dismissible: bool,
    pub actions: Vec<String>,
}

impl PhoneNotification {
    pub fn new(id: &str, app_name: &str, title: &str, body: &str, timestamp: &str) -> Self {
        Self {
            id: id.to_string(),
            app_name: app_name.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            timestamp: timestamp.to_string(),
            icon: None,
            dismissible: true,
            actions: Vec::new(),
        }
    }
}

/// Notification mirroring page
pub struct NotificationsPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<ConnectionManager>>,
}

impl NotificationsPage {
    pub fn new(manager: Rc<RefCell<ConnectionManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Notificacoes");
        page.set_icon_name(Some("preferences-system-notifications-symbolic"));

        // Device selector
        let device_group = adw::PreferencesGroup::builder()
            .title("Dispositivo")
            .build();

        let device_combo = adw::ComboRow::builder()
            .title("Dispositivo ativo")
            .subtitle("Selecione o dispositivo para ver notificacoes")
            .build();
        let devices = gtk4::StringList::new(&[
            "Samsung Galaxy S24",
            "iPad Pro",
        ]);
        device_combo.set_model(Some(&devices));
        device_group.add(&device_combo);

        page.add(&device_group);

        // Settings group
        let settings_group = adw::PreferencesGroup::builder()
            .title("Configuracoes")
            .build();

        let receive_switch = adw::SwitchRow::builder()
            .title("Receber notificacoes")
            .subtitle("Espelhar notificacoes do telefone para o PC")
            .active(true)
            .build();
        settings_group.add(&receive_switch);

        let send_switch = adw::SwitchRow::builder()
            .title("Enviar notificacoes")
            .subtitle("Espelhar notificacoes do PC para o telefone")
            .active(false)
            .build();
        settings_group.add(&send_switch);

        let sound_switch = adw::SwitchRow::builder()
            .title("Som de notificacao")
            .subtitle("Tocar som ao receber notificacao")
            .active(true)
            .build();
        settings_group.add(&sound_switch);

        page.add(&settings_group);

        // Recent notifications group
        let notifications_group = adw::PreferencesGroup::builder()
            .title("Notificacoes Recentes")
            .description("Notificacoes recebidas do dispositivo")
            .build();

        // Sample notifications
        let notifications = vec![
            PhoneNotification::new(
                "1",
                "WhatsApp",
                "Maria Silva",
                "Oi! Voce vem na reuniao hoje?",
                "10:30",
            ),
            PhoneNotification::new(
                "2",
                "Gmail",
                "Novo email de joao@empresa.com",
                "Relatorio mensal de vendas",
                "10:15",
            ),
            PhoneNotification::new(
                "3",
                "Instagram",
                "Nova curtida",
                "@pedro curtiu sua foto",
                "09:45",
            ),
            PhoneNotification::new(
                "4",
                "Banco XYZ",
                "Transacao realizada",
                "Compra aprovada R$ 150,00",
                "09:30",
            ),
            PhoneNotification::new(
                "5",
                "Calendar",
                "Lembrete",
                "Reuniao de equipe em 30 minutos",
                "09:00",
            ),
        ];

        for notification in &notifications {
            let row = NotificationRow::new(notification);
            notifications_group.add(&row.widget());
        }

        // Clear all button
        let clear_button = gtk4::Button::builder()
            .label("Limpar Todas")
            .halign(gtk4::Align::End)
            .margin_top(8)
            .build();
        clear_button.add_css_class("destructive-action");

        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_box.append(&clear_button);
        button_box.set_halign(gtk4::Align::End);

        notifications_group.set_header_suffix(Some(&button_box));

        page.add(&notifications_group);

        // App filter group
        let filter_group = adw::PreferencesGroup::builder()
            .title("Filtro de Apps")
            .description("Selecione quais apps podem enviar notificacoes")
            .build();

        let apps = vec![
            ("WhatsApp", true),
            ("Gmail", true),
            ("Instagram", false),
            ("Telegram", true),
            ("Twitter", false),
            ("Facebook", false),
        ];

        for (app_name, enabled) in apps {
            let row = adw::SwitchRow::builder()
                .title(app_name)
                .active(enabled)
                .build();
            filter_group.add(&row);
        }

        let expand_row = adw::ActionRow::builder()
            .title("Ver todos os apps...")
            .activatable(true)
            .build();
        expand_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        filter_group.add(&expand_row);

        page.add(&filter_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
