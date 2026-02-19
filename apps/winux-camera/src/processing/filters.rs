//! Image filters for camera preview and capture

/// Available filter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterType {
    #[default]
    None,
    Grayscale,
    Sepia,
    Vintage,
    Cool,
    Warm,
    HighContrast,
    LowContrast,
    Negative,
    Posterize,
}

impl FilterType {
    pub fn label(&self) -> &'static str {
        match self {
            FilterType::None => "Normal",
            FilterType::Grayscale => "Grayscale",
            FilterType::Sepia => "Sepia",
            FilterType::Vintage => "Vintage",
            FilterType::Cool => "Cool",
            FilterType::Warm => "Warm",
            FilterType::HighContrast => "High Contrast",
            FilterType::LowContrast => "Low Contrast",
            FilterType::Negative => "Negative",
            FilterType::Posterize => "Posterize",
        }
    }

    pub fn all() -> &'static [FilterType] {
        &[
            FilterType::None,
            FilterType::Grayscale,
            FilterType::Sepia,
            FilterType::Vintage,
            FilterType::Cool,
            FilterType::Warm,
            FilterType::HighContrast,
            FilterType::LowContrast,
            FilterType::Negative,
            FilterType::Posterize,
        ]
    }
}

/// Filter preset with custom parameters
#[derive(Debug, Clone)]
pub struct FilterPreset {
    pub name: String,
    pub brightness: f32,     // -1.0 to 1.0
    pub contrast: f32,       // 0.0 to 2.0
    pub saturation: f32,     // 0.0 to 2.0
    pub hue_shift: f32,      // -180 to 180
    pub temperature: f32,    // -1.0 (cool) to 1.0 (warm)
    pub vignette: f32,       // 0.0 to 1.0
}

impl Default for FilterPreset {
    fn default() -> Self {
        Self {
            name: "Custom".to_string(),
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue_shift: 0.0,
            temperature: 0.0,
            vignette: 0.0,
        }
    }
}

/// Apply a filter to RGBA image data
pub fn apply_filter(data: &[u8], width: u32, height: u32, filter: FilterType) -> Vec<u8> {
    let mut result = data.to_vec();

    match filter {
        FilterType::None => {}
        FilterType::Grayscale => apply_grayscale(&mut result),
        FilterType::Sepia => apply_sepia(&mut result),
        FilterType::Vintage => apply_vintage(&mut result),
        FilterType::Cool => apply_temperature(&mut result, -0.3),
        FilterType::Warm => apply_temperature(&mut result, 0.3),
        FilterType::HighContrast => apply_contrast(&mut result, 1.5),
        FilterType::LowContrast => apply_contrast(&mut result, 0.7),
        FilterType::Negative => apply_negative(&mut result),
        FilterType::Posterize => apply_posterize(&mut result, 4),
    }

    // Apply vignette for certain filters
    if matches!(filter, FilterType::Vintage | FilterType::Sepia) {
        apply_vignette(&mut result, width, height, 0.3);
    }

    result
}

/// Convert to grayscale
fn apply_grayscale(data: &mut [u8]) {
    for chunk in data.chunks_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;

        // Luminosity method (better than simple average)
        let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;

        chunk[0] = gray;
        chunk[1] = gray;
        chunk[2] = gray;
        // Alpha unchanged
    }
}

/// Apply sepia tone
fn apply_sepia(data: &mut [u8]) {
    for chunk in data.chunks_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;

        let new_r = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0) as u8;
        let new_g = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0) as u8;
        let new_b = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0) as u8;

        chunk[0] = new_r;
        chunk[1] = new_g;
        chunk[2] = new_b;
    }
}

/// Apply vintage filter (sepia + reduced saturation + slight fade)
fn apply_vintage(data: &mut [u8]) {
    for chunk in data.chunks_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;

        // Desaturate slightly
        let gray = 0.299 * r + 0.587 * g + 0.114 * b;
        let saturation = 0.6;

        let new_r = gray + saturation * (r - gray);
        let new_g = gray + saturation * (g - gray);
        let new_b = gray + saturation * (b - gray);

        // Add warm tint
        let tint_r = (new_r * 1.1 + 10.0).min(255.0) as u8;
        let tint_g = (new_g * 1.0 + 5.0).min(255.0) as u8;
        let tint_b = (new_b * 0.9).min(255.0) as u8;

        chunk[0] = tint_r;
        chunk[1] = tint_g;
        chunk[2] = tint_b;
    }
}

/// Adjust color temperature
fn apply_temperature(data: &mut [u8], temperature: f32) {
    let r_adjust = 1.0 + temperature * 0.2;
    let b_adjust = 1.0 - temperature * 0.2;

    for chunk in data.chunks_mut(4) {
        let r = (chunk[0] as f32 * r_adjust).clamp(0.0, 255.0) as u8;
        let b = (chunk[2] as f32 * b_adjust).clamp(0.0, 255.0) as u8;

        chunk[0] = r;
        chunk[2] = b;
    }
}

/// Adjust contrast
fn apply_contrast(data: &mut [u8], factor: f32) {
    let mid = 128.0;

    for chunk in data.chunks_mut(4) {
        let r = ((chunk[0] as f32 - mid) * factor + mid).clamp(0.0, 255.0) as u8;
        let g = ((chunk[1] as f32 - mid) * factor + mid).clamp(0.0, 255.0) as u8;
        let b = ((chunk[2] as f32 - mid) * factor + mid).clamp(0.0, 255.0) as u8;

        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
    }
}

/// Apply negative/invert
fn apply_negative(data: &mut [u8]) {
    for chunk in data.chunks_mut(4) {
        chunk[0] = 255 - chunk[0];
        chunk[1] = 255 - chunk[1];
        chunk[2] = 255 - chunk[2];
        // Alpha unchanged
    }
}

/// Posterize effect (reduce color levels)
fn apply_posterize(data: &mut [u8], levels: u8) {
    let step = 255 / levels;

    for chunk in data.chunks_mut(4) {
        chunk[0] = (chunk[0] / step) * step;
        chunk[1] = (chunk[1] / step) * step;
        chunk[2] = (chunk[2] / step) * step;
    }
}

/// Apply vignette effect
fn apply_vignette(data: &mut [u8], width: u32, height: u32, strength: f32) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;

            // Smooth falloff
            let factor = 1.0 - (dist * strength).clamp(0.0, 1.0);

            let idx = ((y * width + x) * 4) as usize;
            data[idx] = (data[idx] as f32 * factor) as u8;
            data[idx + 1] = (data[idx + 1] as f32 * factor) as u8;
            data[idx + 2] = (data[idx + 2] as f32 * factor) as u8;
        }
    }
}

/// Adjust brightness
pub fn apply_brightness(data: &mut [u8], brightness: f32) {
    let offset = (brightness * 255.0) as i32;

    for chunk in data.chunks_mut(4) {
        chunk[0] = (chunk[0] as i32 + offset).clamp(0, 255) as u8;
        chunk[1] = (chunk[1] as i32 + offset).clamp(0, 255) as u8;
        chunk[2] = (chunk[2] as i32 + offset).clamp(0, 255) as u8;
    }
}

/// Adjust saturation
pub fn apply_saturation(data: &mut [u8], saturation: f32) {
    for chunk in data.chunks_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;

        let gray = 0.299 * r + 0.587 * g + 0.114 * b;

        chunk[0] = (gray + saturation * (r - gray)).clamp(0.0, 255.0) as u8;
        chunk[1] = (gray + saturation * (g - gray)).clamp(0.0, 255.0) as u8;
        chunk[2] = (gray + saturation * (b - gray)).clamp(0.0, 255.0) as u8;
    }
}

/// Gamma correction
pub fn apply_gamma(data: &mut [u8], gamma: f32) {
    let inv_gamma = 1.0 / gamma;

    for chunk in data.chunks_mut(4) {
        chunk[0] = ((chunk[0] as f32 / 255.0).powf(inv_gamma) * 255.0) as u8;
        chunk[1] = ((chunk[1] as f32 / 255.0).powf(inv_gamma) * 255.0) as u8;
        chunk[2] = ((chunk[2] as f32 / 255.0).powf(inv_gamma) * 255.0) as u8;
    }
}
