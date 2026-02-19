//! Create backup page

use gtk4::prelude::*;
use gtk4::{Box, Button, CheckButton, Entry, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, ComboRow, EntryRow, ExpanderRow, PreferencesGroup,
    PreferencesPage, SwitchRow,
};

/// Create backup page
pub struct CreatePage {
    widget: gtk4::ScrolledWindow,
}

impl CreatePage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();

        // Backup Type Group
        let type_group = PreferencesGroup::builder()
            .title("Tipo de Backup")
            .description("Selecione o que deseja fazer backup")
            .build();

        let backup_types = [
            ("Sistema Completo", "Backup de todo o sistema operacional", "computer-symbolic"),
            ("Home Folder", "Backup da pasta pessoal (/home)", "user-home-symbolic"),
            ("Pastas Especificas", "Selecione pastas individuais", "folder-symbolic"),
            ("Configuracoes de Apps", "Arquivos de configuracao (~/.config)", "applications-system-symbolic"),
        ];

        let first_radio: Option<CheckButton> = None;
        for (i, (name, desc, icon)) in backup_types.iter().enumerate() {
            let row = ActionRow::builder()
                .title(*name)
                .subtitle(*desc)
                .activatable(true)
                .build();

            row.add_prefix(&gtk4::Image::from_icon_name(*icon));

            let radio = CheckButton::new();
            if i == 1 {
                radio.set_active(true);
            }
            row.add_suffix(&radio);
            type_group.add(&row);
        }

        page.add(&type_group);

        // Custom Folders Group (expandable)
        let folders_group = PreferencesGroup::builder()
            .title("Pastas Personalizadas")
            .description("Adicione pastas especificas ao backup")
            .build();

        let folders_expander = ExpanderRow::builder()
            .title("Pastas Selecionadas")
            .subtitle("3 pastas selecionadas")
            .build();

        let selected_folders = [
            ("/home/user/Documentos", "1.2 GB"),
            ("/home/user/Imagens", "3.5 GB"),
            ("/home/user/Projetos", "8.2 GB"),
        ];

        for (path, size) in selected_folders {
            let row = ActionRow::builder()
                .title(path)
                .subtitle(size)
                .build();

            let remove_btn = Button::from_icon_name("list-remove-symbolic");
            remove_btn.add_css_class("flat");
            remove_btn.add_css_class("circular");
            remove_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&remove_btn);

            folders_expander.add_row(&row);
        }

        let add_folder_row = ActionRow::builder()
            .title("Adicionar Pasta...")
            .activatable(true)
            .build();
        add_folder_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
        folders_expander.add_row(&add_folder_row);

        folders_group.add(&folders_expander);
        page.add(&folders_group);

        // Exclusions Group
        let exclusions_group = PreferencesGroup::builder()
            .title("Exclusoes")
            .description("Pastas e arquivos a ignorar")
            .build();

        let exclusions_expander = ExpanderRow::builder()
            .title("Padroes de Exclusao")
            .subtitle("Arquivos temporarios, cache, etc.")
            .build();

        let exclusion_patterns = [
            ("*.tmp", "Arquivos temporarios"),
            ("**/cache/**", "Diretorios de cache"),
            ("**/.cache/**", "Cache do usuario"),
            ("**/node_modules/**", "Dependencias Node.js"),
            ("**/__pycache__/**", "Cache Python"),
            ("*.log", "Arquivos de log"),
        ];

        for (pattern, desc) in exclusion_patterns {
            let row = ActionRow::builder()
                .title(pattern)
                .subtitle(desc)
                .build();

            let check = CheckButton::new();
            check.set_active(true);
            row.add_suffix(&check);
            exclusions_expander.add_row(&row);
        }

        let add_exclusion_row = ActionRow::builder()
            .title("Adicionar Padrao...")
            .activatable(true)
            .build();
        add_exclusion_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
        exclusions_expander.add_row(&add_exclusion_row);

        exclusions_group.add(&exclusions_expander);
        page.add(&exclusions_group);

        // Destination Group
        let dest_group = PreferencesGroup::builder()
            .title("Destino do Backup")
            .description("Onde salvar o backup")
            .build();

        let backend_row = ComboRow::builder()
            .title("Backend")
            .subtitle("Metodo de armazenamento")
            .build();
        let backends = gtk4::StringList::new(&[
            "Local (Disco Externo)",
            "Rsync (SSH Remoto)",
            "Restic (Incremental)",
            "Google Drive",
            "Dropbox",
            "OneDrive",
        ]);
        backend_row.set_model(Some(&backends));
        dest_group.add(&backend_row);

        // Local destination
        let local_dest = EntryRow::builder()
            .title("Caminho Local")
            .text("/media/backup/winux")
            .build();

        let browse_btn = Button::from_icon_name("folder-open-symbolic");
        browse_btn.add_css_class("flat");
        browse_btn.set_valign(gtk4::Align::Center);
        local_dest.add_suffix(&browse_btn);
        dest_group.add(&local_dest);

        // Remote options (shown when rsync/restic selected)
        let remote_host = EntryRow::builder()
            .title("Host Remoto")
            .text("backup.exemplo.com")
            .build();
        dest_group.add(&remote_host);

        let remote_user = EntryRow::builder()
            .title("Usuario")
            .text("backup")
            .build();
        dest_group.add(&remote_user);

        let remote_path = EntryRow::builder()
            .title("Caminho Remoto")
            .text("/backups/winux")
            .build();
        dest_group.add(&remote_path);

        page.add(&dest_group);

        // Options Group
        let options_group = PreferencesGroup::builder()
            .title("Opcoes de Backup")
            .build();

        let compression_row = ComboRow::builder()
            .title("Compressao")
            .subtitle("Reduz o tamanho do backup")
            .build();
        let compression_levels = gtk4::StringList::new(&[
            "Nenhuma",
            "Rapida (LZ4)",
            "Balanceada (ZSTD)",
            "Maxima (LZMA)",
        ]);
        compression_row.set_model(Some(&compression_levels));
        compression_row.set_selected(2);
        options_group.add(&compression_row);

        let encryption_row = SwitchRow::builder()
            .title("Criptografia")
            .subtitle("Proteger backup com senha (AES-256)")
            .active(false)
            .build();
        options_group.add(&encryption_row);

        let verify_row = SwitchRow::builder()
            .title("Verificar Apos Backup")
            .subtitle("Verificar integridade dos arquivos")
            .active(true)
            .build();
        options_group.add(&verify_row);

        let incremental_row = SwitchRow::builder()
            .title("Backup Incremental")
            .subtitle("Apenas arquivos alterados")
            .active(true)
            .build();
        options_group.add(&incremental_row);

        let dedup_row = SwitchRow::builder()
            .title("Deduplicacao")
            .subtitle("Evitar arquivos duplicados")
            .active(true)
            .build();
        options_group.add(&dedup_row);

        page.add(&options_group);

        // Backup Name Group
        let name_group = PreferencesGroup::builder()
            .title("Identificacao")
            .build();

        let name_row = EntryRow::builder()
            .title("Nome do Backup")
            .text("Backup Home - Manual")
            .build();
        name_group.add(&name_row);

        let tags_row = EntryRow::builder()
            .title("Tags")
            .text("home, manual, importante")
            .build();
        name_group.add(&tags_row);

        page.add(&name_group);

        // Action Buttons Group
        let action_group = PreferencesGroup::new();

        let action_box = Box::new(Orientation::Horizontal, 12);
        action_box.set_halign(gtk4::Align::Center);
        action_box.set_margin_top(12);
        action_box.set_margin_bottom(12);

        let estimate_btn = Button::with_label("Estimar Tamanho");
        estimate_btn.add_css_class("pill");
        action_box.append(&estimate_btn);

        let start_btn = Button::with_label("Iniciar Backup");
        start_btn.add_css_class("suggested-action");
        start_btn.add_css_class("pill");
        action_box.append(&start_btn);

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

impl Default for CreatePage {
    fn default() -> Self {
        Self::new()
    }
}
