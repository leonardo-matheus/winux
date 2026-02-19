// Main window for Winux Builder

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::{ProjectPage, BuildPage, OutputPage};
use crate::projects::ProjectInfo;

pub struct AppState {
    pub current_project: Option<ProjectInfo>,
    pub build_history: Vec<BuildHistoryEntry>,
    pub build_profiles: Vec<BuildProfile>,
}

#[derive(Clone, Debug)]
pub struct BuildHistoryEntry {
    pub project_name: String,
    pub target: String,
    pub timestamp: String,
    pub status: BuildStatus,
    pub output_path: Option<String>,
}

#[derive(Clone, Debug)]
pub enum BuildStatus {
    Success,
    Failed,
    InProgress,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BuildProfile {
    pub name: String,
    pub project_type: String,
    pub targets: Vec<String>,
    pub release_mode: bool,
    pub extra_args: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_project: None,
            build_history: Vec::new(),
            build_profiles: load_profiles(),
        }
    }
}

fn load_profiles() -> Vec<BuildProfile> {
    let config_dir = dirs::config_dir()
        .map(|p| p.join("winux-builder"))
        .unwrap_or_default();

    let profiles_file = config_dir.join("profiles.json");

    if profiles_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&profiles_file) {
            if let Ok(profiles) = serde_json::from_str(&content) {
                return profiles;
            }
        }
    }

    // Default profiles
    vec![
        BuildProfile {
            name: "Release Linux".to_string(),
            project_type: "rust".to_string(),
            targets: vec!["deb".to_string(), "appimage".to_string()],
            release_mode: true,
            extra_args: String::new(),
        },
        BuildProfile {
            name: "All Platforms".to_string(),
            project_type: "electron".to_string(),
            targets: vec!["exe".to_string(), "dmg".to_string(), "deb".to_string()],
            release_mode: true,
            extra_args: String::new(),
        },
    ]
}

pub fn save_profiles(profiles: &[BuildProfile]) {
    let config_dir = dirs::config_dir()
        .map(|p| p.join("winux-builder"))
        .unwrap_or_default();

    let _ = std::fs::create_dir_all(&config_dir);
    let profiles_file = config_dir.join("profiles.json");

    if let Ok(content) = serde_json::to_string_pretty(profiles) {
        let _ = std::fs::write(profiles_file, content);
    }
}

pub fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(AppState::default()));

    let header = HeaderBar::new();

    let stack = ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    // Create pages
    let project_page = ProjectPage::new(state.clone());
    stack.add_titled(&project_page.widget(), Some("project"), "Projeto")
        .set_icon_name(Some("folder-symbolic"));

    let build_page = BuildPage::new(state.clone());
    stack.add_titled(&build_page.widget(), Some("build"), "Build")
        .set_icon_name(Some("system-run-symbolic"));

    let output_page = OutputPage::new(state.clone());
    stack.add_titled(&output_page.widget(), Some("output"), "Terminal")
        .set_icon_name(Some("utilities-terminal-symbolic"));

    // Connect pages for communication
    {
        let output_page_clone = output_page.clone();
        build_page.connect_build_started(move |cmd| {
            output_page_clone.run_command(&cmd);
        });
    }

    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    // Menu button
    let menu_button = gtk4::MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");

    let menu = gio::Menu::new();
    menu.append(Some("Sobre"), Some("app.about"));
    menu.append(Some("Atalhos"), Some("app.shortcuts"));
    menu.append(Some("Sair"), Some("app.quit"));

    menu_button.set_menu_model(Some(&menu));
    header.pack_end(&menu_button);

    // Main layout
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&header);
    main_box.append(&stack);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Builder")
        .default_width(1200)
        .default_height(800)
        .content(&main_box)
        .build();

    // Setup actions
    setup_actions(app, &window);

    window.present();
}

fn setup_actions(app: &Application, window: &ApplicationWindow) {
    use gio::SimpleAction;

    // About action
    let about_action = SimpleAction::new("about", None);
    let window_clone = window.clone();
    about_action.connect_activate(move |_, _| {
        let about = adw::AboutWindow::builder()
            .transient_for(&window_clone)
            .application_name("Winux Builder")
            .application_icon("system-run-symbolic")
            .developer_name("Winux Team")
            .version("1.0.0")
            .website("https://winux.org")
            .issue_url("https://github.com/winux-os/winux/issues")
            .license_type(gtk4::License::MitX11)
            .developers(vec!["Winux Team".to_string()])
            .copyright("2026 Winux OS Project")
            .build();
        about.present();
    });
    app.add_action(&about_action);

    // Quit action
    let quit_action = SimpleAction::new("quit", None);
    let app_clone = app.clone();
    quit_action.connect_activate(move |_, _| {
        app_clone.quit();
    });
    app.add_action(&quit_action);

    // Shortcuts action
    let shortcuts_action = SimpleAction::new("shortcuts", None);
    shortcuts_action.connect_activate(move |_, _| {
        // Show shortcuts window
    });
    app.add_action(&shortcuts_action);
}
