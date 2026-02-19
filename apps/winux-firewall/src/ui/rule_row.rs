//! Rule row widget
//!
//! A custom row widget for displaying firewall rules with actions.

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{Action, Direction, FirewallRule, Protocol};

/// A row widget for displaying a firewall rule
pub struct RuleRow {
    widget: ExpanderRow,
    rule: Rc<RefCell<FirewallRule>>,
}

impl RuleRow {
    pub fn new(rule: FirewallRule) -> Self {
        let rule = Rc::new(RefCell::new(rule));
        let widget = Self::build_widget(&rule.borrow());

        Self { widget, rule }
    }

    fn build_widget(rule: &FirewallRule) -> ExpanderRow {
        // Build title from rule
        let title = Self::format_rule_title(rule);
        let subtitle = Self::format_rule_subtitle(rule);

        let expander = ExpanderRow::builder()
            .title(&title)
            .subtitle(&subtitle)
            .build();

        // Add action icon
        let action_icon = Self::get_action_icon(rule.action);
        expander.add_prefix(&action_icon);

        // Direction icon
        let direction_icon = Self::get_direction_icon(rule.direction);
        expander.add_suffix(&direction_icon);

        // Edit action row
        let edit_row = ActionRow::builder()
            .title("Editar")
            .subtitle("Modificar esta regra")
            .activatable(true)
            .build();

        let edit_icon = Image::from_icon_name("document-edit-symbolic");
        edit_row.add_prefix(&edit_icon);

        edit_row.connect_activated(|_| {
            tracing::info!("Edit rule clicked");
        });

        expander.add_row(&edit_row);

        // Toggle enable/disable
        let enabled_switch = adw::SwitchRow::builder()
            .title("Habilitada")
            .subtitle("Ativar/desativar regra")
            .active(true)
            .build();

        enabled_switch.connect_active_notify(|switch| {
            let enabled = switch.is_active();
            tracing::info!("Rule enabled: {}", enabled);
        });

        expander.add_row(&enabled_switch);

        // Move up/down row
        let order_row = ActionRow::builder()
            .title("Ordem")
            .subtitle("Prioridade da regra")
            .build();

        let order_box = Box::new(Orientation::Horizontal, 4);

        let up_btn = Button::from_icon_name("go-up-symbolic");
        up_btn.add_css_class("flat");
        up_btn.set_tooltip_text(Some("Mover para cima (maior prioridade)"));
        up_btn.connect_clicked(|_| {
            tracing::info!("Move rule up");
        });

        let down_btn = Button::from_icon_name("go-down-symbolic");
        down_btn.add_css_class("flat");
        down_btn.set_tooltip_text(Some("Mover para baixo (menor prioridade)"));
        down_btn.connect_clicked(|_| {
            tracing::info!("Move rule down");
        });

        order_box.append(&up_btn);
        order_box.append(&down_btn);
        order_row.add_suffix(&order_box);

        expander.add_row(&order_row);

        // Details row
        if let Some(ref comment) = rule.comment {
            let comment_row = ActionRow::builder()
                .title("Comentario")
                .subtitle(comment)
                .build();

            let comment_icon = Image::from_icon_name("document-properties-symbolic");
            comment_row.add_prefix(&comment_icon);

            expander.add_row(&comment_row);
        }

        // Delete row
        let delete_row = ActionRow::builder()
            .title("Excluir")
            .subtitle("Remover esta regra permanentemente")
            .activatable(true)
            .build();
        delete_row.add_css_class("error");

        let delete_icon = Image::from_icon_name("user-trash-symbolic");
        delete_row.add_prefix(&delete_icon);

        delete_row.connect_activated(|_| {
            tracing::info!("Delete rule clicked");
        });

        expander.add_row(&delete_row);

        expander
    }

    fn format_rule_title(rule: &FirewallRule) -> String {
        let port_str = rule.port.as_deref().unwrap_or("*");
        let proto_str = match rule.protocol {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
            Protocol::Both => "TCP/UDP",
            Protocol::Any => "*",
        };

        format!("{}/{}", port_str, proto_str)
    }

    fn format_rule_subtitle(rule: &FirewallRule) -> String {
        let action_str = match rule.action {
            Action::Allow => "PERMITIR",
            Action::Deny => "NEGAR",
            Action::Reject => "REJEITAR",
            Action::Limit => "LIMITAR",
        };

        let direction_str = match rule.direction {
            Direction::In => "entrada",
            Direction::Out => "saida",
            Direction::Both => "ambas",
        };

        let from_str = rule.from_ip.as_deref().unwrap_or("qualquer origem");

        format!("{} {} de {}", action_str, direction_str, from_str)
    }

    fn get_action_icon(action: Action) -> Image {
        let (icon_name, css_class) = match action {
            Action::Allow => ("emblem-ok-symbolic", "success"),
            Action::Deny => ("action-unavailable-symbolic", "error"),
            Action::Reject => ("dialog-error-symbolic", "error"),
            Action::Limit => ("speedometer-symbolic", "warning"),
        };

        let icon = Image::from_icon_name(icon_name);
        icon.add_css_class(css_class);
        icon
    }

    fn get_direction_icon(direction: Direction) -> Image {
        let icon_name = match direction {
            Direction::In => "go-down-symbolic",
            Direction::Out => "go-up-symbolic",
            Direction::Both => "network-transmit-receive-symbolic",
        };

        Image::from_icon_name(icon_name)
    }

    /// Get the widget
    pub fn widget(&self) -> &ExpanderRow {
        &self.widget
    }

    /// Get a reference to the rule
    pub fn rule(&self) -> std::cell::Ref<FirewallRule> {
        self.rule.borrow()
    }

    /// Get a mutable reference to the rule
    pub fn rule_mut(&self) -> std::cell::RefMut<FirewallRule> {
        self.rule.borrow_mut()
    }

    /// Update the widget to reflect rule changes
    pub fn refresh(&self) {
        let rule = self.rule.borrow();
        self.widget.set_title(&Self::format_rule_title(&rule));
        self.widget.set_subtitle(&Self::format_rule_subtitle(&rule));
    }
}

/// Create a simple rule row for the rules list
pub fn create_simple_rule_row(
    rule_num: &str,
    action: &str,
    port: &str,
    from: &str,
    comment: Option<&str>,
) -> ActionRow {
    let subtitle = match comment {
        Some(c) => format!("{} from {} - {}", action, from, c),
        None => format!("{} from {}", action, from),
    };

    let row = ActionRow::builder()
        .title(rule_num)
        .subtitle(&subtitle)
        .activatable(true)
        .build();

    // Action indicator
    let (icon_name, css_class) = match action.to_uppercase().as_str() {
        "ALLOW" => ("emblem-ok-symbolic", "success"),
        "DENY" => ("action-unavailable-symbolic", "error"),
        "REJECT" => ("dialog-error-symbolic", "error"),
        _ => ("security-medium-symbolic", "warning"),
    };

    let action_icon = Image::from_icon_name(icon_name);
    action_icon.add_css_class(css_class);
    row.add_prefix(&action_icon);

    // Port label
    let port_label = Label::new(Some(port));
    port_label.add_css_class("dim-label");
    row.add_suffix(&port_label);

    // Navigation arrow
    let arrow = Image::from_icon_name("go-next-symbolic");
    row.add_suffix(&arrow);

    row
}
