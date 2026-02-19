//! Document annotations - highlights, notes, drawings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Annotation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    Highlight {
        color: HighlightColor,
        rects: Vec<AnnotationRect>,
    },
    Note {
        position: (f64, f64),
        text: String,
    },
    Drawing {
        color: DrawingColor,
        stroke_width: f64,
        points: Vec<(f64, f64)>,
    },
    Underline {
        rects: Vec<AnnotationRect>,
    },
    Strikethrough {
        rects: Vec<AnnotationRect>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HighlightColor {
    Yellow,
    Green,
    Blue,
    Pink,
    Orange,
}

impl HighlightColor {
    pub fn to_rgba(&self) -> (f64, f64, f64, f64) {
        match self {
            HighlightColor::Yellow => (1.0, 1.0, 0.0, 0.4),
            HighlightColor::Green => (0.0, 1.0, 0.0, 0.4),
            HighlightColor::Blue => (0.0, 0.5, 1.0, 0.4),
            HighlightColor::Pink => (1.0, 0.4, 0.7, 0.4),
            HighlightColor::Orange => (1.0, 0.6, 0.0, 0.4),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DrawingColor {
    Black,
    Red,
    Blue,
    Green,
}

impl DrawingColor {
    pub fn to_rgb(&self) -> (f64, f64, f64) {
        match self {
            DrawingColor::Black => (0.0, 0.0, 0.0),
            DrawingColor::Red => (1.0, 0.0, 0.0),
            DrawingColor::Blue => (0.0, 0.0, 1.0),
            DrawingColor::Green => (0.0, 0.5, 0.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Single annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: u64,
    pub page: usize,
    pub annotation_type: AnnotationType,
    pub created_at: i64,
    pub modified_at: i64,
}

/// Collection of annotations for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotations {
    document_path: Option<String>,
    annotations: HashMap<usize, Vec<Annotation>>,
    next_id: u64,
    #[serde(skip)]
    dirty: bool,
}

impl Annotations {
    pub fn new() -> Self {
        Self {
            document_path: None,
            annotations: HashMap::new(),
            next_id: 1,
            dirty: false,
        }
    }

    pub fn load_for_document(path: &Path) -> Self {
        let annotations_path = Self::annotations_file_path(path);

        if let Ok(content) = std::fs::read_to_string(&annotations_path) {
            if let Ok(annotations) = serde_json::from_str(&content) {
                return annotations;
            }
        }

        let mut annotations = Self::new();
        annotations.document_path = Some(path.to_string_lossy().to_string());
        annotations
    }

    fn annotations_file_path(doc_path: &Path) -> std::path::PathBuf {
        let annotations_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("winux-documents")
            .join("annotations");

        let _ = std::fs::create_dir_all(&annotations_dir);

        // Use hash of document path as filename
        let hash = format!("{:x}", md5_hash(doc_path.to_string_lossy().as_bytes()));
        annotations_dir.join(format!("{}.json", hash))
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(ref doc_path) = self.document_path {
            let annotations_path = Self::annotations_file_path(Path::new(doc_path));
            let content = serde_json::to_string_pretty(self)?;
            std::fs::write(annotations_path, content)?;
        }
        Ok(())
    }

    /// Add a highlight annotation
    pub fn add_highlight(&mut self, page: usize, color: HighlightColor, rects: Vec<AnnotationRect>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let now = chrono::Utc::now().timestamp();
        let annotation = Annotation {
            id,
            page,
            annotation_type: AnnotationType::Highlight { color, rects },
            created_at: now,
            modified_at: now,
        };

        self.annotations.entry(page).or_default().push(annotation);
        self.dirty = true;
        id
    }

    /// Add a note annotation
    pub fn add_note(&mut self, page: usize, position: (f64, f64), text: String) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let now = chrono::Utc::now().timestamp();
        let annotation = Annotation {
            id,
            page,
            annotation_type: AnnotationType::Note { position, text },
            created_at: now,
            modified_at: now,
        };

        self.annotations.entry(page).or_default().push(annotation);
        self.dirty = true;
        id
    }

    /// Add a drawing annotation
    pub fn add_drawing(&mut self, page: usize, color: DrawingColor, stroke_width: f64, points: Vec<(f64, f64)>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let now = chrono::Utc::now().timestamp();
        let annotation = Annotation {
            id,
            page,
            annotation_type: AnnotationType::Drawing { color, stroke_width, points },
            created_at: now,
            modified_at: now,
        };

        self.annotations.entry(page).or_default().push(annotation);
        self.dirty = true;
        id
    }

    /// Add an underline annotation
    pub fn add_underline(&mut self, page: usize, rects: Vec<AnnotationRect>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let now = chrono::Utc::now().timestamp();
        let annotation = Annotation {
            id,
            page,
            annotation_type: AnnotationType::Underline { rects },
            created_at: now,
            modified_at: now,
        };

        self.annotations.entry(page).or_default().push(annotation);
        self.dirty = true;
        id
    }

    /// Add a strikethrough annotation
    pub fn add_strikethrough(&mut self, page: usize, rects: Vec<AnnotationRect>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let now = chrono::Utc::now().timestamp();
        let annotation = Annotation {
            id,
            page,
            annotation_type: AnnotationType::Strikethrough { rects },
            created_at: now,
            modified_at: now,
        };

        self.annotations.entry(page).or_default().push(annotation);
        self.dirty = true;
        id
    }

    /// Get annotations for a specific page
    pub fn get_page_annotations(&self, page: usize) -> &[Annotation] {
        self.annotations.get(&page).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all annotations
    pub fn all_annotations(&self) -> impl Iterator<Item = &Annotation> {
        self.annotations.values().flatten()
    }

    /// Remove an annotation by ID
    pub fn remove(&mut self, id: u64) -> bool {
        for annotations in self.annotations.values_mut() {
            if let Some(pos) = annotations.iter().position(|a| a.id == id) {
                annotations.remove(pos);
                self.dirty = true;
                return true;
            }
        }
        false
    }

    /// Update a note's text
    pub fn update_note(&mut self, id: u64, new_text: String) -> bool {
        for annotations in self.annotations.values_mut() {
            for annotation in annotations.iter_mut() {
                if annotation.id == id {
                    if let AnnotationType::Note { text, .. } = &mut annotation.annotation_type {
                        *text = new_text;
                        annotation.modified_at = chrono::Utc::now().timestamp();
                        self.dirty = true;
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark as saved
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Render annotations on a Cairo context
    pub fn render(&self, context: &cairo::Context, page: usize, scale: f64) {
        for annotation in self.get_page_annotations(page) {
            match &annotation.annotation_type {
                AnnotationType::Highlight { color, rects } => {
                    let (r, g, b, a) = color.to_rgba();
                    context.set_source_rgba(r, g, b, a);

                    for rect in rects {
                        context.rectangle(
                            rect.x * scale,
                            rect.y * scale,
                            rect.width * scale,
                            rect.height * scale,
                        );
                        let _ = context.fill();
                    }
                }
                AnnotationType::Note { position, .. } => {
                    // Draw note icon
                    context.set_source_rgb(1.0, 0.8, 0.0);
                    let (x, y) = (position.0 * scale, position.1 * scale);
                    context.rectangle(x, y, 20.0 * scale, 20.0 * scale);
                    let _ = context.fill();

                    context.set_source_rgb(0.0, 0.0, 0.0);
                    context.set_line_width(1.0);
                    context.rectangle(x, y, 20.0 * scale, 20.0 * scale);
                    let _ = context.stroke();
                }
                AnnotationType::Drawing { color, stroke_width, points } => {
                    let (r, g, b) = color.to_rgb();
                    context.set_source_rgb(r, g, b);
                    context.set_line_width(*stroke_width * scale);
                    context.set_line_cap(cairo::LineCap::Round);
                    context.set_line_join(cairo::LineJoin::Round);

                    if let Some((first_x, first_y)) = points.first() {
                        context.move_to(first_x * scale, first_y * scale);
                        for (x, y) in points.iter().skip(1) {
                            context.line_to(x * scale, y * scale);
                        }
                        let _ = context.stroke();
                    }
                }
                AnnotationType::Underline { rects } => {
                    context.set_source_rgb(0.0, 0.0, 0.8);
                    context.set_line_width(1.5 * scale);

                    for rect in rects {
                        let y = (rect.y + rect.height) * scale;
                        context.move_to(rect.x * scale, y);
                        context.line_to((rect.x + rect.width) * scale, y);
                        let _ = context.stroke();
                    }
                }
                AnnotationType::Strikethrough { rects } => {
                    context.set_source_rgb(0.8, 0.0, 0.0);
                    context.set_line_width(1.5 * scale);

                    for rect in rects {
                        let y = (rect.y + rect.height / 2.0) * scale;
                        context.move_to(rect.x * scale, y);
                        context.line_to((rect.x + rect.width) * scale, y);
                        let _ = context.stroke();
                    }
                }
            }
        }
    }
}

impl Default for Annotations {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Annotations {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.save();
        }
    }
}

// Simple MD5-like hash for filename generation
fn md5_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    for (i, &byte) in data.iter().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul(31u64.wrapping_pow(i as u32)));
    }
    hash
}
