//! Document viewer backends

mod pdf;
mod epub;
mod djvu;

pub use pdf::PdfDocument;
pub use epub::EpubDocument;
pub use djvu::DjvuDocument;

use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};

/// Supported document types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocumentType {
    Pdf,
    Epub,
    Djvu,
    Xps,
    Cbz,
    Cbr,
}

impl DocumentType {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "pdf" => Some(DocumentType::Pdf),
            "epub" => Some(DocumentType::Epub),
            "djvu" | "djv" => Some(DocumentType::Djvu),
            "xps" | "oxps" => Some(DocumentType::Xps),
            "cbz" => Some(DocumentType::Cbz),
            "cbr" => Some(DocumentType::Cbr),
            _ => None,
        }
    }
}

/// Trait for document backends
pub trait DocumentBackend: Send + Sync {
    /// Get the number of pages
    fn page_count(&self) -> usize;

    /// Get page dimensions (width, height) in points
    fn page_size(&self, page: usize) -> Option<(f64, f64)>;

    /// Render a page to a Cairo surface at given scale
    fn render_page(&self, page: usize, scale: f64) -> Option<cairo::ImageSurface>;

    /// Get document title
    fn title(&self) -> Option<String>;

    /// Get document author
    fn author(&self) -> Option<String>;

    /// Get document subject/description
    fn subject(&self) -> Option<String>;

    /// Get table of contents
    fn toc(&self) -> Vec<TocEntry>;

    /// Search for text in the document
    fn search(&self, query: &str) -> Vec<SearchResult>;

    /// Get text content of a page
    fn page_text(&self, page: usize) -> Option<String>;

    /// Get text selection rectangles for a page
    fn text_rects(&self, page: usize) -> Vec<TextRect>;
}

/// Table of contents entry
#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub page: usize,
    pub children: Vec<TocEntry>,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub page: usize,
    pub text: String,
    pub rects: Vec<(f64, f64, f64, f64)>, // x, y, width, height
}

/// Text rectangle for selection
#[derive(Debug, Clone)]
pub struct TextRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
}

/// Unified document wrapper
pub struct Document {
    backend: Box<dyn DocumentBackend>,
    path: PathBuf,
    doc_type: DocumentType,
}

impl Document {
    /// Open a document from a file path
    pub fn open(path: &Path) -> Result<Self> {
        let doc_type = DocumentType::from_path(path)
            .ok_or_else(|| anyhow!("Unsupported document format"))?;

        let backend: Box<dyn DocumentBackend> = match doc_type {
            DocumentType::Pdf => Box::new(PdfDocument::open(path)?),
            DocumentType::Epub => Box::new(EpubDocument::open(path)?),
            DocumentType::Djvu => Box::new(DjvuDocument::open(path)?),
            DocumentType::Xps => Box::new(pdf::XpsDocument::open(path)?),
            DocumentType::Cbz => Box::new(pdf::ComicDocument::open_cbz(path)?),
            DocumentType::Cbr => Box::new(pdf::ComicDocument::open_cbr(path)?),
        };

        Ok(Self {
            backend,
            path: path.to_path_buf(),
            doc_type,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn doc_type(&self) -> DocumentType {
        self.doc_type
    }

    pub fn page_count(&self) -> usize {
        self.backend.page_count()
    }

    pub fn page_size(&self, page: usize) -> Option<(f64, f64)> {
        self.backend.page_size(page)
    }

    pub fn render_page(&self, page: usize, scale: f64) -> Option<cairo::ImageSurface> {
        self.backend.render_page(page, scale)
    }

    pub fn title(&self) -> Option<String> {
        self.backend.title()
    }

    pub fn author(&self) -> Option<String> {
        self.backend.author()
    }

    pub fn subject(&self) -> Option<String> {
        self.backend.subject()
    }

    pub fn toc(&self) -> Vec<TocEntry> {
        self.backend.toc()
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        self.backend.search(query)
    }

    pub fn page_text(&self, page: usize) -> Option<String> {
        self.backend.page_text(page)
    }

    pub fn text_rects(&self, page: usize) -> Vec<TextRect> {
        self.backend.text_rects(page)
    }
}
