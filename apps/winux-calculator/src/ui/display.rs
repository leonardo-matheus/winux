// Winux Calculator - Display Component
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use crate::engine::Calculator;

/// Calculator display showing current expression and result
pub struct Display {
    widget: Box,
    expression_label: Label,
    result_label: Label,
    memory_indicator: Label,
}

impl Display {
    pub fn new() -> Self {
        let widget = Box::new(Orientation::Vertical, 4);
        widget.set_margin_top(12);
        widget.set_margin_bottom(12);
        widget.set_margin_start(12);
        widget.set_margin_end(12);

        // Memory indicator (shown when memory has value)
        let memory_indicator = Label::new(Some(""));
        memory_indicator.set_halign(gtk4::Align::Start);
        memory_indicator.add_css_class("dim-label");
        memory_indicator.add_css_class("caption");
        widget.append(&memory_indicator);

        // Expression label (smaller, shows the ongoing calculation)
        let expression_label = Label::new(Some(""));
        expression_label.set_halign(gtk4::Align::End);
        expression_label.set_selectable(true);
        expression_label.add_css_class("dim-label");
        expression_label.set_wrap(true);
        expression_label.set_wrap_mode(gtk4::pango::WrapMode::Char);
        widget.append(&expression_label);

        // Result label (larger, shows current value)
        let result_label = Label::new(Some("0"));
        result_label.set_halign(gtk4::Align::End);
        result_label.set_selectable(true);
        result_label.add_css_class("title-1");
        result_label.set_wrap(true);
        result_label.set_wrap_mode(gtk4::pango::WrapMode::Char);
        widget.append(&result_label);

        // Add some styling via CSS
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_string(
            r#"
            .calculator-display {
                background-color: @card_bg_color;
                border-radius: 12px;
                padding: 12px;
            }
            "#
        );

        widget.add_css_class("calculator-display");

        Self {
            widget,
            expression_label,
            result_label,
            memory_indicator,
        }
    }

    pub fn widget(&self) -> Box {
        self.widget.clone()
    }

    /// Update display from calculator state
    pub fn update(&self, calc: &Calculator) {
        // Update expression
        let expr = calc.get_expression();
        self.expression_label.set_text(expr);

        // Update result
        self.result_label.set_text(calc.get_display());

        // Update memory indicator
        if calc.memory != 0.0 {
            self.memory_indicator.set_text("M");
        } else {
            self.memory_indicator.set_text("");
        }
    }

    /// Set the result text directly
    pub fn set_result(&self, text: &str) {
        self.result_label.set_text(text);
    }

    /// Set the expression text directly
    pub fn set_expression(&self, text: &str) {
        self.expression_label.set_text(text);
    }

    /// Get current result text
    pub fn get_result(&self) -> String {
        self.result_label.text().to_string()
    }

    /// Show error state
    pub fn show_error(&self, message: &str) {
        self.result_label.set_text(message);
        self.result_label.add_css_class("error");
    }

    /// Clear error state
    pub fn clear_error(&self) {
        self.result_label.remove_css_class("error");
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}
