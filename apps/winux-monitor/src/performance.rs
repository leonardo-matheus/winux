//! Performance monitoring view for Winux Monitor
//!
//! Displays real-time graphs for CPU, memory, disk, and network usage.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::{gdk, glib};
use std::cell::RefCell;
use std::collections::VecDeque;
use sysinfo::{CpuRefreshKind, Disks, Networks, RefreshKind, System};
use tracing::debug;

/// Number of data points to keep in history
const HISTORY_SIZE: usize = 60;

/// Performance data history
#[derive(Default)]
pub struct PerformanceHistory {
    pub cpu: VecDeque<f64>,
    pub memory: VecDeque<f64>,
    pub swap: VecDeque<f64>,
    pub disk_read: VecDeque<f64>,
    pub disk_write: VecDeque<f64>,
    pub network_rx: VecDeque<f64>,
    pub network_tx: VecDeque<f64>,
}

impl PerformanceHistory {
    pub fn new() -> Self {
        let mut history = Self::default();
        // Initialize with zeros
        for _ in 0..HISTORY_SIZE {
            history.cpu.push_back(0.0);
            history.memory.push_back(0.0);
            history.swap.push_back(0.0);
            history.disk_read.push_back(0.0);
            history.disk_write.push_back(0.0);
            history.network_rx.push_back(0.0);
            history.network_tx.push_back(0.0);
        }
        history
    }

    fn push_value(queue: &mut VecDeque<f64>, value: f64) {
        if queue.len() >= HISTORY_SIZE {
            queue.pop_front();
        }
        queue.push_back(value);
    }
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct PerformanceView {
        pub system: RefCell<System>,
        pub disks: RefCell<Disks>,
        pub networks: RefCell<Networks>,
        pub history: RefCell<PerformanceHistory>,
        pub cpu_graph: OnceCell<gtk4::DrawingArea>,
        pub memory_graph: OnceCell<gtk4::DrawingArea>,
        pub disk_graph: OnceCell<gtk4::DrawingArea>,
        pub network_graph: OnceCell<gtk4::DrawingArea>,
        pub cpu_label: OnceCell<gtk4::Label>,
        pub memory_label: OnceCell<gtk4::Label>,
        pub disk_label: OnceCell<gtk4::Label>,
        pub network_label: OnceCell<gtk4::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PerformanceView {
        const NAME: &'static str = "WinuxMonitorPerformanceView";
        type Type = super::PerformanceView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for PerformanceView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for PerformanceView {}
    impl BoxImpl for PerformanceView {}
}

glib::wrapper! {
    pub struct PerformanceView(ObjectSubclass<imp::PerformanceView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl PerformanceView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(24);
        self.set_margin_start(24);
        self.set_margin_end(24);
        self.set_margin_top(24);
        self.set_margin_bottom(24);

        // Initialize system info
        {
            let mut system = imp.system.borrow_mut();
            system.refresh_all();
        }

        // Initialize history
        imp.history.replace(PerformanceHistory::new());

        // Create 2x2 grid of performance graphs
        let grid = gtk4::Grid::new();
        grid.set_row_spacing(24);
        grid.set_column_spacing(24);
        grid.set_row_homogeneous(true);
        grid.set_column_homogeneous(true);

        // CPU section
        let cpu_card = self.create_performance_card(
            "CPU",
            "processor-symbolic",
            &|view| view.imp().cpu_graph.get().cloned(),
            &|view| view.imp().cpu_label.get().cloned(),
        );
        grid.attach(&cpu_card, 0, 0, 1, 1);

        // Memory section
        let memory_card = self.create_performance_card(
            "Memory",
            "drive-harddisk-symbolic",
            &|view| view.imp().memory_graph.get().cloned(),
            &|view| view.imp().memory_label.get().cloned(),
        );
        grid.attach(&memory_card, 1, 0, 1, 1);

        // Disk section
        let disk_card = self.create_performance_card(
            "Disk",
            "drive-harddisk-symbolic",
            &|view| view.imp().disk_graph.get().cloned(),
            &|view| view.imp().disk_label.get().cloned(),
        );
        grid.attach(&disk_card, 0, 1, 1, 1);

        // Network section
        let network_card = self.create_performance_card(
            "Network",
            "network-wired-symbolic",
            &|view| view.imp().network_graph.get().cloned(),
            &|view| view.imp().network_label.get().cloned(),
        );
        grid.attach(&network_card, 1, 1, 1, 1);

        self.append(&grid);

        // Setup drawing
        self.setup_graphs();

        // Initial refresh
        self.refresh();
    }

    fn create_performance_card(
        &self,
        title: &str,
        icon: &str,
        graph_getter: &dyn Fn(&Self) -> Option<gtk4::DrawingArea>,
        label_getter: &dyn Fn(&Self) -> Option<gtk4::Label>,
    ) -> gtk4::Box {
        let imp = self.imp();

        let card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        card.add_css_class("card");
        card.set_margin_start(12);
        card.set_margin_end(12);
        card.set_margin_top(12);
        card.set_margin_bottom(12);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let icon_widget = gtk4::Image::from_icon_name(icon);
        header.append(&icon_widget);

        let title_label = gtk4::Label::new(Some(title));
        title_label.add_css_class("title-3");
        title_label.set_hexpand(true);
        title_label.set_halign(gtk4::Align::Start);
        header.append(&title_label);

        let value_label = gtk4::Label::new(Some("0%"));
        value_label.add_css_class("title-2");

        // Store reference based on title
        match title {
            "CPU" => { imp.cpu_label.set(value_label.clone()).ok(); }
            "Memory" => { imp.memory_label.set(value_label.clone()).ok(); }
            "Disk" => { imp.disk_label.set(value_label.clone()).ok(); }
            "Network" => { imp.network_label.set(value_label.clone()).ok(); }
            _ => {}
        }

        header.append(&value_label);
        card.append(&header);

        // Graph
        let graph = gtk4::DrawingArea::new();
        graph.set_height_request(150);
        graph.set_vexpand(true);
        graph.add_css_class("performance-graph");

        // Store reference
        match title {
            "CPU" => { imp.cpu_graph.set(graph.clone()).ok(); }
            "Memory" => { imp.memory_graph.set(graph.clone()).ok(); }
            "Disk" => { imp.disk_graph.set(graph.clone()).ok(); }
            "Network" => { imp.network_graph.set(graph.clone()).ok(); }
            _ => {}
        }

        card.append(&graph);

        card
    }

    fn setup_graphs(&self) {
        let imp = self.imp();

        // Setup CPU graph drawing
        if let Some(graph) = imp.cpu_graph.get() {
            let view_weak = self.downgrade();
            graph.set_draw_func(move |_, cr, width, height| {
                if let Some(view) = view_weak.upgrade() {
                    let history = view.imp().history.borrow();
                    draw_graph(cr, width, height, &history.cpu, (0.2, 0.6, 1.0));
                }
            });
        }

        // Setup Memory graph drawing
        if let Some(graph) = imp.memory_graph.get() {
            let view_weak = self.downgrade();
            graph.set_draw_func(move |_, cr, width, height| {
                if let Some(view) = view_weak.upgrade() {
                    let history = view.imp().history.borrow();
                    draw_graph(cr, width, height, &history.memory, (0.6, 0.2, 0.8));
                }
            });
        }

        // Setup Disk graph drawing
        if let Some(graph) = imp.disk_graph.get() {
            let view_weak = self.downgrade();
            graph.set_draw_func(move |_, cr, width, height| {
                if let Some(view) = view_weak.upgrade() {
                    let history = view.imp().history.borrow();
                    draw_dual_graph(
                        cr,
                        width,
                        height,
                        &history.disk_read,
                        &history.disk_write,
                        (0.2, 0.8, 0.4),
                        (0.8, 0.4, 0.2),
                    );
                }
            });
        }

        // Setup Network graph drawing
        if let Some(graph) = imp.network_graph.get() {
            let view_weak = self.downgrade();
            graph.set_draw_func(move |_, cr, width, height| {
                if let Some(view) = view_weak.upgrade() {
                    let history = view.imp().history.borrow();
                    draw_dual_graph(
                        cr,
                        width,
                        height,
                        &history.network_rx,
                        &history.network_tx,
                        (0.2, 0.7, 0.9),
                        (0.9, 0.5, 0.2),
                    );
                }
            });
        }
    }

    pub fn refresh(&self) {
        let imp = self.imp();

        // Refresh system data
        {
            let mut system = imp.system.borrow_mut();
            system.refresh_cpu_specifics(CpuRefreshKind::everything());
            system.refresh_memory();
        }

        {
            let mut disks = imp.disks.borrow_mut();
            disks.refresh_list();
        }

        {
            let mut networks = imp.networks.borrow_mut();
            networks.refresh_list();
        }

        let system = imp.system.borrow();

        // Calculate CPU usage
        let cpu_usage: f64 = system.cpus().iter().map(|c| c.cpu_usage() as f64).sum::<f64>()
            / system.cpus().len() as f64;

        // Calculate memory usage
        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;
        let memory_percent = (used_memory / total_memory) * 100.0;

        // Update history
        {
            let mut history = imp.history.borrow_mut();
            PerformanceHistory::push_value(&mut history.cpu, cpu_usage);
            PerformanceHistory::push_value(&mut history.memory, memory_percent);

            // Disk and network would need proper tracking in production
            // For now, use placeholder values
            PerformanceHistory::push_value(&mut history.disk_read, 0.0);
            PerformanceHistory::push_value(&mut history.disk_write, 0.0);
            PerformanceHistory::push_value(&mut history.network_rx, 0.0);
            PerformanceHistory::push_value(&mut history.network_tx, 0.0);
        }

        // Update labels
        if let Some(label) = imp.cpu_label.get() {
            label.set_text(&format!("{:.1}%", cpu_usage));
        }

        if let Some(label) = imp.memory_label.get() {
            let used_gb = used_memory / 1024.0 / 1024.0 / 1024.0;
            let total_gb = total_memory / 1024.0 / 1024.0 / 1024.0;
            label.set_text(&format!("{:.1}/{:.1} GB", used_gb, total_gb));
        }

        if let Some(label) = imp.disk_label.get() {
            label.set_text("0 MB/s");
        }

        if let Some(label) = imp.network_label.get() {
            label.set_text("0 KB/s");
        }

        // Request graph redraws
        if let Some(graph) = imp.cpu_graph.get() {
            graph.queue_draw();
        }
        if let Some(graph) = imp.memory_graph.get() {
            graph.queue_draw();
        }
        if let Some(graph) = imp.disk_graph.get() {
            graph.queue_draw();
        }
        if let Some(graph) = imp.network_graph.get() {
            graph.queue_draw();
        }
    }
}

impl Default for PerformanceView {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a single-line performance graph
fn draw_graph(
    cr: &gtk4::cairo::Context,
    width: i32,
    height: i32,
    data: &VecDeque<f64>,
    color: (f64, f64, f64),
) {
    let width = width as f64;
    let height = height as f64;
    let padding = 2.0;

    // Draw background
    cr.set_source_rgba(0.1, 0.1, 0.1, 0.3);
    cr.rectangle(0.0, 0.0, width, height);
    let _ = cr.fill();

    // Draw grid lines
    cr.set_source_rgba(0.3, 0.3, 0.3, 0.3);
    cr.set_line_width(0.5);

    for i in 1..4 {
        let y = height * (i as f64 / 4.0);
        cr.move_to(0.0, y);
        cr.line_to(width, y);
        let _ = cr.stroke();
    }

    if data.is_empty() {
        return;
    }

    // Draw line graph
    let step = (width - 2.0 * padding) / (HISTORY_SIZE - 1) as f64;

    // Draw filled area
    cr.move_to(padding, height - padding);
    for (i, &value) in data.iter().enumerate() {
        let x = padding + i as f64 * step;
        let y = height - padding - (value / 100.0) * (height - 2.0 * padding);
        cr.line_to(x, y);
    }
    cr.line_to(padding + (data.len() - 1) as f64 * step, height - padding);
    cr.close_path();

    // Create gradient for fill
    let gradient = gtk4::cairo::LinearGradient::new(0.0, 0.0, 0.0, height);
    gradient.add_color_stop_rgba(0.0, color.0, color.1, color.2, 0.6);
    gradient.add_color_stop_rgba(1.0, color.0, color.1, color.2, 0.1);
    cr.set_source(&gradient).ok();
    let _ = cr.fill_preserve();

    // Draw line
    cr.set_source_rgba(color.0, color.1, color.2, 1.0);
    cr.set_line_width(2.0);

    cr.move_to(padding, height - padding - (data[0] / 100.0) * (height - 2.0 * padding));
    for (i, &value) in data.iter().enumerate().skip(1) {
        let x = padding + i as f64 * step;
        let y = height - padding - (value / 100.0) * (height - 2.0 * padding);
        cr.line_to(x, y);
    }
    let _ = cr.stroke();
}

/// Draw a dual-line graph (for read/write, upload/download)
fn draw_dual_graph(
    cr: &gtk4::cairo::Context,
    width: i32,
    height: i32,
    data1: &VecDeque<f64>,
    data2: &VecDeque<f64>,
    color1: (f64, f64, f64),
    color2: (f64, f64, f64),
) {
    let width = width as f64;
    let height = height as f64;
    let padding = 2.0;

    // Draw background
    cr.set_source_rgba(0.1, 0.1, 0.1, 0.3);
    cr.rectangle(0.0, 0.0, width, height);
    let _ = cr.fill();

    // Draw grid lines
    cr.set_source_rgba(0.3, 0.3, 0.3, 0.3);
    cr.set_line_width(0.5);

    for i in 1..4 {
        let y = height * (i as f64 / 4.0);
        cr.move_to(0.0, y);
        cr.line_to(width, y);
        let _ = cr.stroke();
    }

    // Find max value for scaling
    let max_val = data1
        .iter()
        .chain(data2.iter())
        .cloned()
        .fold(1.0f64, f64::max);

    let step = (width - 2.0 * padding) / (HISTORY_SIZE - 1) as f64;

    // Draw first line
    if !data1.is_empty() {
        cr.set_source_rgba(color1.0, color1.1, color1.2, 0.8);
        cr.set_line_width(1.5);

        cr.move_to(padding, height - padding - (data1[0] / max_val) * (height - 2.0 * padding));
        for (i, &value) in data1.iter().enumerate().skip(1) {
            let x = padding + i as f64 * step;
            let y = height - padding - (value / max_val) * (height - 2.0 * padding);
            cr.line_to(x, y);
        }
        let _ = cr.stroke();
    }

    // Draw second line
    if !data2.is_empty() {
        cr.set_source_rgba(color2.0, color2.1, color2.2, 0.8);
        cr.set_line_width(1.5);

        cr.move_to(padding, height - padding - (data2[0] / max_val) * (height - 2.0 * padding));
        for (i, &value) in data2.iter().enumerate().skip(1) {
            let x = padding + i as f64 * step;
            let y = height - padding - (value / max_val) * (height - 2.0 * padding);
            cr.line_to(x, y);
        }
        let _ = cr.stroke();
    }
}
