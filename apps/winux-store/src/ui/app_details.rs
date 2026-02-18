//! Application details page showing full information about an app

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use std::cell::RefCell;

use crate::backend::AppPackage;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct AppDetailsPage {
        pub package: RefCell<Option<AppPackage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppDetailsPage {
        const NAME: &'static str = "WinuxStoreAppDetailsPage";
        type Type = super::AppDetailsPage;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for AppDetailsPage {}
    impl WidgetImpl for AppDetailsPage {}
    impl BoxImpl for AppDetailsPage {}
}

glib::wrapper! {
    pub struct AppDetailsPage(ObjectSubclass<imp::AppDetailsPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl AppDetailsPage {
    pub fn new(package: &AppPackage) -> Self {
        let page: Self = glib::Object::builder().build();
        page.imp().package.replace(Some(package.clone()));
        page.setup_ui(package);
        page
    }

    fn setup_ui(&self, package: &AppPackage) {
        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(0);

        // Scrolled window for content
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);

        // Header section
        let header = self.create_header(package);
        content.append(&header);

        // Screenshots carousel
        let screenshots = self.create_screenshots_section(package);
        content.append(&screenshots);

        // Description section
        let description = self.create_description_section(package);
        content.append(&description);

        // Details section
        let details = self.create_details_section(package);
        content.append(&details);

        // Reviews section
        let reviews = self.create_reviews_section(package);
        content.append(&reviews);

        scrolled.set_child(Some(&content));
        self.append(&scrolled);

        // Bottom action bar
        let action_bar = self.create_action_bar(package);
        self.append(&action_bar);
    }

    fn create_header(&self, package: &AppPackage) -> gtk4::Box {
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);

        // App icon
        let icon = gtk4::Image::builder()
            .icon_name(&package.icon)
            .pixel_size(128)
            .build();
        icon.add_css_class("icon-dropshadow");
        header.append(&icon);

        // Info box
        let info = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        info.set_valign(gtk4::Align::Center);
        info.set_hexpand(true);

        // App name
        let name_label = gtk4::Label::new(Some(&package.name));
        name_label.add_css_class("title-1");
        name_label.set_halign(gtk4::Align::Start);
        info.append(&name_label);

        // Summary
        let summary_label = gtk4::Label::new(Some(&package.summary));
        summary_label.add_css_class("dim-label");
        summary_label.set_halign(gtk4::Align::Start);
        summary_label.set_wrap(true);
        info.append(&summary_label);

        // Source badge
        let source_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        source_box.set_margin_top(8);

        let source_badge = gtk4::Label::new(Some(&package.source.to_string()));
        source_badge.add_css_class("caption");
        source_badge.add_css_class("accent");
        source_box.append(&source_badge);

        // Version
        let version_label = gtk4::Label::new(Some(&format!("v{}", package.version)));
        version_label.add_css_class("caption");
        version_label.add_css_class("dim-label");
        source_box.append(&version_label);

        // Rating if available
        if let Some(rating) = package.rating {
            let rating_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
            let star = gtk4::Image::from_icon_name("starred-symbolic");
            star.add_css_class("warning");
            rating_box.append(&star);

            let rating_label = gtk4::Label::new(Some(&format!("{:.1}", rating)));
            rating_label.add_css_class("caption");
            rating_box.append(&rating_label);

            let count_label = gtk4::Label::new(Some(&format!("({})", package.rating_count)));
            count_label.add_css_class("caption");
            count_label.add_css_class("dim-label");
            rating_box.append(&count_label);

            source_box.append(&rating_box);
        }

        info.append(&source_box);
        header.append(&info);

        header
    }

    fn create_screenshots_section(&self, package: &AppPackage) -> gtk4::Box {
        let section = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

        let label = gtk4::Label::new(Some("Screenshots"));
        label.add_css_class("title-3");
        label.set_halign(gtk4::Align::Start);
        section.append(&label);

        let carousel = adw::Carousel::new();
        carousel.set_height_request(400);
        carousel.add_css_class("card");

        // Add placeholder screenshots
        for i in 0..3 {
            let placeholder = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            placeholder.set_valign(gtk4::Align::Center);
            placeholder.set_halign(gtk4::Align::Center);
            placeholder.set_hexpand(true);

            let icon = gtk4::Image::from_icon_name("image-missing-symbolic");
            icon.set_pixel_size(64);
            icon.add_css_class("dim-label");
            placeholder.append(&icon);

            let label = gtk4::Label::new(Some(&format!("Screenshot {}", i + 1)));
            label.add_css_class("dim-label");
            placeholder.append(&label);

            carousel.append(&placeholder);
        }

        let indicator = adw::CarouselIndicatorDots::new();
        indicator.set_carousel(Some(&carousel));

        section.append(&carousel);
        section.append(&indicator);

        section
    }

    fn create_description_section(&self, package: &AppPackage) -> gtk4::Box {
        let section = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

        let label = gtk4::Label::new(Some("Description"));
        label.add_css_class("title-3");
        label.set_halign(gtk4::Align::Start);
        section.append(&label);

        let description = if package.description.is_empty() {
            &package.summary
        } else {
            &package.description
        };

        let desc_label = gtk4::Label::new(Some(description));
        desc_label.set_wrap(true);
        desc_label.set_halign(gtk4::Align::Start);
        desc_label.set_xalign(0.0);
        section.append(&desc_label);

        section
    }

    fn create_details_section(&self, package: &AppPackage) -> gtk4::Box {
        let section = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

        let label = gtk4::Label::new(Some("Details"));
        label.add_css_class("title-3");
        label.set_halign(gtk4::Align::Start);
        section.append(&label);

        let details_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        details_box.add_css_class("card");
        details_box.set_margin_top(8);

        let list = gtk4::ListBox::new();
        list.add_css_class("boxed-list");

        // Version
        let version_row = adw::ActionRow::builder()
            .title("Version")
            .subtitle(&package.version)
            .build();
        list.append(&version_row);

        // Size
        let size_str = format_size(package.download_size);
        let size_row = adw::ActionRow::builder()
            .title("Download Size")
            .subtitle(&size_str)
            .build();
        list.append(&size_row);

        // Installed size
        let installed_size_str = format_size(package.installed_size);
        let installed_row = adw::ActionRow::builder()
            .title("Installed Size")
            .subtitle(&installed_size_str)
            .build();
        list.append(&installed_row);

        // License
        if let Some(license) = &package.license {
            let license_row = adw::ActionRow::builder()
                .title("License")
                .subtitle(license)
                .build();
            list.append(&license_row);
        }

        // Homepage
        if let Some(homepage) = &package.homepage {
            let homepage_row = adw::ActionRow::builder()
                .title("Homepage")
                .subtitle(homepage)
                .activatable(true)
                .build();
            homepage_row.add_suffix(&gtk4::Image::from_icon_name("external-link-symbolic"));
            list.append(&homepage_row);
        }

        // Categories
        if !package.categories.is_empty() {
            let categories_str = package.categories.join(", ");
            let categories_row = adw::ActionRow::builder()
                .title("Categories")
                .subtitle(&categories_str)
                .build();
            list.append(&categories_row);
        }

        // Source
        let source_row = adw::ActionRow::builder()
            .title("Source")
            .subtitle(&package.source.to_string())
            .build();
        list.append(&source_row);

        details_box.append(&list);
        section.append(&details_box);

        section
    }

    fn create_reviews_section(&self, package: &AppPackage) -> gtk4::Box {
        let section = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let label = gtk4::Label::new(Some("Reviews"));
        label.add_css_class("title-3");
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);
        header.append(&label);

        let write_review = gtk4::Button::with_label("Write a Review");
        write_review.add_css_class("flat");
        header.append(&write_review);

        section.append(&header);

        // Rating summary
        let rating_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 24);
        rating_box.set_margin_top(8);

        let rating_value = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        rating_value.set_valign(gtk4::Align::Center);

        let rating_num = package.rating.unwrap_or(0.0);
        let rating_label = gtk4::Label::new(Some(&format!("{:.1}", rating_num)));
        rating_label.add_css_class("title-1");
        rating_value.append(&rating_label);

        let stars_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        for i in 0..5 {
            let star = if (i as f32) < rating_num {
                gtk4::Image::from_icon_name("starred-symbolic")
            } else {
                gtk4::Image::from_icon_name("non-starred-symbolic")
            };
            star.add_css_class("warning");
            stars_box.append(&star);
        }
        rating_value.append(&stars_box);

        let count_label = gtk4::Label::new(Some(&format!("{} ratings", package.rating_count)));
        count_label.add_css_class("caption");
        count_label.add_css_class("dim-label");
        rating_value.append(&count_label);

        rating_box.append(&rating_value);

        section.append(&rating_box);

        // Placeholder for reviews
        let placeholder = adw::StatusPage::builder()
            .icon_name("chat-bubble-text-symbolic")
            .title("No Reviews Yet")
            .description("Be the first to review this application")
            .build();
        placeholder.set_height_request(200);

        section.append(&placeholder);

        section
    }

    fn create_action_bar(&self, package: &AppPackage) -> gtk4::Box {
        let bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        bar.add_css_class("toolbar");
        bar.set_margin_start(24);
        bar.set_margin_end(24);
        bar.set_margin_top(12);
        bar.set_margin_bottom(12);

        // Spacer
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        bar.append(&spacer);

        // Install/Uninstall button
        let install_button = match package.status {
            crate::backend::InstallStatus::Installed => {
                let btn = gtk4::Button::with_label("Uninstall");
                btn.add_css_class("destructive-action");
                btn
            }
            crate::backend::InstallStatus::UpdateAvailable => {
                let btn = gtk4::Button::with_label("Update");
                btn.add_css_class("suggested-action");
                btn
            }
            crate::backend::InstallStatus::Installing
            | crate::backend::InstallStatus::Uninstalling
            | crate::backend::InstallStatus::Updating => {
                let btn = gtk4::Button::new();
                let spinner = gtk4::Spinner::new();
                spinner.start();
                btn.set_child(Some(&spinner));
                btn.set_sensitive(false);
                btn
            }
            _ => {
                let btn = gtk4::Button::with_label("Install");
                btn.add_css_class("suggested-action");
                btn
            }
        };
        install_button.add_css_class("pill");
        install_button.set_width_request(120);
        bar.append(&install_button);

        // Open button (if installed)
        if matches!(
            package.status,
            crate::backend::InstallStatus::Installed
                | crate::backend::InstallStatus::UpdateAvailable
        ) {
            let open_button = gtk4::Button::with_label("Open");
            open_button.add_css_class("pill");
            bar.append(&open_button);
        }

        bar
    }
}

/// Format size in bytes to human-readable string
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes > 0 {
        format!("{} B", bytes)
    } else {
        "Unknown".to_string()
    }
}

impl Default for AppDetailsPage {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}
