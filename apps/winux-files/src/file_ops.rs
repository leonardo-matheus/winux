//! File operations - Copy, Move, Delete, Rename

use std::fs;
use std::path::PathBuf;

use relm4::prelude::*;
use tracing::{debug, error, info};

/// File operation progress
#[derive(Debug, Clone)]
pub struct OperationProgress {
    pub current_file: String,
    pub files_done: usize,
    pub files_total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

impl OperationProgress {
    pub fn percentage(&self) -> f64 {
        if self.bytes_total == 0 {
            0.0
        } else {
            (self.bytes_done as f64 / self.bytes_total as f64) * 100.0
        }
    }
}

/// File operation component
pub struct FileOperation {
    /// Current operation in progress
    current_operation: Option<String>,
    /// Progress of current operation
    progress: Option<OperationProgress>,
}

/// File operation input messages
#[derive(Debug)]
pub enum FileOperationInput {
    /// Copy files to destination
    Copy {
        files: Vec<PathBuf>,
        destination: PathBuf,
    },
    /// Move files to destination
    Move {
        files: Vec<PathBuf>,
        destination: PathBuf,
    },
    /// Paste files (copy or move based on is_cut)
    Paste {
        files: Vec<PathBuf>,
        destination: PathBuf,
        is_cut: bool,
    },
    /// Delete files
    Delete {
        files: Vec<PathBuf>,
    },
    /// Rename a file
    Rename {
        source: PathBuf,
        new_name: String,
    },
    /// Create a new directory
    CreateDirectory {
        path: PathBuf,
    },
    /// Create a new file
    CreateFile {
        path: PathBuf,
    },
    /// Cancel current operation
    Cancel,
}

/// File operation output messages
#[derive(Debug)]
pub enum FileOperationMsg {
    OperationComplete,
    OperationError(String),
    Progress(OperationProgress),
}

/// Command output for async operations
pub enum FileOpCommand {
    Complete,
    Error(String),
    Progress(OperationProgress),
}

#[relm4::component(pub)]
impl Component for FileOperation {
    type Init = ();
    type Input = FileOperationInput;
    type Output = FileOperationMsg;
    type CommandOutput = FileOpCommand;

    view! {
        gtk4::Box {
            set_visible: false,
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FileOperation {
            current_operation: None,
            progress: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            FileOperationInput::Copy { files, destination } => {
                info!("Copying {} files to {:?}", files.len(), destination);
                self.current_operation = Some("Copying files...".to_string());

                sender.oneshot_command(async move {
                    match copy_files(&files, &destination) {
                        Ok(()) => FileOpCommand::Complete,
                        Err(e) => FileOpCommand::Error(e.to_string()),
                    }
                });
            }
            FileOperationInput::Move { files, destination } => {
                info!("Moving {} files to {:?}", files.len(), destination);
                self.current_operation = Some("Moving files...".to_string());

                sender.oneshot_command(async move {
                    match move_files(&files, &destination) {
                        Ok(()) => FileOpCommand::Complete,
                        Err(e) => FileOpCommand::Error(e.to_string()),
                    }
                });
            }
            FileOperationInput::Paste { files, destination, is_cut } => {
                if is_cut {
                    sender.input(FileOperationInput::Move { files, destination });
                } else {
                    sender.input(FileOperationInput::Copy { files, destination });
                }
            }
            FileOperationInput::Delete { files } => {
                info!("Deleting {} files", files.len());
                self.current_operation = Some("Deleting files...".to_string());

                sender.oneshot_command(async move {
                    match delete_files(&files) {
                        Ok(()) => FileOpCommand::Complete,
                        Err(e) => FileOpCommand::Error(e.to_string()),
                    }
                });
            }
            FileOperationInput::Rename { source, new_name } => {
                info!("Renaming {:?} to {}", source, new_name);

                sender.oneshot_command(async move {
                    match rename_file(&source, &new_name) {
                        Ok(()) => FileOpCommand::Complete,
                        Err(e) => FileOpCommand::Error(e.to_string()),
                    }
                });
            }
            FileOperationInput::CreateDirectory { path } => {
                info!("Creating directory: {:?}", path);

                sender.oneshot_command(async move {
                    match fs::create_dir_all(&path) {
                        Ok(()) => FileOpCommand::Complete,
                        Err(e) => FileOpCommand::Error(e.to_string()),
                    }
                });
            }
            FileOperationInput::CreateFile { path } => {
                info!("Creating file: {:?}", path);

                sender.oneshot_command(async move {
                    match fs::File::create(&path) {
                        Ok(_) => FileOpCommand::Complete,
                        Err(e) => FileOpCommand::Error(e.to_string()),
                    }
                });
            }
            FileOperationInput::Cancel => {
                self.current_operation = None;
                self.progress = None;
            }
        }
    }

    fn update_cmd(
        &mut self,
        cmd: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match cmd {
            FileOpCommand::Complete => {
                self.current_operation = None;
                self.progress = None;
                let _ = sender.output(FileOperationMsg::OperationComplete);
            }
            FileOpCommand::Error(err) => {
                error!("File operation error: {}", err);
                self.current_operation = None;
                self.progress = None;
                let _ = sender.output(FileOperationMsg::OperationError(err));
            }
            FileOpCommand::Progress(progress) => {
                self.progress = Some(progress.clone());
                let _ = sender.output(FileOperationMsg::Progress(progress));
            }
        }
    }
}

/// Copy files to destination
fn copy_files(files: &[PathBuf], destination: &PathBuf) -> Result<(), std::io::Error> {
    for file in files {
        let file_name = file.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name")
        })?;
        let dest_path = destination.join(file_name);

        debug!("Copying {:?} to {:?}", file, dest_path);

        if file.is_dir() {
            copy_dir_recursive(file, &dest_path)?;
        } else {
            fs::copy(file, &dest_path)?;
        }
    }
    Ok(())
}

/// Copy directory recursively
fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> Result<(), std::io::Error> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Move files to destination
fn move_files(files: &[PathBuf], destination: &PathBuf) -> Result<(), std::io::Error> {
    for file in files {
        let file_name = file.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name")
        })?;
        let dest_path = destination.join(file_name);

        debug!("Moving {:?} to {:?}", file, dest_path);

        // Try rename first (works if same filesystem)
        if fs::rename(file, &dest_path).is_err() {
            // Fall back to copy + delete
            if file.is_dir() {
                copy_dir_recursive(file, &dest_path)?;
                fs::remove_dir_all(file)?;
            } else {
                fs::copy(file, &dest_path)?;
                fs::remove_file(file)?;
            }
        }
    }
    Ok(())
}

/// Delete files (move to trash if possible)
fn delete_files(files: &[PathBuf]) -> Result<(), std::io::Error> {
    for file in files {
        debug!("Deleting {:?}", file);

        // Try to use trash first
        if trash::delete(file).is_err() {
            // Fall back to permanent delete
            if file.is_dir() {
                fs::remove_dir_all(file)?;
            } else {
                fs::remove_file(file)?;
            }
        }
    }
    Ok(())
}

/// Rename a file
fn rename_file(source: &PathBuf, new_name: &str) -> Result<(), std::io::Error> {
    let parent = source.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "No parent directory")
    })?;
    let dest = parent.join(new_name);

    debug!("Renaming {:?} to {:?}", source, dest);
    fs::rename(source, dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_copy_file() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("test.txt");
        let dest_dir = temp.path().join("dest");

        fs::write(&src, "test content").unwrap();
        fs::create_dir(&dest_dir).unwrap();

        copy_files(&[src.clone()], &dest_dir).unwrap();

        assert!(dest_dir.join("test.txt").exists());
    }

    #[test]
    fn test_move_file() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("test.txt");
        let dest_dir = temp.path().join("dest");

        fs::write(&src, "test content").unwrap();
        fs::create_dir(&dest_dir).unwrap();

        move_files(&[src.clone()], &dest_dir).unwrap();

        assert!(!src.exists());
        assert!(dest_dir.join("test.txt").exists());
    }

    #[test]
    fn test_rename_file() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("old.txt");

        fs::write(&src, "test content").unwrap();

        rename_file(&src, "new.txt").unwrap();

        assert!(!src.exists());
        assert!(temp.path().join("new.txt").exists());
    }
}
