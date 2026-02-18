//! Image viewer widget with GPU-accelerated rendering

use crate::config::{ImageConfig, ZoomMode};
use crate::metadata::ImageMetadata;
use gtk4::gdk::RGBA;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{
    gdk, glib, EventControllerMotion, EventControllerScroll, GestureClick, GestureDrag,
    Orientation, Picture, ScrolledWindow, Overlay,
};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Rotation angle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

impl Rotation {
    pub fn rotate_cw(self) -> Self {
        match self {
            Self::None => Self::Clockwise90,
            Self::Clockwise90 => Self::Clockwise180,
            Self::Clockwise180 => Self::Clockwise270,
            Self::Clockwise270 => Self::None,
        }
    }

    pub fn rotate_ccw(self) -> Self {
        match self {
            Self::None => Self::Clockwise270,
            Self::Clockwise90 => Self::None,
            Self::Clockwise180 => Self::Clockwise90,
            Self::Clockwise270 => Self::Clockwise180,
        }
    }

    pub fn degrees(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Clockwise90 => 90.0,
            Self::Clockwise180 => 180.0,
            Self::Clockwise270 => 270.0,
        }
    }
}

/// Flip mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipMode {
    None,
    Horizontal,
    Vertical,
    Both,
}

impl FlipMode {
    pub fn flip_horizontal(self) -> Self {
        match self {
            Self::None => Self::Horizontal,
            Self::Horizontal => Self::None,
            Self::Vertical => Self::Both,
            Self::Both => Self::Vertical,
        }
    }

    pub fn flip_vertical(self) -> Self {
        match self {
            Self::None => Self::Vertical,
            Self::Horizontal => Self::Both,
            Self::Vertical => Self::None,
            Self::Both => Self::Horizontal,
        }
    }
}

/// Image viewer widget
pub struct ImageViewer {
    widget: Overlay,
    picture: Picture,
    scrolled: ScrolledWindow,
    current_path: RefCell<Option<PathBuf>>,
    current_pixbuf: RefCell<Option<Pixbuf>>,
    zoom_level: Cell<f64>,
    zoom_mode: Cell<ZoomMode>,
    rotation: Cell<Rotation>,
    flip: Cell<FlipMode>,
    pan_x: Cell<f64>,
    pan_y: Cell<f64>,
    drag_start_x: Cell<f64>,
    drag_start_y: Cell<f64>,
    on_zoom_changed: RefCell<Option<Box<dyn Fn(f64)>>>,
    on_image_loaded: RefCell<Option<Box<dyn Fn(&ImageMetadata)>>>,
}

impl ImageViewer {
    pub fn new() -> Rc<Self> {
        let picture = Picture::builder()
            .can_shrink(true)
            .keep_aspect_ratio(true)
            .content_fit(gtk4::ContentFit::Contain)
            .hexpand(true)
            .vexpand(true)
            .build();

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .kinetic_scrolling(true)
            .child(&picture)
            .build();

        // Create overlay for additional controls
        let overlay = Overlay::new();
        overlay.set_child(Some(&scrolled));

        let viewer = Rc::new(Self {
            widget: overlay,
            picture,
            scrolled,
            current_path: RefCell::new(None),
            current_pixbuf: RefCell::new(None),
            zoom_level: Cell::new(1.0),
            zoom_mode: Cell::new(ZoomMode::Fit),
            rotation: Cell::new(Rotation::None),
            flip: Cell::new(FlipMode::None),
            pan_x: Cell::new(0.0),
            pan_y: Cell::new(0.0),
            drag_start_x: Cell::new(0.0),
            drag_start_y: Cell::new(0.0),
            on_zoom_changed: RefCell::new(None),
            on_image_loaded: RefCell::new(None),
        });

        viewer.setup_gestures();

        viewer
    }

    fn setup_gestures(self: &Rc<Self>) {
        // Scroll to zoom
        let scroll_controller = EventControllerScroll::new(
            gtk4::EventControllerScrollFlags::VERTICAL |
            gtk4::EventControllerScrollFlags::DISCRETE
        );
        let viewer_weak = Rc::downgrade(self);
        scroll_controller.connect_scroll(move |controller, _dx, dy| {
            if let Some(viewer) = viewer_weak.upgrade() {
                // Check for Ctrl modifier for zoom
                if controller.current_event_state().contains(gdk::ModifierType::CONTROL_MASK) {
                    if dy < 0.0 {
                        viewer.zoom_in();
                    } else {
                        viewer.zoom_out();
                    }
                    return glib::Propagation::Stop;
                }
            }
            glib::Propagation::Proceed
        });
        self.scrolled.add_controller(scroll_controller);

        // Drag to pan
        let drag = GestureDrag::new();
        let viewer_weak = Rc::downgrade(self);
        drag.connect_drag_begin(move |_, x, y| {
            if let Some(viewer) = viewer_weak.upgrade() {
                viewer.drag_start_x.set(x);
                viewer.drag_start_y.set(y);
            }
        });

        let viewer_weak = Rc::downgrade(self);
        drag.connect_drag_update(move |_, offset_x, offset_y| {
            if let Some(viewer) = viewer_weak.upgrade() {
                viewer.pan(offset_x, offset_y);
            }
        });
        self.picture.add_controller(drag);

        // Double-click to toggle zoom
        let click = GestureClick::builder()
            .button(1)
            .build();
        click.set_propagation_phase(gtk4::PropagationPhase::Capture);

        let viewer_weak = Rc::downgrade(self);
        click.connect_pressed(move |gesture, n_press, _x, _y| {
            if n_press == 2 {
                if let Some(viewer) = viewer_weak.upgrade() {
                    viewer.toggle_fit();
                }
                gesture.set_state(gtk4::EventSequenceState::Claimed);
            }
        });
        self.picture.add_controller(click);
    }

    /// Get the widget for embedding
    pub fn widget(&self) -> &Overlay {
        &self.widget
    }

    /// Load an image from path
    pub fn load(&self, path: &Path) -> anyhow::Result<()> {
        // Load pixbuf
        let pixbuf = Pixbuf::from_file(path)?;

        // Get metadata
        let metadata = ImageMetadata::from_file(
            path,
            pixbuf.width() as u32,
            pixbuf.height() as u32,
        );

        // Apply EXIF rotation if needed
        let pixbuf = self.apply_exif_rotation(pixbuf, &metadata);

        // Store
        *self.current_path.borrow_mut() = Some(path.to_path_buf());
        *self.current_pixbuf.borrow_mut() = Some(pixbuf.clone());

        // Reset transformations
        self.rotation.set(Rotation::None);
        self.flip.set(FlipMode::None);

        // Display
        self.picture.set_pixbuf(Some(&pixbuf));

        // Calculate zoom for fit mode
        self.apply_zoom_mode();

        // Notify
        if let Some(callback) = self.on_image_loaded.borrow().as_ref() {
            callback(&metadata);
        }

        Ok(())
    }

    /// Apply EXIF orientation
    fn apply_exif_rotation(&self, pixbuf: Pixbuf, metadata: &ImageMetadata) -> Pixbuf {
        if let Some(orientation) = metadata.exif.get("Orientation") {
            // Apply EXIF orientation (simplified)
            match orientation.as_str() {
                "6" | "right - top" => {
                    if let Some(rotated) = pixbuf.rotate_simple(gtk4::gdk_pixbuf::PixbufRotation::Clockwise) {
                        return rotated;
                    }
                }
                "8" | "left - bottom" => {
                    if let Some(rotated) = pixbuf.rotate_simple(gtk4::gdk_pixbuf::PixbufRotation::Counterclockwise) {
                        return rotated;
                    }
                }
                "3" | "bottom - right" => {
                    if let Some(rotated) = pixbuf.rotate_simple(gtk4::gdk_pixbuf::PixbufRotation::Upsidedown) {
                        return rotated;
                    }
                }
                _ => {}
            }
        }
        pixbuf
    }

    /// Get current image path
    pub fn current_path(&self) -> Option<PathBuf> {
        self.current_path.borrow().clone()
    }

    /// Set zoom mode
    pub fn set_zoom_mode(&self, mode: ZoomMode) {
        self.zoom_mode.set(mode);
        self.apply_zoom_mode();
    }

    /// Apply current zoom mode
    fn apply_zoom_mode(&self) {
        let Some(pixbuf) = self.current_pixbuf.borrow().clone() else {
            return;
        };

        let widget_width = self.scrolled.width() as f64;
        let widget_height = self.scrolled.height() as f64;
        let image_width = pixbuf.width() as f64;
        let image_height = pixbuf.height() as f64;

        let zoom = match self.zoom_mode.get() {
            ZoomMode::Fit => {
                let scale_x = widget_width / image_width;
                let scale_y = widget_height / image_height;
                scale_x.min(scale_y).min(1.0)
            }
            ZoomMode::Fill => {
                let scale_x = widget_width / image_width;
                let scale_y = widget_height / image_height;
                scale_x.max(scale_y)
            }
            ZoomMode::Original => 1.0,
            ZoomMode::Custom(pct) => pct as f64 / 100.0,
        };

        self.set_zoom(zoom);
    }

    /// Set zoom level (1.0 = 100%)
    pub fn set_zoom(&self, zoom: f64) {
        let zoom = zoom.clamp(0.05, 10.0);
        self.zoom_level.set(zoom);

        // Update picture size
        if let Some(pixbuf) = self.current_pixbuf.borrow().as_ref() {
            let width = (pixbuf.width() as f64 * zoom) as i32;
            let height = (pixbuf.height() as f64 * zoom) as i32;

            self.picture.set_size_request(width, height);
        }

        // Notify
        if let Some(callback) = self.on_zoom_changed.borrow().as_ref() {
            callback(zoom);
        }
    }

    /// Get current zoom level
    pub fn zoom_level(&self) -> f64 {
        self.zoom_level.get()
    }

    /// Zoom in by step
    pub fn zoom_in(&self) {
        let current = self.zoom_level.get();
        self.set_zoom(current * 1.1);
        self.zoom_mode.set(ZoomMode::Custom((self.zoom_level.get() * 100.0) as u32));
    }

    /// Zoom out by step
    pub fn zoom_out(&self) {
        let current = self.zoom_level.get();
        self.set_zoom(current / 1.1);
        self.zoom_mode.set(ZoomMode::Custom((self.zoom_level.get() * 100.0) as u32));
    }

    /// Zoom to fit
    pub fn zoom_fit(&self) {
        self.set_zoom_mode(ZoomMode::Fit);
    }

    /// Zoom to fill
    pub fn zoom_fill(&self) {
        self.set_zoom_mode(ZoomMode::Fill);
    }

    /// Zoom to 100%
    pub fn zoom_original(&self) {
        self.set_zoom_mode(ZoomMode::Original);
    }

    /// Toggle between fit and 100%
    pub fn toggle_fit(&self) {
        if matches!(self.zoom_mode.get(), ZoomMode::Fit) {
            self.zoom_original();
        } else {
            self.zoom_fit();
        }
    }

    /// Pan the image
    fn pan(&self, dx: f64, dy: f64) {
        if let Some(hadj) = self.scrolled.hadjustment() {
            hadj.set_value(hadj.value() - dx);
        }
        if let Some(vadj) = self.scrolled.vadjustment() {
            vadj.set_value(vadj.value() - dy);
        }
    }

    /// Rotate clockwise
    pub fn rotate_cw(&self) {
        let rotation = self.rotation.get().rotate_cw();
        self.rotation.set(rotation);
        self.apply_transformations();
    }

    /// Rotate counter-clockwise
    pub fn rotate_ccw(&self) {
        let rotation = self.rotation.get().rotate_ccw();
        self.rotation.set(rotation);
        self.apply_transformations();
    }

    /// Flip horizontal
    pub fn flip_horizontal(&self) {
        let flip = self.flip.get().flip_horizontal();
        self.flip.set(flip);
        self.apply_transformations();
    }

    /// Flip vertical
    pub fn flip_vertical(&self) {
        let flip = self.flip.get().flip_vertical();
        self.flip.set(flip);
        self.apply_transformations();
    }

    /// Apply rotation and flip transformations
    fn apply_transformations(&self) {
        let Some(original) = self.current_pixbuf.borrow().clone() else {
            return;
        };

        let mut pixbuf = original;

        // Apply rotation
        pixbuf = match self.rotation.get() {
            Rotation::None => pixbuf,
            Rotation::Clockwise90 => {
                pixbuf.rotate_simple(gtk4::gdk_pixbuf::PixbufRotation::Clockwise)
                    .unwrap_or(pixbuf)
            }
            Rotation::Clockwise180 => {
                pixbuf.rotate_simple(gtk4::gdk_pixbuf::PixbufRotation::Upsidedown)
                    .unwrap_or(pixbuf)
            }
            Rotation::Clockwise270 => {
                pixbuf.rotate_simple(gtk4::gdk_pixbuf::PixbufRotation::Counterclockwise)
                    .unwrap_or(pixbuf)
            }
        };

        // Apply flip
        pixbuf = match self.flip.get() {
            FlipMode::None => pixbuf,
            FlipMode::Horizontal => {
                pixbuf.flip(true).unwrap_or(pixbuf)
            }
            FlipMode::Vertical => {
                pixbuf.flip(false).unwrap_or(pixbuf)
            }
            FlipMode::Both => {
                let h = pixbuf.flip(true).unwrap_or(pixbuf.clone());
                h.flip(false).unwrap_or(h)
            }
        };

        self.picture.set_pixbuf(Some(&pixbuf));
        self.apply_zoom_mode();
    }

    /// Reset all transformations
    pub fn reset_transformations(&self) {
        self.rotation.set(Rotation::None);
        self.flip.set(FlipMode::None);

        if let Some(pixbuf) = self.current_pixbuf.borrow().as_ref() {
            self.picture.set_pixbuf(Some(pixbuf));
            self.apply_zoom_mode();
        }
    }

    /// Connect zoom changed callback
    pub fn connect_zoom_changed<F: Fn(f64) + 'static>(&self, callback: F) {
        *self.on_zoom_changed.borrow_mut() = Some(Box::new(callback));
    }

    /// Connect image loaded callback
    pub fn connect_image_loaded<F: Fn(&ImageMetadata) + 'static>(&self, callback: F) {
        *self.on_image_loaded.borrow_mut() = Some(Box::new(callback));
    }

    /// Check if an image is loaded
    pub fn has_image(&self) -> bool {
        self.current_path.borrow().is_some()
    }

    /// Clear the viewer
    pub fn clear(&self) {
        *self.current_path.borrow_mut() = None;
        *self.current_pixbuf.borrow_mut() = None;
        self.picture.set_pixbuf(None::<&Pixbuf>);
        self.rotation.set(Rotation::None);
        self.flip.set(FlipMode::None);
        self.zoom_level.set(1.0);
    }
}

impl Default for ImageViewer {
    fn default() -> Rc<Self> {
        Self::new()
    }
}
