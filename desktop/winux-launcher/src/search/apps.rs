//! Application search provider

use crate::config::Config;
use crate::search::{SearchCategory, SearchResult, SearchResultKind};
use freedesktop_desktop_entry::{DesktopEntry, Iter};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, warn};

/// Application entry from .desktop file
#[derive(Debug, Clone)]
struct AppEntry {
    name: String,
    generic_name: Option<String>,
    comment: Option<String>,
    exec: String,
    icon: String,
    categories: Vec<String>,
    keywords: Vec<String>,
    desktop_file: PathBuf,
    no_display: bool,
}

/// Application search provider
pub struct AppSearcher {
    config: Arc<Config>,
    apps: Vec<AppEntry>,
    matcher: SkimMatcherV2,
}

impl AppSearcher {
    /// Create new application searcher
    pub fn new(config: Arc<Config>) -> Self {
        let mut searcher = Self {
            config,
            apps: Vec::new(),
            matcher: SkimMatcherV2::default(),
        };

        searcher.refresh();
        searcher
    }

    /// Refresh application index
    pub fn refresh(&mut self) {
        self.apps.clear();

        // Collect all .desktop files
        let search_paths = &self.config.search.app_search_paths;

        for path in search_paths {
            if !path.exists() {
                continue;
            }

            for entry in Iter::new(std::iter::once(path.clone())) {
                match self.parse_desktop_entry(&entry) {
                    Ok(app) => {
                        if !app.no_display {
                            self.apps.push(app);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to parse desktop entry {:?}: {}", entry, e);
                    }
                }
            }
        }

        debug!("Indexed {} applications", self.apps.len());
    }

    /// Parse a desktop entry file
    fn parse_desktop_entry(&self, path: &PathBuf) -> anyhow::Result<AppEntry> {
        let content = std::fs::read_to_string(path)?;
        let entry = DesktopEntry::decode(path, &content)?;

        let name = entry
            .name(None)
            .map(|s| s.to_string())
            .unwrap_or_else(|| path.file_stem().unwrap_or_default().to_string_lossy().to_string());

        let generic_name = entry.generic_name(None).map(|s| s.to_string());
        let comment = entry.comment(None).map(|s| s.to_string());

        let exec = entry
            .exec()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let icon = entry
            .icon()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application-x-executable".to_string());

        let categories = entry
            .categories()
            .map(|c| c.split(';').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let keywords = entry
            .keywords(None)
            .map(|k| k.split(';').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let no_display = entry.no_display();

        Ok(AppEntry {
            name,
            generic_name,
            comment,
            exec,
            icon,
            categories,
            keywords,
            desktop_file: path.clone(),
            no_display,
        })
    }

    /// Search applications
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(i64, SearchResult)> = Vec::new();

        for app in &self.apps {
            // Calculate match score
            let mut best_score: i64 = 0;

            // Match against name
            if let Some(score) = self.matcher.fuzzy_match(&app.name.to_lowercase(), &query_lower) {
                best_score = best_score.max(score);
            }

            // Match against generic name
            if let Some(ref generic) = app.generic_name {
                if let Some(score) = self.matcher.fuzzy_match(&generic.to_lowercase(), &query_lower) {
                    best_score = best_score.max(score);
                }
            }

            // Match against keywords
            for keyword in &app.keywords {
                if let Some(score) = self.matcher.fuzzy_match(&keyword.to_lowercase(), &query_lower) {
                    best_score = best_score.max(score);
                }
            }

            // Match against categories
            for category in &app.categories {
                if let Some(score) = self.matcher.fuzzy_match(&category.to_lowercase(), &query_lower) {
                    best_score = best_score.max(score / 2); // Lower weight for categories
                }
            }

            if best_score > 0 {
                let subtitle = app.comment.clone()
                    .or_else(|| app.generic_name.clone())
                    .unwrap_or_else(|| format!("Application"));

                let result = SearchResult {
                    id: format!("app:{}", app.desktop_file.display()),
                    title: app.name.clone(),
                    subtitle,
                    icon: app.icon.clone(),
                    category: SearchCategory::Applications,
                    kind: SearchResultKind::Application {
                        desktop_file: app.desktop_file.clone(),
                        exec: app.exec.clone(),
                        categories: app.categories.clone(),
                    },
                    score: (best_score.min(100) as u32),
                    from_history: false,
                };

                results.push((best_score, result));
            }
        }

        // Sort by score (highest first) and take top results
        results.sort_by(|a, b| b.0.cmp(&a.0));
        results.into_iter().take(5).map(|(_, r)| r).collect()
    }
}
