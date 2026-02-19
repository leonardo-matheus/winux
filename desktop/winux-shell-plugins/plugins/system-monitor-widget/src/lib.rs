//! System Monitor Widget Plugin
//!
//! Shows CPU, RAM, and network usage in a compact panel widget.

use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use winux_shell_plugins::prelude::*;

/// Number of data points to keep for graphs
const HISTORY_SIZE: usize = 60;

/// System metrics data
#[derive(Debug, Clone, Default)]
struct SystemMetrics {
    /// CPU usage (0-100)
    cpu_usage: f32,
    /// Memory usage (0-100)
    memory_usage: f32,
    /// Used memory in bytes
    memory_used: u64,
    /// Total memory in bytes
    memory_total: u64,
    /// Network download speed (bytes/sec)
    network_down: u64,
    /// Network upload speed (bytes/sec)
    network_up: u64,
    /// CPU usage history
    cpu_history: VecDeque<f32>,
    /// Memory usage history
    memory_history: VecDeque<f32>,
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitorConfig {
    /// Show CPU usage
    show_cpu: bool,
    /// Show memory usage
    show_memory: bool,
    /// Show network usage
    show_network: bool,
    /// Update interval in milliseconds
    update_interval: u32,
    /// Graph color for CPU
    cpu_color: (f64, f64, f64),
    /// Graph color for memory
    memory_color: (f64, f64, f64),
    /// Show mini graph
    show_graph: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            show_cpu: true,
            show_memory: true,
            show_network: false,
            update_interval: 1000,
            cpu_color: (0.2, 0.6, 1.0),    // Blue
            memory_color: (0.8, 0.4, 0.2), // Orange
            show_graph: true,
        }
    }
}

/// System monitor widget plugin
pub struct SystemMonitorPlugin {
    config: MonitorConfig,
    metrics: Arc<RwLock<SystemMetrics>>,
    system: System,
}

impl Default for SystemMonitorPlugin {
    fn default() -> Self {
        let mut metrics = SystemMetrics::default();
        metrics.cpu_history = VecDeque::with_capacity(HISTORY_SIZE);
        metrics.memory_history = VecDeque::with_capacity(HISTORY_SIZE);

        Self {
            config: MonitorConfig::default(),
            metrics: Arc::new(RwLock::new(metrics)),
            system: System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
        }
    }
}

impl SystemMonitorPlugin {
    /// Update system metrics
    fn update_metrics(&mut self) {
        self.system.refresh_cpu();
        self.system.refresh_memory();

        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        let memory_total = self.system.total_memory();
        let memory_used = self.system.used_memory();
        let memory_usage = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        let mut metrics = self.metrics.write().unwrap();
        metrics.cpu_usage = cpu_usage;
        metrics.memory_usage = memory_usage;
        metrics.memory_used = memory_used;
        metrics.memory_total = memory_total;

        // Update history
        if metrics.cpu_history.len() >= HISTORY_SIZE {
            metrics.cpu_history.pop_front();
        }
        metrics.cpu_history.push_back(cpu_usage);

        if metrics.memory_history.len() >= HISTORY_SIZE {
            metrics.memory_history.pop_front();
        }
        metrics.memory_history.push_back(memory_usage);
    }

    /// Format bytes to human readable
    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.0} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.0} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

impl Plugin for SystemMonitorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "org.winux.system-monitor-widget".into(),
            name: "System Monitor".into(),
            version: Version::new(1, 0, 0),
            description: "Shows CPU, RAM, and network usage in the panel".into(),
            authors: vec!["Winux Team".into()],
            homepage: Some("https://winux.org/plugins/system-monitor".into()),
            license: Some("MIT".into()),
            min_api_version: Version::new(1, 0, 0),
            capabilities: vec![PluginCapability::PanelWidget, PluginCapability::SystemInfo],
            permissions: {
                let mut perms = PermissionSet::new();
                perms.add(Permission::SystemInfo);
                perms.add(Permission::PanelWidgets);
                perms.add(Permission::OwnData);
                perms
            },
            icon: Some("utilities-system-monitor-symbolic".into()),
            category: Some("System".into()),
            keywords: vec!["cpu".into(), "memory".into(), "ram".into(), "monitor".into(), "usage".into()],
            ..Default::default()
        }
    }

    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        // Load config
        let config_path = ctx.config_file("config.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    self.config = config;
                }
            }
        }

        // Initial update
        self.update_metrics();

        log::info!("System monitor widget initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        log::info!("System monitor widget shutting down");
        Ok(())
    }

    fn panel_widget(&self) -> Option<Box<dyn PanelWidget>> {
        Some(Box::new(SystemMonitorPanelWidget {
            metrics: self.metrics.clone(),
            config: self.config.clone(),
        }))
    }

    fn wants_updates(&self) -> bool {
        true
    }

    fn update_interval(&self) -> u32 {
        self.config.update_interval
    }

    fn update(&mut self) -> PluginResult<()> {
        self.update_metrics();
        Ok(())
    }
}

/// Panel widget for system monitor
struct SystemMonitorPanelWidget {
    metrics: Arc<RwLock<SystemMetrics>>,
    config: MonitorConfig,
}

impl PanelWidget for SystemMonitorPanelWidget {
    fn id(&self) -> &str {
        "system-monitor-widget"
    }

    fn name(&self) -> &str {
        "System Monitor"
    }

    fn position(&self) -> PanelPosition {
        PanelPosition::Right
    }

    fn size(&self) -> WidgetSize {
        if self.config.show_graph {
            WidgetSize::Medium
        } else {
            WidgetSize::Small
        }
    }

    fn priority(&self) -> i32 {
        5
    }

    fn state(&self) -> WidgetState {
        let metrics = self.metrics.read().unwrap();

        let label = if self.config.show_cpu && self.config.show_memory {
            format!("{}% | {}%", metrics.cpu_usage as i32, metrics.memory_usage as i32)
        } else if self.config.show_cpu {
            format!("{}%", metrics.cpu_usage as i32)
        } else if self.config.show_memory {
            format!("{}%", metrics.memory_usage as i32)
        } else {
            String::new()
        };

        WidgetState::with_icon("utilities-system-monitor-symbolic")
            .label(&label)
            .tooltip(&format!(
                "CPU: {:.1}%\nMemory: {} / {}",
                metrics.cpu_usage,
                SystemMonitorPlugin::format_bytes(metrics.memory_used),
                SystemMonitorPlugin::format_bytes(metrics.memory_total)
            ))
    }

    fn build_widget(&self) -> gtk::Widget {
        let metrics = self.metrics.read().unwrap();

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        hbox.set_valign(gtk::Align::Center);
        hbox.add_css_class("system-monitor-widget");

        if self.config.show_graph {
            // Mini CPU graph
            let cpu_graph = self.build_mini_graph(&metrics.cpu_history, self.config.cpu_color);
            hbox.append(&cpu_graph);

            // Mini Memory graph
            let mem_graph = self.build_mini_graph(&metrics.memory_history, self.config.memory_color);
            hbox.append(&mem_graph);
        } else {
            // Icon
            let icon = gtk::Image::from_icon_name("utilities-system-monitor-symbolic");
            icon.set_pixel_size(16);
            hbox.append(&icon);

            // Text
            if self.config.show_cpu {
                let cpu_label = gtk::Label::new(Some(&format!("{}%", metrics.cpu_usage as i32)));
                cpu_label.add_css_class("cpu-usage");
                hbox.append(&cpu_label);
            }

            if self.config.show_memory {
                let mem_label = gtk::Label::new(Some(&format!("{}%", metrics.memory_usage as i32)));
                mem_label.add_css_class("mem-usage");
                hbox.append(&mem_label);
            }
        }

        // Tooltip
        let tooltip = format!(
            "CPU: {:.1}%\nMemory: {} / {} ({:.1}%)",
            metrics.cpu_usage,
            SystemMonitorPlugin::format_bytes(metrics.memory_used),
            SystemMonitorPlugin::format_bytes(metrics.memory_total),
            metrics.memory_usage
        );
        hbox.set_tooltip_text(Some(&tooltip));

        hbox.upcast()
    }

    fn on_click(&mut self) -> WidgetAction {
        WidgetAction::ShowPopup
    }

    fn popup_config(&self) -> Option<PopupConfig> {
        Some(PopupConfig {
            width: 300,
            height: 400,
            ..Default::default()
        })
    }

    fn build_popup(&self) -> Option<gtk::Widget> {
        let metrics = self.metrics.read().unwrap();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
        vbox.set_margin_top(16);
        vbox.set_margin_bottom(16);
        vbox.set_margin_start(16);
        vbox.set_margin_end(16);
        vbox.add_css_class("system-monitor-popup");

        // Title
        let title = gtk::Label::new(Some("System Monitor"));
        title.add_css_class("title-3");
        vbox.append(&title);

        // CPU Section
        let cpu_frame = self.build_metric_frame(
            "CPU Usage",
            &format!("{:.1}%", metrics.cpu_usage),
            metrics.cpu_usage / 100.0,
            &metrics.cpu_history,
            self.config.cpu_color,
        );
        vbox.append(&cpu_frame);

        // Memory Section
        let mem_frame = self.build_metric_frame(
            "Memory Usage",
            &format!(
                "{} / {} ({:.1}%)",
                SystemMonitorPlugin::format_bytes(metrics.memory_used),
                SystemMonitorPlugin::format_bytes(metrics.memory_total),
                metrics.memory_usage
            ),
            metrics.memory_usage / 100.0,
            &metrics.memory_history,
            self.config.memory_color,
        );
        vbox.append(&mem_frame);

        // Open system monitor button
        let open_button = gtk::Button::with_label("Open System Monitor");
        open_button.connect_clicked(|_| {
            let _ = std::process::Command::new("winux-monitor").spawn();
        });
        open_button.set_margin_top(12);
        vbox.append(&open_button);

        Some(vbox.upcast())
    }
}

impl SystemMonitorPanelWidget {
    /// Build a mini graph widget
    fn build_mini_graph(&self, data: &VecDeque<f32>, color: (f64, f64, f64)) -> gtk::DrawingArea {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_size_request(40, 24);

        let data: Vec<f32> = data.iter().copied().collect();
        let (r, g, b) = color;

        drawing_area.set_draw_func(move |_, cr, width, height| {
            let width = width as f64;
            let height = height as f64;

            // Background
            cr.set_source_rgba(0.2, 0.2, 0.2, 0.5);
            cr.rectangle(0.0, 0.0, width, height);
            let _ = cr.fill();

            if data.is_empty() {
                return;
            }

            // Draw graph
            cr.set_source_rgba(r, g, b, 0.8);
            cr.set_line_width(1.0);

            let step = width / data.len().max(1) as f64;

            cr.move_to(0.0, height);
            for (i, &value) in data.iter().enumerate() {
                let x = i as f64 * step;
                let y = height - (value as f64 / 100.0 * height);
                cr.line_to(x, y);
            }
            cr.line_to(width, height);
            cr.close_path();

            // Fill
            cr.set_source_rgba(r, g, b, 0.3);
            let _ = cr.fill_preserve();

            // Stroke
            cr.set_source_rgba(r, g, b, 0.8);
            let _ = cr.stroke();
        });

        drawing_area
    }

    /// Build a metric frame with graph
    fn build_metric_frame(
        &self,
        title: &str,
        value: &str,
        progress: f32,
        history: &VecDeque<f32>,
        color: (f64, f64, f64),
    ) -> gtk::Box {
        let frame = gtk::Box::new(gtk::Orientation::Vertical, 8);
        frame.add_css_class("metric-frame");
        frame.set_margin_top(8);

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let title_label = gtk::Label::new(Some(title));
        title_label.set_halign(gtk::Align::Start);
        title_label.add_css_class("title-4");
        header.append(&title_label);

        let value_label = gtk::Label::new(Some(value));
        value_label.set_halign(gtk::Align::End);
        value_label.set_hexpand(true);
        header.append(&value_label);

        frame.append(&header);

        // Progress bar
        let progress_bar = gtk::ProgressBar::new();
        progress_bar.set_fraction(progress as f64);
        frame.append(&progress_bar);

        // Graph
        let graph = gtk::DrawingArea::new();
        graph.set_size_request(-1, 60);

        let data: Vec<f32> = history.iter().copied().collect();
        let (r, g, b) = color;

        graph.set_draw_func(move |_, cr, width, height| {
            let width = width as f64;
            let height = height as f64;

            // Background
            cr.set_source_rgba(0.1, 0.1, 0.1, 0.5);
            cr.rectangle(0.0, 0.0, width, height);
            let _ = cr.fill();

            if data.is_empty() {
                return;
            }

            // Grid lines
            cr.set_source_rgba(0.3, 0.3, 0.3, 0.5);
            cr.set_line_width(0.5);
            for i in 1..4 {
                let y = height * i as f64 / 4.0;
                cr.move_to(0.0, y);
                cr.line_to(width, y);
            }
            let _ = cr.stroke();

            // Draw graph
            let step = width / data.len().max(1) as f64;

            cr.move_to(0.0, height);
            for (i, &value) in data.iter().enumerate() {
                let x = i as f64 * step;
                let y = height - (value as f64 / 100.0 * height);
                cr.line_to(x, y);
            }
            cr.line_to(width, height);
            cr.close_path();

            // Fill gradient
            cr.set_source_rgba(r, g, b, 0.2);
            let _ = cr.fill_preserve();

            // Stroke
            cr.set_line_width(2.0);
            cr.set_source_rgba(r, g, b, 0.9);
            let _ = cr.stroke();
        });

        frame.append(&graph);

        frame
    }
}

// Plugin entry point
winux_shell_plugins::declare_plugin!(SystemMonitorPlugin, SystemMonitorPlugin::default);
