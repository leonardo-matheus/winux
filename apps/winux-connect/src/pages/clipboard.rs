//! Clipboard page - Automatic clipboard synchronization

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::protocol::ConnectionManager;

/// Clipboard entry
#[derive(Clone)]
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    pub content_type: ClipboardContentType,
    pub timestamp: String,
    pub source: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ClipboardContentType {
    Text,
    Url,
    Image,
    File,
}

impl ClipboardContentType {
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Text => "edit-paste-symbolic",
            Self::Url => "web-browser-symbolic",
            Self::Image => "image-x-generic-symbolic",
            Self::File => "text-x-generic-symbolic",
        }
    }
}

impl ClipboardEntry {
    pub fn new(
        id: &str,
        content: &str,
        content_type: ClipboardContentType,
        timestamp: &str,
        source: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            content: content.to_string(),
            content_type,
            timestamp: timestamp.to_string(),
            source: source.to_string(),
        }
    }
}

/// Clipboard sync page
pub struct ClipboardPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<ConnectionManager>>,
}

impl ClipboardPage {
    pub fn new(manager: Rc<RefCell<ConnectionManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Clipboard");
        page.set_icon_name(Some("edit-paste-symbolic"));

        // Sync settings
        let settings_group = adw::PreferencesGroup::builder()
            .title("Sincronizacao")
            .build();

        let auto_sync = adw::SwitchRow::builder()
            .title("Sincronizacao automatica")
            .subtitle("Sincronizar clipboard entre dispositivos automaticamente")
            .active(true)
            .build();
        settings_group.add(&auto_sync);

        let sync_images = adw::SwitchRow::builder()
            .title("Sincronizar imagens")
            .subtitle("Incluir imagens copiadas na sincronizacao")
            .active(true)
            .build();
        settings_group.add(&sync_images);

        let sync_files = adw::SwitchRow::builder()
            .title("Sincronizar arquivos")
            .subtitle("Sincronizar caminhos de arquivos copiados")
            .active(false)
            .build();
        settings_group.add(&sync_files);

        let history_size = adw::SpinRow::with_range(10.0, 100.0, 10.0);
        history_size.set_title("Tamanho do historico");
        history_size.set_subtitle("Numero maximo de itens no historico");
        history_size.set_value(50.0);
        settings_group.add(&history_size);

        page.add(&settings_group);

        // Device filter
        let device_group = adw::PreferencesGroup::builder()
            .title("Dispositivos")
            .description("Selecione com quais dispositivos sincronizar")
            .build();

        let device_switches = vec![
            ("Samsung Galaxy S24", true),
            ("iPad Pro", true),
            ("Pixel 8 Pro", false),
        ];

        for (device, enabled) in device_switches {
            let row = adw::SwitchRow::builder()
                .title(device)
                .active(enabled)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name("phone-symbolic"));
            device_group.add(&row);
        }

        page.add(&device_group);

        // Current clipboard
        let current_group = adw::PreferencesGroup::builder()
            .title("Clipboard Atual")
            .build();

        let current_content = "https://github.com/winux-os/winux";

        let current_row = adw::ActionRow::builder()
            .title("Conteudo atual")
            .subtitle(current_content)
            .build();
        current_row.add_prefix(&gtk4::Image::from_icon_name("web-browser-symbolic"));

        let copy_btn = gtk4::Button::from_icon_name("edit-copy-symbolic");
        copy_btn.set_tooltip_text(Some("Copiar"));
        copy_btn.set_valign(gtk4::Align::Center);
        current_row.add_suffix(&copy_btn);

        let send_btn = gtk4::Button::builder()
            .label("Enviar")
            .valign(gtk4::Align::Center)
            .build();
        send_btn.add_css_class("suggested-action");
        current_row.add_suffix(&send_btn);

        current_group.add(&current_row);

        page.add(&current_group);

        // History
        let history_group = adw::PreferencesGroup::builder()
            .title("Historico")
            .description("Itens copiados recentemente")
            .build();

        let history = vec![
            ClipboardEntry::new(
                "1",
                "https://github.com/winux-os/winux",
                ClipboardContentType::Url,
                "10:30",
                "Samsung Galaxy S24",
            ),
            ClipboardEntry::new(
                "2",
                "func main() { fmt.Println(\"Hello, World!\") }",
                ClipboardContentType::Text,
                "10:15",
                "PC",
            ),
            ClipboardEntry::new(
                "3",
                "Reuniao amanha as 15h na sala de conferencias",
                ClipboardContentType::Text,
                "09:45",
                "Samsung Galaxy S24",
            ),
            ClipboardEntry::new(
                "4",
                "/home/user/Documents/relatorio.pdf",
                ClipboardContentType::File,
                "09:30",
                "PC",
            ),
            ClipboardEntry::new(
                "5",
                "[Imagem: screenshot_2024.png]",
                ClipboardContentType::Image,
                "09:00",
                "Samsung Galaxy S24",
            ),
        ];

        for entry in &history {
            let row = Self::create_history_row(entry);
            history_group.add(&row);
        }

        // Clear history button
        let clear_btn = gtk4::Button::builder()
            .label("Limpar")
            .build();
        clear_btn.add_css_class("destructive-action");

        let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        btn_box.append(&clear_btn);
        history_group.set_header_suffix(Some(&btn_box));

        page.add(&history_group);

        // Privacy settings
        let privacy_group = adw::PreferencesGroup::builder()
            .title("Privacidade")
            .build();

        let exclude_passwords = adw::SwitchRow::builder()
            .title("Excluir senhas")
            .subtitle("Nao sincronizar conteudo de gerenciadores de senhas")
            .active(true)
            .build();
        privacy_group.add(&exclude_passwords);

        let exclude_sensitive = adw::SwitchRow::builder()
            .title("Detectar dados sensiveis")
            .subtitle("Nao sincronizar numeros de cartao, CPF, etc.")
            .active(true)
            .build();
        privacy_group.add(&exclude_sensitive);

        let auto_clear = adw::ComboRow::builder()
            .title("Limpar historico automaticamente")
            .subtitle("Tempo para manter itens no historico")
            .build();
        let clear_options = gtk4::StringList::new(&[
            "Nunca",
            "Apos 1 hora",
            "Apos 24 horas",
            "Apos 7 dias",
        ]);
        auto_clear.set_model(Some(&clear_options));
        auto_clear.set_selected(1);
        privacy_group.add(&auto_clear);

        page.add(&privacy_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    fn create_history_row(entry: &ClipboardEntry) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(&entry.content)
            .subtitle(&format!("{} - {}", entry.timestamp, entry.source))
            .activatable(true)
            .build();

        row.add_prefix(&gtk4::Image::from_icon_name(entry.content_type.icon_name()));

        let copy_btn = gtk4::Button::from_icon_name("edit-copy-symbolic");
        copy_btn.set_tooltip_text(Some("Copiar"));
        copy_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&copy_btn);

        let delete_btn = gtk4::Button::from_icon_name("edit-delete-symbolic");
        delete_btn.set_tooltip_text(Some("Remover"));
        delete_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&delete_btn);

        row
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
