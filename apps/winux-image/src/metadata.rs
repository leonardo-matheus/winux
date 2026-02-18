//! EXIF and image metadata handling

use gtk4::prelude::*;
use gtk4::{glib, Box, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use std::collections::HashMap;
use std::path::Path;

/// Image metadata container
#[derive(Debug, Clone, Default)]
pub struct ImageMetadata {
    /// Basic file info
    pub filename: String,
    pub file_size: u64,
    pub file_modified: String,
    /// Image dimensions
    pub width: u32,
    pub height: u32,
    pub color_depth: Option<u32>,
    /// Format info
    pub format: String,
    pub mime_type: String,
    /// EXIF data
    pub exif: HashMap<String, String>,
}

impl ImageMetadata {
    /// Create metadata from file path and image info
    pub fn from_file(path: &Path, width: u32, height: u32) -> Self {
        let mut metadata = Self {
            width,
            height,
            ..Default::default()
        };

        // Basic file info
        if let Some(name) = path.file_name() {
            metadata.filename = name.to_string_lossy().to_string();
        }

        if let Ok(meta) = std::fs::metadata(path) {
            metadata.file_size = meta.len();
            if let Ok(modified) = meta.modified() {
                let datetime: chrono::DateTime<chrono::Local> = modified.into();
                metadata.file_modified = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
            }
        }

        // Format from extension
        if let Some(ext) = path.extension() {
            metadata.format = ext.to_string_lossy().to_uppercase();
        }

        // MIME type
        metadata.mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // Read EXIF data
        metadata.read_exif(path);

        metadata
    }

    /// Read EXIF data from file
    fn read_exif(&mut self, path: &Path) {
        if let Ok(file) = std::fs::File::open(path) {
            let mut bufreader = std::io::BufReader::new(&file);
            if let Ok(exif_reader) = exif::Reader::new().read_from_container(&mut bufreader) {
                for field in exif_reader.fields() {
                    let tag_name = field.tag.to_string();
                    let value = field.display_value().with_unit(&exif_reader).to_string();
                    self.exif.insert(tag_name, value);
                }
            }
        }
    }

    /// Get camera make and model
    pub fn camera(&self) -> Option<String> {
        let make = self.exif.get("Make")?;
        let model = self.exif.get("Model")?;
        Some(format!("{} {}", make.trim(), model.trim()))
    }

    /// Get exposure info
    pub fn exposure(&self) -> Option<String> {
        let time = self.exif.get("ExposureTime")?;
        let aperture = self.exif.get("FNumber");
        let iso = self.exif.get("ISOSpeedRatings");

        let mut info = time.clone();
        if let Some(f) = aperture {
            info.push_str(&format!(" | f/{}", f));
        }
        if let Some(i) = iso {
            info.push_str(&format!(" | ISO {}", i));
        }
        Some(info)
    }

    /// Get date taken
    pub fn date_taken(&self) -> Option<&String> {
        self.exif.get("DateTimeOriginal")
            .or_else(|| self.exif.get("DateTime"))
    }

    /// Get GPS coordinates
    pub fn gps(&self) -> Option<(f64, f64)> {
        // Simplified GPS parsing - full implementation would parse DMS format
        let lat = self.exif.get("GPSLatitude")?;
        let lon = self.exif.get("GPSLongitude")?;
        // This is a placeholder - real parsing is more complex
        Some((0.0, 0.0))
    }

    /// Format file size for display
    pub fn formatted_size(&self) -> String {
        let size = self.file_size as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Format dimensions for display
    pub fn formatted_dimensions(&self) -> String {
        let megapixels = (self.width as f64 * self.height as f64) / 1_000_000.0;
        format!("{} x {} ({:.1} MP)", self.width, self.height, megapixels)
    }
}

/// Metadata panel widget
pub struct MetadataPanel {
    widget: ScrolledWindow,
    list_box: ListBox,
}

impl MetadataPanel {
    pub fn new() -> Self {
        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_width(280)
            .child(&list_box)
            .build();

        Self {
            widget: scrolled,
            list_box,
        }
    }

    /// Get the widget for embedding
    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    /// Update displayed metadata
    pub fn update(&self, metadata: &ImageMetadata) {
        // Clear existing rows
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        // File section
        self.add_section_header("File Information");
        self.add_row("Name", &metadata.filename);
        self.add_row("Size", &metadata.formatted_size());
        self.add_row("Modified", &metadata.file_modified);
        self.add_row("Format", &metadata.format);

        // Image section
        self.add_section_header("Image Details");
        self.add_row("Dimensions", &metadata.formatted_dimensions());
        if let Some(depth) = metadata.color_depth {
            self.add_row("Color Depth", &format!("{} bit", depth));
        }

        // EXIF section (if available)
        if !metadata.exif.is_empty() {
            self.add_section_header("Camera Information");

            if let Some(camera) = metadata.camera() {
                self.add_row("Camera", &camera);
            }
            if let Some(exposure) = metadata.exposure() {
                self.add_row("Exposure", &exposure);
            }
            if let Some(date) = metadata.date_taken() {
                self.add_row("Date Taken", date);
            }
            if let Some(lens) = metadata.exif.get("LensModel") {
                self.add_row("Lens", lens);
            }
            if let Some(focal) = metadata.exif.get("FocalLength") {
                self.add_row("Focal Length", focal);
            }
            if let Some(flash) = metadata.exif.get("Flash") {
                self.add_row("Flash", flash);
            }
            if let Some(wb) = metadata.exif.get("WhiteBalance") {
                self.add_row("White Balance", wb);
            }
            if let Some(software) = metadata.exif.get("Software") {
                self.add_row("Software", software);
            }
        }
    }

    /// Add a section header
    fn add_section_header(&self, title: &str) {
        let label = Label::builder()
            .label(title)
            .xalign(0.0)
            .css_classes(["heading"])
            .margin_top(12)
            .margin_bottom(6)
            .margin_start(12)
            .build();

        let row = ListBoxRow::builder()
            .selectable(false)
            .activatable(false)
            .child(&label)
            .build();

        self.list_box.append(&row);
    }

    /// Add a key-value row
    fn add_row(&self, key: &str, value: &str) {
        let hbox = Box::new(Orientation::Horizontal, 12);
        hbox.set_margin_start(12);
        hbox.set_margin_end(12);
        hbox.set_margin_top(6);
        hbox.set_margin_bottom(6);

        let key_label = Label::builder()
            .label(key)
            .xalign(0.0)
            .css_classes(["dim-label"])
            .width_chars(12)
            .build();

        let value_label = Label::builder()
            .label(value)
            .xalign(0.0)
            .hexpand(true)
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::WordChar)
            .selectable(true)
            .build();

        hbox.append(&key_label);
        hbox.append(&value_label);

        let row = ListBoxRow::builder()
            .selectable(false)
            .activatable(false)
            .child(&hbox)
            .build();

        self.list_box.append(&row);
    }

    /// Clear all metadata
    pub fn clear(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
    }
}

impl Default for MetadataPanel {
    fn default() -> Self {
        Self::new()
    }
}
