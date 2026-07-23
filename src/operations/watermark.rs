use crate::cli::AnchorPosition;
use ab_glyph::{FontRef, PxScale};
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use std::path::Path;

pub struct WatermarkOptions {
    pub text: Option<String>,
    pub image_path: Option<std::path::PathBuf>,
    pub anchor: AnchorPosition,
    pub opacity: f32,
    pub scale: f32,
}

impl Default for WatermarkOptions {
    fn default() -> Self {
        Self {
            text: None,
            image_path: None,
            anchor: AnchorPosition::BottomRight,
            opacity: 0.8,
            scale: 0.2,
        }
    }
}

/// Apply image or text watermark to a dynamic image
pub fn apply_watermark(img: &DynamicImage, opts: &WatermarkOptions) -> anyhow::Result<DynamicImage> {
    let mut base = img.to_rgba8();

    if let Some(ref image_path) = opts.image_path {
        base = overlay_image_watermark(&base, image_path, opts.anchor, opts.opacity, opts.scale)?;
    }

    if let Some(ref text) = opts.text {
        base = overlay_text_watermark(&base, text, opts.anchor, opts.opacity, opts.scale)?;
    }

    Ok(DynamicImage::ImageRgba8(base))
}

fn overlay_image_watermark(
    base: &RgbaImage,
    wm_path: &Path,
    anchor: AnchorPosition,
    opacity: f32,
    scale: f32,
) -> anyhow::Result<RgbaImage> {
    let wm_img = image::open(wm_path)?;
    let (bw, bh) = base.dimensions();
    let (ww_orig, wh_orig) = wm_img.dimensions();

    if bw == 0 || bh == 0 || ww_orig == 0 || wh_orig == 0 {
        return Ok(base.clone());
    }

    // Scale watermark relative to base image width
    let target_ww = ((bw as f32) * scale.clamp(0.01, 1.0)).round() as u32;
    let aspect = wh_orig as f32 / ww_orig as f32;
    let target_wh = ((target_ww as f32) * aspect).round() as u32;

    let target_ww = target_ww.max(1).min(bw);
    let target_wh = target_wh.max(1).min(bh);

    let resized_wm = wm_img.resize_exact(target_ww, target_wh, image::imageops::FilterType::Lanczos3);
    let wm_rgba = resized_wm.to_rgba8();

    let (x, y) = compute_anchor_offset(bw, bh, target_ww, target_wh, anchor, 20);

    let mut result = base.clone();
    let opacity_clamped = opacity.clamp(0.0, 1.0);

    for wy in 0..target_wh {
        for wx in 0..target_ww {
            let px = x + wx;
            let py = y + wy;
            if px < bw && py < bh {
                let wm_pixel = wm_rgba.get_pixel(wx, wy);
                let base_pixel = result.get_pixel(px, py);

                let alpha = (wm_pixel[3] as f32 / 255.0) * opacity_clamped;
                if alpha > 0.0 {
                    let blended = blend_rgba(*base_pixel, *wm_pixel, alpha);
                    result.put_pixel(px, py, blended);
                }
            }
        }
    }

    Ok(result)
}

fn overlay_text_watermark(
    base: &RgbaImage,
    text: &str,
    anchor: AnchorPosition,
    opacity: f32,
    scale: f32,
) -> anyhow::Result<RgbaImage> {
    let (bw, bh) = base.dimensions();
    if bw == 0 || bh == 0 || text.is_empty() {
        return Ok(base.clone());
    }

    // Embed default font data or fallback font
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let font = FontRef::try_from_slice(font_data as &[u8])
        .map_err(|e| anyhow::anyhow!("Failed to load embedded font: {}", e))?;

    let font_size = ((bh as f32) * (0.05 * scale * 5.0)).clamp(12.0, 120.0);
    let scale_px = PxScale::from(font_size);

    // Approximate text dimensions
    let char_count = text.chars().count() as u32;
    let approx_width = (char_count as f32 * font_size * 0.55) as u32;
    let approx_height = (font_size * 1.2) as u32;

    let (x, y) = compute_anchor_offset(bw, bh, approx_width, approx_height, anchor, 20);

    let mut result = base.clone();
    let alpha = (255.0 * opacity.clamp(0.0, 1.0)) as u8;
    let color = Rgba([255, 255, 255, alpha]);
    let shadow_color = Rgba([0, 0, 0, (alpha as f32 * 0.6) as u8]);

    // Draw drop shadow for contrast
    draw_text_mut(&mut result, shadow_color, x as i32 + 2, y as i32 + 2, scale_px, &font, text);
    draw_text_mut(&mut result, color, x as i32, y as i32, scale_px, &font, text);

    Ok(result)
}

fn compute_anchor_offset(
    bw: u32,
    bh: u32,
    ww: u32,
    wh: u32,
    anchor: AnchorPosition,
    padding: u32,
) -> (u32, u32) {
    let padding_x = padding.min(bw.saturating_sub(ww) / 2);
    let padding_y = padding.min(bh.saturating_sub(wh) / 2);

    match anchor {
        AnchorPosition::TopLeft => (padding_x, padding_y),
        AnchorPosition::TopCenter => (bw.saturating_sub(ww) / 2, padding_y),
        AnchorPosition::TopRight => (bw.saturating_sub(ww).saturating_sub(padding_x), padding_y),
        AnchorPosition::CenterLeft => (padding_x, bh.saturating_sub(wh) / 2),
        AnchorPosition::Center => (bw.saturating_sub(ww) / 2, bh.saturating_sub(wh) / 2),
        AnchorPosition::CenterRight => (bw.saturating_sub(ww).saturating_sub(padding_x), bh.saturating_sub(wh) / 2),
        AnchorPosition::BottomLeft => (padding_x, bh.saturating_sub(wh).saturating_sub(padding_y)),
        AnchorPosition::BottomCenter => (bw.saturating_sub(ww) / 2, bh.saturating_sub(wh).saturating_sub(padding_y)),
        AnchorPosition::BottomRight => (
            bw.saturating_sub(ww).saturating_sub(padding_x),
            bh.saturating_sub(wh).saturating_sub(padding_y),
        ),
    }
}

fn blend_rgba(base: Rgba<u8>, overlay: Rgba<u8>, alpha: f32) -> Rgba<u8> {
    let a = alpha.clamp(0.0, 1.0);
    let inv_a = 1.0 - a;

    let r = (base[0] as f32 * inv_a + overlay[0] as f32 * a).round() as u8;
    let g = (base[1] as f32 * inv_a + overlay[1] as f32 * a).round() as u8;
    let b = (base[2] as f32 * inv_a + overlay[2] as f32 * a).round() as u8;
    let final_a = (base[3] as f32).max(overlay[3] as f32 * a).round() as u8;

    Rgba([r, g, b, final_a])
}
