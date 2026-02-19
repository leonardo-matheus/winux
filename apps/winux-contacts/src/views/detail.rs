// Contact Detail View - Shows full contact information with actions

use crate::data::{storage::ContactStorage, Contact};
use crate::ui::{avatar::AvatarHelper, field_row::FieldRow};
use gtk4::prelude::*;
use gtk4::{
    Box, Button, Label, Orientation, ScrolledWindow, Separator, Widget,
};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct DetailView {
    container: Box,
    scroll: ScrolledWindow,
    content: Box,
    storage: Rc<RefCell<ContactStorage>>,
    current_contact: Rc<RefCell<Option<Contact>>>,
    on_edit_requested: Rc<RefCell<Option<Box<dyn Fn(Contact)>>>>,
    on_deleted: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl DetailView {
    pub fn new(storage: Rc<RefCell<ContactStorage>>) -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let content = Box::new(Orientation::Vertical, 0);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);

        scroll.set_child(Some(&content));
        container.append(&scroll);

        Self {
            container,
            scroll,
            content,
            storage,
            current_contact: Rc::new(RefCell::new(None)),
            on_edit_requested: Rc::new(RefCell::new(None)),
            on_deleted: Rc::new(RefCell::new(None)),
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn set_contact(&self, contact: Option<Contact>) {
        *self.current_contact.borrow_mut() = contact.clone();
        self.rebuild_content();
    }

    fn rebuild_content(&self) {
        // Clear existing content
        while let Some(child) = self.content.first_child() {
            self.content.remove(&child);
        }

        let contact = match self.current_contact.borrow().clone() {
            Some(c) => c,
            None => return,
        };

        // Header section with avatar and name
        let header = self.build_header(&contact);
        self.content.append(&header);

        // Action buttons
        let actions = self.build_actions(&contact);
        self.content.append(&actions);

        // Phone numbers
        if !contact.phones.is_empty() {
            let section = self.build_section("Phone", "call-start-symbolic");
            for phone in &contact.phones {
                let row = FieldRow::new(
                    &phone.number,
                    phone.phone_type.as_str(),
                    Some("call-start-symbolic"),
                );
                row.connect_action({
                    let number = phone.number.clone();
                    move || {
                        // Open dialer with number
                        let _ = open::that(format!("tel:{}", number));
                    }
                });
                row.connect_copy({
                    let number = phone.number.clone();
                    move || {
                        if let Some(display) = gdk4::Display::default() {
                            display.clipboard().set_text(&number);
                        }
                    }
                });
                section.append(row.widget());
            }
            self.content.append(&section);
        }

        // Email addresses
        if !contact.emails.is_empty() {
            let section = self.build_section("Email", "mail-unread-symbolic");
            for email in &contact.emails {
                let row = FieldRow::new(
                    &email.email,
                    email.email_type.as_str(),
                    Some("mail-send-symbolic"),
                );
                row.connect_action({
                    let email = email.email.clone();
                    move || {
                        let _ = open::that(format!("mailto:{}", email));
                    }
                });
                row.connect_copy({
                    let email = email.email.clone();
                    move || {
                        if let Some(display) = gdk4::Display::default() {
                            display.clipboard().set_text(&email);
                        }
                    }
                });
                section.append(row.widget());
            }
            self.content.append(&section);
        }

        // Addresses
        if !contact.addresses.is_empty() {
            let section = self.build_section("Address", "mark-location-symbolic");
            for address in &contact.addresses {
                if !address.is_empty() {
                    let formatted = address.formatted();
                    let row = FieldRow::new(
                        &formatted,
                        address.address_type.as_str(),
                        Some("map-symbolic"),
                    );
                    row.connect_action({
                        let addr = formatted.replace('\n', ", ");
                        move || {
                            let encoded = url::form_urlencoded::byte_serialize(addr.as_bytes())
                                .collect::<String>();
                            let _ = open::that(format!("https://maps.google.com/maps?q={}", encoded));
                        }
                    });
                    row.connect_copy({
                        let addr = formatted.clone();
                        move || {
                            if let Some(display) = gdk4::Display::default() {
                                display.clipboard().set_text(&addr);
                            }
                        }
                    });
                    section.append(row.widget());
                }
            }
            self.content.append(&section);
        }

        // Company info
        if contact.company.is_some() || contact.job_title.is_some() {
            let section = self.build_section("Work", "office-building-symbolic");

            if let Some(company) = &contact.company {
                let mut work_info = company.clone();
                if let Some(title) = &contact.job_title {
                    work_info = format!("{}\n{}", title, company);
                }
                let row = FieldRow::new(&work_info, "Company", None);
                section.append(row.widget());
            } else if let Some(title) = &contact.job_title {
                let row = FieldRow::new(title, "Job Title", None);
                section.append(row.widget());
            }

            self.content.append(&section);
        }

        // Birthday & Anniversary
        if contact.birthday.is_some() || contact.anniversary.is_some() {
            let section = self.build_section("Important Dates", "x-office-calendar-symbolic");

            if let Some(birthday) = &contact.birthday {
                let formatted = birthday.format("%B %d, %Y").to_string();
                let row = FieldRow::new(&formatted, "Birthday", None);
                section.append(row.widget());
            }

            if let Some(anniversary) = &contact.anniversary {
                let formatted = anniversary.format("%B %d, %Y").to_string();
                let row = FieldRow::new(&formatted, "Anniversary", None);
                section.append(row.widget());
            }

            self.content.append(&section);
        }

        // Website
        if let Some(website) = &contact.website {
            let section = self.build_section("Website", "web-browser-symbolic");
            let row = FieldRow::new(website, "URL", Some("web-browser-symbolic"));
            row.connect_action({
                let url = website.clone();
                move || {
                    let _ = open::that(&url);
                }
            });
            section.append(row.widget());
            self.content.append(&section);
        }

        // Notes
        if let Some(notes) = &contact.notes {
            let section = self.build_section("Notes", "document-text-symbolic");
            let label = Label::new(Some(notes));
            label.set_halign(gtk4::Align::Start);
            label.set_wrap(true);
            label.set_selectable(true);
            label.set_margin_start(16);
            label.set_margin_end(16);
            label.set_margin_top(8);
            label.set_margin_bottom(8);
            section.append(&label);
            self.content.append(&section);
        }

        // Groups
        if !contact.groups.is_empty() {
            let section = self.build_section("Groups", "folder-symbolic");
            let flow = gtk4::FlowBox::new();
            flow.set_selection_mode(gtk4::SelectionMode::None);
            flow.set_margin_start(16);
            flow.set_margin_end(16);
            flow.set_margin_top(8);
            flow.set_margin_bottom(8);

            for group in &contact.groups {
                let chip = Button::with_label(group);
                chip.add_css_class("pill");
                chip.add_css_class("dim-label");
                chip.set_can_focus(false);
                flow.append(&chip);
            }

            section.append(&flow);
            self.content.append(&section);
        }
    }

    fn build_header(&self, contact: &Contact) -> Box {
        let header = Box::new(Orientation::Vertical, 16);
        header.set_halign(gtk4::Align::Center);
        header.set_margin_bottom(24);

        // Large avatar
        let avatar = AvatarHelper::create_avatar(contact, 128);
        header.append(&avatar);

        // Name
        let name = Label::new(Some(&contact.display_name()));
        name.add_css_class("title-1");
        header.append(&name);

        // Company/Title subtitle
        if let Some(company) = &contact.company {
            let subtitle = if let Some(title) = &contact.job_title {
                format!("{} at {}", title, company)
            } else {
                company.clone()
            };
            let label = Label::new(Some(&subtitle));
            label.add_css_class("dim-label");
            header.append(&label);
        }

        header
    }

    fn build_actions(&self, contact: &Contact) -> Box {
        let actions = Box::new(Orientation::Horizontal, 12);
        actions.set_halign(gtk4::Align::Center);
        actions.set_margin_bottom(24);

        // Call button
        if let Some(phone) = contact.primary_phone() {
            let call_btn = Button::builder()
                .icon_name("call-start-symbolic")
                .tooltip_text("Call")
                .build();
            call_btn.add_css_class("circular");
            call_btn.add_css_class("suggested-action");

            let number = phone.number.clone();
            call_btn.connect_clicked(move |_| {
                let _ = open::that(format!("tel:{}", number));
            });

            actions.append(&call_btn);
        }

        // Message button
        if let Some(phone) = contact.primary_phone() {
            let msg_btn = Button::builder()
                .icon_name("chat-symbolic")
                .tooltip_text("Message")
                .build();
            msg_btn.add_css_class("circular");

            let number = phone.number.clone();
            msg_btn.connect_clicked(move |_| {
                let _ = open::that(format!("sms:{}", number));
            });

            actions.append(&msg_btn);
        }

        // Email button
        if let Some(email) = contact.primary_email() {
            let email_btn = Button::builder()
                .icon_name("mail-send-symbolic")
                .tooltip_text("Email")
                .build();
            email_btn.add_css_class("circular");

            let email_addr = email.email.clone();
            email_btn.connect_clicked(move |_| {
                let _ = open::that(format!("mailto:{}", email_addr));
            });

            actions.append(&email_btn);
        }

        // Favorite button
        let fav_icon = if contact.is_favorite {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        };
        let fav_btn = Button::builder()
            .icon_name(fav_icon)
            .tooltip_text("Toggle Favorite")
            .build();
        fav_btn.add_css_class("circular");

        let storage = self.storage.clone();
        let contact_id = contact.id.clone();
        let current = self.current_contact.clone();
        fav_btn.connect_clicked(move |btn| {
            if let Ok(mut storage) = storage.try_borrow_mut() {
                if let Ok(is_fav) = storage.toggle_favorite(&contact_id) {
                    let icon = if is_fav {
                        "starred-symbolic"
                    } else {
                        "non-starred-symbolic"
                    };
                    btn.set_icon_name(icon);

                    // Update current contact
                    if let Some(ref mut c) = *current.borrow_mut() {
                        c.is_favorite = is_fav;
                    }
                }
            }
        });

        actions.append(&fav_btn);

        // Edit button
        let edit_btn = Button::builder()
            .icon_name("document-edit-symbolic")
            .tooltip_text("Edit")
            .build();
        edit_btn.add_css_class("circular");

        let on_edit = self.on_edit_requested.clone();
        let current = self.current_contact.clone();
        edit_btn.connect_clicked(move |_| {
            if let Some(callback) = on_edit.borrow().as_ref() {
                if let Some(contact) = current.borrow().clone() {
                    callback(contact);
                }
            }
        });

        actions.append(&edit_btn);

        // Delete button
        let del_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Delete")
            .build();
        del_btn.add_css_class("circular");
        del_btn.add_css_class("destructive-action");

        let storage = self.storage.clone();
        let contact_id = contact.id.clone();
        let on_deleted = self.on_deleted.clone();
        del_btn.connect_clicked(move |_| {
            if let Ok(mut storage) = storage.try_borrow_mut() {
                if storage.delete_contact(&contact_id).is_ok() {
                    if let Some(callback) = on_deleted.borrow().as_ref() {
                        callback();
                    }
                }
            }
        });

        actions.append(&del_btn);

        actions
    }

    fn build_section(&self, title: &str, icon: &str) -> Box {
        let section = Box::new(Orientation::Vertical, 4);
        section.set_margin_top(16);

        // Section header
        let header = Box::new(Orientation::Horizontal, 8);
        header.set_margin_start(16);

        let icon_widget = gtk4::Image::from_icon_name(icon);
        icon_widget.add_css_class("dim-label");
        header.append(&icon_widget);

        let label = Label::new(Some(title));
        label.add_css_class("heading");
        label.add_css_class("dim-label");
        header.append(&label);

        section.append(&header);

        // Content frame
        let frame = adw::PreferencesGroup::new();
        section.append(&frame);

        section
    }

    pub fn connect_edit_requested<F: Fn(Contact) + 'static>(&self, callback: F) {
        *self.on_edit_requested.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_deleted<F: Fn() + 'static>(&self, callback: F) {
        *self.on_deleted.borrow_mut() = Some(Box::new(callback));
    }
}
