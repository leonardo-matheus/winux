//! Syntax highlighting management for Winux Edit
//!
//! Handles language detection, syntax highlighting configuration,
//! and style scheme management using GtkSourceView.

use sourceview5::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info};

/// Manages syntax highlighting for the editor
pub struct SyntaxManager {
    language_manager: sourceview5::LanguageManager,
    scheme_manager: sourceview5::StyleSchemeManager,
    extension_map: HashMap<String, String>,
}

impl SyntaxManager {
    pub fn new() -> Self {
        let language_manager = sourceview5::LanguageManager::default();
        let scheme_manager = sourceview5::StyleSchemeManager::default();

        // Build extension to language ID mapping
        let mut extension_map = HashMap::new();

        // Programming languages
        extension_map.insert("rs".to_string(), "rust".to_string());
        extension_map.insert("py".to_string(), "python3".to_string());
        extension_map.insert("pyw".to_string(), "python3".to_string());
        extension_map.insert("js".to_string(), "javascript".to_string());
        extension_map.insert("mjs".to_string(), "javascript".to_string());
        extension_map.insert("ts".to_string(), "typescript".to_string());
        extension_map.insert("tsx".to_string(), "typescript".to_string());
        extension_map.insert("jsx".to_string(), "javascript".to_string());
        extension_map.insert("c".to_string(), "c".to_string());
        extension_map.insert("h".to_string(), "c".to_string());
        extension_map.insert("cpp".to_string(), "cpp".to_string());
        extension_map.insert("cxx".to_string(), "cpp".to_string());
        extension_map.insert("cc".to_string(), "cpp".to_string());
        extension_map.insert("hpp".to_string(), "cpp".to_string());
        extension_map.insert("hxx".to_string(), "cpp".to_string());
        extension_map.insert("java".to_string(), "java".to_string());
        extension_map.insert("kt".to_string(), "kotlin".to_string());
        extension_map.insert("kts".to_string(), "kotlin".to_string());
        extension_map.insert("go".to_string(), "go".to_string());
        extension_map.insert("rb".to_string(), "ruby".to_string());
        extension_map.insert("php".to_string(), "php".to_string());
        extension_map.insert("swift".to_string(), "swift".to_string());
        extension_map.insert("cs".to_string(), "c-sharp".to_string());
        extension_map.insert("fs".to_string(), "fsharp".to_string());
        extension_map.insert("scala".to_string(), "scala".to_string());
        extension_map.insert("hs".to_string(), "haskell".to_string());
        extension_map.insert("lhs".to_string(), "haskell".to_string());
        extension_map.insert("lua".to_string(), "lua".to_string());
        extension_map.insert("r".to_string(), "r".to_string());
        extension_map.insert("jl".to_string(), "julia".to_string());
        extension_map.insert("pl".to_string(), "perl".to_string());
        extension_map.insert("pm".to_string(), "perl".to_string());

        // Web technologies
        extension_map.insert("html".to_string(), "html".to_string());
        extension_map.insert("htm".to_string(), "html".to_string());
        extension_map.insert("xhtml".to_string(), "html".to_string());
        extension_map.insert("css".to_string(), "css".to_string());
        extension_map.insert("scss".to_string(), "scss".to_string());
        extension_map.insert("sass".to_string(), "scss".to_string());
        extension_map.insert("less".to_string(), "less".to_string());
        extension_map.insert("vue".to_string(), "html".to_string());
        extension_map.insert("svelte".to_string(), "html".to_string());

        // Data formats
        extension_map.insert("json".to_string(), "json".to_string());
        extension_map.insert("xml".to_string(), "xml".to_string());
        extension_map.insert("yaml".to_string(), "yaml".to_string());
        extension_map.insert("yml".to_string(), "yaml".to_string());
        extension_map.insert("toml".to_string(), "toml".to_string());
        extension_map.insert("ini".to_string(), "ini".to_string());
        extension_map.insert("conf".to_string(), "ini".to_string());
        extension_map.insert("cfg".to_string(), "ini".to_string());
        extension_map.insert("csv".to_string(), "csv".to_string());

        // Shell and scripts
        extension_map.insert("sh".to_string(), "sh".to_string());
        extension_map.insert("bash".to_string(), "sh".to_string());
        extension_map.insert("zsh".to_string(), "sh".to_string());
        extension_map.insert("fish".to_string(), "sh".to_string());
        extension_map.insert("ps1".to_string(), "powershell".to_string());
        extension_map.insert("bat".to_string(), "dosbatch".to_string());
        extension_map.insert("cmd".to_string(), "dosbatch".to_string());

        // Database
        extension_map.insert("sql".to_string(), "sql".to_string());

        // Markup and documentation
        extension_map.insert("md".to_string(), "markdown".to_string());
        extension_map.insert("markdown".to_string(), "markdown".to_string());
        extension_map.insert("rst".to_string(), "rst".to_string());
        extension_map.insert("tex".to_string(), "latex".to_string());
        extension_map.insert("latex".to_string(), "latex".to_string());

        // Build and config files
        extension_map.insert("makefile".to_string(), "makefile".to_string());
        extension_map.insert("cmake".to_string(), "cmake".to_string());
        extension_map.insert("gradle".to_string(), "groovy".to_string());

        // Misc
        extension_map.insert("diff".to_string(), "diff".to_string());
        extension_map.insert("patch".to_string(), "diff".to_string());
        extension_map.insert("dockerfile".to_string(), "dockerfile".to_string());
        extension_map.insert("proto".to_string(), "protobuf".to_string());

        Self {
            language_manager,
            scheme_manager,
            extension_map,
        }
    }

    /// Get language by extension
    pub fn language_for_extension(&self, extension: &str) -> Option<sourceview5::Language> {
        let ext_lower = extension.to_lowercase();

        // First try our mapping
        if let Some(lang_id) = self.extension_map.get(&ext_lower) {
            if let Some(lang) = self.language_manager.language(lang_id) {
                return Some(lang);
            }
        }

        // Fall back to LanguageManager guess
        self.language_manager.guess_language(
            Some(&format!("file.{}", extension)),
            None,
        )
    }

    /// Get language by ID
    pub fn language_by_id(&self, id: &str) -> Option<sourceview5::Language> {
        self.language_manager.language(id)
    }

    /// Get all available languages
    pub fn available_languages(&self) -> Vec<LanguageInfo> {
        let ids = self.language_manager.language_ids();
        let mut languages: Vec<LanguageInfo> = ids
            .iter()
            .filter_map(|id| {
                self.language_manager.language(id).map(|lang| LanguageInfo {
                    id: id.to_string(),
                    name: lang.name().to_string(),
                    section: lang.section().map(|s| s.to_string()),
                })
            })
            .collect();

        languages.sort_by(|a, b| a.name.cmp(&b.name));
        languages
    }

    /// Get style scheme by ID
    pub fn scheme(&self, id: &str) -> Option<sourceview5::StyleScheme> {
        self.scheme_manager.scheme(id)
    }

    /// Get all available style schemes
    pub fn available_schemes(&self) -> Vec<SchemeInfo> {
        let ids = self.scheme_manager.scheme_ids();
        let mut schemes: Vec<SchemeInfo> = ids
            .iter()
            .filter_map(|id| {
                self.scheme_manager.scheme(id).map(|scheme| SchemeInfo {
                    id: id.to_string(),
                    name: scheme.name().to_string(),
                    description: scheme.description().map(|s| s.to_string()),
                })
            })
            .collect();

        schemes.sort_by(|a, b| a.name.cmp(&b.name));
        schemes
    }

    /// Get recommended dark schemes
    pub fn dark_schemes(&self) -> Vec<String> {
        vec![
            "Adwaita-dark".to_string(),
            "dracula".to_string(),
            "monokai".to_string(),
            "solarized-dark".to_string(),
            "cobalt".to_string(),
            "oblivion".to_string(),
        ]
    }

    /// Get recommended light schemes
    pub fn light_schemes(&self) -> Vec<String> {
        vec![
            "Adwaita".to_string(),
            "classic".to_string(),
            "solarized-light".to_string(),
            "kate".to_string(),
            "tango".to_string(),
        ]
    }

    /// Detect language from filename
    pub fn detect_language(&self, filename: &str) -> Option<sourceview5::Language> {
        self.language_manager.guess_language(Some(filename), None)
    }

    /// Detect language from MIME type
    pub fn language_for_mime(&self, mime_type: &str) -> Option<sourceview5::Language> {
        self.language_manager.guess_language(None, Some(mime_type))
    }
}

impl Default for SyntaxManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a language
#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub id: String,
    pub name: String,
    pub section: Option<String>,
}

/// Information about a style scheme
#[derive(Debug, Clone)]
pub struct SchemeInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_mapping() {
        let manager = SyntaxManager::new();

        // Test some common extensions
        assert!(manager.extension_map.contains_key("rs"));
        assert!(manager.extension_map.contains_key("py"));
        assert!(manager.extension_map.contains_key("js"));
    }
}
