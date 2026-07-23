use image::DynamicImage;

pub struct RotateOptions {
    pub degrees: f32,
    pub flip_h: bool,
    pub flip_v: bool,
}

impl Default for RotateOptions {
    fn default() -> Self {
        Self {
            degrees: 0.0,
            flip_h: false,
            flip_v: false,
        }
    }
}

pub fn rotate_image(img: &DynamicImage, opts: &RotateOptions) -> DynamicImage {
    let mut result = img.clone();

    let deg = ((opts.degrees % 360.0) + 360.0) % 360.0;
    if (deg - 90.0).abs() < 1e-3 {
        result = result.rotate90();
    } else if (deg - 180.0).abs() < 1e-3 {
        result = result.rotate180();
    } else if (deg - 270.0).abs() < 1e-3 {
        result = result.rotate270();
    }

    if opts.flip_h {
        result = result.fliph();
    }

    if opts.flip_v {
        result = result.flipv();
    }

    result
}
