// Terminal output page with build history

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, ListBox, Orientation, Paned, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, StatusPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::{AppState, BuildHistoryEntry, BuildStatus};

#[derive(Clone)]
pub struct OutputPage {
    widget: Box,
    state: Rc<RefCell<AppState>>,
    terminal_box: Box,
    history_list: ListBox,
    current_command: Rc<RefCell<Option<String>>>,
}

impl OutputPage {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let widget = Box::new(Orientation::Vertical, 0);

        // Main paned view - terminal on left, history on right
        let paned = Paned::new(Orientation::Horizontal);
        paned.set_vexpand(true);

        // Terminal section
        let terminal_section = Box::new(Orientation::Vertical, 0);

        let terminal_header = adw::HeaderBar::new();
        terminal_header.set_show_end_title_buttons(false);
        terminal_header.set_show_start_title_buttons(false);

        let terminal_title = Label::new(Some("Terminal de Build"));
        terminal_header.set_title_widget(Some(&terminal_title));

        // Terminal controls
        let clear_btn = Button::from_icon_name("edit-clear-symbolic");
        clear_btn.set_tooltip_text(Some("Limpar terminal"));
        terminal_header.pack_end(&clear_btn);

        let stop_btn = Button::from_icon_name("process-stop-symbolic");
        stop_btn.set_tooltip_text(Some("Parar build"));
        stop_btn.add_css_class("destructive-action");
        terminal_header.pack_end(&stop_btn);

        terminal_section.append(&terminal_header);

        // Terminal widget placeholder
        // In a real implementation, this would be a VTE terminal
        let terminal_box = Box::new(Orientation::Vertical, 0);
        terminal_box.set_vexpand(true);
        terminal_box.add_css_class("terminal-view");

        // Terminal placeholder content
        let terminal_placeholder = StatusPage::builder()
            .icon_name("utilities-terminal-symbolic")
            .title("Terminal de Build")
            .description("Inicie um build para ver a saida aqui")
            .build();

        terminal_box.append(&terminal_placeholder);

        let terminal_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .child(&terminal_box)
            .build();

        terminal_section.append(&terminal_scroll);

        // Add CSS for terminal styling
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(
            r#"
            .terminal-view {
                background-color: #1e1e1e;
                color: #d4d4d4;
                font-family: monospace;
                padding: 8px;
            }
            .terminal-output {
                font-family: 'JetBrains Mono', 'Fira Code', monospace;
                font-size: 10pt;
            }
            .terminal-success {
                color: #4ec9b0;
            }
            .terminal-error {
                color: #f14c4c;
            }
            .terminal-warning {
                color: #cca700;
            }
            "#,
        );

        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        paned.set_start_child(Some(&terminal_section));
        paned.set_resize_start_child(true);

        // History section
        let history_section = Box::new(Orientation::Vertical, 0);

        let history_header = adw::HeaderBar::new();
        history_header.set_show_end_title_buttons(false);
        history_header.set_show_start_title_buttons(false);

        let history_title = Label::new(Some("Historico"));
        history_header.set_title_widget(Some(&history_title));

        let clear_history_btn = Button::from_icon_name("user-trash-symbolic");
        clear_history_btn.set_tooltip_text(Some("Limpar historico"));
        history_header.pack_end(&clear_history_btn);

        history_section.append(&history_header);

        let history_list = ListBox::new();
        history_list.add_css_class("boxed-list");
        history_list.set_selection_mode(gtk4::SelectionMode::None);

        // Placeholder for empty history
        let history_placeholder = ActionRow::builder()
            .title("Nenhum build realizado")
            .subtitle("O historico aparecera aqui")
            .build();
        history_placeholder.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));
        history_list.append(&history_placeholder);

        let history_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .child(&history_list)
            .build();

        history_section.append(&history_scroll);

        paned.set_end_child(Some(&history_section));
        paned.set_resize_end_child(false);
        paned.set_shrink_end_child(false);
        paned.set_position(700);

        widget.append(&paned);

        let output_page = Self {
            widget,
            state,
            terminal_box,
            history_list,
            current_command: Rc::new(RefCell::new(None)),
        };

        // Connect clear terminal button
        let term_box = output_page.terminal_box.clone();
        clear_btn.connect_clicked(move |_| {
            Self::clear_terminal(&term_box);
        });

        // Connect clear history button
        let hist_list = output_page.history_list.clone();
        let state_clone = output_page.state.clone();
        clear_history_btn.connect_clicked(move |_| {
            Self::clear_history(&hist_list, &state_clone);
        });

        output_page
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn run_command(&self, command: &str) {
        *self.current_command.borrow_mut() = Some(command.to_string());

        // Clear terminal
        Self::clear_terminal(&self.terminal_box);

        // Add command header
        let header = Label::new(Some(&format!("$ {}", command)));
        header.set_xalign(0.0);
        header.add_css_class("terminal-output");
        header.set_selectable(true);
        header.set_wrap(true);
        self.terminal_box.append(&header);

        let separator = gtk4::Separator::new(Orientation::Horizontal);
        separator.set_margin_top(4);
        separator.set_margin_bottom(4);
        self.terminal_box.append(&separator);

        // In a real implementation, we would:
        // 1. Spawn the command using tokio or std::process
        // 2. Stream stdout/stderr to the terminal
        // 3. Update build status in history

        // For now, show a placeholder
        let output = Label::new(Some("Build iniciado...\n\nEm uma implementacao completa, o output do comando apareceria aqui em tempo real usando VTE ou streaming de processo."));
        output.set_xalign(0.0);
        output.add_css_class("terminal-output");
        output.set_selectable(true);
        output.set_wrap(true);
        self.terminal_box.append(&output);

        // Add to history
        self.add_to_history(command, BuildStatus::InProgress);
    }

    fn clear_terminal(terminal_box: &Box) {
        while let Some(child) = terminal_box.first_child() {
            terminal_box.remove(&child);
        }
    }

    fn clear_history(history_list: &ListBox, state: &Rc<RefCell<AppState>>) {
        while let Some(child) = history_list.first_child() {
            history_list.remove(&child);
        }

        state.borrow_mut().build_history.clear();

        // Add placeholder
        let placeholder = ActionRow::builder()
            .title("Nenhum build realizado")
            .subtitle("O historico aparecera aqui")
            .build();
        placeholder.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));
        history_list.append(&placeholder);
    }

    fn add_to_history(&self, command: &str, status: BuildStatus) {
        // Remove placeholder if present
        if let Some(first) = self.history_list.first_child() {
            if first
                .first_child()
                .and_then(|c| c.downcast::<ActionRow>().ok())
                .map(|r| r.title() == "Nenhum build realizado")
                .unwrap_or(false)
            {
                self.history_list.remove(&first);
            }
        }

        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

        let entry = BuildHistoryEntry {
            project_name: self
                .state
                .borrow()
                .current_project
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            target: "native".to_string(),
            timestamp: timestamp.clone(),
            status: status.clone(),
            output_path: None,
        };

        // Create history row
        let row = ActionRow::builder()
            .title(&entry.project_name)
            .subtitle(&format!("{} - {}", entry.target, timestamp))
            .build();

        let icon_name = match status {
            BuildStatus::Success => "emblem-ok-symbolic",
            BuildStatus::Failed => "dialog-error-symbolic",
            BuildStatus::InProgress => "content-loading-symbolic",
        };

        let icon = gtk4::Image::from_icon_name(icon_name);
        match status {
            BuildStatus::Success => icon.add_css_class("success"),
            BuildStatus::Failed => icon.add_css_class("error"),
            BuildStatus::InProgress => {}
        }
        row.add_prefix(&icon);

        // Prepend to show most recent first
        self.history_list.prepend(&row);

        // Update state
        self.state.borrow_mut().build_history.push(entry);
    }

    pub fn update_build_status(&self, status: BuildStatus) {
        // Update the most recent history entry
        if let Some(first) = self.history_list.first_child() {
            if let Some(row) = first.first_child().and_then(|c| c.downcast::<ActionRow>().ok()) {
                // Remove old icon
                if let Some(prefix) = row.first_child() {
                    row.remove(&prefix);
                }

                let icon_name = match status {
                    BuildStatus::Success => "emblem-ok-symbolic",
                    BuildStatus::Failed => "dialog-error-symbolic",
                    BuildStatus::InProgress => "content-loading-symbolic",
                };

                let icon = gtk4::Image::from_icon_name(icon_name);
                match status {
                    BuildStatus::Success => icon.add_css_class("success"),
                    BuildStatus::Failed => icon.add_css_class("error"),
                    BuildStatus::InProgress => {}
                }
                row.add_prefix(&icon);
            }
        }

        // Update state
        if let Some(entry) = self.state.borrow_mut().build_history.last_mut() {
            entry.status = status;
        }
    }
}
