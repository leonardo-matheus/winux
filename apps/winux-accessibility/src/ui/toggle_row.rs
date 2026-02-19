//! Custom toggle row widget for accessibility settings

use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A custom row widget with a toggle switch and optional description
pub struct ToggleRow {
    row: adw::ActionRow,
    switch: gtk4::Switch,
    on_change: Rc<RefCell<Option<Box<dyn Fn(bool)>>>>,
}

impl ToggleRow {
    /// Create a new toggle row
    pub fn new(title: &str) -> Self {
        let row = adw::ActionRow::builder()
            .title(title)
            .build();

        let switch = gtk4::Switch::new();
        switch.set_valign(gtk4::Align::Center);
        row.add_suffix(&switch);
        row.set_activatable_widget(Some(&switch));

        let on_change: Rc<RefCell<Option<Box<dyn Fn(bool)>>>> = Rc::new(RefCell::new(None));

        let on_change_clone = on_change.clone();
        switch.connect_state_set(move |_, state| {
            if let Some(ref callback) = *on_change_clone.borrow() {
                callback(state);
            }
            glib::Propagation::Proceed
        });

        Self {
            row,
            switch,
            on_change,
        }
    }

    /// Create a new toggle row with builder pattern
    pub fn builder() -> ToggleRowBuilder {
        ToggleRowBuilder::new()
    }

    /// Set the title
    pub fn set_title(&self, title: &str) {
        self.row.set_title(title);
    }

    /// Set the subtitle
    pub fn set_subtitle(&self, subtitle: &str) {
        self.row.set_subtitle(subtitle);
    }

    /// Set the active state
    pub fn set_active(&self, active: bool) {
        self.switch.set_active(active);
    }

    /// Get the active state
    pub fn is_active(&self) -> bool {
        self.switch.is_active()
    }

    /// Connect to state changes
    pub fn connect_changed<F: Fn(bool) + 'static>(&self, callback: F) {
        *self.on_change.borrow_mut() = Some(Box::new(callback));
    }

    /// Get the underlying widget
    pub fn widget(&self) -> &adw::ActionRow {
        &self.row
    }

    /// Add a prefix widget (e.g., icon)
    pub fn add_prefix<W: IsA<gtk4::Widget>>(&self, widget: &W) {
        self.row.add_prefix(widget);
    }

    /// Set sensitive state
    pub fn set_sensitive(&self, sensitive: bool) {
        self.row.set_sensitive(sensitive);
    }
}

/// Builder for ToggleRow
pub struct ToggleRowBuilder {
    title: Option<String>,
    subtitle: Option<String>,
    active: bool,
    icon_name: Option<String>,
    sensitive: bool,
}

impl ToggleRowBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            title: None,
            subtitle: None,
            active: false,
            icon_name: None,
            sensitive: true,
        }
    }

    /// Set the title
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set the subtitle
    pub fn subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    /// Set the initial active state
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Set an icon
    pub fn icon_name(mut self, icon: &str) -> Self {
        self.icon_name = Some(icon.to_string());
        self
    }

    /// Set sensitive state
    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    /// Build the toggle row
    pub fn build(self) -> ToggleRow {
        let row = ToggleRow::new(self.title.as_deref().unwrap_or(""));

        if let Some(subtitle) = self.subtitle {
            row.set_subtitle(&subtitle);
        }

        row.set_active(self.active);
        row.set_sensitive(self.sensitive);

        if let Some(icon) = self.icon_name {
            let image = gtk4::Image::from_icon_name(&icon);
            row.add_prefix(&image);
        }

        row
    }
}

impl Default for ToggleRowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A preference row with a scale/slider
pub struct ScaleRow {
    row: adw::ActionRow,
    scale: gtk4::Scale,
    value_label: gtk4::Label,
}

impl ScaleRow {
    /// Create a new scale row
    pub fn new(title: &str, min: f64, max: f64, step: f64) -> Self {
        let row = adw::ActionRow::builder()
            .title(title)
            .build();

        let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, min, max, step);
        scale.set_width_request(200);
        scale.set_draw_value(false);
        scale.set_valign(gtk4::Align::Center);

        let value_label = gtk4::Label::new(None);
        value_label.add_css_class("dim-label");
        value_label.set_width_chars(5);

        let value_label_clone = value_label.clone();
        scale.connect_value_changed(move |s| {
            value_label_clone.set_text(&format!("{:.1}", s.value()));
        });

        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        hbox.append(&scale);
        hbox.append(&value_label);
        hbox.set_valign(gtk4::Align::Center);

        row.add_suffix(&hbox);

        Self {
            row,
            scale,
            value_label,
        }
    }

    /// Set the value
    pub fn set_value(&self, value: f64) {
        self.scale.set_value(value);
        self.value_label.set_text(&format!("{:.1}", value));
    }

    /// Get the value
    pub fn value(&self) -> f64 {
        self.scale.value()
    }

    /// Connect to value changes
    pub fn connect_value_changed<F: Fn(f64) + 'static>(&self, callback: F) {
        self.scale.connect_value_changed(move |s| {
            callback(s.value());
        });
    }

    /// Get the widget
    pub fn widget(&self) -> &adw::ActionRow {
        &self.row
    }

    /// Set subtitle
    pub fn set_subtitle(&self, subtitle: &str) {
        self.row.set_subtitle(subtitle);
    }

    /// Add marks to the scale
    pub fn add_mark(&self, value: f64, position: gtk4::PositionType, markup: Option<&str>) {
        self.scale.add_mark(value, position, markup);
    }

    /// Set value suffix (e.g., "px", "ms", "%")
    pub fn set_value_suffix(&self, suffix: &str) {
        let suffix = suffix.to_string();
        let value_label = self.value_label.clone();
        self.scale.connect_value_changed(move |s| {
            value_label.set_text(&format!("{:.0}{}", s.value(), suffix));
        });
    }
}

/// A preference row with a combo box
pub struct ComboRow {
    row: adw::ComboRow,
}

impl ComboRow {
    /// Create a new combo row
    pub fn new(title: &str, options: &[&str]) -> Self {
        let row = adw::ComboRow::builder()
            .title(title)
            .build();

        let model = gtk4::StringList::new(options);
        row.set_model(Some(&model));

        Self { row }
    }

    /// Set the selected index
    pub fn set_selected(&self, index: u32) {
        self.row.set_selected(index);
    }

    /// Get the selected index
    pub fn selected(&self) -> u32 {
        self.row.selected()
    }

    /// Connect to selection changes
    pub fn connect_selected_notify<F: Fn(u32) + 'static>(&self, callback: F) {
        self.row.connect_selected_notify(move |r| {
            callback(r.selected());
        });
    }

    /// Get the widget
    pub fn widget(&self) -> &adw::ComboRow {
        &self.row
    }

    /// Set subtitle
    pub fn set_subtitle(&self, subtitle: &str) {
        self.row.set_subtitle(subtitle);
    }
}
