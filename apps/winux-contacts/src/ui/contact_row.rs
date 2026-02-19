// Contact Row - List item widget for contacts

use crate::data::Contact;
use crate::ui::avatar::AvatarHelper;
use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Widget};
use libadwaita as adw;
use adw::prelude::*;

pub struct ContactRow {
    container: Box,
    contact_id: String,
}

impl ContactRow {
    pub fn new(contact: &Contact) -> Self {
        let container = Box::new(Orientation::Horizontal, 12);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(8);
        container.set_margin_bottom(8);

        // Avatar
        let avatar = AvatarHelper::create_avatar(contact, 40);
        container.append(&avatar);

        // Name and subtitle
        let info_box = Box::new(Orientation::Vertical, 2);
        info_box.set_valign(gtk4::Align::Center);
        info_box.set_hexpand(true);

        let name_label = Label::new(Some(&contact.display_name()));
        name_label.set_halign(gtk4::Align::Start);
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name_label.add_css_class("heading");
        info_box.append(&name_label);

        // Subtitle - show company, phone, or email
        let subtitle = Self::get_subtitle(contact);
        if !subtitle.is_empty() {
            let subtitle_label = Label::new(Some(&subtitle));
            subtitle_label.set_halign(gtk4::Align::Start);
            subtitle_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            subtitle_label.add_css_class("dim-label");
            subtitle_label.add_css_class("caption");
            info_box.append(&subtitle_label);
        }

        container.append(&info_box);

        // Favorite star
        if contact.is_favorite {
            let star = gtk4::Image::from_icon_name("starred-symbolic");
            star.add_css_class("accent");
            container.append(&star);
        }

        Self {
            container,
            contact_id: contact.id.clone(),
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn contact_id(&self) -> &str {
        &self.contact_id
    }

    fn get_subtitle(contact: &Contact) -> String {
        if let Some(company) = &contact.company {
            if let Some(title) = &contact.job_title {
                return format!("{}, {}", title, company);
            }
            return company.clone();
        }

        if let Some(phone) = contact.primary_phone() {
            return phone.number.clone();
        }

        if let Some(email) = contact.primary_email() {
            return email.email.clone();
        }

        String::new()
    }
}

/// A more compact contact row for use in selection dialogs
pub struct CompactContactRow {
    container: Box,
    contact_id: String,
}

impl CompactContactRow {
    pub fn new(contact: &Contact) -> Self {
        let container = Box::new(Orientation::Horizontal, 8);
        container.set_margin_start(8);
        container.set_margin_end(8);
        container.set_margin_top(4);
        container.set_margin_bottom(4);

        // Small avatar
        let avatar = AvatarHelper::create_avatar(contact, 32);
        container.append(&avatar);

        // Name only
        let name_label = Label::new(Some(&contact.display_name()));
        name_label.set_halign(gtk4::Align::Start);
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name_label.set_hexpand(true);
        container.append(&name_label);

        Self {
            container,
            contact_id: contact.id.clone(),
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn contact_id(&self) -> &str {
        &self.contact_id
    }
}
