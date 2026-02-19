//! Updates page - List and install available updates

use gtk4::prelude::*;
use gtk4::{Box, Button, CheckButton, Label, Orientation, ProgressBar, Spinner, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage, ExpanderRow, Clamp};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{UpdateManager, PackageUpdate, UpdateSource};
use crate::ui::{UpdateRow, ProgressWidget};

pub struct UpdatesPage {
    widget: Box,
    content_stack: gtk4::Stack,
    update_manager: Rc<RefCell<UpdateManager>>,
    selected_updates: Rc<RefCell<Vec<String>>>,
}

impl UpdatesPage {
    pub fn new(update_manager: Rc<RefCell<UpdateManager>>) -> Self {
        let widget = Box::new(Orientation::Vertical, 0);
        let selected_updates = Rc::new(RefCell::new(Vec::new()));

        // Stack for different states (loading, empty, list)
        let content_stack = gtk4::Stack::new();
        content_stack.set_vexpand(true);

        // Loading state
        let loading_page = Self::create_loading_state();
        content_stack.add_named(&loading_page, Some("loading"));

        // Empty state (no updates)
        let empty_page = Self::create_empty_state();
        content_stack.add_named(&empty_page, Some("empty"));

        // Updates list state
        let updates_content = Self::create_updates_list(
            update_manager.clone(),
            selected_updates.clone(),
        );
        content_stack.add_named(&updates_content, Some("updates"));

        // Progress state (installing)
        let progress_page = Self::create_progress_state();
        content_stack.add_named(&progress_page, Some("progress"));

        // Default to updates list for demo
        content_stack.set_visible_child_name("updates");

        widget.append(&content_stack);

        // Bottom action bar
        let action_bar = Self::create_action_bar(update_manager.clone(), selected_updates.clone());
        widget.append(&action_bar);

        Self {
            widget,
            content_stack,
            update_manager,
            selected_updates,
        }
    }

    fn create_loading_state() -> StatusPage {
        let spinner = Spinner::new();
        spinner.set_size_request(48, 48);
        spinner.start();

        StatusPage::builder()
            .icon_name("emblem-synchronizing-symbolic")
            .title("Verificando Atualizacoes")
            .description("Por favor, aguarde...")
            .child(&spinner)
            .build()
    }

    fn create_empty_state() -> StatusPage {
        StatusPage::builder()
            .icon_name("emblem-ok-symbolic")
            .title("Sistema Atualizado")
            .description("Nao ha atualizacoes disponiveis no momento")
            .build()
    }

    fn create_updates_list(
        update_manager: Rc<RefCell<UpdateManager>>,
        selected_updates: Rc<RefCell<Vec<String>>>,
    ) -> ScrolledWindow {
        let page = PreferencesPage::new();

        // Summary section
        let summary_group = PreferencesGroup::builder()
            .title("Resumo")
            .build();

        let summary_row = ActionRow::builder()
            .title("12 atualizacoes disponiveis")
            .subtitle("Total: 156.4 MB para download")
            .build();
        summary_row.add_prefix(&gtk4::Image::from_icon_name("software-update-available-symbolic"));

        let select_all_check = CheckButton::new();
        select_all_check.set_active(true);
        select_all_check.set_tooltip_text(Some("Selecionar todas"));
        select_all_check.set_valign(gtk4::Align::Center);
        summary_row.add_suffix(&select_all_check);

        summary_group.add(&summary_row);
        page.add(&summary_group);

        // System updates (APT)
        let apt_group = PreferencesGroup::builder()
            .title("Sistema (APT)")
            .description("Pacotes do sistema e seguranca")
            .build();

        let apt_updates = vec![
            ("linux-image-6.8.0-45", "6.8.0-44", "6.8.0-45", "78.5 MB", "Kernel Linux com correcoes de seguranca"),
            ("mesa-vulkan-drivers", "24.0.5-1", "24.0.8-1", "15.2 MB", "Drivers Vulkan Mesa atualizados"),
            ("firefox", "130.0", "131.0", "52.3 MB", "Navegador Firefox com melhorias de desempenho"),
            ("libc6", "2.38-10", "2.38-12", "4.8 MB", "Biblioteca C padrao - correcao de seguranca"),
            ("openssl", "3.0.13-1", "3.0.14-1", "5.7 MB", "Correcao de vulnerabilidade critica"),
        ];

        let apt_expander = ExpanderRow::builder()
            .title("5 atualizacoes do sistema")
            .subtitle("156.5 MB")
            .build();
        apt_expander.add_prefix(&gtk4::Image::from_icon_name("package-x-generic-symbolic"));

        for (name, current, new, size, desc) in apt_updates {
            let row = Self::create_update_row(name, current, new, size, desc, UpdateSource::Apt);
            apt_expander.add_row(&row);
        }

        apt_group.add(&apt_expander);
        page.add(&apt_group);

        // Flatpak updates
        let flatpak_group = PreferencesGroup::builder()
            .title("Flatpak")
            .description("Aplicativos Flatpak")
            .build();

        let flatpak_updates = vec![
            ("com.spotify.Client", "1.2.25", "1.2.26", "45.0 MB", "Spotify - cliente de streaming"),
            ("org.gimp.GIMP", "2.10.36", "2.10.38", "125.8 MB", "GIMP - editor de imagens"),
            ("com.discordapp.Discord", "0.0.40", "0.0.42", "89.2 MB", "Discord - comunicacao"),
        ];

        let flatpak_expander = ExpanderRow::builder()
            .title("3 aplicativos Flatpak")
            .subtitle("260.0 MB")
            .build();
        flatpak_expander.add_prefix(&gtk4::Image::from_icon_name("application-x-executable-symbolic"));

        for (name, current, new, size, desc) in flatpak_updates {
            let row = Self::create_update_row(name, current, new, size, desc, UpdateSource::Flatpak);
            flatpak_expander.add_row(&row);
        }

        flatpak_group.add(&flatpak_expander);
        page.add(&flatpak_group);

        // Snap updates
        let snap_group = PreferencesGroup::builder()
            .title("Snap")
            .description("Pacotes Snap")
            .build();

        let snap_updates = vec![
            ("code", "1.85.1", "1.86.0", "78.5 MB", "Visual Studio Code"),
            ("vlc", "3.0.19", "3.0.20", "45.2 MB", "VLC media player"),
        ];

        let snap_expander = ExpanderRow::builder()
            .title("2 pacotes Snap")
            .subtitle("123.7 MB")
            .build();
        snap_expander.add_prefix(&gtk4::Image::from_icon_name("snap-symbolic"));

        for (name, current, new, size, desc) in snap_updates {
            let row = Self::create_update_row(name, current, new, size, desc, UpdateSource::Snap);
            snap_expander.add_row(&row);
        }

        snap_group.add(&snap_expander);
        page.add(&snap_group);

        // Firmware updates
        let firmware_group = PreferencesGroup::builder()
            .title("Firmware")
            .description("Atualizacoes de firmware via fwupd")
            .build();

        let firmware_expander = ExpanderRow::builder()
            .title("2 atualizacoes de firmware")
            .subtitle("12.4 MB")
            .build();
        firmware_expander.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-solidstate-symbolic"));

        let fw_row1 = Self::create_update_row(
            "UEFI dbx",
            "327",
            "331",
            "2.1 MB",
            "Atualizacao de seguranca UEFI Secure Boot",
            UpdateSource::Fwupd,
        );
        firmware_expander.add_row(&fw_row1);

        let fw_row2 = Self::create_update_row(
            "System Firmware",
            "1.12.1",
            "1.14.0",
            "10.3 MB",
            "BIOS/UEFI firmware update",
            UpdateSource::Fwupd,
        );
        firmware_expander.add_row(&fw_row2);

        firmware_group.add(&firmware_expander);
        page.add(&firmware_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        scrolled
    }

    fn create_update_row(
        name: &str,
        current: &str,
        new: &str,
        size: &str,
        description: &str,
        source: UpdateSource,
    ) -> ActionRow {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(&format!("{} -> {} ({}) - {}", current, new, size, description))
            .build();

        // Checkbox for selection
        let check = CheckButton::new();
        check.set_active(true);
        check.set_valign(gtk4::Align::Center);
        row.add_prefix(&check);

        // Source icon
        let icon_name = match source {
            UpdateSource::Apt => "package-x-generic-symbolic",
            UpdateSource::Flatpak => "application-x-executable-symbolic",
            UpdateSource::Snap => "application-x-addon-symbolic",
            UpdateSource::Fwupd => "drive-harddisk-solidstate-symbolic",
        };

        // Changelog button
        let changelog_btn = Button::builder()
            .icon_name("text-x-generic-symbolic")
            .tooltip_text("Ver changelog")
            .valign(gtk4::Align::Center)
            .build();
        changelog_btn.add_css_class("flat");
        row.add_suffix(&changelog_btn);

        row
    }

    fn create_progress_state() -> Box {
        let content = Box::new(Orientation::Vertical, 24);
        content.set_margin_top(48);
        content.set_margin_bottom(48);
        content.set_margin_start(48);
        content.set_margin_end(48);

        let title = Label::new(Some("Instalando Atualizacoes"));
        title.add_css_class("title-1");
        content.append(&title);

        let subtitle = Label::new(Some("Nao desligue o computador durante a atualizacao"));
        subtitle.add_css_class("dim-label");
        content.append(&subtitle);

        // Overall progress
        let overall_box = Box::new(Orientation::Vertical, 8);
        overall_box.set_margin_top(24);

        let overall_label = Label::new(Some("Progresso geral: 3 de 12 pacotes"));
        overall_label.set_halign(gtk4::Align::Start);
        overall_box.append(&overall_label);

        let overall_progress = ProgressBar::new();
        overall_progress.set_fraction(0.25);
        overall_progress.set_show_text(true);
        overall_progress.set_text(Some("25%"));
        overall_box.append(&overall_progress);

        content.append(&overall_box);

        // Current package
        let current_box = Box::new(Orientation::Vertical, 8);
        current_box.set_margin_top(24);

        let current_label = Label::new(Some("Instalando: firefox 131.0"));
        current_label.set_halign(gtk4::Align::Start);
        current_box.append(&current_label);

        let current_progress = ProgressBar::new();
        current_progress.set_fraction(0.67);
        current_progress.set_show_text(true);
        current_progress.set_text(Some("67% - Baixando..."));
        current_box.append(&current_progress);

        content.append(&current_box);

        // Log output
        let log_frame = gtk4::Frame::new(Some("Log de instalacao"));
        log_frame.set_margin_top(24);

        let log_scroll = ScrolledWindow::builder()
            .min_content_height(150)
            .build();

        let log_text = gtk4::TextView::new();
        log_text.set_editable(false);
        log_text.set_monospace(true);
        log_text.set_wrap_mode(gtk4::WrapMode::Word);

        let buffer = log_text.buffer();
        buffer.set_text(
            "Lendo listas de pacotes...\n\
             Construindo arvore de dependencias...\n\
             Lendo informacoes de estado...\n\
             Os seguintes pacotes serao atualizados:\n\
             firefox mesa-vulkan-drivers libc6\n\
             3 pacotes atualizados, 0 novos, 0 removidos.\n\
             Baixando firefox (52.3 MB)...\n"
        );

        log_scroll.set_child(Some(&log_text));
        log_frame.set_child(Some(&log_scroll));
        content.append(&log_frame);

        // Cancel button
        let cancel_btn = Button::with_label("Cancelar");
        cancel_btn.add_css_class("destructive-action");
        cancel_btn.set_halign(gtk4::Align::Center);
        cancel_btn.set_margin_top(24);
        content.append(&cancel_btn);

        let clamp = Clamp::new();
        clamp.set_maximum_size(800);
        clamp.set_child(Some(&content));

        let outer = Box::new(Orientation::Vertical, 0);
        outer.append(&clamp);
        outer
    }

    fn create_action_bar(
        update_manager: Rc<RefCell<UpdateManager>>,
        selected_updates: Rc<RefCell<Vec<String>>>,
    ) -> Box {
        let action_bar = Box::new(Orientation::Horizontal, 12);
        action_bar.set_margin_top(12);
        action_bar.set_margin_bottom(12);
        action_bar.set_margin_start(12);
        action_bar.set_margin_end(12);
        action_bar.add_css_class("toolbar");

        // Info label
        let info_label = Label::new(Some("12 atualizacoes selecionadas (540.1 MB)"));
        info_label.set_hexpand(true);
        info_label.set_halign(gtk4::Align::Start);
        action_bar.append(&info_label);

        // Download only button
        let download_btn = Button::with_label("Apenas Baixar");
        download_btn.set_tooltip_text(Some("Baixar atualizacoes sem instalar"));
        action_bar.append(&download_btn);

        // Install button
        let install_btn = Button::with_label("Instalar Atualizacoes");
        install_btn.add_css_class("suggested-action");
        install_btn.set_tooltip_text(Some("Instalar todas as atualizacoes selecionadas"));
        action_bar.append(&install_btn);

        install_btn.connect_clicked(move |_| {
            tracing::info!("Starting update installation");
            // Would switch to progress view and start installation
        });

        action_bar
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn refresh(&self) {
        self.content_stack.set_visible_child_name("loading");
        // Would trigger async update check
    }

    pub fn show_updates(&self) {
        self.content_stack.set_visible_child_name("updates");
    }

    pub fn show_empty(&self) {
        self.content_stack.set_visible_child_name("empty");
    }

    pub fn show_progress(&self) {
        self.content_stack.set_visible_child_name("progress");
    }
}
