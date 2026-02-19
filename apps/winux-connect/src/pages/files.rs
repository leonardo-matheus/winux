//! Files page - File transfer with drag & drop support

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::protocol::ConnectionManager;
use crate::ui::FileRow;

/// File transfer information
#[derive(Clone)]
pub struct FileTransfer {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub progress: f64,
    pub status: TransferStatus,
    pub direction: TransferDirection,
    pub device_name: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TransferDirection {
    Upload,
    Download,
}

impl FileTransfer {
    pub fn new(
        id: &str,
        filename: &str,
        size: u64,
        progress: f64,
        status: TransferStatus,
        direction: TransferDirection,
        device_name: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            filename: filename.to_string(),
            size,
            progress,
            status,
            direction,
            device_name: device_name.to_string(),
        }
    }
}

/// Files page for file transfer
pub struct FilesPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<ConnectionManager>>,
}

impl FilesPage {
    pub fn new(manager: Rc<RefCell<ConnectionManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Arquivos");
        page.set_icon_name(Some("folder-symbolic"));

        // Device selector
        let device_group = adw::PreferencesGroup::builder()
            .title("Dispositivo")
            .build();

        let device_combo = adw::ComboRow::builder()
            .title("Dispositivo de destino")
            .subtitle("Selecione para onde enviar arquivos")
            .build();
        let devices = gtk4::StringList::new(&[
            "Samsung Galaxy S24",
            "iPad Pro",
        ]);
        device_combo.set_model(Some(&devices));
        device_group.add(&device_combo);

        page.add(&device_group);

        // Drop zone
        let drop_group = adw::PreferencesGroup::builder()
            .title("Enviar Arquivos")
            .description("Arraste arquivos para enviar ou clique para selecionar")
            .build();

        // Create drop zone widget
        let drop_zone = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        drop_zone.set_margin_start(24);
        drop_zone.set_margin_end(24);
        drop_zone.set_margin_top(32);
        drop_zone.set_margin_bottom(32);
        drop_zone.set_halign(gtk4::Align::Center);

        let drop_icon = gtk4::Image::from_icon_name("folder-download-symbolic");
        drop_icon.set_pixel_size(64);
        drop_icon.add_css_class("dim-label");
        drop_zone.append(&drop_icon);

        let drop_label = gtk4::Label::new(Some("Arraste arquivos aqui"));
        drop_label.add_css_class("title-2");
        drop_zone.append(&drop_label);

        let drop_sublabel = gtk4::Label::new(Some("ou clique para selecionar"));
        drop_sublabel.add_css_class("dim-label");
        drop_zone.append(&drop_sublabel);

        let select_button = gtk4::Button::builder()
            .label("Selecionar Arquivos")
            .halign(gtk4::Align::Center)
            .margin_top(8)
            .build();
        select_button.add_css_class("suggested-action");
        drop_zone.append(&select_button);

        // Make drop zone a drop target
        let drop_target = gtk4::DropTarget::new(gio::File::static_type(), gdk4::DragAction::COPY);
        drop_zone.add_controller(drop_target);

        // Wrap in a frame
        let drop_frame = gtk4::Frame::new(None);
        drop_frame.set_child(Some(&drop_zone));
        drop_frame.add_css_class("view");

        let drop_row = adw::ActionRow::new();
        drop_row.set_child(Some(&drop_frame));
        drop_group.add(&drop_row);

        page.add(&drop_group);

        // Active transfers
        let active_group = adw::PreferencesGroup::builder()
            .title("Transferencias Ativas")
            .description("Arquivos sendo transferidos")
            .build();

        let active_transfers = vec![
            FileTransfer::new(
                "1",
                "Documento.pdf",
                5_242_880, // 5MB
                0.65,
                TransferStatus::InProgress,
                TransferDirection::Upload,
                "Samsung Galaxy S24",
            ),
            FileTransfer::new(
                "2",
                "Fotos_Ferias.zip",
                157_286_400, // 150MB
                0.23,
                TransferStatus::InProgress,
                TransferDirection::Download,
                "Samsung Galaxy S24",
            ),
        ];

        for transfer in &active_transfers {
            let row = FileRow::new(transfer);
            active_group.add(&row.widget());
        }

        if active_transfers.is_empty() {
            let empty_row = adw::ActionRow::builder()
                .title("Nenhuma transferencia ativa")
                .sensitive(false)
                .build();
            active_group.add(&empty_row);
        }

        page.add(&active_group);

        // History
        let history_group = adw::PreferencesGroup::builder()
            .title("Historico")
            .description("Transferencias recentes")
            .build();

        let history = vec![
            FileTransfer::new(
                "3",
                "Relatorio.xlsx",
                1_048_576, // 1MB
                1.0,
                TransferStatus::Completed,
                TransferDirection::Upload,
                "Samsung Galaxy S24",
            ),
            FileTransfer::new(
                "4",
                "Screenshot_2024.png",
                524_288, // 512KB
                1.0,
                TransferStatus::Completed,
                TransferDirection::Download,
                "iPad Pro",
            ),
            FileTransfer::new(
                "5",
                "Video_Corrupto.mp4",
                52_428_800, // 50MB
                0.45,
                TransferStatus::Failed,
                TransferDirection::Download,
                "Samsung Galaxy S24",
            ),
        ];

        for transfer in &history {
            let row = FileRow::new(transfer);
            history_group.add(&row.widget());
        }

        // Clear history button
        let clear_button = gtk4::Button::builder()
            .label("Limpar")
            .build();

        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_box.append(&clear_button);

        history_group.set_header_suffix(Some(&button_box));

        page.add(&history_group);

        // Browse phone files
        let browse_group = adw::PreferencesGroup::builder()
            .title("Explorar Arquivos")
            .build();

        let browse_row = adw::ActionRow::builder()
            .title("Navegar arquivos do telefone")
            .subtitle("Acesse o sistema de arquivos do dispositivo")
            .activatable(true)
            .build();
        browse_row.add_prefix(&gtk4::Image::from_icon_name("folder-remote-symbolic"));
        browse_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        browse_group.add(&browse_row);

        let dcim_row = adw::ActionRow::builder()
            .title("Fotos e Videos")
            .subtitle("Acesso rapido a pasta DCIM")
            .activatable(true)
            .build();
        dcim_row.add_prefix(&gtk4::Image::from_icon_name("folder-pictures-symbolic"));
        dcim_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        browse_group.add(&dcim_row);

        let downloads_row = adw::ActionRow::builder()
            .title("Downloads")
            .subtitle("Arquivos baixados no telefone")
            .activatable(true)
            .build();
        downloads_row.add_prefix(&gtk4::Image::from_icon_name("folder-download-symbolic"));
        downloads_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        browse_group.add(&downloads_row);

        page.add(&browse_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
