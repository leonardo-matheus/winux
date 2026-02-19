//! File system watcher module

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use notify::{Watcher, RecursiveMode, Event, EventKind};

/// File event
#[derive(Debug, Clone)]
pub struct FileEvent {
    /// Event kind
    pub kind: FileEventKind,
    /// File path
    pub path: PathBuf,
}

/// File event kinds
#[derive(Debug, Clone)]
pub enum FileEventKind {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed (from, to)
    Renamed {
        from: PathBuf,
        to: PathBuf,
    },
}

/// File system watcher
pub struct FileWatcher {
    watcher: notify::RecommendedWatcher,
}

impl FileWatcher {
    /// Create a new file watcher for the given path
    pub fn new(path: &Path) -> Result<(Self, mpsc::Receiver<FileEvent>)> {
        let (tx, rx) = mpsc::channel(1000);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Some(file_event) = Self::convert_event(event) {
                        let _ = tx.blocking_send(file_event);
                    }
                }
                Err(e) => {
                    tracing::error!("Watch error: {:?}", e);
                }
            }
        })?;

        watcher.watch(path, RecursiveMode::Recursive)?;

        Ok((Self { watcher }, rx))
    }

    /// Convert notify event to our FileEvent
    fn convert_event(event: Event) -> Option<FileEvent> {
        let path = event.paths.first()?.clone();

        let kind = match event.kind {
            EventKind::Create(_) => FileEventKind::Created,
            EventKind::Modify(_) => FileEventKind::Modified,
            EventKind::Remove(_) => FileEventKind::Deleted,
            EventKind::Rename(rename_mode) => {
                match rename_mode {
                    notify::event::RenameMode::From => {
                        // Wait for the "To" event
                        return None;
                    }
                    notify::event::RenameMode::To => {
                        // This should have the destination path
                        FileEventKind::Modified
                    }
                    notify::event::RenameMode::Both => {
                        if event.paths.len() >= 2 {
                            FileEventKind::Renamed {
                                from: event.paths[0].clone(),
                                to: event.paths[1].clone(),
                            }
                        } else {
                            FileEventKind::Modified
                        }
                    }
                    _ => FileEventKind::Modified,
                }
            }
            _ => return None,
        };

        Some(FileEvent { kind, path })
    }

    /// Stop watching
    pub fn stop(&mut self) -> Result<()> {
        // Watcher stops when dropped
        Ok(())
    }
}

/// Debounced file watcher - aggregates events over a time window
pub struct DebouncedWatcher {
    watcher: FileWatcher,
    debounce_duration: Duration,
}

impl DebouncedWatcher {
    /// Create a new debounced watcher
    pub fn new(path: &Path, debounce_ms: u64) -> Result<(Self, mpsc::Receiver<Vec<FileEvent>>)> {
        let (watcher, mut inner_rx) = FileWatcher::new(path)?;
        let (tx, rx) = mpsc::channel(100);
        let debounce_duration = Duration::from_millis(debounce_ms);

        // Spawn debounce task
        tokio::spawn(async move {
            let mut pending_events: Vec<FileEvent> = Vec::new();
            let mut last_event_time = std::time::Instant::now();

            loop {
                tokio::select! {
                    Some(event) = inner_rx.recv() => {
                        pending_events.push(event);
                        last_event_time = std::time::Instant::now();
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        if !pending_events.is_empty() && last_event_time.elapsed() >= debounce_duration {
                            // Deduplicate and send events
                            let events = Self::deduplicate_events(&pending_events);
                            pending_events.clear();

                            if !events.is_empty() {
                                if tx.send(events).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok((Self { watcher, debounce_duration }, rx))
    }

    /// Deduplicate events - keep only the latest event per path
    fn deduplicate_events(events: &[FileEvent]) -> Vec<FileEvent> {
        use std::collections::HashMap;

        let mut latest: HashMap<PathBuf, FileEvent> = HashMap::new();

        for event in events {
            latest.insert(event.path.clone(), event.clone());
        }

        latest.into_values().collect()
    }
}

/// Ignore pattern matcher
pub struct IgnoreMatcher {
    patterns: Vec<glob::Pattern>,
}

impl IgnoreMatcher {
    /// Create a new ignore matcher
    pub fn new(patterns: &[String]) -> Result<Self> {
        let patterns: Result<Vec<_>, _> = patterns
            .iter()
            .map(|p| glob::Pattern::new(p))
            .collect();

        Ok(Self {
            patterns: patterns?,
        })
    }

    /// Check if a path should be ignored
    pub fn is_ignored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.patterns {
            if pattern.matches(&path_str) {
                return true;
            }

            // Also check file name only
            if let Some(name) = path.file_name() {
                if pattern.matches(&name.to_string_lossy()) {
                    return true;
                }
            }
        }

        false
    }

    /// Add a pattern
    pub fn add_pattern(&mut self, pattern: &str) -> Result<()> {
        self.patterns.push(glob::Pattern::new(pattern)?);
        Ok(())
    }

    /// Remove a pattern
    pub fn remove_pattern(&mut self, pattern: &str) {
        self.patterns.retain(|p| p.as_str() != pattern);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_matcher() {
        let patterns = vec![
            "*.tmp".to_string(),
            ".git/**".to_string(),
            "node_modules/**".to_string(),
        ];

        let matcher = IgnoreMatcher::new(&patterns).unwrap();

        assert!(matcher.is_ignored(Path::new("file.tmp")));
        assert!(matcher.is_ignored(Path::new("/home/user/file.tmp")));
        assert!(!matcher.is_ignored(Path::new("file.txt")));
    }
}
