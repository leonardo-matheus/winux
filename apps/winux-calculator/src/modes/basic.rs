// Winux Calculator - Basic Mode
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Box, Button, Grid, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

use crate::engine::Calculator;
use crate::ui::{Display, History};

pub struct BasicMode {
    widget: Box,
    display: Rc<RefCell<Display>>,
    calculator: Rc<RefCell<Calculator>>,
    history: Rc<RefCell<History>>,
}

impl BasicMode {
    pub fn new(calculator: Rc<RefCell<Calculator>>, history: Rc<RefCell<History>>) -> Self {
        let display = Rc::new(RefCell::new(Display::new()));

        let widget = Box::new(Orientation::Vertical, 0);
        widget.set_margin_top(12);
        widget.set_margin_bottom(12);
        widget.set_margin_start(12);
        widget.set_margin_end(12);

        // Add display
        widget.append(&display.borrow().widget());

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

    fn create_keypad(
        calculator: Rc<RefCell<Calculator>>,
        display: Rc<RefCell<Display>>,
        history: Rc<RefCell<History>>,
    ) -> Grid {
        let grid = Grid::new();
        grid.set_row_spacing(6);
        grid.set_column_spacing(6);
        grid.set_vexpand(true);
        grid.set_margin_top(12);

        // Button layout
        let buttons = [
            // Row 0: Memory operations
            ["MC", "MR", "M+", "M-", "MS"],
            // Row 1: Clear and operations
            ["C", "CE", "%", "/", ""],
            // Row 2-4: Numbers and operations
            ["7", "8", "9", "*", ""],
            ["4", "5", "6", "-", ""],
            ["1", "2", "3", "+", ""],
            // Row 5: Zero, decimal, equals
            ["+/-", "0", ".", "=", ""],
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
                    "+" | "-" | "*" | "/" | "%" => {
                        button.add_css_class("accent");
                    }
                    "MC" | "MR" | "M+" | "M-" | "MS" => {
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

                // Special handling for = button (spans 2 rows on right side)
                if *label == "=" {
                    grid.attach(&button, col as i32, row as i32, 1, 1);
                } else {
                    grid.attach(&button, col as i32, row as i32, 1, 1);
                }
            }
        }

        // Add backspace button
        let backspace = Button::from_icon_name("edit-clear-symbolic");
        backspace.set_tooltip_text(Some("Apagar"));
        backspace.set_hexpand(true);
        backspace.set_vexpand(true);
        backspace.add_css_class("flat");

        let calc = calculator.clone();
        let disp = display.clone();
        backspace.connect_clicked(move |_| {
            calc.borrow_mut().backspace();
            disp.borrow().update(&calc.borrow());
        });

        grid.attach(&backspace, 4, 1, 1, 1);

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
            "%" => calc.percentage(),
            // Actions
            "=" => {
                let _ = calc.calculate();
                history.borrow_mut().update(&calc);
            }
            "C" => calc.clear_all(),
            "CE" => calc.clear_entry(),
            "+/-" => calc.toggle_sign(),
            // Memory
            "MC" => calc.memory_clear(),
            "MR" => calc.memory_recall(),
            "M+" => calc.memory_add(),
            "M-" => calc.memory_subtract(),
            "MS" => calc.memory_store(),
            _ => {}
        }

        display.borrow().update(&calc);
    }
}
