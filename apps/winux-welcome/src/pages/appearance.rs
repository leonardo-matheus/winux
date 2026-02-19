// Winux Welcome - Appearance Settings
// Theme (light/dark/auto), accent color, wallpaper selection

use gtk4::prelude::*;
use gtk4::{Box, CheckButton, FlowBox, Grid, Label, Orientation, Frame, Picture, Button};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::WelcomeState;

pub fn create_page(state: Rc<RefCell<WelcomeState>>) -> gtk4::ScrolledWindow {
    let page = Box::new(Orientation::Vertical, 24);
    page.set_margin_top(32);
    page.set_margin_bottom(32);
    page.set_margin_start(48);
    page.set_margin_end(48);

    // Title
    let title = Label::new(Some("Aparencia"));
    title.add_css_class("title-1");
    page.append(&title);

    let subtitle = Label::new(Some("Personalize a aparencia do seu sistema"));
    subtitle.add_css_class("dim-label");
    page.append(&subtitle);

    // Theme Section
    let theme_section = create_theme_section(state.clone());
    page.append(&theme_section);

    // Accent Color Section
    let color_section = create_accent_color_section(state.clone());
    page.append(&color_section);

    // Wallpaper Section
    let wallpaper_section = create_wallpaper_section(state.clone());
    page.append(&wallpaper_section);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_theme_section(state: Rc<RefCell<WelcomeState>>) -> Box {
    let section = Box::new(Orientation::Vertical, 12);
    section.set_margin_top(24);

    let header = Label::new(Some("Tema"));
    header.add_css_class("title-3");
    header.set_halign(gtk4::Align::Start);
    section.append(&header);

    let themes_box = Box::new(Orientation::Horizontal, 16);
    themes_box.set_halign(gtk4::Align::Center);
    themes_box.set_margin_top(12);

    // Radio button group
    let light_radio = CheckButton::new();
    let dark_radio = CheckButton::new();
    let auto_radio = CheckButton::new();

    dark_radio.set_group(Some(&light_radio));
    auto_radio.set_group(Some(&light_radio));

    // Light theme card
    let light_card = create_theme_card(
        "Claro",
        "Para ambientes iluminados",
        false,
        light_radio.clone(),
    );
    themes_box.append(&light_card);

    // Dark theme card
    let dark_card = create_theme_card(
        "Escuro",
        "Reduz cansaco visual",
        true,
        dark_radio.clone(),
    );
    dark_radio.set_active(true); // Default
    themes_box.append(&dark_card);

    // Auto theme card
    let auto_card = create_theme_card(
        "Automatico",
        "Segue o horario do dia",
        false, // Will show mixed preview
        auto_radio.clone(),
    );
    themes_box.append(&auto_card);

    section.append(&themes_box);

    // Connect signals
    let state_clone = state.clone();
    light_radio.connect_toggled(move |btn| {
        if btn.is_active() {
            state_clone.borrow_mut().theme = Some("Claro".to_string());
        }
    });

    let state_clone = state.clone();
    dark_radio.connect_toggled(move |btn| {
        if btn.is_active() {
            state_clone.borrow_mut().theme = Some("Escuro".to_string());
        }
    });

    let state_clone = state.clone();
    auto_radio.connect_toggled(move |btn| {
        if btn.is_active() {
            state_clone.borrow_mut().theme = Some("Automatico".to_string());
        }
    });

    // Set default
    state.borrow_mut().theme = Some("Escuro".to_string());

    section
}

fn create_theme_card(title: &str, description: &str, is_dark: bool, radio: CheckButton) -> Box {
    let card = Box::new(Orientation::Vertical, 8);
    card.add_css_class("card");
    card.set_size_request(180, 160);

    let inner = Box::new(Orientation::Vertical, 8);
    inner.set_margin_top(12);
    inner.set_margin_bottom(12);
    inner.set_margin_start(12);
    inner.set_margin_end(12);

    // Theme preview
    let preview = Frame::new(None);
    preview.set_size_request(156, 80);

    let preview_box = Box::new(Orientation::Vertical, 4);
    if is_dark {
        preview_box.add_css_class("view");
        // Simulate dark theme
    } else {
        preview_box.add_css_class("view");
    }
    preview_box.set_margin_top(8);
    preview_box.set_margin_bottom(8);
    preview_box.set_margin_start(8);
    preview_box.set_margin_end(8);

    // Mock window in preview
    let mock_header = Box::new(Orientation::Horizontal, 0);
    mock_header.add_css_class("toolbar");
    mock_header.set_size_request(-1, 16);
    preview_box.append(&mock_header);

    let mock_content = Box::new(Orientation::Vertical, 0);
    mock_content.set_vexpand(true);
    preview_box.append(&mock_content);

    preview.set_child(Some(&preview_box));
    inner.append(&preview);

    // Title
    let title_label = Label::new(Some(title));
    title_label.add_css_class("heading");
    inner.append(&title_label);

    // Description
    let desc = Label::new(Some(description));
    desc.add_css_class("caption");
    desc.add_css_class("dim-label");
    inner.append(&desc);

    // Radio
    let radio_box = Box::new(Orientation::Horizontal, 4);
    radio_box.set_halign(gtk4::Align::Center);
    radio_box.append(&radio);
    inner.append(&radio_box);

    card.append(&inner);

    // Make card clickable
    let gesture = gtk4::GestureClick::new();
    let radio_clone = radio.clone();
    gesture.connect_released(move |_, _, _, _| {
        radio_clone.set_active(true);
    });
    card.add_controller(gesture);

    card
}

fn create_accent_color_section(state: Rc<RefCell<WelcomeState>>) -> Box {
    let section = Box::new(Orientation::Vertical, 12);
    section.set_margin_top(24);

    let header = Label::new(Some("Cor de Destaque"));
    header.add_css_class("title-3");
    header.set_halign(gtk4::Align::Start);
    section.append(&header);

    let colors_box = Box::new(Orientation::Horizontal, 12);
    colors_box.set_halign(gtk4::Align::Center);
    colors_box.set_margin_top(12);

    let colors = [
        ("Azul", "#58a6ff", true),
        ("Verde", "#3fb950", false),
        ("Roxo", "#a371f7", false),
        ("Rosa", "#f778ba", false),
        ("Laranja", "#f0883e", false),
        ("Vermelho", "#f85149", false),
        ("Ciano", "#79c0ff", false),
    ];

    // Radio group
    let first_radio = CheckButton::new();
    first_radio.set_active(true);

    for (i, (name, color, is_default)) in colors.iter().enumerate() {
        let color_box = Box::new(Orientation::Vertical, 6);
        color_box.set_halign(gtk4::Align::Center);

        // Color swatch
        let swatch = gtk4::DrawingArea::new();
        swatch.set_size_request(48, 48);
        swatch.add_css_class("circular");
        swatch.add_css_class("card");

        // Set background color via CSS
        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "drawingarea {{ background-color: {}; border-radius: 24px; }}",
            color
        );
        css_provider.load_from_data(&css);
        swatch.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        color_box.append(&swatch);

        // Radio button
        let radio = if i == 0 {
            first_radio.clone()
        } else {
            let r = CheckButton::new();
            r.set_group(Some(&first_radio));
            r
        };

        if *is_default {
            radio.set_active(true);
        }

        color_box.append(&radio);

        // Label
        let label = Label::new(Some(*name));
        label.add_css_class("caption");
        color_box.append(&label);

        // Connect signal
        let state_clone = state.clone();
        let color_name = name.to_string();
        radio.connect_toggled(move |btn| {
            if btn.is_active() {
                state_clone.borrow_mut().accent_color = Some(color_name.clone());
            }
        });

        // Make swatch clickable
        let gesture = gtk4::GestureClick::new();
        let radio_clone = radio.clone();
        gesture.connect_released(move |_, _, _, _| {
            radio_clone.set_active(true);
        });
        swatch.add_controller(gesture);

        colors_box.append(&color_box);
    }

    section.append(&colors_box);

    // Set default
    state.borrow_mut().accent_color = Some("Azul".to_string());

    section
}

fn create_wallpaper_section(state: Rc<RefCell<WelcomeState>>) -> Box {
    let section = Box::new(Orientation::Vertical, 12);
    section.set_margin_top(24);

    let header = Label::new(Some("Papel de Parede"));
    header.add_css_class("title-3");
    header.set_halign(gtk4::Align::Start);
    section.append(&header);

    let wallpapers_box = Box::new(Orientation::Horizontal, 12);
    wallpapers_box.set_halign(gtk4::Align::Center);
    wallpapers_box.set_margin_top(12);

    let wallpapers = [
        ("Winux Aurora", "wallpaper-aurora", true),
        ("Winux Dark", "wallpaper-dark", false),
        ("Winux Gradient", "wallpaper-gradient", false),
        ("Winux Minimal", "wallpaper-minimal", false),
    ];

    let first_radio = CheckButton::new();
    first_radio.set_active(true);

    for (i, (name, _id, is_default)) in wallpapers.iter().enumerate() {
        let wp_card = Box::new(Orientation::Vertical, 8);
        wp_card.add_css_class("card");
        wp_card.set_size_request(140, 120);

        let inner = Box::new(Orientation::Vertical, 6);
        inner.set_margin_top(8);
        inner.set_margin_bottom(8);
        inner.set_margin_start(8);
        inner.set_margin_end(8);

        // Wallpaper preview placeholder
        let preview = Frame::new(None);
        preview.set_size_request(124, 70);
        preview.add_css_class("view");

        // Placeholder gradient
        let preview_inner = Box::new(Orientation::Vertical, 0);
        preview_inner.add_css_class("view");
        preview_inner.set_vexpand(true);

        // Add gradient colors based on wallpaper
        let gradient_css = match i {
            0 => "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);",
            1 => "background: linear-gradient(135deg, #0d0d0d 0%, #1a1a1a 50%, #2d2d2d 100%);",
            2 => "background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);",
            _ => "background: linear-gradient(135deg, #232526 0%, #414345 100%);",
        };

        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(&format!("box {{ {} }}", gradient_css));
        preview_inner.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        preview.set_child(Some(&preview_inner));
        inner.append(&preview);

        // Radio and label
        let radio_box = Box::new(Orientation::Horizontal, 4);
        radio_box.set_halign(gtk4::Align::Center);

        let radio = if i == 0 {
            first_radio.clone()
        } else {
            let r = CheckButton::new();
            r.set_group(Some(&first_radio));
            r
        };

        if *is_default {
            radio.set_active(true);
        }

        radio_box.append(&radio);

        let label = Label::new(Some(*name));
        label.add_css_class("caption");
        radio_box.append(&label);

        inner.append(&radio_box);
        wp_card.append(&inner);

        // Connect signal
        let state_clone = state.clone();
        let wp_name = name.to_string();
        radio.connect_toggled(move |btn| {
            if btn.is_active() {
                state_clone.borrow_mut().wallpaper = Some(wp_name.clone());
            }
        });

        // Make card clickable
        let gesture = gtk4::GestureClick::new();
        let radio_clone = radio.clone();
        gesture.connect_released(move |_, _, _, _| {
            radio_clone.set_active(true);
        });
        wp_card.add_controller(gesture);

        wallpapers_box.append(&wp_card);
    }

    section.append(&wallpapers_box);

    // Custom wallpaper button
    let custom_btn = Button::with_label("Escolher Imagem Personalizada...");
    custom_btn.add_css_class("flat");
    custom_btn.set_halign(gtk4::Align::Center);
    custom_btn.set_margin_top(12);
    section.append(&custom_btn);

    // Set default
    state.borrow_mut().wallpaper = Some("Winux Aurora".to_string());

    section
}
