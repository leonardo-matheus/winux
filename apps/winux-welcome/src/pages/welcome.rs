// Winux Welcome - Welcome Page
// Initial welcome screen with animated Winux logo

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Picture};
use libadwaita as adw;
use adw::prelude::*;

pub fn create_page() -> Box {
    let page = Box::new(Orientation::Vertical, 24);
    page.set_valign(gtk4::Align::Center);
    page.set_halign(gtk4::Align::Center);
    page.set_margin_top(48);
    page.set_margin_bottom(48);
    page.set_margin_start(48);
    page.set_margin_end(48);

    // Logo container with animation class
    let logo_container = Box::new(Orientation::Vertical, 0);
    logo_container.add_css_class("welcome-logo");
    logo_container.set_halign(gtk4::Align::Center);

    // Winux logo (using system icon as placeholder)
    let logo = gtk4::Image::from_icon_name("computer-symbolic");
    logo.set_pixel_size(128);
    logo.add_css_class("welcome-logo-icon");
    logo_container.append(&logo);

    // Animated text logo
    let logo_text = Label::new(Some("WINUX"));
    logo_text.add_css_class("title-1");
    logo_text.add_css_class("welcome-logo-text");
    logo_container.append(&logo_text);

    page.append(&logo_container);

    // Welcome message
    let welcome_label = Label::new(Some("Bem-vindo ao Winux OS"));
    welcome_label.add_css_class("title-1");
    welcome_label.set_margin_top(24);
    page.append(&welcome_label);

    // Subtitle
    let subtitle = Label::new(Some("Vamos configurar seu sistema em poucos passos"));
    subtitle.add_css_class("title-3");
    subtitle.add_css_class("dim-label");
    page.append(&subtitle);

    // Description
    let description = Label::new(Some(
        "O Winux combina o melhor do Windows, macOS e Linux em uma \n\
         experiencia unica e personalizavel. Configure seu sistema \n\
         do jeito que voce prefere."
    ));
    description.add_css_class("body");
    description.set_justify(gtk4::Justification::Center);
    description.set_margin_top(16);
    page.append(&description);

    // Features preview
    let features_box = Box::new(Orientation::Horizontal, 32);
    features_box.set_halign(gtk4::Align::Center);
    features_box.set_margin_top(32);

    let features = [
        ("view-grid-symbolic", "Multiplas Interfaces"),
        ("preferences-desktop-appearance-symbolic", "Personalizacao"),
        ("emblem-system-symbolic", "Privacidade"),
    ];

    for (icon, text) in features {
        let feature = Box::new(Orientation::Vertical, 8);
        feature.set_halign(gtk4::Align::Center);

        let icon_widget = gtk4::Image::from_icon_name(icon);
        icon_widget.set_pixel_size(48);
        icon_widget.add_css_class("accent");
        feature.append(&icon_widget);

        let label = Label::new(Some(text));
        label.add_css_class("caption");
        feature.append(&label);

        features_box.append(&feature);
    }

    page.append(&features_box);

    // Start hint
    let hint = Label::new(Some("Clique em 'Proximo' para comecar"));
    hint.add_css_class("dim-label");
    hint.add_css_class("caption");
    hint.set_margin_top(32);
    page.append(&hint);

    page
}
