//! Launcher extension API
//!
//! Allows plugins to provide custom search results in the launcher.

use serde::{Deserialize, Serialize};
use std::any::Any;

/// Category of search result
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearchCategory {
    /// Applications
    Application,
    /// Files and folders
    File,
    /// Web search/URL
    Web,
    /// Calculator result
    Calculator,
    /// System command
    Command,
    /// Settings page
    Settings,
    /// Contact
    Contact,
    /// Calendar event
    Calendar,
    /// Recent item
    Recent,
    /// Custom category
    Custom(String),
}

impl Default for SearchCategory {
    fn default() -> Self {
        Self::Custom("Other".to_string())
    }
}

impl std::fmt::Display for SearchCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Application => write!(f, "Applications"),
            Self::File => write!(f, "Files"),
            Self::Web => write!(f, "Web"),
            Self::Calculator => write!(f, "Calculator"),
            Self::Command => write!(f, "Commands"),
            Self::Settings => write!(f, "Settings"),
            Self::Contact => write!(f, "Contacts"),
            Self::Calendar => write!(f, "Calendar"),
            Self::Recent => write!(f, "Recent"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// A search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique result ID (within provider)
    pub id: String,
    /// Display title
    pub title: String,
    /// Subtitle/description
    pub subtitle: Option<String>,
    /// Icon name or path
    pub icon: Option<String>,
    /// Category
    pub category: SearchCategory,
    /// Relevance score (0-100, higher is more relevant)
    pub score: u32,
    /// Keywords for additional matching
    pub keywords: Vec<String>,
    /// Whether this result can be copied to clipboard
    pub copyable: bool,
    /// Text to copy if copyable
    pub copy_text: Option<String>,
    /// Whether this result supports preview
    pub has_preview: bool,
    /// Actions available for this result
    pub actions: Vec<SearchAction>,
    /// Custom data
    #[serde(skip)]
    pub data: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for SearchResult {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            subtitle: None,
            icon: None,
            category: SearchCategory::default(),
            score: 50,
            keywords: Vec::new(),
            copyable: false,
            copy_text: None,
            has_preview: false,
            actions: Vec::new(),
            data: None,
        }
    }
}

impl SearchResult {
    /// Create a new search result
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            ..Default::default()
        }
    }

    /// Create an application result
    pub fn application(id: &str, name: &str, icon: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            title: name.to_string(),
            icon: icon.map(String::from),
            category: SearchCategory::Application,
            score: 80,
            ..Default::default()
        }
    }

    /// Create a file result
    pub fn file(path: &str, name: &str, icon: Option<&str>) -> Self {
        Self {
            id: path.to_string(),
            title: name.to_string(),
            subtitle: Some(path.to_string()),
            icon: icon.map(String::from),
            category: SearchCategory::File,
            score: 60,
            ..Default::default()
        }
    }

    /// Create a command result
    pub fn command(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            title: name.to_string(),
            subtitle: Some(description.to_string()),
            icon: Some("utilities-terminal".to_string()),
            category: SearchCategory::Command,
            score: 70,
            ..Default::default()
        }
    }

    /// Set subtitle
    pub fn with_subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    /// Set icon
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Set category
    pub fn with_category(mut self, category: SearchCategory) -> Self {
        self.category = category;
        self
    }

    /// Set score
    pub fn with_score(mut self, score: u32) -> Self {
        self.score = score.min(100);
        self
    }

    /// Add keywords
    pub fn with_keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Make copyable
    pub fn copyable(mut self, text: &str) -> Self {
        self.copyable = true;
        self.copy_text = Some(text.to_string());
        self
    }

    /// Enable preview
    pub fn with_preview(mut self) -> Self {
        self.has_preview = true;
        self
    }

    /// Add action
    pub fn with_action(mut self, action: SearchAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set custom data
    pub fn with_data<T: Any + Send + Sync>(mut self, data: T) -> Self {
        self.data = Some(Box::new(data));
        self
    }
}

/// An action that can be performed on a search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAction {
    /// Action ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Icon name
    pub icon: Option<String>,
    /// Keyboard shortcut hint
    pub shortcut: Option<String>,
    /// Whether this is the default action
    pub is_default: bool,
}

impl SearchAction {
    /// Create a new action
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            icon: None,
            shortcut: None,
            is_default: false,
        }
    }

    /// Create the default "open" action
    pub fn open() -> Self {
        Self {
            id: "open".to_string(),
            name: "Open".to_string(),
            icon: None,
            shortcut: Some("Enter".to_string()),
            is_default: true,
        }
    }

    /// Set icon
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Set shortcut
    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    /// Set as default
    pub fn default(mut self) -> Self {
        self.is_default = true;
        self
    }
}

/// Search context provided to providers
#[derive(Debug, Clone)]
pub struct SearchContext {
    /// The search query
    pub query: String,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Whether this is an incremental search
    pub incremental: bool,
    /// Previous query (for incremental search)
    pub previous_query: Option<String>,
    /// Categories to search (empty = all)
    pub categories: Vec<SearchCategory>,
}

impl Default for SearchContext {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_results: 10,
            incremental: false,
            previous_query: None,
            categories: Vec::new(),
        }
    }
}

impl SearchContext {
    /// Create a new search context
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            ..Default::default()
        }
    }

    /// Set max results
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Limit to specific categories
    pub fn categories(mut self, categories: &[SearchCategory]) -> Self {
        self.categories = categories.to_vec();
        self
    }
}

/// Activation result
#[derive(Debug, Clone)]
pub enum ActivationResult {
    /// Successfully activated
    Success,
    /// Close the launcher
    Close,
    /// Keep launcher open and update results
    Refresh,
    /// Show an error
    Error(String),
    /// Do nothing
    None,
}

/// Preview content for a search result
#[derive(Debug, Clone)]
pub enum PreviewContent {
    /// Simple text preview
    Text(String),
    /// Markdown content
    Markdown(String),
    /// HTML content
    Html(String),
    /// Image path
    Image(String),
    /// File preview (auto-detect type)
    File(String),
    /// Custom GTK widget (as serialized widget definition)
    Widget(String),
}

/// Trait for launcher search providers
pub trait LauncherProvider: Send + Sync {
    /// Get provider ID
    fn id(&self) -> &str;

    /// Get provider name
    fn name(&self) -> &str;

    /// Get supported categories
    fn categories(&self) -> Vec<SearchCategory>;

    /// Get provider priority (higher = searched first)
    fn priority(&self) -> i32 {
        0
    }

    /// Check if provider can handle query
    fn can_handle(&self, query: &str) -> bool {
        !query.is_empty()
    }

    /// Get prefix that triggers this provider (e.g., "!")
    fn trigger_prefix(&self) -> Option<&str> {
        None
    }

    /// Search for results
    fn search(&self, context: &SearchContext) -> Vec<SearchResult>;

    /// Activate a result (return true to close launcher)
    fn activate(&mut self, result_id: &str, action_id: Option<&str>) -> ActivationResult {
        let _ = (result_id, action_id);
        ActivationResult::None
    }

    /// Get preview content for a result
    fn preview(&self, result_id: &str) -> Option<PreviewContent> {
        let _ = result_id;
        None
    }

    /// Called when search is cancelled
    fn cancel(&mut self) {}

    /// Get recent/suggested items (shown when query is empty)
    fn suggestions(&self) -> Vec<SearchResult> {
        Vec::new()
    }
}

/// Helper for fuzzy matching
pub fn fuzzy_match(query: &str, text: &str) -> Option<u32> {
    let query = query.to_lowercase();
    let text = text.to_lowercase();

    // Exact match = highest score
    if text == query {
        return Some(100);
    }

    // Starts with = high score
    if text.starts_with(&query) {
        return Some(90);
    }

    // Contains = medium score
    if text.contains(&query) {
        return Some(70);
    }

    // Fuzzy match (all query chars appear in order)
    let mut query_chars = query.chars().peekable();
    let mut score = 0u32;
    let mut consecutive = 0;

    for c in text.chars() {
        if let Some(&qc) = query_chars.peek() {
            if c == qc {
                query_chars.next();
                consecutive += 1;
                score += 10 + consecutive * 5;
            } else {
                consecutive = 0;
            }
        } else {
            break;
        }
    }

    if query_chars.peek().is_none() {
        // All query chars matched
        Some(score.min(60))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact() {
        assert_eq!(fuzzy_match("test", "test"), Some(100));
    }

    #[test]
    fn test_fuzzy_match_prefix() {
        assert_eq!(fuzzy_match("tes", "testing"), Some(90));
    }

    #[test]
    fn test_fuzzy_match_contains() {
        assert_eq!(fuzzy_match("est", "testing"), Some(70));
    }

    #[test]
    fn test_fuzzy_match_fuzzy() {
        assert!(fuzzy_match("tst", "testing").is_some());
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        assert!(fuzzy_match("xyz", "testing").is_none());
    }

    #[test]
    fn test_search_result_builder() {
        let result = SearchResult::new("test", "Test")
            .with_subtitle("A test result")
            .with_icon("test-icon")
            .with_score(75)
            .copyable("copied text");

        assert_eq!(result.title, "Test");
        assert_eq!(result.subtitle, Some("A test result".to_string()));
        assert!(result.copyable);
    }
}
