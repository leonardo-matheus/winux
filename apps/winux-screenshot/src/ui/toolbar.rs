//! Editor toolbar component

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Button, Box as GtkBox, Orientation, ToggleButton, Scale, ColorButton, Separator};
use std::cell::RefCell;
use std::rc::Rc;

use crate::editor::{EditorState, ToolType, COLOR_PALETTE};

/// Toolbar for the editor
pub struct EditorToolbar {
    pub container: GtkBox,
    tool_buttons: Vec<ToggleButton>,
}

impl EditorToolbar {
    pub fn new(
        editor_state: Rc<RefCell<EditorState>>,
        on_tool_changed: impl Fn(ToolType) + 'static,
        on_undo: impl Fn() + 'static,
        on_redo: impl Fn() + 'static,
        on_clear: impl Fn() + 'static,
    ) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 6);
        container.set_margin_start(6);
        container.set_margin_end(6);
        container.set_margin_top(6);
        container.set_margin_bottom(6);

        // Tool buttons
        let tools_box = GtkBox::new(Orientation::Horizontal, 0);
        tools_box.add_css_class("linked");

        let on_tool_changed = Rc::new(on_tool_changed);
        let mut tool_buttons = Vec::new();

        for tool in ToolType::all() {
            let button = ToggleButton::builder()
                .icon_name(tool.icon())
                .tooltip_text(tool.tooltip())
                .build();

            if *tool == ToolType::Arrow {
                button.set_active(true);
            }

            let tool = *tool;
            let on_tool_changed = on_tool_changed.clone();
            let editor_state = editor_state.clone();

            button.connect_toggled(move |btn| {
                if btn.is_active() {
                    editor_state.borrow_mut().current_tool = tool;
                    on_tool_changed(tool);
                }
            });

            tools_box.append(&button);
            tool_buttons.push(button);
        }

        // Make tool buttons mutually exclusive
        for (i, button) in tool_buttons.iter().enumerate() {
            let others: Vec<_> = tool_buttons
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, b)| b.clone())
                .collect();

            button.connect_toggled(move |btn| {
                if btn.is_active() {
                    for other in &others {
                        other.set_active(false);
                    }
                }
            });
        }

        container.append(&tools_box);

        // Separator
        container.append(&Separator::new(Orientation::Vertical));

        // Color selection
        let color_box = GtkBox::new(Orientation::Horizontal, 2);

        for color in COLOR_PALETTE {
            let color_btn = Button::new();
            color_btn.set_size_request(24, 24);

            // Apply color as CSS
            let css = format!(
                "button {{ background-color: rgba({}, {}, {}, {}); min-width: 24px; min-height: 24px; padding: 0; }}",
                (color.red() * 255.0) as u8,
                (color.green() * 255.0) as u8,
                (color.blue() * 255.0) as u8,
                color.alpha()
            );

            let provider = gtk::CssProvider::new();
            provider.load_from_string(&css);
            color_btn.style_context().add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

            let color = *color;
            let editor_state = editor_state.clone();
            color_btn.connect_clicked(move |_| {
                editor_state.borrow_mut().color = color;
            });

            color_box.append(&color_btn);
        }

        // Custom color button
        let custom_color_btn = ColorButton::new();
        custom_color_btn.set_tooltip_text(Some("Custom color"));
        {
            let editor_state = editor_state.clone();
            custom_color_btn.connect_color_set(move |btn| {
                editor_state.borrow_mut().color = btn.rgba();
            });
        }
        color_box.append(&custom_color_btn);

        container.append(&color_box);

        // Separator
        container.append(&Separator::new(Orientation::Vertical));

        // Stroke width slider
        let stroke_box = GtkBox::new(Orientation::Horizontal, 4);

        let stroke_icon = gtk::Image::from_icon_name("format-stroke-width-symbolic");
        stroke_box.append(&stroke_icon);

        let stroke_scale = Scale::with_range(Orientation::Horizontal, 1.0, 20.0, 1.0);
        stroke_scale.set_value(3.0);
        stroke_scale.set_size_request(80, -1);
        stroke_scale.set_tooltip_text(Some("Stroke width"));

        {
            let editor_state = editor_state.clone();
            stroke_scale.connect_value_changed(move |scale| {
                editor_state.borrow_mut().stroke_width = scale.value();
            });
        }
        stroke_box.append(&stroke_scale);

        container.append(&stroke_box);

        // Separator
        container.append(&Separator::new(Orientation::Vertical));

        // Undo/Redo buttons
        let history_box = GtkBox::new(Orientation::Horizontal, 0);
        history_box.add_css_class("linked");

        let undo_btn = Button::builder()
            .icon_name("edit-undo-symbolic")
            .tooltip_text("Undo (Ctrl+Z)")
            .build();

        let redo_btn = Button::builder()
            .icon_name("edit-redo-symbolic")
            .tooltip_text("Redo (Ctrl+Shift+Z)")
            .build();

        let on_undo = Rc::new(on_undo);
        let on_redo = Rc::new(on_redo);

        {
            let on_undo = on_undo.clone();
            undo_btn.connect_clicked(move |_| {
                on_undo();
            });
        }

        {
            let on_redo = on_redo.clone();
            redo_btn.connect_clicked(move |_| {
                on_redo();
            });
        }

        history_box.append(&undo_btn);
        history_box.append(&redo_btn);
        container.append(&history_box);

        // Clear button
        let clear_btn = Button::builder()
            .icon_name("edit-clear-all-symbolic")
            .tooltip_text("Clear all annotations")
            .build();

        let on_clear = Rc::new(on_clear);
        clear_btn.connect_clicked(move |_| {
            on_clear();
        });
        container.append(&clear_btn);

        // Spacer
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        container.append(&spacer);

        Self {
            container,
            tool_buttons,
        }
    }

    /// Set the active tool
    pub fn set_active_tool(&self, tool: ToolType) {
        for (i, btn) in self.tool_buttons.iter().enumerate() {
            btn.set_active(ToolType::all()[i] == tool);
        }
    }
}

/// Bottom action bar for the editor
pub struct ActionBar {
    pub container: GtkBox,
}

impl ActionBar {
    pub fn new(
        on_save: impl Fn() + 'static,
        on_copy: impl Fn() + 'static,
        on_share: impl Fn() + 'static,
        on_discard: impl Fn() + 'static,
    ) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 12);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_halign(gtk::Align::Center);

        // Discard button
        let discard_btn = Button::builder()
            .label("Discard")
            .build();
        discard_btn.add_css_class("destructive-action");

        let on_discard = Rc::new(on_discard);
        discard_btn.connect_clicked(move |_| {
            on_discard();
        });
        container.append(&discard_btn);

        // Copy to clipboard button
        let copy_btn = Button::builder()
            .icon_name("edit-copy-symbolic")
            .label("Copy")
            .build();

        let on_copy = Rc::new(on_copy);
        copy_btn.connect_clicked(move |_| {
            on_copy();
        });
        container.append(&copy_btn);

        // Share button
        let share_btn = Button::builder()
            .icon_name("send-to-symbolic")
            .label("Share")
            .build();

        let on_share = Rc::new(on_share);
        share_btn.connect_clicked(move |_| {
            on_share();
        });
        container.append(&share_btn);

        // Save button
        let save_btn = Button::builder()
            .icon_name("document-save-symbolic")
            .label("Save")
            .build();
        save_btn.add_css_class("suggested-action");

        let on_save = Rc::new(on_save);
        save_btn.connect_clicked(move |_| {
            on_save();
        });
        container.append(&save_btn);

        Self { container }
    }
}
