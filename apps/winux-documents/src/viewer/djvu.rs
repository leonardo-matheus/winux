//! DjVu document backend

use super::{DocumentBackend, TocEntry, SearchResult, TextRect};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::process::Command;

/// DjVu document backend
/// Note: This implementation uses djvulibre command-line tools as a backend
/// For a production implementation, consider using djvulibre bindings
pub struct DjvuDocument {
    path: std::path::PathBuf,
    page_count: usize,
    title: Option<String>,
}

impl DjvuDocument {
    pub fn open(path: &Path) -> Result<Self> {
        // Use djvused to get document info
        let output = Command::new("djvused")
            .arg("-e")
            .arg("n")
            .arg(path)
            .output();

        let page_count = match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.trim().parse().unwrap_or(1)
            }
            Err(_) => {
                // Fallback: assume it's a valid DjVu file with at least one page
                // A real implementation would use libdjvu bindings
                1
            }
        };

        // Try to get title from metadata
        let title = Command::new("djvused")
            .arg("-e")
            .arg("print-meta")
            .arg(path)
            .output()
            .ok()
            .and_then(|out| {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Parse metadata for title
                for line in stdout.lines() {
                    if line.starts_with("title") {
                        let parts: Vec<&str> = line.splitn(2, '\t').collect();
                        if parts.len() == 2 {
                            return Some(parts[1].trim_matches('"').to_string());
                        }
                    }
                }
                None
            });

        Ok(Self {
            path: path.to_path_buf(),
            page_count,
            title,
        })
    }

    fn get_page_size(&self, page: usize) -> Option<(f64, f64)> {
        // Use djvused to get page size
        let output = Command::new("djvused")
            .arg("-e")
            .arg(format!("select {}; size", page + 1))
            .arg(&self.path)
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: width=W height=H
        let mut width = 612.0;
        let mut height = 792.0;

        for part in stdout.split_whitespace() {
            if part.starts_with("width=") {
                width = part.trim_start_matches("width=").parse().unwrap_or(612.0);
            } else if part.starts_with("height=") {
                height = part.trim_start_matches("height=").parse().unwrap_or(792.0);
            }
        }

        Some((width, height))
    }
}

impl DocumentBackend for DjvuDocument {
    fn page_count(&self) -> usize {
        self.page_count
    }

    fn page_size(&self, page: usize) -> Option<(f64, f64)> {
        self.get_page_size(page)
    }

    fn render_page(&self, page: usize, scale: f64) -> Option<cairo::ImageSurface> {
        let (width, height) = self.get_page_size(page)?;

        let scaled_width = (width * scale) as i32;
        let scaled_height = (height * scale) as i32;

        // Use ddjvu to render to PBM/PGM/PPM
        let temp_file = std::env::temp_dir().join(format!("djvu_page_{}.ppm", page));

        let result = Command::new("ddjvu")
            .arg("-format=ppm")
            .arg(format!("-page={}", page + 1))
            .arg(format!("-scale={}", (scale * 100.0) as i32))
            .arg(&self.path)
            .arg(&temp_file)
            .status();

        if result.is_err() || !temp_file.exists() {
            // Fallback: create placeholder surface
            return create_placeholder_surface(scaled_width, scaled_height, page);
        }

        // Load PPM file
        let img = image::open(&temp_file).ok()?;
        let _ = std::fs::remove_file(&temp_file);

        let rgba = img.to_rgba8();

        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            rgba.width() as i32,
            rgba.height() as i32,
        ).ok()?;

        {
            let mut data = surface.data().ok()?;
            for (i, pixel) in rgba.pixels().enumerate() {
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
        // Use djvused to get outline
        let output = Command::new("djvused")
            .arg("-e")
            .arg("print-outline")
            .arg(&self.path)
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_djvu_outline(&stdout)
        } else {
            Vec::new()
        }
    }

    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();

        // Use djvutxt to extract text and search
        for page in 0..self.page_count {
            if let Some(text) = self.page_text(page) {
                if text.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(SearchResult {
                        page,
                        text: extract_context(&text, query),
                        rects: Vec::new(),
                    });
                }
            }
        }

        results
    }

    fn page_text(&self, page: usize) -> Option<String> {
        let output = Command::new("djvutxt")
            .arg(format!("--page={}", page + 1))
            .arg(&self.path)
            .output()
            .ok()?;

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn text_rects(&self, _page: usize) -> Vec<TextRect> {
        Vec::new()
    }
}

fn create_placeholder_surface(width: i32, height: i32, page: usize) -> Option<cairo::ImageSurface> {
    let surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        width,
        height,
    ).ok()?;

    let context = cairo::Context::new(&surface).ok()?;

    // Light gray background
    context.set_source_rgb(0.95, 0.95, 0.95);
    context.paint().ok()?;

    // Border
    context.set_source_rgb(0.8, 0.8, 0.8);
    context.set_line_width(2.0);
    context.rectangle(1.0, 1.0, width as f64 - 2.0, height as f64 - 2.0);
    context.stroke().ok()?;

    // Page number
    context.set_source_rgb(0.5, 0.5, 0.5);
    context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    context.set_font_size(24.0);
    let text = format!("DjVu Page {}", page + 1);
    let extents = context.text_extents(&text).ok()?;
    context.move_to(
        (width as f64 - extents.width()) / 2.0,
        (height as f64 + extents.height()) / 2.0,
    );
    context.show_text(&text).ok()?;

    Some(surface)
}

fn parse_djvu_outline(outline: &str) -> Vec<TocEntry> {
    // Simplified outline parser
    // Format: (bookmarks ("title" "#page" ...) ...)
    let mut entries = Vec::new();

    // Simple regex-free parsing
    let mut depth = 0;
    let mut current_title = String::new();
    let mut current_page = 0;
    let mut in_string = false;
    let mut string_content = String::new();

    for c in outline.chars() {
        match c {
            '(' => {
                depth += 1;
            }
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '"' => {
                if in_string {
                    // End of string
                    if current_title.is_empty() {
                        current_title = string_content.clone();
                    } else if string_content.starts_with('#') {
                        if let Ok(page) = string_content[1..].parse::<usize>() {
                            current_page = page.saturating_sub(1);
                            entries.push(TocEntry {
                                title: current_title.clone(),
                                page: current_page,
                                children: Vec::new(),
                            });
                            current_title.clear();
                        }
                    }
                    string_content.clear();
                }
                in_string = !in_string;
            }
            _ if in_string => {
                string_content.push(c);
            }
            _ => {}
        }
    }

    entries
}

fn extract_context(text: &str, query: &str) -> String {
    let lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(pos) = lower.find(&query_lower) {
        let start = pos.saturating_sub(50);
        let end = (pos + query.len() + 50).min(text.len());

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
