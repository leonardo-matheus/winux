// Code Block - Syntax highlighted code display with copy button

use crate::integrations::ClipboardManager;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, ScrolledWindow, Align};
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View as SourceView};

#[derive(Clone)]
pub struct CodeBlock {
    pub widget: GtkBox,
}

impl CodeBlock {
    pub fn new(code: &str, language: &str) -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        widget.add_css_class("code-block");

        // Header with language label and copy button
        let header = GtkBox::new(Orientation::Horizontal, 8);
        header.add_css_class("code-block-header");

        // Language label
        let lang_label = Label::new(Some(if language.is_empty() { "code" } else { language }));
        lang_label.add_css_class("code-language");
        lang_label.set_hexpand(true);
        lang_label.set_xalign(0.0);
        header.append(&lang_label);

        // Copy button
        let copy_btn = Button::builder()
            .icon_name("edit-copy-symbolic")
            .tooltip_text("Copy code")
            .build();
        copy_btn.add_css_class("code-copy-button");
        copy_btn.add_css_class("flat");

        let code_clone = code.to_string();
        let copy_btn_clone = copy_btn.clone();
        copy_btn.connect_clicked(move |_| {
            if ClipboardManager::set_text(&code_clone).is_ok() {
                copy_btn_clone.set_icon_name("emblem-ok-symbolic");
                copy_btn_clone.set_tooltip_text(Some("Copied!"));

                // Reset after 2 seconds
                let btn = copy_btn_clone.clone();
                glib::timeout_add_seconds_local_once(2, move || {
                    btn.set_icon_name("edit-copy-symbolic");
                    btn.set_tooltip_text(Some("Copy code"));
                });
            }
        });
        header.append(&copy_btn);

        widget.append(&header);

        // Code content with syntax highlighting
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .max_content_height(400)
            .build();

        let buffer = Buffer::new(None);
        buffer.set_text(code);

        // Set language for syntax highlighting
        if !language.is_empty() {
            let lang_manager = LanguageManager::default();
            if let Some(lang) = lang_manager.language(language)
                .or_else(|| Self::guess_language(&lang_manager, language))
            {
                buffer.set_language(Some(&lang));
            }
        }

        // Set color scheme
        let scheme_manager = StyleSchemeManager::default();
        if let Some(scheme) = scheme_manager.scheme("Adwaita-dark")
            .or_else(|| scheme_manager.scheme("classic-dark"))
        {
            buffer.set_style_scheme(Some(&scheme));
        }

        let view = SourceView::with_buffer(&buffer);
        view.set_editable(false);
        view.set_cursor_visible(false);
        view.set_show_line_numbers(true);
        view.set_monospace(true);
        view.set_wrap_mode(gtk4::WrapMode::None);
        view.add_css_class("code-content");

        scroll.set_child(Some(&view));
        widget.append(&scroll);

        Self { widget }
    }

    /// Try to guess the language from common aliases
    fn guess_language(manager: &LanguageManager, lang: &str) -> Option<sourceview5::Language> {
        let aliases = [
            ("js", "javascript"),
            ("ts", "typescript"),
            ("py", "python"),
            ("rb", "ruby"),
            ("rs", "rust"),
            ("sh", "sh"),
            ("bash", "sh"),
            ("zsh", "sh"),
            ("yml", "yaml"),
            ("md", "markdown"),
            ("jsx", "javascript"),
            ("tsx", "typescript"),
            ("cpp", "cpp"),
            ("c++", "cpp"),
            ("h", "c"),
            ("hpp", "cpp"),
            ("cs", "c-sharp"),
            ("kt", "kotlin"),
            ("swift", "swift"),
            ("go", "go"),
            ("java", "java"),
            ("php", "php"),
            ("html", "html"),
            ("css", "css"),
            ("scss", "scss"),
            ("sql", "sql"),
            ("json", "json"),
            ("xml", "xml"),
            ("toml", "toml"),
            ("ini", "ini"),
            ("dockerfile", "dockerfile"),
            ("docker", "dockerfile"),
            ("makefile", "makefile"),
            ("make", "makefile"),
        ];

        let lang_lower = lang.to_lowercase();

        for (alias, actual) in aliases {
            if lang_lower == alias {
                if let Some(l) = manager.language(actual) {
                    return Some(l);
                }
            }
        }

        // Try the original language name
        manager.language(&lang_lower)
    }
}

/// Create a simple code block without syntax highlighting (for inline use)
pub fn create_inline_code(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("monospace");
    label.set_selectable(true);
    label
}
