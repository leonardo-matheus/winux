// Winux Calculator - Keypad Component
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Button, Grid};

/// Reusable keypad component
pub struct Keypad {
    widget: Grid,
}

impl Keypad {
    pub fn new() -> Self {
        let widget = Grid::new();
        widget.set_row_spacing(6);
        widget.set_column_spacing(6);
        widget.set_vexpand(true);
        widget.set_margin_top(12);

        Self { widget }
    }

    pub fn widget(&self) -> Grid {
        self.widget.clone()
    }

    /// Add a button to the keypad
    pub fn add_button(
        &self,
        label: &str,
        row: i32,
        col: i32,
        row_span: i32,
        col_span: i32,
    ) -> Button {
        let button = Button::with_label(label);
        button.set_hexpand(true);
        button.set_vexpand(true);

        self.widget.attach(&button, col, row, col_span, row_span);
        button
    }

    /// Add a button with an icon
    pub fn add_icon_button(
        &self,
        icon_name: &str,
        tooltip: &str,
        row: i32,
        col: i32,
    ) -> Button {
        let button = Button::from_icon_name(icon_name);
        button.set_tooltip_text(Some(tooltip));
        button.set_hexpand(true);
        button.set_vexpand(true);

        self.widget.attach(&button, col, row, 1, 1);
        button
    }

    /// Style a button as primary action
    pub fn style_primary(button: &Button) {
        button.add_css_class("suggested-action");
    }

    /// Style a button as destructive action
    pub fn style_destructive(button: &Button) {
        button.add_css_class("destructive-action");
    }

    /// Style a button as operator
    pub fn style_operator(button: &Button) {
        button.add_css_class("accent");
    }

    /// Style a button as flat
    pub fn style_flat(button: &Button) {
        button.add_css_class("flat");
    }
}

impl Default for Keypad {
    fn default() -> Self {
        Self::new()
    }
}

/// Button types for calculator
#[derive(Clone, Debug, PartialEq)]
pub enum ButtonType {
    Digit(char),
    Operator(String),
    Function(String),
    Action(String),
    Memory(String),
}

impl ButtonType {
    pub fn from_label(label: &str) -> Self {
        match label {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." => {
                ButtonType::Digit(label.chars().next().unwrap())
            }
            "+" | "-" | "*" | "/" | "%" | "^" => {
                ButtonType::Operator(label.to_string())
            }
            "sin" | "cos" | "tan" | "log" | "ln" | "sqrt" | "abs" => {
                ButtonType::Function(label.to_string())
            }
            "MC" | "MR" | "M+" | "M-" | "MS" => {
                ButtonType::Memory(label.to_string())
            }
            _ => ButtonType::Action(label.to_string()),
        }
    }
}
