// Winux Notes - Note Card Widget
// Copyright (c) 2026 Winux OS Project

use crate::data::Note;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, Box, CheckButton, Frame, Image, Label, Orientation};
use std::cell::RefCell;

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct NoteCard {
        pub note_id: RefCell<String>,
        pub frame: OnceCell<Frame>,
        pub title_label: OnceCell<Label>,
        pub preview_label: OnceCell<Label>,
        pub date_label: OnceCell<Label>,
        pub pin_icon: OnceCell<Image>,
        pub star_icon: OnceCell<Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteCard {
        const NAME: &'static str = "WinuxNotesNoteCard";
        type Type = super::NoteCard;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for NoteCard {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for NoteCard {}
    impl BoxImpl for NoteCard {}
}

glib::wrapper! {
    pub struct NoteCard(ObjectSubclass<imp::NoteCard>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl NoteCard {
    pub fn new(note: &Note) -> Self {
        let card: Self = glib::Object::builder().build();
        card.setup_ui(note);
        card
    }

    fn setup_ui(&self, note: &Note) {
        let imp = self.imp();
        *imp.note_id.borrow_mut() = note.id.clone();

        // Configure the box
        self.set_orientation(Orientation::Vertical);
        self.set_width_request(220);
        self.set_height_request(180);

        // Create frame with color
        let frame = Frame::builder()
            .css_classes(vec!["card", note.color.to_css_class()])
            .build();

        let content = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        // Header row (title + icons)
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        // Pin icon
        let pin_icon = Image::builder()
            .icon_name("view-pin-symbolic")
            .pixel_size(14)
            .visible(note.pinned)
            .css_classes(vec!["dim-label"])
            .build();
        header.append(&pin_icon);
        imp.pin_icon.set(pin_icon).unwrap();

        // Title
        let title_label = Label::builder()
            .label(&note.title)
            .css_classes(vec!["heading"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(25)
            .build();
        header.append(&title_label);
        imp.title_label.set(title_label).unwrap();

        // Star icon
        let star_icon = Image::builder()
            .icon_name("starred-symbolic")
            .pixel_size(14)
            .visible(note.favorite)
            .css_classes(vec!["accent"])
            .build();
        header.append(&star_icon);
        imp.star_icon.set(star_icon).unwrap();

        content.append(&header);

        // Preview (content or checklist)
        if !note.checklist.is_empty() {
            // Show checklist preview
            let checklist_box = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(2)
                .vexpand(true)
                .build();

            for item in note.checklist.iter().take(4) {
                let item_row = Box::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(6)
                    .build();

                let check = CheckButton::builder()
                    .active(item.checked)
                    .sensitive(false)
                    .build();
                item_row.append(&check);

                let text = Label::builder()
                    .label(&item.text)
                    .halign(gtk4::Align::Start)
                    .ellipsize(gtk4::pango::EllipsizeMode::End)
                    .max_width_chars(20)
                    .build();

                if item.checked {
                    text.add_css_class("dim-label");
                }

                item_row.append(&text);
                checklist_box.append(&item_row);
            }

            if note.checklist.len() > 4 {
                let more = Label::builder()
                    .label(&format!("+ {} more", note.checklist.len() - 4))
                    .css_classes(vec!["caption", "dim-label"])
                    .halign(gtk4::Align::Start)
                    .build();
                checklist_box.append(&more);
            }

            content.append(&checklist_box);
        } else {
            // Show content preview
            let preview_label = Label::builder()
                .label(&note.content_preview(100))
                .css_classes(vec!["body"])
                .halign(gtk4::Align::Start)
                .valign(gtk4::Align::Start)
                .wrap(true)
                .wrap_mode(gtk4::pango::WrapMode::Word)
                .vexpand(true)
                .max_width_chars(30)
                .lines(4)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            content.append(&preview_label);
            imp.preview_label.set(preview_label).unwrap();
        }

        // Tags row
        if !note.tags.is_empty() {
            let tags_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(4)
                .build();

            for tag in note.tags.iter().take(2) {
                let tag_label = Label::builder()
                    .label(tag)
                    .css_classes(vec!["caption", "tag-badge"])
                    .build();
                tags_box.append(&tag_label);
            }

            if note.tags.len() > 2 {
                let more = Label::builder()
                    .label(&format!("+{}", note.tags.len() - 2))
                    .css_classes(vec!["caption", "dim-label"])
                    .build();
                tags_box.append(&more);
            }

            content.append(&tags_box);
        }

        // Footer with date
        let date_label = Label::builder()
            .label(&note.relative_time())
            .css_classes(vec!["caption", "dim-label"])
            .halign(gtk4::Align::Start)
            .build();
        content.append(&date_label);
        imp.date_label.set(date_label).unwrap();

        frame.set_child(Some(&content));
        imp.frame.set(frame.clone()).unwrap();
        self.append(&frame);

        // Add hover effect
        let controller = gtk4::EventControllerMotion::new();
        let frame_clone = frame.clone();
        controller.connect_enter(move |_, _, _| {
            frame_clone.add_css_class("card-hover");
        });
        let frame_clone = frame.clone();
        controller.connect_leave(move |_| {
            frame_clone.remove_css_class("card-hover");
        });
        self.add_controller(controller);
    }

    pub fn note_id(&self) -> String {
        self.imp().note_id.borrow().clone()
    }

    pub fn connect_clicked<F: Fn(&Self) + 'static>(&self, callback: F) {
        let gesture = gtk4::GestureClick::new();
        let card = self.clone();
        gesture.connect_released(move |_, _, _, _| {
            callback(&card);
        });
        self.add_controller(gesture);
    }

    pub fn update_note(&self, note: &Note) {
        let imp = self.imp();

        if let Some(title) = imp.title_label.get() {
            title.set_label(&note.title);
        }

        if let Some(preview) = imp.preview_label.get() {
            preview.set_label(&note.content_preview(100));
        }

        if let Some(date) = imp.date_label.get() {
            date.set_label(&note.relative_time());
        }

        if let Some(pin) = imp.pin_icon.get() {
            pin.set_visible(note.pinned);
        }

        if let Some(star) = imp.star_icon.get() {
            star.set_visible(note.favorite);
        }

        // Update color
        if let Some(frame) = imp.frame.get() {
            // Remove old color classes
            for color in crate::data::NoteColor::all() {
                frame.remove_css_class(color.to_css_class());
            }
            frame.add_css_class(note.color.to_css_class());
        }
    }
}

impl Default for NoteCard {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}
