//! Thumbnail strip widget for navigation

use crate::config::ImageConfig;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{glib, Box, EventControllerScroll, FlowBox, FlowBoxChild, Image, Orientation, ScrolledWindow};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Thumbnail cache entry
#[derive(Clone)]
struct ThumbnailCacheEntry {
    pixbuf: Pixbuf,
    path: PathBuf,
}

/// Thumbnail strip widget
pub struct ThumbnailStrip {
    widget: ScrolledWindow,
    flow_box: FlowBox,
    images: RefCell<Vec<PathBuf>>,
    current_index: Cell<i32>,
    thumbnail_size: Cell<u32>,
    cache: RefCell<HashMap<PathBuf, Pixbuf>>,
    on_selected: RefCell<Option<Box<dyn Fn(usize)>>>,
}

impl ThumbnailStrip {
    pub fn new() -> Rc<Self> {
        let flow_box = FlowBox::builder()
            .orientation(Orientation::Horizontal)
            .max_children_per_line(1000)
            .min_children_per_line(1)
            .selection_mode(gtk4::SelectionMode::Single)
            .homogeneous(true)
            .row_spacing(4)
            .column_spacing(4)
            .build();

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .height_request(100)
            .child(&flow_box)
            .build();

        // Enable horizontal scrolling with mouse wheel
        let scroll_controller = EventControllerScroll::new(
            gtk4::EventControllerScrollFlags::VERTICAL |
            gtk4::EventControllerScrollFlags::HORIZONTAL
        );
        let scrolled_weak = scrolled.downgrade();
        scroll_controller.connect_scroll(move |_, dx, dy| {
            if let Some(scrolled) = scrolled_weak.upgrade() {
                if let Some(adj) = scrolled.hadjustment() {
                    adj.set_value(adj.value() + dy * 50.0);
                }
            }
            glib::Propagation::Stop
        });
        scrolled.add_controller(scroll_controller);

        let strip = Rc::new(Self {
            widget: scrolled,
            flow_box,
            images: RefCell::new(Vec::new()),
            current_index: Cell::new(-1),
            thumbnail_size: Cell::new(80),
            cache: RefCell::new(HashMap::new()),
            on_selected: RefCell::new(None),
        });

        // Connect selection handler
        let strip_weak = Rc::downgrade(&strip);
        strip.flow_box.connect_child_activated(move |_, child| {
            if let Some(strip) = strip_weak.upgrade() {
                let index = child.index() as usize;
                strip.current_index.set(index as i32);
                if let Some(callback) = strip.on_selected.borrow().as_ref() {
                    callback(index);
                }
            }
        });

        strip
    }

    /// Get the widget for embedding
    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    /// Set thumbnail size
    pub fn set_thumbnail_size(&self, size: u32) {
        self.thumbnail_size.set(size);
        self.widget.set_height_request(size as i32 + 20);
    }

    /// Set callback for thumbnail selection
    pub fn connect_selected<F: Fn(usize) + 'static>(&self, callback: F) {
        *self.on_selected.borrow_mut() = Some(Box::new(callback));
    }

    /// Load images from a directory or file list
    pub fn load_images(&self, images: Vec<PathBuf>) {
        // Clear existing
        self.clear();

        *self.images.borrow_mut() = images.clone();

        // Add thumbnails
        let size = self.thumbnail_size.get();
        for (index, path) in images.iter().enumerate() {
            self.add_thumbnail(path, index, size);
        }
    }

    /// Add a single thumbnail
    fn add_thumbnail(&self, path: &Path, index: usize, size: u32) {
        let image = Image::builder()
            .pixel_size(size as i32)
            .css_classes(["thumbnail"])
            .build();

        // Set placeholder icon first
        image.set_from_icon_name(Some("image-x-generic-symbolic"));

        // Load actual thumbnail async
        let path_clone = path.to_path_buf();
        let image_weak = image.downgrade();
        let size_clone = size;
        let cache_ref = self.cache.clone();

        glib::spawn_future_local(async move {
            if let Some(image) = image_weak.upgrade() {
                // Check cache first
                if let Some(pixbuf) = cache_ref.borrow().get(&path_clone) {
                    image.set_from_pixbuf(Some(pixbuf));
                    return;
                }

                // Load thumbnail on blocking thread
                let path = path_clone.clone();
                let result = tokio::task::spawn_blocking(move || {
                    Self::generate_thumbnail(&path, size_clone)
                }).await;

                if let Ok(Some(pixbuf)) = result {
                    // Cache and display
                    cache_ref.borrow_mut().insert(path_clone, pixbuf.clone());
                    image.set_from_pixbuf(Some(&pixbuf));
                }
            }
        });

        // Wrap in frame for selection highlight
        let frame = gtk4::Frame::builder()
            .child(&image)
            .css_classes(["thumbnail-frame"])
            .build();

        self.flow_box.append(&frame);
    }

    /// Generate a thumbnail for an image
    fn generate_thumbnail(path: &Path, size: u32) -> Option<Pixbuf> {
        Pixbuf::from_file_at_scale(
            path,
            size as i32,
            size as i32,
            true,
        ).ok()
    }

    /// Select thumbnail by index
    pub fn select(&self, index: usize) {
        if index >= self.images.borrow().len() {
            return;
        }

        self.current_index.set(index as i32);

        if let Some(child) = self.flow_box.child_at_index(index as i32) {
            self.flow_box.select_child(&child);
            // Scroll to make visible
            child.grab_focus();
        }
    }

    /// Get current selection index
    pub fn current_index(&self) -> Option<usize> {
        let index = self.current_index.get();
        if index >= 0 {
            Some(index as usize)
        } else {
            None
        }
    }

    /// Get path at index
    pub fn path_at(&self, index: usize) -> Option<PathBuf> {
        self.images.borrow().get(index).cloned()
    }

    /// Get total count
    pub fn count(&self) -> usize {
        self.images.borrow().len()
    }

    /// Clear all thumbnails
    pub fn clear(&self) {
        while let Some(child) = self.flow_box.first_child() {
            self.flow_box.remove(&child);
        }
        self.images.borrow_mut().clear();
        self.current_index.set(-1);
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
    }

    /// Navigate to next
    pub fn next(&self) -> Option<usize> {
        let current = self.current_index.get();
        let count = self.images.borrow().len() as i32;
        if count == 0 {
            return None;
        }

        let next = if current < 0 {
            0
        } else if current < count - 1 {
            current + 1
        } else {
            current // Stay at end
        };

        self.select(next as usize);
        Some(next as usize)
    }

    /// Navigate to previous
    pub fn previous(&self) -> Option<usize> {
        let current = self.current_index.get();
        let count = self.images.borrow().len() as i32;
        if count == 0 {
            return None;
        }

        let prev = if current <= 0 {
            0
        } else {
            current - 1
        };

        self.select(prev as usize);
        Some(prev as usize)
    }

    /// Check if can go next
    pub fn can_next(&self) -> bool {
        let current = self.current_index.get();
        let count = self.images.borrow().len() as i32;
        current < count - 1
    }

    /// Check if can go previous
    pub fn can_previous(&self) -> bool {
        self.current_index.get() > 0
    }
}

impl Default for ThumbnailStrip {
    fn default() -> Rc<Self> {
        Self::new()
    }
}
