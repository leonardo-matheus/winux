// Winux Notes - Editor Toolbar
// Copyright (c) 2026 Winux OS Project

use crate::editor::FormatAction;
use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, Separator, ToggleButton, Widget};
use std::cell::RefCell;
use std::rc::Rc;

/// Toolbar for the note editor with formatting buttons
pub struct Toolbar {
    container: Box,
    on_format: Rc<RefCell<Option<Box<dyn Fn(FormatAction)>>>>,
    on_preview: Rc<RefCell<Option<Box<dyn Fn(bool)>>>>,
    on_checklist: Rc<RefCell<Option<Box<dyn Fn(bool)>>>>,
    preview_btn: ToggleButton,
    checklist_btn: ToggleButton,
}

impl Toolbar {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .margin_start(10)
            .margin_end(10)
            .margin_top(5)
            .margin_bottom(5)
            .css_classes(vec!["toolbar"])
            .build();

        let on_format: Rc<RefCell<Option<Box<dyn Fn(FormatAction)>>>> = Rc::new(RefCell::new(None));
        let on_preview: Rc<RefCell<Option<Box<dyn Fn(bool)>>>> = Rc::new(RefCell::new(None));
        let on_checklist: Rc<RefCell<Option<Box<dyn Fn(bool)>>>> = Rc::new(RefCell::new(None));

        // Text formatting group
        let format_box = Box::new(Orientation::Horizontal, 0);
        format_box.add_css_class("linked");

        let bold_btn = Self::create_format_button(
            "format-text-bold-symbolic",
            "Bold (Ctrl+B)",
            FormatAction::Bold,
            &on_format,
        );
        format_box.append(&bold_btn);

        let italic_btn = Self::create_format_button(
            "format-text-italic-symbolic",
            "Italic (Ctrl+I)",
            FormatAction::Italic,
            &on_format,
        );
        format_box.append(&italic_btn);

        let underline_btn = Self::create_format_button(
            "format-text-underline-symbolic",
            "Underline (Ctrl+U)",
            FormatAction::Underline,
            &on_format,
        );
        format_box.append(&underline_btn);

        let strike_btn = Self::create_format_button(
            "format-text-strikethrough-symbolic",
            "Strikethrough",
            FormatAction::Strikethrough,
            &on_format,
        );
        format_box.append(&strike_btn);

        container.append(&format_box);

        // Separator
        let sep1 = Separator::new(Orientation::Vertical);
        sep1.set_margin_start(6);
        sep1.set_margin_end(6);
        container.append(&sep1);

        // Headings
        let heading_box = Box::new(Orientation::Horizontal, 0);
        heading_box.add_css_class("linked");

        let h1_btn = Button::builder()
            .label("H1")
            .css_classes(vec!["flat"])
            .tooltip_text("Heading 1")
            .build();
        let on_format_clone = on_format.clone();
        h1_btn.connect_clicked(move |_| {
            if let Some(cb) = on_format_clone.borrow().as_ref() {
                cb(FormatAction::Heading1);
            }
        });
        heading_box.append(&h1_btn);

        let h2_btn = Button::builder()
            .label("H2")
            .css_classes(vec!["flat"])
            .tooltip_text("Heading 2")
            .build();
        let on_format_clone = on_format.clone();
        h2_btn.connect_clicked(move |_| {
            if let Some(cb) = on_format_clone.borrow().as_ref() {
                cb(FormatAction::Heading2);
            }
        });
        heading_box.append(&h2_btn);

        let h3_btn = Button::builder()
            .label("H3")
            .css_classes(vec!["flat"])
            .tooltip_text("Heading 3")
            .build();
        let on_format_clone = on_format.clone();
        h3_btn.connect_clicked(move |_| {
            if let Some(cb) = on_format_clone.borrow().as_ref() {
                cb(FormatAction::Heading3);
            }
        });
        heading_box.append(&h3_btn);

        container.append(&heading_box);

        // Separator
        let sep2 = Separator::new(Orientation::Vertical);
        sep2.set_margin_start(6);
        sep2.set_margin_end(6);
        container.append(&sep2);

        // Lists and blocks
        let list_box = Box::new(Orientation::Horizontal, 0);
        list_box.add_css_class("linked");

        let bullet_btn = Self::create_format_button(
            "view-list-bullet-symbolic",
            "Bullet List",
            FormatAction::BulletList,
            &on_format,
        );
        list_box.append(&bullet_btn);

        let numbered_btn = Self::create_format_button(
            "view-list-ordered-symbolic",
            "Numbered List",
            FormatAction::NumberedList,
            &on_format,
        );
        list_box.append(&numbered_btn);

        let quote_btn = Self::create_format_button(
            "format-indent-more-symbolic",
            "Quote",
            FormatAction::Quote,
            &on_format,
        );
        list_box.append(&quote_btn);

        let code_btn = Self::create_format_button(
            "code-symbolic",
            "Code",
            FormatAction::Code,
            &on_format,
        );
        list_box.append(&code_btn);

        container.append(&list_box);

        // Separator
        let sep3 = Separator::new(Orientation::Vertical);
        sep3.set_margin_start(6);
        sep3.set_margin_end(6);
        container.append(&sep3);

        // Link button
        let link_btn = Self::create_format_button(
            "insert-link-symbolic",
            "Insert Link",
            FormatAction::Link,
            &on_format,
        );
        container.append(&link_btn);

        // Clear formatting
        let clear_btn = Self::create_format_button(
            "edit-clear-symbolic",
            "Clear Formatting",
            FormatAction::Clear,
            &on_format,
        );
        container.append(&clear_btn);

        // Spacer
        let spacer = Box::builder()
            .hexpand(true)
            .build();
        container.append(&spacer);

        // Checklist toggle
        let checklist_btn = ToggleButton::builder()
            .icon_name("view-list-symbolic")
            .tooltip_text("Checklist Mode")
            .css_classes(vec!["flat"])
            .build();

        let on_checklist_clone = on_checklist.clone();
        checklist_btn.connect_toggled(move |btn| {
            if let Some(cb) = on_checklist_clone.borrow().as_ref() {
                cb(btn.is_active());
            }
        });
        container.append(&checklist_btn);

        // Preview toggle
        let preview_btn = ToggleButton::builder()
            .icon_name("view-reveal-symbolic")
            .tooltip_text("Preview Markdown")
            .css_classes(vec!["flat"])
            .build();

        let on_preview_clone = on_preview.clone();
        preview_btn.connect_toggled(move |btn| {
            if let Some(cb) = on_preview_clone.borrow().as_ref() {
                cb(btn.is_active());
            }
        });
        container.append(&preview_btn);

        Self {
            container,
            on_format,
            on_preview,
            on_checklist,
            preview_btn,
            checklist_btn,
        }
    }

    fn create_format_button(
        icon: &str,
        tooltip: &str,
        action: FormatAction,
        callback: &Rc<RefCell<Option<Box<dyn Fn(FormatAction)>>>>,
    ) -> Button {
        let btn = Button::builder()
            .icon_name(icon)
            .tooltip_text(tooltip)
            .css_classes(vec!["flat"])
            .build();

        let callback = callback.clone();
        btn.connect_clicked(move |_| {
            if let Some(cb) = callback.borrow().as_ref() {
                cb(action);
            }
        });

        btn
    }

    pub fn widget(&self) -> &Widget {
        self.container.upcast_ref()
    }

    pub fn connect_format_action<F: Fn(FormatAction) + 'static>(&self, callback: F) {
        *self.on_format.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_preview_toggle<F: Fn(bool) + 'static>(&self, callback: F) {
        *self.on_preview.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_checklist_toggle<F: Fn(bool) + 'static>(&self, callback: F) {
        *self.on_checklist.borrow_mut() = Some(Box::new(callback));
    }

    pub fn set_preview_active(&self, active: bool) {
        self.preview_btn.set_active(active);
    }

    pub fn set_checklist_active(&self, active: bool) {
        self.checklist_btn.set_active(active);
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}
