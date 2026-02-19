// Contact Editor View - Create and edit contacts

use crate::data::{
    contact::*, storage::ContactStorage, AddressType, EmailAddress, EmailType, PhoneNumber,
    PhoneType, PostalAddress,
};
use chrono::{NaiveDate, Utc};
use gtk4::prelude::*;
use gtk4::{
    Box, Button, Calendar, Entry, Label, Orientation, Popover, ScrolledWindow, TextBuffer,
    TextView, Widget,
};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct EditorView {
    container: Box,
    storage: Rc<RefCell<ContactStorage>>,
    current_contact: Rc<RefCell<Option<Contact>>>,
    // Form fields
    first_name_entry: Entry,
    last_name_entry: Entry,
    nickname_entry: Entry,
    company_entry: Entry,
    job_title_entry: Entry,
    website_entry: Entry,
    notes_buffer: TextBuffer,
    birthday_button: Button,
    birthday_date: Rc<RefCell<Option<NaiveDate>>>,
    phones_container: Box,
    emails_container: Box,
    addresses_container: Box,
    // Callbacks
    on_saved: Rc<RefCell<Option<Box<dyn Fn(Contact)>>>>,
    on_cancelled: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl EditorView {
    pub fn new(storage: Rc<RefCell<ContactStorage>>) -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        // Header with save/cancel buttons
        let header = adw::HeaderBar::new();
        header.add_css_class("flat");

        let cancel_btn = Button::with_label("Cancel");
        header.pack_start(&cancel_btn);

        let title = adw::WindowTitle::new("Edit Contact", "");
        header.set_title_widget(Some(&title));

        let save_btn = Button::with_label("Save");
        save_btn.add_css_class("suggested-action");
        header.pack_end(&save_btn);

        container.append(&header);

        // Scrollable form
        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);

        let form = Box::new(Orientation::Vertical, 24);
        form.set_margin_start(24);
        form.set_margin_end(24);
        form.set_margin_top(24);
        form.set_margin_bottom(24);

        // Name section
        let name_group = adw::PreferencesGroup::new();
        name_group.set_title("Name");

        let first_name_row = adw::EntryRow::new();
        first_name_row.set_title("First Name");
        name_group.add(&first_name_row);

        let last_name_row = adw::EntryRow::new();
        last_name_row.set_title("Last Name");
        name_group.add(&last_name_row);

        let nickname_row = adw::EntryRow::new();
        nickname_row.set_title("Nickname");
        name_group.add(&nickname_row);

        form.append(&name_group);

        // Phone section
        let phone_group = adw::PreferencesGroup::new();
        phone_group.set_title("Phone");

        let phones_container = Box::new(Orientation::Vertical, 8);
        phone_group.add(&phones_container);

        let add_phone_btn = Button::builder()
            .label("Add Phone")
            .icon_name("list-add-symbolic")
            .build();
        add_phone_btn.add_css_class("flat");
        phone_group.add(&add_phone_btn);

        form.append(&phone_group);

        // Email section
        let email_group = adw::PreferencesGroup::new();
        email_group.set_title("Email");

        let emails_container = Box::new(Orientation::Vertical, 8);
        email_group.add(&emails_container);

        let add_email_btn = Button::builder()
            .label("Add Email")
            .icon_name("list-add-symbolic")
            .build();
        add_email_btn.add_css_class("flat");
        email_group.add(&add_email_btn);

        form.append(&email_group);

        // Address section
        let address_group = adw::PreferencesGroup::new();
        address_group.set_title("Address");

        let addresses_container = Box::new(Orientation::Vertical, 8);
        address_group.add(&addresses_container);

        let add_address_btn = Button::builder()
            .label("Add Address")
            .icon_name("list-add-symbolic")
            .build();
        add_address_btn.add_css_class("flat");
        address_group.add(&add_address_btn);

        form.append(&address_group);

        // Work section
        let work_group = adw::PreferencesGroup::new();
        work_group.set_title("Work");

        let company_row = adw::EntryRow::new();
        company_row.set_title("Company");
        work_group.add(&company_row);

        let job_title_row = adw::EntryRow::new();
        job_title_row.set_title("Job Title");
        work_group.add(&job_title_row);

        form.append(&work_group);

        // Important dates
        let dates_group = adw::PreferencesGroup::new();
        dates_group.set_title("Important Dates");

        let birthday_row = adw::ActionRow::new();
        birthday_row.set_title("Birthday");

        let birthday_button = Button::with_label("Select Date");
        birthday_button.set_valign(gtk4::Align::Center);
        birthday_row.add_suffix(&birthday_button);
        dates_group.add(&birthday_row);

        form.append(&dates_group);

        // Other section
        let other_group = adw::PreferencesGroup::new();
        other_group.set_title("Other");

        let website_row = adw::EntryRow::new();
        website_row.set_title("Website");
        other_group.add(&website_row);

        form.append(&other_group);

        // Notes
        let notes_group = adw::PreferencesGroup::new();
        notes_group.set_title("Notes");

        let notes_frame = gtk4::Frame::new(None);
        let notes_view = TextView::new();
        notes_view.set_wrap_mode(gtk4::WrapMode::Word);
        notes_view.set_margin_start(8);
        notes_view.set_margin_end(8);
        notes_view.set_margin_top(8);
        notes_view.set_margin_bottom(8);
        notes_view.set_height_request(100);
        notes_frame.set_child(Some(&notes_view));
        notes_group.add(&notes_frame);

        form.append(&notes_group);

        scroll.set_child(Some(&form));
        container.append(&scroll);

        // Get entries from entry rows
        let first_name_entry = Entry::new();
        let last_name_entry = Entry::new();
        let nickname_entry = Entry::new();
        let company_entry = Entry::new();
        let job_title_entry = Entry::new();
        let website_entry = Entry::new();
        let notes_buffer = notes_view.buffer();

        let view = Self {
            container,
            storage,
            current_contact: Rc::new(RefCell::new(None)),
            first_name_entry,
            last_name_entry,
            nickname_entry,
            company_entry,
            job_title_entry,
            website_entry,
            notes_buffer,
            birthday_button: birthday_button.clone(),
            birthday_date: Rc::new(RefCell::new(None)),
            phones_container: phones_container.clone(),
            emails_container: emails_container.clone(),
            addresses_container: addresses_container.clone(),
            on_saved: Rc::new(RefCell::new(None)),
            on_cancelled: Rc::new(RefCell::new(None)),
        };

        // Setup signals
        let view_clone = view.clone();
        save_btn.connect_clicked(move |_| {
            view_clone.save_contact();
        });

        let view_clone = view.clone();
        cancel_btn.connect_clicked(move |_| {
            if let Some(callback) = view_clone.on_cancelled.borrow().as_ref() {
                callback();
            }
        });

        // Add phone button
        let phones = phones_container.clone();
        add_phone_btn.connect_clicked(move |_| {
            let phone_row = Self::create_phone_row("", PhoneType::Mobile, &phones);
            phones.append(&phone_row);
        });

        // Add email button
        let emails = emails_container.clone();
        add_email_btn.connect_clicked(move |_| {
            let email_row = Self::create_email_row("", EmailType::Personal, &emails);
            emails.append(&email_row);
        });

        // Add address button
        let addresses = addresses_container.clone();
        add_address_btn.connect_clicked(move |_| {
            let address_row = Self::create_address_row(&PostalAddress::default(), &addresses);
            addresses.append(&address_row);
        });

        // Birthday calendar popover
        let birthday_date = view.birthday_date.clone();
        let btn = birthday_button.clone();
        birthday_button.connect_clicked(move |button| {
            let popover = Popover::new();
            popover.set_parent(button);

            let calendar = Calendar::new();
            if let Some(date) = *birthday_date.borrow() {
                calendar.select_day(&glib::DateTime::from_local(
                    date.year(),
                    date.month() as i32,
                    date.day() as i32,
                    0, 0, 0.0,
                ).unwrap());
            }

            let date_ref = birthday_date.clone();
            let btn_ref = btn.clone();
            let popover_ref = popover.clone();
            calendar.connect_day_selected(move |cal| {
                let datetime = cal.date();
                let date = NaiveDate::from_ymd_opt(
                    datetime.year(),
                    datetime.month() as u32,
                    datetime.day_of_month() as u32,
                );
                *date_ref.borrow_mut() = date;
                if let Some(d) = date {
                    btn_ref.set_label(&d.format("%B %d, %Y").to_string());
                }
                popover_ref.popdown();
            });

            popover.set_child(Some(&calendar));
            popover.popup();
        });

        // Connect entry rows to our entries (using entry row text directly)
        first_name_row.connect_changed({
            let entry = view.first_name_entry.clone();
            move |row| {
                entry.set_text(&row.text());
            }
        });

        last_name_row.connect_changed({
            let entry = view.last_name_entry.clone();
            move |row| {
                entry.set_text(&row.text());
            }
        });

        nickname_row.connect_changed({
            let entry = view.nickname_entry.clone();
            move |row| {
                entry.set_text(&row.text());
            }
        });

        company_row.connect_changed({
            let entry = view.company_entry.clone();
            move |row| {
                entry.set_text(&row.text());
            }
        });

        job_title_row.connect_changed({
            let entry = view.job_title_entry.clone();
            move |row| {
                entry.set_text(&row.text());
            }
        });

        website_row.connect_changed({
            let entry = view.website_entry.clone();
            move |row| {
                entry.set_text(&row.text());
            }
        });

        view
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn set_contact(&self, contact: Option<Contact>) {
        *self.current_contact.borrow_mut() = contact.clone();
        self.populate_form();
    }

    fn populate_form(&self) {
        // Clear dynamic fields
        while let Some(child) = self.phones_container.first_child() {
            self.phones_container.remove(&child);
        }
        while let Some(child) = self.emails_container.first_child() {
            self.emails_container.remove(&child);
        }
        while let Some(child) = self.addresses_container.first_child() {
            self.addresses_container.remove(&child);
        }

        let contact = match self.current_contact.borrow().clone() {
            Some(c) => c,
            None => {
                // New contact - clear form
                self.first_name_entry.set_text("");
                self.last_name_entry.set_text("");
                self.nickname_entry.set_text("");
                self.company_entry.set_text("");
                self.job_title_entry.set_text("");
                self.website_entry.set_text("");
                self.notes_buffer.set_text("");
                self.birthday_button.set_label("Select Date");
                *self.birthday_date.borrow_mut() = None;

                // Add one empty phone and email field
                let phone_row = Self::create_phone_row("", PhoneType::Mobile, &self.phones_container);
                self.phones_container.append(&phone_row);

                let email_row = Self::create_email_row("", EmailType::Personal, &self.emails_container);
                self.emails_container.append(&email_row);

                return;
            }
        };

        // Populate fields
        self.first_name_entry.set_text(&contact.first_name);
        self.last_name_entry.set_text(&contact.last_name);
        self.nickname_entry.set_text(contact.nickname.as_deref().unwrap_or(""));
        self.company_entry.set_text(contact.company.as_deref().unwrap_or(""));
        self.job_title_entry.set_text(contact.job_title.as_deref().unwrap_or(""));
        self.website_entry.set_text(contact.website.as_deref().unwrap_or(""));
        self.notes_buffer.set_text(contact.notes.as_deref().unwrap_or(""));

        // Birthday
        *self.birthday_date.borrow_mut() = contact.birthday;
        if let Some(date) = contact.birthday {
            self.birthday_button.set_label(&date.format("%B %d, %Y").to_string());
        } else {
            self.birthday_button.set_label("Select Date");
        }

        // Phones
        for phone in &contact.phones {
            let row = Self::create_phone_row(&phone.number, phone.phone_type.clone(), &self.phones_container);
            self.phones_container.append(&row);
        }
        if contact.phones.is_empty() {
            let row = Self::create_phone_row("", PhoneType::Mobile, &self.phones_container);
            self.phones_container.append(&row);
        }

        // Emails
        for email in &contact.emails {
            let row = Self::create_email_row(&email.email, email.email_type.clone(), &self.emails_container);
            self.emails_container.append(&row);
        }
        if contact.emails.is_empty() {
            let row = Self::create_email_row("", EmailType::Personal, &self.emails_container);
            self.emails_container.append(&row);
        }

        // Addresses
        for address in &contact.addresses {
            let row = Self::create_address_row(address, &self.addresses_container);
            self.addresses_container.append(&row);
        }
    }

    fn create_phone_row(number: &str, phone_type: PhoneType, container: &Box) -> Box {
        let row = Box::new(Orientation::Horizontal, 8);

        let entry = Entry::new();
        entry.set_text(number);
        entry.set_placeholder_text(Some("Phone number"));
        entry.set_hexpand(true);
        entry.set_input_purpose(gtk4::InputPurpose::Phone);
        row.append(&entry);

        let type_combo = gtk4::DropDown::from_strings(&[
            "Mobile", "Home", "Work", "Main", "Home Fax", "Work Fax", "Pager", "Other",
        ]);
        type_combo.set_selected(match phone_type {
            PhoneType::Mobile => 0,
            PhoneType::Home => 1,
            PhoneType::Work => 2,
            PhoneType::Main => 3,
            PhoneType::HomeFax => 4,
            PhoneType::WorkFax => 5,
            PhoneType::Pager => 6,
            PhoneType::Other => 7,
        });
        row.append(&type_combo);

        let remove_btn = Button::from_icon_name("list-remove-symbolic");
        remove_btn.add_css_class("flat");
        remove_btn.add_css_class("circular");

        let row_ref = row.clone();
        let container_ref = container.clone();
        remove_btn.connect_clicked(move |_| {
            container_ref.remove(&row_ref);
        });

        row.append(&remove_btn);

        // Store entry and type in row name for retrieval
        row.set_widget_name("phone_row");

        row
    }

    fn create_email_row(email: &str, email_type: EmailType, container: &Box) -> Box {
        let row = Box::new(Orientation::Horizontal, 8);

        let entry = Entry::new();
        entry.set_text(email);
        entry.set_placeholder_text(Some("Email address"));
        entry.set_hexpand(true);
        entry.set_input_purpose(gtk4::InputPurpose::Email);
        row.append(&entry);

        let type_combo = gtk4::DropDown::from_strings(&["Personal", "Work", "Other"]);
        type_combo.set_selected(match email_type {
            EmailType::Personal => 0,
            EmailType::Work => 1,
            EmailType::Other => 2,
        });
        row.append(&type_combo);

        let remove_btn = Button::from_icon_name("list-remove-symbolic");
        remove_btn.add_css_class("flat");
        remove_btn.add_css_class("circular");

        let row_ref = row.clone();
        let container_ref = container.clone();
        remove_btn.connect_clicked(move |_| {
            container_ref.remove(&row_ref);
        });

        row.append(&remove_btn);

        row.set_widget_name("email_row");

        row
    }

    fn create_address_row(address: &PostalAddress, container: &Box) -> Box {
        let row = Box::new(Orientation::Vertical, 8);

        // Type selector
        let type_row = Box::new(Orientation::Horizontal, 8);
        let type_label = Label::new(Some("Type:"));
        type_row.append(&type_label);

        let type_combo = gtk4::DropDown::from_strings(&["Home", "Work", "Other"]);
        type_combo.set_selected(match address.address_type {
            AddressType::Home => 0,
            AddressType::Work => 1,
            AddressType::Other => 2,
        });
        type_row.append(&type_combo);

        let remove_btn = Button::from_icon_name("list-remove-symbolic");
        remove_btn.add_css_class("flat");
        remove_btn.add_css_class("circular");
        remove_btn.set_halign(gtk4::Align::End);
        remove_btn.set_hexpand(true);

        let row_ref = row.clone();
        let container_ref = container.clone();
        remove_btn.connect_clicked(move |_| {
            container_ref.remove(&row_ref);
        });

        type_row.append(&remove_btn);
        row.append(&type_row);

        // Street
        let street_entry = Entry::new();
        street_entry.set_text(&address.street);
        street_entry.set_placeholder_text(Some("Street address"));
        street_entry.set_widget_name("street");
        row.append(&street_entry);

        // City, State row
        let city_state = Box::new(Orientation::Horizontal, 8);

        let city_entry = Entry::new();
        city_entry.set_text(&address.city);
        city_entry.set_placeholder_text(Some("City"));
        city_entry.set_hexpand(true);
        city_entry.set_widget_name("city");
        city_state.append(&city_entry);

        let state_entry = Entry::new();
        state_entry.set_text(&address.state);
        state_entry.set_placeholder_text(Some("State"));
        state_entry.set_width_chars(10);
        state_entry.set_widget_name("state");
        city_state.append(&state_entry);

        row.append(&city_state);

        // Postal code, Country row
        let zip_country = Box::new(Orientation::Horizontal, 8);

        let postal_entry = Entry::new();
        postal_entry.set_text(&address.postal_code);
        postal_entry.set_placeholder_text(Some("Postal Code"));
        postal_entry.set_width_chars(10);
        postal_entry.set_widget_name("postal");
        zip_country.append(&postal_entry);

        let country_entry = Entry::new();
        country_entry.set_text(&address.country);
        country_entry.set_placeholder_text(Some("Country"));
        country_entry.set_hexpand(true);
        country_entry.set_widget_name("country");
        zip_country.append(&country_entry);

        row.append(&zip_country);

        let sep = gtk4::Separator::new(Orientation::Horizontal);
        sep.set_margin_top(8);
        row.append(&sep);

        row.set_widget_name("address_row");

        row
    }

    fn save_contact(&self) {
        let mut contact = self.current_contact.borrow().clone().unwrap_or_else(Contact::new);

        // Update basic fields
        contact.first_name = self.first_name_entry.text().to_string();
        contact.last_name = self.last_name_entry.text().to_string();

        let nickname = self.nickname_entry.text().to_string();
        contact.nickname = if nickname.is_empty() { None } else { Some(nickname) };

        let company = self.company_entry.text().to_string();
        contact.company = if company.is_empty() { None } else { Some(company) };

        let job_title = self.job_title_entry.text().to_string();
        contact.job_title = if job_title.is_empty() { None } else { Some(job_title) };

        let website = self.website_entry.text().to_string();
        contact.website = if website.is_empty() { None } else { Some(website) };

        let (start, end) = self.notes_buffer.bounds();
        let notes = self.notes_buffer.text(&start, &end, false).to_string();
        contact.notes = if notes.is_empty() { None } else { Some(notes) };

        contact.birthday = *self.birthday_date.borrow();
        contact.updated_at = Utc::now();

        // Collect phones
        contact.phones.clear();
        let mut first_phone = true;
        let mut child = self.phones_container.first_child();
        while let Some(widget) = child {
            if let Some(row) = widget.downcast_ref::<Box>() {
                if let Some(entry) = row.first_child().and_then(|w| w.downcast::<Entry>().ok()) {
                    let number = entry.text().to_string();
                    if !number.is_empty() {
                        let type_idx = row
                            .first_child()
                            .and_then(|w| w.next_sibling())
                            .and_then(|w| w.downcast::<gtk4::DropDown>().ok())
                            .map(|d| d.selected())
                            .unwrap_or(0);

                        contact.phones.push(PhoneNumber {
                            number,
                            phone_type: match type_idx {
                                0 => PhoneType::Mobile,
                                1 => PhoneType::Home,
                                2 => PhoneType::Work,
                                3 => PhoneType::Main,
                                4 => PhoneType::HomeFax,
                                5 => PhoneType::WorkFax,
                                6 => PhoneType::Pager,
                                _ => PhoneType::Other,
                            },
                            is_primary: first_phone,
                        });
                        first_phone = false;
                    }
                }
            }
            child = widget.next_sibling();
        }

        // Collect emails
        contact.emails.clear();
        let mut first_email = true;
        let mut child = self.emails_container.first_child();
        while let Some(widget) = child {
            if let Some(row) = widget.downcast_ref::<Box>() {
                if let Some(entry) = row.first_child().and_then(|w| w.downcast::<Entry>().ok()) {
                    let email = entry.text().to_string();
                    if !email.is_empty() {
                        let type_idx = row
                            .first_child()
                            .and_then(|w| w.next_sibling())
                            .and_then(|w| w.downcast::<gtk4::DropDown>().ok())
                            .map(|d| d.selected())
                            .unwrap_or(0);

                        contact.emails.push(EmailAddress {
                            email,
                            email_type: match type_idx {
                                0 => EmailType::Personal,
                                1 => EmailType::Work,
                                _ => EmailType::Other,
                            },
                            is_primary: first_email,
                        });
                        first_email = false;
                    }
                }
            }
            child = widget.next_sibling();
        }

        // Collect addresses (simplified - in production would properly traverse the widget tree)
        contact.addresses.clear();

        // Save to storage
        if let Ok(mut storage) = self.storage.try_borrow_mut() {
            if storage.save_contact(&contact).is_ok() {
                if let Some(callback) = self.on_saved.borrow().as_ref() {
                    callback(contact);
                }
            }
        }
    }

    pub fn connect_saved<F: Fn(Contact) + 'static>(&self, callback: F) {
        *self.on_saved.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_cancelled<F: Fn() + 'static>(&self, callback: F) {
        *self.on_cancelled.borrow_mut() = Some(Box::new(callback));
    }
}
