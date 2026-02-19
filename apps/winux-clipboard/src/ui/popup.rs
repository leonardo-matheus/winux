//! Main popup window for clipboard history

use glib::Object;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{
    glib, Application, ApplicationWindow, Button, Entry, Label, ListBox, ListBoxRow,
    Orientation, Paned, ScrolledWindow, SelectionMode, Box as GtkBox, CssProvider,
    PopoverMenu, gio,
};
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::history::{ClipboardHistory, ClipboardItem, ContentType, ItemId};
use super::item_row::{ItemRow, item_row_css};
use super::preview::{PreviewPanel, preview_css};

mod imp {
    use super::*;

    pub struct ClipboardPopup {
        pub config: RefCell<Config>,
        pub history: RefCell<Option<Arc<RwLock<ClipboardHistory>>>>,
        pub search_entry: RefCell<Option<Entry>>,
        pub list_box: RefCell<Option<ListBox>>,
        pub preview_panel: RefCell<Option<PreviewPanel>>,
        pub filter_type: RefCell<Option<ContentType>>,
        pub on_paste: RefCell<Option<Box<dyn Fn(ClipboardItem) + 'static>>>,
        pub on_delete: RefCell<Option<Box<dyn Fn(ItemId) + 'static>>>,
        pub on_pin: RefCell<Option<Box<dyn Fn(ItemId) + 'static>>>,
        pub on_clear: RefCell<Option<Box<dyn Fn() + 'static>>>,
    }

    impl Default for ClipboardPopup {
        fn default() -> Self {
            Self {
                config: RefCell::new(Config::default()),
                history: RefCell::new(None),
                search_entry: RefCell::new(None),
                list_box: RefCell::new(None),
                preview_panel: RefCell::new(None),
                filter_type: RefCell::new(None),
                on_paste: RefCell::new(None),
                on_delete: RefCell::new(None),
                on_pin: RefCell::new(None),
                on_clear: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ClipboardPopup {
        const NAME: &'static str = "WinuxClipboardPopup";
        type Type = super::ClipboardPopup;
        type ParentType = ApplicationWindow;
    }

    impl ObjectImpl for ClipboardPopup {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            self.obj().setup_actions();
        }
    }

    impl WidgetImpl for ClipboardPopup {}
    impl WindowImpl for ClipboardPopup {}
    impl ApplicationWindowImpl for ClipboardPopup {}
}

glib::wrapper! {
    pub struct ClipboardPopup(ObjectSubclass<imp::ClipboardPopup>)
        @extends ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable,
                    gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl ClipboardPopup {
    pub fn new(app: &Application, config: Config) -> Self {
        let window: Self = Object::builder()
            .property("application", app)
            .property("title", "Clipboard History")
            .property("default-width", 800)
            .property("default-height", 600)
            .property("resizable", true)
            .build();

        *window.imp().config.borrow_mut() = config;

        // Load CSS
        window.load_css();

        window
    }

    fn load_css(&self) {
        let provider = CssProvider::new();
        let css = format!(
            r#"
            window.clipboard-popup {{
                background-color: @window_bg_color;
            }}

            .clipboard-header {{
                padding: 12px 16px;
                background-color: @headerbar_bg_color;
                border-bottom: 1px solid @borders;
            }}

            .search-entry {{
                min-width: 300px;
            }}

            .filter-button {{
                padding: 4px 12px;
                border-radius: 99px;
                font-size: 12px;
            }}

            .filter-button.active {{
                background-color: @accent_bg_color;
                color: @accent_fg_color;
            }}

            .history-list {{
                background-color: transparent;
            }}

            .history-list row {{
                padding: 0;
                margin: 2px 8px;
                border-radius: 8px;
            }}

            .history-list row:selected {{
                background-color: alpha(@accent_bg_color, 0.15);
            }}

            .empty-state {{
                padding: 48px;
            }}

            .action-bar {{
                padding: 8px 16px;
                background-color: @headerbar_bg_color;
                border-top: 1px solid @borders;
            }}

            {}
            {}
            "#,
            item_row_css(),
            preview_css()
        );

        provider.load_from_string(&css);

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not get display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        self.add_css_class("clipboard-popup");
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Main container
        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        // Header
        let header = self.create_header();
        main_box.append(&header);

        // Content area with paned view
        let paned = Paned::builder()
            .orientation(Orientation::Horizontal)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .vexpand(true)
            .build();

        // Left side: history list
        let list_container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .width_request(400)
            .build();

        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .activate_on_single_click(false)
            .build();
        list_box.add_css_class("history-list");

        // Connect selection changed
        let preview_panel = PreviewPanel::new();
        let preview_ref = preview_panel.clone();
        list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(item_row) = row.child().and_then(|c| c.downcast::<ItemRow>().ok()) {
                    if let Some(item) = item_row.get_item() {
                        preview_ref.set_item(Some(&item));
                        return;
                    }
                }
            }
            preview_ref.set_item(None);
        });

        // Double-click to paste
        let window = self.clone();
        list_box.connect_row_activated(move |_, row| {
            if let Some(item_row) = row.child().and_then(|c| c.downcast::<ItemRow>().ok()) {
                if let Some(item) = item_row.get_item() {
                    window.emit_paste(item);
                }
            }
        });

        scroll.set_child(Some(&list_box));
        list_container.append(&scroll);

        *imp.list_box.borrow_mut() = Some(list_box);

        paned.set_start_child(Some(&list_container));

        // Right side: preview panel
        *imp.preview_panel.borrow_mut() = Some(preview_panel.clone());
        paned.set_end_child(Some(&preview_panel));

        main_box.append(&paned);

        // Action bar
        let action_bar = self.create_action_bar();
        main_box.append(&action_bar);

        self.set_child(Some(&main_box));

        // Keyboard shortcuts
        self.setup_shortcuts();
    }

    fn create_header(&self) -> GtkBox {
        let imp = self.imp();

        let header = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();
        header.add_css_class("clipboard-header");

        // Search entry
        let search_entry = Entry::builder()
            .placeholder_text("Search clipboard history...")
            .hexpand(true)
            .build();
        search_entry.add_css_class("search-entry");

        let window = self.clone();
        search_entry.connect_changed(move |entry| {
            window.filter_items(entry.text().as_str());
        });

        header.append(&search_entry);
        *imp.search_entry.borrow_mut() = Some(search_entry);

        // Filter buttons
        let filter_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        let filters = [
            ("All", None),
            ("Text", Some(ContentType::Text)),
            ("Images", Some(ContentType::Image)),
            ("Files", Some(ContentType::Files)),
        ];

        for (label, filter_type) in filters {
            let btn = Button::builder()
                .label(label)
                .build();
            btn.add_css_class("filter-button");

            if filter_type.is_none() {
                btn.add_css_class("active");
            }

            let window = self.clone();
            let ft = filter_type.clone();
            btn.connect_clicked(move |clicked_btn| {
                // Update active state
                if let Some(parent) = clicked_btn.parent() {
                    let mut child = parent.first_child();
                    while let Some(c) = child {
                        c.remove_css_class("active");
                        child = c.next_sibling();
                    }
                }
                clicked_btn.add_css_class("active");

                // Apply filter
                *window.imp().filter_type.borrow_mut() = ft.clone();
                window.refresh_list();
            });

            filter_box.append(&btn);
        }

        header.append(&filter_box);

        header
    }

    fn create_action_bar(&self) -> GtkBox {
        let action_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        action_bar.add_css_class("action-bar");

        // Stats label
        let stats_label = Label::builder()
            .label("0 items")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        stats_label.add_css_class("dim-label");
        action_bar.append(&stats_label);

        // Delete button
        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Delete selected")
            .build();
        let window = self.clone();
        delete_btn.connect_clicked(move |_| {
            window.delete_selected();
        });
        action_bar.append(&delete_btn);

        // Pin button
        let pin_btn = Button::builder()
            .icon_name("view-pin-symbolic")
            .tooltip_text("Pin/unpin selected")
            .build();
        let window = self.clone();
        pin_btn.connect_clicked(move |_| {
            window.pin_selected();
        });
        action_bar.append(&pin_btn);

        // Clear button
        let clear_btn = Button::builder()
            .icon_name("edit-clear-all-symbolic")
            .tooltip_text("Clear history")
            .build();
        let window = self.clone();
        clear_btn.connect_clicked(move |_| {
            window.emit_clear();
        });
        action_bar.append(&clear_btn);

        action_bar
    }

    fn setup_shortcuts(&self) {
        // Escape to close
        let window = self.clone();
        let controller = gtk4::EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                window.close();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        self.add_controller(controller);
    }

    fn setup_actions(&self) {
        // Paste action
        let paste_action = gio::SimpleAction::new("paste", None);
        let window = self.clone();
        paste_action.connect_activate(move |_, _| {
            if let Some(list_box) = window.imp().list_box.borrow().as_ref() {
                if let Some(row) = list_box.selected_row() {
                    if let Some(item_row) = row.child().and_then(|c| c.downcast::<ItemRow>().ok()) {
                        if let Some(item) = item_row.get_item() {
                            window.emit_paste(item);
                        }
                    }
                }
            }
        });
        self.add_action(&paste_action);

        // Delete action
        let delete_action = gio::SimpleAction::new("delete", None);
        let window = self.clone();
        delete_action.connect_activate(move |_, _| {
            window.delete_selected();
        });
        self.add_action(&delete_action);

        // Pin action
        let pin_action = gio::SimpleAction::new("pin", None);
        let window = self.clone();
        pin_action.connect_activate(move |_, _| {
            window.pin_selected();
        });
        self.add_action(&pin_action);
    }

    pub fn set_history(&self, history: Arc<RwLock<ClipboardHistory>>) {
        *self.imp().history.borrow_mut() = Some(history);
    }

    pub fn refresh_list(&self) {
        let imp = self.imp();

        let list_box = match imp.list_box.borrow().clone() {
            Some(lb) => lb,
            None => return,
        };

        // Clear existing items
        while let Some(row) = list_box.first_child() {
            list_box.remove(&row);
        }

        let history = match imp.history.borrow().clone() {
            Some(h) => h,
            None => return,
        };

        // Get items (this is a blocking call, should be improved for large histories)
        let rt = tokio::runtime::Handle::try_current();
        let items: Vec<ClipboardItem> = if let Ok(handle) = rt {
            handle.block_on(async {
                let history = history.read().await;
                history.items().to_vec()
            })
        } else {
            return;
        };

        // Apply filter
        let filter_type = imp.filter_type.borrow().clone();
        let search_text = imp.search_entry.borrow()
            .as_ref()
            .map(|e| e.text().to_string())
            .unwrap_or_default();

        let filtered: Vec<&ClipboardItem> = items
            .iter()
            .filter(|item| {
                // Type filter
                if let Some(ref ft) = filter_type {
                    if &item.content_type != ft {
                        return false;
                    }
                }

                // Search filter
                if !search_text.is_empty() {
                    if !item.matches_search(&search_text) {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Show empty state or items
        if filtered.is_empty() {
            let empty_row = ListBoxRow::new();
            let empty_box = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .valign(gtk4::Align::Center)
                .halign(gtk4::Align::Center)
                .spacing(12)
                .build();
            empty_box.add_css_class("empty-state");

            let icon = gtk4::Image::builder()
                .icon_name("edit-paste-symbolic")
                .pixel_size(48)
                .opacity(0.5)
                .build();
            empty_box.append(&icon);

            let label = Label::new(Some("No items in clipboard history"));
            label.add_css_class("dim-label");
            empty_box.append(&label);

            empty_row.set_child(Some(&empty_box));
            empty_row.set_selectable(false);
            empty_row.set_activatable(false);
            list_box.append(&empty_row);
        } else {
            // Add pinned items first
            for item in filtered.iter().filter(|i| i.pinned) {
                self.add_item_row(&list_box, item);
            }

            // Add regular items
            for item in filtered.iter().filter(|i| !i.pinned) {
                self.add_item_row(&list_box, item);
            }
        }
    }

    fn add_item_row(&self, list_box: &ListBox, item: &ClipboardItem) {
        let row = ListBoxRow::builder()
            .selectable(true)
            .activatable(true)
            .build();

        let item_row = ItemRow::new();
        item_row.set_item(item);

        // Context menu
        let menu = gio::Menu::new();
        menu.append(Some("Paste"), Some("win.paste"));
        menu.append(Some("Pin/Unpin"), Some("win.pin"));
        menu.append(Some("Delete"), Some("win.delete"));

        let popover = PopoverMenu::from_model(Some(&menu));
        popover.set_parent(&item_row);

        let gesture = gtk4::GestureClick::builder()
            .button(3) // Right click
            .build();
        let pop = popover.clone();
        gesture.connect_pressed(move |_, _, x, y| {
            pop.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            pop.popup();
        });
        item_row.add_controller(gesture);

        row.set_child(Some(&item_row));
        list_box.append(&row);
    }

    fn filter_items(&self, query: &str) {
        self.refresh_list();
    }

    fn delete_selected(&self) {
        let imp = self.imp();

        if let Some(list_box) = imp.list_box.borrow().as_ref() {
            if let Some(row) = list_box.selected_row() {
                if let Some(item_row) = row.child().and_then(|c| c.downcast::<ItemRow>().ok()) {
                    if let Some(id) = item_row.get_item_id() {
                        if let Some(ref callback) = *imp.on_delete.borrow() {
                            callback(id);
                        }
                    }
                }
            }
        }
    }

    fn pin_selected(&self) {
        let imp = self.imp();

        if let Some(list_box) = imp.list_box.borrow().as_ref() {
            if let Some(row) = list_box.selected_row() {
                if let Some(item_row) = row.child().and_then(|c| c.downcast::<ItemRow>().ok()) {
                    if let Some(id) = item_row.get_item_id() {
                        if let Some(ref callback) = *imp.on_pin.borrow() {
                            callback(id);
                        }
                    }
                }
            }
        }
    }

    fn emit_paste(&self, item: ClipboardItem) {
        let imp = self.imp();
        if let Some(ref callback) = *imp.on_paste.borrow() {
            callback(item);
        }
        self.close();
    }

    fn emit_clear(&self) {
        let imp = self.imp();
        if let Some(ref callback) = *imp.on_clear.borrow() {
            callback();
        }
    }

    pub fn connect_paste<F: Fn(ClipboardItem) + 'static>(&self, f: F) {
        *self.imp().on_paste.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_delete<F: Fn(ItemId) + 'static>(&self, f: F) {
        *self.imp().on_delete.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_pin<F: Fn(ItemId) + 'static>(&self, f: F) {
        *self.imp().on_pin.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_clear<F: Fn() + 'static>(&self, f: F) {
        *self.imp().on_clear.borrow_mut() = Some(Box::new(f));
    }
}
