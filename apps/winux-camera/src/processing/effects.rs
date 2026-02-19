//! Advanced image effects for camera

/// Available effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Effect {
    #[default]
    None,
    Blur,
    Sharpen,
    EdgeDetect,
    Emboss,
    GaussianBlur,
    MotionBlur,
    Noise,
    Pixelate,
}

impl Effect {
    pub fn label(&self) -> &'static str {
        match self {
            Effect::None => "None",
            Effect::Blur => "Blur",
            Effect::Sharpen => "Sharpen",
            Effect::EdgeDetect => "Edge Detect",
            Effect::Emboss => "Emboss",
            Effect::GaussianBlur => "Gaussian Blur",
            Effect::MotionBlur => "Motion Blur",
            Effect::Noise => "Add Noise",
            Effect::Pixelate => "Pixelate",
        }
    }

    pub fn all() -> &'static [Effect] {
        &[
            Effect::None,
            Effect::Blur,
            Effect::Sharpen,
            Effect::EdgeDetect,
            Effect::Emboss,
            Effect::GaussianBlur,
            Effect::MotionBlur,
            Effect::Noise,
            Effect::Pixelate,
        ]
    }
}

/// Effect parameters
#[derive(Debug, Clone)]
pub struct EffectParameters {
    pub intensity: f32,      // 0.0 to 1.0
    pub radius: u32,         // For blur effects
    pub angle: f32,          // For motion blur (radians)
    pub block_size: u32,     // For pixelate
}

impl Default for EffectParameters {
    fn default() -> Self {
        Self {
            intensity: 0.5,
            radius: 3,
            angle: 0.0,
            block_size: 8,
        }
    }
}

/// Apply an effect to RGBA image data
pub fn apply_effect(
    data: &[u8],
    width: u32,
    height: u32,
    effect: Effect,
    params: &EffectParameters,
) -> Vec<u8> {
    match effect {
        Effect::None => data.to_vec(),
        Effect::Blur => apply_box_blur(data, width, height, params.radius),
        Effect::Sharpen => apply_sharpen(data, width, height, params.intensity),
        Effect::EdgeDetect => apply_edge_detect(data, width, height),
        Effect::Emboss => apply_emboss(data, width, height),
        Effect::GaussianBlur => apply_gaussian_blur(data, width, height, params.radius),
        Effect::MotionBlur => apply_motion_blur(data, width, height, params.radius, params.angle),
        Effect::Noise => apply_noise(data, params.intensity),
        Effect::Pixelate => apply_pixelate(data, width, height, params.block_size),
    }
}

/// Simple box blur
fn apply_box_blur(data: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
    let mut result = data.to_vec();
    let radius = radius as i32;
    let kernel_size = (2 * radius + 1) * (2 * radius + 1);

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let mut r_sum: i32 = 0;
            let mut g_sum: i32 = 0;
            let mut b_sum: i32 = 0;

            for ky in -radius..=radius {
                for kx in -radius..=radius {
                    let px = (x + kx).clamp(0, width as i32 - 1) as u32;
                    let py = (y + ky).clamp(0, height as i32 - 1) as u32;
                    let idx = ((py * width + px) * 4) as usize;

                    r_sum += data[idx] as i32;
                    g_sum += data[idx + 1] as i32;
                    b_sum += data[idx + 2] as i32;
                }
            }

            let idx = ((y as u32 * width + x as u32) * 4) as usize;
            result[idx] = (r_sum / kernel_size) as u8;
            result[idx + 1] = (g_sum / kernel_size) as u8;
            result[idx + 2] = (b_sum / kernel_size) as u8;
        }
    }

    result
}

/// Sharpen using unsharp mask
fn apply_sharpen(data: &[u8], width: u32, height: u32, intensity: f32) -> Vec<u8> {
    // First apply a blur
    let blurred = apply_box_blur(data, width, height, 1);
    let mut result = data.to_vec();

    // Subtract blur and add to original (unsharp mask)
    for i in (0..data.len()).step_by(4) {
        let diff_r = data[i] as f32 - blurred[i] as f32;
        let diff_g = data[i + 1] as f32 - blurred[i + 1] as f32;
        let diff_b = data[i + 2] as f32 - blurred[i + 2] as f32;

        result[i] = (data[i] as f32 + diff_r * intensity * 2.0).clamp(0.0, 255.0) as u8;
        result[i + 1] = (data[i + 1] as f32 + diff_g * intensity * 2.0).clamp(0.0, 255.0) as u8;
        result[i + 2] = (data[i + 2] as f32 + diff_b * intensity * 2.0).clamp(0.0, 255.0) as u8;
    }

    result
}

/// Sobel edge detection
fn apply_edge_detect(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut result = vec![0u8; data.len()];

    // Sobel kernels
    let gx: [[i32; 3]; 3] = [
        [-1, 0, 1],
        [-2, 0, 2],
        [-1, 0, 1],
    ];
    let gy: [[i32; 3]; 3] = [
        [-1, -2, -1],
        [0, 0, 0],
        [1, 2, 1],
    ];

    for y in 1..height as i32 - 1 {
        for x in 1..width as i32 - 1 {
            let mut sum_x = 0i32;
            let mut sum_y = 0i32;

            for ky in -1..=1 {
                for kx in -1..=1 {
                    let px = (x + kx) as u32;
                    let py = (y + ky) as u32;
                    let idx = ((py * width + px) * 4) as usize;

                    // Convert to grayscale
                    let gray = (data[idx] as i32 + data[idx + 1] as i32 + data[idx + 2] as i32) / 3;

                    sum_x += gray * gx[(ky + 1) as usize][(kx + 1) as usize];
                    sum_y += gray * gy[(ky + 1) as usize][(kx + 1) as usize];
                }
            }

            let magnitude = ((sum_x * sum_x + sum_y * sum_y) as f32).sqrt() as u8;

            let idx = ((y as u32 * width + x as u32) * 4) as usize;
            result[idx] = magnitude;
            result[idx + 1] = magnitude;
            result[idx + 2] = magnitude;
            result[idx + 3] = data[idx + 3];
        }
    }

    result
}

/// Emboss effect
fn apply_emboss(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut result = vec![128u8; data.len()];

    // Emboss kernel
    let kernel: [[i32; 3]; 3] = [
        [-2, -1, 0],
        [-1, 1, 1],
        [0, 1, 2],
    ];

    for y in 1..height as i32 - 1 {
        for x in 1..width as i32 - 1 {
            let mut sum = 0i32;

            for ky in -1..=1 {
                for kx in -1..=1 {
                    let px = (x + kx) as u32;
                    let py = (y + ky) as u32;
                    let idx = ((py * width + px) * 4) as usize;

                    let gray = (data[idx] as i32 + data[idx + 1] as i32 + data[idx + 2] as i32) / 3;
                    sum += gray * kernel[(ky + 1) as usize][(kx + 1) as usize];
                }
            }

            let val = (sum + 128).clamp(0, 255) as u8;

            let idx = ((y as u32 * width + x as u32) * 4) as usize;
            result[idx] = val;
            result[idx + 1] = val;
            result[idx + 2] = val;
            result[idx + 3] = data[idx + 3];
        }
    }

    result
}

/// Gaussian blur (approximation using multiple box blurs)
fn apply_gaussian_blur(data: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
    // Three passes of box blur approximates Gaussian
    let pass1 = apply_box_blur(data, width, height, radius);
    let pass2 = apply_box_blur(&pass1, width, height, radius);
    apply_box_blur(&pass2, width, height, radius)
}

/// Motion blur in a specific direction
fn apply_motion_blur(data: &[u8], width: u32, height: u32, radius: u32, angle: f32) -> Vec<u8> {
    let mut result = data.to_vec();
    let radius = radius as i32;

    let dx = angle.cos();
    let dy = angle.sin();

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let mut r_sum: i32 = 0;
            let mut g_sum: i32 = 0;
            let mut b_sum: i32 = 0;
            let mut count = 0;

            for i in -radius..=radius {
                let px = (x as f32 + i as f32 * dx) as i32;
                let py = (y as f32 + i as f32 * dy) as i32;

                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    let idx = ((py as u32 * width + px as u32) * 4) as usize;
                    r_sum += data[idx] as i32;
                    g_sum += data[idx + 1] as i32;
                    b_sum += data[idx + 2] as i32;
                    count += 1;
                }
            }

            let idx = ((y as u32 * width + x as u32) * 4) as usize;
            if count > 0 {
                result[idx] = (r_sum / count) as u8;
                result[idx + 1] = (g_sum / count) as u8;
                result[idx + 2] = (b_sum / count) as u8;
            }
        }
    }

    result
}

/// Add random noise
fn apply_noise(data: &[u8], intensity: f32) -> Vec<u8> {
    let mut result = data.to_vec();
    let noise_range = (intensity * 50.0) as i32;

    // Simple pseudo-random noise using pixel values
    for (i, chunk) in result.chunks_mut(4).enumerate() {
        let noise = ((i * 7919 + (chunk[0] as usize * 104729)) % (2 * noise_range as usize + 1)) as i32 - noise_range;

        chunk[0] = (chunk[0] as i32 + noise).clamp(0, 255) as u8;
        chunk[1] = (chunk[1] as i32 + noise).clamp(0, 255) as u8;
        chunk[2] = (chunk[2] as i32 + noise).clamp(0, 255) as u8;
    }

    result
}

/// Pixelate effect
fn apply_pixelate(data: &[u8], width: u32, height: u32, block_size: u32) -> Vec<u8> {
    let mut result = data.to_vec();
    let block_size = block_size.max(1);

    for by in (0..height).step_by(block_size as usize) {
        for bx in (0..width).step_by(block_size as usize) {
            // Calculate average color for block
            let mut r_sum: u32 = 0;
            let mut g_sum: u32 = 0;
            let mut b_sum: u32 = 0;
            let mut count: u32 = 0;

            for y in by..(by + block_size).min(height) {
                for x in bx..(bx + block_size).min(width) {
                    let idx = ((y * width + x) * 4) as usize;
                    r_sum += data[idx] as u32;
                    g_sum += data[idx + 1] as u32;
                    b_sum += data[idx + 2] as u32;
                    count += 1;
                }
            }

            let avg_r = (r_sum / count) as u8;
            let avg_g = (g_sum / count) as u8;
            let avg_b = (b_sum / count) as u8;

            // Fill block with average color
            for y in by..(by + block_size).min(height) {
                for x in bx..(bx + block_size).min(width) {
                    let idx = ((y * width + x) * 4) as usize;
                    result[idx] = avg_r;
                    result[idx + 1] = avg_g;
                    result[idx + 2] = avg_b;
                }
            }
        }
    }

    result
}
