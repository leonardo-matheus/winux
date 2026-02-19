//! File list view for displaying archive contents

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, CellRendererPixbuf, CellRendererText, Image, Label, ListStore, Orientation,
    PolicyType, ScrolledWindow, SelectionMode, TreeView, TreeViewColumn,
};
use crate::archive::ArchiveEntry;

/// Column indices for the list store
mod columns {
    pub const ICON: u32 = 0;
    pub const NAME: u32 = 1;
    pub const SIZE: u32 = 2;
    pub const COMPRESSED: u32 = 3;
    pub const RATIO: u32 = 4;
    pub const DATE: u32 = 5;
    pub const PATH: u32 = 6;
    pub const IS_DIR: u32 = 7;
}

/// File list view component
#[derive(Clone)]
pub struct FileListView {
    container: GtkBox,
    tree_view: TreeView,
    list_store: ListStore,
}

impl FileListView {
    /// Create a new file list view
    pub fn new() -> Self {
        // Create list store with columns:
        // Icon (String), Name (String), Size (String), Compressed (String),
        // Ratio (String), Date (String), Path (String), IsDir (bool)
        let list_store = ListStore::new(&[
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            bool::static_type(),
        ]);

        // Create tree view
        let tree_view = TreeView::with_model(&list_store);
        tree_view.set_headers_visible(true);
        tree_view.set_enable_search(true);
        tree_view.set_search_column(columns::NAME as i32);

        // Icon + Name column
        let name_column = TreeViewColumn::new();
        name_column.set_title("Name");
        name_column.set_expand(true);
        name_column.set_resizable(true);
        name_column.set_sort_column_id(columns::NAME as i32);

        let icon_renderer = CellRendererPixbuf::new();
        let text_renderer = CellRendererText::new();

        name_column.pack_start(&icon_renderer, false);
        name_column.pack_start(&text_renderer, true);
        name_column.add_attribute(&icon_renderer, "icon-name", columns::ICON as i32);
        name_column.add_attribute(&text_renderer, "text", columns::NAME as i32);

        tree_view.append_column(&name_column);

        // Size column
        let size_column = TreeViewColumn::new();
        size_column.set_title("Size");
        size_column.set_resizable(true);
        size_column.set_min_width(80);
        size_column.set_sort_column_id(columns::SIZE as i32);

        let size_renderer = CellRendererText::new();
        size_renderer.set_xalign(1.0);
        size_column.pack_start(&size_renderer, true);
        size_column.add_attribute(&size_renderer, "text", columns::SIZE as i32);

        tree_view.append_column(&size_column);

        // Compressed size column
        let compressed_column = TreeViewColumn::new();
        compressed_column.set_title("Compressed");
        compressed_column.set_resizable(true);
        compressed_column.set_min_width(80);

        let compressed_renderer = CellRendererText::new();
        compressed_renderer.set_xalign(1.0);
        compressed_column.pack_start(&compressed_renderer, true);
        compressed_column.add_attribute(&compressed_renderer, "text", columns::COMPRESSED as i32);

        tree_view.append_column(&compressed_column);

        // Ratio column
        let ratio_column = TreeViewColumn::new();
        ratio_column.set_title("Ratio");
        ratio_column.set_resizable(true);
        ratio_column.set_min_width(60);

        let ratio_renderer = CellRendererText::new();
        ratio_renderer.set_xalign(1.0);
        ratio_column.pack_start(&ratio_renderer, true);
        ratio_column.add_attribute(&ratio_renderer, "text", columns::RATIO as i32);

        tree_view.append_column(&ratio_column);

        // Date column
        let date_column = TreeViewColumn::new();
        date_column.set_title("Modified");
        date_column.set_resizable(true);
        date_column.set_min_width(120);
        date_column.set_sort_column_id(columns::DATE as i32);

        let date_renderer = CellRendererText::new();
        date_column.pack_start(&date_renderer, true);
        date_column.add_attribute(&date_renderer, "text", columns::DATE as i32);

        tree_view.append_column(&date_column);

        // Enable multiple selection
        let selection = tree_view.selection();
        selection.set_mode(SelectionMode::Multiple);

        // Wrap in scrolled window
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .child(&tree_view)
            .build();

        // Container
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.append(&scrolled);

        Self {
            container,
            tree_view,
            list_store,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Set archive entries to display
    pub fn set_entries(&self, entries: &[ArchiveEntry]) {
        self.list_store.clear();

        // Sort entries: directories first, then by name
        let mut sorted_entries: Vec<_> = entries.iter().collect();
        sorted_entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        for entry in sorted_entries {
            let icon = if entry.is_directory {
                "folder-symbolic"
            } else {
                get_file_icon(&entry.name)
            };

            let size = if entry.is_directory {
                String::new()
            } else {
                format_size(entry.uncompressed_size)
            };

            let compressed = if entry.is_directory {
                String::new()
            } else {
                format_size(entry.compressed_size)
            };

            let ratio = if entry.is_directory || entry.uncompressed_size == 0 {
                String::new()
            } else {
                format!("{:.1}%", entry.compression_ratio())
            };

            let date = entry.modified_time
                .map(|t| format_timestamp(t))
                .unwrap_or_default();

            let iter = self.list_store.append();
            self.list_store.set(
                &iter,
                &[
                    (columns::ICON, &icon),
                    (columns::NAME, &entry.name),
                    (columns::SIZE, &size),
                    (columns::COMPRESSED, &compressed),
                    (columns::RATIO, &ratio),
                    (columns::DATE, &date),
                    (columns::PATH, &entry.path),
                    (columns::IS_DIR, &entry.is_directory),
                ],
            );
        }
    }

    /// Get selected entries
    pub fn get_selected(&self) -> Vec<String> {
        let selection = self.tree_view.selection();
        let (paths, model) = selection.selected_rows();

        paths
            .iter()
            .filter_map(|path| {
                model.iter(path).and_then(|iter| {
                    model.get_value(&iter, columns::PATH as i32)
                        .get::<String>()
                        .ok()
                })
            })
            .collect()
    }

    /// Clear the list
    pub fn clear(&self) {
        self.list_store.clear();
    }

    /// Connect to row activated signal
    pub fn connect_row_activated<F: Fn(&str, bool) + 'static>(&self, callback: F) {
        let list_store = self.list_store.clone();

        self.tree_view.connect_row_activated(move |_, path, _| {
            if let Some(iter) = list_store.iter(path) {
                if let (Ok(entry_path), Ok(is_dir)) = (
                    list_store.get_value(&iter, columns::PATH as i32).get::<String>(),
                    list_store.get_value(&iter, columns::IS_DIR as i32).get::<bool>(),
                ) {
                    callback(&entry_path, is_dir);
                }
            }
        });
    }

    /// Show empty state
    pub fn show_empty(&self, message: &str) {
        self.list_store.clear();

        let iter = self.list_store.append();
        self.list_store.set(
            &iter,
            &[
                (columns::ICON, &"dialog-information-symbolic"),
                (columns::NAME, &message),
                (columns::SIZE, &""),
                (columns::COMPRESSED, &""),
                (columns::RATIO, &""),
                (columns::DATE, &""),
                (columns::PATH, &""),
                (columns::IS_DIR, &false),
            ],
        );
    }
}

/// Get icon name for a file
fn get_file_icon(filename: &str) -> &'static str {
    let extension = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        // Archives
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst" => "package-x-generic-symbolic",

        // Documents
        "pdf" => "application-pdf-symbolic",
        "doc" | "docx" | "odt" | "rtf" => "x-office-document-symbolic",
        "xls" | "xlsx" | "ods" | "csv" => "x-office-spreadsheet-symbolic",
        "ppt" | "pptx" | "odp" => "x-office-presentation-symbolic",
        "txt" | "md" | "rst" => "text-x-generic-symbolic",

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" => "image-x-generic-symbolic",

        // Audio
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" => "audio-x-generic-symbolic",

        // Video
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "wmv" | "flv" => "video-x-generic-symbolic",

        // Code
        "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" | "go" | "rb" | "php" | "html" | "css" | "json" | "xml" | "yaml" | "yml" | "toml" => "text-x-script-symbolic",

        // Executables
        "exe" | "msi" | "sh" | "bin" | "appimage" | "deb" | "rpm" => "application-x-executable-symbolic",

        // Disc images
        "iso" | "img" | "dmg" => "media-optical-symbolic",

        // Default
        _ => "text-x-generic-symbolic",
    }
}

/// Format file size
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Format Unix timestamp
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Local, TimeZone};

    match Local.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => String::new(),
    }
}
