//! Blur and pixelate effects for hiding sensitive information

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

/// Blur effect for hiding sensitive information
pub struct BlurEffect;

impl BlurEffect {
    /// Apply a blur effect to a region of an image
    pub fn apply(
        image: &DynamicImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        strength: u32,
    ) -> DynamicImage {
        let mut result = image.clone();

        // Clamp region to image bounds
        let img_width = image.width();
        let img_height = image.height();

        let x = x.min(img_width);
        let y = y.min(img_height);
        let width = width.min(img_width - x);
        let height = height.min(img_height - y);

        if width == 0 || height == 0 {
            return result;
        }

        // Extract the region
        let region = image.crop_imm(x, y, width, height);

        // Apply box blur
        let blurred = Self::box_blur(&region, strength);

        // Copy back to result
        if let Some(rgba) = result.as_mut_rgba8() {
            for py in 0..height {
                for px in 0..width {
                    let pixel = blurred.get_pixel(px, py);
                    let dest_x = x + px;
                    let dest_y = y + py;
                    if dest_x < img_width && dest_y < img_height {
                        rgba.put_pixel(dest_x, dest_y, *pixel);
                    }
                }
            }
        }

        result
    }

    /// Simple box blur implementation
    fn box_blur(image: &DynamicImage, radius: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();

        if radius == 0 || width == 0 || height == 0 {
            return rgba;
        }

        let radius = radius as i32;
        let kernel_size = (radius * 2 + 1) as f32;

        let mut result = ImageBuffer::new(width, height);

        // Horizontal pass
        let mut temp = ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                let mut count = 0u32;

                for kx in -radius..=radius {
                    let src_x = (x as i32 + kx).clamp(0, width as i32 - 1) as u32;
                    let pixel = rgba.get_pixel(src_x, y);
                    r_sum += pixel[0] as u32;
                    g_sum += pixel[1] as u32;
                    b_sum += pixel[2] as u32;
                    a_sum += pixel[3] as u32;
                    count += 1;
                }

                temp.put_pixel(x, y, Rgba([
                    (r_sum / count) as u8,
                    (g_sum / count) as u8,
                    (b_sum / count) as u8,
                    (a_sum / count) as u8,
                ]));
            }
        }

        // Vertical pass
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                let mut count = 0u32;

                for ky in -radius..=radius {
                    let src_y = (y as i32 + ky).clamp(0, height as i32 - 1) as u32;
                    let pixel = temp.get_pixel(x, src_y);
                    r_sum += pixel[0] as u32;
                    g_sum += pixel[1] as u32;
                    b_sum += pixel[2] as u32;
                    a_sum += pixel[3] as u32;
                    count += 1;
                }

                result.put_pixel(x, y, Rgba([
                    (r_sum / count) as u8,
                    (g_sum / count) as u8,
                    (b_sum / count) as u8,
                    (a_sum / count) as u8,
                ]));
            }
        }

        result
    }

    /// Apply pixelation effect to a region
    pub fn pixelate(
        image: &DynamicImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        block_size: u32,
    ) -> DynamicImage {
        let mut result = image.clone();

        let img_width = image.width();
        let img_height = image.height();

        let x = x.min(img_width);
        let y = y.min(img_height);
        let width = width.min(img_width - x);
        let height = height.min(img_height - y);

        if width == 0 || height == 0 || block_size == 0 {
            return result;
        }

        let block_size = block_size.max(2);

        if let Some(rgba) = result.as_mut_rgba8() {
            // Process in blocks
            let mut by = y;
            while by < y + height {
                let block_height = block_size.min(y + height - by);

                let mut bx = x;
                while bx < x + width {
                    let block_width = block_size.min(x + width - bx);

                    // Calculate average color in block
                    let mut r_sum = 0u64;
                    let mut g_sum = 0u64;
                    let mut b_sum = 0u64;
                    let mut a_sum = 0u64;
                    let mut count = 0u64;

                    for py in by..(by + block_height) {
                        for px in bx..(bx + block_width) {
                            if px < img_width && py < img_height {
                                let pixel = image.get_pixel(px, py);
                                r_sum += pixel[0] as u64;
                                g_sum += pixel[1] as u64;
                                b_sum += pixel[2] as u64;
                                a_sum += pixel[3] as u64;
                                count += 1;
                            }
                        }
                    }

                    if count > 0 {
                        let avg_pixel = Rgba([
                            (r_sum / count) as u8,
                            (g_sum / count) as u8,
                            (b_sum / count) as u8,
                            (a_sum / count) as u8,
                        ]);

                        // Fill block with average color
                        for py in by..(by + block_height) {
                            for px in bx..(bx + block_width) {
                                if px < img_width && py < img_height {
                                    rgba.put_pixel(px, py, avg_pixel);
                                }
                            }
                        }
                    }

                    bx += block_size;
                }

                by += block_size;
            }
        }

        result
    }
}

/// Gaussian blur for higher quality (slower)
pub struct GaussianBlur;

impl GaussianBlur {
    /// Generate a 1D Gaussian kernel
    fn generate_kernel(radius: u32) -> Vec<f32> {
        let sigma = radius as f32 / 3.0;
        let size = (radius * 2 + 1) as usize;
        let mut kernel = vec![0.0f32; size];
        let mut sum = 0.0f32;

        for i in 0..size {
            let x = i as f32 - radius as f32;
            let value = (-x * x / (2.0 * sigma * sigma)).exp();
            kernel[i] = value;
            sum += value;
        }

        // Normalize
        for value in &mut kernel {
            *value /= sum;
        }

        kernel
    }

    /// Apply Gaussian blur to an image
    pub fn apply(
        image: &DynamicImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        radius: u32,
    ) -> DynamicImage {
        let mut result = image.clone();

        let img_width = image.width();
        let img_height = image.height();

        let x = x.min(img_width);
        let y = y.min(img_height);
        let width = width.min(img_width - x);
        let height = height.min(img_height - y);

        if width == 0 || height == 0 || radius == 0 {
            return result;
        }

        let kernel = Self::generate_kernel(radius);
        let radius = radius as i32;

        // Extract region
        let region = image.crop_imm(x, y, width, height).to_rgba8();

        // Horizontal pass
        let mut temp = ImageBuffer::new(width, height);
        for py in 0..height {
            for px in 0..width {
                let mut r = 0.0f32;
                let mut g = 0.0f32;
                let mut b = 0.0f32;
                let mut a = 0.0f32;

                for (ki, weight) in kernel.iter().enumerate() {
                    let kx = ki as i32 - radius;
                    let src_x = (px as i32 + kx).clamp(0, width as i32 - 1) as u32;
                    let pixel = region.get_pixel(src_x, py);
                    r += pixel[0] as f32 * weight;
                    g += pixel[1] as f32 * weight;
                    b += pixel[2] as f32 * weight;
                    a += pixel[3] as f32 * weight;
                }

                temp.put_pixel(px, py, Rgba([
                    r.clamp(0.0, 255.0) as u8,
                    g.clamp(0.0, 255.0) as u8,
                    b.clamp(0.0, 255.0) as u8,
                    a.clamp(0.0, 255.0) as u8,
                ]));
            }
        }

        // Vertical pass
        let mut blurred = ImageBuffer::new(width, height);
        for py in 0..height {
            for px in 0..width {
                let mut r = 0.0f32;
                let mut g = 0.0f32;
                let mut b = 0.0f32;
                let mut a = 0.0f32;

                for (ki, weight) in kernel.iter().enumerate() {
                    let ky = ki as i32 - radius;
                    let src_y = (py as i32 + ky).clamp(0, height as i32 - 1) as u32;
                    let pixel = temp.get_pixel(px, src_y);
                    r += pixel[0] as f32 * weight;
                    g += pixel[1] as f32 * weight;
                    b += pixel[2] as f32 * weight;
                    a += pixel[3] as f32 * weight;
                }

                blurred.put_pixel(px, py, Rgba([
                    r.clamp(0.0, 255.0) as u8,
                    g.clamp(0.0, 255.0) as u8,
                    b.clamp(0.0, 255.0) as u8,
                    a.clamp(0.0, 255.0) as u8,
                ]));
            }
        }

        // Copy back to result
        if let Some(rgba) = result.as_mut_rgba8() {
            for py in 0..height {
                for px in 0..width {
                    let pixel = blurred.get_pixel(px, py);
                    let dest_x = x + px;
                    let dest_y = y + py;
                    if dest_x < img_width && dest_y < img_height {
                        rgba.put_pixel(dest_x, dest_y, *pixel);
                    }
                }
            }
        }

        result
    }
}
