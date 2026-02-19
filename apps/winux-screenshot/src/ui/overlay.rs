//! Overlay for region selection

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, Window, DrawingArea, glib};
use gtk::gdk::Display;
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::capture::{CaptureResult, region::Region};

/// Selection state during region capture
struct SelectionState {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    is_selecting: bool,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            start_x: 0.0,
            start_y: 0.0,
            end_x: 0.0,
            end_y: 0.0,
            is_selecting: false,
        }
    }
}

/// Show fullscreen overlay for region selection
pub fn show_region_overlay<F>(
    app: &Application,
    _state: &Rc<RefCell<AppState>>,
    callback: F,
)
where
    F: Fn(Option<CaptureResult>) + 'static,
{
    let callback = Rc::new(RefCell::new(Some(callback)));
    let selection = Rc::new(RefCell::new(SelectionState::default()));

    // Get screen dimensions
    let display = Display::default().expect("Could not get display");
    let monitors = display.monitors();
    let monitor = monitors.item(0)
        .and_then(|m| m.downcast::<gtk::gdk::Monitor>().ok())
        .expect("Could not get monitor");

    let geometry = monitor.geometry();
    let screen_width = geometry.width();
    let screen_height = geometry.height();

    // Create fullscreen overlay window
    let window = Window::builder()
        .application(app)
        .decorated(false)
        .default_width(screen_width)
        .default_height(screen_height)
        .build();

    // Make window fullscreen and transparent
    window.fullscreen();

    // Drawing area for the overlay
    let drawing_area = DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    // Set up drawing
    {
        let selection = selection.clone();
        drawing_area.set_draw_func(move |_, cr, width, height| {
            let sel = selection.borrow();

            // Semi-transparent dark overlay
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            cr.paint().ok();

            if sel.is_selecting || (sel.start_x != sel.end_x && sel.start_y != sel.end_y) {
                let x = sel.start_x.min(sel.end_x);
                let y = sel.start_y.min(sel.end_y);
                let w = (sel.end_x - sel.start_x).abs();
                let h = (sel.end_y - sel.start_y).abs();

                // Clear the selected region (show through)
                cr.set_operator(cairo::Operator::Clear);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();

                // Reset operator
                cr.set_operator(cairo::Operator::Over);

                // Draw selection border
                cr.set_source_rgba(0.2, 0.6, 1.0, 1.0);
                cr.set_line_width(2.0);
                cr.rectangle(x, y, w, h);
                cr.stroke().ok();

                // Draw corner handles
                let handle_size = 8.0;
                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);

                // Top-left
                cr.rectangle(x - handle_size / 2.0, y - handle_size / 2.0, handle_size, handle_size);
                cr.fill().ok();

                // Top-right
                cr.rectangle(x + w - handle_size / 2.0, y - handle_size / 2.0, handle_size, handle_size);
                cr.fill().ok();

                // Bottom-left
                cr.rectangle(x - handle_size / 2.0, y + h - handle_size / 2.0, handle_size, handle_size);
                cr.fill().ok();

                // Bottom-right
                cr.rectangle(x + w - handle_size / 2.0, y + h - handle_size / 2.0, handle_size, handle_size);
                cr.fill().ok();

                // Dimensions label
                if w > 50.0 && h > 30.0 {
                    let label = format!("{}x{}", w as i32, h as i32);
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    cr.set_font_size(14.0);

                    // Background for label
                    let extents = cr.text_extents(&label).unwrap();
                    let label_x = x + w / 2.0 - extents.width() / 2.0;
                    let label_y = y + h / 2.0 + extents.height() / 2.0;

                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.7);
                    cr.rectangle(
                        label_x - 4.0,
                        label_y - extents.height() - 4.0,
                        extents.width() + 8.0,
                        extents.height() + 8.0,
                    );
                    cr.fill().ok();

                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    cr.move_to(label_x, label_y);
                    cr.show_text(&label).ok();
                }
            }

            // Draw crosshair
            if !sel.is_selecting {
                draw_crosshair_instructions(cr, width as f64, height as f64);
            }
        });
    }

    window.set_child(Some(&drawing_area));

    // Mouse handling
    let gesture = gtk::GestureDrag::new();

    {
        let selection = selection.clone();
        let drawing_area = drawing_area.clone();

        gesture.connect_drag_begin(move |_, x, y| {
            let mut sel = selection.borrow_mut();
            sel.start_x = x;
            sel.start_y = y;
            sel.end_x = x;
            sel.end_y = y;
            sel.is_selecting = true;
            drawing_area.queue_draw();
        });
    }

    {
        let selection = selection.clone();
        let drawing_area = drawing_area.clone();

        gesture.connect_drag_update(move |_, offset_x, offset_y| {
            let mut sel = selection.borrow_mut();
            sel.end_x = sel.start_x + offset_x;
            sel.end_y = sel.start_y + offset_y;
            drawing_area.queue_draw();
        });
    }

    {
        let selection = selection.clone();
        let window = window.clone();
        let callback = callback.clone();

        gesture.connect_drag_end(move |_, offset_x, offset_y| {
            let sel = selection.borrow();
            let end_x = sel.start_x + offset_x;
            let end_y = sel.start_y + offset_y;

            let region = Region::from_points(
                sel.start_x as i32,
                sel.start_y as i32,
                end_x as i32,
                end_y as i32,
            );

            window.close();

            if region.is_valid() {
                // Small delay to ensure overlay is gone
                let callback = callback.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    match crate::capture::region::capture_region_direct(&region) {
                        Ok(result) => {
                            if let Some(cb) = callback.borrow_mut().take() {
                                cb(Some(result));
                            }
                        }
                        Err(e) => {
                            eprintln!("Region capture failed: {}", e);
                            if let Some(cb) = callback.borrow_mut().take() {
                                cb(None);
                            }
                        }
                    }
                });
            } else {
                if let Some(cb) = callback.borrow_mut().take() {
                    cb(None);
                }
            }
        });
    }

    drawing_area.add_controller(gesture);

    // Keyboard handling (Escape to cancel)
    let key_controller = gtk::EventControllerKey::new();
    {
        let window = window.clone();
        let callback = callback.clone();

        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                window.close();
                if let Some(cb) = callback.borrow_mut().take() {
                    cb(None);
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
    }
    window.add_controller(key_controller);

    // Set cursor to crosshair
    window.set_cursor_from_name(Some("crosshair"));

    window.present();
}

fn draw_crosshair_instructions(cr: &cairo::Context, width: f64, height: f64) {
    // Instructions at the top center
    let instructions = "Click and drag to select region | Press Escape to cancel";

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    cr.set_font_size(16.0);

    let extents = cr.text_extents(instructions).unwrap();
    let x = width / 2.0 - extents.width() / 2.0;
    let y = 50.0;

    // Background
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.8);
    cr.rectangle(x - 12.0, y - extents.height() - 8.0, extents.width() + 24.0, extents.height() + 16.0);
    // Rounded corners
    cr.fill().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    cr.move_to(x, y);
    cr.show_text(instructions).ok();
}
