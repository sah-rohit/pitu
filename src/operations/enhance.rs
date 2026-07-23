use image::DynamicImage;
use imageproc::filter::gaussian_blur_f32;

/// Enhance image quality: Unsharp mask sharpening, contrast normalization, and clarity boost
pub fn enhance_image(img: &DynamicImage, strength: f32) -> DynamicImage {
    let sigma = 1.2f32;
    let amount = strength.clamp(0.1, 3.0);

    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width(), rgb.height());

    // 1. Unsharp mask (Gaussian blur subtraction)
    let blurred = gaussian_blur_f32(&rgb, sigma);
    let mut sharpened = image::RgbImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let orig_p = rgb.get_pixel(x, y);
            let blur_p = blurred.get_pixel(x, y);

            let r = (orig_p[0] as f32 + amount * (orig_p[0] as f32 - blur_p[0] as f32)).clamp(0.0, 255.0) as u8;
            let g = (orig_p[1] as f32 + amount * (orig_p[1] as f32 - blur_p[1] as f32)).clamp(0.0, 255.0) as u8;
            let b = (orig_p[2] as f32 + amount * (orig_p[2] as f32 - blur_p[2] as f32)).clamp(0.0, 255.0) as u8;

            sharpened.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }

    let dyn_sharpened = DynamicImage::ImageRgb8(sharpened);

    // 2. Contrast normalization & brightness balance
    dyn_sharpened.adjust_contrast(12.0)
}
