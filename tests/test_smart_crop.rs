#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/operations/smart_crop.rs"]
mod smart_crop;

use image::{DynamicImage, Rgb, RgbImage};
use smart_crop::{find_optimal_crop_box, parse_aspect_ratio, smart_crop, SmartCropOptions};

#[test]
fn test_aspect_ratio_parsing() {
    assert_eq!(parse_aspect_ratio("16:9"), Some((16, 9)));
    assert_eq!(parse_aspect_ratio("1:1"), Some((1, 1)));
    assert_eq!(parse_aspect_ratio("4:3"), Some((4, 3)));
    assert_eq!(parse_aspect_ratio("invalid"), None);
}

#[test]
fn test_smart_crop_focal_point() {
    // Create an image: 200x100 white background, with a high-contrast dark square at x=150, y=25
    let mut img_buf = RgbImage::new(200, 100);
    for y in 0..100 {
        for x in 0..200 {
            img_buf.put_pixel(x, y, Rgb([255, 255, 255]));
        }
    }
    // Draw visual interest (dark textured box) at (140, 20)
    for y in 20..80 {
        for x in 140..190 {
            let color = if (x + y) % 2 == 0 { 0 } else { 255 };
            img_buf.put_pixel(x, y, Rgb([color, 0, 0]));
        }
    }

    let dyn_img = DynamicImage::ImageRgb8(img_buf);

    // Smart crop to 50x50 box
    let crop_box = find_optimal_crop_box(&dyn_img, 50, 50, 0.5);

    // The optimal crop box should center around the visual detail on the right side (x >= 100)
    assert!(
        crop_box.x >= 100,
        "Expected crop box to focus on high-entropy region on right side, got x={}",
        crop_box.x
    );
    assert_eq!(crop_box.width, 50);
    assert_eq!(crop_box.height, 50);

    let cropped = smart_crop(
        &dyn_img,
        &SmartCropOptions {
            target_width: Some(50),
            target_height: Some(50),
            ..Default::default()
        },
    );
    assert_eq!(cropped.width(), 50);
    assert_eq!(cropped.height(), 50);
}
