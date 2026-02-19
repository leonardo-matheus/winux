//! Custom row widget for displaying updates

use gtk4::prelude::*;
use gtk4::{Box, Button, CheckButton, Label, Orientation, Revealer};
use libadwaita as adw;
use adw::prelude::*;
use adw::ActionRow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{PackageUpdate, UpdateSource, UpdatePriority};

/// Custom row widget for displaying an update
pub struct UpdateRow {
    row: ActionRow,
    checkbox: CheckButton,
    is_selected: Rc<RefCell<bool>>,
}

impl UpdateRow {
    /// Create a new update row
    pub fn new(update: &PackageUpdate) -> Self {
        let row = ActionRow::builder()
            .title(&update.name)
            .subtitle(&Self::format_subtitle(update))
            .activatable(true)
            .build();

        // Checkbox for selection
        let checkbox = CheckButton::new();
        checkbox.set_active(true);
        checkbox.set_valign(gtk4::Align::Center);
        checkbox.set_tooltip_text(Some("Selecionar para atualizacao"));
        row.add_prefix(&checkbox);

        // Source icon
        let source_icon = gtk4::Image::from_icon_name(Self::source_icon(update.source));
        source_icon.set_tooltip_text(Some(&update.source.to_string()));
        row.add_prefix(&source_icon);

        // Priority indicator
        if update.priority == UpdatePriority::Security {
            let security_icon = gtk4::Image::from_icon_name("security-high-symbolic");
            security_icon.set_tooltip_text(Some("Atualizacao de seguranca"));
            security_icon.add_css_class("warning");
            row.add_suffix(&security_icon);
        }

        // Restart indicator
        if update.requires_restart {
            let restart_icon = gtk4::Image::from_icon_name("system-reboot-symbolic");
            restart_icon.set_tooltip_text(Some("Requer reinicializacao"));
            restart_icon.add_css_class("dim-label");
            row.add_suffix(&restart_icon);
        }

        // Size label
        let size_label = Label::new(Some(&update.size_display()));
        size_label.add_css_class("dim-label");
        size_label.set_valign(gtk4::Align::Center);
        row.add_suffix(&size_label);

        // Changelog button
        let changelog_btn = Button::builder()
            .icon_name("text-x-generic-symbolic")
            .tooltip_text("Ver changelog")
            .valign(gtk4::Align::Center)
            .build();
        changelog_btn.add_css_class("flat");

        if update.changelog.is_some() {
            let changelog = update.changelog.clone().unwrap_or_default();
            changelog_btn.connect_clicked(move |_| {
                // Would show changelog dialog
                tracing::info!("Show changelog: {}", changelog);
            });
            row.add_suffix(&changelog_btn);
        }

        let is_selected = Rc::new(RefCell::new(true));
        let is_selected_clone = is_selected.clone();

        checkbox.connect_toggled(move |cb| {
            *is_selected_clone.borrow_mut() = cb.is_active();
        });

        Self {
            row,
            checkbox,
            is_selected,
        }
    }

    /// Create a row with expanded details
    pub fn new_expanded(update: &PackageUpdate) -> adw::ExpanderRow {
        let row = adw::ExpanderRow::builder()
            .title(&update.name)
            .subtitle(&format!(
                "{} -> {} ({})",
                update.current_version,
                update.new_version,
                update.size_display()
            ))
            .build();

        // Source icon
        let source_icon = gtk4::Image::from_icon_name(Self::source_icon(update.source));
        row.add_prefix(&source_icon);

        // Checkbox
        let checkbox = CheckButton::new();
        checkbox.set_active(true);
        checkbox.set_valign(gtk4::Align::Center);
        row.add_suffix(&checkbox);

        // Description row
        let desc_row = ActionRow::builder()
            .title("Descricao")
            .subtitle(&update.description)
            .build();
        row.add_row(&desc_row);

        // Version details row
        let version_row = ActionRow::builder()
            .title("Versao")
            .subtitle(&format!(
                "Atual: {} -> Nova: {}",
                update.current_version, update.new_version
            ))
            .build();
        row.add_row(&version_row);

        // Download size row
        let size_row = ActionRow::builder()
            .title("Tamanho do Download")
            .subtitle(&update.size_display())
            .build();
        row.add_row(&size_row);

        // Priority row
        let priority_str = match update.priority {
            UpdatePriority::Security => "Seguranca (Critica)",
            UpdatePriority::Important => "Importante",
            UpdatePriority::Normal => "Normal",
            UpdatePriority::Optional => "Opcional",
        };
        let priority_row = ActionRow::builder()
            .title("Prioridade")
            .subtitle(priority_str)
            .build();
        row.add_row(&priority_row);

        // Changelog row (if available)
        if let Some(changelog) = &update.changelog {
            let changelog_row = ActionRow::builder()
                .title("Changelog")
                .subtitle(changelog)
                .build();
            row.add_row(&changelog_row);
        }

        row
    }

    /// Format subtitle with version and description
    fn format_subtitle(update: &PackageUpdate) -> String {
        format!(
            "{} -> {} ({}) - {}",
            update.current_version,
            update.new_version,
            update.size_display(),
            update.description
        )
    }

    /// Get icon name for update source
    fn source_icon(source: UpdateSource) -> &'static str {
        match source {
            UpdateSource::Apt => "package-x-generic-symbolic",
            UpdateSource::Flatpak => "application-x-executable-symbolic",
            UpdateSource::Snap => "application-x-addon-symbolic",
            UpdateSource::Fwupd => "drive-harddisk-solidstate-symbolic",
        }
    }

    /// Get the row widget
    pub fn widget(&self) -> &ActionRow {
        &self.row
    }

    /// Get selection state
    pub fn is_selected(&self) -> bool {
        *self.is_selected.borrow()
    }

    /// Set selection state
    pub fn set_selected(&self, selected: bool) {
        self.checkbox.set_active(selected);
        *self.is_selected.borrow_mut() = selected;
    }

    /// Get checkbox widget
    pub fn checkbox(&self) -> &CheckButton {
        &self.checkbox
    }
}

/// Group header for update sources
pub struct SourceGroupHeader {
    widget: Box,
    label: Label,
    count_label: Label,
    size_label: Label,
}

impl SourceGroupHeader {
    pub fn new(source: UpdateSource, count: usize, total_size: u64) -> Self {
        let widget = Box::new(Orientation::Horizontal, 12);
        widget.set_margin_start(12);
        widget.set_margin_end(12);
        widget.set_margin_top(8);
        widget.set_margin_bottom(8);

        // Source icon
        let icon = gtk4::Image::from_icon_name(UpdateRow::source_icon(source));
        widget.append(&icon);

        // Source label
        let label = Label::new(Some(&source.to_string()));
        label.add_css_class("heading");
        label.set_hexpand(true);
        label.set_halign(gtk4::Align::Start);
        widget.append(&label);

        // Count label
        let count_label = Label::new(Some(&format!("{} pacotes", count)));
        count_label.add_css_class("dim-label");
        widget.append(&count_label);

        // Size label
        let size_label = Label::new(Some(&format_size(total_size)));
        size_label.add_css_class("dim-label");
        widget.append(&size_label);

        Self {
            widget,
            label,
            count_label,
            size_label,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn update_counts(&self, count: usize, total_size: u64) {
        self.count_label.set_text(&format!("{} pacotes", count));
        self.size_label.set_text(&format_size(total_size));
    }
}

/// Helper function to format size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
