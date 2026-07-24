use image::{DynamicImage, ImageBuffer, Rgba};

pub struct FilterOptions {
    pub grayscale: bool,
    pub sepia: bool,
    pub invert: bool,
    pub brightness: Option<i32>,
    pub contrast: Option<f32>,
    pub blur: Option<f32>,
    pub sharpen: Option<f32>,
    pub warmth: Option<f32>,
    pub vignette: Option<f32>,
    pub structure: Option<f32>,
    pub hdr_scape: bool,
    pub glamour_glow: bool,
    pub haze_removal: bool,
    pub frame_width: Option<u32>,
    // Extended Snapseed / Photoshop parameters
    pub exposure: Option<f32>,
    pub saturation: Option<f32>,
    pub shadows: Option<f32>,
    pub highlights: Option<f32>,
    pub noir: bool,
    pub vintage: bool,
    pub grunge: bool,
    pub lens_blur: Option<f32>,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            grayscale: false,
            sepia: false,
            invert: false,
            brightness: None,
            contrast: None,
            blur: None,
            sharpen: None,
            warmth: None,
            vignette: None,
            structure: None,
            hdr_scape: false,
            glamour_glow: false,
            haze_removal: false,
            frame_width: None,
            exposure: None,
            saturation: None,
            shadows: None,
            highlights: None,
            noir: false,
            vintage: false,
            grunge: false,
            lens_blur: None,
        }
    }
}

pub fn apply_filters(img: &DynamicImage, opts: &FilterOptions) -> DynamicImage {
    let mut result = img.clone();

    if opts.grayscale {
        result = DynamicImage::ImageLuma8(result.to_luma8()).into();
    }

    if opts.sepia {
        result = apply_sepia(&result);
    }

    if opts.invert {
        result.invert();
    }

    if let Some(b) = opts.brightness {
        result = result.brighten(b);
    }

    if let Some(c) = opts.contrast {
        result = result.adjust_contrast(c);
    }

    if let Some(sigma) = opts.blur {
        if sigma > 0.0 {
            result = result.blur(sigma);
        }
    }

    if let Some(s) = opts.sharpen {
        if s > 0.0 {
            result = result.unsharpen(s, 1);
        }
    }

    if let Some(w) = opts.warmth {
        if w.abs() > 0.05 {
            result = apply_warmth(&result, w);
        }
    }

    if let Some(v) = opts.vignette {
        if v > 0.05 {
            result = apply_vignette(&result, v);
        }
    }

    if let Some(st) = opts.structure {
        if st > 0.05 {
            result = apply_structure(&result, st);
        }
    }

    if opts.hdr_scape {
        result = apply_hdr_scape(&result);
    }

    if opts.glamour_glow {
        result = apply_glamour_glow(&result);
    }

    if opts.haze_removal {
        result = apply_haze_removal(&result);
    }

    if let Some(exp) = opts.exposure {
        if exp.abs() > 0.05 {
            result = apply_exposure(&result, exp);
        }
    }

    if let Some(sat) = opts.saturation {
        if (sat - 1.0).abs() > 0.05 {
            result = apply_saturation(&result, sat);
        }
    }

    if let Some(sh) = opts.shadows {
        if sh.abs() > 0.05 {
            result = apply_shadows(&result, sh);
        }
    }

    if let Some(hi) = opts.highlights {
        if hi.abs() > 0.05 {
            result = apply_highlights(&result, hi);
        }
    }

    if opts.noir {
        result = apply_noir(&result);
    }

    if opts.vintage {
        result = apply_vintage(&result);
    }

    if opts.grunge {
        result = apply_grunge(&result);
    }

    if let Some(lb) = opts.lens_blur {
        if lb > 0.05 {
            result = apply_lens_blur(&result, lb);
        }
    }

    if let Some(border) = opts.frame_width {
        if border > 0 {
            result = apply_frame(&result, border);
        }
    }

    result
}

fn apply_sepia(img: &DynamicImage) -> DynamicImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = ImageBuffer::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let p = rgba.get_pixel(x, y);
            let r = p[0] as f32;
            let g = p[1] as f32;
            let b = p[2] as f32;

            let sr = (r * 0.393 + g * 0.769 + b * 0.189).min(255.0) as u8;
            let sg = (r * 0.349 + g * 0.686 + b * 0.168).min(255.0) as u8;
            let sb = (r * 0.272 + g * 0.534 + b * 0.131).min(255.0) as u8;

            out.put_pixel(x, y, Rgba([sr, sg, sb, p[3]]));
        }
    }

    DynamicImage::ImageRgba8(out)
}

fn apply_warmth(img: &DynamicImage, warmth: f32) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let shift_r = warmth * 20.0;
    let shift_b = -warmth * 20.0;

    for p in rgba.pixels_mut() {
        p[0] = (p[0] as f32 + shift_r).clamp(0.0, 255.0) as u8;
        p[2] = (p[2] as f32 + shift_b).clamp(0.0, 255.0) as u8;
    }
    DynamicImage::ImageRgba8(rgba)
}

fn apply_vignette(img: &DynamicImage, strength: f32) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let (w, h) = (rgba.width() as f32, rgba.height() as f32);
    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    for (x, y, p) in rgba.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let dist = (dx * dx + dy * dy).sqrt() / max_dist;
        let factor = (1.0 - (dist * strength).powf(1.8)).clamp(0.0, 1.0);

        p[0] = (p[0] as f32 * factor) as u8;
        p[1] = (p[1] as f32 * factor) as u8;
        p[2] = (p[2] as f32 * factor) as u8;
    }
    DynamicImage::ImageRgba8(rgba)
}

fn apply_structure(img: &DynamicImage, strength: f32) -> DynamicImage {
    img.unsharpen(strength * 2.0, 1)
}

/// HDR Scape: boost local contrast in shadows/midtones/highlights
fn apply_hdr_scape(img: &DynamicImage) -> DynamicImage {
    let blurred = img.blur(8.0);
    let rgba_orig = img.to_rgba8();
    let rgba_blur = blurred.to_rgba8();
    let (w, h) = rgba_orig.dimensions();
    let mut out = ImageBuffer::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let po = rgba_orig.get_pixel(x, y);
            let pb = rgba_blur.get_pixel(x, y);
            let mut channels = [0u8; 4];
            for c in 0..3 {
                let orig = po[c] as f32;
                let blur = pb[c] as f32;
                let detail = orig - blur;
                channels[c] = (orig + detail * 0.6).clamp(0.0, 255.0) as u8;
            }
            channels[3] = po[3];
            out.put_pixel(x, y, Rgba(channels));
        }
    }
    DynamicImage::ImageRgba8(out)
}

/// Glamour Glow: soft diffused highlight glow for portraits
fn apply_glamour_glow(img: &DynamicImage) -> DynamicImage {
    let blurred = img.blur(6.0).brighten(15);
    let rgba_orig = img.to_rgba8();
    let rgba_glow = blurred.to_rgba8();
    let (w, h) = rgba_orig.dimensions();
    let mut out = ImageBuffer::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let po = rgba_orig.get_pixel(x, y);
            let pg = rgba_glow.get_pixel(x, y);
            let mut channels = [0u8; 4];
            for c in 0..3 {
                let base = po[c] as f32 / 255.0;
                let glow = pg[c] as f32 / 255.0;
                // Screen blend mode
                let blended = 1.0 - (1.0 - base) * (1.0 - glow * 0.4);
                channels[c] = (blended * 255.0).clamp(0.0, 255.0) as u8;
            }
            channels[3] = po[3];
            out.put_pixel(x, y, Rgba(channels));
        }
    }
    DynamicImage::ImageRgba8(out)
}

/// Haze Removal: increase local contrast and reduce atmospheric haze
fn apply_haze_removal(img: &DynamicImage) -> DynamicImage {
    let result = img.adjust_contrast(20.0);
    let rgba = result.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = ImageBuffer::new(w, h);

    // Simple dark channel prior approximation
    for y in 0..h {
        for x in 0..w {
            let p = rgba.get_pixel(x, y);
            let min_ch = p[0].min(p[1]).min(p[2]) as f32;
            let haze_factor = (min_ch / 255.0) * 0.3;
            let mut channels = [0u8; 4];
            for c in 0..3 {
                let val = p[c] as f32;
                channels[c] = ((val - haze_factor * 60.0) / (1.0 - haze_factor * 0.3)).clamp(0.0, 255.0) as u8;
            }
            channels[3] = p[3];
            out.put_pixel(x, y, Rgba(channels));
        }
    }
    DynamicImage::ImageRgba8(out)
}

/// Add a solid-color border frame around the image
fn apply_frame(img: &DynamicImage, border: u32) -> DynamicImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let new_w = w + border * 2;
    let new_h = h + border * 2;
    let mut out = ImageBuffer::from_pixel(new_w, new_h, Rgba([20, 20, 20, 255]));

    for y in 0..h {
        for x in 0..w {
            out.put_pixel(x + border, y + border, *rgba.get_pixel(x, y));
        }
    }
    DynamicImage::ImageRgba8(out)
}

fn apply_exposure(img: &DynamicImage, exp: f32) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let multiplier = 2.0f32.powf(exp);
    for p in rgba.pixels_mut() {
        p[0] = (p[0] as f32 * multiplier).clamp(0.0, 255.0) as u8;
        p[1] = (p[1] as f32 * multiplier).clamp(0.0, 255.0) as u8;
        p[2] = (p[2] as f32 * multiplier).clamp(0.0, 255.0) as u8;
    }
    DynamicImage::ImageRgba8(rgba)
}

fn apply_saturation(img: &DynamicImage, sat: f32) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    for p in rgba.pixels_mut() {
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;
        let gray = 0.299 * r + 0.587 * g + 0.114 * b;
        p[0] = (gray + (r - gray) * sat).clamp(0.0, 255.0) as u8;
        p[1] = (gray + (g - gray) * sat).clamp(0.0, 255.0) as u8;
        p[2] = (gray + (b - gray) * sat).clamp(0.0, 255.0) as u8;
    }
    DynamicImage::ImageRgba8(rgba)
}

fn apply_shadows(img: &DynamicImage, sh: f32) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    for p in rgba.pixels_mut() {
        for c in 0..3 {
            let val = p[c] as f32;
            let factor = (1.0 - val / 255.0).clamp(0.0, 1.0);
            p[c] = (val + sh * 50.0 * factor).clamp(0.0, 255.0) as u8;
        }
    }
    DynamicImage::ImageRgba8(rgba)
}

fn apply_highlights(img: &DynamicImage, hi: f32) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    for p in rgba.pixels_mut() {
        for c in 0..3 {
            let val = p[c] as f32;
            let factor = (val / 255.0).clamp(0.0, 1.0);
            p[c] = (val + hi * 50.0 * factor).clamp(0.0, 255.0) as u8;
        }
    }
    DynamicImage::ImageRgba8(rgba)
}

fn apply_noir(img: &DynamicImage) -> DynamicImage {
    let mut result = DynamicImage::ImageLuma8(img.to_luma8());
    result = result.adjust_contrast(30.0);
    apply_vignette(&result, 0.75)
}

fn apply_vintage(img: &DynamicImage) -> DynamicImage {
    let tinted = apply_warmth(img, 0.35);
    let desaturated = apply_saturation(&tinted, 0.6);
    apply_vignette(&desaturated, 0.4)
}

fn apply_grunge(img: &DynamicImage) -> DynamicImage {
    let mut result = apply_saturation(img, 0.4);
    result = result.adjust_contrast(35.0);
    result = apply_vignette(&result, 0.8);
    let mut rgba = result.to_rgba8();
    let mut rng_state = 12345u32;
    for p in rgba.pixels_mut() {
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((rng_state % 31) as f32 - 15.0) * 0.8;
        p[0] = (p[0] as f32 + noise).clamp(0.0, 255.0) as u8;
        p[1] = (p[1] as f32 + noise).clamp(0.0, 255.0) as u8;
        p[2] = (p[2] as f32 + noise).clamp(0.0, 255.0) as u8;
    }
    DynamicImage::ImageRgba8(rgba)
}

fn apply_lens_blur(img: &DynamicImage, strength: f32) -> DynamicImage {
    img.blur(strength * 3.5)
}
