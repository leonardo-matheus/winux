//! Screen page - Screen mirroring/casting via scrcpy

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::protocol::ConnectionManager;

/// Screen mirroring page
pub struct ScreenPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<ConnectionManager>>,
}

impl ScreenPage {
    pub fn new(manager: Rc<RefCell<ConnectionManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Tela");
        page.set_icon_name(Some("video-display-symbolic"));

        // Device selector
        let device_group = adw::PreferencesGroup::builder()
            .title("Dispositivo")
            .build();

        let device_combo = adw::ComboRow::builder()
            .title("Dispositivo")
            .subtitle("Selecione o dispositivo para espelhar")
            .build();
        let devices = gtk4::StringList::new(&[
            "Samsung Galaxy S24",
            "iPad Pro",
        ]);
        device_combo.set_model(Some(&devices));
        device_group.add(&device_combo);

        page.add(&device_group);

        // Screen mirroring preview
        let preview_group = adw::PreferencesGroup::builder()
            .title("Espelhamento de Tela")
            .description("Visualize e controle a tela do dispositivo")
            .build();

        // Preview container
        let preview_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        preview_box.set_margin_start(24);
        preview_box.set_margin_end(24);
        preview_box.set_margin_top(16);
        preview_box.set_margin_bottom(16);

        // Phone frame placeholder
        let frame = gtk4::Frame::new(None);
        frame.set_halign(gtk4::Align::Center);

        let phone_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        phone_box.set_size_request(240, 520);
        phone_box.add_css_class("card");

        // Placeholder content
        let placeholder = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        placeholder.set_valign(gtk4::Align::Center);
        placeholder.set_vexpand(true);

        let icon = gtk4::Image::from_icon_name("computer-symbolic");
        icon.set_pixel_size(64);
        icon.add_css_class("dim-label");
        placeholder.append(&icon);

        let label = gtk4::Label::new(Some("Espelhamento desativado"));
        label.add_css_class("dim-label");
        placeholder.append(&label);

        phone_box.append(&placeholder);
        frame.set_child(Some(&phone_box));
        preview_box.append(&frame);

        // Start/Stop buttons
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(16);

        let start_btn = gtk4::Button::builder()
            .label("Iniciar Espelhamento")
            .build();
        start_btn.add_css_class("suggested-action");
        button_box.append(&start_btn);

        let fullscreen_btn = gtk4::Button::from_icon_name("view-fullscreen-symbolic");
        fullscreen_btn.set_tooltip_text(Some("Tela cheia"));
        button_box.append(&fullscreen_btn);

        let record_btn = gtk4::Button::from_icon_name("media-record-symbolic");
        record_btn.set_tooltip_text(Some("Gravar tela"));
        button_box.append(&record_btn);

        preview_box.append(&button_box);

        let preview_row = adw::ActionRow::new();
        preview_row.set_child(Some(&preview_box));
        preview_group.add(&preview_row);

        page.add(&preview_group);

        // Quality settings
        let quality_group = adw::PreferencesGroup::builder()
            .title("Qualidade")
            .build();

        let resolution = adw::ComboRow::builder()
            .title("Resolucao")
            .subtitle("Resolucao maxima do espelhamento")
            .build();
        let resolutions = gtk4::StringList::new(&[
            "Original",
            "1920x1080 (Full HD)",
            "1280x720 (HD)",
            "854x480 (SD)",
        ]);
        resolution.set_model(Some(&resolutions));
        resolution.set_selected(1);
        quality_group.add(&resolution);

        let bitrate = adw::ComboRow::builder()
            .title("Taxa de bits")
            .subtitle("Qualidade de video")
            .build();
        let bitrates = gtk4::StringList::new(&[
            "8 Mbps (Alta)",
            "4 Mbps (Media)",
            "2 Mbps (Baixa)",
            "1 Mbps (Muito baixa)",
        ]);
        bitrate.set_model(Some(&bitrates));
        bitrate.set_selected(1);
        quality_group.add(&bitrate);

        let framerate = adw::SpinRow::with_range(15.0, 60.0, 5.0);
        framerate.set_title("Taxa de quadros");
        framerate.set_subtitle("FPS do espelhamento");
        framerate.set_value(30.0);
        quality_group.add(&framerate);

        page.add(&quality_group);

        // Control settings
        let control_group = adw::PreferencesGroup::builder()
            .title("Controle")
            .build();

        let touch_control = adw::SwitchRow::builder()
            .title("Controle por toque/mouse")
            .subtitle("Controlar o telefone com mouse e teclado")
            .active(true)
            .build();
        control_group.add(&touch_control);

        let show_touches = adw::SwitchRow::builder()
            .title("Mostrar toques")
            .subtitle("Exibir indicador visual dos toques")
            .active(false)
            .build();
        control_group.add(&show_touches);

        let stay_awake = adw::SwitchRow::builder()
            .title("Manter tela ligada")
            .subtitle("Impedir que a tela do telefone apague")
            .active(true)
            .build();
        control_group.add(&stay_awake);

        let turn_screen_off = adw::SwitchRow::builder()
            .title("Desligar tela do telefone")
            .subtitle("Economizar bateria desligando a tela fisica")
            .active(false)
            .build();
        control_group.add(&turn_screen_off);

        page.add(&control_group);

        // Audio settings
        let audio_group = adw::PreferencesGroup::builder()
            .title("Audio")
            .build();

        let forward_audio = adw::SwitchRow::builder()
            .title("Encaminhar audio")
            .subtitle("Reproduzir audio do telefone no PC")
            .active(true)
            .build();
        audio_group.add(&forward_audio);

        let audio_source = adw::ComboRow::builder()
            .title("Fonte de audio")
            .subtitle("Qual audio capturar do dispositivo")
            .build();
        let sources = gtk4::StringList::new(&[
            "Saida (apps, musica)",
            "Microfone",
            "Ambos",
        ]);
        audio_source.set_model(Some(&sources));
        audio_group.add(&audio_source);

        page.add(&audio_group);

        // Advanced settings
        let advanced_group = adw::PreferencesGroup::builder()
            .title("Avancado")
            .build();

        let adb_over_wifi = adw::SwitchRow::builder()
            .title("ADB sobre Wi-Fi")
            .subtitle("Conectar sem cabo USB (requer configuracao inicial)")
            .active(false)
            .build();
        advanced_group.add(&adb_over_wifi);

        let crop = adw::ActionRow::builder()
            .title("Recorte de tela")
            .subtitle("Definir area especifica para espelhar")
            .activatable(true)
            .build();
        crop.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        advanced_group.add(&crop);

        let keyboard_shortcuts = adw::ActionRow::builder()
            .title("Atalhos de teclado")
            .subtitle("Configurar atalhos para acoes rapidas")
            .activatable(true)
            .build();
        keyboard_shortcuts.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        advanced_group.add(&keyboard_shortcuts);

        page.add(&advanced_group);

        // Requirements info
        let info_group = adw::PreferencesGroup::builder()
            .title("Requisitos")
            .build();

        let usb_debug = adw::ActionRow::builder()
            .title("Depuracao USB")
            .subtitle("Necessario: Ativar nas opcoes do desenvolvedor")
            .build();
        usb_debug.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        info_group.add(&usb_debug);

        let scrcpy_info = adw::ActionRow::builder()
            .title("scrcpy")
            .subtitle("Usando scrcpy para espelhamento de alta performance")
            .build();
        scrcpy_info.add_prefix(&gtk4::Image::from_icon_name("emblem-system-symbolic"));
        info_group.add(&scrcpy_info);

        page.add(&info_group);

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
