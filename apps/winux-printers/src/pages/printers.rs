//! Configured printers list page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::cups::{CupsManager, Printer, PrinterStatus};
use crate::ui::PrinterCard;

/// Printers page - shows all configured printers
pub struct PrintersPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<CupsManager>>,
}

impl PrintersPage {
    pub fn new(manager: Rc<RefCell<CupsManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Impressoras");
        page.set_icon_name(Some("printer-symbolic"));

        // Default printer group
        let default_group = adw::PreferencesGroup::builder()
            .title("Impressora Padrao")
            .description("Impressora utilizada por padrao para impressao")
            .build();

        // Sample default printer
        let default_printer = Printer::new(
            "HP-LaserJet-Pro",
            "HP LaserJet Pro M404dn",
            "ipp://192.168.1.100:631/printers/HP-LaserJet-Pro",
            PrinterStatus::Ready,
            true,
            true,
        );

        let default_row = Self::create_printer_row(&default_printer, true);
        default_group.add(&default_row);

        page.add(&default_group);

        // Other printers group
        let printers_group = adw::PreferencesGroup::builder()
            .title("Outras Impressoras")
            .description("Impressoras configuradas no sistema")
            .build();

        let printers = vec![
            Printer::new(
                "Canon-PIXMA",
                "Canon PIXMA TS8350",
                "usb://Canon/PIXMA%20TS8350?serial=12345",
                PrinterStatus::Ready,
                true,
                false,
            ),
            Printer::new(
                "Brother-MFC",
                "Brother MFC-L2750DW",
                "ipp://192.168.1.101:631/printers/Brother-MFC",
                PrinterStatus::Offline,
                true,
                false,
            ),
            Printer::new(
                "Epson-EcoTank",
                "Epson EcoTank ET-4760",
                "dnssd://Epson%20EcoTank._ipp._tcp.local",
                PrinterStatus::Error("Falta de papel".to_string()),
                false,
                false,
            ),
            Printer::new(
                "PDF-Printer",
                "Imprimir para PDF",
                "cups-pdf:/",
                PrinterStatus::Ready,
                true,
                false,
            ),
        ];

        for printer in &printers {
            let row = Self::create_printer_row(printer, false);
            printers_group.add(&row);
        }

        if printers.is_empty() {
            let empty_row = adw::ActionRow::builder()
                .title("Nenhuma impressora configurada")
                .subtitle("Adicione uma nova impressora para comecar")
                .sensitive(false)
                .build();
            printers_group.add(&empty_row);
        }

        page.add(&printers_group);

        // Quick actions group
        let actions_group = adw::PreferencesGroup::builder()
            .title("Acoes Rapidas")
            .build();

        let add_row = adw::ActionRow::builder()
            .title("Adicionar Impressora")
            .subtitle("Configurar uma nova impressora")
            .activatable(true)
            .build();
        add_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
        add_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        actions_group.add(&add_row);

        let test_row = adw::ActionRow::builder()
            .title("Imprimir Pagina de Teste")
            .subtitle("Verificar se a impressora padrao esta funcionando")
            .activatable(true)
            .build();
        test_row.add_prefix(&gtk4::Image::from_icon_name("document-print-symbolic"));
        actions_group.add(&test_row);

        let cups_row = adw::ActionRow::builder()
            .title("Abrir Interface CUPS")
            .subtitle("Gerenciamento avancado via navegador")
            .activatable(true)
            .build();
        cups_row.add_prefix(&gtk4::Image::from_icon_name("applications-internet-symbolic"));
        cups_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        actions_group.add(&cups_row);

        page.add(&actions_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    fn create_printer_row(printer: &Printer, is_default: bool) -> adw::ExpanderRow {
        let status_text = match &printer.status {
            PrinterStatus::Ready => "Pronta",
            PrinterStatus::Printing => "Imprimindo...",
            PrinterStatus::Offline => "Offline",
            PrinterStatus::Error(msg) => msg.as_str(),
            PrinterStatus::Paused => "Pausada",
        };

        let row = adw::ExpanderRow::builder()
            .title(&printer.description)
            .subtitle(status_text)
            .build();

        // Printer icon
        let icon = gtk4::Image::from_icon_name("printer-symbolic");
        row.add_prefix(&icon);

        // Status indicator
        let status_icon = match &printer.status {
            PrinterStatus::Ready => {
                let icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
                icon.add_css_class("success");
                icon
            }
            PrinterStatus::Printing => {
                let icon = gtk4::Image::from_icon_name("content-loading-symbolic");
                icon.add_css_class("accent");
                icon
            }
            PrinterStatus::Offline => {
                let icon = gtk4::Image::from_icon_name("network-offline-symbolic");
                icon.add_css_class("dim-label");
                icon
            }
            PrinterStatus::Error(_) => {
                let icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
                icon.add_css_class("error");
                icon
            }
            PrinterStatus::Paused => {
                let icon = gtk4::Image::from_icon_name("media-playback-pause-symbolic");
                icon.add_css_class("warning");
                icon
            }
        };
        row.add_suffix(&status_icon);

        // Default indicator
        if is_default {
            let default_badge = gtk4::Label::new(Some("Padrao"));
            default_badge.add_css_class("accent");
            default_badge.add_css_class("caption");
            default_badge.set_valign(gtk4::Align::Center);
            row.add_suffix(&default_badge);
        }

        // Expandable actions
        // Print test page
        let test_row = adw::ActionRow::builder()
            .title("Imprimir Pagina de Teste")
            .activatable(true)
            .build();
        test_row.add_prefix(&gtk4::Image::from_icon_name("document-print-symbolic"));
        row.add_row(&test_row);

        // Set as default
        if !is_default {
            let default_row = adw::ActionRow::builder()
                .title("Definir como Padrao")
                .activatable(true)
                .build();
            default_row.add_prefix(&gtk4::Image::from_icon_name("emblem-default-symbolic"));
            row.add_row(&default_row);
        }

        // Enable/disable toggle
        let enable_row = adw::SwitchRow::builder()
            .title(if printer.enabled { "Habilitada" } else { "Desabilitada" })
            .subtitle("Aceitar novos trabalhos de impressao")
            .active(printer.enabled)
            .build();
        row.add_row(&enable_row);

        // Pause/resume based on status
        let pause_row = adw::ActionRow::builder()
            .title(if matches!(printer.status, PrinterStatus::Paused) {
                "Retomar Impressao"
            } else {
                "Pausar Impressao"
            })
            .activatable(true)
            .build();
        let pause_icon = if matches!(printer.status, PrinterStatus::Paused) {
            "media-playback-start-symbolic"
        } else {
            "media-playback-pause-symbolic"
        };
        pause_row.add_prefix(&gtk4::Image::from_icon_name(pause_icon));
        row.add_row(&pause_row);

        // View queue
        let queue_row = adw::ActionRow::builder()
            .title("Ver Fila de Impressao")
            .activatable(true)
            .build();
        queue_row.add_prefix(&gtk4::Image::from_icon_name("view-list-symbolic"));
        queue_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        row.add_row(&queue_row);

        // Properties
        let props_row = adw::ActionRow::builder()
            .title("Propriedades")
            .subtitle(&printer.uri)
            .activatable(true)
            .build();
        props_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        row.add_row(&props_row);

        // Configure
        let config_row = adw::ActionRow::builder()
            .title("Configurar")
            .subtitle("Tamanho do papel, qualidade, duplex...")
            .activatable(true)
            .build();
        config_row.add_prefix(&gtk4::Image::from_icon_name("emblem-system-symbolic"));
        config_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        row.add_row(&config_row);

        // Remove
        let remove_row = adw::ActionRow::builder()
            .title("Remover Impressora")
            .subtitle("Excluir esta impressora do sistema")
            .activatable(true)
            .build();
        remove_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        remove_row.add_css_class("error");
        row.add_row(&remove_row);

        row
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
