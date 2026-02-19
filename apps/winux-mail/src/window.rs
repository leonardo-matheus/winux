// Winux Mail - Main Window
// Copyright (c) 2026 Winux OS Project

use crate::backend::account::AccountManager;
use crate::data::cache::EmailCache;
use crate::data::folder::Folder;
use crate::data::message::Message;
use crate::views::{accounts::AccountsView, compose::ComposeView, mailbox::MailboxView, message::MessageView};
use crate::ui::folder_row::FolderRow;

use gtk4::prelude::*;
use gtk4::{
    gio, glib, Application, Box as GtkBox, Button, HeaderBar, Label, ListBox,
    MenuButton, Orientation, Paned, PopoverMenu, ScrolledWindow, SearchBar,
    SearchEntry, Separator, Stack, StackSidebar, ToggleButton,
};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct MailWindow {
    pub window: adw::ApplicationWindow,
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub cache: Arc<EmailCache>,
    pub folder_list: ListBox,
    pub mailbox_view: Rc<RefCell<MailboxView>>,
    pub message_view: Rc<RefCell<MessageView>>,
    pub current_folder: Rc<RefCell<Option<Folder>>>,
    pub search_entry: SearchEntry,
    pub search_bar: SearchBar,
    pub unread_count: Rc<RefCell<u32>>,
}

impl MailWindow {
    pub fn new(app: &Application) -> Self {
        let cache = Arc::new(EmailCache::new().expect("Failed to initialize email cache"));
        let account_manager = Arc::new(RwLock::new(
            AccountManager::new(cache.clone()).expect("Failed to initialize account manager")
        ));

        // Create main window
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Mail")
            .default_width(1200)
            .default_height(800)
            .build();

        // Create header bar
        let header = HeaderBar::new();

        // Compose button
        let compose_btn = Button::builder()
            .icon_name("mail-message-new-symbolic")
            .tooltip_text("Compose")
            .build();

        // Search toggle
        let search_toggle = ToggleButton::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search")
            .build();

        // Menu button
        let menu_btn = MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menu")
            .build();

        let menu = gio::Menu::new();
        menu.append(Some("Refresh"), Some("win.refresh"));
        menu.append(Some("Mark All as Read"), Some("win.mark-all-read"));

        let accounts_section = gio::Menu::new();
        accounts_section.append(Some("Accounts"), Some("win.accounts"));
        accounts_section.append(Some("Preferences"), Some("win.preferences"));
        menu.append_section(None, &accounts_section);

        let help_section = gio::Menu::new();
        help_section.append(Some("Keyboard Shortcuts"), Some("win.shortcuts"));
        help_section.append(Some("About Mail"), Some("win.about"));
        menu.append_section(None, &help_section);

        menu_btn.set_menu_model(Some(&menu));

        header.pack_start(&compose_btn);
        header.pack_end(&menu_btn);
        header.pack_end(&search_toggle);

        // Search bar
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search emails...")
            .hexpand(true)
            .build();

        let search_bar = SearchBar::builder()
            .child(&search_entry)
            .build();

        search_toggle.bind_property("active", &search_bar, "search-mode-enabled")
            .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
            .build();

        // Main content - three pane layout
        let main_paned = Paned::builder()
            .orientation(Orientation::Horizontal)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .build();

        // Left pane - Folders
        let folders_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .width_request(200)
            .build();

        // Account selector
        let account_combo = gtk4::DropDown::builder()
            .tooltip_text("Select account")
            .build();

        let all_accounts_item = gtk4::StringList::new(&["All Accounts"]);
        account_combo.set_model(Some(&all_accounts_item));

        folders_box.append(&account_combo);
        folders_box.append(&Separator::new(Orientation::Horizontal));

        // Folder list
        let folder_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .build();

        let folder_list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(vec!["navigation-sidebar"])
            .build();

        // Add default folders
        Self::populate_default_folders(&folder_list);

        folder_scroll.set_child(Some(&folder_list));
        folders_box.append(&folder_scroll);

        // Add account button at bottom
        let add_account_btn = Button::builder()
            .label("Add Account")
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();
        folders_box.append(&add_account_btn);

        main_paned.set_start_child(Some(&folders_box));

        // Right pane - Message list and content
        let content_paned = Paned::builder()
            .orientation(Orientation::Horizontal)
            .position(350)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .build();

        // Message list (middle)
        let mailbox_view = MailboxView::new();
        content_paned.set_start_child(Some(&mailbox_view.container));

        // Message content (right)
        let message_view = MessageView::new();
        content_paned.set_end_child(Some(&message_view.container));

        main_paned.set_end_child(Some(&content_paned));
        main_paned.set_position(200);

        // Main layout
        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        main_box.append(&header);
        main_box.append(&search_bar);
        main_box.append(&main_paned);

        window.set_content(Some(&main_box));

        let mail_window = Self {
            window,
            account_manager,
            cache,
            folder_list,
            mailbox_view: Rc::new(RefCell::new(mailbox_view)),
            message_view: Rc::new(RefCell::new(message_view)),
            current_folder: Rc::new(RefCell::new(None)),
            search_entry,
            search_bar,
            unread_count: Rc::new(RefCell::new(0)),
        };

        mail_window.setup_actions();
        mail_window.setup_signals(compose_btn, add_account_btn);
        mail_window.load_emails();

        mail_window
    }

    fn populate_default_folders(folder_list: &ListBox) {
        let folders = vec![
            ("mail-inbox-symbolic", "Inbox", 0),
            ("mail-mark-important-symbolic", "Starred", 0),
            ("mail-send-symbolic", "Sent", 0),
            ("document-edit-symbolic", "Drafts", 0),
            ("mail-mark-junk-symbolic", "Spam", 0),
            ("user-trash-symbolic", "Trash", 0),
            ("folder-symbolic", "Archive", 0),
        ];

        for (icon, name, count) in folders {
            let row = FolderRow::new(icon, name, count);
            folder_list.append(&row.row);
        }
    }

    fn setup_actions(&self) {
        let window = &self.window;

        // Refresh action
        let refresh_action = gio::SimpleAction::new("refresh", None);
        let am = self.account_manager.clone();
        refresh_action.connect_activate(move |_, _| {
            // Trigger email sync
            tracing::info!("Refreshing emails...");
        });
        window.add_action(&refresh_action);

        // Mark all as read
        let mark_read_action = gio::SimpleAction::new("mark-all-read", None);
        mark_read_action.connect_activate(|_, _| {
            tracing::info!("Marking all as read...");
        });
        window.add_action(&mark_read_action);

        // Accounts action
        let accounts_action = gio::SimpleAction::new("accounts", None);
        let win = window.clone();
        accounts_action.connect_activate(move |_, _| {
            let accounts_dialog = AccountsView::new(&win);
            accounts_dialog.present();
        });
        window.add_action(&accounts_action);

        // Preferences action
        let prefs_action = gio::SimpleAction::new("preferences", None);
        prefs_action.connect_activate(|_, _| {
            tracing::info!("Opening preferences...");
        });
        window.add_action(&prefs_action);

        // Shortcuts action
        let shortcuts_action = gio::SimpleAction::new("shortcuts", None);
        shortcuts_action.connect_activate(|_, _| {
            tracing::info!("Showing shortcuts...");
        });
        window.add_action(&shortcuts_action);

        // About action
        let about_action = gio::SimpleAction::new("about", None);
        let win = window.clone();
        about_action.connect_activate(move |_, _| {
            let about = adw::AboutWindow::builder()
                .transient_for(&win)
                .application_name("Mail")
                .application_icon("mail-client")
                .developer_name("Winux Team")
                .version("1.0.0")
                .copyright("Copyright 2026 Winux OS Project")
                .license_type(gtk4::License::Gpl30)
                .website("https://winux.org")
                .issue_url("https://github.com/winux-os/winux/issues")
                .build();
            about.present();
        });
        window.add_action(&about_action);
    }

    fn setup_signals(&self, compose_btn: Button, add_account_btn: Button) {
        // Compose button
        let win = self.window.clone();
        compose_btn.connect_clicked(move |_| {
            let compose = ComposeView::new(&win, None);
            compose.present();
        });

        // Add account button
        let win = self.window.clone();
        add_account_btn.connect_clicked(move |_| {
            let accounts_view = AccountsView::new(&win);
            accounts_view.present();
        });

        // Folder selection
        let mailbox = self.mailbox_view.clone();
        let message = self.message_view.clone();
        let cache = self.cache.clone();
        self.folder_list.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let folder_name = row.widget_name();
                tracing::info!("Selected folder: {}", folder_name);

                // Load messages for this folder
                // mailbox.borrow_mut().load_folder(&folder_name);
            }
        });

        // Search
        let mailbox = self.mailbox_view.clone();
        self.search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            // mailbox.borrow_mut().filter(&query);
        });
    }

    fn load_emails(&self) {
        // Load cached emails on startup
        let cache = self.cache.clone();
        let mailbox = self.mailbox_view.clone();

        glib::spawn_future_local(async move {
            // Load inbox by default
            // let messages = cache.get_messages_for_folder("INBOX").await;
            // mailbox.borrow_mut().set_messages(messages);
        });
    }

    pub fn compose_mailto(&self, mailto: &str) {
        let compose = ComposeView::new(&self.window, Some(mailto));
        compose.present();
    }

    pub fn present(&self) {
        self.window.present();
    }
}
