//! Web search provider

use crate::config::Config;
use crate::search::{SearchCategory, SearchResult, SearchResultKind};
use std::sync::Arc;
use tracing::debug;

/// Web search provider
pub struct WebSearcher {
    config: Arc<Config>,
}

impl WebSearcher {
    /// Create new web searcher
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// Search for web search patterns
    pub fn search(&self, query: &str) -> Option<SearchResult> {
        let query = query.trim();

        // Check for engine prefix (e.g., "g: query", "ddg: query")
        if let Some((prefix, search_query)) = self.parse_prefix(query) {
            return self.create_search_result(&prefix, search_query);
        }

        // Check if query looks like a URL
        if self.looks_like_url(query) {
            return Some(self.create_url_result(query));
        }

        None
    }

    /// Parse search prefix
    fn parse_prefix(&self, query: &str) -> Option<(String, String)> {
        // Try "prefix: query" format
        if let Some(pos) = query.find(':') {
            let prefix = query[..pos].trim().to_lowercase();
            let search_query = query[pos + 1..].trim();

            if !search_query.is_empty() && self.config.web_engines.contains_key(&prefix) {
                return Some((prefix, search_query.to_string()));
            }
        }

        // Try "prefix query" format (without colon)
        let parts: Vec<&str> = query.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let prefix = parts[0].to_lowercase();
            let search_query = parts[1].trim();

            if !search_query.is_empty() && self.config.web_engines.contains_key(&prefix) {
                return Some((prefix, search_query.to_string()));
            }
        }

        None
    }

    /// Create search result for a web engine
    fn create_search_result(&self, prefix: &str, query: String) -> Option<SearchResult> {
        let engine = self.config.web_engines.get(prefix)?;

        let url = engine.url.replace("{query}", &urlencoding::encode(&query));

        debug!("Web search: {} on {} -> {}", query, engine.name, url);

        Some(SearchResult {
            id: format!("web:{}:{}", prefix, query),
            title: format!("Search {} for \"{}\"", engine.name, query),
            subtitle: engine.url.split('?').next().unwrap_or(&engine.url).to_string(),
            icon: engine.icon.clone(),
            category: SearchCategory::WebSearch,
            kind: SearchResultKind::WebSearch {
                engine: engine.name.clone(),
                query,
                url,
            },
            score: 90,
            from_history: false,
        })
    }

    /// Check if query looks like a URL
    fn looks_like_url(&self, query: &str) -> bool {
        // Check for common URL patterns
        query.starts_with("http://")
            || query.starts_with("https://")
            || query.starts_with("www.")
            || (query.contains('.') && !query.contains(' ') && self.is_valid_domain(query))
    }

    /// Basic check if string could be a domain
    fn is_valid_domain(&self, s: &str) -> bool {
        let tlds = [
            ".com", ".org", ".net", ".io", ".dev", ".app", ".co", ".me",
            ".info", ".edu", ".gov", ".br", ".uk", ".de", ".fr", ".es",
        ];

        tlds.iter().any(|tld| s.ends_with(tld))
    }

    /// Create result for direct URL
    fn create_url_result(&self, query: &str) -> SearchResult {
        let url = if query.starts_with("http://") || query.starts_with("https://") {
            query.to_string()
        } else {
            format!("https://{}", query)
        };

        SearchResult {
            id: format!("url:{}", query),
            title: format!("Open {}", query),
            subtitle: "Open in browser".to_string(),
            icon: "web-browser-symbolic".to_string(),
            category: SearchCategory::WebSearch,
            kind: SearchResultKind::WebSearch {
                engine: "Direct".to_string(),
                query: query.to_string(),
                url,
            },
            score: 85,
            from_history: false,
        }
    }

    /// Get available search engines for display
    pub fn available_engines(&self) -> Vec<(&String, &str)> {
        self.config
            .web_engines
            .iter()
            .map(|(k, v)| (k, v.name.as_str()))
            .collect()
    }
}
