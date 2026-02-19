// Winux Mail - Folder Data Structures
// Copyright (c) 2026 Winux OS Project

use serde::{Deserialize, Serialize};

/// Folder type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FolderType {
    /// Inbox - incoming messages
    Inbox,

    /// Sent - sent messages
    Sent,

    /// Drafts - unsent drafts
    Drafts,

    /// Trash - deleted messages
    Trash,

    /// Spam/Junk
    Spam,

    /// Archive
    Archive,

    /// Starred/Flagged (virtual folder)
    Starred,

    /// All Mail (virtual folder)
    All,

    /// Custom user folder
    Custom,
}

impl FolderType {
    /// Get the icon name for this folder type
    pub fn icon_name(&self) -> &'static str {
        match self {
            FolderType::Inbox => "mail-inbox-symbolic",
            FolderType::Sent => "mail-send-symbolic",
            FolderType::Drafts => "document-edit-symbolic",
            FolderType::Trash => "user-trash-symbolic",
            FolderType::Spam => "mail-mark-junk-symbolic",
            FolderType::Archive => "folder-symbolic",
            FolderType::Starred => "starred-symbolic",
            FolderType::All => "mail-inbox-symbolic",
            FolderType::Custom => "folder-symbolic",
        }
    }

    /// Get the display name for this folder type
    pub fn display_name(&self) -> &'static str {
        match self {
            FolderType::Inbox => "Inbox",
            FolderType::Sent => "Sent",
            FolderType::Drafts => "Drafts",
            FolderType::Trash => "Trash",
            FolderType::Spam => "Spam",
            FolderType::Archive => "Archive",
            FolderType::Starred => "Starred",
            FolderType::All => "All Mail",
            FolderType::Custom => "Folder",
        }
    }

    /// Check if this is a special folder
    pub fn is_special(&self) -> bool {
        !matches!(self, FolderType::Custom)
    }

    /// Check if messages should be auto-expunged
    pub fn auto_expunge(&self) -> bool {
        matches!(self, FolderType::Trash | FolderType::Spam)
    }

    /// Get sort order (for displaying folders)
    pub fn sort_order(&self) -> u8 {
        match self {
            FolderType::Inbox => 0,
            FolderType::Starred => 1,
            FolderType::Sent => 2,
            FolderType::Drafts => 3,
            FolderType::Archive => 4,
            FolderType::Spam => 5,
            FolderType::Trash => 6,
            FolderType::All => 7,
            FolderType::Custom => 8,
        }
    }
}

/// Email folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    /// Unique ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Full IMAP path
    pub path: String,

    /// Folder type
    pub folder_type: FolderType,

    /// Unread message count
    pub unread_count: u32,

    /// Total message count
    pub total_count: u32,

    /// Parent folder ID (for nested folders)
    pub parent_id: Option<String>,

    /// IMAP delimiter (e.g., "/", ".")
    pub delimiter: Option<String>,

    /// Can select this folder (not just a container)
    pub selectable: bool,
}

impl Folder {
    /// Create a new folder
    pub fn new(name: &str, path: &str, folder_type: FolderType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            path: path.to_string(),
            folder_type,
            unread_count: 0,
            total_count: 0,
            parent_id: None,
            delimiter: None,
            selectable: true,
        }
    }

    /// Create standard folders for a new account
    pub fn create_standard_folders(account_id: &str) -> Vec<Folder> {
        vec![
            Folder::new("Inbox", "INBOX", FolderType::Inbox),
            Folder::new("Sent", "Sent", FolderType::Sent),
            Folder::new("Drafts", "Drafts", FolderType::Drafts),
            Folder::new("Spam", "Spam", FolderType::Spam),
            Folder::new("Trash", "Trash", FolderType::Trash),
            Folder::new("Archive", "Archive", FolderType::Archive),
        ]
    }

    /// Get icon name
    pub fn icon(&self) -> &'static str {
        self.folder_type.icon_name()
    }

    /// Get display name (uses folder type name for special folders)
    pub fn display_name(&self) -> &str {
        if self.folder_type.is_special() {
            self.folder_type.display_name()
        } else {
            &self.name
        }
    }

    /// Check if this folder should show unread count badge
    pub fn show_badge(&self) -> bool {
        matches!(
            self.folder_type,
            FolderType::Inbox | FolderType::Drafts | FolderType::Custom
        ) && self.unread_count > 0
    }

    /// Get the depth level (for nested folders)
    pub fn depth(&self, delimiter: &str) -> usize {
        self.path.matches(delimiter).count()
    }

    /// Check if this is a child of another folder
    pub fn is_child_of(&self, parent: &Folder) -> bool {
        if let Some(delim) = &self.delimiter {
            self.path.starts_with(&format!("{}{}", parent.path, delim))
        } else {
            false
        }
    }
}

/// Virtual folder for unified view
#[derive(Debug, Clone)]
pub struct UnifiedFolder {
    /// Folder type
    pub folder_type: FolderType,

    /// Source folders from different accounts
    pub sources: Vec<(String, String)>, // (account_id, folder_path)

    /// Total unread count
    pub unread_count: u32,

    /// Total message count
    pub total_count: u32,
}

impl UnifiedFolder {
    /// Create a unified inbox
    pub fn unified_inbox(accounts: &[(String, Vec<Folder>)]) -> Self {
        let mut sources = Vec::new();
        let mut unread_count = 0;
        let mut total_count = 0;

        for (account_id, folders) in accounts {
            if let Some(inbox) = folders.iter().find(|f| f.folder_type == FolderType::Inbox) {
                sources.push((account_id.clone(), inbox.path.clone()));
                unread_count += inbox.unread_count;
                total_count += inbox.total_count;
            }
        }

        Self {
            folder_type: FolderType::Inbox,
            sources,
            unread_count,
            total_count,
        }
    }

    /// Create unified folder by type
    pub fn unified(folder_type: FolderType, accounts: &[(String, Vec<Folder>)]) -> Self {
        let mut sources = Vec::new();
        let mut unread_count = 0;
        let mut total_count = 0;

        for (account_id, folders) in accounts {
            for folder in folders.iter().filter(|f| f.folder_type == folder_type) {
                sources.push((account_id.clone(), folder.path.clone()));
                unread_count += folder.unread_count;
                total_count += folder.total_count;
            }
        }

        Self {
            folder_type,
            sources,
            unread_count,
            total_count,
        }
    }

    pub fn icon(&self) -> &'static str {
        self.folder_type.icon_name()
    }

    pub fn name(&self) -> String {
        format!("All {}", self.folder_type.display_name())
    }
}

/// Folder tree for hierarchical display
#[derive(Debug, Clone)]
pub struct FolderTree {
    pub folder: Folder,
    pub children: Vec<FolderTree>,
    pub expanded: bool,
}

impl FolderTree {
    /// Build folder tree from flat list
    pub fn build(folders: Vec<Folder>) -> Vec<FolderTree> {
        let mut trees: Vec<FolderTree> = Vec::new();
        let mut folder_map: std::collections::HashMap<String, FolderTree> = std::collections::HashMap::new();

        // First pass: create all trees
        for folder in folders {
            folder_map.insert(folder.id.clone(), FolderTree {
                folder,
                children: Vec::new(),
                expanded: false,
            });
        }

        // Second pass: build hierarchy
        let ids: Vec<String> = folder_map.keys().cloned().collect();

        for id in ids {
            let tree = folder_map.remove(&id).unwrap();

            if let Some(parent_id) = &tree.folder.parent_id {
                if let Some(parent) = folder_map.get_mut(parent_id) {
                    parent.children.push(tree);
                } else {
                    trees.push(tree);
                }
            } else {
                trees.push(tree);
            }
        }

        // Sort by folder type and name
        trees.sort_by(|a, b| {
            let type_order = a.folder.folder_type.sort_order()
                .cmp(&b.folder.folder_type.sort_order());
            if type_order == std::cmp::Ordering::Equal {
                a.folder.name.cmp(&b.folder.name)
            } else {
                type_order
            }
        });

        trees
    }

    /// Flatten tree for display
    pub fn flatten(&self, depth: usize) -> Vec<(usize, &Folder)> {
        let mut result = vec![(depth, &self.folder)];

        if self.expanded {
            for child in &self.children {
                result.extend(child.flatten(depth + 1));
            }
        }

        result
    }
}
