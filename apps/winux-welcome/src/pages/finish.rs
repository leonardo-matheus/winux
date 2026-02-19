// Winux Welcome - Finish Page
// Summary of choices and start button

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, StatusPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::WelcomeState;

pub fn create_page(state: Rc<RefCell<WelcomeState>>) -> gtk4::ScrolledWindow {
    let main_box = Box::new(Orientation::Vertical, 0);

    // Status page with completion message
    let status = StatusPage::builder()
        .icon_name("emblem-ok-symbolic")
        .title("Tudo Pronto!")
        .description("Seu Winux esta configurado e pronto para uso")
        .build();
    status.add_css_class("success");

    main_box.append(&status);

    // Summary
    let page = PreferencesPage::new();

    let summary_group = PreferencesGroup::builder()
        .title("Resumo das Configuracoes")
        .description("Suas escolhas serao aplicadas")
        .build();

    // Desktop mode row
    let desktop_row = ActionRow::builder()
        .title("Estilo de Desktop")
        .build();
    desktop_row.add_prefix(&gtk4::Image::from_icon_name("view-grid-symbolic"));

    let state_ref = state.borrow();
    let desktop_mode = state_ref.desktop_mode.clone().unwrap_or("Linux Like".to_string());
    desktop_row.set_subtitle(&desktop_mode);
    drop(state_ref);

    summary_group.add(&desktop_row);

    // Theme row
    let theme_row = ActionRow::builder()
        .title("Tema")
        .build();
    theme_row.add_prefix(&gtk4::Image::from_icon_name("preferences-desktop-appearance-symbolic"));

    let state_ref = state.borrow();
    let theme = state_ref.theme.clone().unwrap_or("Escuro".to_string());
    let accent = state_ref.accent_color.clone().unwrap_or("Azul".to_string());
    theme_row.set_subtitle(&format!("{} com destaque {}", theme, accent));
    drop(state_ref);

    summary_group.add(&theme_row);

    // Wallpaper row
    let wallpaper_row = ActionRow::builder()
        .title("Papel de Parede")
        .build();
    wallpaper_row.add_prefix(&gtk4::Image::from_icon_name("preferences-desktop-wallpaper-symbolic"));

    let state_ref = state.borrow();
    let wallpaper = state_ref.wallpaper.clone().unwrap_or("Winux Aurora".to_string());
    wallpaper_row.set_subtitle(&wallpaper);
    drop(state_ref);

    summary_group.add(&wallpaper_row);

    // Apps row
    let apps_row = ActionRow::builder()
        .title("Aplicativos")
        .build();
    apps_row.add_prefix(&gtk4::Image::from_icon_name("system-software-install-symbolic"));

    let state_ref = state.borrow();
    let apps_count = state_ref.selected_apps.len();
    apps_row.set_subtitle(&format!("{} aplicativos selecionados", apps_count));
    drop(state_ref);

    summary_group.add(&apps_row);

    // Dev setup row
    let dev_row = ActionRow::builder()
        .title("Desenvolvimento")
        .build();
    dev_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));

    let state_ref = state.borrow();
    let langs_count = state_ref.dev_languages.len();
    let ides_count = state_ref.dev_ides.len();
    if langs_count > 0 || ides_count > 0 {
        dev_row.set_subtitle(&format!("{} linguagens, {} IDEs", langs_count, ides_count));
    } else {
        dev_row.set_subtitle("Nao configurado");
    }
    drop(state_ref);

    summary_group.add(&dev_row);

    // Gaming row
    let gaming_row = ActionRow::builder()
        .title("Gaming")
        .build();
    gaming_row.add_prefix(&gtk4::Image::from_icon_name("input-gaming-symbolic"));

    let state_ref = state.borrow();
    let platforms_count = state_ref.gaming_platforms.len();
    let emulators_count = state_ref.gaming_emulators.len();
    if platforms_count > 0 || emulators_count > 0 {
        gaming_row.set_subtitle(&format!("{} plataformas, {} emuladores", platforms_count, emulators_count));
    } else {
        gaming_row.set_subtitle("Nao configurado");
    }
    drop(state_ref);

    summary_group.add(&gaming_row);

    // Privacy row
    let privacy_row = ActionRow::builder()
        .title("Privacidade")
        .build();
    privacy_row.add_prefix(&gtk4::Image::from_icon_name("security-high-symbolic"));

    let state_ref = state.borrow();
    let mut privacy_status = Vec::new();
    if state_ref.telemetry_enabled {
        privacy_status.push("Telemetria ativa");
    }
    if state_ref.crash_reports_enabled {
        privacy_status.push("Relatorios de erro ativos");
    }
    if privacy_status.is_empty() {
        privacy_row.set_subtitle("Privacidade maxima");
    } else {
        privacy_row.set_subtitle(&privacy_status.join(", "));
    }
    drop(state_ref);

    summary_group.add(&privacy_row);

    page.add(&summary_group);

    // What happens next
    let next_group = PreferencesGroup::builder()
        .title("O que acontece agora?")
        .build();

    let steps = [
        ("1. Aplicar configuracoes", "Suas preferencias serao salvas"),
        ("2. Instalar aplicativos", "Os apps selecionados serao baixados em segundo plano"),
        ("3. Comecar a usar", "Explore seu novo sistema Winux!"),
    ];

    for (title, subtitle) in steps {
        let row = ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();
        row.add_prefix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        next_group.add(&row);
    }

    page.add(&next_group);

    // Tips
    let tips_group = PreferencesGroup::builder()
        .title("Dicas Rapidas")
        .build();

    let tips = [
        ("Super", "Abre a visao de atividades"),
        ("Super + A", "Abre o menu de aplicativos"),
        ("Super + E", "Abre o gerenciador de arquivos"),
        ("Super + T", "Abre o terminal"),
    ];

    for (shortcut, description) in tips {
        let row = ActionRow::builder()
            .title(shortcut)
            .subtitle(description)
            .build();
        row.add_css_class("monospace");
        tips_group.add(&row);
    }

    page.add(&tips_group);

    // Help section
    let help_group = PreferencesGroup::builder()
        .title("Precisa de Ajuda?")
        .build();

    let docs_row = ActionRow::builder()
        .title("Documentacao")
        .subtitle("Guias e tutoriais completos")
        .activatable(true)
        .build();
    docs_row.add_prefix(&gtk4::Image::from_icon_name("help-contents-symbolic"));
    docs_row.add_suffix(&gtk4::Image::from_icon_name("external-link-symbolic"));
    help_group.add(&docs_row);

    let community_row = ActionRow::builder()
        .title("Comunidade")
        .subtitle("Forum, Discord e redes sociais")
        .activatable(true)
        .build();
    community_row.add_prefix(&gtk4::Image::from_icon_name("system-users-symbolic"));
    community_row.add_suffix(&gtk4::Image::from_icon_name("external-link-symbolic"));
    help_group.add(&community_row);

    page.add(&help_group);

    main_box.append(&page);

    // Don't show again checkbox
    let bottom_box = Box::new(Orientation::Vertical, 12);
    bottom_box.set_margin_start(24);
    bottom_box.set_margin_end(24);
    bottom_box.set_margin_bottom(24);

    let dont_show = gtk4::CheckButton::with_label("Nao mostrar novamente na inicializacao");
    dont_show.set_halign(gtk4::Align::Center);
    bottom_box.append(&dont_show);

    main_box.append(&bottom_box);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}
