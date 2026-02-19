// Main window for Winux Connect

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::pages::{
    DevicesPage, NotificationsPage, MessagesPage, FilesPage,
    MediaPage, ClipboardPage, ScreenPage,
};
use crate::protocol::ConnectionManager;

pub struct ConnectWindow {
    window: adw::ApplicationWindow,
}

impl ConnectWindow {
    pub fn new(app: &Application) -> Self {
        let manager = Rc::new(RefCell::new(ConnectionManager::new()));

        // Start discovery service
        {
            let mgr = manager.borrow();
            mgr.start_discovery();
        }

        let header = adw::HeaderBar::new();

        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);

        // Devices page (paired devices list)
        let devices_page = DevicesPage::new(manager.clone());
        stack.add_titled(devices_page.widget(), Some("devices"), "Dispositivos")
            .set_icon_name(Some("phone-symbolic"));

        // Notifications page (bidirectional mirroring)
        let notifications_page = NotificationsPage::new(manager.clone());
        stack.add_titled(notifications_page.widget(), Some("notifications"), "Notificacoes")
            .set_icon_name(Some("preferences-system-notifications-symbolic"));

        // Messages page (SMS/messaging)
        let messages_page = MessagesPage::new(manager.clone());
        stack.add_titled(messages_page.widget(), Some("messages"), "Mensagens")
            .set_icon_name(Some("mail-unread-symbolic"));

        // Files page (file transfer)
        let files_page = FilesPage::new(manager.clone());
        stack.add_titled(files_page.widget(), Some("files"), "Arquivos")
            .set_icon_name(Some("folder-symbolic"));

        // Media page (remote media control)
        let media_page = MediaPage::new(manager.clone());
        stack.add_titled(media_page.widget(), Some("media"), "Midia")
            .set_icon_name(Some("multimedia-player-symbolic"));

        // Clipboard page (sync)
        let clipboard_page = ClipboardPage::new(manager.clone());
        stack.add_titled(clipboard_page.widget(), Some("clipboard"), "Clipboard")
            .set_icon_name(Some("edit-paste-symbolic"));

        // Screen page (mirroring/casting)
        let screen_page = ScreenPage::new(manager.clone());
        stack.add_titled(screen_page.widget(), Some("screen"), "Tela")
            .set_icon_name(Some("video-display-symbolic"));

        let switcher_bar = adw::ViewSwitcherBar::builder()
            .stack(&stack)
            .reveal(true)
            .build();

        let switcher_title = adw::ViewSwitcherTitle::builder()
            .stack(&stack)
            .title("Winux Connect")
            .build();

        switcher_title.connect_title_visible_notify(glib::clone!(
            #[weak] switcher_bar,
            move |title| {
                switcher_bar.set_reveal(title.is_title_visible());
            }
        ));

        header.set_title_widget(Some(&switcher_title));

        // Menu button
        let menu_button = gtk4::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");

        let menu = gio::Menu::new();
        menu.append(Some("Descobrir Dispositivos"), Some("app.discover"));
        menu.append(Some("Adicionar por IP"), Some("app.add-ip"));

        let section = gio::Menu::new();
        section.append(Some("Preferencias"), Some("app.preferences"));
        section.append(Some("Sobre"), Some("app.about"));
        menu.append_section(None, &section);

        menu_button.set_menu_model(Some(&menu));
        header.pack_end(&menu_button);

        // Refresh button
        let refresh_button = gtk4::Button::from_icon_name("view-refresh-symbolic");
        refresh_button.set_tooltip_text(Some("Atualizar lista de dispositivos"));
        let manager_clone = manager.clone();
        refresh_button.connect_clicked(move |_| {
            manager_clone.borrow().start_discovery();
        });
        header.pack_end(&refresh_button);

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);
        main_box.append(&switcher_bar);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Winux Connect")
            .default_width(1000)
            .default_height(700)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        // Setup actions
        Self::setup_actions(&window, manager.clone());

        Self { window }
    }

    fn setup_actions(window: &adw::ApplicationWindow, manager: Rc<RefCell<ConnectionManager>>) {
        let discover_action = gio::SimpleAction::new("discover", None);
        let manager_clone = manager.clone();
        discover_action.connect_activate(move |_, _| {
            manager_clone.borrow().start_discovery();
        });
        window.add_action(&discover_action);

        let add_ip_action = gio::SimpleAction::new("add-ip", None);
        let window_weak = window.downgrade();
        add_ip_action.connect_activate(move |_, _| {
            if let Some(window) = window_weak.upgrade() {
                Self::show_add_ip_dialog(&window);
            }
        });
        window.add_action(&add_ip_action);

        let preferences_action = gio::SimpleAction::new("preferences", None);
        let window_weak = window.downgrade();
        preferences_action.connect_activate(move |_, _| {
            if let Some(window) = window_weak.upgrade() {
                Self::show_preferences_dialog(&window);
            }
        });
        window.add_action(&preferences_action);

        let about_action = gio::SimpleAction::new("about", None);
        let window_weak = window.downgrade();
        about_action.connect_activate(move |_, _| {
            if let Some(window) = window_weak.upgrade() {
                Self::show_about_dialog(&window);
            }
        });
        window.add_action(&about_action);
    }

    fn show_add_ip_dialog(window: &adw::ApplicationWindow) {
        let dialog = adw::AlertDialog::builder()
            .heading("Adicionar Dispositivo por IP")
            .body("Digite o endereco IP do dispositivo")
            .close_response("cancel")
            .default_response("add")
            .build();

        let entry = gtk4::Entry::builder()
            .placeholder_text("192.168.1.100")
            .margin_start(12)
            .margin_end(12)
            .build();

        dialog.set_extra_child(Some(&entry));
        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("add", "Adicionar");
        dialog.set_response_appearance("add", adw::ResponseAppearance::Suggested);

        dialog.present(Some(window));
    }

    fn show_preferences_dialog(window: &adw::ApplicationWindow) {
        let dialog = adw::PreferencesDialog::new();

        // General page
        let general_page = adw::PreferencesPage::builder()
            .title("Geral")
            .icon_name("preferences-system-symbolic")
            .build();

        let sync_group = adw::PreferencesGroup::builder()
            .title("Sincronizacao")
            .build();

        let auto_connect = adw::SwitchRow::builder()
            .title("Conectar automaticamente")
            .subtitle("Conectar a dispositivos pareados quando detectados")
            .active(true)
            .build();
        sync_group.add(&auto_connect);

        let clipboard_sync = adw::SwitchRow::builder()
            .title("Sincronizar clipboard")
            .subtitle("Compartilhar clipboard automaticamente")
            .active(true)
            .build();
        sync_group.add(&clipboard_sync);

        let notification_sync = adw::SwitchRow::builder()
            .title("Espelhar notificacoes")
            .subtitle("Receber notificacoes do telefone no PC")
            .active(true)
            .build();
        sync_group.add(&notification_sync);

        general_page.add(&sync_group);

        // Security group
        let security_group = adw::PreferencesGroup::builder()
            .title("Seguranca")
            .build();

        let encryption = adw::SwitchRow::builder()
            .title("Criptografia E2E")
            .subtitle("Usar criptografia ponta a ponta")
            .active(true)
            .build();
        security_group.add(&encryption);

        let trust_new = adw::SwitchRow::builder()
            .title("Confirmar novos dispositivos")
            .subtitle("Solicitar confirmacao antes de parear")
            .active(true)
            .build();
        security_group.add(&trust_new);

        general_page.add(&security_group);

        dialog.add(&general_page);

        // Files page
        let files_page = adw::PreferencesPage::builder()
            .title("Arquivos")
            .icon_name("folder-symbolic")
            .build();

        let files_group = adw::PreferencesGroup::builder()
            .title("Transferencia de Arquivos")
            .build();

        let download_dir = adw::ActionRow::builder()
            .title("Pasta de downloads")
            .subtitle("~/Downloads/Winux Connect")
            .activatable(true)
            .build();
        download_dir.add_suffix(&gtk4::Image::from_icon_name("folder-symbolic"));
        files_group.add(&download_dir);

        let auto_accept = adw::SwitchRow::builder()
            .title("Aceitar arquivos automaticamente")
            .subtitle("Aceitar arquivos de dispositivos pareados")
            .active(false)
            .build();
        files_group.add(&auto_accept);

        files_page.add(&files_group);
        dialog.add(&files_page);

        dialog.present(Some(window));
    }

    fn show_about_dialog(window: &adw::ApplicationWindow) {
        let about = adw::AboutDialog::builder()
            .application_name("Winux Connect")
            .application_icon("phone-symbolic")
            .version("1.0.0")
            .developer_name("Winux OS Project")
            .license_type(gtk4::License::MitX11)
            .comments("Integracao de smartphones com o Winux OS.\nCompativel com o protocolo KDE Connect.")
            .website("https://github.com/winux-os/winux")
            .build();

        about.add_link("Documentacao", "https://docs.winux.org/connect");
        about.add_link("Reportar Bug", "https://github.com/winux-os/winux/issues");

        about.present(Some(window));
    }

    pub fn present(&self) {
        self.window.present();
    }
}
