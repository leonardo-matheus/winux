//! Startup applications management view for Winux Monitor
//!
//! Allows users to view and manage applications that start automatically.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::{gio, glib};
use std::cell::RefCell;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Startup application entry
#[derive(Debug, Clone)]
pub struct StartupApp {
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub enabled: bool,
    pub path: PathBuf,
    pub comment: Option<String>,
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct StartupView {
        pub apps: RefCell<Vec<StartupApp>>,
        pub list_box: OnceCell<gtk4::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StartupView {
        const NAME: &'static str = "WinuxMonitorStartupView";
        type Type = super::StartupView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for StartupView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for StartupView {}
    impl BoxImpl for StartupView {}
}

glib::wrapper! {
    pub struct StartupView(ObjectSubclass<imp::StartupView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl StartupView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(0);

        // Header with description
        let header = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        header.set_margin_start(24);
        header.set_margin_end(24);
        header.set_margin_top(24);
        header.set_margin_bottom(12);

        let title = gtk4::Label::new(Some("Startup Applications"));
        title.add_css_class("title-2");
        title.set_halign(gtk4::Align::Start);
        header.append(&title);

        let description = gtk4::Label::new(Some(
            "These applications will automatically start when you log in. \
             Disabling unnecessary startup applications can improve boot time.",
        ));
        description.add_css_class("dim-label");
        description.set_halign(gtk4::Align::Start);
        description.set_wrap(true);
        description.set_xalign(0.0);
        header.append(&description);

        self.append(&header);

        // Toolbar
        let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        toolbar.set_margin_start(24);
        toolbar.set_margin_end(24);
        toolbar.set_margin_bottom(12);

        // Add button
        let add_button = gtk4::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Add startup application")
            .build();

        let view_weak = self.downgrade();
        add_button.connect_clicked(move |_| {
            if let Some(view) = view_weak.upgrade() {
                view.show_add_dialog();
            }
        });
        toolbar.append(&add_button);

        // Spacer
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&spacer);

        // App count
        let count_label = gtk4::Label::new(Some("0 startup apps"));
        count_label.add_css_class("dim-label");
        toolbar.append(&count_label);

        self.append(&toolbar);

        // Scrolled list
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_margin_start(24);
        scrolled.set_margin_end(24);
        scrolled.set_margin_bottom(24);

        let list_box = gtk4::ListBox::new();
        list_box.add_css_class("boxed-list");
        list_box.set_selection_mode(gtk4::SelectionMode::None);

        imp.list_box.set(list_box.clone()).unwrap();

        scrolled.set_child(Some(&list_box));
        self.append(&scrolled);

        // Load startup apps
        self.load_startup_apps();
    }

    fn load_startup_apps(&self) {
        let imp = self.imp();
        let mut apps = Vec::new();

        // XDG autostart directories
        let autostart_dirs = get_autostart_dirs();

        for dir in autostart_dirs {
            if !dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "desktop") {
                        if let Some(app) = parse_desktop_file(&path) {
                            apps.push(app);
                        }
                    }
                }
            }
        }

        debug!("Found {} startup applications", apps.len());

        // Update UI
        if let Some(list_box) = imp.list_box.get() {
            // Clear existing
            while let Some(child) = list_box.first_child() {
                list_box.remove(&child);
            }

            // Add rows
            for app in &apps {
                let row = self.create_app_row(app);
                list_box.append(&row);
            }

            // Show empty state if no apps
            if apps.is_empty() {
                let empty_row = adw::ActionRow::builder()
                    .title("No startup applications")
                    .subtitle("Click the + button to add an application")
                    .activatable(false)
                    .build();
                empty_row.add_prefix(&gtk4::Image::from_icon_name("info-symbolic"));
                list_box.append(&empty_row);
            }
        }

        imp.apps.replace(apps);
    }

    fn create_app_row(&self, app: &StartupApp) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(&app.name)
            .subtitle(app.comment.as_deref().unwrap_or(&app.exec))
            .activatable(false)
            .build();

        // Icon
        let icon = gtk4::Image::from_icon_name(&app.icon);
        icon.set_pixel_size(32);
        row.add_prefix(&icon);

        // Enable switch
        let switch = gtk4::Switch::new();
        switch.set_active(app.enabled);
        switch.set_valign(gtk4::Align::Center);

        let path = app.path.clone();
        let view_weak = self.downgrade();
        switch.connect_state_set(move |_, state| {
            if let Some(view) = view_weak.upgrade() {
                view.toggle_app(&path, state);
            }
            glib::Propagation::Proceed
        });

        row.add_suffix(&switch);

        // Delete button
        let delete_button = gtk4::Button::builder()
            .icon_name("user-trash-symbolic")
            .valign(gtk4::Align::Center)
            .tooltip_text("Remove from startup")
            .build();
        delete_button.add_css_class("flat");

        let path = app.path.clone();
        let view_weak = self.downgrade();
        delete_button.connect_clicked(move |_| {
            if let Some(view) = view_weak.upgrade() {
                view.remove_app(&path);
            }
        });

        row.add_suffix(&delete_button);

        row
    }

    fn toggle_app(&self, path: &PathBuf, enabled: bool) {
        info!("Toggling startup app {:?} to {}", path, enabled);

        // In production, this would modify the desktop file
        // to add/remove X-GNOME-Autostart-enabled=false
        if let Ok(contents) = std::fs::read_to_string(path) {
            let new_contents = if enabled {
                // Remove disabled line
                contents
                    .lines()
                    .filter(|l| !l.starts_with("X-GNOME-Autostart-enabled="))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                // Add disabled line
                if contents.contains("X-GNOME-Autostart-enabled=") {
                    contents.replace(
                        "X-GNOME-Autostart-enabled=true",
                        "X-GNOME-Autostart-enabled=false",
                    )
                } else {
                    format!("{}\nX-GNOME-Autostart-enabled=false", contents)
                }
            };

            if let Err(e) = std::fs::write(path, new_contents) {
                warn!("Failed to update startup file: {}", e);
            }
        }
    }

    fn remove_app(&self, path: &PathBuf) {
        info!("Removing startup app: {:?}", path);

        let dialog = adw::AlertDialog::builder()
            .heading("Remove Startup Application?")
            .body("This application will no longer start automatically when you log in.")
            .build();

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("remove", "Remove");
        dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);

        let path_clone = path.clone();
        let view_weak = self.downgrade();
        dialog.connect_response(None, move |_, response| {
            if response == "remove" {
                if let Err(e) = std::fs::remove_file(&path_clone) {
                    warn!("Failed to remove startup file: {}", e);
                } else {
                    if let Some(view) = view_weak.upgrade() {
                        view.load_startup_apps();
                    }
                }
            }
        });

        if let Some(root) = self.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                dialog.present(Some(window));
            }
        }
    }

    fn show_add_dialog(&self) {
        let dialog = adw::Dialog::builder()
            .title("Add Startup Application")
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_width_request(400);

        // Name entry
        let name_row = adw::EntryRow::builder()
            .title("Name")
            .build();
        content.append(&name_row);

        // Command entry
        let command_row = adw::EntryRow::builder()
            .title("Command")
            .build();

        let browse_button = gtk4::Button::builder()
            .icon_name("folder-open-symbolic")
            .valign(gtk4::Align::Center)
            .build();
        browse_button.add_css_class("flat");
        command_row.add_suffix(&browse_button);

        content.append(&command_row);

        // Comment entry
        let comment_row = adw::EntryRow::builder()
            .title("Comment (optional)")
            .build();
        content.append(&comment_row);

        // Buttons
        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::End);
        button_box.set_margin_top(12);

        let cancel_button = gtk4::Button::with_label("Cancel");
        let add_button = gtk4::Button::with_label("Add");
        add_button.add_css_class("suggested-action");

        let dialog_weak = dialog.downgrade();
        cancel_button.connect_clicked(move |_| {
            if let Some(d) = dialog_weak.upgrade() {
                d.close();
            }
        });

        let view_weak = self.downgrade();
        let dialog_weak = dialog.downgrade();
        let name_row_clone = name_row.clone();
        let command_row_clone = command_row.clone();
        let comment_row_clone = comment_row.clone();
        add_button.connect_clicked(move |_| {
            let name = name_row_clone.text().to_string();
            let command = command_row_clone.text().to_string();
            let comment = comment_row_clone.text().to_string();

            if !name.is_empty() && !command.is_empty() {
                if let Some(view) = view_weak.upgrade() {
                    view.add_startup_app(&name, &command, if comment.is_empty() { None } else { Some(&comment) });
                }
                if let Some(d) = dialog_weak.upgrade() {
                    d.close();
                }
            }
        });

        button_box.append(&cancel_button);
        button_box.append(&add_button);
        content.append(&button_box);

        dialog.set_child(Some(&content));

        if let Some(root) = self.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                dialog.present(Some(window));
            }
        }
    }

    fn add_startup_app(&self, name: &str, command: &str, comment: Option<&str>) {
        info!("Adding startup app: {}", name);

        // Create desktop file
        let autostart_dir = dirs::config_dir()
            .map(|p| p.join("autostart"))
            .unwrap_or_else(|| PathBuf::from("~/.config/autostart"));

        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(&autostart_dir) {
            warn!("Failed to create autostart directory: {}", e);
            return;
        }

        let filename = format!("{}.desktop", name.to_lowercase().replace(' ', "-"));
        let path = autostart_dir.join(&filename);

        let mut content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={}\n\
             Exec={}\n\
             X-GNOME-Autostart-enabled=true\n",
            name, command
        );

        if let Some(c) = comment {
            content.push_str(&format!("Comment={}\n", c));
        }

        if let Err(e) = std::fs::write(&path, content) {
            warn!("Failed to create startup file: {}", e);
        } else {
            self.load_startup_apps();
        }
    }
}

impl Default for StartupView {
    fn default() -> Self {
        Self::new()
    }
}

/// Get XDG autostart directories
fn get_autostart_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User autostart directory
    if let Some(config_dir) = dirs::config_dir() {
        dirs.push(config_dir.join("autostart"));
    }

    // System autostart directories
    dirs.push(PathBuf::from("/etc/xdg/autostart"));

    for data_dir in &["/usr/share", "/usr/local/share"] {
        dirs.push(PathBuf::from(data_dir).join("gnome/autostart"));
    }

    dirs
}

/// Parse a .desktop file into StartupApp
fn parse_desktop_file(path: &PathBuf) -> Option<StartupApp> {
    let contents = std::fs::read_to_string(path).ok()?;

    let mut name = None;
    let mut exec = None;
    let mut icon = String::from("application-x-executable");
    let mut enabled = true;
    let mut comment = None;
    let mut hidden = false;
    let mut no_display = false;

    for line in contents.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "Name" => name = Some(value.trim().to_string()),
                "Exec" => exec = Some(value.trim().to_string()),
                "Icon" => icon = value.trim().to_string(),
                "Comment" => comment = Some(value.trim().to_string()),
                "X-GNOME-Autostart-enabled" => {
                    enabled = value.trim().to_lowercase() != "false";
                }
                "Hidden" => hidden = value.trim().to_lowercase() == "true",
                "NoDisplay" => no_display = value.trim().to_lowercase() == "true",
                _ => {}
            }
        }
    }

    // Skip hidden entries
    if hidden {
        return None;
    }

    Some(StartupApp {
        name: name?,
        exec: exec?,
        icon,
        enabled,
        path: path.clone(),
        comment,
    })
}
