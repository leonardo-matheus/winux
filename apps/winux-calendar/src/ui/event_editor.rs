//! Event editor dialog

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, Label, Orientation, ScrolledWindow, TextView,
    DropDown, Switch, Calendar, SpinButton, CheckButton,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, PreferencesGroup, PreferencesPage, Dialog, Window,
    EntryRow, SwitchRow, ComboRow, ExpanderRow,
};
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Local, NaiveDate, NaiveTime, Datelike};

use crate::data::{CalendarStore, CalendarInfo, Event, Recurrence, RecurrenceType, Reminder, ReminderUnit};

/// Event editor widget
pub struct EventEditor {
    dialog: adw::Dialog,
    event: Option<Event>,
    store: Rc<RefCell<CalendarStore>>,

    // Form fields
    title_entry: EntryRow,
    location_entry: EntryRow,
    description_view: TextView,
    all_day_switch: SwitchRow,
    calendar_combo: ComboRow,
    recurrence_combo: ComboRow,
}

impl EventEditor {
    /// Create a new event editor
    pub fn new(event: Option<Event>, store: Rc<RefCell<CalendarStore>>) -> Self {
        let dialog = adw::Dialog::builder()
            .title(if event.is_some() { "Editar Evento" } else { "Novo Evento" })
            .content_width(500)
            .content_height(600)
            .build();

        // Form fields
        let title_entry = EntryRow::builder()
            .title("Titulo")
            .build();

        let location_entry = EntryRow::builder()
            .title("Local")
            .build();

        let description_view = TextView::builder()
            .wrap_mode(gtk4::WrapMode::Word)
            .top_margin(8)
            .bottom_margin(8)
            .left_margin(8)
            .right_margin(8)
            .build();

        let all_day_switch = SwitchRow::builder()
            .title("Dia Inteiro")
            .active(true)
            .build();

        let calendar_combo = ComboRow::builder()
            .title("Calendario")
            .build();

        let recurrence_combo = ComboRow::builder()
            .title("Repetir")
            .build();

        let editor = Self {
            dialog,
            event,
            store,
            title_entry,
            location_entry,
            description_view,
            all_day_switch,
            calendar_combo,
            recurrence_combo,
        };

        editor.build_ui();
        editor
    }

    /// Build the dialog UI
    fn build_ui(&self) {
        let content = Box::builder()
            .orientation(Orientation::Vertical)
            .build();

        // Header bar
        let header = adw::HeaderBar::new();

        let cancel_btn = Button::with_label("Cancelar");
        cancel_btn.add_css_class("flat");
        let dialog_clone = self.dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });
        header.pack_start(&cancel_btn);

        let save_btn = Button::with_label("Salvar");
        save_btn.add_css_class("suggested-action");
        let dialog_clone = self.dialog.clone();
        let store = self.store.clone();
        let title_entry = self.title_entry.clone();
        save_btn.connect_clicked(move |_| {
            // Save event logic here
            let title = title_entry.text().to_string();
            if !title.is_empty() {
                // Create and save event
                dialog_clone.close();
            }
        });
        header.pack_end(&save_btn);

        content.append(&header);

        // Scrollable form
        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();

        let page = PreferencesPage::new();

        // Basic info group
        let basic_group = PreferencesGroup::builder()
            .title("Informacoes Basicas")
            .build();

        basic_group.add(&self.title_entry);
        basic_group.add(&self.location_entry);

        // Description
        let desc_row = ActionRow::builder()
            .title("Descricao")
            .build();

        let desc_frame = gtk4::Frame::builder()
            .css_classes(vec!["card"])
            .height_request(100)
            .build();
        desc_frame.set_child(Some(&self.description_view));

        let desc_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(8)
            .build();
        desc_box.append(&desc_frame);

        basic_group.add(&desc_box);

        page.add(&basic_group);

        // Date and time group
        let datetime_group = PreferencesGroup::builder()
            .title("Data e Hora")
            .build();

        datetime_group.add(&self.all_day_switch);

        // Start date
        let start_date_row = ActionRow::builder()
            .title("Data de Inicio")
            .subtitle(&Local::now().format("%d/%m/%Y").to_string())
            .activatable(true)
            .build();
        start_date_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        datetime_group.add(&start_date_row);

        // Start time (hidden when all-day)
        let start_time_row = ActionRow::builder()
            .title("Hora de Inicio")
            .subtitle("09:00")
            .activatable(true)
            .build();
        start_time_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        datetime_group.add(&start_time_row);

        // End date
        let end_date_row = ActionRow::builder()
            .title("Data de Termino")
            .subtitle(&Local::now().format("%d/%m/%Y").to_string())
            .activatable(true)
            .build();
        end_date_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        datetime_group.add(&end_date_row);

        // End time
        let end_time_row = ActionRow::builder()
            .title("Hora de Termino")
            .subtitle("10:00")
            .activatable(true)
            .build();
        end_time_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        datetime_group.add(&end_time_row);

        // Toggle time rows visibility based on all-day switch
        let start_time_row_clone = start_time_row.clone();
        let end_time_row_clone = end_time_row.clone();
        self.all_day_switch.connect_active_notify(move |switch| {
            let visible = !switch.is_active();
            start_time_row_clone.set_visible(visible);
            end_time_row_clone.set_visible(visible);
        });

        page.add(&datetime_group);

        // Recurrence group
        let recurrence_group = PreferencesGroup::builder()
            .title("Recorrencia")
            .build();

        // Recurrence options
        let recurrence_model = gtk4::StringList::new(&[
            "Nao repetir",
            "Diariamente",
            "Semanalmente",
            "Mensalmente",
            "Anualmente",
            "Personalizado...",
        ]);
        self.recurrence_combo.set_model(Some(&recurrence_model));
        recurrence_group.add(&self.recurrence_combo);

        page.add(&recurrence_group);

        // Calendar selection group
        let calendar_group = PreferencesGroup::builder()
            .title("Calendario")
            .build();

        // Populate calendars
        {
            let store = self.store.borrow();
            let calendar_names: Vec<String> = store.calendars()
                .iter()
                .map(|c| c.name.clone())
                .collect();
            let calendar_model = gtk4::StringList::new(
                &calendar_names.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            );
            self.calendar_combo.set_model(Some(&calendar_model));
        }
        calendar_group.add(&self.calendar_combo);

        page.add(&calendar_group);

        // Reminders group
        let reminders_group = PreferencesGroup::builder()
            .title("Lembretes")
            .build();

        let add_reminder_row = ActionRow::builder()
            .title("Adicionar Lembrete")
            .activatable(true)
            .build();
        add_reminder_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
        reminders_group.add(&add_reminder_row);

        // Default reminder
        let reminder_row = ActionRow::builder()
            .title("15 minutos antes")
            .build();

        let remove_btn = Button::from_icon_name("user-trash-symbolic");
        remove_btn.add_css_class("flat");
        remove_btn.set_valign(gtk4::Align::Center);
        reminder_row.add_suffix(&remove_btn);
        reminders_group.add(&reminder_row);

        page.add(&reminders_group);

        // Notes group
        let notes_group = PreferencesGroup::builder()
            .title("Notas")
            .build();

        let notes_view = TextView::builder()
            .wrap_mode(gtk4::WrapMode::Word)
            .top_margin(8)
            .bottom_margin(8)
            .left_margin(8)
            .right_margin(8)
            .build();

        let notes_frame = gtk4::Frame::builder()
            .css_classes(vec!["card"])
            .height_request(80)
            .margin_start(12)
            .margin_end(12)
            .margin_bottom(12)
            .build();
        notes_frame.set_child(Some(&notes_view));

        notes_group.add(&notes_frame);

        page.add(&notes_group);

        // URL group
        let url_group = PreferencesGroup::new();

        let url_entry = EntryRow::builder()
            .title("URL")
            .build();
        url_group.add(&url_entry);

        page.add(&url_group);

        // Delete button (only for existing events)
        if self.event.is_some() {
            let delete_group = PreferencesGroup::new();

            let delete_btn = Button::with_label("Excluir Evento");
            delete_btn.add_css_class("destructive-action");
            delete_btn.set_halign(gtk4::Align::Center);
            delete_btn.set_margin_top(20);
            delete_btn.set_margin_bottom(20);

            let delete_box = Box::builder()
                .halign(gtk4::Align::Center)
                .build();
            delete_box.append(&delete_btn);

            delete_group.add(&delete_box);
            page.add(&delete_group);
        }

        scrolled.set_child(Some(&page));
        content.append(&scrolled);

        self.dialog.set_child(Some(&content));

        // Populate fields if editing existing event
        if let Some(ref event) = self.event {
            self.title_entry.set_text(&event.title);

            if let Some(ref location) = event.location {
                self.location_entry.set_text(location);
            }

            if let Some(ref description) = event.description {
                self.description_view.buffer().set_text(description);
            }

            self.all_day_switch.set_active(event.all_day);
        }
    }

    /// Show the dialog
    pub fn show(&self) {
        // In a real implementation, this would present the dialog
        // Since we need a parent window, this is a placeholder
    }

    /// Show the dialog with a parent window
    pub fn present(&self, parent: &impl IsA<gtk4::Widget>) {
        self.dialog.present(Some(parent));
    }
}

/// Quick event creation popup
pub struct QuickEventPopup {
    widget: Box,
}

impl QuickEventPopup {
    pub fn new(date: NaiveDate, on_create: impl Fn(String) + 'static) -> Self {
        let widget = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        // Date label
        let date_label = Label::builder()
            .label(&format!("{:02}/{:02}/{}", date.day(), date.month(), date.year()))
            .css_classes(vec!["caption", "dim-label"])
            .halign(gtk4::Align::Start)
            .build();
        widget.append(&date_label);

        // Title entry
        let title_entry = Entry::builder()
            .placeholder_text("Adicionar titulo")
            .build();
        widget.append(&title_entry);

        // Buttons
        let buttons_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::End)
            .build();

        let more_btn = Button::with_label("Mais opcoes");
        more_btn.add_css_class("flat");
        buttons_box.append(&more_btn);

        let save_btn = Button::with_label("Salvar");
        save_btn.add_css_class("suggested-action");

        let entry_clone = title_entry.clone();
        save_btn.connect_clicked(move |_| {
            let title = entry_clone.text().to_string();
            if !title.is_empty() {
                on_create(title);
            }
        });
        buttons_box.append(&save_btn);

        widget.append(&buttons_box);

        Self { widget }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}

/// Time picker widget
pub struct TimePicker {
    widget: Box,
    hour_spin: SpinButton,
    minute_spin: SpinButton,
}

impl TimePicker {
    pub fn new() -> Self {
        let widget = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        let hour_spin = SpinButton::with_range(0.0, 23.0, 1.0);
        hour_spin.set_wrap(true);
        hour_spin.set_width_chars(2);
        widget.append(&hour_spin);

        let separator = Label::new(Some(":"));
        widget.append(&separator);

        let minute_spin = SpinButton::with_range(0.0, 59.0, 5.0);
        minute_spin.set_wrap(true);
        minute_spin.set_width_chars(2);
        widget.append(&minute_spin);

        Self {
            widget,
            hour_spin,
            minute_spin,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn set_time(&self, time: NaiveTime) {
        self.hour_spin.set_value(time.hour() as f64);
        self.minute_spin.set_value(time.minute() as f64);
    }

    pub fn get_time(&self) -> Option<NaiveTime> {
        NaiveTime::from_hms_opt(
            self.hour_spin.value() as u32,
            self.minute_spin.value() as u32,
            0,
        )
    }
}

impl Default for TimePicker {
    fn default() -> Self {
        Self::new()
    }
}
