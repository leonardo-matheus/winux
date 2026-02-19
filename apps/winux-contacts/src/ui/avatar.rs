// Avatar Helper - Creates contact avatars with photos or initials

use crate::data::Contact;
use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

pub struct AvatarHelper;

impl AvatarHelper {
    /// Create an avatar widget for a contact
    pub fn create_avatar(contact: &Contact, size: i32) -> adw::Avatar {
        let avatar = adw::Avatar::new(size, Some(&contact.display_name()), true);

        // Try to load photo if available
        if let Some(data) = &contact.avatar_data {
            if let Ok(texture) = Self::load_texture_from_data(data) {
                // Avatar with custom image would be set here
                // libadwaita::Avatar doesn't directly support custom textures
                // In a real app, you'd use a custom widget or Paintable
            }
        } else if let Some(uri) = &contact.avatar_uri {
            // Could load from URI asynchronously
        }

        avatar
    }

    /// Create an avatar with custom text (initials)
    pub fn create_avatar_with_initials(initials: &str, size: i32) -> adw::Avatar {
        adw::Avatar::new(size, Some(initials), true)
    }

    /// Load a GDK texture from image data
    fn load_texture_from_data(data: &[u8]) -> Result<gdk4::Texture, Box<dyn std::error::Error>> {
        let bytes = glib::Bytes::from(data);
        let texture = gdk4::Texture::from_bytes(&bytes)?;
        Ok(texture)
    }

    /// Generate a consistent color for a contact based on their name
    pub fn get_color_for_name(name: &str) -> &'static str {
        let colors = [
            "#e74c3c", // Red
            "#e91e63", // Pink
            "#9c27b0", // Purple
            "#673ab7", // Deep Purple
            "#3f51b5", // Indigo
            "#2196f3", // Blue
            "#03a9f4", // Light Blue
            "#00bcd4", // Cyan
            "#009688", // Teal
            "#4caf50", // Green
            "#8bc34a", // Light Green
            "#cddc39", // Lime
            "#ffeb3b", // Yellow
            "#ffc107", // Amber
            "#ff9800", // Orange
            "#ff5722", // Deep Orange
        ];

        let hash: usize = name.bytes().map(|b| b as usize).sum();
        colors[hash % colors.len()]
    }

    /// Create a simple circular avatar with initials (for use where adw::Avatar is not available)
    pub fn create_simple_avatar(contact: &Contact, size: i32) -> gtk4::DrawingArea {
        let initials = contact.initials();
        let color = Self::get_color_for_name(&contact.display_name());

        let area = gtk4::DrawingArea::new();
        area.set_size_request(size, size);
        area.set_halign(gtk4::Align::Center);
        area.set_valign(gtk4::Align::Center);

        area.set_draw_func({
            let initials = initials.clone();
            let color = color.to_string();
            move |_, cr, width, height| {
                let radius = width.min(height) as f64 / 2.0;
                let center_x = width as f64 / 2.0;
                let center_y = height as f64 / 2.0;

                // Parse color
                let (r, g, b) = Self::parse_hex_color(&color);

                // Draw circle
                cr.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
                cr.set_source_rgb(r, g, b);
                let _ = cr.fill();

                // Draw text
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
                cr.set_font_size(radius * 0.8);

                let extents = cr.text_extents(&initials).unwrap();
                let x = center_x - extents.width() / 2.0 - extents.x_bearing();
                let y = center_y - extents.height() / 2.0 - extents.y_bearing();

                cr.move_to(x, y);
                let _ = cr.show_text(&initials);
            }
        });

        area
    }

    fn parse_hex_color(hex: &str) -> (f64, f64, f64) {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f64 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f64 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f64 / 255.0;
            (r, g, b)
        } else {
            (0.5, 0.5, 0.5)
        }
    }
}
