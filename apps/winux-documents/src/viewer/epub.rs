//! EPUB document backend

use super::{DocumentBackend, TocEntry, SearchResult, TextRect};
use anyhow::{Result, anyhow};
use std::path::Path;

/// EPUB document backend
pub struct EpubDocument {
    chapters: Vec<EpubChapter>,
    title: Option<String>,
    author: Option<String>,
    toc: Vec<TocEntry>,
}

struct EpubChapter {
    title: String,
    content: String,
    width: f64,
    height: f64,
}

impl EpubDocument {
    pub fn open(path: &Path) -> Result<Self> {
        let doc = epub::doc::EpubDoc::new(path)
            .map_err(|e| anyhow!("Failed to open EPUB: {}", e))?;

        let title = doc.mdata("title").map(|s| s.to_string());
        let author = doc.mdata("creator").map(|s| s.to_string());

        // Extract chapters
        let mut chapters = Vec::new();
        let mut epub = epub::doc::EpubDoc::new(path)?;

        // Get spine items (ordered content)
        let spine_ids: Vec<String> = epub.spine.clone();

        for _id in &spine_ids {
            if let Some((content, _mime)) = epub.get_current_str() {
                // Strip HTML tags for plain text (simplified)
                let plain_text = strip_html_tags(&content);

                chapters.push(EpubChapter {
                    title: String::new(),
                    content: plain_text,
                    width: 612.0,  // Standard page width
                    height: 792.0, // Standard page height
                });
            }
            epub.go_next();
        }

        if chapters.is_empty() {
            // Fallback: iterate through all content
            let mut epub = epub::doc::EpubDoc::new(path)?;
            while epub.go_next() {
                if let Some((content, _mime)) = epub.get_current_str() {
                    let plain_text = strip_html_tags(&content);
                    if !plain_text.trim().is_empty() {
                        chapters.push(EpubChapter {
                            title: String::new(),
                            content: plain_text,
                            width: 612.0,
                            height: 792.0,
                        });
                    }
                }
            }
        }

        // Build TOC from navigation
        let toc = build_epub_toc(&mut epub::doc::EpubDoc::new(path)?);

        Ok(Self {
            chapters,
            title,
            author,
            toc,
        })
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_style = false;
    let mut in_script = false;

    let html_lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = html_lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        // Check for style/script tags
        if i + 6 < chars.len() {
            let next_six: String = lower_chars[i..i+6].iter().collect();
            if next_six == "<style" {
                in_style = true;
            } else if next_six == "</styl" {
                in_style = false;
            }
        }
        if i + 7 < chars.len() {
            let next_seven: String = lower_chars[i..i+7].iter().collect();
            if next_seven == "<script" {
                in_script = true;
            } else if next_seven == "</scrip" {
                in_script = false;
            }
        }

        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag && !in_style && !in_script {
            result.push(c);
        }

        i += 1;
    }

    // Decode common HTML entities
    result = result
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");

    // Normalize whitespace
    let mut normalized = String::new();
    let mut last_was_space = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                normalized.push(' ');
                last_was_space = true;
            }
        } else {
            normalized.push(c);
            last_was_space = false;
        }
    }

    normalized.trim().to_string()
}

fn build_epub_toc(epub: &mut epub::doc::EpubDoc<std::io::BufReader<std::fs::File>>) -> Vec<TocEntry> {
    let mut entries = Vec::new();

    // Try to get TOC from navigation document
    if let Some(toc) = epub.toc.clone() {
        for (i, nav_point) in toc.iter().enumerate() {
            entries.push(TocEntry {
                title: nav_point.label.clone(),
                page: i, // Map to chapter index
                children: Vec::new(),
            });
        }
    }

    entries
}

impl DocumentBackend for EpubDocument {
    fn page_count(&self) -> usize {
        self.chapters.len().max(1)
    }

    fn page_size(&self, page: usize) -> Option<(f64, f64)> {
        self.chapters.get(page).map(|c| (c.width, c.height))
    }

    fn render_page(&self, page: usize, scale: f64) -> Option<cairo::ImageSurface> {
        let chapter = self.chapters.get(page)?;

        let scaled_width = (chapter.width * scale) as i32;
        let scaled_height = (chapter.height * scale) as i32;

        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            scaled_width,
            scaled_height,
        ).ok()?;

        let context = cairo::Context::new(&surface).ok()?;

        // White background
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint().ok()?;

        // Render text
        context.set_source_rgb(0.1, 0.1, 0.1);
        context.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        context.set_font_size(14.0 * scale);

        let margin = 50.0 * scale;
        let line_height = 20.0 * scale;
        let max_width = scaled_width as f64 - 2.0 * margin;

        // Simple text wrapping
        let words: Vec<&str> = chapter.content.split_whitespace().collect();
        let mut current_line = String::new();
        let mut y = margin + line_height;

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let extents = context.text_extents(&test_line).ok()?;

            if extents.width() > max_width && !current_line.is_empty() {
                // Draw current line and start new one
                context.move_to(margin, y);
                context.show_text(&current_line).ok()?;
                y += line_height;

                current_line = word.to_string();

                // Stop if we've filled the page
                if y > scaled_height as f64 - margin {
                    break;
                }
            } else {
                current_line = test_line;
            }
        }

        // Draw remaining text
        if !current_line.is_empty() && y <= scaled_height as f64 - margin {
            context.move_to(margin, y);
            context.show_text(&current_line).ok()?;
        }

        Some(surface)
    }

    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn author(&self) -> Option<String> {
        self.author.clone()
    }

    fn subject(&self) -> Option<String> {
        None
    }

    fn toc(&self) -> Vec<TocEntry> {
        self.toc.clone()
    }

    fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for (page, chapter) in self.chapters.iter().enumerate() {
            if chapter.content.to_lowercase().contains(&query_lower) {
                results.push(SearchResult {
                    page,
                    text: extract_context(&chapter.content, &query_lower),
                    rects: Vec::new(), // Would need layout info for precise rects
                });
            }
        }

        results
    }

    fn page_text(&self, page: usize) -> Option<String> {
        self.chapters.get(page).map(|c| c.content.clone())
    }

    fn text_rects(&self, _page: usize) -> Vec<TextRect> {
        Vec::new()
    }
}

fn extract_context(text: &str, query: &str) -> String {
    let lower = text.to_lowercase();
    if let Some(pos) = lower.find(query) {
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
