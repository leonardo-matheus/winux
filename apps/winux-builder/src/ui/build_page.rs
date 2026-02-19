// Build configuration and execution page

use gtk4::prelude::*;
use gtk4::{Box, Button, CheckButton, Entry, Label, ListBox, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, ExpanderRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::{AppState, BuildProfile, save_profiles};
use crate::projects::{get_build_command, ProjectType};

type BuildCallback = Box<dyn Fn(&str) + 'static>;

#[derive(Clone)]
pub struct BuildPage {
    widget: Box,
    state: Rc<RefCell<AppState>>,
    build_callback: Rc<RefCell<Option<BuildCallback>>>,
    target_list: ListBox,
    release_switch: SwitchRow,
    profile_combo: ComboRow,
}

impl BuildPage {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let widget = Box::new(Orientation::Vertical, 0);

        let page = PreferencesPage::new();

        // Status header
        let status = StatusPage::builder()
            .icon_name("system-run-symbolic")
            .title("Configurar Build")
            .description("Selecione as opcoes de construcao")
            .build();

        // Build mode group
        let mode_group = PreferencesGroup::builder()
            .title("Modo de Build")
            .build();

        let release_switch = SwitchRow::builder()
            .title("Release Mode")
            .subtitle("Otimizacoes habilitadas, sem debug info")
            .active(true)
            .build();
        mode_group.add(&release_switch);

        page.add(&mode_group);

        // Profile group
        let profile_group = PreferencesGroup::builder()
            .title("Perfil de Build")
            .description("Configuracoes salvas")
            .build();

        let profile_combo = ComboRow::builder()
            .title("Perfil")
            .subtitle("Selecione ou crie um perfil")
            .build();

        // Load profiles into combo
        {
            let profiles = &state.borrow().build_profiles;
            let profile_names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
            let model = gtk4::StringList::new(&profile_names);
            profile_combo.set_model(Some(&model));
        }

        profile_group.add(&profile_combo);

        // Save profile button
        let save_profile_row = ActionRow::builder()
            .title("Salvar Perfil Atual")
            .activatable(true)
            .build();
        save_profile_row.add_prefix(&gtk4::Image::from_icon_name("document-save-symbolic"));
        save_profile_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        profile_group.add(&save_profile_row);

        page.add(&profile_group);

        // Target platforms group
        let platform_group = PreferencesGroup::builder()
            .title("Plataformas Alvo")
            .description("Selecione os formatos de saida")
            .build();

        // Linux targets
        let linux_expander = ExpanderRow::builder()
            .title("Linux")
            .subtitle("DEB, RPM, AppImage, Flatpak")
            .build();
        linux_expander.add_prefix(&gtk4::Image::from_icon_name("computer-symbolic"));

        let linux_targets = [
            ("deb", "Debian/Ubuntu (.deb)"),
            ("rpm", "Fedora/RHEL (.rpm)"),
            ("appimage", "AppImage (universal)"),
            ("flatpak", "Flatpak (sandboxed)"),
        ];

        for (id, label) in linux_targets {
            let row = Self::create_target_row(id, label);
            linux_expander.add_row(&row);
        }
        platform_group.add(&linux_expander);

        // Windows targets
        let windows_expander = ExpanderRow::builder()
            .title("Windows")
            .subtitle("EXE, MSI")
            .build();
        windows_expander.add_prefix(&gtk4::Image::from_icon_name("application-x-ms-dos-executable-symbolic"));

        let windows_targets = [
            ("exe", "Executavel (.exe)"),
            ("msi", "Instalador Windows (.msi)"),
        ];

        for (id, label) in windows_targets {
            let row = Self::create_target_row(id, label);
            windows_expander.add_row(&row);
        }
        platform_group.add(&windows_expander);

        // macOS targets
        let macos_expander = ExpanderRow::builder()
            .title("macOS / iOS")
            .subtitle("APP, DMG, PKG, IPA")
            .build();
        macos_expander.add_prefix(&gtk4::Image::from_icon_name("phone-apple-iphone-symbolic"));

        let macos_targets = [
            ("app", "Application Bundle (.app)"),
            ("dmg", "Disk Image (.dmg)"),
            ("pkg", "Installer Package (.pkg)"),
            ("ipa", "iOS App (.ipa)"),
        ];

        for (id, label) in macos_targets {
            let row = Self::create_target_row(id, label);
            macos_expander.add_row(&row);
        }
        platform_group.add(&macos_expander);

        page.add(&platform_group);

        // Additional options group
        let options_group = PreferencesGroup::builder()
            .title("Opcoes Adicionais")
            .build();

        let extra_args_row = ActionRow::builder()
            .title("Argumentos Extra")
            .build();

        let extra_args_entry = Entry::builder()
            .placeholder_text("--features feature1,feature2")
            .hexpand(true)
            .valign(gtk4::Align::Center)
            .build();
        extra_args_row.add_suffix(&extra_args_entry);
        options_group.add(&extra_args_row);

        let strip_row = SwitchRow::builder()
            .title("Strip Binarios")
            .subtitle("Remove simbolos de debug do executavel")
            .active(true)
            .build();
        options_group.add(&strip_row);

        let upx_row = SwitchRow::builder()
            .title("Comprimir com UPX")
            .subtitle("Reduz tamanho do executavel")
            .active(false)
            .build();
        options_group.add(&upx_row);

        page.add(&options_group);

        // Build actions group
        let actions_group = PreferencesGroup::builder()
            .title("Acoes")
            .build();

        // Build button
        let build_row = ActionRow::builder()
            .title("Iniciar Build")
            .subtitle("Construir para os targets selecionados")
            .activatable(true)
            .build();

        let build_icon = gtk4::Image::from_icon_name("media-playback-start-symbolic");
        build_icon.add_css_class("success");
        build_row.add_prefix(&build_icon);

        let build_btn = Button::with_label("Build");
        build_btn.add_css_class("suggested-action");
        build_btn.set_valign(gtk4::Align::Center);
        build_row.add_suffix(&build_btn);

        actions_group.add(&build_row);

        // Clean button
        let clean_row = ActionRow::builder()
            .title("Limpar Build")
            .subtitle("Remove arquivos de build anteriores")
            .activatable(true)
            .build();
        clean_row.add_prefix(&gtk4::Image::from_icon_name("edit-clear-symbolic"));

        let clean_btn = Button::with_label("Clean");
        clean_btn.add_css_class("destructive-action");
        clean_btn.set_valign(gtk4::Align::Center);
        clean_row.add_suffix(&clean_btn);

        actions_group.add(&clean_row);

        page.add(&actions_group);

        // Target list for tracking selections
        let target_list = ListBox::new();

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&page)
            .build();

        widget.append(&status);
        widget.append(&scrolled);

        let build_page = Self {
            widget,
            state,
            build_callback: Rc::new(RefCell::new(None)),
            target_list,
            release_switch,
            profile_combo,
        };

        // Connect build button
        let page_clone = build_page.clone();
        build_btn.connect_clicked(move |_| {
            page_clone.start_build();
        });

        // Connect clean button
        let page_clone2 = build_page.clone();
        clean_btn.connect_clicked(move |_| {
            page_clone2.clean_build();
        });

        // Connect save profile
        let page_clone3 = build_page.clone();
        save_profile_row.connect_activated(move |_| {
            page_clone3.save_current_profile();
        });

        build_page
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn connect_build_started<F: Fn(&str) + 'static>(&self, callback: F) {
        *self.build_callback.borrow_mut() = Some(Box::new(callback));
    }

    fn create_target_row(id: &str, label: &str) -> ActionRow {
        let row = ActionRow::builder()
            .title(label)
            .activatable(true)
            .build();

        let check = CheckButton::new();
        check.set_widget_name(id);
        row.add_prefix(&check);

        // Toggle check when row is activated
        let check_clone = check.clone();
        row.connect_activated(move |_| {
            check_clone.set_active(!check_clone.is_active());
        });

        row
    }

    fn start_build(&self) {
        let state = self.state.borrow();

        let project = match &state.current_project {
            Some(p) => p,
            None => {
                tracing::warn!("No project selected");
                return;
            }
        };

        let release = self.release_switch.is_active();

        // For now, build native target
        // In a full implementation, we'd iterate over selected targets
        match get_build_command(project, "native", release) {
            Ok(cmd) => {
                tracing::info!("Starting build: {}", cmd);

                if let Some(callback) = self.build_callback.borrow().as_ref() {
                    callback(&cmd);
                }
            }
            Err(e) => {
                tracing::error!("Failed to generate build command: {}", e);
            }
        }
    }

    fn clean_build(&self) {
        let state = self.state.borrow();

        let project = match &state.current_project {
            Some(p) => p,
            None => {
                tracing::warn!("No project selected");
                return;
            }
        };

        let cmd = match project.project_type {
            ProjectType::Rust => format!("cd '{}' && cargo clean", project.path),
            ProjectType::DotNet => format!("cd '{}' && dotnet clean", project.path),
            ProjectType::Electron => format!("cd '{}' && rm -rf node_modules dist build", project.path),
            ProjectType::Flutter => format!("cd '{}' && flutter clean", project.path),
            ProjectType::Unknown => return,
        };

        if let Some(callback) = self.build_callback.borrow().as_ref() {
            callback(&cmd);
        }
    }

    fn save_current_profile(&self) {
        let state = self.state.borrow();

        let project = match &state.current_project {
            Some(p) => p,
            None => return,
        };

        let profile = BuildProfile {
            name: format!("{} Profile", project.name),
            project_type: project.project_type.as_str().to_string(),
            targets: vec!["native".to_string()], // Would collect from UI
            release_mode: self.release_switch.is_active(),
            extra_args: String::new(),
        };

        drop(state);

        let mut state = self.state.borrow_mut();
        state.build_profiles.push(profile);
        save_profiles(&state.build_profiles);
    }
}
