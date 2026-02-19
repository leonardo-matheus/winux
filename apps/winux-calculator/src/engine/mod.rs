// Winux Calculator - Engine Module
// Copyright (c) 2026 Winux OS Project

mod parser;
mod evaluator;
mod functions;

pub use parser::Parser;
pub use evaluator::Evaluator;
pub use functions::MathFunctions;

use std::collections::VecDeque;

/// Main calculator state and operations
pub struct Calculator {
    /// Current display value
    pub display: String,
    /// Current expression being built
    pub expression: String,
    /// Memory value
    pub memory: f64,
    /// Calculation history
    pub history: VecDeque<HistoryEntry>,
    /// Parser instance
    parser: Parser,
    /// Evaluator instance
    evaluator: Evaluator,
    /// Last result
    pub last_result: Option<f64>,
    /// Error state
    pub error: Option<String>,
    /// Angle mode (degrees or radians)
    pub angle_mode: AngleMode,
}

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AngleMode {
    Degrees,
    Radians,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            display: "0".to_string(),
            expression: String::new(),
            memory: 0.0,
            history: VecDeque::with_capacity(100),
            parser: Parser::new(),
            evaluator: Evaluator::new(),
            last_result: None,
            error: None,
            angle_mode: AngleMode::Degrees,
        }
    }

    /// Clear everything (AC)
    pub fn clear_all(&mut self) {
        self.display = "0".to_string();
        self.expression.clear();
        self.last_result = None;
        self.error = None;
    }

    /// Clear entry (CE) - just the display
    pub fn clear_entry(&mut self) {
        self.display = "0".to_string();
        self.error = None;
    }

    /// Append a digit or decimal point
    pub fn input_digit(&mut self, digit: char) {
        if self.error.is_some() {
            self.clear_all();
        }

        if digit == '.' && self.display.contains('.') {
            return;
        }

        if self.display == "0" && digit != '.' {
            self.display = digit.to_string();
        } else {
            self.display.push(digit);
        }
    }

    /// Input an operator
    pub fn input_operator(&mut self, op: &str) {
        if self.error.is_some() {
            return;
        }

        if !self.expression.is_empty() || self.last_result.is_some() {
            if let Some(result) = self.last_result {
                self.expression = format!("{} {} ", result, op);
            } else {
                self.expression.push_str(&format!("{} {} ", self.display, op));
            }
        } else {
            self.expression = format!("{} {} ", self.display, op);
        }

        self.display = "0".to_string();
        self.last_result = None;
    }

    /// Calculate the result
    pub fn calculate(&mut self) -> Result<f64, String> {
        if self.error.is_some() {
            return Err(self.error.clone().unwrap());
        }

        let full_expression = if self.expression.is_empty() {
            self.display.clone()
        } else {
            format!("{}{}", self.expression, self.display)
        };

        match self.parser.parse(&full_expression) {
            Ok(tokens) => {
                match self.evaluator.evaluate(&tokens, self.angle_mode) {
                    Ok(result) => {
                        // Add to history
                        self.history.push_front(HistoryEntry {
                            expression: full_expression.clone(),
                            result: format_number(result),
                        });

                        // Keep history limited
                        while self.history.len() > 100 {
                            self.history.pop_back();
                        }

                        self.display = format_number(result);
                        self.expression.clear();
                        self.last_result = Some(result);
                        self.error = None;
                        Ok(result)
                    }
                    Err(e) => {
                        self.error = Some(e.clone());
                        self.display = "Erro".to_string();
                        Err(e)
                    }
                }
            }
            Err(e) => {
                self.error = Some(e.clone());
                self.display = "Erro".to_string();
                Err(e)
            }
        }
    }

    /// Calculate percentage
    pub fn percentage(&mut self) {
        if let Ok(value) = self.display.parse::<f64>() {
            let result = value / 100.0;
            self.display = format_number(result);
        }
    }

    /// Toggle sign (+/-)
    pub fn toggle_sign(&mut self) {
        if self.display.starts_with('-') {
            self.display = self.display[1..].to_string();
        } else if self.display != "0" {
            self.display = format!("-{}", self.display);
        }
    }

    /// Backspace - remove last character
    pub fn backspace(&mut self) {
        if self.error.is_some() {
            self.clear_all();
            return;
        }

        if self.display.len() > 1 {
            self.display.pop();
        } else {
            self.display = "0".to_string();
        }
    }

    // Memory operations
    pub fn memory_clear(&mut self) {
        self.memory = 0.0;
    }

    pub fn memory_recall(&mut self) {
        self.display = format_number(self.memory);
    }

    pub fn memory_add(&mut self) {
        if let Ok(value) = self.display.parse::<f64>() {
            self.memory += value;
        }
    }

    pub fn memory_subtract(&mut self) {
        if let Ok(value) = self.display.parse::<f64>() {
            self.memory -= value;
        }
    }

    pub fn memory_store(&mut self) {
        if let Ok(value) = self.display.parse::<f64>() {
            self.memory = value;
        }
    }

    /// Apply a function (sin, cos, etc.)
    pub fn apply_function(&mut self, func: &str) {
        if let Ok(value) = self.display.parse::<f64>() {
            let result = MathFunctions::apply(func, value, self.angle_mode);
            match result {
                Ok(r) => {
                    self.display = format_number(r);
                    self.last_result = Some(r);
                }
                Err(e) => {
                    self.error = Some(e);
                    self.display = "Erro".to_string();
                }
            }
        }
    }

    /// Insert a constant
    pub fn insert_constant(&mut self, name: &str) {
        let value = match name {
            "pi" | "PI" => std::f64::consts::PI,
            "e" | "E" => std::f64::consts::E,
            "phi" => 1.618033988749895, // Golden ratio
            _ => return,
        };
        self.display = format_number(value);
    }

    /// Get current display value
    pub fn get_display(&self) -> &str {
        &self.display
    }

    /// Get current expression
    pub fn get_expression(&self) -> &str {
        &self.expression
    }

    /// Set display directly (for programmer mode)
    pub fn set_display(&mut self, value: &str) {
        self.display = value.to_string();
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a number for display
pub fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "Erro".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() { "Infinito" } else { "-Infinito" }.to_string();
    }

    // Check if it's effectively an integer
    if value.fract().abs() < 1e-10 && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }

    // For very large or very small numbers, use scientific notation
    if value.abs() >= 1e10 || (value.abs() < 1e-6 && value != 0.0) {
        format!("{:.6e}", value)
    } else {
        // Regular decimal, remove trailing zeros
        let s = format!("{:.10}", value);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}
