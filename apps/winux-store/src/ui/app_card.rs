//! Application card widget for displaying app information in grid/list views

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use std::cell::RefCell;

/// Data for an app card
#[derive(Default, Clone)]
pub struct AppCardData {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub installed: bool,
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct AppCard {
        pub data: RefCell<AppCardData>,
        pub icon_image: OnceCell<gtk4::Image>,
        pub name_label: OnceCell<gtk4::Label>,
        pub description_label: OnceCell<gtk4::Label>,
        pub install_button: OnceCell<gtk4::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppCard {
        const NAME: &'static str = "WinuxStoreAppCard";
        type Type = super::AppCard;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for AppCard {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for AppCard {}
    impl BoxImpl for AppCard {}
}

glib::wrapper! {
    pub struct AppCard(ObjectSubclass<imp::AppCard>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl AppCard {
    pub fn new(name: &str, description: &str, icon: &str) -> Self {
        let card: Self = glib::Object::builder().build();

        {
            let imp = card.imp();
            let mut data = imp.data.borrow_mut();
            data.name = name.to_string();
            data.description = description.to_string();
            data.icon = icon.to_string();
        }

        card.update_ui();
        card
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Configure the box
        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(8);
        self.add_css_class("card");
        self.set_width_request(180);
        self.set_height_request(200);
        self.set_margin_start(6);
        self.set_margin_end(6);
        self.set_margin_top(6);
        self.set_margin_bottom(6);

        // Inner box for padding
        let inner = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        inner.set_margin_start(12);
        inner.set_margin_end(12);
        inner.set_margin_top(12);
        inner.set_margin_bottom(12);
        inner.set_valign(gtk4::Align::Center);

        // App icon
        let icon = gtk4::Image::builder()
            .pixel_size(64)
            .halign(gtk4::Align::Center)
            .build();
        imp.icon_image.set(icon.clone()).unwrap();
        inner.append(&icon);

        // App name
        let name_label = gtk4::Label::builder()
            .halign(gtk4::Align::Center)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(20)
            .build();
        name_label.add_css_class("title-4");
        imp.name_label.set(name_label.clone()).unwrap();
        inner.append(&name_label);

        // App description
        let desc_label = gtk4::Label::builder()
            .halign(gtk4::Align::Center)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(25)
            .lines(2)
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::Word)
            .build();
        desc_label.add_css_class("dim-label");
        desc_label.add_css_class("caption");
        imp.description_label.set(desc_label.clone()).unwrap();
        inner.append(&desc_label);

        // Install/Open button
        let button = gtk4::Button::builder()
            .label("Install")
            .halign(gtk4::Align::Center)
            .margin_top(8)
            .build();
        button.add_css_class("suggested-action");
        button.add_css_class("pill");
        imp.install_button.set(button.clone()).unwrap();
        inner.append(&button);

        self.append(&inner);

        // Make the card clickable
        let gesture = gtk4::GestureClick::new();
        gesture.connect_released(|gesture, _, _, _| {
            let widget = gesture.widget();
            // Navigate to app details page
            tracing::debug!("Card clicked");
        });
        self.add_controller(gesture);
    }

    fn update_ui(&self) {
        let imp = self.imp();
        let data = imp.data.borrow();

        if let Some(icon) = imp.icon_image.get() {
            icon.set_icon_name(Some(&data.icon));
        }

        if let Some(label) = imp.name_label.get() {
            label.set_text(&data.name);
        }

        if let Some(label) = imp.description_label.get() {
            label.set_text(&data.description);
        }

        if let Some(button) = imp.install_button.get() {
            if data.installed {
                button.set_label("Open");
                button.remove_css_class("suggested-action");
            } else {
                button.set_label("Install");
                button.add_css_class("suggested-action");
            }
        }
    }

    pub fn set_installed(&self, installed: bool) {
        {
            let mut data = self.imp().data.borrow_mut();
            data.installed = installed;
        }
        self.update_ui();
    }

    pub fn get_name(&self) -> String {
        self.imp().data.borrow().name.clone()
    }

    pub fn connect_install_clicked<F: Fn(&Self) + 'static>(&self, callback: F) {
        if let Some(button) = self.imp().install_button.get() {
            let card = self.clone();
            button.connect_clicked(move |_| {
                callback(&card);
            });
        }
    }

    pub fn connect_card_clicked<F: Fn(&Self) + 'static>(&self, callback: F) {
        let card = self.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_released(move |_, _, _, _| {
            callback(&card);
        });
        self.add_controller(gesture);
    }
}

impl Default for AppCard {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}
