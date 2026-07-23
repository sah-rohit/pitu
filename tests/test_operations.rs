#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/operations/convert.rs"]
mod convert;
#[path = "../src/operations/crop.rs"]
mod crop;
#[path = "../src/operations/filter.rs"]
mod filter;
#[path = "../src/operations/resize.rs"]
mod resize;
#[path = "../src/operations/rotate.rs"]
mod rotate;

use crop::{crop_image, parse_crop_spec};
use image::{DynamicImage, RgbImage};
use resize::{parse_resize_spec, resize_image, ResizeOptions};
use rotate::{rotate_image, RotateOptions};

#[test]
fn test_resize_spec_parsing() {
    let spec1 = parse_resize_spec("800x600").unwrap();
    assert_eq!(spec1.width, Some(800));
    assert_eq!(spec1.height, Some(600));

    let spec2 = parse_resize_spec("50%").unwrap();
    assert_eq!(spec2.percent, Some(50.0));

    let spec3 = parse_resize_spec("800x-").unwrap();
    assert_eq!(spec3.width, Some(800));
    assert_eq!(spec3.height, None);
}

#[test]
fn test_image_resize_and_crop() {
    let img_buf = RgbImage::new(100, 100);
    let dyn_img = DynamicImage::ImageRgb8(img_buf);

    let resized = resize_image(
        &dyn_img,
        &ResizeOptions {
            width: Some(50),
            height: Some(50),
            ..Default::default()
        },
    );
    assert_eq!(resized.width(), 50);
    assert_eq!(resized.height(), 50);

    let crop_spec = parse_crop_spec("10,10,30,40").unwrap();
    let cropped = crop_image(&dyn_img, &crop_spec);
    assert_eq!(cropped.width(), 30);
    assert_eq!(cropped.height(), 40);
}

#[test]
fn test_rotate_and_flip() {
    let img_buf = RgbImage::new(200, 100);
    let dyn_img = DynamicImage::ImageRgb8(img_buf);

    let rotated = rotate_image(
        &dyn_img,
        &RotateOptions {
            degrees: 90.0,
            flip_h: false,
            flip_v: false,
        },
    );
    assert_eq!(rotated.width(), 100);
    assert_eq!(rotated.height(), 200);
}
