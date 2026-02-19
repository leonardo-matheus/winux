// Winux Logs - Main Window
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{
    gio, Application, Box, Button, Label, Orientation, Paned, ScrolledWindow,
    SearchEntry, ToggleButton, PolicyType, Separator, MenuButton, PopoverMenu,
};
use gtk4::glib;
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ApplicationWindow, HeaderBar, ViewStack, ViewStackSidebar, StatusPage,
    ToastOverlay, Toast,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::sources::{LogSource, LogEntry, LogLevel};
use crate::filters::FilterState;
use crate::ui::{log_view, filters as filters_ui, detail};

pub fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    // Title
    let title = adw::WindowTitle::new("Logs do Sistema", "");
    header.set_title_widget(Some(&title));

    // Search entry
    let search_entry = SearchEntry::builder()
        .placeholder_text("Buscar nos logs...")
        .width_request(300)
        .build();
    header.pack_start(&search_entry);

    // Live tail toggle
    let live_button = ToggleButton::builder()
        .icon_name("media-playback-start-symbolic")
        .tooltip_text("Atualizar em tempo real")
        .build();
    header.pack_end(&live_button);

    // Export menu
    let export_menu = create_export_menu();
    let export_button = MenuButton::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Exportar logs")
        .popover(&export_menu)
        .build();
    header.pack_end(&export_button);

    // Refresh button
    let refresh_button = Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Atualizar")
        .build();
    header.pack_end(&refresh_button);

    // Main paned layout
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned.set_position(280);

    // Left sidebar with sources
    let sidebar_box = Box::new(Orientation::Vertical, 0);

    // Source selection stack
    let source_stack = ViewStack::new();

    // Journal page
    let journal_page = StatusPage::builder()
        .icon_name("text-x-log-symbolic")
        .title("Journal")
        .description("Logs do systemd")
        .build();
    source_stack.add_titled(&journal_page, Some("journal"), "Journal")
        .set_icon_name(Some("text-x-log-symbolic"));

    // Kernel page
    let kernel_page = StatusPage::builder()
        .icon_name("computer-symbolic")
        .title("Kernel")
        .description("Mensagens do kernel (dmesg)")
        .build();
    source_stack.add_titled(&kernel_page, Some("kernel"), "Kernel")
        .set_icon_name(Some("computer-symbolic"));

    // Syslog page
    let syslog_page = StatusPage::builder()
        .icon_name("folder-documents-symbolic")
        .title("Syslog")
        .description("Arquivos em /var/log")
        .build();
    source_stack.add_titled(&syslog_page, Some("syslog"), "Syslog")
        .set_icon_name(Some("folder-documents-symbolic"));

    // Apps page
    let apps_page = StatusPage::builder()
        .icon_name("application-x-executable-symbolic")
        .title("Aplicativos")
        .description("Logs de aplicativos")
        .build();
    source_stack.add_titled(&apps_page, Some("apps"), "Aplicativos")
        .set_icon_name(Some("application-x-executable-symbolic"));

    let sidebar = ViewStackSidebar::new();
    sidebar.set_stack(&source_stack);
    sidebar.set_vexpand(true);

    // Filters section
    let filters_label = Label::new(Some("Filtros"));
    filters_label.add_css_class("heading");
    filters_label.set_margin_start(12);
    filters_label.set_margin_top(12);
    filters_label.set_margin_bottom(6);
    filters_label.set_xalign(0.0);

    let filters_box = filters_ui::create_filters_panel();

    sidebar_box.append(&sidebar);
    sidebar_box.append(&Separator::new(Orientation::Horizontal));
    sidebar_box.append(&filters_label);
    sidebar_box.append(&filters_box);

    paned.set_start_child(Some(&sidebar_box));

    // Right content area
    let content_box = Box::new(Orientation::Vertical, 0);

    // Boot selector
    let boot_box = create_boot_selector();
    content_box.append(&boot_box);
    content_box.append(&Separator::new(Orientation::Horizontal));

    // Horizontal paned for log list and details
    let content_paned = Paned::new(Orientation::Horizontal);
    content_paned.set_shrink_start_child(false);
    content_paned.set_shrink_end_child(false);
    content_paned.set_position(600);
    content_paned.set_vexpand(true);

    // Log list
    let log_list = log_view::create_log_view();
    let log_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&log_list)
        .build();
    content_paned.set_start_child(Some(&log_scroll));

    // Detail panel
    let detail_panel = detail::create_detail_panel();
    content_paned.set_end_child(Some(&detail_panel));

    content_box.append(&content_paned);

    // Status bar
    let status_bar = create_status_bar();
    content_box.append(&Separator::new(Orientation::Horizontal));
    content_box.append(&status_bar);

    paned.set_end_child(Some(&content_box));

    // Toast overlay for notifications
    let toast_overlay = ToastOverlay::new();
    toast_overlay.set_child(Some(&paned));

    // Main box
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&toast_overlay);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Logs do Sistema")
        .default_width(1400)
        .default_height(800)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));

    // Shared state
    let filter_state = Rc::new(RefCell::new(FilterState::default()));
    let log_entries: Rc<RefCell<Vec<LogEntry>>> = Rc::new(RefCell::new(Vec::new()));
    let toast_overlay = Rc::new(toast_overlay);

    // Connect signals
    let log_list_clone = log_list.clone();
    let log_entries_clone = Rc::clone(&log_entries);
    let toast_clone = Rc::clone(&toast_overlay);

    refresh_button.connect_clicked(move |_| {
        load_logs(&log_list_clone, &log_entries_clone, LogSource::Journal);
        let toast = Toast::new("Logs atualizados");
        toast.set_timeout(2);
        toast_clone.add_toast(toast);
    });

    let log_list_clone = log_list.clone();
    let log_entries_clone = Rc::clone(&log_entries);
    let filter_state_clone = Rc::clone(&filter_state);

    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        filter_state_clone.borrow_mut().search_query = if query.is_empty() {
            None
        } else {
            Some(query)
        };
        apply_filters(&log_list_clone, &log_entries_clone.borrow(), &filter_state_clone.borrow());
    });

    // Live tail functionality
    let log_list_clone = log_list.clone();
    let log_entries_clone = Rc::clone(&log_entries);
    let live_active = Rc::new(RefCell::new(false));
    let live_active_clone = Rc::clone(&live_active);

    live_button.connect_toggled(move |button| {
        *live_active_clone.borrow_mut() = button.is_active();
        if button.is_active() {
            button.set_icon_name("media-playback-pause-symbolic");
            // Start live tail
            start_live_tail(&log_list_clone, &log_entries_clone);
        } else {
            button.set_icon_name("media-playback-start-symbolic");
        }
    });

    // Load initial logs
    load_logs(&log_list, &log_entries, LogSource::Journal);

    // Connect source selection
    let log_list_clone = log_list.clone();
    let log_entries_clone = Rc::clone(&log_entries);

    source_stack.connect_visible_child_notify(move |stack| {
        if let Some(name) = stack.visible_child_name() {
            let source = match name.as_str() {
                "journal" => LogSource::Journal,
                "kernel" => LogSource::Kernel,
                "syslog" => LogSource::Syslog,
                "apps" => LogSource::AppLogs,
                _ => LogSource::Journal,
            };
            load_logs(&log_list_clone, &log_entries_clone, source);
        }
    });

    window.present();
}

fn create_export_menu() -> PopoverMenu {
    let menu = gio::Menu::new();
    menu.append(Some("Copiar Selecionados"), Some("app.copy-selected"));
    menu.append(Some("Exportar como Texto"), Some("app.export-text"));
    menu.append(Some("Exportar como JSON"), Some("app.export-json"));

    PopoverMenu::from_model(Some(&menu))
}

fn create_boot_selector() -> Box {
    let boot_box = Box::new(Orientation::Horizontal, 12);
    boot_box.set_margin_start(12);
    boot_box.set_margin_end(12);
    boot_box.set_margin_top(8);
    boot_box.set_margin_bottom(8);

    let boot_label = Label::new(Some("Boot:"));
    boot_label.add_css_class("dim-label");

    let boot_combo = gtk4::ComboBoxText::new();
    boot_combo.append(Some("0"), "Boot atual");
    boot_combo.append(Some("-1"), "Boot anterior");
    boot_combo.append(Some("-2"), "2 boots atras");
    boot_combo.append(Some("-3"), "3 boots atras");
    boot_combo.set_active_id(Some("0"));

    let count_label = Label::new(Some("0 entradas"));
    count_label.add_css_class("dim-label");
    count_label.set_hexpand(true);
    count_label.set_halign(gtk4::Align::End);

    boot_box.append(&boot_label);
    boot_box.append(&boot_combo);
    boot_box.append(&count_label);

    boot_box
}

fn create_status_bar() -> Box {
    let status_box = Box::new(Orientation::Horizontal, 12);
    status_box.set_margin_start(12);
    status_box.set_margin_end(12);
    status_box.set_margin_top(6);
    status_box.set_margin_bottom(6);

    let status_label = Label::new(Some("Pronto"));
    status_label.add_css_class("dim-label");
    status_label.set_hexpand(true);
    status_label.set_halign(gtk4::Align::Start);

    let time_label = Label::new(Some("Ultima atualizacao: agora"));
    time_label.add_css_class("dim-label");

    status_box.append(&status_label);
    status_box.append(&time_label);

    status_box
}

fn load_logs(log_list: &gtk4::ListBox, log_entries: &Rc<RefCell<Vec<LogEntry>>>, source: LogSource) {
    // Clear existing entries
    while let Some(child) = log_list.first_child() {
        log_list.remove(&child);
    }

    // Load logs from source
    let entries = match source {
        LogSource::Journal => crate::sources::journald::load_journal_logs(None, 500),
        LogSource::Kernel => crate::sources::kernel::load_kernel_logs(500),
        LogSource::Syslog => crate::sources::syslog::load_syslog_logs(500),
        LogSource::AppLogs => crate::sources::app_logs::load_app_logs(500),
    };

    *log_entries.borrow_mut() = entries.clone();

    // Add entries to list
    for entry in &entries {
        let row = crate::ui::log_row::create_log_row(entry);
        log_list.append(&row);
    }
}

fn apply_filters(log_list: &gtk4::ListBox, entries: &[LogEntry], filter: &FilterState) {
    // Clear and re-add filtered entries
    while let Some(child) = log_list.first_child() {
        log_list.remove(&child);
    }

    let filtered: Vec<&LogEntry> = entries.iter()
        .filter(|e| filter.matches(e))
        .collect();

    for entry in filtered {
        let row = crate::ui::log_row::create_log_row(entry);
        log_list.append(&row);
    }
}

fn start_live_tail(log_list: &gtk4::ListBox, _log_entries: &Rc<RefCell<Vec<LogEntry>>>) {
    // In a real implementation, this would start a background process
    // to monitor journalctl -f and update the list in real time
    let log_list_weak = log_list.downgrade();

    glib::timeout_add_seconds_local(2, move || {
        if let Some(_log_list) = log_list_weak.upgrade() {
            // Would add new entries here in real implementation
        }
        glib::ControlFlow::Continue
    });
}
