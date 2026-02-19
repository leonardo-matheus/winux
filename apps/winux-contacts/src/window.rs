// Window module - Main application window

use crate::data::storage::ContactStorage;
use crate::views::{detail::DetailView, editor::EditorView, list::ContactListView};
use crate::ui::avatar::AvatarHelper;
use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Entry, MenuButton, Orientation, Paned, Revealer,
    RevealerTransitionType, ScrolledWindow, SearchBar, SearchEntry, Stack, StackTransitionType,
};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_ui(app: &Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Contacts")
        .default_width(1000)
        .default_height(700)
        .build();

    // Initialize storage
    let storage = Rc::new(RefCell::new(
        ContactStorage::new().expect("Failed to initialize contact storage"),
    ));

    // Main layout
    let main_box = Box::new(Orientation::Vertical, 0);

    // Header bar
    let header = build_header_bar(&window, &storage);
    main_box.append(&header);

    // Content area with paned view
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned.set_position(350);

    // Left side: Contact list
    let list_box = Box::new(Orientation::Vertical, 0);
    list_box.set_width_request(300);

    // Search bar
    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search contacts..."));
    search_entry.set_margin_start(12);
    search_entry.set_margin_end(12);
    search_entry.set_margin_top(12);
    search_entry.set_margin_bottom(12);

    list_box.append(&search_entry);

    // Contact list view
    let list_scroll = ScrolledWindow::new();
    list_scroll.set_vexpand(true);
    list_scroll.set_hexpand(true);

    let contact_list = ContactListView::new(storage.clone());
    list_scroll.set_child(Some(contact_list.widget()));
    list_box.append(&list_scroll);

    // Sidebar styling
    list_box.add_css_class("sidebar");

    paned.set_start_child(Some(&list_box));

    // Right side: Detail/Editor stack
    let right_stack = Stack::new();
    right_stack.set_transition_type(StackTransitionType::Crossfade);
    right_stack.set_transition_duration(200);

    // Empty state
    let empty_state = build_empty_state();
    right_stack.add_named(&empty_state, Some("empty"));

    // Detail view
    let detail_view = DetailView::new(storage.clone());
    right_stack.add_named(detail_view.widget(), Some("detail"));

    // Editor view
    let editor_view = EditorView::new(storage.clone());
    right_stack.add_named(editor_view.widget(), Some("editor"));

    right_stack.set_visible_child_name("empty");
    paned.set_end_child(Some(&right_stack));

    main_box.append(&paned);

    // Connect signals
    let storage_clone = storage.clone();
    let right_stack_clone = right_stack.clone();
    let detail_view_clone = detail_view.clone();

    contact_list.connect_contact_selected(move |contact_id| {
        if let Some(id) = contact_id {
            if let Ok(storage) = storage_clone.try_borrow() {
                if let Ok(Some(contact)) = storage.get_contact(&id) {
                    detail_view_clone.set_contact(Some(contact));
                    right_stack_clone.set_visible_child_name("detail");
                }
            }
        } else {
            right_stack_clone.set_visible_child_name("empty");
        }
    });

    // Search filtering
    let contact_list_clone = contact_list.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        contact_list_clone.filter(&query);
    });

    // Edit button in detail view
    let right_stack_clone = right_stack.clone();
    let editor_view_clone = editor_view.clone();
    detail_view.connect_edit_requested(move |contact| {
        editor_view_clone.set_contact(Some(contact));
        right_stack_clone.set_visible_child_name("editor");
    });

    // Save/Cancel in editor
    let right_stack_clone = right_stack.clone();
    let contact_list_clone = contact_list.clone();
    let detail_view_clone = detail_view.clone();
    editor_view.connect_saved(move |contact| {
        contact_list_clone.refresh();
        detail_view_clone.set_contact(Some(contact));
        right_stack_clone.set_visible_child_name("detail");
    });

    let right_stack_clone = right_stack.clone();
    editor_view.connect_cancelled(move || {
        right_stack_clone.set_visible_child_name("detail");
    });

    // Delete action
    let right_stack_clone = right_stack.clone();
    let contact_list_clone = contact_list.clone();
    detail_view.connect_deleted(move || {
        contact_list_clone.refresh();
        right_stack_clone.set_visible_child_name("empty");
    });

    window.set_content(Some(&main_box));
    window.present();
}

fn build_header_bar(
    window: &adw::ApplicationWindow,
    storage: &Rc<RefCell<ContactStorage>>,
) -> adw::HeaderBar {
    let header = adw::HeaderBar::new();

    // Title
    let title = adw::WindowTitle::new("Contacts", "");
    header.set_title_widget(Some(&title));

    // Add contact button
    let add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add Contact")
        .build();
    add_button.add_css_class("suggested-action");
    header.pack_start(&add_button);

    // Menu button
    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Menu")
        .build();

    let menu = gio::Menu::new();

    let import_section = gio::Menu::new();
    import_section.append(Some("Import from vCard..."), Some("app.import-vcard"));
    import_section.append(Some("Import from CSV..."), Some("app.import-csv"));
    menu.append_section(Some("Import"), &import_section);

    let export_section = gio::Menu::new();
    export_section.append(Some("Export to vCard..."), Some("app.export-vcard"));
    export_section.append(Some("Export to CSV..."), Some("app.export-csv"));
    menu.append_section(Some("Export"), &export_section);

    let sync_section = gio::Menu::new();
    sync_section.append(Some("Add CardDAV Account..."), Some("app.add-carddav"));
    sync_section.append(Some("Sync Now"), Some("app.sync-now"));
    menu.append_section(Some("Sync"), &sync_section);

    let other_section = gio::Menu::new();
    other_section.append(Some("Preferences"), Some("app.preferences"));
    other_section.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));
    other_section.append(Some("About Contacts"), Some("app.about"));
    menu.append_section(None, &other_section);

    menu_button.set_menu_model(Some(&menu));
    header.pack_end(&menu_button);

    // Groups/Labels button
    let groups_button = Button::builder()
        .icon_name("folder-symbolic")
        .tooltip_text("Groups")
        .build();
    header.pack_end(&groups_button);

    // Sort button
    let sort_button = MenuButton::builder()
        .icon_name("view-sort-descending-symbolic")
        .tooltip_text("Sort")
        .build();

    let sort_menu = gio::Menu::new();
    sort_menu.append(Some("First Name"), Some("app.sort-firstname"));
    sort_menu.append(Some("Last Name"), Some("app.sort-lastname"));
    sort_menu.append(Some("Company"), Some("app.sort-company"));
    sort_menu.append(Some("Recently Added"), Some("app.sort-recent"));
    sort_button.set_menu_model(Some(&sort_menu));
    header.pack_end(&sort_button);

    header
}

fn build_empty_state() -> Box {
    let empty_box = Box::new(Orientation::Vertical, 12);
    empty_box.set_valign(gtk4::Align::Center);
    empty_box.set_halign(gtk4::Align::Center);

    let icon = gtk4::Image::from_icon_name("contact-new-symbolic");
    icon.set_pixel_size(128);
    icon.add_css_class("dim-label");
    empty_box.append(&icon);

    let title = gtk4::Label::new(Some("No Contact Selected"));
    title.add_css_class("title-1");
    empty_box.append(&title);

    let subtitle = gtk4::Label::new(Some("Select a contact from the list or add a new one"));
    subtitle.add_css_class("dim-label");
    empty_box.append(&subtitle);

    let add_button = Button::builder()
        .label("Add Contact")
        .build();
    add_button.add_css_class("suggested-action");
    add_button.add_css_class("pill");
    add_button.set_margin_top(24);
    empty_box.append(&add_button);

    empty_box
}
