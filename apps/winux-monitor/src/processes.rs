//! Process list view for Winux Monitor
//!
//! Displays running processes with CPU, memory usage, and management options.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use std::cell::RefCell;
use sysinfo::{Pid, ProcessStatus, System};
use tracing::{debug, info, warn};

/// Process information for display
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub status: String,
    pub user: String,
    pub command: String,
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct ProcessesView {
        pub system: RefCell<System>,
        pub processes: RefCell<Vec<ProcessInfo>>,
        pub list_store: OnceCell<gio::ListStore>,
        pub column_view: OnceCell<gtk4::ColumnView>,
        pub search_entry: OnceCell<gtk4::SearchEntry>,
        pub search_bar: OnceCell<gtk4::SearchBar>,
        pub sort_column: RefCell<String>,
        pub sort_ascending: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProcessesView {
        const NAME: &'static str = "WinuxMonitorProcessesView";
        type Type = super::ProcessesView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for ProcessesView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for ProcessesView {}
    impl BoxImpl for ProcessesView {}
}

glib::wrapper! {
    pub struct ProcessesView(ObjectSubclass<imp::ProcessesView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl ProcessesView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(0);

        // Initialize system info
        {
            let mut system = imp.system.borrow_mut();
            system.refresh_all();
        }

        // Search bar
        let search_bar = gtk4::SearchBar::new();
        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search processes..."));
        search_entry.set_hexpand(true);
        search_bar.set_child(Some(&search_entry));
        search_bar.connect_entry(&search_entry);

        imp.search_entry.set(search_entry.clone()).unwrap();
        imp.search_bar.set(search_bar.clone()).unwrap();

        self.append(&search_bar);

        // Toolbar
        let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        toolbar.set_margin_start(12);
        toolbar.set_margin_end(12);
        toolbar.set_margin_top(6);
        toolbar.set_margin_bottom(6);

        // Process count label
        let count_label = gtk4::Label::new(Some("0 processes"));
        count_label.add_css_class("dim-label");
        toolbar.append(&count_label);

        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&spacer);

        // End process button
        let end_button = gtk4::Button::with_label("End Process");
        end_button.add_css_class("destructive-action");
        end_button.set_sensitive(false);
        toolbar.append(&end_button);

        self.append(&toolbar);

        // Process list with column view
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);

        // Create list store with process object
        let list_store = gio::ListStore::new::<ProcessObject>();
        imp.list_store.set(list_store.clone()).unwrap();

        // Selection model
        let selection = gtk4::SingleSelection::new(Some(list_store.clone()));

        // Column view
        let column_view = gtk4::ColumnView::new(Some(selection.clone()));
        column_view.add_css_class("data-table");

        // Name column
        let name_factory = gtk4::SignalListItemFactory::new();
        name_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let label = gtk4::Label::new(None);
            label.set_halign(gtk4::Align::Start);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            item.set_child(Some(&label));
        });
        name_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let obj = item.item().and_downcast::<ProcessObject>().unwrap();
            let label = item.child().and_downcast::<gtk4::Label>().unwrap();
            label.set_text(&obj.name());
        });

        let name_column = gtk4::ColumnViewColumn::new(Some("Process Name"), Some(name_factory));
        name_column.set_resizable(true);
        name_column.set_expand(true);
        column_view.append_column(&name_column);

        // PID column
        let pid_factory = gtk4::SignalListItemFactory::new();
        pid_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let label = gtk4::Label::new(None);
            label.set_halign(gtk4::Align::End);
            item.set_child(Some(&label));
        });
        pid_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let obj = item.item().and_downcast::<ProcessObject>().unwrap();
            let label = item.child().and_downcast::<gtk4::Label>().unwrap();
            label.set_text(&obj.pid().to_string());
        });

        let pid_column = gtk4::ColumnViewColumn::new(Some("PID"), Some(pid_factory));
        pid_column.set_resizable(true);
        pid_column.set_fixed_width(80);
        column_view.append_column(&pid_column);

        // CPU column
        let cpu_factory = gtk4::SignalListItemFactory::new();
        cpu_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let progress = gtk4::ProgressBar::new();
            progress.set_show_text(true);
            progress.set_width_request(100);
            item.set_child(Some(&progress));
        });
        cpu_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let obj = item.item().and_downcast::<ProcessObject>().unwrap();
            let progress = item.child().and_downcast::<gtk4::ProgressBar>().unwrap();
            let cpu = obj.cpu();
            progress.set_fraction((cpu / 100.0) as f64);
            progress.set_text(Some(&format!("{:.1}%", cpu)));
        });

        let cpu_column = gtk4::ColumnViewColumn::new(Some("CPU"), Some(cpu_factory));
        cpu_column.set_resizable(true);
        cpu_column.set_fixed_width(120);
        column_view.append_column(&cpu_column);

        // Memory column
        let mem_factory = gtk4::SignalListItemFactory::new();
        mem_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let label = gtk4::Label::new(None);
            label.set_halign(gtk4::Align::End);
            item.set_child(Some(&label));
        });
        mem_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let obj = item.item().and_downcast::<ProcessObject>().unwrap();
            let label = item.child().and_downcast::<gtk4::Label>().unwrap();
            let mem = obj.memory();
            label.set_text(&format_memory(mem));
        });

        let mem_column = gtk4::ColumnViewColumn::new(Some("Memory"), Some(mem_factory));
        mem_column.set_resizable(true);
        mem_column.set_fixed_width(100);
        column_view.append_column(&mem_column);

        // Status column
        let status_factory = gtk4::SignalListItemFactory::new();
        status_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let label = gtk4::Label::new(None);
            label.set_halign(gtk4::Align::Start);
            item.set_child(Some(&label));
        });
        status_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let obj = item.item().and_downcast::<ProcessObject>().unwrap();
            let label = item.child().and_downcast::<gtk4::Label>().unwrap();
            label.set_text(&obj.status());
        });

        let status_column = gtk4::ColumnViewColumn::new(Some("Status"), Some(status_factory));
        status_column.set_resizable(true);
        status_column.set_fixed_width(80);
        column_view.append_column(&status_column);

        imp.column_view.set(column_view.clone()).unwrap();

        scrolled.set_child(Some(&column_view));
        self.append(&scrolled);

        // Connect search
        let view_weak = self.downgrade();
        search_entry.connect_search_changed(move |entry| {
            if let Some(view) = view_weak.upgrade() {
                view.filter_processes(&entry.text());
            }
        });

        // Connect selection for end process button
        let end_button_clone = end_button.clone();
        selection.connect_selected_notify(move |sel| {
            end_button_clone.set_sensitive(sel.selected() != gtk4::INVALID_LIST_POSITION);
        });

        // Connect end process button
        let view_weak = self.downgrade();
        let selection_clone = selection.clone();
        end_button.connect_clicked(move |_| {
            if let Some(view) = view_weak.upgrade() {
                if let Some(item) = selection_clone.selected_item() {
                    if let Some(obj) = item.downcast_ref::<ProcessObject>() {
                        view.end_process(obj.pid());
                    }
                }
            }
        });

        // Initial refresh
        self.refresh();
    }

    pub fn refresh(&self) {
        let imp = self.imp();

        // Refresh system info
        {
            let mut system = imp.system.borrow_mut();
            system.refresh_all();
        }

        let system = imp.system.borrow();
        let mut processes: Vec<ProcessInfo> = system
            .processes()
            .iter()
            .map(|(pid, process)| {
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                    status: format_status(process.status()),
                    user: process
                        .user_id()
                        .map(|u| u.to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    command: process.cmd().join(" "),
                }
            })
            .collect();

        // Sort by CPU usage by default
        processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Update list store
        if let Some(store) = imp.list_store.get() {
            store.remove_all();
            for process in &processes {
                let obj = ProcessObject::new(process);
                store.append(&obj);
            }
        }

        imp.processes.replace(processes);
    }

    fn filter_processes(&self, query: &str) {
        let imp = self.imp();
        let query_lower = query.to_lowercase();

        if let Some(store) = imp.list_store.get() {
            store.remove_all();

            let processes = imp.processes.borrow();
            for process in processes.iter() {
                if query.is_empty()
                    || process.name.to_lowercase().contains(&query_lower)
                    || process.pid.to_string().contains(query)
                {
                    let obj = ProcessObject::new(process);
                    store.append(&obj);
                }
            }
        }
    }

    fn end_process(&self, pid: u32) {
        info!("Attempting to end process: {}", pid);

        // Show confirmation dialog
        let dialog = adw::AlertDialog::builder()
            .heading("End Process?")
            .body(&format!(
                "Are you sure you want to end process {}? Unsaved data may be lost.",
                pid
            ))
            .build();

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("end", "End Process");
        dialog.set_response_appearance("end", adw::ResponseAppearance::Destructive);

        let view_weak = self.downgrade();
        dialog.connect_response(None, move |_, response| {
            if response == "end" {
                if let Some(view) = view_weak.upgrade() {
                    view.kill_process(pid);
                }
            }
        });

        if let Some(root) = self.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                dialog.present(Some(window));
            }
        }
    }

    fn kill_process(&self, pid: u32) {
        let imp = self.imp();
        let system = imp.system.borrow();

        if let Some(process) = system.process(Pid::from_u32(pid)) {
            if process.kill() {
                info!("Successfully killed process: {}", pid);
            } else {
                warn!("Failed to kill process: {}", pid);
            }
        }

        // Refresh after killing
        drop(system);
        self.refresh();
    }

    pub fn toggle_search(&self) {
        if let Some(search_bar) = self.imp().search_bar.get() {
            search_bar.set_search_mode(!search_bar.is_search_mode());
        }
    }
}

impl Default for ProcessesView {
    fn default() -> Self {
        Self::new()
    }
}

/// Format process status
fn format_status(status: ProcessStatus) -> String {
    match status {
        ProcessStatus::Run => "Running".to_string(),
        ProcessStatus::Sleep => "Sleeping".to_string(),
        ProcessStatus::Stop => "Stopped".to_string(),
        ProcessStatus::Zombie => "Zombie".to_string(),
        ProcessStatus::Idle => "Idle".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Format memory size
fn format_memory(mb: f64) -> String {
    if mb >= 1024.0 {
        format!("{:.1} GB", mb / 1024.0)
    } else {
        format!("{:.1} MB", mb)
    }
}

// GObject wrapper for process data
mod process_object {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    pub struct ProcessObjectPrivate {
        pub pid: Cell<u32>,
        pub name: RefCell<String>,
        pub cpu: Cell<f32>,
        pub memory: Cell<f64>,
        pub status: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProcessObjectPrivate {
        const NAME: &'static str = "WinuxMonitorProcessObject";
        type Type = super::ProcessObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ProcessObjectPrivate {}
}

glib::wrapper! {
    pub struct ProcessObject(ObjectSubclass<process_object::ProcessObjectPrivate>);
}

impl ProcessObject {
    pub fn new(info: &ProcessInfo) -> Self {
        let obj: Self = glib::Object::builder().build();
        let priv_ = obj.imp();
        priv_.pid.set(info.pid);
        priv_.name.replace(info.name.clone());
        priv_.cpu.set(info.cpu_usage);
        priv_.memory.set(info.memory_mb);
        priv_.status.replace(info.status.clone());
        obj
    }

    pub fn pid(&self) -> u32 {
        self.imp().pid.get()
    }

    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }

    pub fn cpu(&self) -> f32 {
        self.imp().cpu.get()
    }

    pub fn memory(&self) -> f64 {
        self.imp().memory.get()
    }

    pub fn status(&self) -> String {
        self.imp().status.borrow().clone()
    }
}

use gtk4::gio;
