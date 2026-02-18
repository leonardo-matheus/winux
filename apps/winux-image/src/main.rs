use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, Button, Picture, ScrolledWindow, Box as GtkBox, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{HeaderBar, ApplicationWindow as AdwApplicationWindow};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const APP_ID: &str = "org.winux.image";

fn main() -> gtk::glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        adw::init().expect("Failed to initialize libadwaita");
    });

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // State
    let current_path: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let current_files: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));
    let current_index: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let zoom_level: Rc<RefCell<f64>> = Rc::new(RefCell::new(1.0));

    // Apply dark theme
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

    // Picture widget for displaying images
    let picture = Picture::new();
    picture.set_can_shrink(true);
    picture.set_keep_aspect_ratio(true);

    // Scrolled window for zooming
    let scrolled_window = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&picture)
        .build();

    // Header bar buttons
    let open_button = Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open Image")
        .build();

    let zoom_in_button = Button::builder()
        .icon_name("zoom-in-symbolic")
        .tooltip_text("Zoom In")
        .build();

    let zoom_out_button = Button::builder()
        .icon_name("zoom-out-symbolic")
        .tooltip_text("Zoom Out")
        .build();

    let zoom_reset_button = Button::builder()
        .icon_name("zoom-fit-best-symbolic")
        .tooltip_text("Reset Zoom")
        .build();

    let prev_button = Button::builder()
        .icon_name("go-previous-symbolic")
        .tooltip_text("Previous Image")
        .build();

    let next_button = Button::builder()
        .icon_name("go-next-symbolic")
        .tooltip_text("Next Image")
        .build();

    // Zoom controls box
    let zoom_box = GtkBox::new(Orientation::Horizontal, 0);
    zoom_box.add_css_class("linked");
    zoom_box.append(&zoom_out_button);
    zoom_box.append(&zoom_reset_button);
    zoom_box.append(&zoom_in_button);

    // Navigation box
    let nav_box = GtkBox::new(Orientation::Horizontal, 0);
    nav_box.add_css_class("linked");
    nav_box.append(&prev_button);
    nav_box.append(&next_button);

    // Header bar
    let header_bar = HeaderBar::new();
    header_bar.pack_start(&open_button);
    header_bar.pack_start(&nav_box);
    header_bar.pack_end(&zoom_box);

    // Main content
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.append(&header_bar);
    content.append(&scrolled_window);

    // Window
    let window = AdwApplicationWindow::builder()
        .application(app)
        .title("Winux Image Viewer")
        .default_width(800)
        .default_height(600)
        .content(&content)
        .build();

    // Open button handler
    {
        let window_clone = window.clone();
        let picture_clone = picture.clone();
        let current_path_clone = current_path.clone();
        let current_files_clone = current_files.clone();
        let current_index_clone = current_index.clone();
        let zoom_level_clone = zoom_level.clone();

        open_button.connect_clicked(move |_| {
            let file_dialog = gtk::FileDialog::builder()
                .title("Open Image")
                .modal(true)
                .build();

            let filter = gtk::FileFilter::new();
            filter.add_mime_type("image/*");
            filter.set_name(Some("Images"));

            let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            file_dialog.set_filters(Some(&filters));

            let picture = picture_clone.clone();
            let current_path = current_path_clone.clone();
            let current_files = current_files_clone.clone();
            let current_index = current_index_clone.clone();
            let zoom_level = zoom_level_clone.clone();

            file_dialog.open(Some(&window_clone), gtk::gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        load_image(&picture, &path, &zoom_level);
                        update_file_list(&path, &current_path, &current_files, &current_index);
                    }
                }
            });
        });
    }

    // Zoom in handler
    {
        let picture_clone = picture.clone();
        let zoom_level_clone = zoom_level.clone();

        zoom_in_button.connect_clicked(move |_| {
            let mut level = zoom_level_clone.borrow_mut();
            *level = (*level * 1.25).min(5.0);
            apply_zoom(&picture_clone, *level);
        });
    }

    // Zoom out handler
    {
        let picture_clone = picture.clone();
        let zoom_level_clone = zoom_level.clone();

        zoom_out_button.connect_clicked(move |_| {
            let mut level = zoom_level_clone.borrow_mut();
            *level = (*level / 1.25).max(0.1);
            apply_zoom(&picture_clone, *level);
        });
    }

    // Zoom reset handler
    {
        let picture_clone = picture.clone();
        let zoom_level_clone = zoom_level.clone();

        zoom_reset_button.connect_clicked(move |_| {
            *zoom_level_clone.borrow_mut() = 1.0;
            apply_zoom(&picture_clone, 1.0);
        });
    }

    // Previous button handler
    {
        let picture_clone = picture.clone();
        let current_files_clone = current_files.clone();
        let current_index_clone = current_index.clone();
        let zoom_level_clone = zoom_level.clone();

        prev_button.connect_clicked(move |_| {
            let files = current_files_clone.borrow();
            if files.is_empty() {
                return;
            }
            let mut index = current_index_clone.borrow_mut();
            if *index > 0 {
                *index -= 1;
            } else {
                *index = files.len() - 1;
            }
            let path = files[*index].clone();
            drop(index);
            drop(files);
            *zoom_level_clone.borrow_mut() = 1.0;
            load_image(&picture_clone, &path, &zoom_level_clone);
        });
    }

    // Next button handler
    {
        let picture_clone = picture.clone();
        let current_files_clone = current_files.clone();
        let current_index_clone = current_index.clone();
        let zoom_level_clone = zoom_level.clone();

        next_button.connect_clicked(move |_| {
            let files = current_files_clone.borrow();
            if files.is_empty() {
                return;
            }
            let mut index = current_index_clone.borrow_mut();
            *index = (*index + 1) % files.len();
            let path = files[*index].clone();
            drop(index);
            drop(files);
            *zoom_level_clone.borrow_mut() = 1.0;
            load_image(&picture_clone, &path, &zoom_level_clone);
        });
    }

    window.present();
}

fn load_image(picture: &Picture, path: &PathBuf, zoom_level: &Rc<RefCell<f64>>) {
    let file = gtk::gio::File::for_path(path);
    picture.set_file(Some(&file));
    apply_zoom(picture, *zoom_level.borrow());
}

fn apply_zoom(picture: &Picture, level: f64) {
    let width = (800.0 * level) as i32;
    let height = (600.0 * level) as i32;
    picture.set_size_request(width, height);
}

fn update_file_list(
    path: &PathBuf,
    current_path: &Rc<RefCell<Option<PathBuf>>>,
    current_files: &Rc<RefCell<Vec<PathBuf>>>,
    current_index: &Rc<RefCell<usize>>,
) {
    let image_extensions = ["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "svg"];

    if let Some(parent) = path.parent() {
        *current_path.borrow_mut() = Some(path.clone());

        let mut files: Vec<PathBuf> = std::fs::read_dir(parent)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| {
                p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| image_extensions.contains(&ext.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .collect();

        files.sort();

        let index = files.iter().position(|p| p == path).unwrap_or(0);
        *current_files.borrow_mut() = files;
        *current_index.borrow_mut() = index;
    }
}
