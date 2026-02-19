// Winux Welcome - Page Indicator
// Dot indicators showing current page in the wizard

use gtk4::prelude::*;
use gtk4::{Box, Orientation};

/// Creates a page indicator with dots for each page
pub fn create_page_indicator(total_pages: usize) -> Box {
    let indicator = Box::new(Orientation::Horizontal, 8);
    indicator.set_halign(gtk4::Align::Center);
    indicator.set_margin_top(16);
    indicator.set_margin_bottom(16);

    for i in 0..total_pages {
        let dot = gtk4::DrawingArea::new();
        dot.set_size_request(10, 10);
        dot.add_css_class("page-dot");

        if i == 0 {
            dot.add_css_class("active");
        }

        indicator.append(&dot);
    }

    indicator
}

/// Updates the page indicator to show the current page
pub fn update_indicator(indicator: &Box, current_page: usize) {
    let mut child = indicator.first_child();
    let mut index = 0;

    while let Some(widget) = child {
        if let Some(dot) = widget.downcast_ref::<gtk4::DrawingArea>() {
            if index == current_page {
                dot.add_css_class("active");
            } else {
                dot.remove_css_class("active");
            }
        }
        child = widget.next_sibling();
        index += 1;
    }
}
