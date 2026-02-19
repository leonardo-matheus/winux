// Winux Mobile Studio - Projects Page
// Copyright (c) 2026 Winux OS Project
//
// Manage mobile projects: create, import, and list projects
// Supports Flutter, React Native, Android Native, Swift

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Frame, Grid, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, SearchEntry, Image, FileChooserAction, FileChooserDialog,
    ResponseType,
};
use libadwaita as adw;
use adw::prelude::*;

#[derive(Clone, Debug)]
pub struct MobileProject {
    pub name: String,
    pub path: String,
    pub project_type: ProjectType,
    pub last_modified: String,
    pub platform: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProjectType {
    Flutter,
    ReactNative,
    AndroidNative,
    Swift,
    Kotlin,
}

impl ProjectType {
    pub fn icon_name(&self) -> &str {
        match self {
            ProjectType::Flutter => "applications-science-symbolic",
            ProjectType::ReactNative => "applications-internet-symbolic",
            ProjectType::AndroidNative => "phone-symbolic",
            ProjectType::Swift => "phone-apple-iphone-symbolic",
            ProjectType::Kotlin => "phone-symbolic",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            ProjectType::Flutter => "Flutter",
            ProjectType::ReactNative => "React Native",
            ProjectType::AndroidNative => "Android (Java)",
            ProjectType::Swift => "Swift/iOS",
            ProjectType::Kotlin => "Android (Kotlin)",
        }
    }
}

pub fn create_page() -> Box {
    let page = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Header with actions
    let header = create_header();
    page.append(&header);

    // Main content
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .vexpand(true)
        .build();

    // Projects list
    let projects_panel = create_projects_panel();
    content.append(&projects_panel);

    // Project details
    let details_panel = create_details_panel();
    content.append(&details_panel);

    page.append(&content);
    page
}

fn create_header() -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_top(15)
        .margin_bottom(15)
        .build();

    let title = Label::builder()
        .label("Projetos Mobile")
        .css_classes(vec!["title-2"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // Search
    let search = SearchEntry::builder()
        .placeholder_text("Buscar projetos...")
        .width_request(250)
        .build();
    header.append(&search);

    // New project button
    let new_btn = Button::builder()
        .label("Novo Projeto")
        .css_classes(vec!["suggested-action"])
        .build();
    new_btn.connect_clicked(|_| {
        show_new_project_dialog();
    });
    header.append(&new_btn);

    // Import button
    let import_btn = Button::builder()
        .label("Importar")
        .build();
    import_btn.connect_clicked(|btn| {
        show_import_dialog(btn);
    });
    header.append(&import_btn);

    header
}

fn create_projects_panel() -> Box {
    let panel = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(400)
        .build();

    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .css_classes(vec!["navigation-sidebar"])
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .build();

    // Sample projects
    let projects = get_sample_projects();
    for project in &projects {
        let row = create_project_row(project);
        list.append(&row);
    }

    scrolled.set_child(Some(&list));
    panel.append(&scrolled);

    panel
}

fn create_project_row(project: &MobileProject) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    let icon = Image::from_icon_name(project.project_type.icon_name());
    icon.set_pixel_size(32);
    row_box.append(&icon);

    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let name_label = Label::builder()
        .label(&project.name)
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .build();
    info_box.append(&name_label);

    let type_label = Label::builder()
        .label(project.project_type.display_name())
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    info_box.append(&type_label);

    let path_label = Label::builder()
        .label(&project.path)
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::Middle)
        .build();
    info_box.append(&path_label);

    row_box.append(&info_box);

    let platforms = project.platform.join(", ");
    let platform_label = Label::builder()
        .label(&platforms)
        .css_classes(vec!["caption"])
        .build();
    row_box.append(&platform_label);

    ListBoxRow::builder()
        .child(&row_box)
        .build()
}

fn create_details_panel() -> Box {
    let panel = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    // Project info card
    let info_frame = Frame::builder()
        .css_classes(vec!["card"])
        .build();

    let info_content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    let select_label = Label::builder()
        .label("Selecione um projeto para ver detalhes")
        .css_classes(vec!["dim-label"])
        .valign(gtk4::Align::Center)
        .vexpand(true)
        .build();
    info_content.append(&select_label);

    info_frame.set_child(Some(&info_content));
    panel.append(&info_frame);

    // Quick actions
    let actions_label = Label::builder()
        .label("Acoes Rapidas")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .margin_top(20)
        .build();
    panel.append(&actions_label);

    let actions_grid = Grid::builder()
        .column_spacing(10)
        .row_spacing(10)
        .margin_top(10)
        .build();

    let build_btn = create_action_button("Build Debug", "system-run-symbolic");
    actions_grid.attach(&build_btn, 0, 0, 1, 1);

    let release_btn = create_action_button("Build Release", "emblem-ok-symbolic");
    actions_grid.attach(&release_btn, 1, 0, 1, 1);

    let run_btn = create_action_button("Run no Device", "media-playback-start-symbolic");
    actions_grid.attach(&run_btn, 2, 0, 1, 1);

    let clean_btn = create_action_button("Clean", "edit-clear-symbolic");
    actions_grid.attach(&clean_btn, 0, 1, 1, 1);

    let terminal_btn = create_action_button("Terminal", "utilities-terminal-symbolic");
    actions_grid.attach(&terminal_btn, 1, 1, 1, 1);

    let folder_btn = create_action_button("Abrir Pasta", "folder-open-symbolic");
    actions_grid.attach(&folder_btn, 2, 1, 1, 1);

    panel.append(&actions_grid);

    panel
}

fn create_action_button(label: &str, icon: &str) -> Button {
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let icon_widget = Image::from_icon_name(icon);
    content.append(&icon_widget);

    let label_widget = Label::new(Some(label));
    content.append(&label_widget);

    Button::builder()
        .child(&content)
        .css_classes(vec!["flat"])
        .build()
}

fn show_new_project_dialog() {
    // TODO: Implement new project dialog
    tracing::info!("Opening new project dialog");
}

fn show_import_dialog(btn: &Button) {
    if let Some(window) = btn.root().and_downcast::<gtk4::Window>() {
        let dialog = FileChooserDialog::new(
            Some("Importar Projeto"),
            Some(&window),
            FileChooserAction::SelectFolder,
            &[
                ("Cancelar", ResponseType::Cancel),
                ("Importar", ResponseType::Accept),
            ],
        );

        dialog.connect_response(|dialog, response| {
            if response == ResponseType::Accept {
                if let Some(path) = dialog.file().and_then(|f| f.path()) {
                    tracing::info!("Importing project from: {:?}", path);
                }
            }
            dialog.close();
        });

        dialog.show();
    }
}

fn get_sample_projects() -> Vec<MobileProject> {
    vec![
        MobileProject {
            name: "MyFlutterApp".to_string(),
            path: "~/Projects/my_flutter_app".to_string(),
            project_type: ProjectType::Flutter,
            last_modified: "2026-02-18".to_string(),
            platform: vec!["Android".to_string(), "iOS".to_string()],
        },
        MobileProject {
            name: "ReactNativeDemo".to_string(),
            path: "~/Projects/react_native_demo".to_string(),
            project_type: ProjectType::ReactNative,
            last_modified: "2026-02-17".to_string(),
            platform: vec!["Android".to_string(), "iOS".to_string()],
        },
        MobileProject {
            name: "AndroidApp".to_string(),
            path: "~/Projects/android_app".to_string(),
            project_type: ProjectType::Kotlin,
            last_modified: "2026-02-15".to_string(),
            platform: vec!["Android".to_string()],
        },
        MobileProject {
            name: "SwiftProject".to_string(),
            path: "~/Projects/swift_project".to_string(),
            project_type: ProjectType::Swift,
            last_modified: "2026-02-10".to_string(),
            platform: vec!["iOS".to_string()],
        },
    ]
}
