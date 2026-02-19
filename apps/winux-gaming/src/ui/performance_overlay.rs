// Performance overlay widget
// Shows real-time system metrics in-app (separate from MangoHud in-game overlay)

use gtk4::prelude::*;
use gtk4::{Box, Frame, Label, LevelBar, Orientation, ProgressBar};
use std::time::Duration;

/// Performance metrics data
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub frametime_ms: f64,
    pub cpu_usage: f64,
    pub cpu_temp: f64,
    pub gpu_usage: f64,
    pub gpu_temp: f64,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
}

impl PerformanceMetrics {
    pub fn vram_percentage(&self) -> f64 {
        if self.vram_total_mb > 0 {
            (self.vram_used_mb as f64 / self.vram_total_mb as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn ram_percentage(&self) -> f64 {
        if self.ram_total_mb > 0 {
            (self.ram_used_mb as f64 / self.ram_total_mb as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Create a compact performance overlay widget
pub fn create_performance_overlay() -> Frame {
    let frame = Frame::builder()
        .css_classes(vec!["performance-overlay"])
        .build();

    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(20)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    // FPS
    let fps_box = create_metric_box("FPS", "60", "fps-value");
    content.append(&fps_box);

    // Frametime
    let frametime_box = create_metric_box("Frame", "16.7ms", "frametime-value");
    content.append(&frametime_box);

    // CPU
    let cpu_box = create_metric_box("CPU", "45%", "cpu-value");
    content.append(&cpu_box);

    // GPU
    let gpu_box = create_metric_box("GPU", "78%", "gpu-value");
    content.append(&gpu_box);

    // Temps
    let temp_box = create_metric_box("Temp", "65C", "temp-value");
    content.append(&temp_box);

    frame.set_child(Some(&content));
    frame
}

/// Create a detailed performance panel
pub fn create_performance_panel() -> Box {
    let panel = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(15)
        .margin_end(15)
        .margin_top(15)
        .margin_bottom(15)
        .css_classes(vec!["card"])
        .build();

    // Header
    let header = Label::builder()
        .label("Performance em Tempo Real")
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .build();
    panel.append(&header);

    // FPS Section
    let fps_section = create_fps_section();
    panel.append(&fps_section);

    // CPU Section
    let cpu_section = create_usage_section("CPU", 45.0, 65.0);
    panel.append(&cpu_section);

    // GPU Section
    let gpu_section = create_usage_section("GPU", 78.0, 72.0);
    panel.append(&gpu_section);

    // Memory Section
    let memory_section = create_memory_section();
    panel.append(&memory_section);

    panel
}

fn create_metric_box(label: &str, value: &str, value_class: &str) -> Box {
    let metric_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .halign(gtk4::Align::Center)
        .build();

    let value_label = Label::builder()
        .label(value)
        .css_classes(vec!["stat-value", value_class])
        .build();
    metric_box.append(&value_label);

    let name_label = Label::builder()
        .label(label)
        .css_classes(vec!["stat-label"])
        .build();
    metric_box.append(&name_label);

    metric_box
}

fn create_fps_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    // FPS header with value
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let label = Label::builder()
        .label("FPS")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&label);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let fps_value = Label::builder()
        .label("60")
        .css_classes(vec!["stat-value"])
        .build();
    header.append(&fps_value);

    section.append(&header);

    // Frametime graph placeholder
    let frametime_bar = LevelBar::builder()
        .min_value(0.0)
        .max_value(33.3) // 30fps = 33.3ms
        .value(16.7) // 60fps
        .build();
    frametime_bar.add_offset_value("good", 16.7);
    frametime_bar.add_offset_value("ok", 25.0);
    frametime_bar.add_offset_value("bad", 33.3);
    section.append(&frametime_bar);

    // Frametime label
    let frametime_label = Label::builder()
        .label("Frametime: 16.7ms (avg)")
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    section.append(&frametime_label);

    section
}

fn create_usage_section(name: &str, usage: f64, temp: f64) -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    // Header with values
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let label = Label::builder()
        .label(name)
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&label);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let usage_label = Label::builder()
        .label(&format!("{:.0}%", usage))
        .css_classes(vec!["title-4"])
        .build();
    header.append(&usage_label);

    let temp_label = Label::builder()
        .label(&format!(" | {:.0}C", temp))
        .css_classes(vec!["dim-label"])
        .build();
    header.append(&temp_label);

    section.append(&header);

    // Usage bar
    let usage_bar = ProgressBar::builder()
        .fraction(usage / 100.0)
        .build();
    section.append(&usage_bar);

    section
}

fn create_memory_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let header = Label::builder()
        .label("Memoria")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    section.append(&header);

    // RAM
    let ram_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let ram_label = Label::builder()
        .label("RAM")
        .css_classes(vec!["caption"])
        .width_request(50)
        .build();
    ram_box.append(&ram_label);

    let ram_bar = ProgressBar::builder()
        .fraction(0.65)
        .hexpand(true)
        .build();
    ram_box.append(&ram_bar);

    let ram_value = Label::builder()
        .label("10.4/16 GB")
        .css_classes(vec!["caption"])
        .build();
    ram_box.append(&ram_value);

    section.append(&ram_box);

    // VRAM
    let vram_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let vram_label = Label::builder()
        .label("VRAM")
        .css_classes(vec!["caption"])
        .width_request(50)
        .build();
    vram_box.append(&vram_label);

    let vram_bar = ProgressBar::builder()
        .fraction(0.45)
        .hexpand(true)
        .build();
    vram_box.append(&vram_bar);

    let vram_value = Label::builder()
        .label("3.6/8 GB")
        .css_classes(vec!["caption"])
        .build();
    vram_box.append(&vram_value);

    section.append(&vram_box);

    section
}

/// Get color class based on temperature
pub fn get_temp_color_class(temp: f64) -> &'static str {
    if temp < 60.0 {
        "success"
    } else if temp < 80.0 {
        "warning"
    } else {
        "error"
    }
}

/// Get color class based on usage percentage
pub fn get_usage_color_class(usage: f64) -> &'static str {
    if usage < 50.0 {
        "success"
    } else if usage < 80.0 {
        "warning"
    } else {
        "error"
    }
}

/// Format bytes to human readable
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
