//! Terminal widget using VTE

use std::path::PathBuf;

use gtk4::prelude::*;
use tracing::{debug, info};
use vte4::prelude::*;

use crate::config::Config;
use crate::themes::Theme;

/// Terminal widget wrapper
#[derive(Clone)]
pub struct TerminalWidget {
    /// VTE terminal widget
    terminal: vte4::Terminal,
    /// Terminal ID
    id: u32,
    /// Working directory
    working_dir: PathBuf,
    /// Process ID of the shell
    child_pid: Option<i32>,
}

impl TerminalWidget {
    /// Create a new terminal widget
    pub fn new(id: u32, config: &Config, theme: &Theme, working_dir: Option<PathBuf>) -> Self {
        let terminal = vte4::Terminal::new();

        // Configure terminal
        terminal.set_cursor_blink_mode(if config.cursor_blink {
            vte4::CursorBlinkMode::On
        } else {
            vte4::CursorBlinkMode::Off
        });

        terminal.set_cursor_shape(match config.cursor_shape.as_str() {
            "block" => vte4::CursorShape::Block,
            "ibeam" => vte4::CursorShape::Ibeam,
            "underline" => vte4::CursorShape::Underline,
            _ => vte4::CursorShape::Block,
        });

        terminal.set_scrollback_lines(config.scrollback_lines as i64);
        terminal.set_scroll_on_output(config.scroll_on_output);
        terminal.set_scroll_on_keystroke(config.scroll_on_keystroke);
        terminal.set_audible_bell(config.audible_bell);
        terminal.set_allow_hyperlink(true);
        terminal.set_mouse_autohide(true);

        // Set font
        let font_desc = gtk4::pango::FontDescription::from_string(&format!(
            "{} {}",
            config.font_family, config.font_size
        ));
        terminal.set_font(Some(&font_desc));

        // Apply theme colors
        Self::apply_theme(&terminal, theme);

        // Determine working directory
        let working_dir = working_dir
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

        let mut widget = TerminalWidget {
            terminal,
            id,
            working_dir,
            child_pid: None,
        };

        // Spawn shell
        widget.spawn_shell();

        widget
    }

    /// Apply theme colors to terminal
    fn apply_theme(terminal: &vte4::Terminal, theme: &Theme) {
        // Parse colors
        let bg = parse_color(&theme.background);
        let fg = parse_color(&theme.foreground);

        terminal.set_color_background(&bg);
        terminal.set_color_foreground(&fg);

        // Set palette colors
        let palette: Vec<gtk4::gdk::RGBA> = theme
            .palette
            .iter()
            .map(|c| parse_color(c))
            .collect();

        terminal.set_colors(Some(&fg), Some(&bg), &palette);

        // Set cursor and selection colors
        if let Some(cursor) = &theme.cursor {
            terminal.set_color_cursor(Some(&parse_color(cursor)));
        }

        if let Some(selection) = &theme.selection {
            terminal.set_color_highlight(Some(&parse_color(selection)));
        }
    }

    /// Spawn the shell process
    fn spawn_shell(&mut self) {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

        debug!("Spawning shell: {} in {:?}", shell, self.working_dir);

        // Use the async spawn method
        self.terminal.spawn_async(
            vte4::PtyFlags::DEFAULT,
            Some(&self.working_dir.to_string_lossy()),
            &[&shell],
            &[],
            gtk4::glib::SpawnFlags::DEFAULT,
            || {},
            -1,
            None::<&gtk4::gio::Cancellable>,
            |result| {
                match result {
                    Ok(pid) => {
                        info!("Shell spawned with PID: {}", pid);
                    }
                    Err(e) => {
                        tracing::error!("Failed to spawn shell: {}", e);
                    }
                }
            },
        );
    }

    /// Get the terminal widget
    pub fn widget(&self) -> &vte4::Terminal {
        &self.terminal
    }

    /// Get terminal ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get current working directory
    pub fn current_directory(&self) -> Option<PathBuf> {
        self.terminal
            .current_directory_uri()
            .and_then(|uri| {
                gtk4::glib::filename_from_uri(&uri)
                    .ok()
                    .map(|(path, _)| path)
            })
    }

    /// Get window title
    pub fn title(&self) -> String {
        self.terminal
            .window_title()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Terminal".to_string())
    }

    /// Copy selection to clipboard
    pub fn copy_selection(&self) {
        self.terminal.copy_clipboard_format(vte4::Format::Text);
    }

    /// Paste from clipboard
    pub fn paste(&self) {
        self.terminal.paste_clipboard();
    }

    /// Select all text
    pub fn select_all(&self) {
        self.terminal.select_all();
    }

    /// Clear scrollback
    pub fn clear_scrollback(&self) {
        self.terminal.reset(true, true);
    }

    /// Search text
    pub fn search(&self, pattern: &str, case_sensitive: bool) {
        let regex = vte4::Regex::for_search(
            pattern,
            if case_sensitive { 0 } else { 1 }, // PCRE2_CASELESS = 1
        )
        .ok();

        if let Some(regex) = regex {
            self.terminal.search_set_regex(Some(&regex), 0);
            self.terminal.search_find_next();
        }
    }

    /// Search next match
    pub fn search_next(&self) {
        self.terminal.search_find_next();
    }

    /// Search previous match
    pub fn search_previous(&self) {
        self.terminal.search_find_previous();
    }

    /// Increase font size
    pub fn zoom_in(&self) {
        let scale = self.terminal.font_scale();
        self.terminal.set_font_scale(scale * 1.1);
    }

    /// Decrease font size
    pub fn zoom_out(&self) {
        let scale = self.terminal.font_scale();
        self.terminal.set_font_scale(scale / 1.1);
    }

    /// Reset font size
    pub fn zoom_reset(&self) {
        self.terminal.set_font_scale(1.0);
    }

    /// Feed text to terminal
    pub fn feed(&self, text: &str) {
        self.terminal.feed(text.as_bytes());
    }

    /// Feed child (send to shell)
    pub fn feed_child(&self, text: &str) {
        self.terminal.feed_child(text.as_bytes());
    }

    /// Check if has selection
    pub fn has_selection(&self) -> bool {
        self.terminal.has_selection()
    }

    /// Get selected text
    pub fn get_selection(&self) -> Option<String> {
        if self.has_selection() {
            // VTE doesn't have a direct method to get selection text
            // We'd need to copy to clipboard and read from there
            None
        } else {
            None
        }
    }

    /// Connect to child exited signal
    pub fn connect_child_exited<F: Fn(i32) + 'static>(&self, callback: F) {
        self.terminal.connect_child_exited(move |_, status| {
            callback(status);
        });
    }

    /// Connect to title changed signal
    pub fn connect_title_changed<F: Fn(&str) + 'static>(&self, callback: F) {
        self.terminal.connect_window_title_changed(move |term| {
            let title = term
                .window_title()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Terminal".to_string());
            callback(&title);
        });
    }

    /// Connect to bell signal
    pub fn connect_bell<F: Fn() + 'static>(&self, callback: F) {
        self.terminal.connect_bell(move |_| {
            callback();
        });
    }
}

/// Parse a hex color string to RGBA
fn parse_color(color: &str) -> gtk4::gdk::RGBA {
    gtk4::gdk::RGBA::parse(color).unwrap_or_else(|_| gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        let black = parse_color("#000000");
        assert_eq!(black.red(), 0.0);
        assert_eq!(black.green(), 0.0);
        assert_eq!(black.blue(), 0.0);

        let white = parse_color("#ffffff");
        assert_eq!(white.red(), 1.0);
        assert_eq!(white.green(), 1.0);
        assert_eq!(white.blue(), 1.0);
    }
}
