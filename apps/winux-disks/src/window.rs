// Winux Disks - Main Window
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, Box, ListBox, Orientation, ScrolledWindow, Label, Separator};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, NavigationSplitView, ActionRow, StatusPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::pages::{OverviewPage, DiskDetailPage, PartitionPage, FormatPage};
use crate::backend::{DiskManager, BlockDevice};

/// Main application window for Winux Disks
pub struct DisksWindow {
    window: ApplicationWindow,
}

impl DisksWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Discos")
            .default_width(1100)
            .default_height(700)
            .build();

        // Create main layout
        let split_view = NavigationSplitView::new();

        // Create sidebar
        let sidebar = Self::create_sidebar();
        split_view.set_sidebar(Some(&sidebar));

        // Create main content - start with overview
        let content = Self::create_content();
        split_view.set_content(Some(&content));

        // Create header bar
        let header = HeaderBar::new();
        let title = adw::WindowTitle::new("Discos", "Gerenciamento de Discos");
        header.set_title_widget(Some(&title));

        // Refresh button
        let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_tooltip_text(Some("Atualizar lista de discos"));
        header.pack_start(&refresh_btn);

        // Menu button
        let menu_btn = gtk4::MenuButton::new();
        menu_btn.set_icon_name("open-menu-symbolic");

        let menu = gio::Menu::new();
        menu.append(Some("Benchmark"), Some("app.benchmark"));
        menu.append(Some("Criar Imagem de Disco"), Some("app.create-image"));
        menu.append(Some("Restaurar Imagem"), Some("app.restore-image"));
        menu.append(Some("Sobre"), Some("app.about"));

        let popover = gtk4::PopoverMenu::from_model(Some(&menu));
        menu_btn.set_popover(Some(&popover));
        header.pack_end(&menu_btn);

        // Main box
        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&header);
        main_box.append(&split_view);

        window.set_content(Some(&main_box));

        Self { window }
    }

    fn create_sidebar() -> Box {
        let sidebar_box = Box::new(Orientation::Vertical, 0);

        // Header
        let sidebar_header = Box::new(Orientation::Vertical, 8);
        sidebar_header.set_margin_start(12);
        sidebar_header.set_margin_end(12);
        sidebar_header.set_margin_top(12);
        sidebar_header.set_margin_bottom(12);

        let disks_label = Label::new(Some("Dispositivos"));
        disks_label.add_css_class("title-4");
        disks_label.set_xalign(0.0);
        sidebar_header.append(&disks_label);

        sidebar_box.append(&sidebar_header);

        // Disk list
        let disk_list = ListBox::new();
        disk_list.add_css_class("navigation-sidebar");
        disk_list.set_selection_mode(gtk4::SelectionMode::Single);

        // Load disks from backend
        let disk_manager = DiskManager::new();
        let devices = disk_manager.get_block_devices();

        // Add "Overview" option first
        let overview_row = ActionRow::builder()
            .title("Visao Geral")
            .subtitle("Todos os discos")
            .activatable(true)
            .build();
        overview_row.add_prefix(&gtk4::Image::from_icon_name("drive-multidisk-symbolic"));
        disk_list.append(&overview_row);

        // Separator
        let sep = Separator::new(Orientation::Horizontal);
        sep.set_margin_top(8);
        sep.set_margin_bottom(8);
        disk_list.append(&sep);

        // Add disks
        for device in devices {
            let icon_name = match device.device_type.as_str() {
                "disk" => {
                    if device.is_removable {
                        "drive-removable-media-symbolic"
                    } else if device.model.to_lowercase().contains("nvme") || device.name.starts_with("nvme") {
                        "drive-harddisk-solidstate-symbolic"
                    } else if device.is_rotational {
                        "drive-harddisk-symbolic"
                    } else {
                        "drive-harddisk-solidstate-symbolic"
                    }
                },
                "loop" => "media-optical-symbolic",
                "rom" => "drive-optical-symbolic",
                _ => "drive-harddisk-symbolic",
            };

            let size_str = bytesize::ByteSize::b(device.size).to_string_as(true);
            let subtitle = if device.model.is_empty() {
                format!("{} - {}", device.name, size_str)
            } else {
                format!("{} - {}", device.model, size_str)
            };

            let row = ActionRow::builder()
                .title(&device.name)
                .subtitle(&subtitle)
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon_name));

            // Health indicator
            let health_icon = if device.smart_healthy.unwrap_or(true) {
                let icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
                icon.add_css_class("success");
                icon
            } else {
                let icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
                icon.add_css_class("warning");
                icon
            };
            row.add_suffix(&health_icon);

            disk_list.append(&row);
        }

        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&disk_list)
            .build();

        sidebar_box.append(&scrolled);

        sidebar_box
    }

    fn create_content() -> Box {
        let content_box = Box::new(Orientation::Vertical, 0);

        // Start with overview page
        let overview = OverviewPage::new();
        content_box.append(overview.widget());

        content_box
    }

    pub fn present(&self) {
        self.window.present();
    }
}
