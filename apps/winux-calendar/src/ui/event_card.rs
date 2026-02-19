//! Event card component

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Frame, GestureClick};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::data::Event;

/// Event card widget
pub struct EventCard {
    widget: Frame,
    event: Rc<RefCell<Event>>,
}

impl EventCard {
    /// Create a new event card
    pub fn new(event: Event) -> Self {
        let widget = Frame::builder()
            .css_classes(vec!["card"])
            .build();

        let event = Rc::new(RefCell::new(event));

        let card = Self { widget, event };
        card.build_ui();
        card
    }

    /// Get the widget
    pub fn widget(&self) -> &Frame {
        &self.widget
    }

    /// Build the UI
    fn build_ui(&self) {
        let event = self.event.borrow();

        let content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Color indicator
        let color_bar = Box::builder()
            .width_request(4)
            .build();

        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "box {{ background-color: {}; border-radius: 2px; }}",
            event.color
        );
        css_provider.load_from_string(&css);
        color_bar.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        content.append(&color_bar);

        // Event info
        let info_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .build();

        // Title
        let title = Label::builder()
            .label(&event.title)
            .css_classes(vec!["title-4"])
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        info_box.append(&title);

        // Time
        let time_str = if event.all_day {
            "Dia inteiro".to_string()
        } else if let Some(start) = event.start_time {
            if let Some(end) = event.end_time {
                format!("{:02}:{:02} - {:02}:{:02}",
                    start.hour(), start.minute(),
                    end.hour(), end.minute())
            } else {
                format!("{:02}:{:02}", start.hour(), start.minute())
            }
        } else {
            String::new()
        };

        if !time_str.is_empty() {
            let time_label = Label::builder()
                .label(&time_str)
                .css_classes(vec!["caption", "dim-label"])
                .halign(gtk4::Align::Start)
                .build();
            info_box.append(&time_label);
        }

        // Location
        if let Some(ref location) = event.location {
            let loc_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(4)
                .build();

            let loc_icon = gtk4::Image::from_icon_name("mark-location-symbolic");
            loc_icon.add_css_class("dim-label");
            loc_box.append(&loc_icon);

            let loc_label = Label::builder()
                .label(location)
                .css_classes(vec!["caption", "dim-label"])
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            loc_box.append(&loc_label);

            info_box.append(&loc_box);
        }

        content.append(&info_box);

        // Calendar indicator
        let calendar_label = Label::builder()
            .label(&event.calendar_name)
            .css_classes(vec!["caption", "dim-label"])
            .valign(gtk4::Align::Center)
            .build();
        content.append(&calendar_label);

        // Apply event color as border
        let frame_css = gtk4::CssProvider::new();
        let frame_style = format!(
            "frame {{ border-left: 4px solid {}; }}",
            event.color
        );
        frame_css.load_from_string(&frame_style);
        self.widget.style_context().add_provider(&frame_css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        self.widget.set_child(Some(&content));

        // Click handler
        let gesture = GestureClick::new();
        let event_clone = self.event.clone();
        gesture.connect_released(move |_, _, _, _| {
            // Open event details/editor
            let _event = event_clone.borrow();
            // TODO: Show event detail popup
        });
        self.widget.add_controller(gesture);
    }

    /// Get the event
    pub fn event(&self) -> std::cell::Ref<Event> {
        self.event.borrow()
    }

    /// Update the event
    pub fn set_event(&self, event: Event) {
        *self.event.borrow_mut() = event;
        self.build_ui();
    }
}

/// Compact event pill for month view
pub struct EventPill {
    widget: Box,
}

impl EventPill {
    pub fn new(event: &Event) -> Self {
        let widget = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .margin_start(2)
            .margin_end(2)
            .css_classes(vec!["card"])
            .build();

        // Color dot
        let color_dot = Box::builder()
            .width_request(8)
            .height_request(8)
            .valign(gtk4::Align::Center)
            .build();

        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "box {{ background-color: {}; border-radius: 4px; }}",
            event.color
        );
        css_provider.load_from_string(&css);
        color_dot.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        widget.append(&color_dot);

        // Title
        let title = Label::builder()
            .label(&event.title)
            .css_classes(vec!["caption"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .halign(gtk4::Align::Start)
            .build();
        widget.append(&title);

        Self { widget }
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}

/// Time block for week/day views
pub struct TimeBlock {
    widget: Frame,
}

impl TimeBlock {
    pub fn new(event: &Event, height: i32) -> Self {
        let widget = Frame::builder()
            .height_request(height)
            .css_classes(vec!["card"])
            .build();

        let content = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // Apply event color
        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "frame {{ background-color: alpha({}, 0.3); border-left: 3px solid {}; }}",
            event.color, event.color
        );
        css_provider.load_from_string(&css);
        widget.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Title
        let title = Label::builder()
            .label(&event.title)
            .css_classes(vec!["caption"])
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        content.append(&title);

        // Time
        if let Some(start) = event.start_time {
            let time_str = format!("{:02}:{:02}", start.hour(), start.minute());
            let time_label = Label::builder()
                .label(&time_str)
                .css_classes(vec!["caption", "dim-label"])
                .halign(gtk4::Align::Start)
                .build();
            content.append(&time_label);
        }

        // Location (if space allows)
        if height > 40 {
            if let Some(ref location) = event.location {
                let loc_label = Label::builder()
                    .label(location)
                    .css_classes(vec!["caption", "dim-label"])
                    .halign(gtk4::Align::Start)
                    .ellipsize(gtk4::pango::EllipsizeMode::End)
                    .build();
                content.append(&loc_label);
            }
        }

        widget.set_child(Some(&content));

        Self { widget }
    }

    pub fn widget(&self) -> &Frame {
        &self.widget
    }
}
