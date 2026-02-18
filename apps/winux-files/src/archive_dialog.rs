//! Archive Dialog Module - UI dialogs for archive operations
//!
//! Provides dialogs for:
//! - Extraction (destination, options)
//! - Compression (format, level, password)
//! - Archive content preview

use std::path::PathBuf;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use relm4::prelude::*;

use crate::archive::{
    ArchiveEntry, ArchiveFormat, ArchiveInfo, ArchiveManager, ArchiveProgress,
    CompressionLevel, CompressOptions, ExtractOptions,
};

// ============================================================================
// Extract Dialog
// ============================================================================

/// Input messages for the extract dialog
#[derive(Debug)]
pub enum ExtractDialogInput {
    /// Set the archive to extract
    SetArchive(PathBuf),
    /// Open destination folder chooser
    ChooseDestination,
    /// Toggle preserve permissions
    TogglePreservePermissions,
    /// Toggle overwrite existing
    ToggleOverwrite,
    /// Set password
    SetPassword(String),
    /// Start extraction
    Extract,
    /// Close dialog
    Close,
    /// Update progress
    UpdateProgress(ArchiveProgress),
    /// Extraction complete
    ExtractionComplete(Result<(), String>),
}

/// Output messages from the extract dialog
#[derive(Debug)]
pub enum ExtractDialogOutput {
    /// Extraction started
    ExtractionStarted(PathBuf),
    /// Extraction completed successfully
    ExtractionCompleted(PathBuf),
    /// Extraction failed
    ExtractionFailed(String),
    /// Dialog closed
    Closed,
}

/// Model for the extract dialog
pub struct ExtractDialog {
    /// Archive path
    archive_path: Option<PathBuf>,
    /// Archive info
    archive_info: Option<ArchiveInfo>,
    /// Destination directory
    destination: PathBuf,
    /// Preserve file permissions
    preserve_permissions: bool,
    /// Overwrite existing files
    overwrite: bool,
    /// Password for encrypted archives
    password: Option<String>,
    /// Is extracting
    is_extracting: bool,
    /// Current progress
    progress: Option<ArchiveProgress>,
    /// Visible state
    visible: bool,
}

#[relm4::component(pub)]
impl Component for ExtractDialog {
    type Init = ();
    type Input = ExtractDialogInput;
    type Output = ExtractDialogOutput;
    type CommandOutput = ();

    view! {
        adw::Window {
            set_title: Some("Extract Archive"),
            set_default_width: 500,
            set_default_height: 400,
            set_modal: true,
            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                // Header bar
                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Extract Archive",
                        #[watch]
                        set_subtitle: &model.archive_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    },
                },

                // Content
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 12,

                    // Archive info section
                    adw::PreferencesGroup {
                        set_title: "Archive Information",

                        adw::ActionRow {
                            set_title: "Format",
                            #[watch]
                            set_subtitle: &model.archive_info
                                .as_ref()
                                .map(|i| i.format.display_name())
                                .unwrap_or("Unknown"),
                        },

                        adw::ActionRow {
                            set_title: "Files",
                            #[watch]
                            set_subtitle: &model.archive_info
                                .as_ref()
                                .map(|i| format!("{} files, {} directories", i.file_count, i.dir_count))
                                .unwrap_or_default(),
                        },

                        adw::ActionRow {
                            set_title: "Size",
                            #[watch]
                            set_subtitle: &model.archive_info
                                .as_ref()
                                .map(|i| format!("{} (compressed: {})",
                                    format_size(i.total_size),
                                    format_size(i.compressed_size)))
                                .unwrap_or_default(),
                        },
                    },

                    // Destination section
                    adw::PreferencesGroup {
                        set_title: "Destination",

                        adw::ActionRow {
                            set_title: "Extract to",
                            #[watch]
                            set_subtitle: &model.destination.to_string_lossy(),

                            add_suffix = &gtk::Button {
                                set_icon_name: "folder-symbolic",
                                set_valign: gtk::Align::Center,
                                add_css_class: "flat",
                                connect_clicked => ExtractDialogInput::ChooseDestination,
                            },
                        },
                    },

                    // Options section
                    adw::PreferencesGroup {
                        set_title: "Options",

                        adw::SwitchRow {
                            set_title: "Preserve permissions",
                            set_subtitle: "Keep original file permissions (Unix only)",
                            #[watch]
                            set_active: model.preserve_permissions,
                            connect_active_notify => move |_| {
                                ExtractDialogInput::TogglePreservePermissions
                            },
                        },

                        adw::SwitchRow {
                            set_title: "Overwrite existing",
                            set_subtitle: "Replace files that already exist",
                            #[watch]
                            set_active: model.overwrite,
                            connect_active_notify => move |_| {
                                ExtractDialogInput::ToggleOverwrite
                            },
                        },
                    },

                    // Password section (shown if archive is encrypted)
                    #[local_ref]
                    password_group -> adw::PreferencesGroup {
                        set_title: "Password",
                        #[watch]
                        set_visible: model.archive_info
                            .as_ref()
                            .map(|i| i.encrypted)
                            .unwrap_or(false),

                        adw::PasswordEntryRow {
                            set_title: "Archive password",
                            connect_changed[sender] => move |entry| {
                                sender.input(ExtractDialogInput::SetPassword(
                                    entry.text().to_string()
                                ));
                            },
                        },
                    },

                    // Progress section
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 6,
                        #[watch]
                        set_visible: model.is_extracting,

                        gtk::ProgressBar {
                            #[watch]
                            set_fraction: model.progress
                                .as_ref()
                                .map(|p| p.percentage() / 100.0)
                                .unwrap_or(0.0),
                            set_show_text: true,
                            #[watch]
                            set_text: model.progress
                                .as_ref()
                                .map(|p| format!("{}/{} files", p.current_index, p.total_files))
                                .as_deref(),
                        },

                        gtk::Label {
                            set_xalign: 0.0,
                            add_css_class: "dim-label",
                            set_ellipsize: gtk::pango::EllipsizeMode::Middle,
                            #[watch]
                            set_label: &model.progress
                                .as_ref()
                                .and_then(|p| p.current_file.clone())
                                .unwrap_or_default(),
                        },
                    },

                    // Spacer
                    gtk::Box {
                        set_vexpand: true,
                    },

                    // Action buttons
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 12,
                        set_halign: gtk::Align::End,

                        gtk::Button {
                            set_label: "Cancel",
                            connect_clicked => ExtractDialogInput::Close,
                        },

                        gtk::Button {
                            set_label: "Extract",
                            add_css_class: "suggested-action",
                            #[watch]
                            set_sensitive: !model.is_extracting && model.archive_path.is_some(),
                            connect_clicked => ExtractDialogInput::Extract,
                        },
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
        let model = Self {
            archive_path: None,
            archive_info: None,
            destination: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            preserve_permissions: true,
            overwrite: false,
            password: None,
            is_extracting: false,
            progress: None,
            visible: false,
        };

        let password_group = adw::PreferencesGroup::new();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            ExtractDialogInput::SetArchive(path) => {
                self.archive_path = Some(path.clone());
                self.visible = true;

                // Load archive info
                let manager = ArchiveManager::new();
                if let Ok(info) = manager.get_info(&path, None) {
                    // Set default destination to archive directory
                    if let Some(parent) = path.parent() {
                        self.destination = parent.to_path_buf();
                    }
                    self.archive_info = Some(info);
                }
            }

            ExtractDialogInput::ChooseDestination => {
                let dialog = gtk::FileDialog::builder()
                    .title("Choose Destination")
                    .modal(true)
                    .build();

                let sender_clone = sender.clone();
                let current_dest = self.destination.clone();

                glib::spawn_future_local(async move {
                    let file = gio::File::for_path(&current_dest);
                    dialog.set_initial_folder(Some(&file));

                    // Note: In a real implementation, we'd use select_folder
                    // For now, this is a placeholder
                });
            }

            ExtractDialogInput::TogglePreservePermissions => {
                self.preserve_permissions = !self.preserve_permissions;
            }

            ExtractDialogInput::ToggleOverwrite => {
                self.overwrite = !self.overwrite;
            }

            ExtractDialogInput::SetPassword(password) => {
                self.password = if password.is_empty() {
                    None
                } else {
                    Some(password)
                };
            }

            ExtractDialogInput::Extract => {
                if let Some(ref path) = self.archive_path {
                    self.is_extracting = true;

                    let path = path.clone();
                    let options = ExtractOptions {
                        destination: self.destination.clone(),
                        preserve_permissions: self.preserve_permissions,
                        overwrite: self.overwrite,
                        paths: Vec::new(),
                        password: self.password.clone(),
                        progress: None, // Progress callback would be set up here
                    };

                    let _ = sender.output(ExtractDialogOutput::ExtractionStarted(path.clone()));

                    // Spawn extraction task
                    let sender_clone = sender.clone();
                    std::thread::spawn(move || {
                        let manager = ArchiveManager::new();
                        let result = manager.extract(&path, &options);

                        match result {
                            Ok(()) => {
                                sender_clone.input(ExtractDialogInput::ExtractionComplete(Ok(())));
                            }
                            Err(e) => {
                                sender_clone.input(ExtractDialogInput::ExtractionComplete(
                                    Err(e.to_string()),
                                ));
                            }
                        }
                    });
                }
            }

            ExtractDialogInput::Close => {
                self.visible = false;
                let _ = sender.output(ExtractDialogOutput::Closed);
            }

            ExtractDialogInput::UpdateProgress(progress) => {
                self.progress = Some(progress);
            }

            ExtractDialogInput::ExtractionComplete(result) => {
                self.is_extracting = false;
                self.progress = None;

                match result {
                    Ok(()) => {
                        if let Some(ref path) = self.archive_path {
                            let _ = sender.output(ExtractDialogOutput::ExtractionCompleted(
                                path.clone(),
                            ));
                        }
                        self.visible = false;
                    }
                    Err(e) => {
                        let _ = sender.output(ExtractDialogOutput::ExtractionFailed(e));
                    }
                }
            }
        }
    }
}

// ============================================================================
// Compress Dialog
// ============================================================================

/// Input messages for the compress dialog
#[derive(Debug)]
pub enum CompressDialogInput {
    /// Set files to compress
    SetFiles(Vec<PathBuf>),
    /// Set archive format
    SetFormat(ArchiveFormat),
    /// Set compression level
    SetLevel(CompressionLevel),
    /// Set output filename
    SetFilename(String),
    /// Set password
    SetPassword(String),
    /// Start compression
    Compress,
    /// Close dialog
    Close,
    /// Update progress
    UpdateProgress(ArchiveProgress),
    /// Compression complete
    CompressionComplete(Result<PathBuf, String>),
}

/// Output messages from the compress dialog
#[derive(Debug)]
pub enum CompressDialogOutput {
    /// Compression started
    CompressionStarted,
    /// Compression completed successfully
    CompressionCompleted(PathBuf),
    /// Compression failed
    CompressionFailed(String),
    /// Dialog closed
    Closed,
}

/// Model for the compress dialog
pub struct CompressDialog {
    /// Files to compress
    files: Vec<PathBuf>,
    /// Selected format
    format: ArchiveFormat,
    /// Compression level
    level: CompressionLevel,
    /// Output filename (without extension)
    filename: String,
    /// Output directory
    output_dir: PathBuf,
    /// Password for encryption
    password: Option<String>,
    /// Is compressing
    is_compressing: bool,
    /// Current progress
    progress: Option<ArchiveProgress>,
    /// Visible state
    visible: bool,
}

impl CompressDialog {
    /// Get the full output path
    fn output_path(&self) -> PathBuf {
        let filename = format!("{}{}", self.filename, self.format.extension());
        self.output_dir.join(filename)
    }
}

#[relm4::component(pub)]
impl Component for CompressDialog {
    type Init = ();
    type Input = CompressDialogInput;
    type Output = CompressDialogOutput;
    type CommandOutput = ();

    view! {
        adw::Window {
            set_title: Some("Create Archive"),
            set_default_width: 500,
            set_default_height: 500,
            set_modal: true,
            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                // Header bar
                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Create Archive",
                        #[watch]
                        set_subtitle: &format!("{} files selected", model.files.len()),
                    },
                },

                // Content
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 12,

                    // Filename section
                    adw::PreferencesGroup {
                        set_title: "Archive Name",

                        adw::EntryRow {
                            set_title: "Filename",
                            #[watch]
                            set_text: &model.filename,
                            connect_changed[sender] => move |entry| {
                                sender.input(CompressDialogInput::SetFilename(
                                    entry.text().to_string()
                                ));
                            },
                        },

                        adw::ActionRow {
                            set_title: "Output location",
                            #[watch]
                            set_subtitle: &model.output_dir.to_string_lossy(),
                        },
                    },

                    // Format section
                    adw::PreferencesGroup {
                        set_title: "Format",

                        adw::ComboRow {
                            set_title: "Archive format",
                            #[wrap(Some)]
                            set_model = &gtk::StringList::new(&[
                                "ZIP (.zip)",
                                "TAR (.tar)",
                                "Gzipped TAR (.tar.gz)",
                                "Bzip2 TAR (.tar.bz2)",
                                "XZ TAR (.tar.xz)",
                                "Zstd TAR (.tar.zst)",
                                "7-Zip (.7z)",
                            ]),
                            #[watch]
                            set_selected: match model.format {
                                ArchiveFormat::Zip => 0,
                                ArchiveFormat::Tar => 1,
                                ArchiveFormat::TarGz => 2,
                                ArchiveFormat::TarBz2 => 3,
                                ArchiveFormat::TarXz => 4,
                                ArchiveFormat::TarZst => 5,
                                ArchiveFormat::SevenZip => 6,
                                _ => 0,
                            },
                            connect_selected_notify[sender] => move |row| {
                                let format = match row.selected() {
                                    0 => ArchiveFormat::Zip,
                                    1 => ArchiveFormat::Tar,
                                    2 => ArchiveFormat::TarGz,
                                    3 => ArchiveFormat::TarBz2,
                                    4 => ArchiveFormat::TarXz,
                                    5 => ArchiveFormat::TarZst,
                                    6 => ArchiveFormat::SevenZip,
                                    _ => ArchiveFormat::Zip,
                                };
                                sender.input(CompressDialogInput::SetFormat(format));
                            },
                        },
                    },

                    // Compression level section
                    adw::PreferencesGroup {
                        set_title: "Compression",
                        #[watch]
                        set_visible: model.format.supports_compression_level(),

                        adw::ComboRow {
                            set_title: "Compression level",
                            #[wrap(Some)]
                            set_model = &gtk::StringList::new(&[
                                "Store (no compression)",
                                "Fast (less compression)",
                                "Normal (balanced)",
                                "Best (maximum compression)",
                            ]),
                            #[watch]
                            set_selected: match model.level {
                                CompressionLevel::Store => 0,
                                CompressionLevel::Fast => 1,
                                CompressionLevel::Normal => 2,
                                CompressionLevel::Best => 3,
                                CompressionLevel::Custom(_) => 2,
                            },
                            connect_selected_notify[sender] => move |row| {
                                let level = match row.selected() {
                                    0 => CompressionLevel::Store,
                                    1 => CompressionLevel::Fast,
                                    2 => CompressionLevel::Normal,
                                    3 => CompressionLevel::Best,
                                    _ => CompressionLevel::Normal,
                                };
                                sender.input(CompressDialogInput::SetLevel(level));
                            },
                        },
                    },

                    // Password section
                    adw::PreferencesGroup {
                        set_title: "Password Protection",
                        #[watch]
                        set_visible: model.format.supports_password(),

                        adw::PasswordEntryRow {
                            set_title: "Password (optional)",
                            connect_changed[sender] => move |entry| {
                                sender.input(CompressDialogInput::SetPassword(
                                    entry.text().to_string()
                                ));
                            },
                        },
                    },

                    // Progress section
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 6,
                        #[watch]
                        set_visible: model.is_compressing,

                        gtk::ProgressBar {
                            #[watch]
                            set_fraction: model.progress
                                .as_ref()
                                .map(|p| p.percentage() / 100.0)
                                .unwrap_or(0.0),
                            set_show_text: true,
                            #[watch]
                            set_text: model.progress
                                .as_ref()
                                .map(|p| format!("{}/{} files", p.current_index, p.total_files))
                                .as_deref(),
                        },

                        gtk::Label {
                            set_xalign: 0.0,
                            add_css_class: "dim-label",
                            set_ellipsize: gtk::pango::EllipsizeMode::Middle,
                            #[watch]
                            set_label: &model.progress
                                .as_ref()
                                .and_then(|p| p.current_file.clone())
                                .unwrap_or_default(),
                        },
                    },

                    // Spacer
                    gtk::Box {
                        set_vexpand: true,
                    },

                    // Action buttons
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 12,
                        set_halign: gtk::Align::End,

                        gtk::Button {
                            set_label: "Cancel",
                            connect_clicked => CompressDialogInput::Close,
                        },

                        gtk::Button {
                            set_label: "Create",
                            add_css_class: "suggested-action",
                            #[watch]
                            set_sensitive: !model.is_compressing
                                && !model.files.is_empty()
                                && !model.filename.is_empty(),
                            connect_clicked => CompressDialogInput::Compress,
                        },
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
        let model = Self {
            files: Vec::new(),
            format: ArchiveFormat::Zip,
            level: CompressionLevel::Normal,
            filename: String::from("archive"),
            output_dir: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            password: None,
            is_compressing: false,
            progress: None,
            visible: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            CompressDialogInput::SetFiles(files) => {
                self.files = files.clone();
                self.visible = true;

                // Set default filename based on first file/directory
                if let Some(first) = files.first() {
                    if let Some(name) = first.file_stem() {
                        self.filename = name.to_string_lossy().to_string();
                    }
                    if let Some(parent) = first.parent() {
                        self.output_dir = parent.to_path_buf();
                    }
                }
            }

            CompressDialogInput::SetFormat(format) => {
                self.format = format;
            }

            CompressDialogInput::SetLevel(level) => {
                self.level = level;
            }

            CompressDialogInput::SetFilename(filename) => {
                self.filename = filename;
            }

            CompressDialogInput::SetPassword(password) => {
                self.password = if password.is_empty() {
                    None
                } else {
                    Some(password)
                };
            }

            CompressDialogInput::Compress => {
                if !self.files.is_empty() {
                    self.is_compressing = true;

                    let files = self.files.clone();
                    let output = self.output_path();
                    let options = CompressOptions {
                        output: output.clone(),
                        format: self.format,
                        level: self.level,
                        password: self.password.clone(),
                        comment: None,
                        progress: None,
                    };

                    let _ = sender.output(CompressDialogOutput::CompressionStarted);

                    // Spawn compression task
                    let sender_clone = sender.clone();
                    std::thread::spawn(move || {
                        let manager = ArchiveManager::new();
                        let result = manager.compress(&files, &options);

                        match result {
                            Ok(()) => {
                                sender_clone.input(CompressDialogInput::CompressionComplete(
                                    Ok(output),
                                ));
                            }
                            Err(e) => {
                                sender_clone.input(CompressDialogInput::CompressionComplete(
                                    Err(e.to_string()),
                                ));
                            }
                        }
                    });
                }
            }

            CompressDialogInput::Close => {
                self.visible = false;
                let _ = sender.output(CompressDialogOutput::Closed);
            }

            CompressDialogInput::UpdateProgress(progress) => {
                self.progress = Some(progress);
            }

            CompressDialogInput::CompressionComplete(result) => {
                self.is_compressing = false;
                self.progress = None;

                match result {
                    Ok(path) => {
                        let _ = sender.output(CompressDialogOutput::CompressionCompleted(path));
                        self.visible = false;
                    }
                    Err(e) => {
                        let _ = sender.output(CompressDialogOutput::CompressionFailed(e));
                    }
                }
            }
        }
    }
}

// ============================================================================
// Archive Preview Dialog
// ============================================================================

/// Input messages for the preview dialog
#[derive(Debug)]
pub enum PreviewDialogInput {
    /// Set the archive to preview
    SetArchive(PathBuf),
    /// Close dialog
    Close,
    /// Extract selected files
    ExtractSelected,
    /// Select/deselect entry
    ToggleEntry(usize),
    /// Select all
    SelectAll,
    /// Deselect all
    DeselectAll,
}

/// Output messages from the preview dialog
#[derive(Debug)]
pub enum PreviewDialogOutput {
    /// Extract specific files
    ExtractFiles(PathBuf, Vec<PathBuf>),
    /// Dialog closed
    Closed,
}

/// Model for the preview dialog
pub struct PreviewDialog {
    /// Archive path
    archive_path: Option<PathBuf>,
    /// Archive info
    archive_info: Option<ArchiveInfo>,
    /// Archive entries
    entries: Vec<ArchiveEntry>,
    /// Selected entries (indices)
    selected: Vec<usize>,
    /// Visible state
    visible: bool,
}

#[relm4::component(pub)]
impl Component for PreviewDialog {
    type Init = ();
    type Input = PreviewDialogInput;
    type Output = PreviewDialogOutput;
    type CommandOutput = ();

    view! {
        adw::Window {
            set_title: Some("Archive Contents"),
            set_default_width: 700,
            set_default_height: 500,
            set_modal: true,
            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                // Header bar
                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Archive Contents",
                        #[watch]
                        set_subtitle: &model.archive_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    },

                    pack_start = &gtk::Button {
                        set_icon_name: "edit-select-all-symbolic",
                        set_tooltip_text: Some("Select All"),
                        connect_clicked => PreviewDialogInput::SelectAll,
                    },
                },

                // Info bar
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,
                    set_margin_all: 12,
                    add_css_class: "toolbar",

                    gtk::Label {
                        #[watch]
                        set_label: &model.archive_info
                            .as_ref()
                            .map(|i| format!("{} files, {} directories",
                                i.file_count, i.dir_count))
                            .unwrap_or_default(),
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &model.archive_info
                            .as_ref()
                            .map(|i| format!("Total: {} | Compressed: {} ({:.1}% ratio)",
                                format_size(i.total_size),
                                format_size(i.compressed_size),
                                i.compression_ratio()))
                            .unwrap_or_default(),
                    },

                    gtk::Box {
                        set_hexpand: true,
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &format!("{} selected", model.selected.len()),
                        add_css_class: "dim-label",
                    },
                },

                // Content list
                gtk::ScrolledWindow {
                    set_vexpand: true,

                    gtk::ListView {
                        // In a real implementation, this would use a GtkListStore
                        // and proper selection handling
                    },
                },

                // Action bar
                gtk::ActionBar {
                    pack_start = &gtk::Button {
                        set_label: "Close",
                        connect_clicked => PreviewDialogInput::Close,
                    },

                    pack_end = &gtk::Button {
                        set_label: "Extract Selected",
                        add_css_class: "suggested-action",
                        #[watch]
                        set_sensitive: !model.selected.is_empty(),
                        connect_clicked => PreviewDialogInput::ExtractSelected,
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
        let model = Self {
            archive_path: None,
            archive_info: None,
            entries: Vec::new(),
            selected: Vec::new(),
            visible: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            PreviewDialogInput::SetArchive(path) => {
                self.archive_path = Some(path.clone());
                self.visible = true;
                self.selected.clear();

                // Load archive contents
                let manager = ArchiveManager::new();
                if let Ok(info) = manager.get_info(&path, None) {
                    self.archive_info = Some(info);
                }
                if let Ok(entries) = manager.list_contents(&path, None) {
                    self.entries = entries;
                }
            }

            PreviewDialogInput::Close => {
                self.visible = false;
                let _ = sender.output(PreviewDialogOutput::Closed);
            }

            PreviewDialogInput::ExtractSelected => {
                if let Some(ref path) = self.archive_path {
                    let selected_paths: Vec<PathBuf> = self
                        .selected
                        .iter()
                        .filter_map(|&i| self.entries.get(i))
                        .map(|e| e.path.clone())
                        .collect();

                    let _ = sender.output(PreviewDialogOutput::ExtractFiles(
                        path.clone(),
                        selected_paths,
                    ));
                }
            }

            PreviewDialogInput::ToggleEntry(index) => {
                if let Some(pos) = self.selected.iter().position(|&i| i == index) {
                    self.selected.remove(pos);
                } else {
                    self.selected.push(index);
                }
            }

            PreviewDialogInput::SelectAll => {
                self.selected = (0..self.entries.len()).collect();
            }

            PreviewDialogInput::DeselectAll => {
                self.selected.clear();
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format file size for display
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }
}
