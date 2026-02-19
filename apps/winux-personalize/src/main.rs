// Winux Personalize - System Personalization Manager
// Copyright (c) 2026 Winux OS Project
//
// Provides three interface modes:
// - Windows-like: Taskbar at bottom, Start menu
// - Linux-like: Traditional GNOME experience
// - Mac-like: Dock at bottom, colored window buttons

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, CheckButton, Image, Label, Orientation, Picture,
    ToggleButton, Grid,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, ApplicationWindow, Clamp, HeaderBar, PreferencesGroup,
    PreferencesPage, StatusPage, ViewStack, ViewSwitcher, ViewSwitcherBar,
};
use std::fs;
use std::path::PathBuf;

const APP_ID: &str = "org.winux.Personalize";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let header = HeaderBar::new();
    let title = adw::WindowTitle::new("Personalizar", "Configure sua experiencia");
    header.set_title_widget(Some(&title));

    let stack = ViewStack::new();

    // Interface Mode Page
    let mode_page = create_mode_page();
    stack.add_titled(&mode_page, Some("modes"), "Estilos")
        .set_icon_name(Some("view-grid-symbolic"));

    // Theme Page
    let theme_page = create_theme_page();
    stack.add_titled(&theme_page, Some("themes"), "Temas")
        .set_icon_name(Some("preferences-desktop-appearance-symbolic"));

    // Wallpaper Page
    let wallpaper_page = create_wallpaper_page();
    stack.add_titled(&wallpaper_page, Some("wallpapers"), "Papeis de Parede")
        .set_icon_name(Some("preferences-desktop-wallpaper-symbolic"));

    // Icons Page
    let icons_page = create_icons_page();
    stack.add_titled(&icons_page, Some("icons"), "Icones")
        .set_icon_name(Some("folder-symbolic"));

    // View switcher
    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();
    header.set_title_widget(Some(&switcher));

    let switcher_bar = ViewSwitcherBar::builder()
        .stack(&stack)
        .build();

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&stack);
    main_box.append(&switcher_bar);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Personalize")
        .default_width(900)
        .default_height(700)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));

    // Responsive switcher
    let switcher_bar_clone = switcher_bar.clone();
    window.connect_default_width_notify(move |w| {
        switcher_bar_clone.set_reveal(w.default_width() < 600);
    });

    window.present();
}

fn create_mode_page() -> Box {
    let page = Box::new(Orientation::Vertical, 24);
    page.set_margin_top(24);
    page.set_margin_bottom(24);
    page.set_margin_start(24);
    page.set_margin_end(24);

    let title = Label::new(Some("Escolha seu Estilo"));
    title.add_css_class("title-1");
    page.append(&title);

    let subtitle = Label::new(Some("Selecione a experiencia que mais combina com voce"));
    subtitle.add_css_class("dim-label");
    page.append(&subtitle);

    let modes_box = Box::new(Orientation::Horizontal, 24);
    modes_box.set_halign(gtk4::Align::Center);
    modes_box.set_margin_top(24);

    // Windows-like mode
    let windows_card = create_mode_card(
        "Windows Like",
        "Barra de tarefas fixa na parte inferior com Menu Iniciar.\nFamiliar para usuarios Windows.",
        "windows",
        include_str!("../../../assets/mode-previews/windows-preview.txt"),
    );
    modes_box.append(&windows_card);

    // Linux-like mode
    let linux_card = create_mode_card(
        "Linux Like",
        "Experiencia tradicional com barra superior e dock.\nPadrao GNOME otimizado.",
        "linux",
        include_str!("../../../assets/mode-previews/linux-preview.txt"),
    );
    modes_box.append(&linux_card);

    // Mac-like mode
    let mac_card = create_mode_card(
        "Mac Like",
        "Dock centralizado, botoes coloridos nas janelas.\nDesign elegante e minimalista.",
        "macos",
        include_str!("../../../assets/mode-previews/mac-preview.txt"),
    );
    modes_box.append(&mac_card);

    page.append(&modes_box);

    // Apply button
    let apply_btn = Button::with_label("Aplicar Estilo Selecionado");
    apply_btn.add_css_class("suggested-action");
    apply_btn.add_css_class("pill");
    apply_btn.set_halign(gtk4::Align::Center);
    apply_btn.set_margin_top(24);
    apply_btn.connect_clicked(|_| {
        println!("Applying selected style...");
        // TODO: Apply the selected interface mode
    });
    page.append(&apply_btn);

    page
}

fn create_mode_card(title: &str, description: &str, mode_id: &str, _preview: &str) -> Box {
    let card = Box::new(Orientation::Vertical, 12);
    card.add_css_class("card");
    card.set_margin_top(12);
    card.set_margin_bottom(12);
    card.set_margin_start(12);
    card.set_margin_end(12);
    card.set_size_request(250, 300);

    let inner = Box::new(Orientation::Vertical, 12);
    inner.set_margin_top(16);
    inner.set_margin_bottom(16);
    inner.set_margin_start(16);
    inner.set_margin_end(16);

    // Preview area
    let preview_frame = gtk4::Frame::new(None);
    preview_frame.set_size_request(218, 140);
    preview_frame.add_css_class("view");

    let preview_content = create_mode_preview(mode_id);
    preview_frame.set_child(Some(&preview_content));
    inner.append(&preview_frame);

    // Title
    let title_label = Label::new(Some(title));
    title_label.add_css_class("title-3");
    inner.append(&title_label);

    // Description
    let desc_label = Label::new(Some(description));
    desc_label.add_css_class("dim-label");
    desc_label.add_css_class("caption");
    desc_label.set_wrap(true);
    desc_label.set_justify(gtk4::Justification::Center);
    inner.append(&desc_label);

    // Radio button
    let radio = CheckButton::with_label("Selecionar");
    radio.set_halign(gtk4::Align::Center);
    inner.append(&radio);

    card.append(&inner);
    card
}

fn create_mode_preview(mode_id: &str) -> Box {
    let preview = Box::new(Orientation::Vertical, 0);
    preview.set_vexpand(true);

    match mode_id {
        "windows" => {
            // Windows-like preview: top area + bottom taskbar
            let content = Box::new(Orientation::Vertical, 0);
            content.set_vexpand(true);

            let desktop = Box::new(Orientation::Vertical, 0);
            desktop.set_vexpand(true);
            desktop.add_css_class("view");

            // Desktop icons
            let icons_grid = Grid::new();
            icons_grid.set_margin_top(8);
            icons_grid.set_margin_start(8);
            icons_grid.set_column_spacing(4);
            icons_grid.set_row_spacing(4);

            let icon1 = Label::new(Some(""));
            icon1.set_size_request(24, 24);
            icons_grid.attach(&icon1, 0, 0, 1, 1);

            desktop.append(&icons_grid);
            content.append(&desktop);

            // Taskbar at bottom
            let taskbar = Box::new(Orientation::Horizontal, 4);
            taskbar.add_css_class("toolbar");
            taskbar.set_size_request(-1, 28);

            let start_btn = Button::new();
            start_btn.set_size_request(24, 24);
            start_btn.set_icon_name("view-grid-symbolic");
            taskbar.append(&start_btn);

            let sep = gtk4::Separator::new(Orientation::Vertical);
            taskbar.append(&sep);

            content.append(&taskbar);
            preview.append(&content);
        }
        "linux" => {
            // Linux-like preview: top bar + content
            let content = Box::new(Orientation::Vertical, 0);

            // Top bar
            let topbar = Box::new(Orientation::Horizontal, 4);
            topbar.add_css_class("toolbar");
            topbar.set_size_request(-1, 20);

            let activities = Label::new(Some("Activities"));
            activities.add_css_class("caption");
            activities.set_margin_start(8);
            topbar.append(&activities);

            let spacer = Box::new(Orientation::Horizontal, 0);
            spacer.set_hexpand(true);
            topbar.append(&spacer);

            let clock = Label::new(Some("12:00"));
            clock.add_css_class("caption");
            clock.set_margin_end(8);
            topbar.append(&clock);

            content.append(&topbar);

            // Desktop area
            let desktop = Box::new(Orientation::Vertical, 0);
            desktop.set_vexpand(true);
            desktop.add_css_class("view");
            content.append(&desktop);

            preview.append(&content);
        }
        "macos" => {
            // Mac-like preview: menubar + content + dock
            let content = Box::new(Orientation::Vertical, 0);

            // Menu bar
            let menubar = Box::new(Orientation::Horizontal, 8);
            menubar.add_css_class("toolbar");
            menubar.set_size_request(-1, 18);

            let logo = Label::new(Some("W"));
            logo.add_css_class("caption");
            logo.set_margin_start(8);
            menubar.append(&logo);

            let spacer = Box::new(Orientation::Horizontal, 0);
            spacer.set_hexpand(true);
            menubar.append(&spacer);

            content.append(&menubar);

            // Desktop
            let desktop = Box::new(Orientation::Vertical, 0);
            desktop.set_vexpand(true);
            desktop.add_css_class("view");
            content.append(&desktop);

            // Dock at bottom
            let dock_container = Box::new(Orientation::Horizontal, 0);
            dock_container.set_halign(gtk4::Align::Center);
            dock_container.set_margin_bottom(4);

            let dock = Box::new(Orientation::Horizontal, 4);
            dock.add_css_class("card");
            dock.set_margin_top(2);
            dock.set_margin_bottom(2);
            dock.set_margin_start(8);
            dock.set_margin_end(8);

            for _ in 0..5 {
                let icon = Box::new(Orientation::Vertical, 0);
                icon.set_size_request(18, 18);
                icon.add_css_class("circular");
                dock.append(&icon);
            }

            dock_container.append(&dock);
            content.append(&dock_container);

            preview.append(&content);
        }
        _ => {}
    }

    preview
}

fn create_theme_page() -> PreferencesPage {
    let page = PreferencesPage::new();

    // Color scheme group
    let scheme_group = PreferencesGroup::builder()
        .title("Esquema de Cores")
        .description("Escolha entre claro, escuro ou automatico")
        .build();

    let light_row = ActionRow::builder()
        .title("Claro")
        .subtitle("Tema claro para ambientes bem iluminados")
        .activatable(true)
        .build();
    light_row.add_suffix(&CheckButton::new());
    scheme_group.add(&light_row);

    let dark_row = ActionRow::builder()
        .title("Escuro")
        .subtitle("Tema escuro para reduzir cansaco visual")
        .activatable(true)
        .build();
    dark_row.add_suffix(&CheckButton::new());
    scheme_group.add(&dark_row);

    let auto_row = ActionRow::builder()
        .title("Automatico")
        .subtitle("Alterna baseado no horario do dia")
        .activatable(true)
        .build();
    auto_row.add_suffix(&CheckButton::new());
    scheme_group.add(&auto_row);

    page.add(&scheme_group);

    // Accent color group
    let accent_group = PreferencesGroup::builder()
        .title("Cor de Destaque")
        .description("Personalize a cor principal do sistema")
        .build();

    let colors = [
        ("Azul Winux", "#58a6ff"),
        ("Verde", "#3fb950"),
        ("Roxo", "#a371f7"),
        ("Rosa", "#f778ba"),
        ("Laranja", "#f0883e"),
        ("Vermelho", "#f85149"),
    ];

    for (name, _color) in colors {
        let row = ActionRow::builder()
            .title(name)
            .activatable(true)
            .build();
        row.add_suffix(&CheckButton::new());
        accent_group.add(&row);
    }

    page.add(&accent_group);

    page
}

fn create_wallpaper_page() -> PreferencesPage {
    let page = PreferencesPage::new();

    let wallpaper_group = PreferencesGroup::builder()
        .title("Papeis de Parede")
        .description("Selecione ou adicione papeis de parede")
        .build();

    let default_row = ActionRow::builder()
        .title("Winux Default")
        .subtitle("Papel de parede padrao do sistema")
        .activatable(true)
        .build();
    wallpaper_group.add(&default_row);

    let dark_row = ActionRow::builder()
        .title("Winux Dark")
        .subtitle("Versao escura do papel de parede")
        .activatable(true)
        .build();
    wallpaper_group.add(&dark_row);

    let gradient_row = ActionRow::builder()
        .title("Winux Gradient")
        .subtitle("Gradiente minimalista")
        .activatable(true)
        .build();
    wallpaper_group.add(&gradient_row);

    let custom_row = ActionRow::builder()
        .title("Personalizado...")
        .subtitle("Selecione uma imagem do seu computador")
        .activatable(true)
        .build();
    custom_row.add_suffix(&Image::from_icon_name("folder-open-symbolic"));
    wallpaper_group.add(&custom_row);

    page.add(&wallpaper_group);

    page
}

fn create_icons_page() -> PreferencesPage {
    let page = PreferencesPage::new();

    let desktop_group = PreferencesGroup::builder()
        .title("Icones da Area de Trabalho")
        .description("Configure quais icones aparecem no desktop")
        .build();

    let options = [
        ("Pasta Pessoal", "Mostra atalho para sua pasta de usuario", true),
        ("Lixeira", "Mostra a lixeira na area de trabalho", true),
        ("Computador", "Mostra dispositivos montados", false),
        ("Rede", "Mostra locais de rede", false),
    ];

    for (title, subtitle, default) in options {
        let row = ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .activatable(true)
            .build();

        let switch = gtk4::Switch::new();
        switch.set_active(default);
        switch.set_valign(gtk4::Align::Center);
        row.add_suffix(&switch);
        row.set_activatable_widget(Some(&switch));

        desktop_group.add(&row);
    }

    page.add(&desktop_group);

    // Icon theme group
    let theme_group = PreferencesGroup::builder()
        .title("Tema de Icones")
        .build();

    let themes = [
        ("Papirus", "Tema de icones moderno e colorido"),
        ("Adwaita", "Tema padrao do GNOME"),
        ("Winux Icons", "Icones personalizados do Winux"),
    ];

    for (name, desc) in themes {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(desc)
            .activatable(true)
            .build();
        row.add_suffix(&CheckButton::new());
        theme_group.add(&row);
    }

    page.add(&theme_group);

    page
}
