//! Search module - handles all search providers

pub mod apps;
pub mod calculator;
pub mod commands;
pub mod files;
pub mod plugins;
pub mod web;

use crate::config::Config;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

pub use apps::AppSearcher;
pub use calculator::Calculator;
pub use commands::CommandSearcher;
pub use files::FileSearcher;
pub use plugins::PluginManager;
pub use web::WebSearcher;

/// Search result representation
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Unique identifier
    pub id: String,

    /// Display title
    pub title: String,

    /// Subtitle/description
    pub subtitle: String,

    /// Icon name or path
    pub icon: String,

    /// Category for grouping
    pub category: SearchCategory,

    /// Result kind with associated data
    pub kind: SearchResultKind,

    /// Relevance score (0-100)
    pub score: u32,

    /// Whether this is from history
    pub from_history: bool,
}

/// Search result categories
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SearchCategory {
    TopHit,
    Applications,
    Files,
    Calculator,
    Conversion,
    WebSearch,
    Commands,
    Plugins,
    History,
}

impl SearchCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::TopHit => "Top Hit",
            Self::Applications => "Applications",
            Self::Files => "Files",
            Self::Calculator => "Calculator",
            Self::Conversion => "Conversion",
            Self::WebSearch => "Web Search",
            Self::Commands => "Commands",
            Self::Plugins => "Plugins",
            Self::History => "Recent",
        }
    }

    pub fn priority(&self) -> u32 {
        match self {
            Self::TopHit => 0,
            Self::Calculator => 1,
            Self::Conversion => 2,
            Self::Applications => 3,
            Self::Commands => 4,
            Self::Files => 5,
            Self::WebSearch => 6,
            Self::Plugins => 7,
            Self::History => 8,
        }
    }
}

/// Search result kinds with associated data
#[derive(Debug, Clone)]
pub enum SearchResultKind {
    Application {
        desktop_file: PathBuf,
        exec: String,
        categories: Vec<String>,
    },
    File {
        path: PathBuf,
    },
    Calculator {
        expression: String,
        result: String,
    },
    Conversion {
        from_value: String,
        from_unit: String,
        to_value: String,
        to_unit: String,
        result: String,
    },
    WebSearch {
        engine: String,
        query: String,
        url: String,
    },
    Command {
        command: String,
    },
    Plugin {
        plugin_id: String,
        action: String,
    },
}

/// Main search engine that coordinates all providers
pub struct SearchEngine {
    config: Arc<Config>,
    app_searcher: AppSearcher,
    file_searcher: FileSearcher,
    calculator: Calculator,
    web_searcher: WebSearcher,
    command_searcher: CommandSearcher,
    plugin_manager: PluginManager,
}

impl SearchEngine {
    /// Create new search engine
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            app_searcher: AppSearcher::new(config.clone()),
            file_searcher: FileSearcher::new(config.clone()),
            calculator: Calculator::new(),
            web_searcher: WebSearcher::new(config.clone()),
            command_searcher: CommandSearcher::new(),
            plugin_manager: PluginManager::new(config.clone()),
            config,
        }
    }

    /// Perform search across all providers
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query = query.trim();

        if query.len() < self.config.search.min_query_length {
            return vec![];
        }

        let mut results = Vec::new();

        // Calculator (highest priority for math expressions)
        if self.config.search.calculator_enabled {
            if let Some(calc_result) = self.calculator.evaluate(query) {
                results.push(calc_result);
            }

            // Check for unit conversions
            if let Some(conversion) = self.calculator.convert(query) {
                results.push(conversion);
            }
        }

        // Web search (check for prefix)
        if self.config.search.web_enabled {
            if let Some(web_result) = self.web_searcher.search(query) {
                results.push(web_result);
            }
        }

        // System commands
        if self.config.search.commands_enabled {
            results.extend(self.command_searcher.search(query));
        }

        // Applications
        if self.config.search.apps_enabled {
            results.extend(self.app_searcher.search(query));
        }

        // Files
        if self.config.search.files_enabled {
            results.extend(self.file_searcher.search(query));
        }

        // Plugins
        if self.config.search.plugins_enabled {
            results.extend(self.plugin_manager.search(query));
        }

        // Sort by category priority, then by score
        results.sort_by(|a, b| {
            let cat_cmp = a.category.priority().cmp(&b.category.priority());
            if cat_cmp == std::cmp::Ordering::Equal {
                b.score.cmp(&a.score)
            } else {
                cat_cmp
            }
        });

        // Limit results
        results.truncate(self.config.general.max_results);

        // Mark top result
        if let Some(first) = results.first_mut() {
            if first.score > 80 {
                first.category = SearchCategory::TopHit;
            }
        }

        debug!("Search '{}' returned {} results", query, results.len());

        results
    }

    /// Refresh search indexes
    pub fn refresh(&mut self) {
        self.app_searcher.refresh();
        self.file_searcher.refresh();
        self.plugin_manager.refresh();
    }
}
