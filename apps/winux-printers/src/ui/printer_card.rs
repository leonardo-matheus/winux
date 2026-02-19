//! Printer card widget for displaying printer information

use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::cups::{Printer, PrinterStatus};

/// A card widget displaying printer information with actions
#[derive(Debug, Clone)]
pub struct PrinterCard {
    widget: adw::ActionRow,
    status_icon: gtk4::Image,
    default_badge: gtk4::Label,
}

impl PrinterCard {
    /// Create a new printer card
    pub fn new(printer: &Printer) -> Self {
        let status_text = printer.status.display_string();

        let widget = adw::ActionRow::builder()
            .title(&printer.description)
            .subtitle(status_text)
            .activatable(true)
            .build();

        // Printer icon
        let icon = gtk4::Image::from_icon_name("printer-symbolic");
        icon.set_pixel_size(32);
        widget.add_prefix(&icon);

        // Status indicator icon
        let status_icon = gtk4::Image::from_icon_name(printer.status.icon_name());
        status_icon.set_valign(gtk4::Align::Center);

        // Apply appropriate CSS class based on status
        match &printer.status {
            PrinterStatus::Ready => status_icon.add_css_class("success"),
            PrinterStatus::Printing => status_icon.add_css_class("accent"),
            PrinterStatus::Offline => status_icon.add_css_class("dim-label"),
            PrinterStatus::Error(_) => status_icon.add_css_class("error"),
            PrinterStatus::Paused => status_icon.add_css_class("warning"),
        }

        widget.add_suffix(&status_icon.clone());

        // Default badge
        let default_badge = gtk4::Label::new(Some("Padrao"));
        default_badge.add_css_class("accent");
        default_badge.add_css_class("caption");
        default_badge.set_valign(gtk4::Align::Center);
        default_badge.set_margin_end(8);
        default_badge.set_visible(printer.is_default);
        widget.add_suffix(&default_badge.clone());

        // Navigate arrow
        let arrow = gtk4::Image::from_icon_name("go-next-symbolic");
        arrow.set_valign(gtk4::Align::Center);
        widget.add_suffix(&arrow);

        Self {
            widget,
            status_icon,
            default_badge,
        }
    }

    /// Create a detailed printer card with expandable actions
    pub fn new_expandable(printer: &Printer) -> adw::ExpanderRow {
        let status_text = printer.status.display_string();

        let row = adw::ExpanderRow::builder()
            .title(&printer.description)
            .subtitle(status_text)
            .build();

        // Printer icon
        let icon = gtk4::Image::from_icon_name("printer-symbolic");
        icon.set_pixel_size(24);
        row.add_prefix(&icon);

        // Status indicator
        let status_icon = gtk4::Image::from_icon_name(printer.status.icon_name());
        status_icon.set_valign(gtk4::Align::Center);

        match &printer.status {
            PrinterStatus::Ready => status_icon.add_css_class("success"),
            PrinterStatus::Printing => status_icon.add_css_class("accent"),
            PrinterStatus::Offline => status_icon.add_css_class("dim-label"),
            PrinterStatus::Error(_) => status_icon.add_css_class("error"),
            PrinterStatus::Paused => status_icon.add_css_class("warning"),
        }

        row.add_suffix(&status_icon);

        // Default badge
        if printer.is_default {
            let default_badge = gtk4::Label::new(Some("Padrao"));
            default_badge.add_css_class("accent");
            default_badge.add_css_class("caption");
            default_badge.set_valign(gtk4::Align::Center);
            default_badge.set_margin_end(8);
            row.add_suffix(&default_badge);
        }

        // Connection type indicator
        if let Some(conn_type) = printer.connection_type() {
            let conn_label = gtk4::Label::new(Some(conn_type.display_name()));
            conn_label.add_css_class("dim-label");
            conn_label.add_css_class("caption");
            conn_label.set_valign(gtk4::Align::Center);
            conn_label.set_margin_end(8);
            row.add_suffix(&conn_label);
        }

        // Add action rows
        Self::add_action_rows(&row, printer);

        row
    }

    /// Add action rows to an expander row
    fn add_action_rows(expander: &adw::ExpanderRow, printer: &Printer) {
        // Test print
        let test_row = adw::ActionRow::builder()
            .title("Imprimir Pagina de Teste")
            .activatable(true)
            .build();
        test_row.add_prefix(&gtk4::Image::from_icon_name("document-print-symbolic"));
        expander.add_row(&test_row);

        // Set as default (if not already)
        if !printer.is_default {
            let default_row = adw::ActionRow::builder()
                .title("Definir como Padrao")
                .activatable(true)
                .build();
            default_row.add_prefix(&gtk4::Image::from_icon_name("emblem-default-symbolic"));
            expander.add_row(&default_row);
        }

        // Enable/disable
        let enable_row = adw::SwitchRow::builder()
            .title(if printer.enabled {
                "Habilitada"
            } else {
                "Desabilitada"
            })
            .subtitle("Aceitar novos trabalhos")
            .active(printer.enabled)
            .build();
        expander.add_row(&enable_row);

        // Pause/resume
        let is_paused = matches!(printer.status, PrinterStatus::Paused);
        let pause_row = adw::ActionRow::builder()
            .title(if is_paused {
                "Retomar Impressao"
            } else {
                "Pausar Impressao"
            })
            .activatable(true)
            .build();
        let pause_icon = if is_paused {
            "media-playback-start-symbolic"
        } else {
            "media-playback-pause-symbolic"
        };
        pause_row.add_prefix(&gtk4::Image::from_icon_name(pause_icon));
        expander.add_row(&pause_row);

        // View queue
        let queue_row = adw::ActionRow::builder()
            .title("Ver Fila de Impressao")
            .activatable(true)
            .build();
        queue_row.add_prefix(&gtk4::Image::from_icon_name("view-list-symbolic"));
        queue_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        expander.add_row(&queue_row);

        // Properties/Info
        let props_row = adw::ActionRow::builder()
            .title("Propriedades")
            .subtitle(&printer.uri)
            .activatable(true)
            .build();
        props_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        expander.add_row(&props_row);

        // Configure
        let config_row = adw::ActionRow::builder()
            .title("Configurar")
            .subtitle("Papel, qualidade, duplex...")
            .activatable(true)
            .build();
        config_row.add_prefix(&gtk4::Image::from_icon_name("emblem-system-symbolic"));
        config_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        expander.add_row(&config_row);

        // Remove
        let remove_row = adw::ActionRow::builder()
            .title("Remover Impressora")
            .activatable(true)
            .build();
        remove_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        remove_row.add_css_class("error");
        expander.add_row(&remove_row);
    }

    /// Get the underlying widget
    pub fn widget(&self) -> &adw::ActionRow {
        &self.widget
    }

    /// Update the printer status display
    pub fn update_status(&self, status: &PrinterStatus) {
        self.widget.set_subtitle(Some(status.display_string()));
        self.status_icon.set_icon_name(Some(status.icon_name()));

        // Clear existing CSS classes
        self.status_icon.remove_css_class("success");
        self.status_icon.remove_css_class("accent");
        self.status_icon.remove_css_class("dim-label");
        self.status_icon.remove_css_class("error");
        self.status_icon.remove_css_class("warning");

        // Apply new CSS class
        match status {
            PrinterStatus::Ready => self.status_icon.add_css_class("success"),
            PrinterStatus::Printing => self.status_icon.add_css_class("accent"),
            PrinterStatus::Offline => self.status_icon.add_css_class("dim-label"),
            PrinterStatus::Error(_) => self.status_icon.add_css_class("error"),
            PrinterStatus::Paused => self.status_icon.add_css_class("warning"),
        }
    }

    /// Update the default badge visibility
    pub fn set_is_default(&self, is_default: bool) {
        self.default_badge.set_visible(is_default);
    }

    /// Connect to the activated signal
    pub fn connect_activated<F: Fn() + 'static>(&self, callback: F) {
        self.widget.connect_activated(move |_| {
            callback();
        });
    }
}
