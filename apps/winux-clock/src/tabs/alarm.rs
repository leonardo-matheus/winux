// Alarm Tab - Create and manage alarms

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, ScrolledWindow, Switch};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, ExpanderRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::data::alarm::{Alarm, RepeatDays, AlarmStore};
use crate::ui::time_picker::TimePicker;

pub fn create_alarm_tab() -> Box {
    let main_box = Box::new(Orientation::Vertical, 0);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let page = PreferencesPage::new();

    // Alarms group
    let alarms_group = PreferencesGroup::builder()
        .title("Alarmes")
        .description("Seus alarmes configurados")
        .build();

    // Add alarm button
    let add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Novo alarme")
        .build();
    add_button.add_css_class("flat");
    alarms_group.set_header_suffix(Some(&add_button));

    // Alarm store
    let alarm_store: Rc<RefCell<AlarmStore>> = Rc::new(RefCell::new(AlarmStore::load()));

    // Create alarm rows
    let alarm_rows: Rc<RefCell<Vec<AlarmRow>>> = Rc::new(RefCell::new(Vec::new()));

    // Add some sample alarms if store is empty
    if alarm_store.borrow().alarms.is_empty() {
        let mut store = alarm_store.borrow_mut();
        store.alarms.push(Alarm {
            id: 1,
            hour: 7,
            minute: 0,
            label: "Acordar".to_string(),
            enabled: true,
            repeat: RepeatDays::weekdays(),
            snooze_minutes: 10,
            sound: "default".to_string(),
        });
        store.alarms.push(Alarm {
            id: 2,
            hour: 8,
            minute: 30,
            label: "Reuniao".to_string(),
            enabled: true,
            repeat: RepeatDays::none(),
            snooze_minutes: 5,
            sound: "default".to_string(),
        });
        store.alarms.push(Alarm {
            id: 3,
            hour: 22,
            minute: 0,
            label: "Dormir".to_string(),
            enabled: false,
            repeat: RepeatDays::all(),
            snooze_minutes: 10,
            sound: "gentle".to_string(),
        });
    }

    // Display alarms
    for alarm in alarm_store.borrow().alarms.iter() {
        let row = create_alarm_row(alarm);
        alarms_group.add(&row.widget);
        alarm_rows.borrow_mut().push(row);
    }

    page.add(&alarms_group);
    scrolled.set_child(Some(&page));
    main_box.append(&scrolled);

    // Add alarm button handler
    let main_box_clone = main_box.clone();
    let alarm_store_clone = alarm_store.clone();
    add_button.connect_clicked(move |_| {
        show_add_alarm_dialog(&main_box_clone, alarm_store_clone.clone());
    });

    // Check alarms every minute
    let alarm_store_check = alarm_store.clone();
    glib::timeout_add_seconds_local(60, move || {
        check_alarms(&alarm_store_check.borrow());
        glib::ControlFlow::Continue
    });

    main_box
}

struct AlarmRow {
    widget: ExpanderRow,
}

fn create_alarm_row(alarm: &Alarm) -> AlarmRow {
    let row = ExpanderRow::builder()
        .title(&format!("{:02}:{:02}", alarm.hour, alarm.minute))
        .subtitle(&alarm.label)
        .show_enable_switch(true)
        .enable_expansion(false)
        .build();

    row.set_expanded(false);
    row.add_css_class("alarm-row");

    if !alarm.enabled {
        row.add_css_class("alarm-disabled");
    }

    // Enable switch
    let switch = Switch::new();
    switch.set_active(alarm.enabled);
    switch.set_valign(gtk4::Align::Center);

    let row_clone = row.clone();
    switch.connect_state_set(move |_, state| {
        if state {
            row_clone.remove_css_class("alarm-disabled");
        } else {
            row_clone.add_css_class("alarm-disabled");
        }
        glib::Propagation::Proceed
    });

    row.add_suffix(&switch);

    // Repeat info
    let repeat_label = Label::new(Some(&alarm.repeat.to_string()));
    repeat_label.add_css_class("alarm-repeat");
    repeat_label.set_margin_end(12);
    row.add_suffix(&repeat_label);

    // Expanded content - Edit options
    let edit_row = ActionRow::builder()
        .title("Editar")
        .activatable(true)
        .build();
    edit_row.add_suffix(&gtk4::Image::from_icon_name("document-edit-symbolic"));
    row.add_row(&edit_row);

    // Label edit
    let label_row = ActionRow::builder()
        .title("Rotulo")
        .subtitle(&alarm.label)
        .build();
    let label_entry = Entry::new();
    label_entry.set_text(&alarm.label);
    label_entry.set_valign(gtk4::Align::Center);
    label_entry.set_width_chars(15);
    label_row.add_suffix(&label_entry);
    row.add_row(&label_row);

    // Repeat edit
    let repeat_row = ActionRow::builder()
        .title("Repetir")
        .subtitle(&alarm.repeat.to_string())
        .activatable(true)
        .build();
    repeat_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    row.add_row(&repeat_row);

    // Snooze
    let snooze_row = ActionRow::builder()
        .title("Soneca")
        .subtitle(&format!("{} minutos", alarm.snooze_minutes))
        .build();
    row.add_row(&snooze_row);

    // Sound
    let sound_row = ActionRow::builder()
        .title("Som")
        .subtitle(&alarm.sound)
        .activatable(true)
        .build();
    sound_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    row.add_row(&sound_row);

    // Delete
    let delete_row = ActionRow::builder()
        .title("Excluir Alarme")
        .activatable(true)
        .build();
    delete_row.add_css_class("error");
    delete_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
    row.add_row(&delete_row);

    AlarmRow { widget: row }
}

fn show_add_alarm_dialog(parent: &Box, _store: Rc<RefCell<AlarmStore>>) {
    let dialog = adw::Dialog::new();
    dialog.set_title("Novo Alarme");
    dialog.set_content_width(400);
    dialog.set_content_height(600);

    let toolbar_view = adw::ToolbarView::new();

    let header = adw::HeaderBar::new();

    // Cancel button
    let cancel_btn = Button::with_label("Cancelar");
    cancel_btn.add_css_class("flat");
    header.pack_start(&cancel_btn);

    // Save button
    let save_btn = Button::with_label("Salvar");
    save_btn.add_css_class("suggested-action");
    header.pack_end(&save_btn);

    toolbar_view.add_top_bar(&header);

    let content = Box::new(Orientation::Vertical, 0);

    // Time picker
    let time_picker = TimePicker::new();
    time_picker.widget().set_margin_top(20);
    time_picker.widget().set_margin_bottom(20);
    content.append(time_picker.widget());

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let page = PreferencesPage::new();

    // Options group
    let options_group = PreferencesGroup::new();

    // Label
    let label_row = ActionRow::builder()
        .title("Rotulo")
        .build();
    let label_entry = Entry::new();
    label_entry.set_placeholder_text(Some("Alarme"));
    label_entry.set_valign(gtk4::Align::Center);
    label_entry.set_width_chars(20);
    label_row.add_suffix(&label_entry);
    options_group.add(&label_row);

    // Repeat
    let repeat_row = ExpanderRow::builder()
        .title("Repetir")
        .subtitle("Nunca")
        .build();

    let days = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sab"];
    let days_box = Box::new(Orientation::Horizontal, 8);
    days_box.set_margin_top(12);
    days_box.set_margin_bottom(12);
    days_box.set_margin_start(12);
    days_box.set_margin_end(12);
    days_box.set_halign(gtk4::Align::Center);

    for day in days {
        let btn = Button::with_label(day);
        btn.add_css_class("day-button");
        btn.add_css_class("circular");

        let btn_clone = btn.clone();
        btn.connect_clicked(move |_| {
            if btn_clone.has_css_class("selected") {
                btn_clone.remove_css_class("selected");
            } else {
                btn_clone.add_css_class("selected");
            }
        });

        days_box.append(&btn);
    }

    let days_action_row = ActionRow::new();
    days_action_row.set_child(Some(&days_box));
    repeat_row.add_row(&days_action_row);

    // Quick select buttons
    let quick_box = Box::new(Orientation::Horizontal, 8);
    quick_box.set_margin_top(8);
    quick_box.set_margin_bottom(12);
    quick_box.set_margin_start(12);
    quick_box.set_margin_end(12);
    quick_box.set_halign(gtk4::Align::Center);

    let weekdays_btn = Button::with_label("Dias Uteis");
    weekdays_btn.add_css_class("pill");
    quick_box.append(&weekdays_btn);

    let weekend_btn = Button::with_label("Fim de Semana");
    weekend_btn.add_css_class("pill");
    quick_box.append(&weekend_btn);

    let everyday_btn = Button::with_label("Todos os Dias");
    everyday_btn.add_css_class("pill");
    quick_box.append(&everyday_btn);

    let quick_action_row = ActionRow::new();
    quick_action_row.set_child(Some(&quick_box));
    repeat_row.add_row(&quick_action_row);

    options_group.add(&repeat_row);

    // Snooze
    let snooze_row = adw::ComboRow::builder()
        .title("Soneca")
        .build();
    let snooze_options = gtk4::StringList::new(&[
        "5 minutos",
        "10 minutos",
        "15 minutos",
        "20 minutos",
        "30 minutos",
    ]);
    snooze_row.set_model(Some(&snooze_options));
    snooze_row.set_selected(1);
    options_group.add(&snooze_row);

    // Sound
    let sound_row = adw::ComboRow::builder()
        .title("Som")
        .build();
    let sound_options = gtk4::StringList::new(&[
        "Padrao",
        "Suave",
        "Digital",
        "Classico",
        "Natureza",
    ]);
    sound_row.set_model(Some(&sound_options));
    options_group.add(&sound_row);

    page.add(&options_group);
    scrolled.set_child(Some(&page));
    content.append(&scrolled);

    toolbar_view.set_content(Some(&content));
    dialog.set_child(Some(&toolbar_view));

    // Button handlers
    let dialog_clone = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    save_btn.connect_clicked(move |_| {
        // Save alarm logic here
        dialog_clone.close();
    });

    dialog.present(Some(parent));
}

fn check_alarms(store: &AlarmStore) {
    let now = chrono::Local::now();
    let current_hour = now.format("%H").to_string().parse::<u32>().unwrap_or(0);
    let current_minute = now.format("%M").to_string().parse::<u32>().unwrap_or(0);

    for alarm in &store.alarms {
        if alarm.enabled && alarm.hour == current_hour && alarm.minute == current_minute {
            // Trigger alarm notification
            crate::notifications::alarm_notify::trigger_alarm(alarm);
        }
    }
}
