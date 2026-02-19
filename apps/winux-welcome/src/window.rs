// Winux Welcome - Main Window
// Manages page navigation with smooth transitions

use gtk4::prelude::*;
use gtk4::{Application, Box, Button, Orientation, Stack, StackTransitionType};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, Carousel, CarouselIndicatorDots};
use std::cell::RefCell;
use std::rc::Rc;

use crate::pages;
use crate::ui;

pub fn build_ui(app: &Application) {
    // Header bar (minimal for welcome experience)
    let header = HeaderBar::new();
    header.add_css_class("flat");

    let title = adw::WindowTitle::new("Bem-vindo ao Winux", "");
    header.set_title_widget(Some(&title));

    // Skip button on header
    let skip_btn = Button::with_label("Pular");
    skip_btn.add_css_class("flat");
    header.pack_end(&skip_btn);

    // Main carousel for pages
    let carousel = Carousel::builder()
        .allow_scroll_wheel(false)
        .allow_mouse_drag(false)
        .hexpand(true)
        .vexpand(true)
        .build();

    // State for tracking selections
    let state = Rc::new(RefCell::new(WelcomeState::default()));

    // Create all pages
    let welcome_page = pages::welcome::create_page();
    carousel.append(&welcome_page);

    let desktop_mode_page = pages::desktop_mode::create_page(state.clone());
    carousel.append(&desktop_mode_page);

    let appearance_page = pages::appearance::create_page(state.clone());
    carousel.append(&appearance_page);

    let apps_page = pages::apps::create_page(state.clone());
    carousel.append(&apps_page);

    let dev_setup_page = pages::dev_setup::create_page(state.clone());
    carousel.append(&dev_setup_page);

    let gaming_page = pages::gaming::create_page(state.clone());
    carousel.append(&gaming_page);

    let privacy_page = pages::privacy::create_page(state.clone());
    carousel.append(&privacy_page);

    let finish_page = pages::finish::create_page(state.clone());
    carousel.append(&finish_page);

    // Page indicator dots
    let indicator = CarouselIndicatorDots::builder()
        .carousel(&carousel)
        .build();

    // Navigation buttons
    let nav_box = Box::new(Orientation::Horizontal, 12);
    nav_box.set_halign(gtk4::Align::Center);
    nav_box.set_margin_bottom(24);
    nav_box.set_margin_top(12);

    let back_btn = Button::with_label("Voltar");
    back_btn.add_css_class("pill");
    back_btn.set_sensitive(false);
    nav_box.append(&back_btn);

    let next_btn = Button::with_label("Proximo");
    next_btn.add_css_class("pill");
    next_btn.add_css_class("suggested-action");
    nav_box.append(&next_btn);

    // Progress bar
    let progress = ui::progress_bar::create_progress_bar(8);

    // Main layout
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&carousel);
    main_box.append(&indicator);
    main_box.append(&progress);
    main_box.append(&nav_box);

    // Window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Bem-vindo ao Winux")
        .default_width(900)
        .default_height(700)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));

    // Navigation logic
    let carousel_clone = carousel.clone();
    let back_btn_clone = back_btn.clone();
    let next_btn_clone = next_btn.clone();
    let progress_clone = progress.clone();
    let window_clone = window.clone();

    next_btn.connect_clicked(move |btn| {
        let current = carousel_clone.position() as u32;
        let total = carousel_clone.n_pages();

        if current < total - 1 {
            carousel_clone.scroll_to(&carousel_clone.nth_page(current + 1), true);
            back_btn_clone.set_sensitive(true);
            ui::progress_bar::update_progress(&progress_clone, (current + 2) as usize, total as usize);

            if current + 1 == total - 1 {
                btn.set_label("Concluir");
            }
        } else {
            // Finish - close the welcome app
            window_clone.close();
        }
    });

    let carousel_clone2 = carousel.clone();
    let next_btn_clone2 = next_btn.clone();
    let progress_clone2 = progress.clone();

    back_btn.connect_clicked(move |btn| {
        let current = carousel_clone2.position() as u32;

        if current > 0 {
            carousel_clone2.scroll_to(&carousel_clone2.nth_page(current - 1), true);
            next_btn_clone2.set_label("Proximo");
            ui::progress_bar::update_progress(&progress_clone2, current as usize, carousel_clone2.n_pages() as usize);

            if current == 1 {
                btn.set_sensitive(false);
            }
        }
    });

    // Skip button closes the app
    let window_clone2 = window.clone();
    skip_btn.connect_clicked(move |_| {
        window_clone2.close();
    });

    // Add custom CSS
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(include_str!("../style.css"));
    gtk4::style_context_add_provider_for_display(
        &window.display(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.present();
}

/// State structure to track user selections throughout the wizard
#[derive(Default, Clone)]
pub struct WelcomeState {
    pub desktop_mode: Option<String>,
    pub theme: Option<String>,
    pub accent_color: Option<String>,
    pub wallpaper: Option<String>,
    pub selected_apps: Vec<String>,
    pub dev_languages: Vec<String>,
    pub dev_ides: Vec<String>,
    pub gaming_platforms: Vec<String>,
    pub gaming_emulators: Vec<String>,
    pub telemetry_enabled: bool,
    pub crash_reports_enabled: bool,
}

impl WelcomeState {
    pub fn summary(&self) -> String {
        let mut summary = String::new();

        if let Some(ref mode) = self.desktop_mode {
            summary.push_str(&format!("Estilo: {}\n", mode));
        }

        if let Some(ref theme) = self.theme {
            summary.push_str(&format!("Tema: {}\n", theme));
        }

        if let Some(ref color) = self.accent_color {
            summary.push_str(&format!("Cor de destaque: {}\n", color));
        }

        if !self.selected_apps.is_empty() {
            summary.push_str(&format!("Apps: {}\n", self.selected_apps.join(", ")));
        }

        if !self.dev_languages.is_empty() {
            summary.push_str(&format!("Linguagens: {}\n", self.dev_languages.join(", ")));
        }

        if !self.gaming_platforms.is_empty() {
            summary.push_str(&format!("Gaming: {}\n", self.gaming_platforms.join(", ")));
        }

        summary
    }
}
