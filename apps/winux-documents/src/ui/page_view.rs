//! Page view widget for document rendering

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{DrawingArea, Box as GtkBox, Orientation, GestureClick, EventControllerScroll};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::{AppState, FitMode};
use crate::features::search;

/// Page view widget
#[derive(Clone)]
pub struct PageView {
    container: GtkBox,
    drawing_area: DrawingArea,
    state: Rc<RefCell<AppState>>,
}

impl PageView {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        // Container box for centering
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);
        container.append(&drawing_area);

        let view = Self {
            container,
            drawing_area,
            state,
        };

        view.setup_drawing();
        view.setup_gestures();

        view
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    fn setup_drawing(&self) {
        let state = self.state.clone();

        self.drawing_area.set_draw_func(move |_area, context, width, height| {
            let app_state = state.borrow();

            // Clear background
            if app_state.night_mode {
                context.set_source_rgb(0.1, 0.1, 0.1);
            } else {
                context.set_source_rgb(0.3, 0.3, 0.3);
            }
            context.paint().ok();

            // Render document page if available
            if let Some(ref document) = app_state.document {
                let page = app_state.current_page;

                // Calculate scale based on fit mode
                let scale = match app_state.fit_mode {
                    FitMode::None => app_state.zoom_level,
                    FitMode::Page => {
                        if let Some((page_width, page_height)) = document.page_size(page) {
                            let scale_x = (width as f64 - 20.0) / page_width;
                            let scale_y = (height as f64 - 20.0) / page_height;
                            scale_x.min(scale_y).min(5.0).max(0.1)
                        } else {
                            app_state.zoom_level
                        }
                    }
                    FitMode::Width => {
                        if let Some((page_width, _)) = document.page_size(page) {
                            ((width as f64 - 20.0) / page_width).min(5.0).max(0.1)
                        } else {
                            app_state.zoom_level
                        }
                    }
                };

                // Render the page
                if let Some(surface) = document.render_page(page, scale) {
                    let page_width = surface.width() as f64;
                    let page_height = surface.height() as f64;

                    // Center the page
                    let x = (width as f64 - page_width) / 2.0;
                    let y = (height as f64 - page_height) / 2.0;

                    // Draw shadow
                    context.set_source_rgba(0.0, 0.0, 0.0, 0.3);
                    context.rectangle(x + 4.0, y + 4.0, page_width, page_height);
                    context.fill().ok();

                    // Draw page
                    context.set_source_surface(&surface, x, y).ok();

                    // Apply night mode filter
                    if app_state.night_mode {
                        context.paint().ok();

                        // Invert colors using blend mode
                        context.set_operator(cairo::Operator::Difference);
                        context.set_source_rgb(1.0, 1.0, 1.0);
                        context.rectangle(x, y, page_width, page_height);
                        context.fill().ok();
                        context.set_operator(cairo::Operator::Over);
                    } else {
                        context.paint().ok();
                    }

                    // Draw search highlights
                    context.save().ok();
                    context.translate(x, y);
                    search::render_search_highlights(
                        context,
                        &app_state.search_state,
                        page,
                        scale,
                        true,
                    );
                    context.restore().ok();

                    // Draw annotations
                    context.save().ok();
                    context.translate(x, y);
                    app_state.annotations.render(context, page, scale);
                    context.restore().ok();

                    // Draw border
                    context.set_source_rgb(0.2, 0.2, 0.2);
                    context.set_line_width(1.0);
                    context.rectangle(x, y, page_width, page_height);
                    context.stroke().ok();
                }
            } else {
                // No document loaded - show placeholder
                context.set_source_rgb(0.5, 0.5, 0.5);
                context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
                context.set_font_size(18.0);

                let text = "Open a document to view";
                let extents = context.text_extents(text).ok();
                if let Some(ext) = extents {
                    context.move_to(
                        (width as f64 - ext.width()) / 2.0,
                        (height as f64 + ext.height()) / 2.0,
                    );
                    context.show_text(text).ok();
                }
            }
        });
    }

    fn setup_gestures(&self) {
        // Click gesture for annotations
        let click_gesture = GestureClick::new();
        click_gesture.set_button(0); // All buttons

        let state = self.state.clone();
        let drawing_area = self.drawing_area.clone();

        click_gesture.connect_pressed(move |gesture, n_press, x, y| {
            let button = gesture.current_button();

            if button == 1 && n_press == 2 {
                // Double-click: toggle fullscreen or other action
            }

            // Right-click: context menu
            if button == 3 {
                // Would show context menu here
            }
        });

        self.drawing_area.add_controller(click_gesture);

        // Scroll gesture for zooming with Ctrl
        let scroll_controller = EventControllerScroll::new(
            gtk::EventControllerScrollFlags::VERTICAL |
            gtk::EventControllerScrollFlags::DISCRETE
        );

        let state = self.state.clone();
        let drawing_area = self.drawing_area.clone();

        scroll_controller.connect_scroll(move |controller, _dx, dy| {
            let modifiers = controller.current_event_state();

            if modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                // Zoom with Ctrl+scroll
                let mut app_state = state.borrow_mut();

                if dy < 0.0 {
                    app_state.zoom_level = (app_state.zoom_level * 1.1).min(5.0);
                } else {
                    app_state.zoom_level = (app_state.zoom_level / 1.1).max(0.1);
                }
                app_state.fit_mode = FitMode::None;

                drop(app_state);
                drawing_area.queue_draw();

                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });

        self.drawing_area.add_controller(scroll_controller);
    }

    pub fn render_current_page(&self) {
        self.drawing_area.queue_draw();
    }

    pub fn update_zoom(&self) {
        self.drawing_area.queue_draw();
    }

    pub fn set_page(&self, page: usize) {
        {
            let mut state = self.state.borrow_mut();
            if page < state.total_pages {
                state.current_page = page;
            }
        }
        self.render_current_page();
    }

    pub fn get_visible_rect(&self) -> (f64, f64, f64, f64) {
        let width = self.drawing_area.width() as f64;
        let height = self.drawing_area.height() as f64;
        (0.0, 0.0, width, height)
    }
}
