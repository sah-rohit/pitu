#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/utils.rs"]
mod utils;
#[path = "../src/batch.rs"]
mod batch;
#[path = "../src/operations/mod.rs"]
mod operations;
#[path = "../src/ui/mod.rs"]
mod ui;

use image::{DynamicImage, Rgb, RgbImage};
use ui::ascii_preview::{render_ascii_thumbnail, render_entropy_heatmap_preview};

#[test]
fn test_ascii_preview_renderers() {
    let mut img_buf = RgbImage::new(40, 20);
    for y in 0..20 {
        for x in 0..40 {
            let color = if x > 20 { 255 } else { 0 };
            img_buf.put_pixel(x, y, Rgb([color, 128, 64]));
        }
    }
    let dyn_img = DynamicImage::ImageRgb8(img_buf);

    let ascii_thumb = render_ascii_thumbnail(&dyn_img, 20);
    assert!(ascii_thumb.contains("Terminal Preview"));
    assert!(ascii_thumb.contains("40x20"));

    let heatmap = render_entropy_heatmap_preview(&dyn_img, 20);
    assert!(heatmap.contains("Smart Entropy Energy Map"));
    assert!(heatmap.contains("High Focal Interest"));
}
