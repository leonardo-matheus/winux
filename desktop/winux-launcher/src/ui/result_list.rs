//! Result list component

use crate::config::Config;
use crate::search::{SearchCategory, SearchResult};
use gtk4::prelude::*;
use gtk4::{
    glib, Box as GtkBox, Image, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow,
    SelectionMode,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Result list widget
#[derive(Clone)]
pub struct ResultList {
    container: ScrolledWindow,
    list_box: ListBox,
    results: Rc<RefCell<Vec<SearchResult>>>,
    config: Arc<Config>,
}

impl ResultList {
    /// Create new result list
    pub fn new(config: Arc<Config>) -> Self {
        let list_box = ListBox::new();
        list_box.add_css_class("results-list");
        list_box.set_selection_mode(SelectionMode::Single);

        let container = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_height(0)
            .max_content_height(config.ui.max_height)
            .propagate_natural_height(true)
            .child(&list_box)
            .build();
        container.add_css_class("results-container");
        container.set_hexpand(true);

        Self {
            container,
            list_box,
            results: Rc::new(RefCell::new(Vec::new())),
            config,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &ScrolledWindow {
        &self.container
    }

    /// Set results
    pub fn set_results(&self, results: Vec<SearchResult>) {
        // Clear existing rows
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        if results.is_empty() {
            // Show no results message
            let no_results = self.create_no_results();
            self.list_box.append(&no_results);
            self.results.borrow_mut().clear();
            return;
        }

        // Group results by category if enabled
        let show_categories = self.config.ui.show_categories;
        let mut current_category: Option<SearchCategory> = None;

        for (index, result) in results.iter().enumerate() {
            // Add category header if needed
            if show_categories {
                if current_category.as_ref() != Some(&result.category) {
                    let header = self.create_category_header(&result.category);
                    self.list_box.append(&header);
                    current_category = Some(result.category.clone());
                }
            }

            // Create result row
            let row = self.create_result_row(result, index);
            self.list_box.append(&row);
        }

        // Store results
        *self.results.borrow_mut() = results;

        // Select first row
        if let Some(row) = self.list_box.row_at_index(if show_categories { 1 } else { 0 }) {
            self.list_box.select_row(Some(&row));
        }
    }

    /// Create a result row
    fn create_result_row(&self, result: &SearchResult, _index: usize) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("result-row");
        row.set_activatable(true);

        let content = GtkBox::new(Orientation::Horizontal, 0);
        content.set_margin_start(8);
        content.set_margin_end(8);

        // Icon
        if self.config.ui.show_icons {
            let icon = Image::from_icon_name(&result.icon);
            icon.add_css_class("result-icon");
            icon.set_pixel_size(self.config.ui.icon_size);
            content.append(&icon);
        }

        // Text container
        let text_box = GtkBox::new(Orientation::Vertical, 0);
        text_box.set_hexpand(true);
        text_box.set_valign(gtk4::Align::Center);

        // Title
        let title = Label::new(Some(&result.title));
        title.add_css_class("result-title");
        title.set_halign(gtk4::Align::Start);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        text_box.append(&title);

        // Subtitle
        if !result.subtitle.is_empty() {
            let subtitle = Label::new(Some(&result.subtitle));
            subtitle.add_css_class("result-subtitle");
            subtitle.set_halign(gtk4::Align::Start);
            subtitle.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
            text_box.append(&subtitle);
        }

        content.append(&text_box);

        // Category badge (for non-top hits)
        if result.category != SearchCategory::TopHit {
            let category = Label::new(Some(result.category.display_name()));
            category.add_css_class("result-category");
            content.append(&category);
        }

        // History badge
        if result.from_history {
            let history = Label::new(Some("Recent"));
            history.add_css_class("history-badge");
            content.append(&history);
        }

        row.set_child(Some(&content));
        row
    }

    /// Create category header
    fn create_category_header(&self, category: &SearchCategory) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_selectable(false);
        row.set_activatable(false);

        let label = Label::new(Some(category.display_name()));
        label.add_css_class("category-header");
        label.set_halign(gtk4::Align::Start);

        row.set_child(Some(&label));
        row
    }

    /// Create no results message
    fn create_no_results(&self) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("no-results");
        row.set_selectable(false);
        row.set_activatable(false);

        let content = GtkBox::new(Orientation::Vertical, 8);
        content.set_halign(gtk4::Align::Center);
        content.set_valign(gtk4::Align::Center);
        content.set_margin_top(20);
        content.set_margin_bottom(20);

        let icon = Image::from_icon_name("system-search-symbolic");
        icon.add_css_class("no-results-icon");
        icon.set_pixel_size(48);
        icon.set_opacity(0.3);
        content.append(&icon);

        let label = Label::new(Some("No results found"));
        label.add_css_class("no-results-text");
        content.append(&label);

        row.set_child(Some(&content));
        row
    }

    /// Get selected result
    pub fn selected(&self) -> Option<SearchResult> {
        let selected_row = self.list_box.selected_row()?;
        let index = selected_row.index() as usize;

        // Account for category headers
        let results = self.results.borrow();
        if self.config.ui.show_categories {
            // Count how many category headers come before this index
            let mut result_index = 0;
            let mut current_category: Option<SearchCategory> = None;

            for result in results.iter() {
                if current_category.as_ref() != Some(&result.category) {
                    current_category = Some(result.category.clone());
                    if result_index + 1 >= index {
                        break;
                    }
                    result_index += 1;
                }
                if result_index >= index {
                    break;
                }
                result_index += 1;
            }

            // Calculate actual result index
            let mut actual_index = 0;
            let mut row_count = 0;
            let mut last_category: Option<SearchCategory> = None;

            for result in results.iter() {
                if last_category.as_ref() != Some(&result.category) {
                    last_category = Some(result.category.clone());
                    row_count += 1; // Category header
                }
                if row_count == index {
                    return Some(result.clone());
                }
                row_count += 1;
                actual_index += 1;
            }

            None
        } else {
            results.get(index).cloned()
        }
    }

    /// Select next result
    pub fn select_next(&self) {
        if let Some(row) = self.list_box.selected_row() {
            let next_index = row.index() + 1;
            if let Some(next_row) = self.list_box.row_at_index(next_index) {
                // Skip non-selectable rows (category headers)
                if next_row.is_selectable() {
                    self.list_box.select_row(Some(&next_row));
                } else if let Some(skip_row) = self.list_box.row_at_index(next_index + 1) {
                    self.list_box.select_row(Some(&skip_row));
                }
            }
        }
    }

    /// Select previous result
    pub fn select_previous(&self) {
        if let Some(row) = self.list_box.selected_row() {
            let prev_index = row.index() - 1;
            if prev_index >= 0 {
                if let Some(prev_row) = self.list_box.row_at_index(prev_index) {
                    // Skip non-selectable rows (category headers)
                    if prev_row.is_selectable() {
                        self.list_box.select_row(Some(&prev_row));
                    } else if prev_index > 0 {
                        if let Some(skip_row) = self.list_box.row_at_index(prev_index - 1) {
                            self.list_box.select_row(Some(&skip_row));
                        }
                    }
                }
            }
        }
    }

    /// Clear results
    pub fn clear(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        self.results.borrow_mut().clear();
    }
}
