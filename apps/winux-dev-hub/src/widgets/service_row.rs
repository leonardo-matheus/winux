// Winux Dev Hub - Service Row Widget
// Copyright (c) 2026 Winux OS Project
//
// Widget for displaying and controlling system services

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Switch};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow};
use std::process::Command;
use std::cell::RefCell;
use std::rc::Rc;

use crate::pages::services::ServiceStatus;

/// Creates a row widget to display and control a systemd service
pub fn create_service_row(service_name: &str, display_name: &str, description: &str) -> ExpanderRow {
    let status = get_service_status(service_name);
    let enabled = is_service_enabled(service_name);

    let row = ExpanderRow::builder()
        .title(display_name)
        .subtitle(description)
        .build();

    // Status icon
    let status_icon = gtk4::Image::from_icon_name(status.icon());
    status_icon.add_css_class(status.css_class());
    row.add_prefix(&status_icon);

    // Toggle switch
    let switch = Switch::new();
    switch.set_active(status == ServiceStatus::Active);
    switch.set_valign(gtk4::Align::Center);

    let service_name_clone = service_name.to_string();
    switch.connect_state_set(move |_, state| {
        let action = if state { "start" } else { "stop" };
        let _ = Command::new("pkexec")
            .args(["systemctl", action, &service_name_clone])
            .spawn();
        glib::Propagation::Proceed
    });

    row.add_suffix(&switch);

    // Status badge
    let status_label = Label::new(Some(status.display_name()));
    status_label.add_css_class("badge");
    status_label.add_css_class(status.css_class());
    row.add_suffix(&status_label);

    // Service details
    let unit_row = ActionRow::builder()
        .title("Unit")
        .subtitle(&format!("{}.service", service_name))
        .build();
    row.add_row(&unit_row);

    // Enable on boot
    let enable_row = ActionRow::builder()
        .title("Iniciar no Boot")
        .subtitle(if enabled { "Habilitado" } else { "Desabilitado" })
        .build();

    let enable_switch = Switch::new();
    enable_switch.set_active(enabled);
    enable_switch.set_valign(gtk4::Align::Center);

    let service_name_clone2 = service_name.to_string();
    enable_switch.connect_state_set(move |_, state| {
        let action = if state { "enable" } else { "disable" };
        let _ = Command::new("pkexec")
            .args(["systemctl", action, &service_name_clone2])
            .spawn();
        glib::Propagation::Proceed
    });

    enable_row.add_suffix(&enable_switch);
    row.add_row(&enable_row);

    // Actions
    let start_row = ActionRow::builder()
        .title("Iniciar")
        .activatable(true)
        .build();
    start_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));

    let service_name_start = service_name.to_string();
    start_row.connect_activated(move |_| {
        let _ = Command::new("pkexec")
            .args(["systemctl", "start", &service_name_start])
            .spawn();
    });
    row.add_row(&start_row);

    let stop_row = ActionRow::builder()
        .title("Parar")
        .activatable(true)
        .build();
    stop_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-stop-symbolic"));

    let service_name_stop = service_name.to_string();
    stop_row.connect_activated(move |_| {
        let _ = Command::new("pkexec")
            .args(["systemctl", "stop", &service_name_stop])
            .spawn();
    });
    row.add_row(&stop_row);

    let restart_row = ActionRow::builder()
        .title("Reiniciar")
        .activatable(true)
        .build();
    restart_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));

    let service_name_restart = service_name.to_string();
    restart_row.connect_activated(move |_| {
        let _ = Command::new("pkexec")
            .args(["systemctl", "restart", &service_name_restart])
            .spawn();
    });
    row.add_row(&restart_row);

    let logs_row = ActionRow::builder()
        .title("Ver Logs")
        .subtitle("Abre journalctl para este servico")
        .activatable(true)
        .build();
    logs_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));

    let service_name_logs = service_name.to_string();
    logs_row.connect_activated(move |_| {
        // Open terminal with journalctl
        let cmd = format!("journalctl -u {} -f", service_name_logs);
        let _ = Command::new("winux-terminal")
            .args(["-e", "bash", "-c", &cmd])
            .spawn()
            .or_else(|_| {
                Command::new("gnome-terminal")
                    .args(["--", "bash", "-c", &cmd])
                    .spawn()
            });
    });
    row.add_row(&logs_row);

    let status_row = ActionRow::builder()
        .title("Status Detalhado")
        .subtitle("systemctl status")
        .activatable(true)
        .build();
    status_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));

    let service_name_status = service_name.to_string();
    status_row.connect_activated(move |_| {
        let cmd = format!("systemctl status {} ; read -p 'Press enter to close'", service_name_status);
        let _ = Command::new("winux-terminal")
            .args(["-e", "bash", "-c", &cmd])
            .spawn()
            .or_else(|_| {
                Command::new("gnome-terminal")
                    .args(["--", "bash", "-c", &cmd])
                    .spawn()
            });
    });
    row.add_row(&status_row);

    row
}

/// Creates a simple row for quick toggle
pub fn create_simple_service_row(service_name: &str, display_name: &str) -> ActionRow {
    let status = get_service_status(service_name);

    let row = ActionRow::builder()
        .title(display_name)
        .subtitle(status.display_name())
        .build();

    // Status icon
    let status_icon = gtk4::Image::from_icon_name(status.icon());
    status_icon.add_css_class(status.css_class());
    row.add_prefix(&status_icon);

    // Toggle switch
    let switch = Switch::new();
    switch.set_active(status == ServiceStatus::Active);
    switch.set_valign(gtk4::Align::Center);

    let service_name_clone = service_name.to_string();
    switch.connect_state_set(move |_, state| {
        let action = if state { "start" } else { "stop" };
        let _ = Command::new("pkexec")
            .args(["systemctl", action, &service_name_clone])
            .spawn();
        glib::Propagation::Proceed
    });

    row.add_suffix(&switch);

    row
}

/// Get the current status of a systemd service
fn get_service_status(service_name: &str) -> ServiceStatus {
    let output = Command::new("systemctl")
        .args(["is-active", service_name])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let status = stdout.trim();
            match status {
                "active" => ServiceStatus::Active,
                "inactive" => ServiceStatus::Inactive,
                "failed" => ServiceStatus::Failed,
                _ => ServiceStatus::Unknown,
            }
        }
        Err(_) => ServiceStatus::Unknown,
    }
}

/// Check if a service is enabled to start on boot
fn is_service_enabled(service_name: &str) -> bool {
    Command::new("systemctl")
        .args(["is-enabled", service_name])
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.trim() == "enabled"
        })
        .unwrap_or(false)
}
