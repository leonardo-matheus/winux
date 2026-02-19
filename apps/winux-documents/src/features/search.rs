//! Document search functionality

use crate::viewer::SearchResult;
use crate::window::AppState;
use std::cell::RefCell;
use std::rc::Rc;

/// Search state
#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub current_result_index: Option<usize>,
    pub case_sensitive: bool,
    pub whole_word: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            current_result_index: None,
            case_sensitive: false,
            whole_word: false,
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.current_result_index = None;
    }

    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    pub fn current_result(&self) -> Option<&SearchResult> {
        self.current_result_index
            .and_then(|idx| self.results.get(idx))
    }

    pub fn next_result(&mut self) -> Option<&SearchResult> {
        if self.results.is_empty() {
            return None;
        }

        let new_index = match self.current_result_index {
            Some(idx) => (idx + 1) % self.results.len(),
            None => 0,
        };
        self.current_result_index = Some(new_index);
        self.results.get(new_index)
    }

    pub fn prev_result(&mut self) -> Option<&SearchResult> {
        if self.results.is_empty() {
            return None;
        }

        let new_index = match self.current_result_index {
            Some(idx) => {
                if idx == 0 {
                    self.results.len() - 1
                } else {
                    idx - 1
                }
            }
            None => self.results.len() - 1,
        };
        self.current_result_index = Some(new_index);
        self.results.get(new_index)
    }

    pub fn go_to_result(&mut self, index: usize) -> Option<&SearchResult> {
        if index < self.results.len() {
            self.current_result_index = Some(index);
            self.results.get(index)
        } else {
            None
        }
    }

    pub fn results_on_page(&self, page: usize) -> Vec<&SearchResult> {
        self.results.iter().filter(|r| r.page == page).collect()
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

/// Perform search in the document
pub fn search_document(query: &str, state: Rc<RefCell<AppState>>) {
    let mut app_state = state.borrow_mut();

    if query.is_empty() {
        app_state.search_state.clear();
        return;
    }

    app_state.search_state.query = query.to_string();

    if let Some(ref document) = app_state.document {
        let results = document.search(query);
        app_state.search_state.results = results;
        app_state.search_state.current_result_index = if app_state.search_state.results.is_empty() {
            None
        } else {
            Some(0)
        };
    }
}

/// Navigate to next search result and return the page number
pub fn next_search_result(state: Rc<RefCell<AppState>>) -> Option<usize> {
    let mut app_state = state.borrow_mut();
    app_state.search_state.next_result().map(|r| r.page)
}

/// Navigate to previous search result and return the page number
pub fn prev_search_result(state: Rc<RefCell<AppState>>) -> Option<usize> {
    let mut app_state = state.borrow_mut();
    app_state.search_state.prev_result().map(|r| r.page)
}

/// Highlight search results on a Cairo context
pub fn render_search_highlights(
    context: &cairo::Context,
    search_state: &SearchState,
    page: usize,
    scale: f64,
    current_highlight: bool,
) {
    let results = search_state.results_on_page(page);

    for (i, result) in results.iter().enumerate() {
        let is_current = search_state.current_result_index
            .map(|idx| {
                search_state.results.get(idx)
                    .map(|r| r.page == page && std::ptr::eq(*result, r))
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        // Different colors for current vs other results
        if is_current && current_highlight {
            context.set_source_rgba(1.0, 0.6, 0.0, 0.5); // Orange for current
        } else {
            context.set_source_rgba(1.0, 1.0, 0.0, 0.4); // Yellow for others
        }

        for (x, y, width, height) in &result.rects {
            context.rectangle(
                x * scale,
                y * scale,
                width * scale,
                height * scale,
            );
            let _ = context.fill();
        }
    }
}

/// Find and replace functionality (for supported formats)
#[derive(Debug, Clone)]
pub struct FindReplace {
    pub find_text: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
}

impl FindReplace {
    pub fn new() -> Self {
        Self {
            find_text: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
        }
    }

    /// Build a regex pattern from the search options
    pub fn build_pattern(&self) -> Result<regex::Regex, regex::Error> {
        let pattern = if self.use_regex {
            self.find_text.clone()
        } else {
            regex::escape(&self.find_text)
        };

        let pattern = if self.whole_word {
            format!(r"\b{}\b", pattern)
        } else {
            pattern
        };

        if self.case_sensitive {
            regex::Regex::new(&pattern)
        } else {
            regex::RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
        }
    }
}

impl Default for FindReplace {
    fn default() -> Self {
        Self::new()
    }
}
