// Winux Welcome - Recommended Apps
// Select browsers, office suites, media players to install

use gtk4::prelude::*;
use gtk4::{Box, CheckButton, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, ExpanderRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::WelcomeState;

pub fn create_page(state: Rc<RefCell<WelcomeState>>) -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Header section
    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(24);
    header_box.set_margin_bottom(12);
    header_box.set_halign(gtk4::Align::Center);

    let title = Label::new(Some("Aplicativos Recomendados"));
    title.add_css_class("title-1");
    header_box.append(&title);

    let subtitle = Label::new(Some("Selecione os aplicativos que deseja instalar"));
    subtitle.add_css_class("dim-label");
    header_box.append(&subtitle);

    let header_group = PreferencesGroup::new();
    header_group.add(&header_box);
    page.add(&header_group);

    // Browsers Section
    let browsers_group = PreferencesGroup::builder()
        .title("Navegadores")
        .description("Escolha seu navegador web preferido")
        .build();

    let browsers = [
        ("Firefox", "Navegador rapido e focado em privacidade", "firefox", true),
        ("Google Chrome", "Navegador popular do Google", "google-chrome", false),
        ("Brave", "Navegador com bloqueio de anuncios integrado", "brave-browser", false),
        ("Chromium", "Versao open-source do Chrome", "chromium", false),
        ("Microsoft Edge", "Navegador da Microsoft baseado em Chromium", "microsoft-edge", false),
    ];

    for (name, desc, package, default) in browsers {
        let row = create_app_row(name, desc, package, default, state.clone());
        browsers_group.add(&row);
    }

    page.add(&browsers_group);

    // Office Section
    let office_group = PreferencesGroup::builder()
        .title("Escritorio")
        .description("Suite de produtividade")
        .build();

    let office_apps = [
        ("LibreOffice", "Suite completa compativel com MS Office", "libreoffice", true),
        ("OnlyOffice", "Suite moderna com boa compatibilidade", "onlyoffice", false),
        ("WPS Office", "Suite leve e compativel com MS Office", "wps-office", false),
    ];

    for (name, desc, package, default) in office_apps {
        let row = create_app_row(name, desc, package, default, state.clone());
        office_group.add(&row);
    }

    page.add(&office_group);

    // Media Section
    let media_group = PreferencesGroup::builder()
        .title("Multimidia")
        .description("Players de audio e video")
        .build();

    let media_apps = [
        ("VLC", "Player de video universal", "vlc", true),
        ("Spotify", "Streaming de musica", "spotify", false),
        ("Audacious", "Player de musica leve", "audacious", false),
        ("Rhythmbox", "Player de musica do GNOME", "rhythmbox", false),
        ("OBS Studio", "Gravacao e streaming de video", "obs-studio", false),
    ];

    for (name, desc, package, default) in media_apps {
        let row = create_app_row(name, desc, package, default, state.clone());
        media_group.add(&row);
    }

    page.add(&media_group);

    // Communication Section
    let comm_group = PreferencesGroup::builder()
        .title("Comunicacao")
        .description("Mensagens e videoconferencia")
        .build();

    let comm_apps = [
        ("Discord", "Comunicacao para comunidades e gamers", "discord", false),
        ("Telegram", "Mensageiro rapido e seguro", "telegram-desktop", false),
        ("Slack", "Comunicacao para equipes", "slack", false),
        ("Zoom", "Videoconferencia", "zoom", false),
    ];

    for (name, desc, package, default) in comm_apps {
        let row = create_app_row(name, desc, package, default, state.clone());
        comm_group.add(&row);
    }

    page.add(&comm_group);

    // Graphics Section
    let graphics_group = PreferencesGroup::builder()
        .title("Graficos")
        .description("Edicao de imagens e design")
        .build();

    let graphics_apps = [
        ("GIMP", "Editor de imagens avancado", "gimp", false),
        ("Inkscape", "Editor de graficos vetoriais", "inkscape", false),
        ("Krita", "Pintura digital profissional", "krita", false),
        ("Blender", "Modelagem e animacao 3D", "blender", false),
    ];

    for (name, desc, package, default) in graphics_apps {
        let row = create_app_row(name, desc, package, default, state.clone());
        graphics_group.add(&row);
    }

    page.add(&graphics_group);

    // Info text
    let info_group = PreferencesGroup::new();
    let info = Label::new(Some("Os aplicativos serao instalados ao finalizar. Voce pode instalar mais aplicativos depois pela Winux Store."));
    info.add_css_class("dim-label");
    info.add_css_class("caption");
    info.set_wrap(true);
    info.set_margin_top(12);
    info.set_margin_bottom(12);
    info_group.add(&info);
    page.add(&info_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_app_row(
    name: &str,
    description: &str,
    package: &str,
    default: bool,
    state: Rc<RefCell<WelcomeState>>,
) -> ActionRow {
    let row = ActionRow::builder()
        .title(name)
        .subtitle(description)
        .activatable(true)
        .build();

    let check = CheckButton::new();
    check.set_active(default);
    check.set_valign(gtk4::Align::Center);
    row.add_suffix(&check);
    row.set_activatable_widget(Some(&check));

    // Update state when toggled
    let package_name = package.to_string();
    let state_clone = state.clone();

    // Initialize default
    if default {
        state.borrow_mut().selected_apps.push(package_name.clone());
    }

    check.connect_toggled(move |btn| {
        let mut state = state_clone.borrow_mut();
        if btn.is_active() {
            if !state.selected_apps.contains(&package_name) {
                state.selected_apps.push(package_name.clone());
            }
        } else {
            state.selected_apps.retain(|x| x != &package_name);
        }
    });

    row
}
