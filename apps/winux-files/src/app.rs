//! Application model and state management for Winux Files

use std::path::PathBuf;

use gtk4::prelude::*;
use libadwaita as adw;
use relm4::prelude::*;
use tracing::{debug, info};

use crate::config::Config;
use crate::file_ops::{FileOperation, FileOperationMsg};
use crate::file_view::{FileView, FileViewInput, FileViewOutput, ViewMode};
use crate::sidebar::{Sidebar, SidebarInput, SidebarOutput};

/// Main application model
pub struct AppModel {
    /// Current directory path
    current_path: PathBuf,
    /// Application configuration
    config: Config,
    /// View mode (grid or list)
    view_mode: ViewMode,
    /// Show hidden files
    show_hidden: bool,
    /// Sort order
    sort_by: SortBy,
    /// Search query
    search_query: Option<String>,
    /// Selected files
    selected_files: Vec<PathBuf>,
    /// Clipboard contents
    clipboard: Option<ClipboardContents>,
    /// File view component
    file_view: Controller<FileView>,
    /// Sidebar component
    sidebar: Controller<Sidebar>,
    /// File operation handler
    file_ops: Controller<FileOperation>,
}

/// Sort options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    #[default]
    Name,
    Size,
    Modified,
    Type,
}

/// Clipboard contents for copy/cut operations
#[derive(Debug, Clone)]
pub struct ClipboardContents {
    pub files: Vec<PathBuf>,
    pub is_cut: bool,
}

/// Application input messages
#[derive(Debug)]
pub enum AppInput {
    /// Navigate to a path
    NavigateTo(PathBuf),
    /// Go back in history
    GoBack,
    /// Go forward in history
    GoForward,
    /// Go to parent directory
    GoUp,
    /// Go to home directory
    GoHome,
    /// Refresh current view
    Refresh,
    /// Toggle view mode
    ToggleViewMode,
    /// Set view mode
    SetViewMode(ViewMode),
    /// Toggle hidden files
    ToggleHidden,
    /// Set sort order
    SetSortBy(SortBy),
    /// Search files
    Search(String),
    /// Clear search
    ClearSearch,
    /// Select file(s)
    Select(Vec<PathBuf>),
    /// Open selected file(s)
    OpenSelected,
    /// Copy selected files
    Copy,
    /// Cut selected files
    Cut,
    /// Paste files
    Paste,
    /// Delete selected files
    Delete,
    /// Rename selected file
    Rename(String),
    /// Create new folder
    NewFolder(String),
    /// Create new file
    NewFile(String),
    /// Show properties
    ShowProperties,
    /// Open terminal here
    OpenTerminal,
    /// Sidebar message
    SidebarMsg(SidebarOutput),
    /// File view message
    FileViewMsg(FileViewOutput),
    /// File operation message
    FileOpsMsg(FileOperationMsg),
}

/// Application output messages
#[derive(Debug)]
pub enum AppOutput {}

#[relm4::component(pub)]
impl Component for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = AppOutput;
    type CommandOutput = ();

    view! {
        adw::ApplicationWindow {
            set_default_width: 1200,
            set_default_height: 800,
            set_title: Some("Winux Files"),

            adw::ToolbarView {
                // Header bar
                add_top_bar = &adw::HeaderBar {
                    // Navigation buttons
                    pack_start = &gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 4,

                        gtk4::Button {
                            set_icon_name: "go-previous-symbolic",
                            set_tooltip_text: Some("Go back"),
                            connect_clicked => AppInput::GoBack,
                        },

                        gtk4::Button {
                            set_icon_name: "go-next-symbolic",
                            set_tooltip_text: Some("Go forward"),
                            connect_clicked => AppInput::GoForward,
                        },

                        gtk4::Button {
                            set_icon_name: "go-up-symbolic",
                            set_tooltip_text: Some("Go to parent"),
                            connect_clicked => AppInput::GoUp,
                        },

                        gtk4::Button {
                            set_icon_name: "go-home-symbolic",
                            set_tooltip_text: Some("Go home"),
                            connect_clicked => AppInput::GoHome,
                        },
                    },

                    // Path bar in title
                    #[wrap(Some)]
                    set_title_widget = &gtk4::Entry {
                        set_hexpand: true,
                        set_width_request: 400,
                        #[watch]
                        set_text: model.current_path.to_str().unwrap_or(""),
                        set_placeholder_text: Some("Enter path..."),
                        connect_activate[sender] => move |entry| {
                            let text = entry.text();
                            let path = PathBuf::from(text.as_str());
                            sender.input(AppInput::NavigateTo(path));
                        },
                    },

                    // Action buttons
                    pack_end = &gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 4,

                        gtk4::ToggleButton {
                            set_icon_name: "view-grid-symbolic",
                            set_tooltip_text: Some("Grid view"),
                            #[watch]
                            set_active: model.view_mode == ViewMode::Grid,
                            connect_toggled[sender] => move |btn| {
                                if btn.is_active() {
                                    sender.input(AppInput::SetViewMode(ViewMode::Grid));
                                }
                            },
                        },

                        gtk4::ToggleButton {
                            set_icon_name: "view-list-symbolic",
                            set_tooltip_text: Some("List view"),
                            #[watch]
                            set_active: model.view_mode == ViewMode::List,
                            connect_toggled[sender] => move |btn| {
                                if btn.is_active() {
                                    sender.input(AppInput::SetViewMode(ViewMode::List));
                                }
                            },
                        },

                        gtk4::Separator {},

                        gtk4::ToggleButton {
                            set_icon_name: "view-reveal-symbolic",
                            set_tooltip_text: Some("Show hidden files"),
                            #[watch]
                            set_active: model.show_hidden,
                            connect_toggled => AppInput::ToggleHidden,
                        },

                        gtk4::MenuButton {
                            set_icon_name: "open-menu-symbolic",
                            set_tooltip_text: Some("Menu"),
                        },
                    },
                },

                // Main content
                #[wrap(Some)]
                set_content = &gtk4::Paned {
                    set_position: 200,
                    set_shrink_start_child: false,
                    set_shrink_end_child: false,

                    // Sidebar
                    #[wrap(Some)]
                    set_start_child = model.sidebar.widget(),

                    // File view
                    #[wrap(Some)]
                    set_end_child = &gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,

                        // Search bar
                        gtk4::SearchBar {
                            gtk4::SearchEntry {
                                set_hexpand: true,
                                set_placeholder_text: Some("Search files..."),
                                connect_search_changed[sender] => move |entry| {
                                    let text = entry.text();
                                    if text.is_empty() {
                                        sender.input(AppInput::ClearSearch);
                                    } else {
                                        sender.input(AppInput::Search(text.to_string()));
                                    }
                                },
                            },
                        },

                        // File view
                        model.file_view.widget().clone() {
                            set_vexpand: true,
                            set_hexpand: true,
                        },
                    },
                },

                // Status bar
                add_bottom_bar = &gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 8,
                    add_css_class: "toolbar",

                    gtk4::Label {
                        #[watch]
                        set_label: &format!("{} items selected", model.selected_files.len()),
                    },

                    gtk4::Separator {
                        set_orientation: gtk4::Orientation::Vertical,
                    },

                    gtk4::Label {
                        #[watch]
                        set_label: model.current_path.to_str().unwrap_or(""),
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        info!("Initializing Winux Files");

        // Load configuration
        let config = Config::load().unwrap_or_default();

        // Get initial path
        let current_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        // Initialize components
        let file_view = FileView::builder()
            .launch(current_path.clone())
            .forward(sender.input_sender(), AppInput::FileViewMsg);

        let sidebar = Sidebar::builder()
            .launch(())
            .forward(sender.input_sender(), AppInput::SidebarMsg);

        let file_ops = FileOperation::builder()
            .launch(())
            .forward(sender.input_sender(), AppInput::FileOpsMsg);

        let model = AppModel {
            current_path,
            config,
            view_mode: ViewMode::Grid,
            show_hidden: false,
            sort_by: SortBy::Name,
            search_query: None,
            selected_files: Vec::new(),
            clipboard: None,
            file_view,
            sidebar,
            file_ops,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AppInput::NavigateTo(path) => {
                debug!("Navigating to {:?}", path);
                if path.is_dir() {
                    self.current_path = path.clone();
                    self.file_view.emit(FileViewInput::LoadDirectory(path));
                    self.selected_files.clear();
                }
            }
            AppInput::GoBack => {
                debug!("Going back");
                // TODO: Implement history
            }
            AppInput::GoForward => {
                debug!("Going forward");
                // TODO: Implement history
            }
            AppInput::GoUp => {
                if let Some(parent) = self.current_path.parent() {
                    sender.input(AppInput::NavigateTo(parent.to_path_buf()));
                }
            }
            AppInput::GoHome => {
                if let Some(home) = dirs::home_dir() {
                    sender.input(AppInput::NavigateTo(home));
                }
            }
            AppInput::Refresh => {
                self.file_view
                    .emit(FileViewInput::LoadDirectory(self.current_path.clone()));
            }
            AppInput::ToggleViewMode => {
                self.view_mode = match self.view_mode {
                    ViewMode::Grid => ViewMode::List,
                    ViewMode::List => ViewMode::Grid,
                };
                self.file_view.emit(FileViewInput::SetViewMode(self.view_mode));
            }
            AppInput::SetViewMode(mode) => {
                self.view_mode = mode;
                self.file_view.emit(FileViewInput::SetViewMode(mode));
            }
            AppInput::ToggleHidden => {
                self.show_hidden = !self.show_hidden;
                self.file_view.emit(FileViewInput::SetShowHidden(self.show_hidden));
            }
            AppInput::SetSortBy(sort) => {
                self.sort_by = sort;
                self.file_view.emit(FileViewInput::SetSortBy(sort));
            }
            AppInput::Search(query) => {
                self.search_query = Some(query.clone());
                self.file_view.emit(FileViewInput::Search(query));
            }
            AppInput::ClearSearch => {
                self.search_query = None;
                self.file_view.emit(FileViewInput::ClearSearch);
            }
            AppInput::Select(files) => {
                self.selected_files = files;
            }
            AppInput::OpenSelected => {
                for file in &self.selected_files {
                    if file.is_dir() {
                        sender.input(AppInput::NavigateTo(file.clone()));
                        break;
                    } else {
                        // Open file with default application
                        let _ = open::that(file);
                    }
                }
            }
            AppInput::Copy => {
                if !self.selected_files.is_empty() {
                    self.clipboard = Some(ClipboardContents {
                        files: self.selected_files.clone(),
                        is_cut: false,
                    });
                }
            }
            AppInput::Cut => {
                if !self.selected_files.is_empty() {
                    self.clipboard = Some(ClipboardContents {
                        files: self.selected_files.clone(),
                        is_cut: true,
                    });
                }
            }
            AppInput::Paste => {
                if let Some(clipboard) = &self.clipboard {
                    self.file_ops.emit(crate::file_ops::FileOperationInput::Paste {
                        files: clipboard.files.clone(),
                        destination: self.current_path.clone(),
                        is_cut: clipboard.is_cut,
                    });
                    if clipboard.is_cut {
                        self.clipboard = None;
                    }
                }
            }
            AppInput::Delete => {
                if !self.selected_files.is_empty() {
                    self.file_ops.emit(crate::file_ops::FileOperationInput::Delete {
                        files: self.selected_files.clone(),
                    });
                }
            }
            AppInput::Rename(new_name) => {
                if let Some(file) = self.selected_files.first() {
                    self.file_ops.emit(crate::file_ops::FileOperationInput::Rename {
                        source: file.clone(),
                        new_name,
                    });
                }
            }
            AppInput::NewFolder(name) => {
                let path = self.current_path.join(&name);
                self.file_ops.emit(crate::file_ops::FileOperationInput::CreateDirectory { path });
            }
            AppInput::NewFile(name) => {
                let path = self.current_path.join(&name);
                self.file_ops.emit(crate::file_ops::FileOperationInput::CreateFile { path });
            }
            AppInput::ShowProperties => {
                // TODO: Show properties dialog
            }
            AppInput::OpenTerminal => {
                let _ = std::process::Command::new("winux-terminal")
                    .current_dir(&self.current_path)
                    .spawn();
            }
            AppInput::SidebarMsg(msg) => match msg {
                SidebarOutput::Navigate(path) => {
                    sender.input(AppInput::NavigateTo(path));
                }
            },
            AppInput::FileViewMsg(msg) => match msg {
                FileViewOutput::SelectionChanged(files) => {
                    sender.input(AppInput::Select(files));
                }
                FileViewOutput::OpenFile(path) => {
                    if path.is_dir() {
                        sender.input(AppInput::NavigateTo(path));
                    } else {
                        let _ = open::that(path);
                    }
                }
            },
            AppInput::FileOpsMsg(msg) => match msg {
                FileOperationMsg::OperationComplete => {
                    sender.input(AppInput::Refresh);
                }
                FileOperationMsg::OperationError(err) => {
                    tracing::error!("File operation failed: {}", err);
                }
                FileOperationMsg::Progress(_progress) => {
                    // Update progress indicator
                }
            },
        }
    }
}
