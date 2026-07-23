use image::{DynamicImage, GenericImageView};

pub struct CropOptions {
    pub x: u32,
    pub y: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub aspect_ratio: Option<(u32, u32)>,
}

impl Default for CropOptions {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: None,
            height: None,
            aspect_ratio: None,
        }
    }
}

pub fn crop_image(img: &DynamicImage, opts: &CropOptions) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();
    if img_w == 0 || img_h == 0 {
        return img.clone();
    }

    if let Some((rw, rh)) = opts.aspect_ratio {
        let aspect_ratio = rw as f64 / rh as f64;
        let img_aspect = img_w as f64 / img_h as f64;

        let (target_w, target_h) = if aspect_ratio > img_aspect {
            (img_w, (img_w as f64 / aspect_ratio).round() as u32)
        } else {
            ((img_h as f64 * aspect_ratio).round() as u32, img_h)
        };

        let target_w = target_w.min(img_w);
        let target_h = target_h.min(img_h);

        let center_x = img_w.saturating_sub(target_w) / 2;
        let center_y = img_h.saturating_sub(target_h) / 2;

        return img.crop_imm(center_x, center_y, target_w, target_h);
    }

    let crop_w = opts.width.unwrap_or(img_w).min(img_w.saturating_sub(opts.x));
    let crop_h = opts.height.unwrap_or(img_h).min(img_h.saturating_sub(opts.y));

    if crop_w == 0 || crop_h == 0 {
        return img.clone();
    }

    img.crop_imm(opts.x, opts.y, crop_w, crop_h)
}

/// Parse manual crop spec string like "10,20,400,300"
pub fn parse_crop_spec(spec: &str) -> Option<CropOptions> {
    let parts: Vec<&str> = spec.split(',').collect();
    if parts.len() == 4 {
        let x = parts[0].trim().parse::<u32>().ok()?;
        let y = parts[1].trim().parse::<u32>().ok()?;
        let width = parts[2].trim().parse::<u32>().ok()?;
        let height = parts[3].trim().parse::<u32>().ok()?;
        return Some(CropOptions {
            x,
            y,
            width: Some(width),
            height: Some(height),
            aspect_ratio: None,
        });
    }
    None
}
