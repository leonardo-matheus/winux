// Project selection and management page

use gtk4::prelude::*;
use gtk4::{Box, Button, FileChooserAction, FileChooserNative, Label, ListBox, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;

use crate::window::AppState;
use crate::projects::{detect_project_type, scan_for_projects, ProjectInfo, ProjectType};

#[derive(Clone)]
pub struct ProjectPage {
    widget: Box,
    state: Rc<RefCell<AppState>>,
    project_list: ListBox,
    info_group: PreferencesGroup,
    current_project_label: Label,
}

impl ProjectPage {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let widget = Box::new(Orientation::Vertical, 0);

        let page = PreferencesPage::new();

        // Header with current project info
        let status = StatusPage::builder()
            .icon_name("folder-symbolic")
            .title("Selecione um Projeto")
            .description("Escolha um projeto para construir")
            .build();

        // Current project info label
        let current_project_label = Label::new(Some("Nenhum projeto selecionado"));
        current_project_label.add_css_class("dim-label");

        // Project selection group
        let select_group = PreferencesGroup::builder()
            .title("Abrir Projeto")
            .description("Selecione a pasta do seu projeto")
            .build();

        // Open folder button
        let open_row = ActionRow::builder()
            .title("Abrir Pasta...")
            .subtitle("Selecione a pasta raiz do projeto")
            .activatable(true)
            .build();

        let open_icon = gtk4::Image::from_icon_name("folder-open-symbolic");
        open_row.add_prefix(&open_icon);
        open_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

        select_group.add(&open_row);
        page.add(&select_group);

        // Project info group (hidden initially)
        let info_group = PreferencesGroup::builder()
            .title("Projeto Atual")
            .build();
        info_group.set_visible(false);
        page.add(&info_group);

        // Recent projects group
        let recent_group = PreferencesGroup::builder()
            .title("Projetos Recentes")
            .description("Projetos abertos anteriormente")
            .build();

        let project_list = ListBox::new();
        project_list.add_css_class("boxed-list");
        project_list.set_selection_mode(gtk4::SelectionMode::None);

        // Add placeholder for recent projects
        let placeholder = ActionRow::builder()
            .title("Nenhum projeto recente")
            .subtitle("Abra um projeto para comecar")
            .build();
        placeholder.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));
        project_list.append(&placeholder);

        recent_group.add(&project_list);
        page.add(&recent_group);

        // Scan directory group
        let scan_group = PreferencesGroup::builder()
            .title("Escanear Diretorio")
            .description("Buscar projetos em uma pasta")
            .build();

        let scan_row = ActionRow::builder()
            .title("Escanear Pasta...")
            .subtitle("Encontra todos os projetos em um diretorio")
            .activatable(true)
            .build();

        scan_row.add_prefix(&gtk4::Image::from_icon_name("folder-saved-search-symbolic"));
        scan_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

        scan_group.add(&scan_row);
        page.add(&scan_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&page)
            .build();

        widget.append(&status);
        widget.append(&scrolled);

        let project_page = Self {
            widget,
            state,
            project_list,
            info_group,
            current_project_label,
        };

        // Connect open folder action
        let page_clone = project_page.clone();
        open_row.connect_activated(move |row| {
            page_clone.open_folder_dialog(row);
        });

        // Connect scan action
        let page_clone2 = project_page.clone();
        scan_row.connect_activated(move |row| {
            page_clone2.scan_folder_dialog(row);
        });

        project_page
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    fn open_folder_dialog(&self, row: &ActionRow) {
        let window = row
            .root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok());

        let dialog = FileChooserNative::new(
            Some("Selecionar Projeto"),
            window.as_ref(),
            FileChooserAction::SelectFolder,
            Some("Abrir"),
            Some("Cancelar"),
        );

        let state = self.state.clone();
        let info_group = self.info_group.clone();

        dialog.connect_response(move |dialog, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(folder) = dialog.file() {
                    if let Some(path) = folder.path() {
                        Self::load_project(&state, &info_group, path);
                    }
                }
            }
        });

        dialog.show();
    }

    fn scan_folder_dialog(&self, row: &ActionRow) {
        let window = row
            .root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok());

        let dialog = FileChooserNative::new(
            Some("Escanear Diretorio"),
            window.as_ref(),
            FileChooserAction::SelectFolder,
            Some("Escanear"),
            Some("Cancelar"),
        );

        let project_list = self.project_list.clone();
        let state = self.state.clone();
        let info_group = self.info_group.clone();

        dialog.connect_response(move |dialog, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(folder) = dialog.file() {
                    if let Some(path) = folder.path() {
                        Self::scan_and_show_projects(&project_list, &state, &info_group, path);
                    }
                }
            }
        });

        dialog.show();
    }

    fn load_project(
        state: &Rc<RefCell<AppState>>,
        info_group: &PreferencesGroup,
        path: PathBuf,
    ) {
        match detect_project_type(&path) {
            Ok(project) => {
                Self::update_project_info(info_group, &project);

                // Update state
                state.borrow_mut().current_project = Some(project);
            }
            Err(e) => {
                tracing::error!("Failed to detect project: {}", e);
            }
        }
    }

    fn update_project_info(info_group: &PreferencesGroup, project: &ProjectInfo) {
        // Clear existing children
        while let Some(child) = info_group.first_child() {
            info_group.remove(&child);
        }

        // Project name
        let name_row = ActionRow::builder()
            .title("Nome")
            .subtitle(&project.name)
            .build();
        name_row.add_prefix(&gtk4::Image::from_icon_name("application-x-executable-symbolic"));
        info_group.add(&name_row);

        // Project type
        let type_row = ActionRow::builder()
            .title("Tipo")
            .subtitle(project.project_type.as_str())
            .build();
        type_row.add_prefix(&gtk4::Image::from_icon_name(project.project_type.icon_name()));
        info_group.add(&type_row);

        // Version if available
        if let Some(version) = &project.version {
            let version_row = ActionRow::builder()
                .title("Versao")
                .subtitle(version)
                .build();
            version_row.add_prefix(&gtk4::Image::from_icon_name("document-properties-symbolic"));
            info_group.add(&version_row);
        }

        // Path
        let path_row = ActionRow::builder()
            .title("Caminho")
            .subtitle(&project.path)
            .build();
        path_row.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));
        info_group.add(&path_row);

        // Detected files
        if !project.detected_files.is_empty() {
            let files_str = project.detected_files.join(", ");
            let files_row = ActionRow::builder()
                .title("Arquivos Detectados")
                .subtitle(&files_str)
                .build();
            files_row.add_prefix(&gtk4::Image::from_icon_name("document-symbolic"));
            info_group.add(&files_row);
        }

        // Supported targets
        let targets = project.project_type.supported_targets();
        if !targets.is_empty() {
            let targets_str = targets.join(", ");
            let targets_row = ActionRow::builder()
                .title("Targets Suportados")
                .subtitle(&targets_str)
                .build();
            targets_row.add_prefix(&gtk4::Image::from_icon_name("emblem-system-symbolic"));
            info_group.add(&targets_row);
        }

        info_group.set_visible(true);
    }

    fn scan_and_show_projects(
        project_list: &ListBox,
        state: &Rc<RefCell<AppState>>,
        info_group: &PreferencesGroup,
        path: PathBuf,
    ) {
        // Clear existing items
        while let Some(child) = project_list.first_child() {
            project_list.remove(&child);
        }

        // Scan for projects
        let projects = scan_for_projects(&path, 3);

        if projects.is_empty() {
            let placeholder = ActionRow::builder()
                .title("Nenhum projeto encontrado")
                .subtitle("Tente outro diretorio")
                .build();
            placeholder.add_prefix(&gtk4::Image::from_icon_name("dialog-warning-symbolic"));
            project_list.append(&placeholder);
            return;
        }

        // Add found projects
        for project in projects {
            let row = ActionRow::builder()
                .title(&project.name)
                .subtitle(&format!("{} - {}", project.project_type.as_str(), project.path))
                .activatable(true)
                .build();

            row.add_prefix(&gtk4::Image::from_icon_name(project.project_type.icon_name()));
            row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

            let state_clone = state.clone();
            let info_group_clone = info_group.clone();
            let project_clone = project.clone();

            row.connect_activated(move |_| {
                Self::update_project_info(&info_group_clone, &project_clone);
                state_clone.borrow_mut().current_project = Some(project_clone.clone());
            });

            project_list.append(&row);
        }
    }
}
