//! Main application window

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::pages::{SeeingPage, HearingPage, TypingPage, PointingPage, ZoomPage};

/// Main accessibility settings window
pub struct AccessibilityWindow {
    window: adw::ApplicationWindow,
}

impl AccessibilityWindow {
    /// Create a new accessibility window
    pub fn new(app: &Application) -> Self {
        let header = adw::HeaderBar::new();

        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);

        // Seeing Page (Vision)
        let seeing_page = SeeingPage::new();
        stack.add_titled(seeing_page.widget(), Some("seeing"), "Visao")
            .set_icon_name(Some("eye-open-negative-filled-symbolic"));

        // Hearing Page
        let hearing_page = HearingPage::new();
        stack.add_titled(hearing_page.widget(), Some("hearing"), "Audicao")
            .set_icon_name(Some("audio-speakers-symbolic"));

        // Typing Page
        let typing_page = TypingPage::new();
        stack.add_titled(typing_page.widget(), Some("typing"), "Digitacao")
            .set_icon_name(Some("input-keyboard-symbolic"));

        // Pointing Page (Mouse)
        let pointing_page = PointingPage::new();
        stack.add_titled(pointing_page.widget(), Some("pointing"), "Mouse")
            .set_icon_name(Some("input-mouse-symbolic"));

        // Zoom Page
        let zoom_page = ZoomPage::new();
        stack.add_titled(zoom_page.widget(), Some("zoom"), "Zoom")
            .set_icon_name(Some("find-location-symbolic"));

        let switcher = adw::ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header.set_title_widget(Some(&switcher));

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Acessibilidade")
            .default_width(900)
            .default_height(700)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        Self { window }
    }

    /// Present the window
    pub fn present(&self) {
        self.window.present();
    }
}
