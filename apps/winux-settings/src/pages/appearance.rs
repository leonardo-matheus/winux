//! Appearance settings page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::info;

/// Appearance settings page
pub struct AppearancePage {
    widget: adw::PreferencesPage,
}

impl AppearancePage {
    /// Create a new appearance settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Appearance");
        page.set_icon_name(Some("applications-graphics-symbolic"));

        // Style group
        let style_group = adw::PreferencesGroup::new();
        style_group.set_title("Style");

        // Color scheme
        let scheme_row = adw::ActionRow::new();
        scheme_row.set_title("Color Scheme");

        let scheme_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        scheme_box.set_halign(gtk4::Align::End);

        // Light option
        let light_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        let light_preview = gtk4::DrawingArea::new();
        light_preview.set_size_request(60, 40);
        light_preview.add_css_class("card");
        light_preview.set_draw_func(|_, cr, w, h| {
            cr.set_source_rgb(0.95, 0.95, 0.95);
            cr.rectangle(0.0, 0.0, w as f64, h as f64);
            let _ = cr.fill();
        });
        let light_label = gtk4::Label::new(Some("Light"));
        light_box.append(&light_preview);
        light_box.append(&light_label);

        // Dark option
        let dark_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        let dark_preview = gtk4::DrawingArea::new();
        dark_preview.set_size_request(60, 40);
        dark_preview.add_css_class("card");
        dark_preview.set_draw_func(|_, cr, w, h| {
            cr.set_source_rgb(0.15, 0.15, 0.2);
            cr.rectangle(0.0, 0.0, w as f64, h as f64);
            let _ = cr.fill();
        });
        let dark_label = gtk4::Label::new(Some("Dark"));
        dark_box.append(&dark_preview);
        dark_box.append(&dark_label);

        // Auto option
        let auto_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        let auto_preview = gtk4::DrawingArea::new();
        auto_preview.set_size_request(60, 40);
        auto_preview.add_css_class("card");
        auto_preview.set_draw_func(|_, cr, w, h| {
            // Half light, half dark
            cr.set_source_rgb(0.95, 0.95, 0.95);
            cr.rectangle(0.0, 0.0, w as f64 / 2.0, h as f64);
            let _ = cr.fill();
            cr.set_source_rgb(0.15, 0.15, 0.2);
            cr.rectangle(w as f64 / 2.0, 0.0, w as f64 / 2.0, h as f64);
            let _ = cr.fill();
        });
        let auto_label = gtk4::Label::new(Some("Auto"));
        auto_box.append(&auto_preview);
        auto_box.append(&auto_label);

        scheme_box.append(&light_box);
        scheme_box.append(&dark_box);
        scheme_box.append(&auto_box);
        scheme_row.add_suffix(&scheme_box);
        style_group.add(&scheme_row);

        page.add(&style_group);

        // Accent color group
        let accent_group = adw::PreferencesGroup::new();
        accent_group.set_title("Accent Color");
        accent_group.set_description(Some("Choose an accent color for buttons and highlights"));

        let accent_row = adw::ActionRow::new();

        let colors_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        colors_box.set_halign(gtk4::Align::End);

        let accent_colors = [
            ("#3584e4", "Blue"),
            ("#33d17a", "Green"),
            ("#f6d32d", "Yellow"),
            ("#ff7800", "Orange"),
            ("#e01b24", "Red"),
            ("#9141ac", "Purple"),
            ("#63452c", "Brown"),
            ("#5e5c64", "Gray"),
        ];

        for (color, name) in accent_colors {
            let btn = gtk4::Button::new();
            btn.set_size_request(32, 32);
            btn.set_tooltip_text(Some(name));
            btn.add_css_class("circular");

            let css_provider = gtk4::CssProvider::new();
            css_provider.load_from_data(&format!(
                "button {{ background-color: {}; min-width: 32px; min-height: 32px; }}",
                color
            ));
            btn.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            colors_box.append(&btn);
        }

        accent_row.add_suffix(&colors_box);
        accent_group.add(&accent_row);

        page.add(&accent_group);

        // Wallpaper group
        let wallpaper_group = adw::PreferencesGroup::new();
        wallpaper_group.set_title("Wallpaper");

        // Current wallpaper preview
        let wallpaper_row = adw::ActionRow::new();
        wallpaper_row.set_title("Desktop Wallpaper");
        wallpaper_row.set_subtitle("Click to change");
        wallpaper_row.set_activatable(true);

        let preview = gtk4::Image::from_icon_name("image-x-generic-symbolic");
        preview.set_pixel_size(64);
        wallpaper_row.add_prefix(&preview);
        wallpaper_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        wallpaper_group.add(&wallpaper_row);

        // Wallpaper fit
        let fit_row = adw::ComboRow::new();
        fit_row.set_title("Fit");
        let fits = gtk4::StringList::new(&[
            "Fill",
            "Fit",
            "Stretch",
            "Tile",
            "Center",
            "Span",
        ]);
        fit_row.set_model(Some(&fits));
        wallpaper_group.add(&fit_row);

        page.add(&wallpaper_group);

        // Fonts group
        let fonts_group = adw::PreferencesGroup::new();
        fonts_group.set_title("Fonts");

        // Interface font
        let interface_font = adw::ActionRow::new();
        interface_font.set_title("Interface Font");
        interface_font.set_subtitle("Cantarell 11");
        interface_font.set_activatable(true);
        interface_font.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        fonts_group.add(&interface_font);

        // Document font
        let doc_font = adw::ActionRow::new();
        doc_font.set_title("Document Font");
        doc_font.set_subtitle("Cantarell 11");
        doc_font.set_activatable(true);
        doc_font.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        fonts_group.add(&doc_font);

        // Monospace font
        let mono_font = adw::ActionRow::new();
        mono_font.set_title("Monospace Font");
        mono_font.set_subtitle("JetBrains Mono 10");
        mono_font.set_activatable(true);
        mono_font.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        fonts_group.add(&mono_font);

        // Font scaling
        let font_scale = adw::ActionRow::new();
        font_scale.set_title("Font Scaling");

        let scale_slider = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.5, 2.0, 0.1);
        scale_slider.set_value(1.0);
        scale_slider.set_draw_value(true);
        scale_slider.set_width_request(200);
        scale_slider.add_mark(1.0, gtk4::PositionType::Bottom, Some("100%"));
        font_scale.add_suffix(&scale_slider);
        fonts_group.add(&font_scale);

        // Hinting
        let hinting_row = adw::ComboRow::new();
        hinting_row.set_title("Font Hinting");
        let hinting = gtk4::StringList::new(&[
            "None",
            "Slight",
            "Medium",
            "Full",
        ]);
        hinting_row.set_model(Some(&hinting));
        hinting_row.set_selected(1);
        fonts_group.add(&hinting_row);

        // Antialiasing
        let aa_row = adw::ComboRow::new();
        aa_row.set_title("Antialiasing");
        let aa = gtk4::StringList::new(&[
            "None",
            "Grayscale",
            "Subpixel (LCD)",
        ]);
        aa_row.set_model(Some(&aa));
        aa_row.set_selected(2);
        fonts_group.add(&aa_row);

        page.add(&fonts_group);

        // Interface group
        let interface_group = adw::PreferencesGroup::new();
        interface_group.set_title("Interface");

        // Animations
        let animations = adw::SwitchRow::new();
        animations.set_title("Animations");
        animations.set_subtitle("Enable interface animations");
        animations.set_active(true);
        interface_group.add(&animations);

        // Transparency
        let transparency = adw::SwitchRow::new();
        transparency.set_title("Transparency Effects");
        transparency.set_subtitle("Enable window transparency and blur");
        transparency.set_active(true);
        interface_group.add(&transparency);

        // Rounded corners
        let corners = adw::ActionRow::new();
        corners.set_title("Window Corners");

        let corners_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 24.0, 1.0);
        corners_scale.set_value(12.0);
        corners_scale.set_draw_value(true);
        corners_scale.set_width_request(150);
        corners.add_suffix(&corners_scale);
        interface_group.add(&corners);

        page.add(&interface_group);

        // Icons group
        let icons_group = adw::PreferencesGroup::new();
        icons_group.set_title("Icons");

        // Icon theme
        let icon_theme = adw::ComboRow::new();
        icon_theme.set_title("Icon Theme");
        let themes = gtk4::StringList::new(&[
            "Winux",
            "Adwaita",
            "Papirus",
            "Numix",
        ]);
        icon_theme.set_model(Some(&themes));
        icons_group.add(&icon_theme);

        // Cursor theme
        let cursor_theme = adw::ComboRow::new();
        cursor_theme.set_title("Cursor Theme");
        let cursors = gtk4::StringList::new(&[
            "Winux",
            "Adwaita",
            "DMZ-White",
            "DMZ-Black",
        ]);
        cursor_theme.set_model(Some(&cursors));
        icons_group.add(&cursor_theme);

        // Cursor size
        let cursor_size = adw::ComboRow::new();
        cursor_size.set_title("Cursor Size");
        let sizes = gtk4::StringList::new(&[
            "Small (24)",
            "Default (32)",
            "Large (48)",
            "Extra Large (64)",
        ]);
        cursor_size.set_model(Some(&sizes));
        cursor_size.set_selected(1);
        icons_group.add(&cursor_size);

        page.add(&icons_group);

        AppearancePage { widget: page }
    }

    /// Get the page widget
    pub fn widget(&self) -> &adw::PreferencesPage {
        &self.widget
    }
}

impl Default for AppearancePage {
    fn default() -> Self {
        Self::new()
    }
}
