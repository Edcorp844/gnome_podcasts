use image::{GenericImageView, ImageReader};
use std::io::Cursor;

use crate::util::color::RGBAColor;

pub struct GradientColorExtractor;

impl GradientColorExtractor {
    /// Extracts a specified number of average gradient colors from a raw image byte buffer
    pub fn extract_from_bytes(bytes: &[u8], color_count: u32) -> Vec<RGBAColor> {
        if bytes.is_empty() || color_count == 0 {
            return Vec::new();
        }

        // 1. Load and guess image format from the byte array stream
        let Ok(reader) = ImageReader::new(Cursor::new(bytes)).with_guessed_format() else {
            return Vec::new();
        };

        // 2. Decode the raw vector into memory
        let Ok(img) = reader.decode() else {
            return Vec::new();
        };

        let (width, height) = img.dimensions();
        if height == 0 || width == 0 {
            return Vec::new();
        }

        // Divide the image vertically into horizontal stripes (tiles) based on requested color count
        let tile_height = height / color_count;
        let mut extracted_colors = Vec::new();

        // 3. Performance Optimization Step
        for i in 0..color_count {
            let start_y = i * tile_height;
            let end_y = if i == color_count- 1 {
                height
            } else {
                start_y + tile_height
            };

            let mut total_r: u64 = 0;
            let mut total_g: u64 = 0;
            let mut total_b: u64 = 0;
            let mut total_a: u64 = 0;
            let mut pixel_count: u64 = 0;

            for y in start_y..end_y {
                for x in 0..width {
                    let pixel = img.get_pixel(x, y);
                    total_r += pixel[0] as u64;
                    total_g += pixel[1] as u64;
                    total_b += pixel[2] as u64;
                    total_a += pixel[3] as u64;
                    pixel_count += 1;
                }
            }

            if pixel_count > 0 {
                extracted_colors.push(RGBAColor {
                    red: (total_r as f64 / pixel_count as f64) / 255.0,
                    green: (total_g as f64 / pixel_count as f64) / 255.0,
                    blue: (total_b as f64 / pixel_count as f64) / 255.0,
                    alpha: (total_a as f64 / pixel_count as f64) / 255.0,
                });
            }
        }

        extracted_colors
    }
}
