// Winux Monitor - System resource monitor
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Label, Orientation, HeaderBar, ProgressBar, Grid, Frame};
use libadwaita as adw;
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "org.winux.monitor";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let header = HeaderBar::new();
    header.set_title_widget(Some(&Label::new(Some("System Monitor"))));

    let main_box = Box::new(Orientation::Vertical, 24);
    main_box.set_margin_all(24);

    // CPU Section
    let cpu_frame = Frame::new(Some("CPU"));
    let cpu_box = Box::new(Orientation::Vertical, 12);
    cpu_box.set_margin_all(12);

    let cpu_label = Label::new(Some("CPU Usage"));
    cpu_label.set_xalign(0.0);
    let cpu_bar = ProgressBar::new();
    cpu_bar.set_show_text(true);

    cpu_box.append(&cpu_label);
    cpu_box.append(&cpu_bar);
    cpu_frame.set_child(Some(&cpu_box));

    // Memory Section
    let mem_frame = Frame::new(Some("Memory"));
    let mem_box = Box::new(Orientation::Vertical, 12);
    mem_box.set_margin_all(12);

    let mem_label = Label::new(Some("Memory Usage"));
    mem_label.set_xalign(0.0);
    let mem_bar = ProgressBar::new();
    mem_bar.set_show_text(true);

    let mem_details = Label::new(Some(""));
    mem_details.set_xalign(0.0);
    mem_details.add_css_class("dim-label");

    mem_box.append(&mem_label);
    mem_box.append(&mem_bar);
    mem_box.append(&mem_details);
    mem_frame.set_child(Some(&mem_box));

    // Disk Section
    let disk_frame = Frame::new(Some("Disk"));
    let disk_box = Box::new(Orientation::Vertical, 12);
    disk_box.set_margin_all(12);

    let disk_label = Label::new(Some("Disk Usage"));
    disk_label.set_xalign(0.0);
    let disk_bar = ProgressBar::new();
    disk_bar.set_show_text(true);
    disk_bar.set_fraction(0.45);
    disk_bar.set_text(Some("45% used"));

    disk_box.append(&disk_label);
    disk_box.append(&disk_bar);
    disk_frame.set_child(Some(&disk_box));

    main_box.append(&cpu_frame);
    main_box.append(&mem_frame);
    main_box.append(&disk_frame);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("System Monitor")
        .default_width(500)
        .default_height(500)
        .build();

    window.set_titlebar(Some(&header));
    window.set_child(Some(&main_box));

    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    // Update system info periodically
    let sys = Rc::new(RefCell::new(System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
    )));

    let cpu_bar_clone = cpu_bar.clone();
    let mem_bar_clone = mem_bar.clone();
    let mem_details_clone = mem_details.clone();
    let sys_clone = sys.clone();

    glib::timeout_add_seconds_local(1, move || {
        let mut sys = sys_clone.borrow_mut();
        sys.refresh_cpu();
        sys.refresh_memory();

        let cpu_usage = sys.global_cpu_info().cpu_usage() / 100.0;
        cpu_bar_clone.set_fraction(cpu_usage as f64);
        cpu_bar_clone.set_text(Some(&format!("{:.1}%", cpu_usage * 100.0)));

        let used_mem = sys.used_memory();
        let total_mem = sys.total_memory();
        let mem_percent = used_mem as f64 / total_mem as f64;
        mem_bar_clone.set_fraction(mem_percent);
        mem_bar_clone.set_text(Some(&format!("{:.1}%", mem_percent * 100.0)));

        let used_gb = used_mem as f64 / 1_073_741_824.0;
        let total_gb = total_mem as f64 / 1_073_741_824.0;
        mem_details_clone.set_text(&format!("{:.1} GB / {:.1} GB", used_gb, total_gb));

        glib::ControlFlow::Continue
    });

    window.present();
}
