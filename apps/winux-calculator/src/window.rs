// Winux Calculator - Main Window
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};
use std::cell::RefCell;
use std::rc::Rc;

use crate::modes::{BasicMode, ScientificMode, ProgrammerMode, ConverterMode};
use crate::engine::Calculator;
use crate::ui::{Display, History};

pub fn build_ui(app: &Application) {
    let calculator = Rc::new(RefCell::new(Calculator::new()));
    let history = Rc::new(RefCell::new(History::new()));

    let header = HeaderBar::new();

    let stack = ViewStack::new();
    stack.set_vexpand(true);

    // Basic Mode
    let basic_mode = BasicMode::new(calculator.clone(), history.clone());
    stack.add_titled(&basic_mode.widget(), Some("basic"), "Basica")
        .set_icon_name(Some("accessories-calculator-symbolic"));

    // Scientific Mode
    let scientific_mode = ScientificMode::new(calculator.clone(), history.clone());
    stack.add_titled(&scientific_mode.widget(), Some("scientific"), "Cientifica")
        .set_icon_name(Some("accessories-calculator-symbolic"));

    // Programmer Mode
    let programmer_mode = ProgrammerMode::new(calculator.clone(), history.clone());
    stack.add_titled(&programmer_mode.widget(), Some("programmer"), "Programador")
        .set_icon_name(Some("utilities-terminal-symbolic"));

    // Converter Mode
    let converter_mode = ConverterMode::new();
    stack.add_titled(&converter_mode.widget(), Some("converter"), "Conversor")
        .set_icon_name(Some("view-refresh-symbolic"));

    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    // History sidebar button
    let history_btn = gtk4::ToggleButton::new();
    history_btn.set_icon_name("document-open-recent-symbolic");
    history_btn.set_tooltip_text(Some("Historico"));
    header.pack_end(&history_btn);

    let main_box = Box::new(Orientation::Vertical, 0);

    // Create a horizontal box for content and history
    let content_box = Box::new(Orientation::Horizontal, 0);
    content_box.append(&stack);

    // History panel (initially hidden)
    let history_panel = history.borrow().widget();
    history_panel.set_visible(false);
    content_box.append(&history_panel);

    // Connect history toggle
    let history_panel_clone = history_panel.clone();
    history_btn.connect_toggled(move |btn| {
        history_panel_clone.set_visible(btn.is_active());
    });

    main_box.append(&content_box);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Calculadora")
        .default_width(400)
        .default_height(600)
        .content(&main_box)
        .build();

    // Setup keyboard shortcuts
    setup_keyboard_shortcuts(&window, calculator.clone());

    window.set_titlebar(Some(&header));
    window.present();
}

fn setup_keyboard_shortcuts(window: &ApplicationWindow, _calculator: Rc<RefCell<Calculator>>) {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(move |_, key, _keycode, _state| {
        match key {
            gdk4::Key::Escape => {
                // Clear could be triggered here
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed
        }
    });

    window.add_controller(controller);
}
