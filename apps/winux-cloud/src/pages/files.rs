//! Files page - Cloud files browser

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, SearchEntry, DropDown, StringList};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage};

/// Cloud files browser page
pub struct FilesPage {
    widget: gtk4::ScrolledWindow,
}

impl FilesPage {
    pub fn new() -> Self {
        let content = Box::new(Orientation::Vertical, 0);

        // Toolbar with search and filters
        let toolbar = Box::new(Orientation::Horizontal, 8);
        toolbar.set_margin_start(16);
        toolbar.set_margin_end(16);
        toolbar.set_margin_top(8);
        toolbar.set_margin_bottom(8);

        let search = SearchEntry::builder()
            .placeholder_text("Buscar arquivos...")
            .hexpand(true)
            .build();
        toolbar.append(&search);

        // Provider filter
        let providers = StringList::new(&["Todas as Contas", "Google Drive", "OneDrive", "Dropbox"]);
        let provider_dropdown = DropDown::builder()
            .model(&providers)
            .build();
        toolbar.append(&provider_dropdown);

        // View mode buttons
        let list_btn = Button::from_icon_name("view-list-symbolic");
        list_btn.add_css_class("flat");
        list_btn.set_tooltip_text(Some("Visualizacao em lista"));
        toolbar.append(&list_btn);

        let grid_btn = Button::from_icon_name("view-grid-symbolic");
        grid_btn.add_css_class("flat");
        grid_btn.set_tooltip_text(Some("Visualizacao em grade"));
        toolbar.append(&grid_btn);

        content.append(&toolbar);

        // Breadcrumb navigation
        let breadcrumb = Box::new(Orientation::Horizontal, 4);
        breadcrumb.set_margin_start(16);
        breadcrumb.set_margin_end(16);

        let home_btn = Button::with_label("Home");
        home_btn.add_css_class("flat");
        breadcrumb.append(&home_btn);

        let separator1 = gtk4::Label::new(Some("/"));
        separator1.add_css_class("dim-label");
        breadcrumb.append(&separator1);

        let docs_btn = Button::with_label("Documentos");
        docs_btn.add_css_class("flat");
        breadcrumb.append(&docs_btn);

        let separator2 = gtk4::Label::new(Some("/"));
        separator2.add_css_class("dim-label");
        breadcrumb.append(&separator2);

        let project_btn = Button::with_label("Projeto 2026");
        project_btn.add_css_class("flat");
        breadcrumb.append(&project_btn);

        content.append(&breadcrumb);

        let page = PreferencesPage::new();

        // Quick Access Group
        let quick_group = PreferencesGroup::builder()
            .title("Acesso Rapido")
            .build();

        let quick_items = [
            ("Recentes", "document-open-recent-symbolic", "15 arquivos"),
            ("Favoritos", "starred-symbolic", "8 arquivos"),
            ("Compartilhados Comigo", "emblem-shared-symbolic", "23 arquivos"),
            ("Lixeira", "user-trash-symbolic", "5 arquivos"),
        ];

        for (name, icon, count) in quick_items {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(count)
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));
            row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            quick_group.add(&row);
        }

        page.add(&quick_group);

        // Folders Group
        let folders_group = PreferencesGroup::builder()
            .title("Pastas")
            .build();

        let folders = [
            ("Documentos", "folder-documents-symbolic", "45 itens", "Google Drive"),
            ("Imagens", "folder-pictures-symbolic", "234 itens", "OneDrive"),
            ("Projetos", "folder-symbolic", "12 itens", "Dropbox"),
            ("Backups", "folder-symbolic", "8 itens", "Google Drive"),
            ("Trabalho", "folder-symbolic", "67 itens", "OneDrive"),
        ];

        for (name, icon, count, provider) in folders {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} - {}", count, provider))
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            // Sync status
            let sync_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            sync_icon.set_tooltip_text(Some("Sincronizado"));
            row.add_suffix(&sync_icon);

            row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            folders_group.add(&row);
        }

        page.add(&folders_group);

        // Files Group
        let files_group = PreferencesGroup::builder()
            .title("Arquivos")
            .build();

        let files = [
            ("Relatorio_Final.pdf", "application-pdf-symbolic", "2.3 MB", "Hoje, 14:30", true),
            ("Planilha_Orcamento.xlsx", "x-office-spreadsheet-symbolic", "156 KB", "Hoje, 10:15", true),
            ("Apresentacao.pptx", "x-office-presentation-symbolic", "8.7 MB", "Ontem", false),
            ("Contrato_2026.docx", "x-office-document-symbolic", "45 KB", "15/02/2026", true),
            ("foto_equipe.jpg", "image-x-generic-symbolic", "1.8 MB", "14/02/2026", true),
        ];

        for (name, icon, size, modified, synced) in files {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} - Modificado: {}", size, modified))
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            // Sync status
            let status_icon = if synced {
                gtk4::Image::from_icon_name("emblem-ok-symbolic")
            } else {
                gtk4::Image::from_icon_name("emblem-synchronizing-symbolic")
            };
            row.add_suffix(&status_icon);

            // Share button
            let share_btn = Button::from_icon_name("emblem-shared-symbolic");
            share_btn.add_css_class("flat");
            share_btn.set_valign(gtk4::Align::Center);
            share_btn.set_tooltip_text(Some("Compartilhar"));
            row.add_suffix(&share_btn);

            // More options
            let more_btn = Button::from_icon_name("view-more-symbolic");
            more_btn.add_css_class("flat");
            more_btn.set_valign(gtk4::Align::Center);
            more_btn.set_tooltip_text(Some("Mais opcoes"));
            row.add_suffix(&more_btn);

            files_group.add(&row);
        }

        page.add(&files_group);

        // Shared Links Group
        let shared_group = PreferencesGroup::builder()
            .title("Links Compartilhados")
            .description("Arquivos que voce compartilhou")
            .build();

        let shared = [
            ("Proposta_Comercial.pdf", "3 visualizacoes", "Expira em 7 dias"),
            ("Fotos_Evento.zip", "12 downloads", "Link permanente"),
        ];

        for (name, views, expiry) in shared {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} - {}", views, expiry))
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name("emblem-shared-symbolic"));

            let copy_btn = Button::from_icon_name("edit-copy-symbolic");
            copy_btn.add_css_class("flat");
            copy_btn.set_valign(gtk4::Align::Center);
            copy_btn.set_tooltip_text(Some("Copiar link"));
            row.add_suffix(&copy_btn);

            let revoke_btn = Button::from_icon_name("window-close-symbolic");
            revoke_btn.add_css_class("flat");
            revoke_btn.set_valign(gtk4::Align::Center);
            revoke_btn.set_tooltip_text(Some("Revogar acesso"));
            row.add_suffix(&revoke_btn);

            shared_group.add(&row);
        }

        page.add(&shared_group);

        content.append(&page);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&content)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}

impl Default for FilesPage {
    fn default() -> Self {
        Self::new()
    }
}
