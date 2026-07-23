use crate::cli::{FilterMode, ResizeFitMode};
use image::{DynamicImage, GenericImageView};

pub struct ResizeOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub percent: Option<f32>,
    pub fit_mode: ResizeFitMode,
    pub filter: FilterMode,
}

impl Default for ResizeOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            percent: None,
            fit_mode: ResizeFitMode::Fit,
            filter: FilterMode::Lanczos3,
        }
    }
}

pub fn resize_image(img: &DynamicImage, opts: &ResizeOptions) -> DynamicImage {
    let (orig_w, orig_h) = img.dimensions();
    if orig_w == 0 || orig_h == 0 {
        return img.clone();
    }

    let filter_type = opts.filter.to_filter_type();

    if let Some(pct) = opts.percent {
        let scale = (pct / 100.0).max(0.001);
        let new_w = ((orig_w as f32) * scale).round() as u32;
        let new_h = ((orig_h as f32) * scale).round() as u32;
        return img.resize_exact(new_w.max(1), new_h.max(1), filter_type);
    }

    let (target_w, target_h) = match (opts.width, opts.height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let aspect = orig_h as f32 / orig_w as f32;
            (w, ((w as f32) * aspect).round() as u32)
        }
        (None, Some(h)) => {
            let aspect = orig_w as f32 / orig_h as f32;
            (((h as f32) * aspect).round() as u32, h)
        }
        (None, None) => return img.clone(),
    };

    let target_w = target_w.max(1);
    let target_h = target_h.max(1);

    match opts.fit_mode {
        ResizeFitMode::Fit => img.resize(target_w, target_h, filter_type),
        ResizeFitMode::Fill => img.resize_to_fill(target_w, target_h, filter_type),
        ResizeFitMode::Exact | ResizeFitMode::Stretch => img.resize_exact(target_w, target_h, filter_type),
    }
}

/// Parse resize command strings like "800x600", "50%", "800x-", "-x600"
pub fn parse_resize_spec(spec: &str) -> Option<ResizeOptions> {
    let spec = spec.trim();
    if spec.ends_with('%') {
        let pct_str = &spec[..spec.len() - 1];
        let pct = pct_str.parse::<f32>().ok()?;
        return Some(ResizeOptions {
            percent: Some(pct),
            ..Default::default()
        });
    }

    if let Some((w_str, h_str)) = spec.split_once('x') {
        let width = if w_str == "-" || w_str.is_empty() {
            None
        } else {
            Some(w_str.parse::<u32>().ok()?)
        };
        let height = if h_str == "-" || h_str.is_empty() {
            None
        } else {
            Some(h_str.parse::<u32>().ok()?)
        };

        if width.is_some() || height.is_some() {
            return Some(ResizeOptions {
                width,
                height,
                ..Default::default()
            });
        }
    }

    None
}
