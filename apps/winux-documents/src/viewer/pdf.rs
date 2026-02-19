//! PDF rendering backend using poppler-glib

use super::{DocumentBackend, TocEntry, SearchResult, TextRect};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

/// PDF document backend using poppler
pub struct PdfDocument {
    document: poppler::Document,
}

impl PdfDocument {
    pub fn open(path: &Path) -> Result<Self> {
        let uri = format!("file://{}", path.to_string_lossy());
        let document = poppler::Document::from_file(&uri, None)
            .map_err(|e| anyhow!("Failed to open PDF: {}", e))?;

        Ok(Self { document })
    }
}

impl DocumentBackend for PdfDocument {
    fn page_count(&self) -> usize {
        self.document.n_pages() as usize
    }

    fn page_size(&self, page: usize) -> Option<(f64, f64)> {
        let page = self.document.page(page as i32)?;
        Some(page.size())
    }

    fn render_page(&self, page_num: usize, scale: f64) -> Option<cairo::ImageSurface> {
        let page = self.document.page(page_num as i32)?;
        let (width, height) = page.size();

        let scaled_width = (width * scale) as i32;
        let scaled_height = (height * scale) as i32;

        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            scaled_width,
            scaled_height,
        ).ok()?;

        let context = cairo::Context::new(&surface).ok()?;

        // White background
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint().ok()?;

        // Scale for rendering
        context.scale(scale, scale);

        // Render the page
        page.render(&context);

        Some(surface)
    }

    fn title(&self) -> Option<String> {
        self.document.title().map(|s| s.to_string())
    }

    fn author(&self) -> Option<String> {
        self.document.author().map(|s| s.to_string())
    }

    fn subject(&self) -> Option<String> {
        self.document.subject().map(|s| s.to_string())
    }

    fn toc(&self) -> Vec<TocEntry> {
        let mut entries = Vec::new();

        if let Some(index) = self.document.index_iter() {
            parse_toc_iter(&self.document, &index, &mut entries);
        }

        entries
    }

    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for page_num in 0..self.page_count() {
            if let Some(page) = self.document.page(page_num as i32) {
                if let Some(text) = page.text() {
                    if text.to_lowercase().contains(&query_lower) {
                        // Find matching rectangles
                        let rects: Vec<(f64, f64, f64, f64)> = page
                            .find_text(query)
                            .into_iter()
                            .map(|rect| (rect.x1(), rect.y1(), rect.x2() - rect.x1(), rect.y2() - rect.y1()))
                            .collect();

                        results.push(SearchResult {
                            page: page_num,
                            text: extract_context(&text, &query_lower),
                            rects,
                        });
                    }
                }
            }
        }

        results
    }

    fn page_text(&self, page_num: usize) -> Option<String> {
        let page = self.document.page(page_num as i32)?;
        page.text().map(|s| s.to_string())
    }

    fn text_rects(&self, page_num: usize) -> Vec<TextRect> {
        // Poppler doesn't provide easy access to individual character/word rects
        // This is a simplified implementation
        Vec::new()
    }
}

fn parse_toc_iter(
    document: &poppler::Document,
    iter: &poppler::IndexIter,
    entries: &mut Vec<TocEntry>,
) {
    loop {
        if let Some(action) = iter.action() {
            if let Some(title) = action.title() {
                let page = match action.action_type() {
                    poppler::ActionType::GotoDest => {
                        if let Some(dest) = action.goto_dest_dest() {
                            dest.page_num() as usize
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };

                let mut entry = TocEntry {
                    title: title.to_string(),
                    page: page.saturating_sub(1), // Convert to 0-indexed
                    children: Vec::new(),
                };

                // Recurse into children
                if let Some(child_iter) = iter.child() {
                    parse_toc_iter(document, &child_iter, &mut entry.children);
                }

                entries.push(entry);
            }
        }

        if !iter.next() {
            break;
        }
    }
}

fn extract_context(text: &str, query: &str) -> String {
    let lower = text.to_lowercase();
    if let Some(pos) = lower.find(query) {
        let start = pos.saturating_sub(30);
        let end = (pos + query.len() + 30).min(text.len());

        let mut context = String::new();
        if start > 0 {
            context.push_str("...");
        }
        context.push_str(&text[start..end]);
        if end < text.len() {
            context.push_str("...");
        }
        context
    } else {
        String::new()
    }
}

/// XPS document backend (simplified implementation)
pub struct XpsDocument {
    pages: Vec<XpsPage>,
    title: Option<String>,
}

struct XpsPage {
    width: f64,
    height: f64,
    content: Vec<u8>,
}

impl XpsDocument {
    pub fn open(path: &Path) -> Result<Self> {
        // XPS files are ZIP archives containing XAML pages
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut pages = Vec::new();
        let mut title = None;

        // Parse fixed document sequence
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.ends_with(".fpage") {
                // Simplified: treat as 8.5x11 pages
                pages.push(XpsPage {
                    width: 612.0, // 8.5 inches at 72 DPI
                    height: 792.0, // 11 inches at 72 DPI
                    content: Vec::new(),
                });
            }

            if name.contains("CoreProperties") {
                // Would parse title from metadata here
            }
        }

        if pages.is_empty() {
            return Err(anyhow!("No pages found in XPS document"));
        }

        Ok(Self { pages, title })
    }
}

impl DocumentBackend for XpsDocument {
    fn page_count(&self) -> usize {
        self.pages.len()
    }

    fn page_size(&self, page: usize) -> Option<(f64, f64)> {
        self.pages.get(page).map(|p| (p.width, p.height))
    }

    fn render_page(&self, page: usize, scale: f64) -> Option<cairo::ImageSurface> {
        let page_data = self.pages.get(page)?;

        let scaled_width = (page_data.width * scale) as i32;
        let scaled_height = (page_data.height * scale) as i32;

        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            scaled_width,
            scaled_height,
        ).ok()?;

        let context = cairo::Context::new(&surface).ok()?;

        // White background
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint().ok()?;

        // Placeholder text for XPS (full implementation would render XAML)
        context.set_source_rgb(0.5, 0.5, 0.5);
        context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        context.set_font_size(24.0 * scale);
        context.move_to(50.0 * scale, 100.0 * scale);
        context.show_text("XPS Page").ok()?;

        Some(surface)
    }

    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn author(&self) -> Option<String> {
        None
    }

    fn subject(&self) -> Option<String> {
        None
    }

    fn toc(&self) -> Vec<TocEntry> {
        Vec::new()
    }

    fn search(&self, _query: &str) -> Vec<SearchResult> {
        Vec::new()
    }

    fn page_text(&self, _page: usize) -> Option<String> {
        None
    }

    fn text_rects(&self, _page: usize) -> Vec<TextRect> {
        Vec::new()
    }
}

/// Comic book document backend (CBZ/CBR)
pub struct ComicDocument {
    images: Vec<ComicPage>,
    title: Option<String>,
}

struct ComicPage {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl ComicDocument {
    pub fn open_cbz(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut image_files: Vec<(String, Vec<u8>)> = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_lowercase();

            if name.ends_with(".jpg") || name.ends_with(".jpeg") ||
               name.ends_with(".png") || name.ends_with(".gif") ||
               name.ends_with(".webp") {
                let mut data = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut data)?;
                image_files.push((file.name().to_string(), data));
            }
        }

        // Sort by filename
        image_files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut images = Vec::new();
        for (_, data) in image_files {
            if let Ok(img) = image::load_from_memory(&data) {
                images.push(ComicPage {
                    data,
                    width: img.width(),
                    height: img.height(),
                });
            }
        }

        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        Ok(Self { images, title })
    }

    pub fn open_cbr(path: &Path) -> Result<Self> {
        // CBR is RAR format
        // Note: unrar crate requires the unrar library to be installed
        let archive = unrar::Archive::new(path)
            .open_for_processing()
            .map_err(|e| anyhow!("Failed to open RAR archive: {:?}", e))?;

        let mut image_files: Vec<(String, Vec<u8>)> = Vec::new();

        for entry in archive {
            let entry = entry.map_err(|e| anyhow!("Failed to read RAR entry: {:?}", e))?;
            let name = entry.filename.to_lowercase();

            if name.ends_with(".jpg") || name.ends_with(".jpeg") ||
               name.ends_with(".png") || name.ends_with(".gif") ||
               name.ends_with(".webp") {
                let data = entry.read()
                    .map_err(|e| anyhow!("Failed to extract RAR entry: {:?}", e))?;
                image_files.push((entry.filename.clone(), data.1));
            }
        }

        // Sort by filename
        image_files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut images = Vec::new();
        for (_, data) in image_files {
            if let Ok(img) = image::load_from_memory(&data) {
                images.push(ComicPage {
                    data,
                    width: img.width(),
                    height: img.height(),
                });
            }
        }

        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        Ok(Self { images, title })
    }
}

impl DocumentBackend for ComicDocument {
    fn page_count(&self) -> usize {
        self.images.len()
    }

    fn page_size(&self, page: usize) -> Option<(f64, f64)> {
        self.images.get(page).map(|p| (p.width as f64, p.height as f64))
    }

    fn render_page(&self, page: usize, scale: f64) -> Option<cairo::ImageSurface> {
        let comic_page = self.images.get(page)?;

        let img = image::load_from_memory(&comic_page.data).ok()?;
        let img = img.to_rgba8();

        let scaled_width = (comic_page.width as f64 * scale) as u32;
        let scaled_height = (comic_page.height as f64 * scale) as u32;

        // Resize image if needed
        let resized = if scale != 1.0 {
            image::imageops::resize(
                &img,
                scaled_width,
                scaled_height,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            img
        };

        // Convert to Cairo surface
        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            scaled_width as i32,
            scaled_height as i32,
        ).ok()?;

        {
            let mut data = surface.data().ok()?;
            for (i, pixel) in resized.pixels().enumerate() {
                let offset = i * 4;
                // Cairo uses BGRA format
                data[offset] = pixel[2];     // B
                data[offset + 1] = pixel[1]; // G
                data[offset + 2] = pixel[0]; // R
                data[offset + 3] = pixel[3]; // A
            }
        }

        surface.mark_dirty();
        Some(surface)
    }

    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn author(&self) -> Option<String> {
        None
    }

    fn subject(&self) -> Option<String> {
        None
    }

    fn toc(&self) -> Vec<TocEntry> {
        Vec::new()
    }

    fn search(&self, _query: &str) -> Vec<SearchResult> {
        Vec::new()
    }

    fn page_text(&self, _page: usize) -> Option<String> {
        None
    }

    fn text_rects(&self, _page: usize) -> Vec<TextRect> {
        Vec::new()
    }
}
