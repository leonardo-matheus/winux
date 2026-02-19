// Winux Calculator - Converter Mode
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Grid, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ComboRow, PreferencesGroup, PreferencesPage, ActionRow};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConverterCategory {
    Length,
    Weight,
    Temperature,
    Area,
    Volume,
    Time,
    Speed,
    Data,
}

pub struct ConverterMode {
    widget: ScrolledWindow,
    category: Rc<RefCell<ConverterCategory>>,
    from_value: Rc<RefCell<f64>>,
    from_unit: Rc<RefCell<usize>>,
    to_unit: Rc<RefCell<usize>>,
}

impl ConverterMode {
    pub fn new() -> Self {
        let category = Rc::new(RefCell::new(ConverterCategory::Length));
        let from_value = Rc::new(RefCell::new(0.0));
        let from_unit = Rc::new(RefCell::new(0));
        let to_unit = Rc::new(RefCell::new(1));

        let page = PreferencesPage::new();

        // Category selector
        let cat_group = PreferencesGroup::builder()
            .title("Categoria")
            .build();

        let categories = [
            ("Comprimento", ConverterCategory::Length),
            ("Peso/Massa", ConverterCategory::Weight),
            ("Temperatura", ConverterCategory::Temperature),
            ("Area", ConverterCategory::Area),
            ("Volume", ConverterCategory::Volume),
            ("Tempo", ConverterCategory::Time),
            ("Velocidade", ConverterCategory::Speed),
            ("Dados", ConverterCategory::Data),
        ];

        let cat_row = ComboRow::builder()
            .title("Tipo de Conversao")
            .build();
        let cat_model = gtk4::StringList::new(&categories.iter().map(|(s, _)| *s).collect::<Vec<_>>());
        cat_row.set_model(Some(&cat_model));
        cat_group.add(&cat_row);
        page.add(&cat_group);

        // From/To group
        let conv_group = PreferencesGroup::builder()
            .title("Conversao")
            .build();

        // From row
        let from_row = ActionRow::builder()
            .title("De")
            .build();
        let from_entry = Entry::builder()
            .placeholder_text("0")
            .hexpand(true)
            .build();
        from_entry.set_input_purpose(gtk4::InputPurpose::Number);
        from_row.add_suffix(&from_entry);
        conv_group.add(&from_row);

        // From unit selector
        let from_unit_row = ComboRow::builder()
            .title("Unidade de Origem")
            .build();
        let from_unit_model = Self::get_units_for_category(ConverterCategory::Length);
        from_unit_row.set_model(Some(&from_unit_model));
        conv_group.add(&from_unit_row);

        // Swap button row
        let swap_row = ActionRow::builder()
            .title("Trocar")
            .activatable(true)
            .build();
        swap_row.add_suffix(&gtk4::Image::from_icon_name("object-flip-vertical-symbolic"));
        conv_group.add(&swap_row);

        // To unit selector
        let to_unit_row = ComboRow::builder()
            .title("Unidade de Destino")
            .build();
        let to_unit_model = Self::get_units_for_category(ConverterCategory::Length);
        to_unit_row.set_model(Some(&to_unit_model));
        to_unit_row.set_selected(1);
        conv_group.add(&to_unit_row);

        // Result row
        let result_row = ActionRow::builder()
            .title("Resultado")
            .subtitle("0")
            .build();
        let result_label = Label::new(Some("0"));
        result_label.add_css_class("title-2");
        result_label.set_selectable(true);
        result_row.add_suffix(&result_label);
        conv_group.add(&result_row);

        page.add(&conv_group);

        // Quick reference group
        let ref_group = PreferencesGroup::builder()
            .title("Referencia Rapida")
            .description("Conversoes comuns")
            .build();

        page.add(&ref_group);

        // Connect signals
        let cat = category.clone();
        let from_unit_row_clone = from_unit_row.clone();
        let to_unit_row_clone = to_unit_row.clone();

        cat_row.connect_selected_notify(move |row| {
            let idx = row.selected();
            let new_cat = categories[idx as usize].1;
            *cat.borrow_mut() = new_cat;

            // Update unit models
            let units = Self::get_units_for_category(new_cat);
            from_unit_row_clone.set_model(Some(&units));
            to_unit_row_clone.set_model(Some(&units.clone()));
            from_unit_row_clone.set_selected(0);
            to_unit_row_clone.set_selected(1);
        });

        // Convert on value change
        let result = result_label.clone();
        let from_val = from_value.clone();
        let from_u = from_unit.clone();
        let to_u = to_unit.clone();
        let cat_ref = category.clone();
        let from_unit_row_ref = from_unit_row.clone();
        let to_unit_row_ref = to_unit_row.clone();

        from_entry.connect_changed(move |entry| {
            let text = entry.text();
            if let Ok(val) = text.parse::<f64>() {
                *from_val.borrow_mut() = val;
                let cat = *cat_ref.borrow();
                let from_idx = from_unit_row_ref.selected() as usize;
                let to_idx = to_unit_row_ref.selected() as usize;
                let converted = Self::convert(val, cat, from_idx, to_idx);
                result.set_text(&format!("{:.6}", converted).trim_end_matches('0').trim_end_matches('.'));
            }
        });

        // Update on unit change
        let result2 = result_label.clone();
        let from_val2 = from_value.clone();
        let cat_ref2 = category.clone();
        let to_unit_row_ref2 = to_unit_row.clone();

        from_unit_row.connect_selected_notify(move |row| {
            let val = *from_val2.borrow();
            let cat = *cat_ref2.borrow();
            let from_idx = row.selected() as usize;
            let to_idx = to_unit_row_ref2.selected() as usize;
            let converted = Self::convert(val, cat, from_idx, to_idx);
            result2.set_text(&format!("{:.6}", converted).trim_end_matches('0').trim_end_matches('.'));
        });

        let result3 = result_label.clone();
        let from_val3 = from_value.clone();
        let cat_ref3 = category.clone();
        let from_unit_row_ref3 = from_unit_row.clone();

        to_unit_row.connect_selected_notify(move |row| {
            let val = *from_val3.borrow();
            let cat = *cat_ref3.borrow();
            let from_idx = from_unit_row_ref3.selected() as usize;
            let to_idx = row.selected() as usize;
            let converted = Self::convert(val, cat, from_idx, to_idx);
            result3.set_text(&format!("{:.6}", converted).trim_end_matches('0').trim_end_matches('.'));
        });

        // Swap button action
        let from_unit_row_swap = from_unit_row.clone();
        let to_unit_row_swap = to_unit_row.clone();

        swap_row.connect_activated(move |_| {
            let from = from_unit_row_swap.selected();
            let to = to_unit_row_swap.selected();
            from_unit_row_swap.set_selected(to);
            to_unit_row_swap.set_selected(from);
        });

        let widget = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget,
            category,
            from_value,
            from_unit,
            to_unit,
        }
    }

    pub fn widget(&self) -> ScrolledWindow {
        self.widget.clone()
    }

    fn get_units_for_category(category: ConverterCategory) -> gtk4::StringList {
        let units: Vec<&str> = match category {
            ConverterCategory::Length => vec![
                "Metros (m)",
                "Quilometros (km)",
                "Centimetros (cm)",
                "Milimetros (mm)",
                "Milhas (mi)",
                "Jardas (yd)",
                "Pes (ft)",
                "Polegadas (in)",
                "Milhas Nauticas",
            ],
            ConverterCategory::Weight => vec![
                "Gramas (g)",
                "Quilogramas (kg)",
                "Miligramas (mg)",
                "Toneladas (t)",
                "Libras (lb)",
                "Oncas (oz)",
                "Pedras (st)",
            ],
            ConverterCategory::Temperature => vec![
                "Celsius (C)",
                "Fahrenheit (F)",
                "Kelvin (K)",
            ],
            ConverterCategory::Area => vec![
                "Metros Quadrados (m2)",
                "Quilometros Quadrados (km2)",
                "Hectares (ha)",
                "Acres",
                "Pes Quadrados (ft2)",
                "Polegadas Quadradas (in2)",
            ],
            ConverterCategory::Volume => vec![
                "Litros (L)",
                "Mililitros (mL)",
                "Metros Cubicos (m3)",
                "Galoes (US)",
                "Galoes (UK)",
                "Quartos (US)",
                "Pintas (US)",
                "Copos",
            ],
            ConverterCategory::Time => vec![
                "Segundos (s)",
                "Minutos (min)",
                "Horas (h)",
                "Dias",
                "Semanas",
                "Meses",
                "Anos",
                "Milissegundos (ms)",
            ],
            ConverterCategory::Speed => vec![
                "Metros por Segundo (m/s)",
                "Quilometros por Hora (km/h)",
                "Milhas por Hora (mph)",
                "Nos (kn)",
                "Pes por Segundo (ft/s)",
            ],
            ConverterCategory::Data => vec![
                "Bytes (B)",
                "Kilobytes (KB)",
                "Megabytes (MB)",
                "Gigabytes (GB)",
                "Terabytes (TB)",
                "Bits (b)",
                "Kilobits (Kb)",
                "Megabits (Mb)",
            ],
        };

        gtk4::StringList::new(&units)
    }

    fn convert(value: f64, category: ConverterCategory, from_idx: usize, to_idx: usize) -> f64 {
        // First convert to base unit, then to target unit
        let in_base = Self::to_base_unit(value, category, from_idx);
        Self::from_base_unit(in_base, category, to_idx)
    }

    fn to_base_unit(value: f64, category: ConverterCategory, unit_idx: usize) -> f64 {
        match category {
            ConverterCategory::Length => {
                // Base unit: meters
                match unit_idx {
                    0 => value,                    // m
                    1 => value * 1000.0,           // km
                    2 => value * 0.01,             // cm
                    3 => value * 0.001,            // mm
                    4 => value * 1609.344,         // mi
                    5 => value * 0.9144,           // yd
                    6 => value * 0.3048,           // ft
                    7 => value * 0.0254,           // in
                    8 => value * 1852.0,           // nautical mile
                    _ => value,
                }
            }
            ConverterCategory::Weight => {
                // Base unit: grams
                match unit_idx {
                    0 => value,                    // g
                    1 => value * 1000.0,           // kg
                    2 => value * 0.001,            // mg
                    3 => value * 1_000_000.0,      // t
                    4 => value * 453.592,          // lb
                    5 => value * 28.3495,          // oz
                    6 => value * 6350.29,          // st
                    _ => value,
                }
            }
            ConverterCategory::Temperature => {
                // Base unit: Celsius
                match unit_idx {
                    0 => value,                         // C
                    1 => (value - 32.0) * 5.0 / 9.0,   // F
                    2 => value - 273.15,                // K
                    _ => value,
                }
            }
            ConverterCategory::Area => {
                // Base unit: square meters
                match unit_idx {
                    0 => value,                         // m2
                    1 => value * 1_000_000.0,          // km2
                    2 => value * 10_000.0,             // ha
                    3 => value * 4046.86,              // acres
                    4 => value * 0.092903,             // ft2
                    5 => value * 0.00064516,           // in2
                    _ => value,
                }
            }
            ConverterCategory::Volume => {
                // Base unit: liters
                match unit_idx {
                    0 => value,                    // L
                    1 => value * 0.001,            // mL
                    2 => value * 1000.0,           // m3
                    3 => value * 3.78541,          // gal US
                    4 => value * 4.54609,          // gal UK
                    5 => value * 0.946353,         // qt US
                    6 => value * 0.473176,         // pt US
                    7 => value * 0.24,             // cups
                    _ => value,
                }
            }
            ConverterCategory::Time => {
                // Base unit: seconds
                match unit_idx {
                    0 => value,                    // s
                    1 => value * 60.0,             // min
                    2 => value * 3600.0,           // h
                    3 => value * 86400.0,          // days
                    4 => value * 604800.0,         // weeks
                    5 => value * 2_629_746.0,      // months (avg)
                    6 => value * 31_556_952.0,     // years
                    7 => value * 0.001,            // ms
                    _ => value,
                }
            }
            ConverterCategory::Speed => {
                // Base unit: m/s
                match unit_idx {
                    0 => value,                    // m/s
                    1 => value / 3.6,              // km/h
                    2 => value * 0.44704,          // mph
                    3 => value * 0.514444,         // knots
                    4 => value * 0.3048,           // ft/s
                    _ => value,
                }
            }
            ConverterCategory::Data => {
                // Base unit: bytes
                match unit_idx {
                    0 => value,                         // B
                    1 => value * 1024.0,               // KB
                    2 => value * 1_048_576.0,          // MB
                    3 => value * 1_073_741_824.0,      // GB
                    4 => value * 1_099_511_627_776.0,  // TB
                    5 => value / 8.0,                  // bits
                    6 => value * 128.0,                // Kb
                    7 => value * 131_072.0,            // Mb
                    _ => value,
                }
            }
        }
    }

    fn from_base_unit(value: f64, category: ConverterCategory, unit_idx: usize) -> f64 {
        match category {
            ConverterCategory::Length => {
                match unit_idx {
                    0 => value,
                    1 => value / 1000.0,
                    2 => value / 0.01,
                    3 => value / 0.001,
                    4 => value / 1609.344,
                    5 => value / 0.9144,
                    6 => value / 0.3048,
                    7 => value / 0.0254,
                    8 => value / 1852.0,
                    _ => value,
                }
            }
            ConverterCategory::Weight => {
                match unit_idx {
                    0 => value,
                    1 => value / 1000.0,
                    2 => value / 0.001,
                    3 => value / 1_000_000.0,
                    4 => value / 453.592,
                    5 => value / 28.3495,
                    6 => value / 6350.29,
                    _ => value,
                }
            }
            ConverterCategory::Temperature => {
                match unit_idx {
                    0 => value,
                    1 => value * 9.0 / 5.0 + 32.0,
                    2 => value + 273.15,
                    _ => value,
                }
            }
            ConverterCategory::Area => {
                match unit_idx {
                    0 => value,
                    1 => value / 1_000_000.0,
                    2 => value / 10_000.0,
                    3 => value / 4046.86,
                    4 => value / 0.092903,
                    5 => value / 0.00064516,
                    _ => value,
                }
            }
            ConverterCategory::Volume => {
                match unit_idx {
                    0 => value,
                    1 => value / 0.001,
                    2 => value / 1000.0,
                    3 => value / 3.78541,
                    4 => value / 4.54609,
                    5 => value / 0.946353,
                    6 => value / 0.473176,
                    7 => value / 0.24,
                    _ => value,
                }
            }
            ConverterCategory::Time => {
                match unit_idx {
                    0 => value,
                    1 => value / 60.0,
                    2 => value / 3600.0,
                    3 => value / 86400.0,
                    4 => value / 604800.0,
                    5 => value / 2_629_746.0,
                    6 => value / 31_556_952.0,
                    7 => value / 0.001,
                    _ => value,
                }
            }
            ConverterCategory::Speed => {
                match unit_idx {
                    0 => value,
                    1 => value * 3.6,
                    2 => value / 0.44704,
                    3 => value / 0.514444,
                    4 => value / 0.3048,
                    _ => value,
                }
            }
            ConverterCategory::Data => {
                match unit_idx {
                    0 => value,
                    1 => value / 1024.0,
                    2 => value / 1_048_576.0,
                    3 => value / 1_073_741_824.0,
                    4 => value / 1_099_511_627_776.0,
                    5 => value * 8.0,
                    6 => value / 128.0,
                    7 => value / 131_072.0,
                    _ => value,
                }
            }
        }
    }
}

impl Default for ConverterMode {
    fn default() -> Self {
        Self::new()
    }
}
