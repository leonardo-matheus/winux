// Install fonts page - drag & drop, install/uninstall fonts

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Label, ListBox, Orientation, ScrolledWindow,
    SelectionMode, Separator, DropTarget, DragSource, Frame,
    FileFilter, FileDialog,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow};
use gdk4::ContentFormats;
use gio::Cancellable;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;

use crate::fonts::FontManager;

pub fn create_page(font_manager: Rc<RefCell<FontManager>>) -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Drop zone for installing fonts
    let drop_zone = create_drop_zone(font_manager.clone());
    page.add(&drop_zone);

    // Installation options
    let options_group = PreferencesGroup::builder()
        .title("Opcoes de Instalacao")
        .build();

    let system_install = SwitchRow::builder()
        .title("Instalar para todos os usuarios")
        .subtitle("Requer permissao de administrador")
        .active(false)
        .build();
    options_group.add(&system_install);

    let auto_enable = SwitchRow::builder()
        .title("Habilitar fonte automaticamente")
        .subtitle("A fonte ficara disponivel imediatamente")
        .active(true)
        .build();
    options_group.add(&auto_enable);

    page.add(&options_group);

    // Recent installations
    let recent_group = PreferencesGroup::builder()
        .title("Instaladas Recentemente")
        .description("Fontes instaladas pelo usuario")
        .build();

    // Sample recently installed fonts
    let recent_fonts = [
        ("Fira Code", "~/.local/share/fonts/FiraCode.ttf", "2.5 MB", true),
        ("JetBrains Mono", "~/.local/share/fonts/JetBrainsMono.ttf", "1.8 MB", true),
        ("Inter", "~/.local/share/fonts/Inter.ttf", "3.2 MB", true),
    ];

    for (name, path, size, installed) in recent_fonts {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(path)
            .build();

        // Font icon
        let icon = gtk4::Image::from_icon_name("font-x-generic-symbolic");
        row.add_prefix(&icon);

        // Size label
        let size_label = Label::new(Some(size));
        size_label.add_css_class("dim-label");
        row.add_suffix(&size_label);

        // Uninstall button
        let uninstall_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(vec!["flat", "circular"])
            .tooltip_text("Desinstalar fonte")
            .valign(gtk4::Align::Center)
            .build();

        {
            let fm = font_manager.clone();
            let font_name = name.to_string();
            uninstall_btn.connect_clicked(move |btn| {
                // Show confirmation dialog in real app
                // For now, just show feedback
                btn.set_sensitive(false);
                btn.set_icon_name("emblem-ok-symbolic");
            });
        }

        row.add_suffix(&uninstall_btn);
        recent_group.add(&row);
    }

    page.add(&recent_group);

    // System fonts info
    let system_group = PreferencesGroup::builder()
        .title("Fontes do Sistema")
        .description("Fontes instaladas pelo sistema")
        .build();

    let system_info = ActionRow::builder()
        .title("Localizacao das Fontes")
        .subtitle("/usr/share/fonts, ~/.local/share/fonts")
        .build();
    system_info.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));
    system_group.add(&system_info);

    let refresh_row = ActionRow::builder()
        .title("Atualizar Cache de Fontes")
        .subtitle("Executar fc-cache para atualizar")
        .activatable(true)
        .build();
    refresh_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));

    let refresh_btn = Button::builder()
        .label("Atualizar")
        .valign(gtk4::Align::Center)
        .build();

    {
        let fm = font_manager.clone();
        refresh_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            btn.set_label("Atualizando...");

            // In real app, run fc-cache -fv
            let btn_clone = btn.clone();
            glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
                btn_clone.set_sensitive(true);
                btn_clone.set_label("Atualizado!");

                let btn_final = btn_clone.clone();
                glib::timeout_add_local_once(std::time::Duration::from_secs(1), move || {
                    btn_final.set_label("Atualizar");
                });
            });
        });
    }

    refresh_row.add_suffix(&refresh_btn);
    system_group.add(&refresh_row);

    page.add(&system_group);

    // Future: Google Fonts section
    let google_group = PreferencesGroup::builder()
        .title("Google Fonts")
        .description("Em breve - download direto do Google Fonts")
        .build();

    let coming_soon = StatusPage::builder()
        .icon_name("cloud-download-alt-symbolic")
        .title("Em Breve")
        .description("Integracao com Google Fonts para download e instalacao de fontes gratuitas")
        .build();
    coming_soon.add_css_class("compact");
    google_group.add(&coming_soon);

    page.add(&google_group);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_drop_zone(font_manager: Rc<RefCell<FontManager>>) -> PreferencesGroup {
    let group = PreferencesGroup::builder()
        .title("Instalar Nova Fonte")
        .description("Arraste arquivos .ttf ou .otf aqui")
        .build();

    let drop_frame = Frame::new(None);
    drop_frame.set_margin_top(8);
    drop_frame.set_margin_bottom(8);

    let drop_box = Box::new(Orientation::Vertical, 12);
    drop_box.set_halign(gtk4::Align::Center);
    drop_box.set_valign(gtk4::Align::Center);
    drop_box.set_margin_top(48);
    drop_box.set_margin_bottom(48);
    drop_box.set_margin_start(24);
    drop_box.set_margin_end(24);

    let icon = gtk4::Image::builder()
        .icon_name("document-save-symbolic")
        .pixel_size(64)
        .css_classes(vec!["dim-label"])
        .build();
    drop_box.append(&icon);

    let label = Label::builder()
        .label("Arraste fontes aqui")
        .css_classes(vec!["title-3", "dim-label"])
        .build();
    drop_box.append(&label);

    let sublabel = Label::builder()
        .label("ou clique para selecionar arquivos")
        .css_classes(vec!["dim-label"])
        .build();
    drop_box.append(&sublabel);

    // Browse button
    let browse_btn = Button::builder()
        .label("Procurar Arquivos")
        .css_classes(vec!["pill", "suggested-action"])
        .margin_top(12)
        .build();

    {
        let fm = font_manager.clone();
        browse_btn.connect_clicked(move |btn| {
            let dialog = FileDialog::builder()
                .title("Selecionar Fontes")
                .modal(true)
                .build();

            // Set up file filter
            let filter = FileFilter::new();
            filter.add_pattern("*.ttf");
            filter.add_pattern("*.otf");
            filter.add_pattern("*.woff");
            filter.add_pattern("*.woff2");
            filter.set_name(Some("Arquivos de Fonte"));

            let filters = gio::ListStore::new::<FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            let fm_clone = fm.clone();
            dialog.open_multiple(
                gtk4::Window::NONE,
                Cancellable::NONE,
                move |result| {
                    if let Ok(files) = result {
                        for i in 0..files.n_items() {
                            if let Some(file) = files.item(i) {
                                if let Some(gio_file) = file.downcast_ref::<gio::File>() {
                                    if let Some(path) = gio_file.path() {
                                        // Install the font
                                        if let Err(e) = fm_clone.borrow_mut().install_font(&path) {
                                            eprintln!("Failed to install font: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            );
        });
    }

    drop_box.append(&browse_btn);
    drop_frame.set_child(Some(&drop_box));

    // Set up drop target
    let drop_target = DropTarget::new(gio::File::static_type(), gdk4::DragAction::COPY);

    {
        let fm = font_manager.clone();
        let icon_clone = icon.clone();
        let label_clone = label.clone();

        drop_target.connect_drop(move |_target, value, _x, _y| {
            if let Ok(file) = value.get::<gio::File>() {
                if let Some(path) = file.path() {
                    // Check file extension
                    let ext = path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if ["ttf", "otf", "woff", "woff2"].contains(&ext.as_str()) {
                        // Install the font
                        match fm.borrow_mut().install_font(&path) {
                            Ok(_) => {
                                icon_clone.set_icon_name(Some("emblem-ok-symbolic"));
                                icon_clone.remove_css_class("dim-label");
                                icon_clone.add_css_class("success");
                                label_clone.set_label("Fonte instalada!");
                                return true;
                            }
                            Err(e) => {
                                eprintln!("Failed to install font: {}", e);
                                icon_clone.set_icon_name(Some("dialog-error-symbolic"));
                                label_clone.set_label("Erro na instalacao");
                            }
                        }
                    }
                }
            }
            false
        });
    }

    drop_frame.add_controller(drop_target);
    group.add(&drop_frame);

    // Supported formats info
    let formats_row = ActionRow::builder()
        .title("Formatos Suportados")
        .subtitle("TrueType (.ttf), OpenType (.otf), WOFF (.woff, .woff2)")
        .build();
    formats_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
    group.add(&formats_row);

    group
}
