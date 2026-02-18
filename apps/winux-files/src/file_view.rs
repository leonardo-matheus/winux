//! File view component - Grid and List views for files

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use gtk4::prelude::*;
use libadwaita as adw;
use relm4::prelude::*;
use tracing::{debug, error};

use crate::app::SortBy;

/// View mode for file display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

/// File entry information
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub icon_name: String,
    pub mime_type: String,
}

impl FileEntry {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let metadata = fs::metadata(&path).ok()?;
        let name = path.file_name()?.to_string_lossy().to_string();
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        let modified = metadata.modified().ok();

        let (icon_name, mime_type) = if is_dir {
            ("folder-symbolic".to_string(), "inode/directory".to_string())
        } else {
            let mime = mime_guess::from_path(&path)
                .first()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            let icon = Self::icon_for_mime(&mime);
            (icon, mime)
        };

        Some(FileEntry {
            path,
            name,
            is_dir,
            size,
            modified,
            icon_name,
            mime_type,
        })
    }

    fn icon_for_mime(mime: &str) -> String {
        match mime.split('/').next().unwrap_or("") {
            "text" => "text-x-generic-symbolic",
            "image" => "image-x-generic-symbolic",
            "audio" => "audio-x-generic-symbolic",
            "video" => "video-x-generic-symbolic",
            "application" => {
                if mime.contains("pdf") {
                    "x-office-document-symbolic"
                } else if mime.contains("zip") || mime.contains("tar") || mime.contains("archive")
                {
                    "package-x-generic-symbolic"
                } else if mime.contains("executable") {
                    "application-x-executable-symbolic"
                } else {
                    "application-x-generic-symbolic"
                }
            }
            _ => "text-x-generic-symbolic",
        }
        .to_string()
    }

    pub fn formatted_size(&self) -> String {
        if self.is_dir {
            String::new()
        } else {
            format_size(self.size)
        }
    }

    pub fn formatted_date(&self) -> String {
        self.modified
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_default()
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// File view model
pub struct FileView {
    /// Current directory
    current_dir: PathBuf,
    /// Files in current directory
    files: Vec<FileEntry>,
    /// Filtered files (after search)
    filtered_files: Vec<FileEntry>,
    /// View mode
    view_mode: ViewMode,
    /// Show hidden files
    show_hidden: bool,
    /// Sort order
    sort_by: SortBy,
    /// Selected file indices
    selected_indices: Vec<usize>,
    /// Search query
    search_query: Option<String>,
    /// Loading state
    is_loading: bool,
}

/// File view input messages
#[derive(Debug)]
pub enum FileViewInput {
    LoadDirectory(PathBuf),
    SetViewMode(ViewMode),
    SetShowHidden(bool),
    SetSortBy(SortBy),
    Search(String),
    ClearSearch,
    SelectItem(usize),
    SelectRange(usize, usize),
    ToggleSelect(usize),
    SelectAll,
    ClearSelection,
    ActivateItem(usize),
}

/// File view output messages
#[derive(Debug)]
pub enum FileViewOutput {
    SelectionChanged(Vec<PathBuf>),
    OpenFile(PathBuf),
}

#[relm4::component(pub)]
impl Component for FileView {
    type Init = PathBuf;
    type Input = FileViewInput;
    type Output = FileViewOutput;
    type CommandOutput = Vec<FileEntry>;

    view! {
        gtk4::ScrolledWindow {
            set_hscrollbar_policy: gtk4::PolicyType::Never,
            set_vscrollbar_policy: gtk4::PolicyType::Automatic,

            #[name = "content"]
            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,

                // Loading spinner
                #[name = "spinner"]
                gtk4::Spinner {
                    #[watch]
                    set_visible: model.is_loading,
                    #[watch]
                    set_spinning: model.is_loading,
                },

                // Grid view
                #[name = "grid_view"]
                gtk4::FlowBox {
                    #[watch]
                    set_visible: model.view_mode == ViewMode::Grid && !model.is_loading,
                    set_homogeneous: true,
                    set_min_children_per_line: 4,
                    set_max_children_per_line: 12,
                    set_selection_mode: gtk4::SelectionMode::Multiple,
                    set_activate_on_single_click: false,

                    connect_child_activated[sender] => move |_, child| {
                        let idx = child.index() as usize;
                        sender.input(FileViewInput::ActivateItem(idx));
                    },

                    connect_selected_children_changed[sender] => move |flow_box| {
                        let indices: Vec<usize> = flow_box
                            .selected_children()
                            .iter()
                            .map(|child| child.index() as usize)
                            .collect();
                        if let Some(&idx) = indices.last() {
                            sender.input(FileViewInput::SelectItem(idx));
                        }
                    },
                },

                // List view
                #[name = "list_view"]
                gtk4::ListView {
                    #[watch]
                    set_visible: model.view_mode == ViewMode::List && !model.is_loading,
                    set_single_click_activate: false,
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FileView {
            current_dir: init.clone(),
            files: Vec::new(),
            filtered_files: Vec::new(),
            view_mode: ViewMode::Grid,
            show_hidden: false,
            sort_by: SortBy::Name,
            selected_indices: Vec::new(),
            search_query: None,
            is_loading: false,
        };

        let widgets = view_output!();

        // Load initial directory
        sender.input(FileViewInput::LoadDirectory(init));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            FileViewInput::LoadDirectory(path) => {
                debug!("Loading directory: {:?}", path);
                self.is_loading = true;
                self.current_dir = path.clone();
                self.selected_indices.clear();

                let show_hidden = self.show_hidden;
                let sort_by = self.sort_by;

                sender.oneshot_command(async move {
                    load_directory(&path, show_hidden, sort_by)
                });
            }
            FileViewInput::SetViewMode(mode) => {
                self.view_mode = mode;
            }
            FileViewInput::SetShowHidden(show) => {
                self.show_hidden = show;
                sender.input(FileViewInput::LoadDirectory(self.current_dir.clone()));
            }
            FileViewInput::SetSortBy(sort) => {
                self.sort_by = sort;
                self.sort_files();
                self.apply_filter();
            }
            FileViewInput::Search(query) => {
                self.search_query = Some(query);
                self.apply_filter();
            }
            FileViewInput::ClearSearch => {
                self.search_query = None;
                self.apply_filter();
            }
            FileViewInput::SelectItem(idx) => {
                self.selected_indices = vec![idx];
                self.emit_selection(&sender);
            }
            FileViewInput::SelectRange(start, end) => {
                self.selected_indices = (start.min(end)..=start.max(end)).collect();
                self.emit_selection(&sender);
            }
            FileViewInput::ToggleSelect(idx) => {
                if let Some(pos) = self.selected_indices.iter().position(|&i| i == idx) {
                    self.selected_indices.remove(pos);
                } else {
                    self.selected_indices.push(idx);
                }
                self.emit_selection(&sender);
            }
            FileViewInput::SelectAll => {
                self.selected_indices = (0..self.filtered_files.len()).collect();
                self.emit_selection(&sender);
            }
            FileViewInput::ClearSelection => {
                self.selected_indices.clear();
                self.emit_selection(&sender);
            }
            FileViewInput::ActivateItem(idx) => {
                if let Some(file) = self.filtered_files.get(idx) {
                    let _ = sender.output(FileViewOutput::OpenFile(file.path.clone()));
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        files: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.files = files;
        self.is_loading = false;
        self.apply_filter();
        self.update_grid_view(root);
    }
}

impl FileView {
    fn sort_files(&mut self) {
        self.files.sort_by(|a, b| {
            // Directories first
            match (a.is_dir, b.is_dir) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }

            match self.sort_by {
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Size => b.size.cmp(&a.size),
                SortBy::Modified => b.modified.cmp(&a.modified),
                SortBy::Type => a.mime_type.cmp(&b.mime_type),
            }
        });
    }

    fn apply_filter(&mut self) {
        if let Some(query) = &self.search_query {
            let query_lower = query.to_lowercase();
            self.filtered_files = self
                .files
                .iter()
                .filter(|f| f.name.to_lowercase().contains(&query_lower))
                .cloned()
                .collect();
        } else {
            self.filtered_files = self.files.clone();
        }
    }

    fn emit_selection(&self, sender: &ComponentSender<Self>) {
        let paths: Vec<PathBuf> = self
            .selected_indices
            .iter()
            .filter_map(|&i| self.filtered_files.get(i))
            .map(|f| f.path.clone())
            .collect();
        let _ = sender.output(FileViewOutput::SelectionChanged(paths));
    }

    fn update_grid_view(&self, root: &gtk4::ScrolledWindow) {
        // Find the grid view and update it
        if let Some(content) = root.child() {
            if let Some(box_widget) = content.downcast_ref::<gtk4::Box>() {
                // Find the FlowBox
                let mut child = box_widget.first_child();
                while let Some(widget) = child {
                    if let Some(flow_box) = widget.downcast_ref::<gtk4::FlowBox>() {
                        // Clear existing children
                        while let Some(child) = flow_box.first_child() {
                            flow_box.remove(&child);
                        }

                        // Add new items
                        for file in &self.filtered_files {
                            let item = create_grid_item(file);
                            flow_box.append(&item);
                        }
                        break;
                    }
                    child = widget.next_sibling();
                }
            }
        }
    }
}

fn load_directory(path: &PathBuf, show_hidden: bool, sort_by: SortBy) -> Vec<FileEntry> {
    let mut files: Vec<FileEntry> = match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_string_lossy().to_string();

                // Filter hidden files
                if !show_hidden && name.starts_with('.') {
                    return None;
                }

                FileEntry::from_path(path)
            })
            .collect(),
        Err(e) => {
            error!("Failed to read directory: {}", e);
            Vec::new()
        }
    };

    // Sort files
    files.sort_by(|a, b| {
        // Directories first
        match (a.is_dir, b.is_dir) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        match sort_by {
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Size => b.size.cmp(&a.size),
            SortBy::Modified => b.modified.cmp(&a.modified),
            SortBy::Type => a.mime_type.cmp(&b.mime_type),
        }
    });

    files
}

fn create_grid_item(file: &FileEntry) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    container.set_margin_all(8);
    container.set_width_request(96);

    // Icon
    let icon = gtk4::Image::from_icon_name(&file.icon_name);
    icon.set_pixel_size(48);
    container.append(&icon);

    // Name label
    let label = gtk4::Label::new(Some(&file.name));
    label.set_max_width_chars(12);
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_lines(2);
    label.set_wrap(true);
    label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    container.append(&label);

    container
}
