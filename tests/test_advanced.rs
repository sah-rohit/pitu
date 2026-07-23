#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/utils.rs"]
mod utils;
#[path = "../src/batch.rs"]
mod batch;
#[path = "../src/config.rs"]
mod config;
#[path = "../src/operations/mod.rs"]
mod operations;
#[path = "../src/session.rs"]
mod session;
#[path = "../src/versioning.rs"]
mod versioning;

use cli::ImageFormatChoice;
use image::{DynamicImage, RgbImage};
use operations::compress::{compress_to_max_size, parse_size_bytes};
use operations::enhance::enhance_image;
use session::EditSession;

#[test]
fn test_parse_size_bytes() {
    assert_eq!(parse_size_bytes("500KB"), Some(500 * 1024));
    assert_eq!(parse_size_bytes("2MB"), Some(2 * 1024 * 1024));
    assert_eq!(parse_size_bytes("800B"), Some(800));
}

#[test]
fn test_compress_to_max_size() {
    let img_buf = RgbImage::new(400, 400);
    let img = DynamicImage::ImageRgb8(img_buf);

    let target_bytes = 50 * 1024; // 50 KB
    let (compressed, quality) = compress_to_max_size(&img, target_bytes, ImageFormatChoice::Webp).unwrap();

    assert!((compressed.len() as u64) <= target_bytes);
    assert!(quality > 0);
}

#[test]
fn test_edit_session_undo_redo() {
    let img1 = DynamicImage::ImageRgb8(RgbImage::new(100, 100));
    let img2 = DynamicImage::ImageRgb8(RgbImage::new(200, 200));

    let mut session = EditSession::new(img1);
    assert_eq!(session.current_image.width(), 100);

    session.apply_action(img2, "Resized to 200".to_string());
    assert_eq!(session.current_image.width(), 200);
    assert!(session.can_undo());

    assert!(session.undo());
    assert_eq!(session.current_image.width(), 100);
    assert!(session.can_redo());

    assert!(session.redo());
    assert_eq!(session.current_image.width(), 200);
}

#[test]
fn test_enhance_image() {
    let img_buf = RgbImage::new(100, 100);
    let img = DynamicImage::ImageRgb8(img_buf);

    let enhanced = enhance_image(&img, 1.2);
    assert_eq!(enhanced.width(), 100);
    assert_eq!(enhanced.height(), 100);
}
