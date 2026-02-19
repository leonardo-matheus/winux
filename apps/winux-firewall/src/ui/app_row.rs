//! App row widget
//!
//! A custom row widget for displaying application firewall permissions.

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

/// Application firewall state
#[derive(Debug, Clone)]
pub struct AppFirewallState {
    pub name: String,
    pub icon: String,
    pub allow_outgoing: bool,
    pub allow_incoming: bool,
    pub active_connections: usize,
    pub data_sent: u64,
    pub data_received: u64,
}

impl AppFirewallState {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            icon: "application-x-executable-symbolic".to_string(),
            allow_outgoing: true,
            allow_incoming: false,
            active_connections: 0,
            data_sent: 0,
            data_received: 0,
        }
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    pub fn with_permissions(mut self, outgoing: bool, incoming: bool) -> Self {
        self.allow_outgoing = outgoing;
        self.allow_incoming = incoming;
        self
    }

    pub fn with_connections(mut self, count: usize) -> Self {
        self.active_connections = count;
        self
    }
}

/// A row widget for displaying an application's firewall state
pub struct AppRow {
    widget: ExpanderRow,
    state: Rc<RefCell<AppFirewallState>>,
}

impl AppRow {
    pub fn new(state: AppFirewallState) -> Self {
        let state = Rc::new(RefCell::new(state));
        let widget = Self::build_widget(&state);

        Self { widget, state }
    }

    fn build_widget(state: &Rc<RefCell<AppFirewallState>>) -> ExpanderRow {
        let s = state.borrow();

        let expander = ExpanderRow::builder()
            .title(&s.name)
            .subtitle(&Self::format_subtitle(&s))
            .build();

        // App icon
        let icon = Image::from_icon_name(&s.icon);
        expander.add_prefix(&icon);

        // Status indicator
        let status_icon = if s.allow_outgoing || s.allow_incoming {
            let icon = Image::from_icon_name("emblem-ok-symbolic");
            icon.add_css_class("success");
            icon
        } else {
            let icon = Image::from_icon_name("action-unavailable-symbolic");
            icon.add_css_class("error");
            icon
        };
        expander.add_suffix(&status_icon);

        drop(s); // Release borrow

        // Outgoing switch
        let outgoing_switch = SwitchRow::builder()
            .title("Permitir Saida")
            .subtitle("Conexoes de saida para a internet")
            .active(state.borrow().allow_outgoing)
            .build();

        {
            let state_clone = state.clone();
            outgoing_switch.connect_active_notify(move |switch| {
                let mut s = state_clone.borrow_mut();
                s.allow_outgoing = switch.is_active();
                tracing::info!("{} outgoing: {}", s.name, s.allow_outgoing);
            });
        }

        expander.add_row(&outgoing_switch);

        // Incoming switch
        let incoming_switch = SwitchRow::builder()
            .title("Permitir Entrada")
            .subtitle("Conexoes de entrada (servidor)")
            .active(state.borrow().allow_incoming)
            .build();

        {
            let state_clone = state.clone();
            incoming_switch.connect_active_notify(move |switch| {
                let mut s = state_clone.borrow_mut();
                s.allow_incoming = switch.is_active();
                tracing::info!("{} incoming: {}", s.name, s.allow_incoming);
            });
        }

        expander.add_row(&incoming_switch);

        // Active connections row
        let connections_row = ActionRow::builder()
            .title("Conexoes Ativas")
            .subtitle("Ver conexoes abertas")
            .activatable(true)
            .build();

        let conn_count = Label::new(Some(&state.borrow().active_connections.to_string()));
        conn_count.add_css_class("title-3");
        connections_row.add_suffix(&conn_count);

        let conn_icon = Image::from_icon_name("go-next-symbolic");
        connections_row.add_suffix(&conn_icon);

        connections_row.connect_activated({
            let state_clone = state.clone();
            move |_| {
                let s = state_clone.borrow();
                tracing::info!("Viewing connections for: {}", s.name);
            }
        });

        expander.add_row(&connections_row);

        // Data usage row
        let usage_row = ActionRow::builder()
            .title("Uso de Dados")
            .build();

        let s = state.borrow();
        let usage_text = format!(
            "v {} | ^ {}",
            Self::format_bytes(s.data_received),
            Self::format_bytes(s.data_sent)
        );
        drop(s);

        let usage_label = Label::new(Some(&usage_text));
        usage_label.add_css_class("dim-label");
        usage_row.add_suffix(&usage_label);

        expander.add_row(&usage_row);

        // Custom rules row
        let rules_row = ActionRow::builder()
            .title("Regras Personalizadas")
            .subtitle("Configurar portas especificas")
            .activatable(true)
            .build();

        let rules_icon = Image::from_icon_name("emblem-system-symbolic");
        rules_row.add_prefix(&rules_icon);

        let rules_arrow = Image::from_icon_name("go-next-symbolic");
        rules_row.add_suffix(&rules_arrow);

        rules_row.connect_activated({
            let state_clone = state.clone();
            move |_| {
                let s = state_clone.borrow();
                tracing::info!("Configuring custom rules for: {}", s.name);
            }
        });

        expander.add_row(&rules_row);

        // Block all row
        let block_row = ActionRow::builder()
            .title("Bloquear Completamente")
            .subtitle("Impedir qualquer acesso a rede")
            .activatable(true)
            .build();
        block_row.add_css_class("error");

        let block_icon = Image::from_icon_name("network-offline-symbolic");
        block_row.add_prefix(&block_icon);

        block_row.connect_activated({
            let state_clone = state.clone();
            let outgoing = outgoing_switch.clone();
            let incoming = incoming_switch.clone();
            move |_| {
                let mut s = state_clone.borrow_mut();
                s.allow_outgoing = false;
                s.allow_incoming = false;
                outgoing.set_active(false);
                incoming.set_active(false);
                tracing::info!("Blocked all network access for: {}", s.name);
            }
        });

        expander.add_row(&block_row);

        expander
    }

    fn format_subtitle(state: &AppFirewallState) -> String {
        let conn_text = if state.active_connections > 0 {
            format!("{} conexoes ativas", state.active_connections)
        } else {
            "Sem conexoes".to_string()
        };

        let perm_text = match (state.allow_outgoing, state.allow_incoming) {
            (true, true) => "Saida e entrada permitidas",
            (true, false) => "Apenas saida permitida",
            (false, true) => "Apenas entrada permitida",
            (false, false) => "Bloqueado",
        };

        format!("{} - {}", conn_text, perm_text)
    }

    fn format_bytes(bytes: u64) -> String {
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

    /// Get the widget
    pub fn widget(&self) -> &ExpanderRow {
        &self.widget
    }

    /// Get a reference to the state
    pub fn state(&self) -> std::cell::Ref<AppFirewallState> {
        self.state.borrow()
    }

    /// Get a mutable reference to the state
    pub fn state_mut(&self) -> std::cell::RefMut<AppFirewallState> {
        self.state.borrow_mut()
    }

    /// Refresh the widget to reflect state changes
    pub fn refresh(&self) {
        let s = self.state.borrow();
        self.widget.set_subtitle(&Self::format_subtitle(&s));
    }
}

/// Create a simple app row for quick display
pub fn create_simple_app_row(
    name: &str,
    icon: &str,
    connections: usize,
    allowed: bool,
) -> ActionRow {
    let subtitle = if allowed {
        format!("{} conexoes - Permitido", connections)
    } else {
        "Bloqueado".to_string()
    };

    let row = ActionRow::builder()
        .title(name)
        .subtitle(&subtitle)
        .activatable(true)
        .build();

    let app_icon = Image::from_icon_name(icon);
    row.add_prefix(&app_icon);

    let status_icon = if allowed {
        let icon = Image::from_icon_name("emblem-ok-symbolic");
        icon.add_css_class("success");
        icon
    } else {
        let icon = Image::from_icon_name("action-unavailable-symbolic");
        icon.add_css_class("error");
        icon
    };
    row.add_suffix(&status_icon);

    let arrow = Image::from_icon_name("go-next-symbolic");
    row.add_suffix(&arrow);

    row
}

/// Create an active connection row
pub fn create_connection_row(
    app_name: &str,
    local_addr: &str,
    remote_addr: &str,
    protocol: &str,
    state: &str,
) -> ActionRow {
    let row = ActionRow::builder()
        .title(&format!("{} -> {}", local_addr, remote_addr))
        .subtitle(&format!("{} - {} - {}", app_name, protocol.to_uppercase(), state))
        .build();

    let proto_icon = match protocol.to_lowercase().as_str() {
        "tcp" => "network-transmit-symbolic",
        "udp" => "network-wireless-symbolic",
        _ => "network-wired-symbolic",
    };

    let icon = Image::from_icon_name(proto_icon);
    row.add_prefix(&icon);

    // State indicator
    let (state_icon, css_class) = match state.to_uppercase().as_str() {
        "ESTABLISHED" => ("emblem-ok-symbolic", "success"),
        "LISTEN" => ("media-playback-start-symbolic", "accent"),
        "TIME_WAIT" | "CLOSE_WAIT" => ("media-playback-pause-symbolic", "warning"),
        "SYN_SENT" | "SYN_RECV" => ("content-loading-symbolic", "accent"),
        _ => ("dialog-question-symbolic", "dim-label"),
    };

    let status = Image::from_icon_name(state_icon);
    status.add_css_class(css_class);
    row.add_suffix(&status);

    row
}
