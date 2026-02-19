// Power profiles page for Winux Power

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label, Button};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{PowerManager, PowerProfile};

pub struct ProfilesPage {
    container: gtk4::ScrolledWindow,
}

impl ProfilesPage {
    pub fn new(manager: Rc<RefCell<PowerManager>>) -> Self {
        let page = adw::PreferencesPage::new();

        // Current Profile Status
        let current_group = adw::PreferencesGroup::builder()
            .title("Perfil Atual")
            .build();

        let current_box = Box::new(Orientation::Vertical, 12);
        current_box.set_halign(gtk4::Align::Center);
        current_box.set_margin_top(24);
        current_box.set_margin_bottom(24);

        let profile_icon = gtk4::Image::from_icon_name("power-profile-balanced-symbolic");
        profile_icon.set_pixel_size(64);

        let profile_name = Label::new(Some("Balanceado"));
        profile_name.add_css_class("title-1");

        let profile_desc = Label::new(Some("Equilibrio entre desempenho e economia de energia"));
        profile_desc.add_css_class("dim-label");

        current_box.append(&profile_icon);
        current_box.append(&profile_name);
        current_box.append(&profile_desc);

        current_group.add(&current_box);
        page.add(&current_group);

        // Power Profiles Group
        let profiles_group = adw::PreferencesGroup::builder()
            .title("Perfis de Energia")
            .description("Selecione um perfil baseado no seu uso")
            .build();

        // Performance profile
        let performance_row = create_profile_row(
            "Alto Desempenho",
            "Maximo desempenho para tarefas intensivas",
            "power-profile-performance-symbolic",
            PowerProfile::Performance,
            manager.clone(),
        );
        profiles_group.add(&performance_row);

        // Balanced profile
        let balanced_row = create_profile_row(
            "Balanceado",
            "Equilibrio entre desempenho e economia",
            "power-profile-balanced-symbolic",
            PowerProfile::Balanced,
            manager.clone(),
        );
        profiles_group.add(&balanced_row);

        // Power saver profile
        let power_saver_row = create_profile_row(
            "Economia de Energia",
            "Maximiza duracao da bateria",
            "power-profile-power-saver-symbolic",
            PowerProfile::PowerSaver,
            manager.clone(),
        );
        profiles_group.add(&power_saver_row);

        page.add(&profiles_group);

        // Profile Details Group
        let details_group = adw::PreferencesGroup::builder()
            .title("Detalhes do Perfil Atual")
            .build();

        // CPU Governor
        let cpu_row = adw::ActionRow::builder()
            .title("Governor da CPU")
            .subtitle("Politica de frequencia do processador")
            .build();
        cpu_row.add_prefix(&gtk4::Image::from_icon_name("cpu-symbolic"));
        let cpu_label = Label::new(Some("schedutil"));
        cpu_row.add_suffix(&cpu_label);
        details_group.add(&cpu_row);

        // CPU Frequency
        let freq_row = adw::ActionRow::builder()
            .title("Frequencia da CPU")
            .subtitle("Faixa de frequencia permitida")
            .build();
        let freq_label = Label::new(Some("800 MHz - 4.2 GHz"));
        freq_row.add_suffix(&freq_label);
        details_group.add(&freq_row);

        // Turbo Boost
        let turbo_row = adw::ActionRow::builder()
            .title("Turbo Boost")
            .subtitle("Frequencias acima do clock base")
            .build();
        let turbo_status = Label::new(Some("Habilitado"));
        turbo_status.add_css_class("success");
        turbo_row.add_suffix(&turbo_status);
        details_group.add(&turbo_row);

        // GPU Profile
        let gpu_row = adw::ActionRow::builder()
            .title("Perfil da GPU")
            .subtitle("Modo de energia da placa de video")
            .build();
        gpu_row.add_prefix(&gtk4::Image::from_icon_name("video-display-symbolic"));
        let gpu_label = Label::new(Some("Auto"));
        gpu_row.add_suffix(&gpu_label);
        details_group.add(&gpu_row);

        page.add(&details_group);

        // Automatic Profiles Group
        let auto_group = adw::PreferencesGroup::builder()
            .title("Perfis Automaticos")
            .description("Mude o perfil automaticamente")
            .build();

        // On battery
        let battery_profile_row = adw::ComboRow::builder()
            .title("Na Bateria")
            .subtitle("Perfil quando desconectado da tomada")
            .build();
        battery_profile_row.add_prefix(&gtk4::Image::from_icon_name("battery-symbolic"));
        let battery_profiles = gtk4::StringList::new(&["Economia de Energia", "Balanceado", "Alto Desempenho"]);
        battery_profile_row.set_model(Some(&battery_profiles));
        battery_profile_row.set_selected(0);
        auto_group.add(&battery_profile_row);

        // On AC
        let ac_profile_row = adw::ComboRow::builder()
            .title("Na Tomada")
            .subtitle("Perfil quando conectado a energia")
            .build();
        ac_profile_row.add_prefix(&gtk4::Image::from_icon_name("ac-adapter-symbolic"));
        let ac_profiles = gtk4::StringList::new(&["Economia de Energia", "Balanceado", "Alto Desempenho"]);
        ac_profile_row.set_model(Some(&ac_profiles));
        ac_profile_row.set_selected(1);
        auto_group.add(&ac_profile_row);

        // Low battery threshold
        let low_battery_row = adw::SpinRow::builder()
            .title("Economia em Bateria Baixa")
            .subtitle("Trocar para economia abaixo de")
            .adjustment(&gtk4::Adjustment::new(20.0, 5.0, 50.0, 1.0, 5.0, 0.0))
            .build();
        auto_group.add(&low_battery_row);

        page.add(&auto_group);

        // Custom Profiles Group
        let custom_group = adw::PreferencesGroup::builder()
            .title("Perfis Personalizados")
            .description("Crie seus proprios perfis de energia")
            .build();

        // Gaming profile (custom)
        let gaming_row = adw::ActionRow::builder()
            .title("Gaming")
            .subtitle("Otimizado para jogos")
            .activatable(true)
            .build();
        gaming_row.add_prefix(&gtk4::Image::from_icon_name("input-gaming-symbolic"));
        let gaming_switch = gtk4::Switch::new();
        gaming_switch.set_valign(gtk4::Align::Center);
        gaming_row.add_suffix(&gaming_switch);
        let edit_btn = Button::from_icon_name("document-edit-symbolic");
        edit_btn.set_valign(gtk4::Align::Center);
        edit_btn.add_css_class("flat");
        gaming_row.add_suffix(&edit_btn);
        custom_group.add(&gaming_row);

        // Development profile (custom)
        let dev_row = adw::ActionRow::builder()
            .title("Desenvolvimento")
            .subtitle("Compilacao e IDEs")
            .activatable(true)
            .build();
        dev_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        let dev_switch = gtk4::Switch::new();
        dev_switch.set_valign(gtk4::Align::Center);
        dev_row.add_suffix(&dev_switch);
        let dev_edit_btn = Button::from_icon_name("document-edit-symbolic");
        dev_edit_btn.set_valign(gtk4::Align::Center);
        dev_edit_btn.add_css_class("flat");
        dev_row.add_suffix(&dev_edit_btn);
        custom_group.add(&dev_row);

        // Add new custom profile
        let add_row = adw::ActionRow::builder()
            .title("Criar Novo Perfil")
            .subtitle("Configure um perfil personalizado")
            .activatable(true)
            .build();
        add_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
        add_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        custom_group.add(&add_row);

        page.add(&custom_group);

        // TLP Integration Group
        let tlp_group = adw::PreferencesGroup::builder()
            .title("TLP (Avancado)")
            .description("Configuracoes avancadas via TLP")
            .build();

        let tlp_status_row = adw::ActionRow::builder()
            .title("Status do TLP")
            .subtitle("Servico de gerenciamento de energia")
            .build();
        let tlp_badge = Label::new(Some("Ativo"));
        tlp_badge.add_css_class("success");
        tlp_status_row.add_suffix(&tlp_badge);
        tlp_group.add(&tlp_status_row);

        let tlp_config_row = adw::ActionRow::builder()
            .title("Configurar TLP")
            .subtitle("Editar configuracoes avancadas")
            .activatable(true)
            .build();
        tlp_config_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        tlp_group.add(&tlp_config_row);

        page.add(&tlp_group);

        let container = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { container }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.container
    }
}

fn create_profile_row(
    title: &str,
    subtitle: &str,
    icon: &str,
    profile: PowerProfile,
    manager: Rc<RefCell<PowerManager>>,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .activatable(true)
        .build();

    row.add_prefix(&gtk4::Image::from_icon_name(icon));

    let radio = gtk4::CheckButton::new();
    radio.set_valign(gtk4::Align::Center);

    // Check if this is the current profile
    let current_profile = manager.borrow().get_current_profile();
    if current_profile == profile {
        radio.set_active(true);
    }

    let manager_clone = manager.clone();
    let profile_clone = profile;
    radio.connect_toggled(move |btn| {
        if btn.is_active() {
            manager_clone.borrow_mut().set_profile(profile_clone);
        }
    });

    row.add_suffix(&radio);

    row
}
