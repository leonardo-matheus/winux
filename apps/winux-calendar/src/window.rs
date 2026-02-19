// Main window for Winux Calendar

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Calendar, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, Separator, ToggleButton,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ApplicationWindow, HeaderBar, ViewStack, ActionRow, PreferencesGroup,
    SplitButton, Clamp,
};
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Local, NaiveDate, Datelike, Weekday, Duration};

use crate::views::{MonthView, WeekView, DayView, AgendaView};
use crate::ui::{MiniCalendar, EventEditor};
use crate::data::{CalendarStore, CalendarInfo, Event};

pub fn build_ui(app: &Application) {
    // Force system theme
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::PreferDark);

    let store = Rc::new(RefCell::new(CalendarStore::new()));

    // Create default calendars
    {
        let mut store_mut = store.borrow_mut();
        store_mut.add_calendar(CalendarInfo::new("Pessoal", "#3584e4"));
        store_mut.add_calendar(CalendarInfo::new("Trabalho", "#e66100"));
        store_mut.add_calendar(CalendarInfo::new("Familia", "#26a269"));
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Calendario")
        .default_width(1200)
        .default_height(800)
        .build();

    // Main layout
    let main_box = Box::new(Orientation::Horizontal, 0);

    // Sidebar
    let sidebar = create_sidebar(store.clone());
    main_box.append(&sidebar);

    let separator = Separator::new(Orientation::Vertical);
    main_box.append(&separator);

    // Content area with header
    let content_box = Box::new(Orientation::Vertical, 0);

    // Header bar
    let header = create_header_bar(store.clone());
    content_box.append(&header);

    // View stack for different calendar views
    let stack = ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    // Month view
    let month_view = MonthView::new(store.clone());
    stack.add_titled(&month_view.widget(), Some("month"), "Mes");

    // Week view
    let week_view = WeekView::new(store.clone());
    stack.add_titled(&week_view.widget(), Some("week"), "Semana");

    // Day view
    let day_view = DayView::new(store.clone());
    stack.add_titled(&day_view.widget(), Some("day"), "Dia");

    // Agenda view
    let agenda_view = AgendaView::new(store.clone());
    stack.add_titled(&agenda_view.widget(), Some("agenda"), "Agenda");

    content_box.append(&stack);
    main_box.append(&content_box);

    window.set_content(Some(&main_box));
    window.present();
}

fn create_header_bar(store: Rc<RefCell<CalendarStore>>) -> Box {
    let header_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let header = HeaderBar::new();

    // Navigation buttons
    let nav_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .css_classes(vec!["linked"])
        .build();

    let prev_btn = Button::from_icon_name("go-previous-symbolic");
    prev_btn.set_tooltip_text(Some("Periodo anterior"));

    let today_btn = Button::with_label("Hoje");
    today_btn.set_tooltip_text(Some("Ir para hoje"));

    let next_btn = Button::from_icon_name("go-next-symbolic");
    next_btn.set_tooltip_text(Some("Proximo periodo"));

    nav_box.append(&prev_btn);
    nav_box.append(&today_btn);
    nav_box.append(&next_btn);

    header.pack_start(&nav_box);

    // Current date label
    let today = Local::now();
    let date_label = Label::builder()
        .label(&format!("{} {}",
            get_month_name(today.month()),
            today.year()))
        .css_classes(vec!["title-3"])
        .build();
    header.set_title_widget(Some(&date_label));

    // View switcher
    let view_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .css_classes(vec!["linked"])
        .build();

    let month_btn = ToggleButton::with_label("Mes");
    month_btn.set_active(true);
    let week_btn = ToggleButton::with_label("Semana");
    week_btn.set_group(Some(&month_btn));
    let day_btn = ToggleButton::with_label("Dia");
    day_btn.set_group(Some(&month_btn));
    let agenda_btn = ToggleButton::with_label("Agenda");
    agenda_btn.set_group(Some(&month_btn));

    view_box.append(&month_btn);
    view_box.append(&week_btn);
    view_box.append(&day_btn);
    view_box.append(&agenda_btn);

    header.pack_end(&view_box);

    // New event button
    let new_event_btn = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Novo evento")
        .css_classes(vec!["suggested-action"])
        .build();

    let store_clone = store.clone();
    new_event_btn.connect_clicked(move |_| {
        show_event_editor(None, store_clone.clone());
    });

    header.pack_end(&new_event_btn);

    header_box.append(&header);
    header_box
}

fn create_sidebar(store: Rc<RefCell<CalendarStore>>) -> Box {
    let sidebar = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(280)
        .css_classes(vec!["sidebar"])
        .build();

    // Mini calendar
    let mini_calendar = MiniCalendar::new();
    sidebar.append(&mini_calendar.widget());

    let separator = Separator::new(Orientation::Horizontal);
    separator.set_margin_top(12);
    separator.set_margin_bottom(12);
    sidebar.append(&separator);

    // Calendars section
    let calendars_label = Label::builder()
        .label("Calendarios")
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .margin_start(16)
        .margin_bottom(8)
        .build();
    sidebar.append(&calendars_label);

    let calendars_list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(vec!["navigation-sidebar"])
        .margin_start(8)
        .margin_end(8)
        .build();

    // Add calendars from store
    {
        let store_ref = store.borrow();
        for calendar in store_ref.calendars() {
            let row = create_calendar_row(calendar);
            calendars_list.append(&row);
        }
    }

    sidebar.append(&calendars_list);

    // Add calendar button
    let add_calendar_btn = Button::builder()
        .label("Adicionar Calendario")
        .css_classes(vec!["flat"])
        .margin_start(16)
        .margin_end(16)
        .margin_top(8)
        .build();
    sidebar.append(&add_calendar_btn);

    // Spacer
    let spacer = Box::builder()
        .vexpand(true)
        .build();
    sidebar.append(&spacer);

    // Tasks section
    let tasks_separator = Separator::new(Orientation::Horizontal);
    tasks_separator.set_margin_bottom(12);
    sidebar.append(&tasks_separator);

    let tasks_label = Label::builder()
        .label("Tarefas")
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .margin_start(16)
        .margin_bottom(8)
        .build();
    sidebar.append(&tasks_label);

    let tasks_list = create_tasks_list(store.clone());
    sidebar.append(&tasks_list);

    // Add task button
    let add_task_btn = Button::builder()
        .label("Nova Tarefa")
        .css_classes(vec!["flat"])
        .margin_start(16)
        .margin_end(16)
        .margin_top(8)
        .margin_bottom(16)
        .build();
    sidebar.append(&add_task_btn);

    sidebar
}

fn create_calendar_row(calendar: &CalendarInfo) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(8)
        .margin_end(8)
        .build();

    // Color indicator
    let color_box = Box::builder()
        .width_request(16)
        .height_request(16)
        .valign(gtk4::Align::Center)
        .build();

    // Apply color via CSS
    let css_provider = gtk4::CssProvider::new();
    let css = format!(
        "box {{ background-color: {}; border-radius: 4px; }}",
        calendar.color
    );
    css_provider.load_from_string(&css);
    color_box.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    row_box.append(&color_box);

    // Calendar name
    let name_label = Label::builder()
        .label(&calendar.name)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    row_box.append(&name_label);

    // Visibility checkbox
    let check = gtk4::CheckButton::builder()
        .active(calendar.visible)
        .build();
    row_box.append(&check);

    ListBoxRow::builder()
        .child(&row_box)
        .build()
}

fn create_tasks_list(store: Rc<RefCell<CalendarStore>>) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .vexpand(false)
        .height_request(150)
        .margin_start(8)
        .margin_end(8)
        .build();

    let list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(vec!["boxed-list"])
        .build();

    // Sample tasks
    let tasks = [
        ("Revisar relatorio", "Hoje", false),
        ("Reuniao com equipe", "Amanha", false),
        ("Enviar proposta", "Sexta", true),
    ];

    for (title, due, completed) in tasks {
        let row = create_task_row(title, due, completed);
        list.append(&row);
    }

    scrolled.set_child(Some(&list));
    scrolled
}

fn create_task_row(title: &str, due: &str, completed: bool) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(8)
        .margin_end(8)
        .build();

    let check = gtk4::CheckButton::builder()
        .active(completed)
        .build();
    row_box.append(&check);

    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let title_label = Label::builder()
        .label(title)
        .halign(gtk4::Align::Start)
        .css_classes(if completed { vec!["dim-label"] } else { vec![] })
        .build();
    info_box.append(&title_label);

    let due_label = Label::builder()
        .label(due)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption", "dim-label"])
        .build();
    info_box.append(&due_label);

    row_box.append(&info_box);

    ListBoxRow::builder()
        .child(&row_box)
        .build()
}

fn show_event_editor(event: Option<Event>, store: Rc<RefCell<CalendarStore>>) {
    // Event editor dialog will be shown here
    let editor = EventEditor::new(event, store);
    editor.show();
}

fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "Janeiro",
        2 => "Fevereiro",
        3 => "Marco",
        4 => "Abril",
        5 => "Maio",
        6 => "Junho",
        7 => "Julho",
        8 => "Agosto",
        9 => "Setembro",
        10 => "Outubro",
        11 => "Novembro",
        12 => "Dezembro",
        _ => "Desconhecido",
    }
}
