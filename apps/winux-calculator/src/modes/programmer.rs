// Winux Calculator - Programmer Mode
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Box, Button, Grid, Label, Orientation, ToggleButton};
use std::cell::RefCell;
use std::rc::Rc;

use crate::engine::Calculator;
use crate::ui::History;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumberBase {
    Decimal,
    Hexadecimal,
    Binary,
    Octal,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WordSize {
    Byte,    // 8 bits
    Word,    // 16 bits
    DWord,   // 32 bits
    QWord,   // 64 bits
}

impl WordSize {
    fn bits(&self) -> u32 {
        match self {
            WordSize::Byte => 8,
            WordSize::Word => 16,
            WordSize::DWord => 32,
            WordSize::QWord => 64,
        }
    }

    fn mask(&self) -> i64 {
        match self {
            WordSize::Byte => 0xFF,
            WordSize::Word => 0xFFFF,
            WordSize::DWord => 0xFFFFFFFF,
            WordSize::QWord => i64::MAX,
        }
    }
}

pub struct ProgrammerMode {
    widget: Box,
    calculator: Rc<RefCell<Calculator>>,
    history: Rc<RefCell<History>>,
    base: Rc<RefCell<NumberBase>>,
    word_size: Rc<RefCell<WordSize>>,
    value: Rc<RefCell<i64>>,
    display_label: Rc<RefCell<Label>>,
    hex_label: Rc<RefCell<Label>>,
    dec_label: Rc<RefCell<Label>>,
    oct_label: Rc<RefCell<Label>>,
    bin_label: Rc<RefCell<Label>>,
}

impl ProgrammerMode {
    pub fn new(calculator: Rc<RefCell<Calculator>>, history: Rc<RefCell<History>>) -> Self {
        let base = Rc::new(RefCell::new(NumberBase::Decimal));
        let word_size = Rc::new(RefCell::new(WordSize::QWord));
        let value = Rc::new(RefCell::new(0i64));

        let widget = Box::new(Orientation::Vertical, 0);
        widget.set_margin_top(12);
        widget.set_margin_bottom(12);
        widget.set_margin_start(12);
        widget.set_margin_end(12);

        // Main display
        let display_label = Rc::new(RefCell::new(Label::new(Some("0"))));
        {
            let label = display_label.borrow();
            label.set_halign(gtk4::Align::End);
            label.set_margin_bottom(6);
            label.add_css_class("title-1");
            label.set_selectable(true);
        }
        widget.append(&display_label.borrow().clone());

        // Multi-base display
        let bases_box = Box::new(Orientation::Vertical, 2);
        bases_box.set_margin_bottom(12);

        let hex_label = Rc::new(RefCell::new(Self::create_base_label("HEX", "0")));
        let dec_label = Rc::new(RefCell::new(Self::create_base_label("DEC", "0")));
        let oct_label = Rc::new(RefCell::new(Self::create_base_label("OCT", "0")));
        let bin_label = Rc::new(RefCell::new(Self::create_base_label("BIN", "0")));

        bases_box.append(&hex_label.borrow().clone());
        bases_box.append(&dec_label.borrow().clone());
        bases_box.append(&oct_label.borrow().clone());
        bases_box.append(&bin_label.borrow().clone());
        widget.append(&bases_box);

        // Word size selector
        let word_box = Self::create_word_size_selector(word_size.clone(), value.clone(),
            display_label.clone(), hex_label.clone(), dec_label.clone(),
            oct_label.clone(), bin_label.clone(), base.clone());
        widget.append(&word_box);

        // Base selector
        let base_box = Self::create_base_selector(base.clone(), display_label.clone(), value.clone());
        widget.append(&base_box);

        // Create keypad
        let keypad = Self::create_keypad(
            calculator.clone(), value.clone(), base.clone(), word_size.clone(),
            display_label.clone(), hex_label.clone(), dec_label.clone(),
            oct_label.clone(), bin_label.clone(), history.clone(),
        );
        widget.append(&keypad);

        Self {
            widget,
            calculator,
            history,
            base,
            word_size,
            value,
            display_label,
            hex_label,
            dec_label,
            oct_label,
            bin_label,
        }
    }

    pub fn widget(&self) -> Box {
        self.widget.clone()
    }

    fn create_base_label(prefix: &str, value: &str) -> Label {
        let label = Label::new(Some(&format!("{}: {}", prefix, value)));
        label.set_halign(gtk4::Align::Start);
        label.add_css_class("caption");
        label.set_selectable(true);
        label
    }

    fn create_word_size_selector(
        word_size: Rc<RefCell<WordSize>>,
        value: Rc<RefCell<i64>>,
        display: Rc<RefCell<Label>>,
        hex: Rc<RefCell<Label>>,
        dec: Rc<RefCell<Label>>,
        oct: Rc<RefCell<Label>>,
        bin: Rc<RefCell<Label>>,
        base: Rc<RefCell<NumberBase>>,
    ) -> Box {
        let word_box = Box::new(Orientation::Horizontal, 6);
        word_box.set_margin_top(6);
        word_box.set_margin_bottom(6);
        word_box.set_halign(gtk4::Align::Center);

        let sizes = [
            ("QWORD", WordSize::QWord),
            ("DWORD", WordSize::DWord),
            ("WORD", WordSize::Word),
            ("BYTE", WordSize::Byte),
        ];

        let mut first_btn: Option<ToggleButton> = None;

        for (label, size) in sizes {
            let btn = ToggleButton::with_label(label);
            if size == WordSize::QWord {
                btn.set_active(true);
            }
            if let Some(ref first) = first_btn {
                btn.set_group(Some(first));
            } else {
                first_btn = Some(btn.clone());
            }

            let ws = word_size.clone();
            let val = value.clone();
            let disp = display.clone();
            let h = hex.clone();
            let d = dec.clone();
            let o = oct.clone();
            let b = bin.clone();
            let bs = base.clone();

            btn.connect_toggled(move |button| {
                if button.is_active() {
                    *ws.borrow_mut() = size;
                    // Mask value to word size
                    let mask = size.mask();
                    let mut v = val.borrow_mut();
                    *v = *v & mask;
                    Self::update_all_displays(&disp, &h, &d, &o, &b, *v, *bs.borrow());
                }
            });

            word_box.append(&btn);
        }

        word_box
    }

    fn create_base_selector(
        base: Rc<RefCell<NumberBase>>,
        display: Rc<RefCell<Label>>,
        value: Rc<RefCell<i64>>,
    ) -> Box {
        let base_box = Box::new(Orientation::Horizontal, 6);
        base_box.set_margin_top(6);
        base_box.set_margin_bottom(6);
        base_box.set_halign(gtk4::Align::Center);

        let bases = [
            ("HEX", NumberBase::Hexadecimal),
            ("DEC", NumberBase::Decimal),
            ("OCT", NumberBase::Octal),
            ("BIN", NumberBase::Binary),
        ];

        let mut first_btn: Option<ToggleButton> = None;

        for (label, b) in bases {
            let btn = ToggleButton::with_label(label);
            if b == NumberBase::Decimal {
                btn.set_active(true);
            }
            if let Some(ref first) = first_btn {
                btn.set_group(Some(first));
            } else {
                first_btn = Some(btn.clone());
            }

            let bs = base.clone();
            let disp = display.clone();
            let val = value.clone();

            btn.connect_toggled(move |button| {
                if button.is_active() {
                    *bs.borrow_mut() = b;
                    // Update main display based on current base
                    let v = *val.borrow();
                    let text = match b {
                        NumberBase::Hexadecimal => format!("{:X}", v),
                        NumberBase::Decimal => format!("{}", v),
                        NumberBase::Octal => format!("{:o}", v),
                        NumberBase::Binary => format!("{:b}", v),
                    };
                    disp.borrow().set_text(&text);
                }
            });

            base_box.append(&btn);
        }

        base_box
    }

    fn create_keypad(
        calculator: Rc<RefCell<Calculator>>,
        value: Rc<RefCell<i64>>,
        base: Rc<RefCell<NumberBase>>,
        word_size: Rc<RefCell<WordSize>>,
        display: Rc<RefCell<Label>>,
        hex: Rc<RefCell<Label>>,
        dec: Rc<RefCell<Label>>,
        oct: Rc<RefCell<Label>>,
        bin: Rc<RefCell<Label>>,
        history: Rc<RefCell<History>>,
    ) -> Grid {
        let grid = Grid::new();
        grid.set_row_spacing(4);
        grid.set_column_spacing(4);
        grid.set_vexpand(true);
        grid.set_margin_top(6);

        // Programmer button layout
        let buttons = [
            // Row 0: Hex digits and operations
            ["A", "B", "AND", "<<", "C"],
            ["C", "D", "OR", ">>", "CE"],
            ["E", "F", "XOR", "NOT", "/"],
            // Row 3: Numbers
            ["7", "8", "9", "MOD", "*"],
            ["4", "5", "6", "(", "-"],
            ["1", "2", "3", ")", "+"],
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
                    "=" => button.add_css_class("suggested-action"),
                    "C" | "CE" => button.add_css_class("destructive-action"),
                    "AND" | "OR" | "XOR" | "NOT" | "<<" | ">>" | "MOD" => {
                        button.add_css_class("flat");
                    }
                    "A" | "B" | "C" | "D" | "E" | "F" => {
                        button.add_css_class("flat");
                    }
                    _ => {}
                }

                // Hex digits A-F need special handling
                let val = value.clone();
                let bs = base.clone();
                let ws = word_size.clone();
                let disp = display.clone();
                let h = hex.clone();
                let d = dec.clone();
                let o = oct.clone();
                let b = bin.clone();
                let hist = history.clone();
                let label_str = label.to_string();

                button.connect_clicked(move |_| {
                    Self::handle_button_click(
                        &label_str, &val, &bs, &ws,
                        &disp, &h, &d, &o, &b, &hist,
                    );
                });

                grid.attach(&button, col as i32, row as i32, 1, 1);
            }
        }

        grid
    }

    fn handle_button_click(
        label: &str,
        value: &Rc<RefCell<i64>>,
        base: &Rc<RefCell<NumberBase>>,
        word_size: &Rc<RefCell<WordSize>>,
        display: &Rc<RefCell<Label>>,
        hex: &Rc<RefCell<Label>>,
        dec: &Rc<RefCell<Label>>,
        oct: &Rc<RefCell<Label>>,
        bin: &Rc<RefCell<Label>>,
        _history: &Rc<RefCell<History>>,
    ) {
        let current_base = *base.borrow();
        let mask = word_size.borrow().mask();

        match label {
            // Hex digits
            "A" | "B" | "C" | "D" | "E" | "F" => {
                if current_base == NumberBase::Hexadecimal {
                    let digit = u8::from_str_radix(label, 16).unwrap() as i64;
                    let mut v = value.borrow_mut();
                    *v = ((*v << 4) | digit) & mask;
                }
            }
            // Decimal digits
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                let digit = label.parse::<i64>().unwrap();
                let mut v = value.borrow_mut();

                match current_base {
                    NumberBase::Decimal => {
                        *v = ((*v * 10) + digit) & mask;
                    }
                    NumberBase::Hexadecimal => {
                        *v = ((*v << 4) | digit) & mask;
                    }
                    NumberBase::Octal => {
                        if digit < 8 {
                            *v = ((*v << 3) | digit) & mask;
                        }
                    }
                    NumberBase::Binary => {
                        if digit < 2 {
                            *v = ((*v << 1) | digit) & mask;
                        }
                    }
                }
            }
            // Clear
            "C" => {
                *value.borrow_mut() = 0;
            }
            "CE" => {
                *value.borrow_mut() = 0;
            }
            // Toggle sign
            "+/-" => {
                let mut v = value.borrow_mut();
                *v = (-*v) & mask;
            }
            // Bitwise operations
            "NOT" => {
                let mut v = value.borrow_mut();
                *v = (!*v) & mask;
            }
            // For binary operations, we'd need a second operand
            // This is a simplified implementation
            "<<" => {
                let mut v = value.borrow_mut();
                *v = (*v << 1) & mask;
            }
            ">>" => {
                let mut v = value.borrow_mut();
                *v = *v >> 1;
            }
            "=" => {
                // Result is already shown
            }
            _ => {}
        }

        let v = *value.borrow();
        Self::update_all_displays(display, hex, dec, oct, bin, v, current_base);
    }

    fn update_all_displays(
        display: &Rc<RefCell<Label>>,
        hex: &Rc<RefCell<Label>>,
        dec: &Rc<RefCell<Label>>,
        oct: &Rc<RefCell<Label>>,
        bin: &Rc<RefCell<Label>>,
        value: i64,
        current_base: NumberBase,
    ) {
        // Update main display based on current base
        let main_text = match current_base {
            NumberBase::Hexadecimal => format!("{:X}", value),
            NumberBase::Decimal => format!("{}", value),
            NumberBase::Octal => format!("{:o}", value),
            NumberBase::Binary => format!("{:b}", value),
        };
        display.borrow().set_text(&main_text);

        // Update all base displays
        hex.borrow().set_text(&format!("HEX: {:X}", value));
        dec.borrow().set_text(&format!("DEC: {}", value));
        oct.borrow().set_text(&format!("OCT: {:o}", value));
        bin.borrow().set_text(&format!("BIN: {:b}", value));
    }
}
