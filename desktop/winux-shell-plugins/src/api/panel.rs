//! Panel extension API
//!
//! Allows plugins to add widgets, indicators, and menus to the Winux panel.

use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Position on the panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelPosition {
    /// Left side of panel (start menu area)
    Left,
    /// Center of panel (taskbar area)
    Center,
    /// Right side of panel (system tray area)
    Right,
}

impl Default for PanelPosition {
    fn default() -> Self {
        Self::Right
    }
}

/// Widget size preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetSize {
    /// Minimal size (icon only)
    Minimal,
    /// Small size (icon + short text)
    Small,
    /// Medium size (icon + text)
    Medium,
    /// Large size (expanded widget)
    Large,
    /// Custom size
    Custom { width: i32, height: i32 },
}

impl Default for WidgetSize {
    fn default() -> Self {
        Self::Small
    }
}

impl WidgetSize {
    /// Get the minimum width for this size
    pub fn min_width(&self) -> i32 {
        match self {
            Self::Minimal => 24,
            Self::Small => 48,
            Self::Medium => 96,
            Self::Large => 200,
            Self::Custom { width, .. } => *width,
        }
    }

    /// Get the minimum height for this size
    pub fn min_height(&self) -> i32 {
        match self {
            Self::Minimal => 24,
            Self::Small => 32,
            Self::Medium => 32,
            Self::Large => 48,
            Self::Custom { height, .. } => *height,
        }
    }
}

/// Actions that can be triggered by widget interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetAction {
    /// No action
    None,
    /// Show a popup menu
    ShowMenu(Vec<MenuItem>),
    /// Show a popup window
    ShowPopup,
    /// Run a command
    RunCommand(String),
    /// Open a URL
    OpenUrl(String),
    /// Toggle widget state
    Toggle,
    /// Custom action with data
    Custom { name: String, data: String },
}

/// Menu item for popup menus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// Menu item ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Icon name (optional)
    pub icon: Option<String>,
    /// Whether the item is enabled
    pub enabled: bool,
    /// Whether this is a separator
    pub is_separator: bool,
    /// Submenu items
    pub submenu: Option<Vec<MenuItem>>,
    /// Action when clicked
    pub action: WidgetAction,
}

impl MenuItem {
    /// Create a new menu item
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: None,
            enabled: true,
            is_separator: false,
            submenu: None,
            action: WidgetAction::None,
        }
    }

    /// Create a separator
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: None,
            enabled: true,
            is_separator: true,
            submenu: None,
            action: WidgetAction::None,
        }
    }

    /// Set icon
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Set action
    pub fn with_action(mut self, action: WidgetAction) -> Self {
        self.action = action;
        self
    }

    /// Set submenu
    pub fn with_submenu(mut self, items: Vec<MenuItem>) -> Self {
        self.submenu = Some(items);
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Widget state that can be displayed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetState {
    /// Icon name to display
    pub icon: Option<String>,
    /// Text label to display
    pub label: Option<String>,
    /// Tooltip text
    pub tooltip: Option<String>,
    /// Badge text (e.g., notification count)
    pub badge: Option<String>,
    /// Progress value (0-100)
    pub progress: Option<u8>,
    /// Whether the widget is active/highlighted
    pub active: bool,
    /// Whether the widget is visible
    pub visible: bool,
    /// CSS classes to apply
    pub css_classes: Vec<String>,
    /// Custom data
    pub data: HashMap<String, String>,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            icon: None,
            label: None,
            tooltip: None,
            badge: None,
            progress: None,
            active: false,
            visible: true,
            css_classes: Vec::new(),
            data: HashMap::new(),
        }
    }
}

impl WidgetState {
    /// Create a new widget state with an icon
    pub fn with_icon(icon: &str) -> Self {
        Self {
            icon: Some(icon.to_string()),
            ..Default::default()
        }
    }

    /// Set the label
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set the tooltip
    pub fn tooltip(mut self, tooltip: &str) -> Self {
        self.tooltip = Some(tooltip.to_string());
        self
    }

    /// Set the badge
    pub fn badge(mut self, badge: &str) -> Self {
        self.badge = Some(badge.to_string());
        self
    }

    /// Set progress
    pub fn progress(mut self, value: u8) -> Self {
        self.progress = Some(value.min(100));
        self
    }

    /// Set active state
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

/// Popup window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupConfig {
    /// Popup width
    pub width: i32,
    /// Popup height
    pub height: i32,
    /// Whether the popup is resizable
    pub resizable: bool,
    /// Whether to close on focus loss
    pub close_on_unfocus: bool,
    /// Custom CSS class
    pub css_class: Option<String>,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            width: 300,
            height: 400,
            resizable: false,
            close_on_unfocus: true,
            css_class: None,
        }
    }
}

/// Trait for panel widgets provided by plugins
pub trait PanelWidget: Send + Sync {
    /// Get the widget ID
    fn id(&self) -> &str;

    /// Get the display name
    fn name(&self) -> &str;

    /// Get the preferred position
    fn position(&self) -> PanelPosition {
        PanelPosition::Right
    }

    /// Get the preferred size
    fn size(&self) -> WidgetSize {
        WidgetSize::Small
    }

    /// Get the priority (higher = more to the left/start)
    fn priority(&self) -> i32 {
        0
    }

    /// Get the current widget state
    fn state(&self) -> WidgetState;

    /// Build the GTK widget
    fn build_widget(&self) -> gtk::Widget;

    /// Handle left click
    fn on_click(&mut self) -> WidgetAction {
        WidgetAction::None
    }

    /// Handle right click
    fn on_right_click(&mut self) -> WidgetAction {
        WidgetAction::None
    }

    /// Handle middle click
    fn on_middle_click(&mut self) -> WidgetAction {
        WidgetAction::None
    }

    /// Handle scroll
    fn on_scroll(&mut self, _delta: f64, _direction: ScrollDirection) {}

    /// Get popup configuration (if widget supports popup)
    fn popup_config(&self) -> Option<PopupConfig> {
        None
    }

    /// Build popup content (if widget supports popup)
    fn build_popup(&self) -> Option<gtk::Widget> {
        None
    }

    /// Called when widget is shown
    fn on_show(&mut self) {}

    /// Called when widget is hidden
    fn on_hide(&mut self) {}

    /// Called periodically to update the widget
    fn update(&mut self) {}

    /// Get the update interval in milliseconds
    fn update_interval(&self) -> u32 {
        1000
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Helper struct to build simple indicator widgets
pub struct SimpleIndicator {
    id: String,
    name: String,
    state: WidgetState,
    position: PanelPosition,
    click_action: WidgetAction,
    menu_items: Vec<MenuItem>,
}

impl SimpleIndicator {
    /// Create a new simple indicator
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            state: WidgetState::default(),
            position: PanelPosition::Right,
            click_action: WidgetAction::None,
            menu_items: Vec::new(),
        }
    }

    /// Set the icon
    pub fn icon(mut self, icon: &str) -> Self {
        self.state.icon = Some(icon.to_string());
        self
    }

    /// Set the label
    pub fn label(mut self, label: &str) -> Self {
        self.state.label = Some(label.to_string());
        self
    }

    /// Set the tooltip
    pub fn tooltip(mut self, tooltip: &str) -> Self {
        self.state.tooltip = Some(tooltip.to_string());
        self
    }

    /// Set the position
    pub fn position(mut self, position: PanelPosition) -> Self {
        self.position = position;
        self
    }

    /// Set click action
    pub fn on_click(mut self, action: WidgetAction) -> Self {
        self.click_action = action;
        self
    }

    /// Add menu item
    pub fn add_menu_item(mut self, item: MenuItem) -> Self {
        self.menu_items.push(item);
        self
    }

    /// Update the state
    pub fn set_state(&mut self, state: WidgetState) {
        self.state = state;
    }

    /// Get mutable state reference
    pub fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }
}

impl PanelWidget for SimpleIndicator {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn position(&self) -> PanelPosition {
        self.position
    }

    fn state(&self) -> WidgetState {
        self.state.clone()
    }

    fn build_widget(&self) -> gtk::Widget {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        hbox.set_valign(gtk::Align::Center);

        if let Some(icon) = &self.state.icon {
            let image = gtk::Image::from_icon_name(icon);
            image.set_pixel_size(16);
            hbox.append(&image);
        }

        if let Some(label) = &self.state.label {
            let lbl = gtk::Label::new(Some(label));
            hbox.append(&lbl);
        }

        if let Some(tooltip) = &self.state.tooltip {
            hbox.set_tooltip_text(Some(tooltip));
        }

        for class in &self.state.css_classes {
            hbox.add_css_class(class);
        }

        hbox.upcast()
    }

    fn on_click(&mut self) -> WidgetAction {
        self.click_action.clone()
    }

    fn on_right_click(&mut self) -> WidgetAction {
        if !self.menu_items.is_empty() {
            WidgetAction::ShowMenu(self.menu_items.clone())
        } else {
            WidgetAction::None
        }
    }
}
