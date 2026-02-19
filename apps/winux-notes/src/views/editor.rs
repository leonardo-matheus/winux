// Winux Notes - Editor View
// Copyright (c) 2026 Winux OS Project

use crate::data::{Note, NoteColor};
use crate::editor::{ChecklistEditor, MarkdownRenderer, RichTextEditor};
use crate::ui::Toolbar;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, Entry, Image, Label, MenuButton, Orientation, Popover, ScrolledWindow,
    Separator, Stack, TextView, Widget,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Editor view for creating and editing notes
pub struct EditorView {
    container: Box,
    title_entry: Entry,
    content_stack: Stack,
    text_editor: RichTextEditor,
    markdown_renderer: MarkdownRenderer,
    checklist_editor: ChecklistEditor,
    toolbar: Toolbar,
    current_note: Rc<RefCell<Option<Note>>>,
    on_note_changed: Rc<RefCell<Option<Box<dyn Fn(&Note)>>>>,
}

impl EditorView {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .css_classes(vec!["view"])
            .build();

        // Header with title
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .margin_start(15)
            .margin_end(15)
            .margin_top(15)
            .build();

        let title_entry = Entry::builder()
            .placeholder_text("Note title...")
            .css_classes(vec!["title-2", "flat"])
            .hexpand(true)
            .build();
        header.append(&title_entry);

        // Note actions
        let actions_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        let pin_btn = Button::builder()
            .icon_name("view-pin-symbolic")
            .css_classes(vec!["flat", "circular"])
            .tooltip_text("Pin note")
            .build();
        actions_box.append(&pin_btn);

        let favorite_btn = Button::builder()
            .icon_name("non-starred-symbolic")
            .css_classes(vec!["flat", "circular"])
            .tooltip_text("Add to favorites")
            .build();
        actions_box.append(&favorite_btn);

        let color_btn = Self::create_color_button();
        actions_box.append(&color_btn);

        let menu_btn = Button::builder()
            .icon_name("view-more-symbolic")
            .css_classes(vec!["flat", "circular"])
            .tooltip_text("More options")
            .build();
        actions_box.append(&menu_btn);

        header.append(&actions_box);
        container.append(&header);

        // Toolbar
        let toolbar = Toolbar::new();
        container.append(toolbar.widget());

        // Content stack (editor modes)
        let content_stack = Stack::builder()
            .vexpand(true)
            .hexpand(true)
            .transition_type(gtk4::StackTransitionType::Crossfade)
            .build();

        // Rich text editor
        let text_editor = RichTextEditor::new();
        content_stack.add_named(text_editor.widget(), Some("editor"));

        // Markdown preview
        let markdown_renderer = MarkdownRenderer::new();
        content_stack.add_named(markdown_renderer.widget(), Some("preview"));

        // Checklist editor
        let checklist_editor = ChecklistEditor::new();
        content_stack.add_named(checklist_editor.widget(), Some("checklist"));

        let scroll = ScrolledWindow::builder()
            .child(&content_stack)
            .vexpand(true)
            .hexpand(true)
            .build();
        container.append(&scroll);

        // Show editor by default
        content_stack.set_visible_child_name("editor");

        let current_note = Rc::new(RefCell::new(None));
        let on_note_changed = Rc::new(RefCell::new(None));

        let editor_view = Self {
            container,
            title_entry,
            content_stack,
            text_editor,
            markdown_renderer,
            checklist_editor,
            toolbar,
            current_note,
            on_note_changed,
        };

        editor_view.setup_signals();
        editor_view
    }

    fn create_color_button() -> MenuButton {
        let btn = MenuButton::builder()
            .icon_name("color-select-symbolic")
            .css_classes(vec!["flat", "circular"])
            .tooltip_text("Change color")
            .build();

        let popover = Popover::new();
        let color_grid = gtk4::Grid::builder()
            .column_spacing(4)
            .row_spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let colors = NoteColor::all();
        for (i, color) in colors.iter().enumerate() {
            let color_btn = Button::builder()
                .css_classes(vec!["flat", "circular", color.to_css_class()])
                .width_request(28)
                .height_request(28)
                .tooltip_text(color.to_string())
                .build();

            let row = (i / 4) as i32;
            let col = (i % 4) as i32;
            color_grid.attach(&color_btn, col, row, 1, 1);
        }

        popover.set_child(Some(&color_grid));
        btn.set_popover(Some(&popover));

        btn
    }

    fn setup_signals(&self) {
        // Connect title changes
        let current_note = self.current_note.clone();
        let on_changed = self.on_note_changed.clone();

        self.title_entry.connect_changed(move |entry| {
            if let Some(ref mut note) = *current_note.borrow_mut() {
                note.title = entry.text().to_string();
                if let Some(ref cb) = *on_changed.borrow() {
                    cb(note);
                }
            }
        });

        // Connect toolbar buttons to editor
        let text_editor = self.text_editor.clone();
        self.toolbar.connect_format_action(move |action| {
            text_editor.apply_format(action);
        });

        // Connect view mode toggle
        let content_stack = self.content_stack.clone();
        let text_editor2 = self.text_editor.clone();
        let markdown_renderer = self.markdown_renderer.clone();

        self.toolbar.connect_preview_toggle(move |preview| {
            if preview {
                // Update markdown preview
                let content = text_editor2.get_content();
                markdown_renderer.set_markdown(&content);
                content_stack.set_visible_child_name("preview");
            } else {
                content_stack.set_visible_child_name("editor");
            }
        });

        // Connect checklist toggle
        let content_stack2 = self.content_stack.clone();
        self.toolbar.connect_checklist_toggle(move |show_checklist| {
            if show_checklist {
                content_stack2.set_visible_child_name("checklist");
            } else {
                content_stack2.set_visible_child_name("editor");
            }
        });
    }

    pub fn widget(&self) -> &Widget {
        self.container.upcast_ref()
    }

    pub fn set_note(&self, note: &Note) {
        *self.current_note.borrow_mut() = Some(note.clone());

        self.title_entry.set_text(&note.title);
        self.text_editor.set_content(&note.content);
        self.checklist_editor.set_items(&note.checklist);

        // Update color styling
        // self.container.remove_css_class(...);
        // self.container.add_css_class(note.color.to_css_class());

        self.content_stack.set_visible_child_name("editor");
    }

    pub fn get_note(&self) -> Option<Note> {
        self.current_note.borrow().clone().map(|mut note| {
            note.title = self.title_entry.text().to_string();
            note.content = self.text_editor.get_content();
            note.checklist = self.checklist_editor.get_items();
            note
        })
    }

    pub fn clear(&self) {
        *self.current_note.borrow_mut() = None;
        self.title_entry.set_text("");
        self.text_editor.set_content("");
        self.checklist_editor.clear();
        self.content_stack.set_visible_child_name("editor");
    }

    pub fn new_note(&self) {
        let note = Note::new("", "");
        self.set_note(&note);
        self.title_entry.grab_focus();
    }

    pub fn on_note_changed<F: Fn(&Note) + 'static>(&self, callback: F) {
        *self.on_note_changed.borrow_mut() = Some(Box::new(callback));
    }
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new()
    }
}
