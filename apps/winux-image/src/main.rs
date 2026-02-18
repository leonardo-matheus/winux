//! Winux Image - Native image viewer for Winux OS
//!
//! A modern, GPU-accelerated image viewer with smooth zoom, pan,
//! thumbnail navigation, metadata display, and slideshow support.

mod config;
mod metadata;
mod thumbnail;
mod viewer;

use adw::prelude::*;
use adw::subclass::prelude::*;
use config::{ImageConfig, ZoomMode};
use gtk4::{gio, glib, Orientation};
use metadata::{ImageMetadata, MetadataPanel};
use std::cell::{Cell, OnceCell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use thumbnail::ThumbnailStrip;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use viewer::ImageViewer;
use walkdir::WalkDir;

/// Application ID
const APP_ID: &str = "com.winux.Image";

/// Supported image extensions
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp", "svg", "ico",
    "heic", "heif", "avif", "raw", "cr2", "nef", "arw", "dng",
];

fn main() -> glib::ExitCode {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    info!("Starting Winux Image");

    // Create and run the application
    let app = ImageApplication::new(APP_ID);
    app.run()
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ImageApplication {
        pub window: OnceCell<adw::ApplicationWindow>,
        pub viewer: OnceCell<Rc<ImageViewer>>,
        pub thumbnails: OnceCell<Rc<ThumbnailStrip>>,
        pub metadata_panel: OnceCell<MetadataPanel>,
        pub config: RefCell<ImageConfig>,
        pub toast_overlay: OnceCell<adw::ToastOverlay>,
        pub zoom_label: OnceCell<gtk4::Label>,
        pub flap: OnceCell<adw::Flap>,
        pub slideshow_source: RefCell<Option<glib::SourceId>>,
        pub is_fullscreen: Cell<bool>,
        pub current_images: RefCell<Vec<PathBuf>>,
        pub current_index: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageApplication {
        const NAME: &'static str = "WinuxImageApplication";
        type Type = super::ImageApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for ImageApplication {}

    impl ApplicationImpl for ImageApplication {
        fn activate(&self) {
            let app = self.obj();
            app.setup_window();
        }

        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            app.setup_actions();
            app.setup_shortcuts();
            app.load_config();
        }

        fn open(&self, files: &[gio::File], _hint: &str) {
            let app = self.obj();
            app.activate();

            if let Some(file) = files.first() {
                if let Some(path) = file.path() {
                    app.open_image(&path);
                }
            }
        }
    }

    impl GtkApplicationImpl for ImageApplication {}
    impl AdwApplicationImpl for ImageApplication {}
}

glib::wrapper! {
    pub struct ImageApplication(ObjectSubclass<imp::ImageApplication>)
        @extends adw::Application, gtk4::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl ImageApplication {
    pub fn new(app_id: &str) -> Self {
        glib::Object::builder()
            .property("application-id", app_id)
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .build()
    }

    fn load_config(&self) {
        let config = ImageConfig::load();
        self.imp().config.replace(config);
    }

    fn setup_window(&self) {
        let imp = self.imp();
        let config = imp.config.borrow();

        // Create main window
        let window = adw::ApplicationWindow::builder()
            .application(self)
            .title("Winux Image")
            .default_width(config.window_width)
            .default_height(config.window_height)
            .build();

        if config.window_maximized {
            window.maximize();
        }

        drop(config);

        // Create main layout
        let main_box = gtk4::Box::new(Orientation::Vertical, 0);

        // Create header bar
        let header = self.create_header_bar();
        main_box.append(&header);

        // Create toast overlay
        let toast_overlay = adw::ToastOverlay::new();

        // Create flap for sidebar (metadata)
        let flap = adw::Flap::builder()
            .flap_position(gtk4::PackType::End)
            .reveal_flap(false)
            .fold_policy(adw::FlapFoldPolicy::Never)
            .build();

        // Content area with image viewer and thumbnails
        let content_box = gtk4::Box::new(Orientation::Vertical, 0);

        // Create image viewer
        let viewer = ImageViewer::new();
        content_box.append(viewer.widget());

        // Create thumbnail strip
        let thumbnails = ThumbnailStrip::new();
        thumbnails.set_thumbnail_size(80);

        // Thumbnail revealer
        let thumb_revealer = gtk4::Revealer::builder()
            .transition_type(gtk4::RevealerTransitionType::SlideUp)
            .reveal_child(true)
            .child(thumbnails.widget())
            .build();
        content_box.append(&thumb_revealer);

        flap.set_content(Some(&content_box));

        // Create metadata panel
        let metadata_panel = MetadataPanel::new();
        let metadata_box = gtk4::Box::new(Orientation::Vertical, 0);
        let metadata_header = gtk4::Label::builder()
            .label("Image Information")
            .css_classes(["title-4"])
            .margin_top(12)
            .margin_bottom(12)
            .build();
        metadata_box.append(&metadata_header);
        metadata_box.append(metadata_panel.widget());
        flap.set_flap(Some(&metadata_box));

        toast_overlay.set_child(Some(&flap));
        main_box.append(&toast_overlay);

        // Connect viewer signals
        let app_weak = self.downgrade();
        viewer.connect_zoom_changed(move |zoom| {
            if let Some(app) = app_weak.upgrade() {
                app.update_zoom_label(zoom);
            }
        });

        let metadata_panel_ref = metadata_panel.clone();
        viewer.connect_image_loaded(move |metadata| {
            metadata_panel_ref.update(metadata);
        });

        // Connect thumbnail selection
        let app_weak = self.downgrade();
        thumbnails.connect_selected(move |index| {
            if let Some(app) = app_weak.upgrade() {
                app.navigate_to(index);
            }
        });

        // Store references
        imp.viewer.set(viewer).unwrap();
        imp.thumbnails.set(thumbnails).unwrap();
        imp.metadata_panel.set(metadata_panel).unwrap();
        imp.toast_overlay.set(toast_overlay).unwrap();
        imp.flap.set(flap).unwrap();

        window.set_content(Some(&main_box));
        imp.window.set(window.clone()).unwrap();

        // Setup key controller for fullscreen
        let key_controller = gtk4::EventControllerKey::new();
        let app_weak = self.downgrade();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if let Some(app) = app_weak.upgrade() {
                match key {
                    gdk4::Key::F11 => {
                        app.toggle_fullscreen();
                        return glib::Propagation::Stop;
                    }
                    gdk4::Key::Escape => {
                        if app.imp().is_fullscreen.get() {
                            app.toggle_fullscreen();
                            return glib::Propagation::Stop;
                        }
                    }
                    _ => {}
                }
            }
            glib::Propagation::Proceed
        });
        window.add_controller(key_controller);

        window.present();
        info!("Winux Image window initialized");
    }

    fn create_header_bar(&self) -> adw::HeaderBar {
        let header = adw::HeaderBar::new();

        // Navigation buttons
        let nav_box = gtk4::Box::new(Orientation::Horizontal, 0);
        nav_box.add_css_class("linked");

        let prev_button = gtk4::Button::builder()
            .icon_name("go-previous-symbolic")
            .tooltip_text("Previous (Left)")
            .action_name("app.previous")
            .build();

        let next_button = gtk4::Button::builder()
            .icon_name("go-next-symbolic")
            .tooltip_text("Next (Right)")
            .action_name("app.next")
            .build();

        nav_box.append(&prev_button);
        nav_box.append(&next_button);
        header.pack_start(&nav_box);

        // Zoom controls
        let zoom_box = gtk4::Box::new(Orientation::Horizontal, 4);

        let zoom_out_btn = gtk4::Button::builder()
            .icon_name("zoom-out-symbolic")
            .tooltip_text("Zoom Out (-)")
            .action_name("app.zoom-out")
            .build();

        let zoom_label = gtk4::Label::builder()
            .label("100%")
            .width_chars(5)
            .build();

        let zoom_in_btn = gtk4::Button::builder()
            .icon_name("zoom-in-symbolic")
            .tooltip_text("Zoom In (+)")
            .action_name("app.zoom-in")
            .build();

        let zoom_fit_btn = gtk4::Button::builder()
            .icon_name("zoom-fit-best-symbolic")
            .tooltip_text("Fit to Window (F)")
            .action_name("app.zoom-fit")
            .build();

        let zoom_original_btn = gtk4::Button::builder()
            .icon_name("zoom-original-symbolic")
            .tooltip_text("Original Size (1)")
            .action_name("app.zoom-original")
            .build();

        zoom_box.append(&zoom_out_btn);
        zoom_box.append(&zoom_label);
        zoom_box.append(&zoom_in_btn);
        zoom_box.append(&zoom_fit_btn);
        zoom_box.append(&zoom_original_btn);

        self.imp().zoom_label.set(zoom_label).unwrap();

        header.set_title_widget(Some(&zoom_box));

        // Transform buttons
        let transform_box = gtk4::Box::new(Orientation::Horizontal, 0);
        transform_box.add_css_class("linked");

        let rotate_ccw_btn = gtk4::Button::builder()
            .icon_name("object-rotate-left-symbolic")
            .tooltip_text("Rotate Left (Ctrl+Left)")
            .action_name("app.rotate-ccw")
            .build();

        let rotate_cw_btn = gtk4::Button::builder()
            .icon_name("object-rotate-right-symbolic")
            .tooltip_text("Rotate Right (Ctrl+Right)")
            .action_name("app.rotate-cw")
            .build();

        let flip_h_btn = gtk4::Button::builder()
            .icon_name("object-flip-horizontal-symbolic")
            .tooltip_text("Flip Horizontal (H)")
            .action_name("app.flip-horizontal")
            .build();

        let flip_v_btn = gtk4::Button::builder()
            .icon_name("object-flip-vertical-symbolic")
            .tooltip_text("Flip Vertical (V)")
            .action_name("app.flip-vertical")
            .build();

        transform_box.append(&rotate_ccw_btn);
        transform_box.append(&rotate_cw_btn);
        transform_box.append(&flip_h_btn);
        transform_box.append(&flip_v_btn);
        header.pack_end(&transform_box);

        // Menu button
        let menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&self.create_app_menu())
            .build();
        header.pack_end(&menu_button);

        // Slideshow button
        let slideshow_btn = gtk4::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .tooltip_text("Start Slideshow (F5)")
            .action_name("app.slideshow")
            .build();
        header.pack_end(&slideshow_btn);

        // Fullscreen button
        let fullscreen_btn = gtk4::Button::builder()
            .icon_name("view-fullscreen-symbolic")
            .tooltip_text("Fullscreen (F11)")
            .action_name("app.fullscreen")
            .build();
        header.pack_end(&fullscreen_btn);

        // Info button (metadata)
        let info_btn = gtk4::ToggleButton::builder()
            .icon_name("help-about-symbolic")
            .tooltip_text("Image Information (I)")
            .action_name("app.toggle-info")
            .build();
        header.pack_end(&info_btn);

        header
    }

    fn create_app_menu(&self) -> gio::Menu {
        let menu = gio::Menu::new();

        let file_section = gio::Menu::new();
        file_section.append(Some("Open..."), Some("app.open"));
        file_section.append(Some("Open Folder..."), Some("app.open-folder"));
        menu.append_section(None, &file_section);

        let view_section = gio::Menu::new();
        view_section.append(Some("Toggle Thumbnails"), Some("app.toggle-thumbnails"));
        view_section.append(Some("Toggle Info Panel"), Some("app.toggle-info"));
        view_section.append(Some("Fullscreen"), Some("app.fullscreen"));
        menu.append_section(None, &view_section);

        let slideshow_section = gio::Menu::new();
        slideshow_section.append(Some("Start Slideshow"), Some("app.slideshow"));
        menu.append_section(None, &slideshow_section);

        let settings_section = gio::Menu::new();
        settings_section.append(Some("Preferences"), Some("app.preferences"));
        settings_section.append(Some("Keyboard Shortcuts"), Some("win.show-help-overlay"));
        settings_section.append(Some("About Winux Image"), Some("app.about"));
        menu.append_section(None, &settings_section);

        menu
    }

    fn setup_actions(&self) {
        // Open action
        let open_action = gio::SimpleAction::new("open", None);
        let app_weak = self.downgrade();
        open_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_open_dialog();
            }
        });
        self.add_action(&open_action);

        // Open folder action
        let open_folder_action = gio::SimpleAction::new("open-folder", None);
        let app_weak = self.downgrade();
        open_folder_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_open_folder_dialog();
            }
        });
        self.add_action(&open_folder_action);

        // Navigation actions
        let next_action = gio::SimpleAction::new("next", None);
        let app_weak = self.downgrade();
        next_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.navigate_next();
            }
        });
        self.add_action(&next_action);

        let prev_action = gio::SimpleAction::new("previous", None);
        let app_weak = self.downgrade();
        prev_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.navigate_previous();
            }
        });
        self.add_action(&prev_action);

        // Zoom actions
        let zoom_in_action = gio::SimpleAction::new("zoom-in", None);
        let app_weak = self.downgrade();
        zoom_in_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(viewer) = app.imp().viewer.get() {
                    viewer.zoom_in();
                }
            }
        });
        self.add_action(&zoom_in_action);

        let zoom_out_action = gio::SimpleAction::new("zoom-out", None);
        let app_weak = self.downgrade();
        zoom_out_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(viewer) = app.imp().viewer.get() {
                    viewer.zoom_out();
                }
            }
        });
        self.add_action(&zoom_out_action);

        let zoom_fit_action = gio::SimpleAction::new("zoom-fit", None);
        let app_weak = self.downgrade();
        zoom_fit_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(viewer) = app.imp().viewer.get() {
                    viewer.zoom_fit();
                }
            }
        });
        self.add_action(&zoom_fit_action);

        let zoom_original_action = gio::SimpleAction::new("zoom-original", None);
        let app_weak = self.downgrade();
        zoom_original_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(viewer) = app.imp().viewer.get() {
                    viewer.zoom_original();
                }
            }
        });
        self.add_action(&zoom_original_action);

        // Rotation actions
        let rotate_cw_action = gio::SimpleAction::new("rotate-cw", None);
        let app_weak = self.downgrade();
        rotate_cw_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(viewer) = app.imp().viewer.get() {
                    viewer.rotate_cw();
                }
            }
        });
        self.add_action(&rotate_cw_action);

        let rotate_ccw_action = gio::SimpleAction::new("rotate-ccw", None);
        let app_weak = self.downgrade();
        rotate_ccw_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(viewer) = app.imp().viewer.get() {
                    viewer.rotate_ccw();
                }
            }
        });
        self.add_action(&rotate_ccw_action);

        // Flip actions
        let flip_h_action = gio::SimpleAction::new("flip-horizontal", None);
        let app_weak = self.downgrade();
        flip_h_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(viewer) = app.imp().viewer.get() {
                    viewer.flip_horizontal();
                }
            }
        });
        self.add_action(&flip_h_action);

        let flip_v_action = gio::SimpleAction::new("flip-vertical", None);
        let app_weak = self.downgrade();
        flip_v_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                if let Some(viewer) = app.imp().viewer.get() {
                    viewer.flip_vertical();
                }
            }
        });
        self.add_action(&flip_v_action);

        // Toggle actions
        let toggle_info_action = gio::SimpleAction::new_stateful("toggle-info", None, &false.to_variant());
        let app_weak = self.downgrade();
        toggle_info_action.connect_activate(move |action, _| {
            let state = action.state().unwrap();
            let current: bool = state.get().unwrap();
            action.set_state(&(!current).to_variant());
            if let Some(app) = app_weak.upgrade() {
                app.toggle_info_panel(!current);
            }
        });
        self.add_action(&toggle_info_action);

        let toggle_thumbnails_action = gio::SimpleAction::new_stateful("toggle-thumbnails", None, &true.to_variant());
        let app_weak = self.downgrade();
        toggle_thumbnails_action.connect_activate(move |action, _| {
            let state = action.state().unwrap();
            let current: bool = state.get().unwrap();
            action.set_state(&(!current).to_variant());
            // Toggle thumbnail visibility would be implemented here
        });
        self.add_action(&toggle_thumbnails_action);

        // Fullscreen action
        let fullscreen_action = gio::SimpleAction::new("fullscreen", None);
        let app_weak = self.downgrade();
        fullscreen_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.toggle_fullscreen();
            }
        });
        self.add_action(&fullscreen_action);

        // Slideshow action
        let slideshow_action = gio::SimpleAction::new("slideshow", None);
        let app_weak = self.downgrade();
        slideshow_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.toggle_slideshow();
            }
        });
        self.add_action(&slideshow_action);

        // Preferences action
        let prefs_action = gio::SimpleAction::new("preferences", None);
        let app_weak = self.downgrade();
        prefs_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_preferences();
            }
        });
        self.add_action(&prefs_action);

        // About action
        let about_action = gio::SimpleAction::new("about", None);
        let app_weak = self.downgrade();
        about_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_about_dialog();
            }
        });
        self.add_action(&about_action);

        // Quit action
        let quit_action = gio::SimpleAction::new("quit", None);
        let app_weak = self.downgrade();
        quit_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.quit();
            }
        });
        self.add_action(&quit_action);
    }

    fn setup_shortcuts(&self) {
        // File
        self.set_accels_for_action("app.open", &["<Control>o"]);
        self.set_accels_for_action("app.quit", &["<Control>q"]);

        // Navigation
        self.set_accels_for_action("app.next", &["Right", "space", "Page_Down"]);
        self.set_accels_for_action("app.previous", &["Left", "BackSpace", "Page_Up"]);

        // Zoom
        self.set_accels_for_action("app.zoom-in", &["plus", "equal", "<Control>plus", "<Control>equal"]);
        self.set_accels_for_action("app.zoom-out", &["minus", "<Control>minus"]);
        self.set_accels_for_action("app.zoom-fit", &["f", "0"]);
        self.set_accels_for_action("app.zoom-original", &["1"]);

        // Transform
        self.set_accels_for_action("app.rotate-cw", &["<Control>Right", "r"]);
        self.set_accels_for_action("app.rotate-ccw", &["<Control>Left", "<Shift>r"]);
        self.set_accels_for_action("app.flip-horizontal", &["h"]);
        self.set_accels_for_action("app.flip-vertical", &["v"]);

        // View
        self.set_accels_for_action("app.fullscreen", &["F11", "f11"]);
        self.set_accels_for_action("app.toggle-info", &["i"]);
        self.set_accels_for_action("app.slideshow", &["F5", "s"]);
    }

    fn show_open_dialog(&self) {
        let window = self.imp().window.get();

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Images"));
        for ext in SUPPORTED_EXTENSIONS {
            filter.add_pattern(&format!("*.{}", ext));
            filter.add_pattern(&format!("*.{}", ext.to_uppercase()));
        }

        let filters = gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);

        let dialog = gtk4::FileDialog::builder()
            .title("Open Image")
            .modal(true)
            .filters(&filters)
            .build();

        let app_weak = self.downgrade();
        dialog.open(window.as_ref(), None::<&gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Some(app) = app_weak.upgrade() {
                        app.open_image(&path);
                    }
                }
            }
        });
    }

    fn show_open_folder_dialog(&self) {
        let window = self.imp().window.get();

        let dialog = gtk4::FileDialog::builder()
            .title("Open Folder")
            .modal(true)
            .build();

        let app_weak = self.downgrade();
        dialog.select_folder(window.as_ref(), None::<&gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Some(app) = app_weak.upgrade() {
                        app.open_folder(&path);
                    }
                }
            }
        });
    }

    pub fn open_image(&self, path: &PathBuf) {
        let imp = self.imp();

        // Find sibling images in the same directory
        if let Some(parent) = path.parent() {
            let images = self.scan_directory(parent);
            let current_index = images.iter().position(|p| p == path).unwrap_or(0);

            *imp.current_images.borrow_mut() = images.clone();
            imp.current_index.set(current_index as i32);

            // Load thumbnails
            if let Some(thumbnails) = imp.thumbnails.get() {
                thumbnails.load_images(images);
                thumbnails.select(current_index);
            }
        }

        // Load the image
        if let Some(viewer) = imp.viewer.get() {
            if let Err(e) = viewer.load(path) {
                warn!("Failed to load image: {}", e);
                self.show_toast(&format!("Failed to load image: {}", e));
                return;
            }
        }

        // Update window title
        if let Some(window) = imp.window.get() {
            let filename = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Image".to_string());
            window.set_title(Some(&format!("{} - Winux Image", filename)));
        }

        // Add to recent files
        let mut config = imp.config.borrow_mut();
        config.add_recent_file(path.clone());
        let _ = config.save();
    }

    fn open_folder(&self, path: &PathBuf) {
        let images = self.scan_directory(path);
        if images.is_empty() {
            self.show_toast("No images found in folder");
            return;
        }

        let imp = self.imp();
        *imp.current_images.borrow_mut() = images.clone();
        imp.current_index.set(0);

        // Load thumbnails
        if let Some(thumbnails) = imp.thumbnails.get() {
            thumbnails.load_images(images.clone());
            thumbnails.select(0);
        }

        // Load first image
        self.open_image(&images[0]);
    }

    fn scan_directory(&self, dir: &std::path::Path) -> Vec<PathBuf> {
        let mut images: Vec<PathBuf> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .collect();

        images.sort_by(|a, b| {
            a.file_name()
                .cmp(&b.file_name())
        });

        images
    }

    fn navigate_to(&self, index: usize) {
        let imp = self.imp();
        let images = imp.current_images.borrow();

        if index >= images.len() {
            return;
        }

        let path = images[index].clone();
        drop(images);

        imp.current_index.set(index as i32);

        if let Some(viewer) = imp.viewer.get() {
            if let Err(e) = viewer.load(&path) {
                warn!("Failed to load image: {}", e);
            }
        }

        if let Some(thumbnails) = imp.thumbnails.get() {
            thumbnails.select(index);
        }

        // Update window title
        if let Some(window) = imp.window.get() {
            let filename = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Image".to_string());
            window.set_title(Some(&format!("{} - Winux Image", filename)));
        }
    }

    fn navigate_next(&self) {
        let imp = self.imp();
        let current = imp.current_index.get();
        let count = imp.current_images.borrow().len() as i32;

        if current < count - 1 {
            self.navigate_to((current + 1) as usize);
        }
    }

    fn navigate_previous(&self) {
        let imp = self.imp();
        let current = imp.current_index.get();

        if current > 0 {
            self.navigate_to((current - 1) as usize);
        }
    }

    fn update_zoom_label(&self, zoom: f64) {
        if let Some(label) = self.imp().zoom_label.get() {
            label.set_label(&format!("{}%", (zoom * 100.0) as i32));
        }
    }

    fn toggle_info_panel(&self, show: bool) {
        if let Some(flap) = self.imp().flap.get() {
            flap.set_reveal_flap(show);
        }
    }

    fn toggle_fullscreen(&self) {
        let imp = self.imp();
        let is_fullscreen = imp.is_fullscreen.get();

        if let Some(window) = imp.window.get() {
            if is_fullscreen {
                window.unfullscreen();
            } else {
                window.fullscreen();
            }
            imp.is_fullscreen.set(!is_fullscreen);
        }
    }

    fn toggle_slideshow(&self) {
        let imp = self.imp();

        // Check if slideshow is running
        if let Some(source_id) = imp.slideshow_source.borrow_mut().take() {
            source_id.remove();
            self.show_toast("Slideshow stopped");
            return;
        }

        // Start slideshow
        let interval = imp.config.borrow().slideshow_interval;
        let app_weak = self.downgrade();

        let source_id = glib::timeout_add_local(
            Duration::from_secs(interval as u64),
            move || {
                if let Some(app) = app_weak.upgrade() {
                    let imp = app.imp();
                    let current = imp.current_index.get();
                    let count = imp.current_images.borrow().len() as i32;

                    if current < count - 1 {
                        app.navigate_next();
                        glib::ControlFlow::Continue
                    } else if imp.config.borrow().slideshow_loop {
                        app.navigate_to(0);
                        glib::ControlFlow::Continue
                    } else {
                        *imp.slideshow_source.borrow_mut() = None;
                        app.show_toast("Slideshow finished");
                        glib::ControlFlow::Break
                    }
                } else {
                    glib::ControlFlow::Break
                }
            },
        );

        *imp.slideshow_source.borrow_mut() = Some(source_id);
        self.show_toast("Slideshow started");

        // Enter fullscreen for slideshow
        if !imp.is_fullscreen.get() {
            self.toggle_fullscreen();
        }
    }

    fn show_preferences(&self) {
        info!("Opening preferences");
        self.show_toast("Preferences not yet implemented");
    }

    fn show_about_dialog(&self) {
        let window = self.imp().window.get();

        let about = adw::AboutDialog::builder()
            .application_name("Winux Image")
            .application_icon("image-viewer")
            .version("1.0.0")
            .developer_name("Winux Team")
            .copyright("2024 Winux Project")
            .license_type(gtk4::License::Gpl30)
            .website("https://winux.org")
            .issue_url("https://github.com/winux-os/winux/issues")
            .comments("A modern, fast image viewer for Winux OS")
            .build();

        about.add_credit_section(Some("Created by"), &["Winux Development Team"]);

        // Add supported formats
        let formats = SUPPORTED_EXTENSIONS.join(", ").to_uppercase();
        about.add_legal_section(
            "Supported Formats",
            None,
            gtk4::License::Unknown,
            Some(&formats),
        );

        if let Some(win) = window {
            about.present(Some(win));
        }
    }

    pub fn show_toast(&self, message: &str) {
        if let Some(toast_overlay) = self.imp().toast_overlay.get() {
            let toast = adw::Toast::new(message);
            toast_overlay.add_toast(toast);
        }
    }
}
