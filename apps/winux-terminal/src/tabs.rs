//! Tab management for terminal

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use gtk4::prelude::*;
use libadwaita as adw;
use tracing::debug;

use crate::config::Config;
use crate::terminal::TerminalWidget;
use crate::themes::Theme;

/// Global tab counter for unique IDs
static TAB_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Tab information
pub struct Tab {
    /// Tab ID
    pub id: u32,
    /// Terminal widget
    pub terminal: TerminalWidget,
    /// Tab page reference
    pub page: adw::TabPage,
}

/// Tab manager
#[derive(Clone)]
pub struct TabManager {
    inner: Rc<TabManagerInner>,
}

struct TabManagerInner {
    /// Tab view widget
    tab_view: adw::TabView,
    /// Tab bar widget
    tab_bar: adw::TabBar,
    /// Container widget
    container: gtk4::Box,
    /// Configuration
    config: Config,
    /// Theme
    theme: Theme,
    /// Active tabs
    tabs: RefCell<Vec<Tab>>,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new(config: Config, theme: Theme) -> Self {
        let tab_view = adw::TabView::new();
        let tab_bar = adw::TabBar::new();
        tab_bar.set_view(Some(&tab_view));
        tab_bar.set_autohide(false);
        tab_bar.set_expand_tabs(true);

        // Create container
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.append(&tab_bar);
        container.append(&tab_view);

        // Connect tab close signal
        tab_view.connect_close_page(|view, page| {
            // Allow closing if more than one tab
            if view.n_pages() > 1 {
                return false; // Allow close
            }
            true // Prevent close (keep at least one tab)
        });

        TabManager {
            inner: Rc::new(TabManagerInner {
                tab_view,
                tab_bar,
                container,
                config,
                theme,
                tabs: RefCell::new(Vec::new()),
            }),
        }
    }

    /// Get the main widget
    pub fn widget(&self) -> &gtk4::Box {
        &self.inner.container
    }

    /// Add a new tab
    pub fn add_tab(&self, working_dir: Option<PathBuf>) {
        let id = TAB_COUNTER.fetch_add(1, Ordering::SeqCst);

        debug!("Creating new tab with ID: {}", id);

        // Create terminal
        let terminal = TerminalWidget::new(
            id,
            &self.inner.config,
            &self.inner.theme,
            working_dir,
        );

        // Create scrolled window for terminal
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);
        scrolled.set_child(Some(terminal.widget()));

        // Add to tab view
        let page = self.inner.tab_view.append(&scrolled);
        page.set_title(&format!("Terminal {}", id + 1));
        page.set_icon(Some(&gtk4::gio::ThemedIcon::new("utilities-terminal-symbolic")));

        // Connect title change
        let page_clone = page.clone();
        terminal.connect_title_changed(move |title| {
            page_clone.set_title(title);
        });

        // Connect child exited
        let tab_view = self.inner.tab_view.clone();
        let page_for_exit = page.clone();
        terminal.connect_child_exited(move |_status| {
            // Close tab when shell exits
            tab_view.close_page(&page_for_exit);
        });

        // Store tab
        let tab = Tab {
            id,
            terminal,
            page: page.clone(),
        };
        self.inner.tabs.borrow_mut().push(tab);

        // Select the new tab
        self.inner.tab_view.set_selected_page(&page);
    }

    /// Close current tab
    pub fn close_current_tab(&self) {
        if let Some(page) = self.inner.tab_view.selected_page() {
            if self.inner.tab_view.n_pages() > 1 {
                self.inner.tab_view.close_page(&page);

                // Remove from our tracking
                let mut tabs = self.inner.tabs.borrow_mut();
                tabs.retain(|t| t.page != page);
            }
        }
    }

    /// Get current terminal
    pub fn current_terminal(&self) -> Option<TerminalWidget> {
        let page = self.inner.tab_view.selected_page()?;
        let tabs = self.inner.tabs.borrow();
        tabs.iter()
            .find(|t| t.page == page)
            .map(|t| t.terminal.clone())
    }

    /// Copy selection from current terminal
    pub fn copy_selection(&self) {
        if let Some(terminal) = self.current_terminal() {
            terminal.copy_selection();
        }
    }

    /// Paste to current terminal
    pub fn paste(&self) {
        if let Some(terminal) = self.current_terminal() {
            terminal.paste();
        }
    }

    /// Get number of tabs
    pub fn tab_count(&self) -> u32 {
        self.inner.tab_view.n_pages() as u32
    }

    /// Select tab by index
    pub fn select_tab(&self, index: u32) {
        if let Some(page) = self.inner.tab_view.nth_page(index as i32) {
            self.inner.tab_view.set_selected_page(&page);
        }
    }

    /// Select next tab
    pub fn next_tab(&self) {
        let current = self
            .inner
            .tab_view
            .selected_page()
            .map(|p| self.inner.tab_view.page_position(&p))
            .unwrap_or(0);

        let next = (current + 1) % self.inner.tab_view.n_pages();
        self.select_tab(next as u32);
    }

    /// Select previous tab
    pub fn prev_tab(&self) {
        let current = self
            .inner
            .tab_view
            .selected_page()
            .map(|p| self.inner.tab_view.page_position(&p))
            .unwrap_or(0);

        let n_pages = self.inner.tab_view.n_pages();
        let prev = if current == 0 {
            n_pages - 1
        } else {
            current - 1
        };
        self.select_tab(prev as u32);
    }

    /// Move current tab to the left
    pub fn move_tab_left(&self) {
        if let Some(page) = self.inner.tab_view.selected_page() {
            let pos = self.inner.tab_view.page_position(&page);
            if pos > 0 {
                self.inner.tab_view.reorder_page(&page, pos - 1);
            }
        }
    }

    /// Move current tab to the right
    pub fn move_tab_right(&self) {
        if let Some(page) = self.inner.tab_view.selected_page() {
            let pos = self.inner.tab_view.page_position(&page);
            if pos < self.inner.tab_view.n_pages() - 1 {
                self.inner.tab_view.reorder_page(&page, pos + 1);
            }
        }
    }

    /// Zoom in current terminal
    pub fn zoom_in(&self) {
        if let Some(terminal) = self.current_terminal() {
            terminal.zoom_in();
        }
    }

    /// Zoom out current terminal
    pub fn zoom_out(&self) {
        if let Some(terminal) = self.current_terminal() {
            terminal.zoom_out();
        }
    }

    /// Reset zoom
    pub fn zoom_reset(&self) {
        if let Some(terminal) = self.current_terminal() {
            terminal.zoom_reset();
        }
    }

    /// Search in current terminal
    pub fn search(&self, pattern: &str, case_sensitive: bool) {
        if let Some(terminal) = self.current_terminal() {
            terminal.search(pattern, case_sensitive);
        }
    }

    /// Find next search result
    pub fn search_next(&self) {
        if let Some(terminal) = self.current_terminal() {
            terminal.search_next();
        }
    }

    /// Find previous search result
    pub fn search_previous(&self) {
        if let Some(terminal) = self.current_terminal() {
            terminal.search_previous();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_counter() {
        let first = TAB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let second = TAB_COUNTER.fetch_add(1, Ordering::SeqCst);
        assert!(second > first);
    }
}
