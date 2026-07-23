use image::{DynamicImage, ImageBuffer, Rgba};

pub struct FilterOptions {
    pub grayscale: bool,
    pub sepia: bool,
    pub invert: bool,
    pub brightness: Option<i32>,
    pub contrast: Option<f32>,
    pub blur: Option<f32>,
    pub sharpen: Option<f32>,
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
