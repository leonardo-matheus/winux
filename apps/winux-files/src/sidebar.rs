//! Sidebar component - Favorites and devices navigation

use std::path::PathBuf;

use gtk4::prelude::*;
use relm4::prelude::*;
use tracing::debug;

/// Sidebar location type
#[derive(Debug, Clone)]
pub enum LocationType {
    Favorite,
    Device,
    Network,
}

/// Sidebar location entry
#[derive(Debug, Clone)]
pub struct SidebarLocation {
    pub name: String,
    pub path: PathBuf,
    pub icon: String,
    pub location_type: LocationType,
}

impl SidebarLocation {
    pub fn favorite(name: &str, path: PathBuf, icon: &str) -> Self {
        Self {
            name: name.to_string(),
            path,
            icon: icon.to_string(),
            location_type: LocationType::Favorite,
        }
    }

    pub fn device(name: &str, path: PathBuf, icon: &str) -> Self {
        Self {
            name: name.to_string(),
            path,
            icon: icon.to_string(),
            location_type: LocationType::Device,
        }
    }
}

/// Sidebar model
pub struct Sidebar {
    favorites: Vec<SidebarLocation>,
    devices: Vec<SidebarLocation>,
    selected_index: Option<usize>,
}

/// Sidebar input messages
#[derive(Debug)]
pub enum SidebarInput {
    SelectLocation(usize),
    AddFavorite(PathBuf),
    RemoveFavorite(usize),
    RefreshDevices,
}

/// Sidebar output messages
#[derive(Debug)]
pub enum SidebarOutput {
    Navigate(PathBuf),
}

#[relm4::component(pub)]
impl Component for Sidebar {
    type Init = ();
    type Input = SidebarInput;
    type Output = SidebarOutput;
    type CommandOutput = ();

    view! {
        gtk4::ScrolledWindow {
            set_hscrollbar_policy: gtk4::PolicyType::Never,
            set_vscrollbar_policy: gtk4::PolicyType::Automatic,
            set_width_request: 180,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,
                add_css_class: "sidebar",

                // Favorites section
                gtk4::Label {
                    set_label: "Favorites",
                    set_halign: gtk4::Align::Start,
                    set_margin_all: 8,
                    add_css_class: "heading",
                },

                #[name = "favorites_list"]
                gtk4::ListBox {
                    set_selection_mode: gtk4::SelectionMode::Single,
                    add_css_class: "navigation-sidebar",

                    connect_row_activated[sender] => move |_, row| {
                        let idx = row.index() as usize;
                        sender.input(SidebarInput::SelectLocation(idx));
                    },
                },

                gtk4::Separator {
                    set_margin_top: 8,
                    set_margin_bottom: 8,
                },

                // Devices section
                gtk4::Label {
                    set_label: "Devices",
                    set_halign: gtk4::Align::Start,
                    set_margin_all: 8,
                    add_css_class: "heading",
                },

                #[name = "devices_list"]
                gtk4::ListBox {
                    set_selection_mode: gtk4::SelectionMode::Single,
                    add_css_class: "navigation-sidebar",

                    connect_row_activated[sender] => move |_, row| {
                        let idx = row.index() as usize;
                        sender.input(SidebarInput::SelectLocation(100 + idx)); // Offset for devices
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Create default favorites
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));

        let favorites = vec![
            SidebarLocation::favorite("Home", home.clone(), "user-home-symbolic"),
            SidebarLocation::favorite(
                "Documents",
                dirs::document_dir().unwrap_or_else(|| home.join("Documents")),
                "folder-documents-symbolic",
            ),
            SidebarLocation::favorite(
                "Downloads",
                dirs::download_dir().unwrap_or_else(|| home.join("Downloads")),
                "folder-download-symbolic",
            ),
            SidebarLocation::favorite(
                "Music",
                dirs::audio_dir().unwrap_or_else(|| home.join("Music")),
                "folder-music-symbolic",
            ),
            SidebarLocation::favorite(
                "Pictures",
                dirs::picture_dir().unwrap_or_else(|| home.join("Pictures")),
                "folder-pictures-symbolic",
            ),
            SidebarLocation::favorite(
                "Videos",
                dirs::video_dir().unwrap_or_else(|| home.join("Videos")),
                "folder-videos-symbolic",
            ),
            SidebarLocation::favorite("Trash", home.join(".local/share/Trash/files"), "user-trash-symbolic"),
        ];

        // Detect devices
        let devices = detect_devices();

        let model = Sidebar {
            favorites,
            devices,
            selected_index: None,
        };

        let widgets = view_output!();

        // Populate favorites list
        for location in &model.favorites {
            let row = create_sidebar_row(location);
            widgets.favorites_list.append(&row);
        }

        // Populate devices list
        for device in &model.devices {
            let row = create_sidebar_row(device);
            widgets.devices_list.append(&row);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            SidebarInput::SelectLocation(idx) => {
                debug!("Selected location: {}", idx);

                let location = if idx >= 100 {
                    // Device
                    self.devices.get(idx - 100)
                } else {
                    // Favorite
                    self.favorites.get(idx)
                };

                if let Some(loc) = location {
                    self.selected_index = Some(idx);
                    let _ = sender.output(SidebarOutput::Navigate(loc.path.clone()));
                }
            }
            SidebarInput::AddFavorite(path) => {
                if let Some(name) = path.file_name() {
                    let location = SidebarLocation::favorite(
                        &name.to_string_lossy(),
                        path,
                        "folder-symbolic",
                    );
                    self.favorites.push(location);
                }
            }
            SidebarInput::RemoveFavorite(idx) => {
                if idx < self.favorites.len() {
                    self.favorites.remove(idx);
                }
            }
            SidebarInput::RefreshDevices => {
                self.devices = detect_devices();
            }
        }
    }
}

fn create_sidebar_row(location: &SidebarLocation) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();

    let box_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    box_widget.set_margin_all(4);

    let icon = gtk4::Image::from_icon_name(&location.icon);
    icon.set_pixel_size(16);
    box_widget.append(&icon);

    let label = gtk4::Label::new(Some(&location.name));
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    box_widget.append(&label);

    row.set_child(Some(&box_widget));
    row
}

fn detect_devices() -> Vec<SidebarLocation> {
    let mut devices = Vec::new();

    // Root filesystem
    devices.push(SidebarLocation::device(
        "Filesystem",
        PathBuf::from("/"),
        "drive-harddisk-symbolic",
    ));

    // Check for mounted drives on Linux
    #[cfg(target_os = "linux")]
    {
        if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
            for line in mounts.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mount_point = parts[1];

                    // Check for USB drives and external media
                    if mount_point.starts_with("/media/") || mount_point.starts_with("/mnt/") {
                        if let Some(name) = PathBuf::from(mount_point).file_name() {
                            devices.push(SidebarLocation::device(
                                &name.to_string_lossy(),
                                PathBuf::from(mount_point),
                                "drive-removable-media-symbolic",
                            ));
                        }
                    }
                }
            }
        }
    }

    // Check for Windows drives
    #[cfg(target_os = "windows")]
    {
        for letter in 'A'..='Z' {
            let path = format!("{}:\\", letter);
            if PathBuf::from(&path).exists() {
                devices.push(SidebarLocation::device(
                    &format!("Drive ({}:)", letter),
                    PathBuf::from(&path),
                    "drive-harddisk-symbolic",
                ));
            }
        }
    }

    devices
}
