// Main window for Winux Clock

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};

use crate::tabs;

pub fn build_window(app: &Application) {
    let header = HeaderBar::new();

    let stack = ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    // World Clock Tab
    let world_clock = tabs::world_clock::create_world_clock_tab();
    stack.add_titled(&world_clock, Some("world-clock"), "Relogio Mundial")
        .set_icon_name(Some("globe-symbolic"));

    // Alarm Tab
    let alarm = tabs::alarm::create_alarm_tab();
    stack.add_titled(&alarm, Some("alarm"), "Alarmes")
        .set_icon_name(Some("alarm-symbolic"));

    // Stopwatch Tab
    let stopwatch = tabs::stopwatch::create_stopwatch_tab();
    stack.add_titled(&stopwatch, Some("stopwatch"), "Cronometro")
        .set_icon_name(Some("stopwatch-symbolic"));

    // Timer Tab
    let timer = tabs::timer::create_timer_tab();
    stack.add_titled(&timer, Some("timer"), "Timer")
        .set_icon_name(Some("hourglass-symbolic"));

    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&stack);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Relogio")
        .default_width(500)
        .default_height(600)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));

    // Add CSS for animations
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_string(include_str!("../style.css"));

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.present();
}
