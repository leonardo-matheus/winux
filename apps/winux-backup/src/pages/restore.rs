//! Restore page - Browse and restore backups

use gtk4::prelude::*;
use gtk4::{Box, Button, CheckButton, Label, Orientation, ProgressBar, TreeView};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, ComboRow, ExpanderRow, PreferencesGroup, PreferencesPage,
    StatusPage, SwitchRow,
};

/// Restore page
pub struct RestorePage {
    widget: gtk4::ScrolledWindow,
}

impl RestorePage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();

        // Select Backup Group
        let select_group = PreferencesGroup::builder()
            .title("Selecionar Backup")
            .description("Escolha um backup para restaurar")
            .build();

        let source_row = ComboRow::builder()
            .title("Origem")
            .subtitle("Local onde o backup esta armazenado")
            .build();
        let sources = gtk4::StringList::new(&[
            "Local: /media/backup/winux",
            "Remoto: backup.exemplo.com",
            "Restic: repo-principal",
            "Google Drive: Meu Drive",
        ]);
        source_row.set_model(Some(&sources));
        select_group.add(&source_row);

        page.add(&select_group);

        // Available Backups Group
        let backups_group = PreferencesGroup::builder()
            .title("Backups Disponiveis")
            .description("Selecione um ponto de restauracao")
            .build();

        let backups = [
            ("Home Folder - Manual", "19/02/2026 14:30", "2.3 GB", true),
            ("Configuracoes de Apps", "19/02/2026 10:00", "156 MB", true),
            ("Sistema Completo", "18/02/2026 03:00", "42.7 GB", true),
            ("Home Folder - Diario", "18/02/2026 03:00", "2.1 GB", true),
            ("Documentos", "17/02/2026 03:00", "1.8 GB", true),
            ("Sistema Completo", "11/02/2026 03:00", "41.2 GB", true),
        ];

        for (name, date, size, verified) in backups {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} - {}", date, size))
                .activatable(true)
                .build();

            let icon = if verified {
                "emblem-ok-symbolic"
            } else {
                "dialog-question-symbolic"
            };
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            let radio = CheckButton::new();
            row.add_suffix(&radio);

            backups_group.add(&row);
        }

        page.add(&backups_group);

        // Browse Files Group
        let browse_group = PreferencesGroup::builder()
            .title("Navegar Arquivos")
            .description("Explore o conteudo do backup selecionado")
            .build();

        let file_browser = ExpanderRow::builder()
            .title("Conteudo do Backup")
            .subtitle("Clique para expandir e navegar")
            .build();

        // Simulated file tree
        let folders = [
            ("home/", "Pasta pessoal"),
            ("  user/", "Usuario principal"),
            ("    Documentos/", "1.2 GB - 234 arquivos"),
            ("    Imagens/", "3.5 GB - 1,230 arquivos"),
            ("    Downloads/", "890 MB - 156 arquivos"),
            ("    .config/", "156 MB - 89 pastas"),
            ("    Projetos/", "8.2 GB - 12,450 arquivos"),
        ];

        for (path, info) in folders {
            let row = ActionRow::builder()
                .title(path)
                .subtitle(info)
                .build();

            let icon = if path.ends_with('/') {
                "folder-symbolic"
            } else {
                "text-x-generic-symbolic"
            };
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            let check = CheckButton::new();
            row.add_suffix(&check);

            file_browser.add_row(&row);
        }

        browse_group.add(&file_browser);

        // Search in backup
        let search_row = ActionRow::builder()
            .title("Buscar Arquivo")
            .subtitle("Encontre arquivos especificos no backup")
            .activatable(true)
            .build();
        search_row.add_prefix(&gtk4::Image::from_icon_name("system-search-symbolic"));
        search_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        browse_group.add(&search_row);

        page.add(&browse_group);

        // Restore Options Group
        let options_group = PreferencesGroup::builder()
            .title("Opcoes de Restauracao")
            .build();

        let restore_type = ComboRow::builder()
            .title("Tipo de Restauracao")
            .build();
        let types = gtk4::StringList::new(&[
            "Restaurar Arquivos Selecionados",
            "Restaurar Backup Completo",
            "Restaurar Sistema (Requer Reboot)",
        ]);
        restore_type.set_model(Some(&types));
        options_group.add(&restore_type);

        let dest_row = ComboRow::builder()
            .title("Destino")
            .subtitle("Onde restaurar os arquivos")
            .build();
        let destinations = gtk4::StringList::new(&[
            "Local Original",
            "Pasta Personalizada",
            "Desktop (para revisao)",
        ]);
        dest_row.set_model(Some(&destinations));
        options_group.add(&dest_row);

        let overwrite_row = ComboRow::builder()
            .title("Arquivos Existentes")
            .subtitle("O que fazer com arquivos ja existentes")
            .build();
        let overwrite_options = gtk4::StringList::new(&[
            "Perguntar",
            "Sobrescrever",
            "Manter Ambos (renomear)",
            "Pular",
        ]);
        overwrite_row.set_model(Some(&overwrite_options));
        options_group.add(&overwrite_row);

        let preserve_perms = SwitchRow::builder()
            .title("Preservar Permissoes")
            .subtitle("Restaurar permissoes originais dos arquivos")
            .active(true)
            .build();
        options_group.add(&preserve_perms);

        let preserve_owner = SwitchRow::builder()
            .title("Preservar Proprietario")
            .subtitle("Restaurar dono original dos arquivos")
            .active(true)
            .build();
        options_group.add(&preserve_owner);

        let verify_after = SwitchRow::builder()
            .title("Verificar Apos Restaurar")
            .subtitle("Verificar integridade dos arquivos restaurados")
            .active(true)
            .build();
        options_group.add(&verify_after);

        page.add(&options_group);

        // Restore Summary Group
        let summary_group = PreferencesGroup::builder()
            .title("Resumo da Restauracao")
            .build();

        let summary_items = [
            ("Arquivos a restaurar", "1,234"),
            ("Tamanho total", "2.3 GB"),
            ("Tempo estimado", "~5 minutos"),
        ];

        for (label, value) in summary_items {
            let row = ActionRow::builder()
                .title(label)
                .subtitle(value)
                .build();
            summary_group.add(&row);
        }

        page.add(&summary_group);

        // Action Buttons Group
        let action_group = PreferencesGroup::new();

        let action_box = Box::new(Orientation::Horizontal, 12);
        action_box.set_halign(gtk4::Align::Center);
        action_box.set_margin_top(12);
        action_box.set_margin_bottom(12);

        let verify_btn = Button::with_label("Verificar Backup");
        verify_btn.add_css_class("pill");
        action_box.append(&verify_btn);

        let preview_btn = Button::with_label("Pre-visualizar");
        preview_btn.add_css_class("pill");
        action_box.append(&preview_btn);

        let restore_btn = Button::with_label("Restaurar");
        restore_btn.add_css_class("suggested-action");
        restore_btn.add_css_class("pill");
        action_box.append(&restore_btn);

        action_group.add(&action_box);
        page.add(&action_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}

impl Default for RestorePage {
    fn default() -> Self {
        Self::new()
    }
}
