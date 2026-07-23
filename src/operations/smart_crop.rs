use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma};
use rayon::prelude::*;

pub struct SmartCropOptions {
    pub target_width: Option<u32>,
    pub target_height: Option<u32>,
    pub aspect_ratio: Option<(u32, u32)>,
    pub entropy_weight: f32, // 0.0 to 1.0 (weight for entropy, 1 - weight for edge gradient)
}

impl Default for SmartCropOptions {
    fn default() -> Self {
        Self {
            target_width: None,
            target_height: None,
            aspect_ratio: None,
            entropy_weight: 0.5,
        }
    }
}

/// Represents a crop window bounding box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Perform content-aware smart cropping using edge detection & Shannon entropy integral images.
pub fn smart_crop(img: &DynamicImage, opts: &SmartCropOptions) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();
    if img_w == 0 || img_h == 0 {
        return img.clone();
    }

    let (crop_w, crop_h) = calculate_crop_dimensions(img_w, img_h, opts);
    if crop_w >= img_w && crop_h >= img_h {
        return img.clone();
    }

    let crop_box = find_optimal_crop_box(img, crop_w, crop_h, opts.entropy_weight);
    img.crop_imm(crop_box.x, crop_box.y, crop_box.width, crop_box.height)
}

/// Computes optimal crop box using integral sum of edge gradient and Shannon entropy energy.
pub fn find_optimal_crop_box(
    img: &DynamicImage,
    crop_w: u32,
    crop_h: u32,
    entropy_weight: f32,
) -> CropBox {
    let (w, h) = img.dimensions();
    let crop_w = crop_w.min(w);
    let crop_h = crop_h.min(h);

    if crop_w == w && crop_h == h {
        return CropBox {
            x: 0,
            y: 0,
            width: w,
            height: h,
        };
    }

    let gray = img.to_luma8();

    // 1. Calculate Sobel Edge Magnitude map
    let edge_map = compute_sobel_gradient(&gray);

    // 2. Calculate Local Shannon Entropy map
    let entropy_map = compute_local_entropy(&gray, 3); // 7x7 local neighborhood

    // 3. Normalize & combine edge and entropy maps into energy map
    let energy_map = combine_energy_maps(&edge_map, &entropy_map, entropy_weight, w, h);

    // 4. Compute 2D Integral Image (Summed-Area Table)
    let integral_image = compute_integral_image(&energy_map, w, h);

    // 5. Search for bounding box maximizing total energy
    let step = if w > 1000 || h > 1000 { 4 } else { 1 };
    let mut max_energy = -1.0f64;
    let mut best_x = 0;
    let mut best_y = 0;

    let max_x = w.saturating_sub(crop_w);
    let max_y = h.saturating_sub(crop_h);

    for y in (0..=max_y).step_by(step as usize) {
        for x in (0..=max_x).step_by(step as usize) {
            let x2 = x + crop_w - 1;
            let y2 = y + crop_h - 1;

            let energy = query_integral_rect(&integral_image, w as usize, x as usize, y as usize, x2 as usize, y2 as usize);
            if energy > max_energy {
                max_energy = energy;
                best_x = x;
                best_y = y;
            }
        }
    }

    CropBox {
        x: best_x,
        y: best_y,
        width: crop_w,
        height: crop_h,
    }
}

/// Calculate target crop dimensions based on options and image aspect ratio.
fn calculate_crop_dimensions(img_w: u32, img_h: u32, opts: &SmartCropOptions) -> (u32, u32) {
    if let (Some(w), Some(h)) = (opts.target_width, opts.target_height) {
        return (w.min(img_w), h.min(img_h));
    }

    let aspect_ratio = if let Some((rw, rh)) = opts.aspect_ratio {
        rw as f64 / rh as f64
    } else if let Some(w) = opts.target_width {
        let cur_aspect = img_w as f64 / img_h as f64;
        return (w.min(img_w), (w as f64 / cur_aspect).round() as u32);
    } else if let Some(h) = opts.target_height {
        let cur_aspect = img_w as f64 / img_h as f64;
        return ((h as f64 * cur_aspect).round() as u32, h.min(img_h));
    } else {
        return (img_w, img_h);
    };

    let img_aspect = img_w as f64 / img_h as f64;
    if aspect_ratio > img_aspect {
        // Fit width, crop height
        let crop_w = img_w;
        let crop_h = (img_w as f64 / aspect_ratio).round() as u32;
        (crop_w, crop_h.min(img_h))
    } else {
        // Fit height, crop width
        let crop_h = img_h;
        let crop_w = (img_h as f64 * aspect_ratio).round() as u32;
        (crop_w.min(img_w), crop_h)
    }
}

/// Compute Sobel edge magnitude
fn compute_sobel_gradient(gray: &GrayImage) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    let (w, h) = gray.dimensions();
    let mut out = ImageBuffer::new(w, h);

    if w < 3 || h < 3 {
        return out;
    }

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let p00 = gray.get_pixel(x - 1, y - 1)[0] as f32;
            let p01 = gray.get_pixel(x, y - 1)[0] as f32;
            let p02 = gray.get_pixel(x + 1, y - 1)[0] as f32;
            let p10 = gray.get_pixel(x - 1, y)[0] as f32;
            let p12 = gray.get_pixel(x + 1, y)[0] as f32;
            let p20 = gray.get_pixel(x - 1, y + 1)[0] as f32;
            let p21 = gray.get_pixel(x, y + 1)[0] as f32;
            let p22 = gray.get_pixel(x + 1, y + 1)[0] as f32;

            let gx = (-1.0 * p00) + (1.0 * p02) + (-2.0 * p10) + (2.0 * p12) + (-1.0 * p20) + (1.0 * p22);
            let gy = (-1.0 * p00) + (-2.0 * p01) + (-1.0 * p02) + (1.0 * p20) + (2.0 * p21) + (1.0 * p22);

            let magnitude = (gx * gx + gy * gy).sqrt();
            out.put_pixel(x, y, Luma([magnitude]));
        }
    }

    out
}

/// Compute local Shannon Entropy in local (2*radius + 1) neighborhood window using Rayon multi-threading
fn compute_local_entropy(gray: &GrayImage, radius: i32) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    let (w, h) = gray.dimensions();
    let mut out_vec = vec![0.0f32; (w * h) as usize];

    out_vec.par_chunks_mut(w as usize).enumerate().for_each(|(y_idx, row)| {
        let y = y_idx as u32;
        let mut hist = [0u32; 256];

        for x in 0..w {
            hist.fill(0);
            let mut total_count = 0u32;

            for dy in -radius..=radius {
                let ny = y as i32 + dy;
                if ny >= 0 && ny < h as i32 {
                    for dx in -radius..=radius {
                        let nx = x as i32 + dx;
                        if nx >= 0 && nx < w as i32 {
                            let val = gray.get_pixel(nx as u32, ny as u32)[0] as usize;
                            hist[val] += 1;
                            total_count += 1;
                        }
                    }
                }
            }

            if total_count == 0 {
                continue;
            }

            let mut entropy = 0.0f32;
            let total_f = total_count as f32;
            for &c in hist.iter() {
                if c > 0 {
                    let p = c as f32 / total_f;
                    entropy -= p * p.log2();
                }
            }

            row[x as usize] = entropy;
        }
    });

    ImageBuffer::from_raw(w, h, out_vec).unwrap()
}

/// Combine edge magnitude and entropy into a single normalized energy map
fn combine_energy_maps(
    edge_map: &ImageBuffer<Luma<f32>, Vec<f32>>,
    entropy_map: &ImageBuffer<Luma<f32>, Vec<f32>>,
    entropy_weight: f32,
    w: u32,
    h: u32,
) -> Vec<f64> {
    let mut max_edge = 1e-5f32;
    let mut max_entropy = 1e-5f32;

    for y in 0..h {
        for x in 0..w {
            let e = edge_map.get_pixel(x, y)[0];
            let h_val = entropy_map.get_pixel(x, y)[0];
            if e > max_edge {
                max_edge = e;
            }
            if h_val > max_entropy {
                max_entropy = h_val;
            }
        }
    }

    let edge_w = (1.0 - entropy_weight.clamp(0.0, 1.0)) as f64;
    let ent_w = entropy_weight.clamp(0.0, 1.0) as f64;

    let mut energy = vec![0.0f64; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let norm_edge = (edge_map.get_pixel(x, y)[0] / max_edge) as f64;
            let norm_ent = (entropy_map.get_pixel(x, y)[0] / max_entropy) as f64;

            energy[idx] = edge_w * norm_edge + ent_w * norm_ent;
        }
    }

    energy
}

/// Computes 2D Integral Image (Summed Area Table) for fast rectangular sum queries
fn compute_integral_image(energy: &[f64], w: u32, h: u32) -> Vec<f64> {
    let mut integral = vec![0.0f64; (w * h) as usize];
    let w_usize = w as usize;

    for y in 0..h as usize {
        let mut row_sum = 0.0f64;
        for x in 0..w as usize {
            let idx = y * w_usize + x;
            row_sum += energy[idx];
            let prev_above = if y > 0 { integral[(y - 1) * w_usize + x] } else { 0.0 };
            integral[idx] = row_sum + prev_above;
        }
    }

    integral
}

/// O(1) query for rectangular sum [x1, y1] to [x2, y2] using Integral Image
fn query_integral_rect(integral: &[f64], width: usize, x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
    let total = integral[y2 * width + x2];
    let top = if y1 > 0 { integral[(y1 - 1) * width + x2] } else { 0.0 };
    let left = if x1 > 0 { integral[y2 * width + (x1 - 1)] } else { 0.0 };
    let top_left = if x1 > 0 && y1 > 0 { integral[(y1 - 1) * width + (x1 - 1)] } else { 0.0 };

    total - top - left + top_left
}

/// Helper to parse aspect ratio string like "16:9" or "1:1"
pub fn parse_aspect_ratio(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 2 {
        let w = parts[0].trim().parse::<u32>().ok()?;
        let h = parts[1].trim().parse::<u32>().ok()?;
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }
    None
}
