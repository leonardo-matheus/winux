// Winux Calculator - Scientific Mode
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Box, Button, Grid, Label, Orientation, ToggleButton};
use std::cell::RefCell;
use std::rc::Rc;

use crate::engine::{Calculator, AngleMode};
use crate::ui::{Display, History};

pub struct ScientificMode {
    widget: Box,
    display: Rc<RefCell<Display>>,
    calculator: Rc<RefCell<Calculator>>,
    history: Rc<RefCell<History>>,
}

impl ScientificMode {
    pub fn new(calculator: Rc<RefCell<Calculator>>, history: Rc<RefCell<History>>) -> Self {
        let display = Rc::new(RefCell::new(Display::new()));

        let widget = Box::new(Orientation::Vertical, 0);
        widget.set_margin_top(12);
        widget.set_margin_bottom(12);
        widget.set_margin_start(12);
        widget.set_margin_end(12);

        // Add display
        widget.append(&display.borrow().widget());

        // Angle mode selector
        let angle_box = Self::create_angle_selector(calculator.clone());
        widget.append(&angle_box);

        // Create keypad
        let keypad = Self::create_keypad(calculator.clone(), display.clone(), history.clone());
        widget.append(&keypad);

        Self {
            widget,
            display,
            calculator,
            history,
        }
    }

    pub fn widget(&self) -> Box {
        self.widget.clone()
    }

    fn create_angle_selector(calculator: Rc<RefCell<Calculator>>) -> Box {
        let angle_box = Box::new(Orientation::Horizontal, 6);
        angle_box.set_margin_top(6);
        angle_box.set_margin_bottom(6);
        angle_box.set_halign(gtk4::Align::Center);

        let deg_btn = ToggleButton::with_label("DEG");
        deg_btn.set_active(true);

        let rad_btn = ToggleButton::with_label("RAD");
        rad_btn.set_group(Some(&deg_btn));

        let calc_deg = calculator.clone();
        deg_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                calc_deg.borrow_mut().angle_mode = AngleMode::Degrees;
            }
        });

        let calc_rad = calculator.clone();
        rad_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                calc_rad.borrow_mut().angle_mode = AngleMode::Radians;
            }
        });

        angle_box.append(&deg_btn);
        angle_box.append(&rad_btn);

        angle_box
    }

    fn create_keypad(
        calculator: Rc<RefCell<Calculator>>,
        display: Rc<RefCell<Display>>,
        history: Rc<RefCell<History>>,
    ) -> Grid {
        let grid = Grid::new();
        grid.set_row_spacing(4);
        grid.set_column_spacing(4);
        grid.set_vexpand(true);
        grid.set_margin_top(6);

        // Scientific button layout
        let buttons = [
            // Row 0: Scientific functions
            ["sin", "cos", "tan", "(", ")", "C"],
            // Row 1: Inverse trig and power
            ["asin", "acos", "atan", "x^2", "x^y", "CE"],
            // Row 2: Log functions
            ["ln", "log", "e^x", "sqrt", "1/x", "/"],
            // Row 3: Constants and numbers
            ["pi", "e", "7", "8", "9", "*"],
            // Row 4: Factorial and numbers
            ["n!", "abs", "4", "5", "6", "-"],
            // Row 5: More operations
            ["EXP", "+/-", "1", "2", "3", "+"],
            // Row 6: Zero row
            ["(", ")", "0", ".", "=", ""],
        ];

        for (row, row_buttons) in buttons.iter().enumerate() {
            for (col, label) in row_buttons.iter().enumerate() {
                if label.is_empty() {
                    continue;
                }

                let button = Button::with_label(label);
                button.set_hexpand(true);
                button.set_vexpand(true);

                // Style buttons
                match *label {
                    "=" => {
                        button.add_css_class("suggested-action");
                    }
                    "C" | "CE" => {
                        button.add_css_class("destructive-action");
                    }
                    "+" | "-" | "*" | "/" => {
                        button.add_css_class("accent");
                    }
                    "sin" | "cos" | "tan" | "asin" | "acos" | "atan" |
                    "ln" | "log" | "e^x" | "sqrt" | "1/x" | "x^2" | "x^y" |
                    "pi" | "e" | "n!" | "abs" | "EXP" => {
                        button.add_css_class("flat");
                    }
                    _ => {}
                }

                let calc = calculator.clone();
                let disp = display.clone();
                let hist = history.clone();
                let label_str = label.to_string();

                button.connect_clicked(move |_| {
                    Self::handle_button_click(&label_str, &calc, &disp, &hist);
                });

                grid.attach(&button, col as i32, row as i32, 1, 1);
            }
        }

        grid
    }

    fn handle_button_click(
        label: &str,
        calculator: &Rc<RefCell<Calculator>>,
        display: &Rc<RefCell<Display>>,
        history: &Rc<RefCell<History>>,
    ) {
        let mut calc = calculator.borrow_mut();

        match label {
            // Digits
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." => {
                calc.input_digit(label.chars().next().unwrap());
            }
            // Operators
            "+" => calc.input_operator("+"),
            "-" => calc.input_operator("-"),
            "*" => calc.input_operator("*"),
            "/" => calc.input_operator("/"),
            "x^y" => calc.input_operator("^"),
            // Actions
            "=" => {
                let _ = calc.calculate();
                history.borrow_mut().update(&calc);
            }
            "C" => calc.clear_all(),
            "CE" => calc.clear_entry(),
            "+/-" => calc.toggle_sign(),
            // Parentheses
            "(" => {
                calc.expression.push('(');
            }
            ")" => {
                calc.expression.push_str(&calc.display);
                calc.expression.push(')');
                calc.display = "0".to_string();
            }
            // Trigonometric functions
            "sin" => calc.apply_function("sin"),
            "cos" => calc.apply_function("cos"),
            "tan" => calc.apply_function("tan"),
            "asin" => calc.apply_function("asin"),
            "acos" => calc.apply_function("acos"),
            "atan" => calc.apply_function("atan"),
            // Logarithmic functions
            "ln" => calc.apply_function("ln"),
            "log" => calc.apply_function("log10"),
            "e^x" => calc.apply_function("exp"),
            // Power and root
            "sqrt" => calc.apply_function("sqrt"),
            "x^2" => calc.apply_function("sq"),
            "1/x" => calc.apply_function("inv"),
            // Constants
            "pi" => calc.insert_constant("pi"),
            "e" => calc.insert_constant("e"),
            // Other functions
            "n!" => calc.apply_function("fact"),
            "abs" => calc.apply_function("abs"),
            "EXP" => {
                // Scientific notation input
                if !calc.display.contains('e') {
                    calc.display.push_str("e");
                }
            }
            _ => {}
        }

        display.borrow().update(&calc);
    }
}
